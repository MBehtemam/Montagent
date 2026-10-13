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
    /// Ruby or interlinear gloss annotation attached to this run, inflating its line's
    /// vertical slot when the gloss exceeds half-leading (ADR-0208).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotation: Option<Annotation>,
    /// Multi-line text range decoration rendered as per-line `SkRRect` fragments behind
    /// the run's glyphs (ADR-0214).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decoration: Option<Decoration>,
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

/// Vertical placement of a run's ruby or interlinear annotation relative to the base line
/// (ADR-0208 §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationPosition {
    /// Above the base line's ascent.
    #[default]
    Over,
    /// Below the base line's descent.
    Under,
}

impl AnnotationPosition {
    pub fn as_str(self) -> &'static str {
        match self {
            AnnotationPosition::Over => "over",
            AnnotationPosition::Under => "under",
        }
    }
}

/// Horizontal alignment of an annotation against its base run's advance width (ADR-0208 §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationAlign {
    /// Centered over or under the base run's horizontal span.
    #[default]
    Center,
    /// Aligned to the inline-start edge of the base run (`left` for LTR, `right` for RTL).
    Start,
    /// Aligned to the inline-end edge of the base run (`right` for LTR, `left` for RTL).
    End,
}

/// Keyword overhang mode for an annotation (ADR-0208 §1, §3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationOverhangMode {
    /// The annotation may extend beyond the base run's horizontal span; collisions with
    /// neighbouring annotations on the same `(position, tier)` are still checked by
    /// `R-ANNOTATION-OVERLAP`.
    #[default]
    Allow,
    /// An annotation wider than its base run emits `R-ANNOTATION-OVERHANG`.
    None,
}

/// Maximum horizontal overhang beyond the base run's advance bounds before `R-ANNOTATION-OVERHANG`
/// is reported: either a non-negative integer pixel budget (`0`, `12`, …) or `"allow"` / `"none"`
/// (ADR-0208 §1, §3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum AnnotationOverhang {
    /// Maximum allowed overhang in non-negative integer pixels on either side of the base run.
    Pixels(super::Length),
    /// Keyword overhang mode (`"allow"` or `"none"`).
    Mode(AnnotationOverhangMode),
}

impl AnnotationOverhang {
    /// Maximum allowed overhang in pixels, or `None` when overhang is unrestricted (`"allow"`).
    pub fn max_pixels(self) -> Option<i64> {
        match self {
            AnnotationOverhang::Pixels(super::Length(px)) => Some(px),
            AnnotationOverhang::Mode(AnnotationOverhangMode::None) => Some(0),
            AnnotationOverhang::Mode(AnnotationOverhangMode::Allow) => None,
        }
    }
}

/// A ruby or interlinear gloss attached to one run, laid out in step with the paragraph and
/// inflating its line's vertical slot when the gloss exceeds half-leading (ADR-0208).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Annotation {
    /// The gloss string. Single-line, non-empty.
    #[schemars(length(min = 1))]
    pub text: String,
    /// Font family key from `project.fonts`. Defaults to the run's resolved font.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    /// Font size in integer pixels. Defaults to `ceil(base_run_size * 0.5)`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 1))]
    pub size: Option<i64>,
    /// Fill colour of the annotation text. Defaults to the run's resolved colour.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Colour>,
    /// Vertical placement relative to the base line (`"over"` or `"under"`). Default `"over"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<AnnotationPosition>,
    /// Horizontal alignment relative to the base run (`"center"`, `"start"`, `"end"`). Default
    /// `"center"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align: Option<AnnotationAlign>,
    /// Non-negative vertical stacking tier (`0`, `1`, `2`, …) away from the base line, for
    /// stepped annotations when adjacent runs have long glosses. Default `0`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier: Option<super::Length>,
    /// Vertical clearance in pixels between the base line's typographic ascent/descent and tier
    /// `0` (and between consecutive tiers). Default `6`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gap: Option<super::Length>,
    /// Maximum integer pixels (or `"allow"` / `"none"`) that the annotation may extend beyond the
    /// horizontal start or end bounds of the base run without triggering `R-ANNOTATION-OVERHANG`.
    /// Default `0`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overhang: Option<AnnotationOverhang>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AnnotationWritten {
    text: String,
    #[serde(default)]
    font: Option<String>,
    #[serde(default)]
    size: Option<i64>,
    #[serde(default)]
    color: Option<Colour>,
    #[serde(default)]
    position: Option<AnnotationPosition>,
    #[serde(default)]
    align: Option<AnnotationAlign>,
    #[serde(default)]
    tier: Option<super::Length>,
    #[serde(default)]
    gap: Option<super::Length>,
    #[serde(default)]
    overhang: Option<AnnotationOverhang>,
}

