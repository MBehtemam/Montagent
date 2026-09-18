//! The `Finding` — the one object every Montaget verb answers with.
//!
//! ADR-0011: *"An error is a finding. Same objects and same stable codes as ADR-0006,
//! including for invocation errors, so there is exactly one thing to parse across the
//! surface."*
//!
//! A finding carries a stable code, a class, a location, and **every relevant number
//! inline** — ADR-0006 is emphatic that a finding saying *"see `vo-sentence-06-a`"*
//! forces a re-read of the whole project, and that the re-read, not the output, is the
//! real context cost. The numbers live in [`Finding::fields`] rather than in a prose
//! string, because JSON is canonical and the prose is generated from it through the
//! registered template: one code, one field set, one template.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::registry;

/// What kind of thing a finding is.
///
/// Three of these are ADR-0006's severities, named for what the reader does. The other
/// two are report categories that are not severities at all and never gate a render:
/// `Unchecked` (ADR-0013 — the disk-agreement half of the question was *unanswerable*,
/// not *failed*) and `Layout` (ADR-0041 — a key-order violation renders identically).
/// `NOT CHECKED`, the report's own printed boundary, is not a finding at all; it lives
/// on the report (see [`crate::report::NOT_CHECKED`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Class {
    /// The render is refused or is guaranteed wrong.
    Error,
    /// Legal, renders, and you must look at a frame to know if it was meant.
    Review,
    /// A fact you may want and will not act on today.
    Note,
    /// A check that could not run. Counted in the summary line, never a verdict.
    Unchecked,
    /// The file is unsafe to edit, not unsafe to render. `validate`-only.
    Layout,
}

impl Class {
    /// Whether this class is one of ADR-0006's three severities.
    pub fn is_severity(self) -> bool {
        matches!(self, Class::Error | Class::Review | Class::Note)
    }

    /// Whether findings of this class print in full rather than collapsing to one
    /// counted line. ADR-0006: *"errors and near-errors print in full; informational
    /// classes collapse to one counted line carrying their code."*
    pub fn prints_in_full(self) -> bool {
        matches!(self, Class::Error | Class::Review)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Class::Error => "error",
            Class::Review => "review",
            Class::Note => "note",
            Class::Unchecked => "unchecked",
            Class::Layout => "layout",
        }
    }
}

/// Where in the project a finding is. Every field is optional because a finding about
/// the file as a whole has no element, and a finding about an element has no column.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_offset: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
}

/// ADR-0043's repair field, orthogonal to severity and carried by `error`-class
/// findings alone.
#[derive(Debug, Clone, PartialEq)]
pub enum Repair {
    /// Refuse-class: the fix depends on knowing what the author meant, which the
    /// document does not and cannot carry. Non-bypassable — no future flag, force mode
    /// or MCP write tool may lift it.
    None,
    /// Advise-class: the correct fix is fully determined by the document, the media on
    /// disk and the published rendering semantics.
    Advise(Value),
}

impl Serialize for Repair {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            // The literal string ADR-0043 specifies, so `repair == "none"` is a
            // machine-checkable gate with no prose parsing and no naming convention.
            Repair::None => s.serialize_str("none"),
            Repair::Advise(v) => v.serialize(s),
        }
    }
}

impl<'de> Deserialize<'de> for Repair {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(d)?;
        Ok(match v.as_str() {
            Some("none") => Repair::None,
            _ => Repair::Advise(v),
        })
    }
}

/// One group of a sibling census: the members that share one observable,
/// document-derived value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CensusGroup {
    pub value: Value,
    pub members: Vec<String>,
}

/// ADR-0006's sibling census — *"four of five are 1597, one is 1537"* — the move that
/// makes a finding actionable without deciding anything.
///
/// Groups keep the order they were declared in. Nothing here sorts by size: ADR-0043 is
/// explicit that a census *"must not be worded in a way that implies the larger group is
/// the correct one"*, and the fixture's own `word-08-target` is a census outlier that is
/// correct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Census {
    pub field: String,
    pub groups: Vec<CensusGroup>,
}

