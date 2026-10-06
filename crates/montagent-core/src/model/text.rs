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
    /// Singles out the one unit of the element's `units` block this run holds, replacing its
    /// delay or any of its lists (ADR-0151 §4). A run's lists are never delayed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<UnitOverride>,
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

/// What one unit of a stagger is (ADR-0151 §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum UnitBy {
    /// A grapheme cluster that is not whitespace. Punctuation and emoji are letters. Under
    /// `letter`, a joined piece of a joining script such as Arabic moves as one body, on its
    /// first letter's timing, and every letter keeps its index and step (ADR-0153).
    Letter,
    /// A word segment holding a letter, a digit or an emoji. Punctuation attaches to the
    /// word before it on the same line, and otherwise to the word after it.
    Word,
    /// One `\n`-separated line holding at least one letter.
    Line,
}

/// Which end of the reading order a stagger starts from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum UnitOrder {
    /// The first unit in reading order starts first.
    #[default]
    Forward,
    /// The last unit in reading order starts first.
    Reverse,
}

/// A stagger: the element's units run the same animation, each unit started a fixed step
/// after the one before (ADR-0151 §2).
///
/// The five lists are written for the first unit, on the absolute clock like every
/// keyframe. A unit's **delay** is its position in `order` times `every`, and the unit runs
/// every list late by its delay: its value at instant *t* is the list's value at
/// *t − delay*. The values are offsets inside the element: `x`, `y` (element pixels, before
/// the element's own `scale` and `rotation`) and `rotation` add to the unit's laid-out pose,
/// `scale` and `opacity` multiply it. At least one list is required. Whitespace and empty
/// lines take no step. A unit below opacity 1 fades as one group, its stroke and fill
/// together; the element's `effects`, `blend` and `opacity` apply after the units are drawn.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Units {
    pub by: UnitBy,
    /// The step between one unit's start and the next, in milliseconds. Static.
    #[schemars(range(min = 1))]
    pub every: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<UnitOrder>,
    /// Each unit's pivot, picked in the unit's box: its own advance without the added
    /// letter spacing (a word's or a line's from its first grapheme's leading edge to its
    /// last's trailing edge) by the line's slot. Default `center`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<super::Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<super::Animatable<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<super::Animatable<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<super::Animatable<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<super::Animatable<super::Scale>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<super::Animatable<f64>>,
}

/// The `units` block as written, before its rules are checked.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UnitsWritten {
    by: UnitBy,
    every: i64,
    #[serde(default)]
    order: Option<UnitOrder>,
    #[serde(default)]
    origin: Option<super::Origin>,
    #[serde(default)]
    x: Option<super::Animatable<i64>>,
    #[serde(default)]
    y: Option<super::Animatable<i64>>,
    #[serde(default)]
    rotation: Option<super::Animatable<f64>>,
    #[serde(default)]
    scale: Option<super::Animatable<super::Scale>>,
    #[serde(default)]
    opacity: Option<super::Animatable<f64>>,
}

impl<'de> Deserialize<'de> for Units {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let w = UnitsWritten::deserialize(deserializer)?;
        if w.every < 1 {
            return Err(D::Error::custom(format!(
                "`every` is {}: the step between one unit's start and the next is a \
                 positive number of milliseconds (ADR-0151)",
                w.every
            )));
        }
        if w.x.is_none()
            && w.y.is_none()
            && w.rotation.is_none()
            && w.scale.is_none()
            && w.opacity.is_none()
        {
            return Err(D::Error::custom(
                "a `units` block names none of `x`, `y`, `rotation`, `scale` and `opacity`: \
                 at least one list is what the units run (ADR-0151)",
            ));
        }
        Ok(Units {
            by: w.by,
            every: w.every,
            order: w.order,
            origin: w.origin,
            x: w.x,
            y: w.y,
            rotation: w.rotation,
            scale: w.scale,
            opacity: w.opacity,
        })
    }
}

/// A run singling out one unit (ADR-0151 §4): the run holds exactly that unit's graphemes.
///
/// `delay` replaces the derived *position × every* for this unit. A list replaces that
/// property's list for this unit wholesale; **a run's lists are never delayed** — they run on
/// the absolute clock as written. A property the run does not name keeps the block's list,
/// run at the run's delay. Only a list the `units` block declares may be named. At least one
/// key is required.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UnitOverride {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 0))]
    pub delay: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<super::Animatable<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<super::Animatable<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<super::Animatable<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<super::Animatable<super::Scale>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<super::Animatable<f64>>,
}

/// A run's `unit` as written, before its rules are checked.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UnitOverrideWritten {
    #[serde(default)]
    delay: Option<i64>,
    #[serde(default)]
    x: Option<super::Animatable<i64>>,
    #[serde(default)]
    y: Option<super::Animatable<i64>>,
    #[serde(default)]
    rotation: Option<super::Animatable<f64>>,
    #[serde(default)]
    scale: Option<super::Animatable<super::Scale>>,
    #[serde(default)]
    opacity: Option<super::Animatable<f64>>,
}

impl<'de> Deserialize<'de> for UnitOverride {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let w = UnitOverrideWritten::deserialize(deserializer)?;
        if let Some(delay) = w.delay.filter(|delay| *delay < 0) {
            return Err(D::Error::custom(format!(
                "a run's `unit.delay` is {delay}: a delay is a whole number of milliseconds \
                 from 0 up (ADR-0151)"
            )));
        }
        if w.delay.is_none()
            && w.x.is_none()
            && w.y.is_none()
            && w.rotation.is_none()
            && w.scale.is_none()
            && w.opacity.is_none()
        {
            return Err(D::Error::custom(
                "a run's `unit` is empty: it names a `delay` or a list (`x`, `y`, \
                 `rotation`, `scale`, `opacity`) that replaces the block's for this unit \
                 (ADR-0151)",
            ));
        }
        Ok(UnitOverride {
            delay: w.delay,
            x: w.x,
            y: w.y,
            rotation: w.rotation,
            scale: w.scale,
            opacity: w.opacity,
        })
    }
}