impl<'de> Deserialize<'de> for Annotation {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let w = AnnotationWritten::deserialize(deserializer)?;
        if w.text.is_empty() {
            return Err(D::Error::custom(
                "a run's `annotation.text` is empty: an annotation carries a non-empty gloss \
                 string (ADR-0208)",
            ));
        }
        if let Some(size) = w.size.filter(|size| *size < 1) {
            return Err(D::Error::custom(format!(
                "a run's `annotation.size` is {size}: an annotation's font size is a positive \
                 integer of pixels (ADR-0208)"
            )));
        }
        Ok(Annotation {
            text: w.text,
            font: w.font,
            size: w.size,
            color: w.color,
            position: w.position,
            align: w.align,
            tier: w.tier,
            gap: w.gap,
            overhang: w.overhang,
        })
    }
}

/// The visual form of a run's text range decoration (ADR-0214 §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DecorationKind {
    /// A stroked and/or filled rounded rectangle around each line fragment.
    #[default]
    Box,
    /// A filled and/or stroked highlight plate behind each line fragment.
    Highlight,
    /// A horizontal bar beneath each line fragment's baseline.
    Underline,
}

/// Symmetric horizontal and vertical padding `{"x": ..., "y": ...}` in non-negative integer
/// pixels (ADR-0214 §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SymmetricPadding {
    pub x: super::Length,
    pub y: super::Length,
}

/// Four-edge padding `{"top": ..., "right": ..., "bottom": ..., "left": ...}` in non-negative
/// integer pixels (ADR-0214 §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EdgePadding {
    pub top: super::Length,
    pub right: super::Length,
    pub bottom: super::Length,
    pub left: super::Length,
}

/// Padding around each line fragment of a run decoration: uniform scalar `p`, two-element
/// `[pad_x, pad_y]`, symmetric `{"x": pad_x, "y": pad_y}`, or four-edge
/// `{"top": ..., "right": ..., "bottom": ..., "left": ...}` in non-negative integer pixels
/// (ADR-0214 §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum DecorationPadding {
    /// Uniform horizontal and vertical padding in pixels.
    Uniform(super::Length),
    /// `[pad_x, pad_y]` horizontal and vertical padding in pixels.
    Axes([super::Length; 2]),
    /// `{"x": pad_x, "y": pad_y}` horizontal and vertical padding in pixels.
    Symmetric(SymmetricPadding),
    /// `{"top": ..., "right": ..., "bottom": ..., "left": ...}` in pixels.
    Edges(EdgePadding),
}

impl DecorationPadding {
    /// `(top, right, bottom, left)` padding in integer pixels.
    pub fn trbl(self) -> (i64, i64, i64, i64) {
        match self {
            DecorationPadding::Uniform(super::Length(p)) => (p, p, p, p),
            DecorationPadding::Axes([super::Length(px), super::Length(py)]) => (py, px, py, px),
            DecorationPadding::Symmetric(SymmetricPadding {
                x: super::Length(px),
                y: super::Length(py),
            }) => (py, px, py, px),
            DecorationPadding::Edges(EdgePadding {
                top: super::Length(t),
                right: super::Length(r),
                bottom: super::Length(b),
                left: super::Length(l),
            }) => (t, r, b, l),
        }
    }
}

/// How a multi-line decoration's corner radii and borders behave across line breaks, modelled
/// after CSS `box-decoration-break` (ADR-0214 §1, §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DecorationBreak {
    /// Each line fragment is rendered as an independently closed box with full corner radius on
    /// all four corners.
    #[default]
    Clone,
    /// The first fragment rounds only its inline-start corners, middle fragments have square
    /// corners, and the last fragment rounds only its inline-end corners.
    Slice,
}

/// Timed active window `[start, end)` on the project's absolute timeline for a run's
/// decoration, with optional linear fade-in and fade-out ramps (ADR-0214 §1, §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DecorationWindow {
    /// Inclusive start timestamp in integer milliseconds on the project timeline.
    pub start: i64,
    /// Exclusive end timestamp in integer milliseconds on the project timeline (`end > start`).
    pub end: i64,
    /// Optional fade-in duration in milliseconds from `start`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fade_in: Option<super::Length>,
    /// Optional fade-out duration in milliseconds ending at `end`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fade_out: Option<super::Length>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DecorationWindowWritten {
    start: i64,
    end: i64,
    #[serde(default)]
    fade_in: Option<super::Length>,
    #[serde(default)]
    fade_out: Option<super::Length>,
}