impl Census {
    pub fn on(field: impl Into<String>) -> Self {
        Census {
            field: field.into(),
            groups: Vec::new(),
        }
    }

    pub fn group<I, S>(mut self, value: Value, members: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.groups.push(CensusGroup {
            value,
            members: members.into_iter().map(Into::into).collect(),
        });
        self
    }
}

/// ADR-0061's fenced exception: a document-derived fact compared against an
/// externally-sourced numeric threshold, cited inline in the finding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Citation {
    pub threshold: Value,
    /// Where the number comes from, in words.
    pub source: String,
    /// The specific ADR that introduced the check — not `validate`'s general docs.
    pub adr: String,
}

/// ADR-0056's mandatory structured reason on `UNCHECKED`, so that a network-flavoured
/// unknown is distinguishable from an unattempted one without promoting either into its
/// own severity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UncheckedReason {
    Timeout,
    Dns,
    Unreachable,
    Http { status: u16 },
    Missing,
    PermissionDenied,
}

/// One fact about the project, with a stable code and every relevant number inline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Finding {
    pub code: String,
    pub class: Class,
    pub location: Location,
    /// The template's field set. Ordered so the canonical JSON is stable across runs.
    pub fields: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repair: Option<Repair>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub census: Option<Census>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation: Option<Citation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<UncheckedReason>,
}

impl Finding {
    /// Start a finding for a registered code. The class comes from the registry rather
    /// than from the call site, so a code cannot mean `error` in one check and `note` in
    /// another.
    ///
    /// # Panics
    ///
    /// If `code` is not registered. A check emitting an unregistered code is a bug in
    /// the check, not a condition of the project under validation.
    #[track_caller]
    pub fn new(code: &str) -> Self {
        let spec =
            registry::spec(code).unwrap_or_else(|| panic!("{code} is not a registered check code"));
        Finding {
            code: spec.code.to_string(),
            class: spec.class,
            location: Location::default(),
            fields: BTreeMap::new(),
            repair: None,
            census: None,
            citation: None,
            reason: None,
        }
    }

    pub fn at_file(mut self, file: impl Into<String>) -> Self {
        self.location.file = file.into();
        self
    }

    pub fn at_element(mut self, element: impl Into<String>) -> Self {
        self.location.element = Some(element.into());
        self
    }

    pub fn at_track(mut self, track: impl Into<String>) -> Self {
        self.location.track = Some(track.into());
        self
    }

    pub fn at_offset(mut self, line: u32, column: u32, byte_offset: u64) -> Self {
        self.location.line = Some(line);
        self.location.column = Some(column);
        self.location.byte_offset = Some(byte_offset);
        self
    }

    pub fn field(mut self, name: impl Into<String>, value: Value) -> Self {
        self.fields.insert(name.into(), value);
        self
    }

    /// ADR-0043: the fix depends on intent the document does not carry.
    pub fn refuse_class(mut self) -> Self {
        debug_assert_eq!(
            self.class,
            Class::Error,
            "`repair` is an axis of `error` alone"
        );
        self.repair = Some(Repair::None);
        self
    }

    /// ADR-0043: the fix is fully determined by the document, the disk and the
    /// published rendering semantics.
    pub fn advise_class(mut self, repair: Value) -> Self {
        debug_assert_eq!(
            self.class,
            Class::Error,
            "`repair` is an axis of `error` alone"
        );
        self.repair = Some(Repair::Advise(repair));
        self
    }

    pub fn census(mut self, census: Census) -> Self {
        self.census = Some(census);
        self
    }

    pub fn citation(mut self, citation: Citation) -> Self {
        self.citation = Some(citation);
        self
    }

    pub fn unchecked_because(mut self, reason: UncheckedReason) -> Self {
        debug_assert_eq!(
            self.class,
            Class::Unchecked,
            "a reason belongs to `UNCHECKED`"
        );
        self.reason = Some(reason);
        self
    }
}
