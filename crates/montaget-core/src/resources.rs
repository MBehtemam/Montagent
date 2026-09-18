//! The two MCP resources: the published schema, and the format docs.
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
//! `schema/montaget.schema.json` exists for readers outside a running server and is checked
//! against the same function by `tests/schema.rs`; nothing serves it.
//!
//! The format docs are `include_str!`d from within this crate rather than read from
//! `docs/` at the repository root, because a released binary has no repository beside it
//! (ADR-0064 ships `cargo install` and six target binaries) and a resource that resolved to
//! a missing file would be a resource that works only in a checkout.

/// One published resource.
///
/// ADR-0011 names *"the schema"* and *"the format docs"* and settles neither their URIs nor
/// their names. The two below are a published surface an agent will cite and should not
/// move; that they rest on this module rather than on a decision is raised as #246.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

/// The generated JSON Schema: the authority on the format's *shape*.
pub const SCHEMA: Resource = Resource {
    uri: "montaget://schema.json",
    name: "montaget-schema",
    title: "Montaget project JSON Schema",
    description: "The published JSON Schema for a Montaget project file — every element \
                  type, every field, every admitted value. Generated from the same types \
                  that parse your project, so it is the schema `validate` enforces rather \
                  than a description of it. Canonical key order is this schema's property \
                  order.",
    mime_type: "application/schema+json",
};

/// The prose: the authority on everything the shape cannot say.
pub const FORMAT: Resource = Resource {
    uri: "montaget://format.md",
    name: "montaget-format",
    title: "The Montaget project format",
    description: "The rules a JSON Schema cannot express: the canonical writing convention \
                  an exact-string edit depends on, half-open time ranges, what a track \
                  does and does not supply, how a layer anchor resolves, why presence is \
                  content, and how sources resolve. Read this alongside the schema before \
                  editing a project by hand.",
    mime_type: "text/markdown",
};

/// The format docs, verbatim.
pub const FORMAT_DOCS: &str = include_str!("../docs/format.md");

/// Every published resource, in the order an agent should meet them: shape first, then the
/// rules over it.
pub fn all() -> &'static [Resource] {
    &[SCHEMA, FORMAT]
}

/// The bytes behind one URI, or `None` if nothing is published there.
pub fn read(uri: &str) -> Option<String> {
    // A chain rather than a `match`: a `Resource`'s `uri` is an associated constant, which
    // is not a pattern, and duplicating the two strings as literals here is exactly the
    // second place a name can go stale.
    if uri == SCHEMA.uri {
        Some(crate::schema::generated_bytes())
    } else if uri == FORMAT.uri {
        Some(FORMAT_DOCS.to_string())
    } else {
        None
    }
}