impl<'de> Deserialize<'de> for DecorationWindow {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let w = DecorationWindowWritten::deserialize(deserializer)?;
        if w.end <= w.start {
            return Err(D::Error::custom(format!(
                "a decoration's `window` has `start: {}` and `end: {}`: `end` must be greater \
                 than `start` (ADR-0214)",
                w.start, w.end
            )));
        }
        let fade_in = w.fade_in.map_or(0, |l| l.0);
        let fade_out = w.fade_out.map_or(0, |l| l.0);
        let duration = w.end - w.start;
        if fade_in + fade_out > duration {
            return Err(D::Error::custom(format!(
                "a decoration's `window` has `fade_in + fade_out` ({} + {} = {} ms) exceeding \
                 `end - start` ({} ms) (ADR-0214)",
                fade_in,
                fade_out,
                fade_in + fade_out,
                duration
            )));
        }
        Ok(DecorationWindow {
            start: w.start,
            end: w.end,
            fade_in: w.fade_in,
            fade_out: w.fade_out,
        })
    }
}

/// A run-level visual decoration (`box`, `highlight`, or `underline`) painted behind the run's
/// glyphs as one `SkRRect` per wrapped line fragment (ADR-0214).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Decoration {
    /// Visual decoration style (`"box"`, `"highlight"`, or `"underline"`). Defaults to `"box"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<DecorationKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<Colour>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<Colour>,
    /// Stroke width in integer pixels. Defaults to `1` for `"box"` and `"underline"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<super::Length>,
    /// Dash pattern `[on, off, ...]` in integer pixels along the fragment perimeter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_dash: Option<super::DashPattern>,
    /// Corner radius in integer pixels (`box` and `highlight` only). Default `0`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<super::Length>,
    /// Padding around each line fragment's glyph bounds. Default `0`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<DecorationPadding>,
    /// Multi-line box-break mode (`"clone"` or `"slice"`, `box` and `highlight` only). Default
    /// `"clone"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decoration_break: Option<DecorationBreak>,
    /// Optional timed window `[start, end)` during which the decoration is drawn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<DecorationWindow>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DecorationWritten {
    #[serde(default)]
    kind: Option<DecorationKind>,
    #[serde(default)]
    fill: Option<Colour>,
    #[serde(default)]
    stroke: Option<Colour>,
    #[serde(default)]
    stroke_width: Option<super::Length>,
    #[serde(default)]
    stroke_dash: Option<super::DashPattern>,
    #[serde(default)]
    radius: Option<super::Length>,
    #[serde(default)]
    padding: Option<DecorationPadding>,
    #[serde(default)]
    decoration_break: Option<DecorationBreak>,
    #[serde(default)]
    window: Option<DecorationWindow>,
}

impl<'de> Deserialize<'de> for Decoration {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let w = DecorationWritten::deserialize(deserializer)?;
        let kind = w.kind.unwrap_or_default();
        if kind != DecorationKind::Underline && w.fill.is_none() && w.stroke.is_none() {
            return Err(D::Error::custom(
                "a run's `decoration` names neither `fill` nor `stroke`: at least one is \
                 required so the decoration has something to paint (ADR-0214)",
            ));
        }
        if kind == DecorationKind::Underline {
            if w.radius.is_some() {
                return Err(D::Error::custom(
                    "`radius` is a field of `kind: \"box\"` and `kind: \"highlight\"` only: \
                     an `underline` decoration has no corners to round (ADR-0214)",
                ));
            }
            if w.decoration_break.is_some() {
                return Err(D::Error::custom(
                    "`decoration_break` is a field of `kind: \"box\"` and `kind: \"highlight\"` \
                     only: an `underline` decoration has no side borders to slice (ADR-0214)",
                ));
            }
        }
        Ok(Decoration {
            kind: w.kind,
            fill: w.fill,
            stroke: w.stroke,
            stroke_width: w.stroke_width,
            stroke_dash: w.stroke_dash,
            radius: w.radius,
            padding: w.padding,
            decoration_break: w.decoration_break,
            window: w.window,
        })
    }
}
