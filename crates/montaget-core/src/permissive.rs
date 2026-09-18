//! The second parse: order- and presence-preserving, for documents the strict model cannot
//! hold.
//!
//! ADR-0042 requires `fmt` to proceed on **any file recognisably a project**, refusing only
//! when `tracks`, `fps` or `frame` are wholesale missing — *"a `note`- or `review`-level
//! finding does not make a file any less a legitimate, safely-formattable Montaget
//! project"*. ADR-0017 makes an unknown key an `error`. Both hold at once, and together
//! they mean a file carrying an unknown key must still format: the strict model, by
//! construction, cannot represent it, so a second representation has to exist.
//!
//! It is built here, with the types, rather than discovered at the `fmt` ticket, because
//! the fact that two representations are needed is a property of the format and not of that
//! command.
//!
//! The representation is deliberately the parser's own tree. `serde_json`'s `preserve_order`
//! keeps a map in the order the file wrote it, and an absent key is simply absent — so key
//! order and field presence, the two things ADR-0041 and ADR-0030 make content, survive
//! without a line of code to carry them.

use serde::Deserialize;
use serde_json::Value;

use crate::write;

/// The three keys that make a document recognisable as a project.
///
/// ADR-0042: the precondition is narrow and structural. A filename check was rejected
/// (a caller has already asserted "this is a Montaget project" by choosing the path) and an
/// in-document marker was rejected (these keys are a stronger discriminator than a fixed
/// string, and a marker would acquire version-number-shaped risk). `validate` and `fmt`
/// share this one predicate rather than each reimplementing it.
pub const REQUIRED: [&str; 3] = ["tracks", "fps", "frame"];

/// A parsed document, held exactly as it was written.
///
/// This is the spine every check hangs off. A check that needs the format's types asks for
/// [`Loose::strict`]; a check that must survive a document the types cannot hold — which is
/// every check that reports on one — reads [`Loose::value`].
#[derive(Debug, Clone, PartialEq)]
pub struct Loose {
    path: String,
    value: Value,
    source: Option<String>,
}

/// The document is not a Montaget project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotAProject {
    /// Which of [`REQUIRED`] the document does not carry, in that order.
    pub missing: Vec<&'static str>,
}

impl std::fmt::Display for NotAProject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ADR-0042: the gap it found was message quality, not detection — a file missing
        // its required keys wholesale should name the likely mismatch, not dump a raw
        // schema error.
        write!(
            f,
            "this does not look like a Montaget project file — no `{}`",
            self.missing.join("`/`")
        )
    }
}

impl std::error::Error for NotAProject {}

impl Loose {
    /// Hold a parsed document. Nothing is checked here at all.
    ///
    /// A document with an unknown key, a retired spelling, or an `fps` of `"abc"` is held
    /// exactly as written: every one of those is a *finding* about a readable document, and
    /// ADR-0011 keeps exit 1 and exit 2 apart precisely so "your timings overlap" and "your
    /// JSON is broken" cannot be confused. Typing them into `serde` at this layer would
    /// turn each into a parse failure and collapse the distinction.
    pub fn new(path: impl Into<String>, value: Value) -> Self {
        Loose {
            path: path.into(),
            value,
            source: None,
        }
    }

    /// The bytes this document was parsed from, kept alongside the tree.
    ///
    /// The `LAYOUT` check (ADR-0041) is the reason: *"is this file written in the canonical
    /// convention"* is a question about the file's **bytes**, and every other check asks
    /// about its tree. Carried here rather than re-read at each asker, because a second
    /// read is a second answer — the file may have changed between them, and a finding
    /// located by line into bytes nobody parsed is a number that measured nothing.
    pub fn as_written(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// The path the document was read from, as given.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The bytes the document was parsed from, where it came from a file.
    ///
    /// `None` only for a document built from a [`Value`] in memory, which today is tests
    /// alone: [`crate::parse::read`] always supplies them.
    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    /// Is there a project here at all?
    ///
    /// The shared structural predicate ADR-0042 asks `validate` and `fmt` to hold in common
    /// rather than each reimplementing. What a verb *does* about a failure is the verb's
    /// own business — `fmt` refuses to write, and what `validate` answers with is its own
    /// ticket's decision — so this reports and does not act.
    pub fn shape(&self) -> Result<(), NotAProject> {
        let missing: Vec<&'static str> = match self.value.as_object() {
            Some(root) => REQUIRED
                .into_iter()
                .filter(|key| !root.contains_key(*key))
                .collect(),
            None => REQUIRED.into(),
        };
        if missing.is_empty() {
            Ok(())
        } else {
            Err(NotAProject { missing })
        }
    }

    /// The document as it was written, key order and field presence intact.
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// The document as the format's types, or the first place it does not fit them.
    ///
    /// A check that needs a typed document asks here; a check that reports on a document
    /// the types cannot hold reads [`Loose::value`] instead. The two are the same bytes —
    /// this is a view, not a second parse of the file.
    pub fn strict(&self) -> Result<crate::model::Project, serde_json::Error> {
        crate::model::Project::deserialize(&self.value)
    }

    /// Every element in the document, in document order, however malformed.
    ///
    /// Array order carries no meaning for timing or for stacking (ADR-0060), so this is a
    /// traversal and never a ranking. It yields raw values rather than typed elements
    /// because the elements worth walking are often exactly the ones that do not parse.
    pub fn elements(&self) -> impl Iterator<Item = &Value> {
        self.elements_in_tracks().map(|(_, element)| element)
    }

    /// The same walk, keeping the name of the track each element sits in.
    ///
    /// A finding's location carries a track as well as an element, so a check that reports
    /// one needs both. Kept as the single traversal with [`Loose::elements`] reading
    /// through it, rather than a second copy of the same two levels in whichever check
    /// happened to want the name — which is how the two would come to disagree about what
    /// "every element" means on a document with a malformed track.
    pub fn elements_in_tracks(&self) -> impl Iterator<Item = (Option<&str>, &Value)> {
        self.value["tracks"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .flat_map(|track| {
                let name = track.get("name").and_then(Value::as_str);
                track["elements"]
                    .as_array()
                    .map(Vec::as_slice)
                    .unwrap_or_default()
                    .iter()
                    .map(move |element| (name, element))
            })
    }

    /// The document in the canonical convention — the same writer the strict model prints
    /// through, because there is one convention and a second implementation of it would be
    /// a second place for it to go stale.
    pub fn canonical(&self) -> String {
        write::canonical(&self.value)
    }
}
