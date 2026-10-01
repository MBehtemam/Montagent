//! The MCP resources: the published schema, the format docs, and the schema index with the
//! pieces it lists (ADR-0137).
//!
//! ADR-0011 makes these resources rather than tools for a cost reason, not a taste one:
//! *"every MCP tool schema occupies the agent's context and degrades tool selection on
//! every turn, including turns with nothing to do with video"*, while a resource costs no
//! tool slot at all. They are what makes *"how does the agent know how to edit
//! `project.json`"* answerable the same way `package.json` is — you read the schema.
//!
//! They live here rather than in the MCP adapter because the adapter is thin **by
//! construction** (ADR-0011): it owns the protocol, and nothing else. What the resources
//! are, what they are called and what bytes they carry is the library's.
//!
//! **The schema resource is generated, never a copy.** [`crate::schema::generated_bytes`]
//! derives it from [`crate::model::Project`] on the spot, so the artifact an agent reads is
//! the artifact that parses its file — the two-artifact drift this project has already
//! found three times has nowhere to happen. The committed
//! `schema/montagent.schema.json` exists for readers outside a running server and is checked
//! against the same function by `tests/schema.rs`; nothing serves it.
//!
//! The format docs are `include_str!`d from within this crate rather than read from
//! `docs/` at the repository root, because a released binary has no repository beside it
//! (ADR-0064 ships `cargo install` and six target binaries) and a resource that resolved to
//! a missing file would be a resource that works only in a checkout. Ratified by ADR-0080,
//! which also names the guard against the cost it admits — the repo's prose living in two
//! places: `every_adr_the_format_docs_cite_exists_and_is_still_accepted`.
//!
//! **The schema is also served in pieces** (ADR-0137), because whole it is larger than a
//! client's tool result. [`INDEX`] is listed beside the other two; the pieces it names are
//! not listed, and [`read`] serves them from [`crate::schema_index`]. The three listed URIs
//! are written once, in the macros below, so the routing text that names them — the server
//! instructions and the descriptions here — cannot name a URI nothing serves.

macro_rules! schema_uri {
    () => {
        "montagent://schema.json"
    };
}

macro_rules! format_uri {
    () => {
        "montagent://format.md"
    };
}

macro_rules! index_uri {
    () => {
        "montagent://schema/index.json"
    };
}

/// What the MCP server tells an agent on connecting: where to start learning the format.
///
/// It starts the agent at the index rather than at `schema.json` (ADR-0137 §7): agents read
/// the URIs they are told about, so the pointer decides whether the pieces are used at all.
pub const SERVER_INSTRUCTIONS: &str = concat!(
    "Montagent reads, checks and renders a declarative video project. Edit the project file \
     with your ordinary file tools — there is no CRUD API — and call these tools for the \
     things a text editor cannot do. To learn the format, read `",
    format_uri!(),
    "` (the rules the schema cannot express) and start the shape at `",
    index_uri!(),
    "`, which lists every element type and effect with its required keys and the URI of \
     the schema piece that describes it.",
);

/// One published resource.
///
/// ADR-0011 names *"the schema"* and *"the format docs"* and settles neither their URIs nor
/// their names. **ADR-0080 does**, and ADR-0137 adds the index: the URIs, names and media
/// types below are ratified as a published surface an agent may cite, and they do not move.
/// `tests/resources.rs` pins all nine strings, so a rename is a failing test rather than a
/// silent break.
// No `PartialEq`: the derive would compare `body`, and comparing function pointers is
// unpredictable. Resources are identified by `uri`, which is what `find` compares.
#[derive(Debug, Clone, Copy)]
pub struct Resource {
    /// The stable URI an agent reads it by.
    pub uri: &'static str,
    /// The programmatic name.
    pub name: &'static str,
    /// The human-readable title.
    pub title: &'static str,
    /// What it is for — the sentence that decides whether an agent opens it.
    pub description: &'static str,
    pub mime_type: &'static str,
    /// Where the bytes come from.
    ///
    /// A function on the declaration rather than a second `match` keyed on `uri`, so
    /// listing a resource and serving one cannot disagree about what is published — the
    /// failure mode being a URI that lists and then reads as nothing.
    body: fn() -> String,
}

