//! A text element's runs, their style deltas, and the timed highlight window.
//!
//! ADR-0007: *"A `text` element carries a base style and an ordered `runs` array. Each run
//! is its own text plus style deltas over the base."* A run boundary is style only — a
//! line break is a `\n` character inside a run's text, confirmed against the fixture,
//! three of whose four multi-line events break with no style change at all.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Colour;

/// One stretch of a text element's content.
///
/// Always an array, never a bare string: ADR-0007 rejected the shorthand because *"it
/// optimises every edit except the one that actually occurred"* — adding a second style to
/// a flat string is a shape change that retypes the original text at precisely the moment
/// the author is doing something typographically delicate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Run {
    /// UTF-8, NFC, raw characters — the serializer never emits `\uXXXX` above U+007F,
    /// because escaping destroys the unique-substring guarantee uniformly rather than
    /// occasionally. Whitespace inside a run is content: the fixture's `cobweb  -  cobweb`
    /// carries deliberate double spaces that no tidying pass may touch.
    pub text: String,
    /// Style deltas over the element's base style. Every one is optional because a delta
    /// that is absent is a delta that was not made.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Colour>,
    /// ADR-0014: on text the stroke falls **outside** the glyph contour and grows into the
    /// declared box rather than enlarging the element.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<Colour>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<i64>,
    /// A per-run direction override with **isolate** semantics only — never LRO/RLO, which
    /// exist for legacy data and turn "my text renders backwards" into an unfalsifiable
    /// mystery (ADR-0007).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dir: Option<Dir>,
    /// The timed window during which this run wears a different style (ADR-0048). Every
    /// highlighted word is its own run, and a run carries at most one window — letting one
    /// run span several words would need sub-run character offsets, which a later text
    /// edit invalidates silently while leaving the JSON structurally valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlight: Option<Highlight>,
}

/// A start/end window inside the element's own range, plus the style that applies during
/// it. Outside the window the run falls back to its unconditional style.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Highlight {
    /// Literal integers, authored once and frozen in the document — never a reference into
    /// an external alignment file, which would put an ASR model's opinion inside the
    /// render path (ADR-0048).
    pub start: i64,
    pub end: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Colour>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<Colour>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<i64>,
}

/// How lines align to each other — text only, and `start`/`end` rather than `left`/`right`
/// because RTL is in v1 (ADR-0007). Distinct from `origin`, which places the box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Align {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Dir {
    Ltr,
    Rtl,
}