impl Resource {
    /// This resource's bytes, generated or embedded as its own declaration says.
    pub fn body(&self) -> String {
        (self.body)()
    }
}

/// The generated JSON Schema: the authority on the format's *shape*.
pub const SCHEMA: Resource = Resource {
    uri: schema_uri!(),
    name: "montagent-schema",
    title: "Montagent project JSON Schema",
    description: concat!(
        "The whole published JSON Schema for a Montagent project file — every element \
         type, every field, every admitted value. Generated from the same types that parse \
         your project, so it is the schema `validate` enforces rather than a description of \
         it. Canonical key order is this schema's property order. It is too large to read \
         in one go in most clients: its pieces are listed in `",
        index_uri!(),
        "`."
    ),
    mime_type: "application/schema+json",
    body: crate::schema::generated_bytes,
};

/// The prose: the authority on everything the shape cannot say.
pub const FORMAT: Resource = Resource {
    uri: format_uri!(),
    name: "montagent-format",
    title: "The Montagent project format",
    description: "The rules a JSON Schema cannot express: the canonical writing convention \
                  an exact-string edit depends on, half-open time ranges, what a track \
                  does and does not supply, how a layer anchor resolves, why presence is \
                  content, and how sources resolve. Read this alongside the schema before \
                  editing a project by hand.",
    mime_type: "text/markdown",
    body: format_docs,
};

/// The schema index: where an agent starts learning the format's shape (ADR-0137).
pub const INDEX: Resource = Resource {
    uri: index_uri!(),
    name: "montagent-schema-index",
    title: "Montagent project schema index",
    description: "Start here for the format's shape. Lists every element type and every \
                  effect with its required keys, and the URI of the schema piece for each \
                  one and for every other definition. Each piece fits one read and carries \
                  the rules for its keys.",
    mime_type: PIECE_MIME_TYPE,
    body: crate::schema_index::index_bytes,
};

/// The media type every schema piece is served with, and the index too.
pub const PIECE_MIME_TYPE: &str = "application/json";

/// The format docs, verbatim.
pub const FORMAT_DOCS: &str = include_str!("../docs/format.md");

fn format_docs() -> String {
    FORMAT_DOCS.to_string()
}

/// Every listed resource: shape first, then the rules over it, then the index.
///
/// The index comes last because ADR-0080 froze the first two in this order before it
/// existed; the server instructions, not the listing order, are what start an agent there.
/// The pieces are not listed: the index is how they are found (ADR-0137 §3).
pub fn all() -> &'static [Resource] {
    &[SCHEMA, FORMAT, INDEX]
}

/// The listed resource at `uri`, or `None` if none is. A schema piece is served but not
/// listed, so it is not found here; [`serve`] covers both.
pub fn find(uri: &str) -> Option<&'static Resource> {
    all().iter().find(|resource| resource.uri == uri)
}

/// What one read returns: the bytes, and the media type to serve them as.
#[derive(Debug, Clone, PartialEq)]
pub struct Contents {
    pub mime_type: &'static str,
    pub text: String,
}

/// What is served at `uri` — a listed resource or a schema piece — or `None` if nothing is.
pub fn serve(uri: &str) -> Option<Contents> {
    match find(uri) {
        Some(resource) => Some(Contents {
            mime_type: resource.mime_type,
            text: resource.body(),
        }),
        None => crate::schema_index::piece_bytes(uri).map(|text| Contents {
            mime_type: PIECE_MIME_TYPE,
            text,
        }),
    }
}

/// The bytes behind one URI, or `None` if nothing is published there.
pub fn read(uri: &str) -> Option<String> {
    serve(uri).map(|contents| contents.text)
}
