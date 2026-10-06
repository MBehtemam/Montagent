//! The project format, as Rust types.
//!
//! This is the module every later ticket reads the document through, and the one place the
//! format's shape is stated once. Three rules run through all of it and are worth knowing
//! before reading any single type:
//!
//! **Field order is the format.** ADR-0041 fixes canonical key order as the published
//! schema's property-declaration order, and the schema is generated from these types — so
//! **struct field order here *is* canonical key order**. Nothing in this module states that
//! order a second time: [`crate::layout`] reads it back out of the generated schema, and is
//! the only place that does. Reordering a field later is a
//! format change that surfaces as a `LAYOUT` finding on every file already written. Every
//! type below carries the universal prefix `id, type, group, start, end` first, then its
//! own order as ADR-0041's table states it, then `effects` last (ADR-0068).
//!
//! **Presence is content.** ADR-0030: omission and explicit-at-default are two spellings
//! of *different declarations* — omission says "give me whatever the default is", an
//! explicit `opacity: 1` says "I have pinned this to 1" — and they diverge the moment the
//! default is revisited. So every defaultable field is an `Option<T>` with
//! `skip_serializing_if`, never a `#[serde(default)]` that would erase the distinction on
//! the way back out.
//!
//! **The schema is closed, everywhere.** ADR-0017: every object shape carries
//! `deny_unknown_fields`, including `run` and `keyframe`, because the unknown-key error is
//! the *only* signal an old binary has that a file was authored against a newer schema —
//! and an optional signal is indistinguishable from no signal.

pub mod effects;
mod element;
pub mod keyframe;
pub mod paint;
pub mod playback;
pub mod text;

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

pub use effects::{Effect, Fraction, MaskShape, ScreenColour};
pub use keyframe::{Animatable, Derivation, Ease, EaseName, Keyframe, is_keyframe_list};
pub use paint::{Gradient, Paint, ResolvedGradient, Stops};
pub use playback::{AudioOverrun, Speed, Volume};
pub use text::{Align, Dir, Highlight, Run, UnitBy, UnitOrder, UnitOverride, Units};

/// `[sx, sy]`, never a bare number.
///
/// ADR-0012 paid the +26 % this costs on the fixture's isotropic Ken Burns lists
/// deliberately: a union would put a shape test in every consumer that touches a keyframe,
/// and `fmt` normalising a scalar on write is not the escape hatch it looks like — the
/// agent's next exact-string replace on the string it just wrote gets zero hits.
pub type Scale = [f64; 2];

/// The static frame-space aperture, `[x, y, width, height]` in absolute integer pixels.
///
/// ADR-0025 settled that it never animates: it is simultaneously the aperture a source is
/// drawn through and the fixed denominator ADR-0015's fit check compares a declared extent
/// against, and the second job has no well-defined predicate if the box has one value per
/// instant.
pub type Clip = [i64; 4];

/// `#RRGGBB` or `#RRGGBBAA`, uppercase, and nothing else.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Colour(String);

impl Colour {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// `[r, g, b, a]` as the four bytes the literal spells. A six-digit colour is opaque.
    pub fn bytes(&self) -> [u8; 4] {
        let body = self.0.trim_start_matches('#');
        let byte = |at: usize| {
            body.get(at..at + 2)
                .and_then(|pair| u8::from_str_radix(pair, 16).ok())
        };
        [
            byte(0).unwrap_or(0),
            byte(2).unwrap_or(0),
            byte(4).unwrap_or(0),
            byte(6).unwrap_or(0xFF),
        ]
    }

    /// The one literal that spells these four bytes: six digits when opaque, eight
    /// otherwise — the form a resolved colour is printed in, so it can be pasted back
    /// (ADR-0146).
    pub fn from_bytes([r, g, b, a]: [u8; 4]) -> Colour {
        match a {
            0xFF => Colour(format!("#{r:02X}{g:02X}{b:02X}")),
            a => Colour(format!("#{r:02X}{g:02X}{b:02X}{a:02X}")),
        }
    }
}

/// A non-negative length in integer pixels: a shape's `width`, `height`, `radius` and
/// `stroke_width`, and a text element's `stroke_width` (ADR-0146 §5).
///
/// `0` is legal — collapsing to nothing is a real move — and a negative value is a schema
/// error, in a static value and in every keyframe record alike. A value between two
/// keyframes may still pass below zero under an overshooting bezier; what that draws is the
/// resolver's rule ([`crate::animatable`]), not this type's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Length(pub i64);

impl<'de> Deserialize<'de> for Length {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let pixels = i64::deserialize(deserializer)?;
        if pixels < 0 {
            return Err(D::Error::custom(format!(
                "a length is {pixels}: a `width`, `height`, `radius` or `stroke_width` is a \
                 non-negative integer of pixels, and `0` is legal (ADR-0146)"
            )));
        }
        Ok(Length(pixels))
    }
}

impl JsonSchema for Length {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Length".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::Schema::try_from(serde_json::json!({
            "type": "integer",
            "format": "int64",
            "description": "A length in integer pixels. `0` is legal, and a negative value \
                            is a schema error, in a static value and in every keyframe \
                            record (ADR-0146).",
            "minimum": 0,
        }))
        .expect("an object literal is a schema")
    }
}

impl<'de> Deserialize<'de> for Colour {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = String::deserialize(d)?;
        let body = text.strip_prefix('#').ok_or_else(|| {
            D::Error::custom(format!(
                "{text} is not a colour; write `#RRGGBB` or `#RRGGBBAA`"
            ))
        })?;
        if !body
            .bytes()
            .all(|b| b.is_ascii_digit() || b.is_ascii_uppercase())
        {
            return Err(D::Error::custom(format!(
                "{text} is not a colour: hex digits are uppercase, and there are no CSS names"
            )));
        }
        if !body.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(D::Error::custom(format!(
                "{text} is not a colour: expected hex digits"
            )));
        }
        // The three-digit shorthand and the fully-opaque eight-digit form are each a second
        // spelling of a value the six-digit form already says, and two spellings of one
        // value break the write-read round trip the same way `center-center` does.
        match body.len() {
            6 => {}
            8 if body.ends_with("FF") => {
                return Err(D::Error::custom(format!(
                    "{text} is the opaque form of #{}; write the six-digit form",
                    &body[..6]
                )));
            }
            8 => {}
            3 => {
                return Err(D::Error::custom(format!(
                    "{text} is the three-digit shorthand; write all six digits"
                )));
            }
            n => {
                return Err(D::Error::custom(format!(
                    "{text} has {n} hex digits; a colour has six, or eight with alpha"
                )));
            }
        }
        Ok(Colour(text))
    }
}

/// The nine-way point of an element's own box that its `x`,`y` places, and about which
/// transforms pivot. Vertical component first.
///
/// `center-center` is deliberately absent and is a schema error naming `center`: the middle
/// elides to `center` alone, because two spellings of one value break the write-read round
/// trip. It is `center-left`, never `middle-left` — `top-center` needs a horizontal-middle
/// word regardless, so a separate `middle` would spell one concept two ways depending on
/// axis (ADR-0013).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Origin {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// A claim about how the author computed `width`/`height` — not a layout mode.
///
/// ADR-0015's load-bearing sentence: the declared rect is authoritative at render, so `fit`
/// **never executes**. No renderer reads it; its only consumer is `validate`. It is the
/// format's first field that is purely an assertion — a provenance tag on two integers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Fit {
    /// Drawn rect ⊇ the `clip` box. Requires `clip`.
    Cover,
    /// Drawn rect ⊆ the `clip` box. Requires `clip`.
    Contain,
    /// No derivation rule. The escape value needed a name and `literal` is not a coinage —
    /// ADR-0007's headline is *"Literal `size`. No fit-to-box."*
    Literal,
}

/// PROTOTYPE #718 (ADR-0155 §2): sub-frame accumulation over a centred shutter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MotionBlur {
    /// Degrees, 1 to 360: the interval is `shutter / 360` of one frame.
    #[schemars(range(min = 1, max = 360))]
    pub shutter: u16,
    /// Paints on a moving frame, 2 to 32.
    #[schemars(range(min = 2, max = 32))]
    pub samples: u8,
}

/// How the finished element composites into what is painted below it (ADR-0147).
///
/// The last step of an element's paint: its `effects` (masks and shadow included), then
/// `opacity`, then this, once, inside its `clip`. The arithmetic runs on the stored sRGB
/// values. Static: a mode has no in-between, so a blend is faded in through `opacity`.
/// Omitted means `normal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Blend {
    /// Source over: what an element without the field draws.
    #[default]
    Normal,
    /// `s·d`: darkens; white is the identity.
    Multiply,
    /// `s + d − s·d`: lightens; black is the identity.
    Screen,
    /// Multiply where the backdrop is dark, screen where it is light.
    Overlay,
    /// `min(1, s + d)`: Skia's `Plus`.
    Add,
}

impl Blend {
    /// The word the file uses.
    pub fn as_str(self) -> &'static str {
        match self {
            Blend::Normal => "normal",
            Blend::Multiply => "multiply",
            Blend::Screen => "screen",
            Blend::Overlay => "overlay",
            Blend::Add => "add",
        }
    }
}

/// What a time-based element does past the end of its (possibly speed-adjusted) source.
///
/// There is no third value and no `"none"`: the field's presence alone signals the
/// condition, so an explicit "no overrun" would be indistinguishable from omission
/// (ADR-0020).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Overrun {
    /// The last in-range frame freezes and repeats. A schema error on audio, where the
    /// correct spelling of "then silence" is a shorter element and a gap.
    Hold,
    /// The source restarts from `source_start` with a hard cut — no crossfade, which would
    /// need an unstated duration and curve.
    Loop,
}

/// The project's pixel dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub width: i64,
    pub height: i64,
}

/// One entry in a font's ordered fallback chain: always a file path relative to the
/// project, never a system family name (ADR-0007).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FontFile {
    pub file: String,
    /// For `.ttc` collections. 49 of the fonts in a stock macOS `/System/Library/Fonts`
    /// are collections, so this is the normal case rather than an exotic one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
}

/// What `fonts vendor` learned about one font file, keyed by that file's path.
///
/// ADR-0057 put this in its own top-level table rather than inline on each chain entry:
/// the `fonts` table is a *reference* structure and one file may appear under several
/// keys, so inlining would duplicate the same hash and licence at every point of reference
/// — and duplicated facts drift the moment one copy is re-vendored and the other is not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FontAttestation {
    pub licence: String,
    pub source: String,
    pub sha256: String,
}

/// An element's own place in the stack, overriding the one its track supplies.
///
/// Polymorphic on purpose, and ranked last on ease of writing by all eight agents in
/// ADR-0004's evaluation — who asked for it anyway, because the alternative is
/// hand-maintaining `panel.layer = title.layer - 1` and re-deriving it every time the
/// title moves.
///
/// `untagged` rather than a wrapper object, because the two forms are what the format
/// writes: `"layer": 12` and `"layer": {"below": "title"}`. There is no third spelling —
/// an anchor carrying a bare string is a retired one, and ADR-0016's unknown-key error
/// names its replacement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Layer {
    /// A plain integer — higher draws in front. **The only form an anchor's target may
    /// carry**: ADR-0019 resolves in exactly one hop, so a target whose own layer is an
    /// anchor is an error rather than a second lookup.
    Absolute(i64),
    /// Stated relative to another element's `id`.
    Relative(Anchor),
}

/// A layer stated relative to another element's `id` — `{"below": "title"}` resolves to
/// that element's layer minus one, wherever either of them sits (ADR-0019).
///
/// The target string resolves against the element `id` namespace **only**, never a track
/// name: the two look alike as bare strings, and one of issue #8's three agents could not
/// tell which `"title"` meant. `CONTEXT.md`'s **Id** entry records the separation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub enum Anchor {
    Below(String),
    Above(String),
}

impl Anchor {
    /// The `id` this anchor names.
    pub fn target(&self) -> &str {
        match self {
            Anchor::Below(target) | Anchor::Above(target) => target,
        }
    }

    // The arithmetic that turns an anchor into a layer is deliberately **not** here. It
    // lives once, in `crate::stack`, with the resolution it is one step of — a second copy
    // on this type would be the two-implementations-one-rule drift `crate::stack` is
    // written against, spelled in the type the rule is about.
}

/// A named container holding elements, with an integer `layer` giving its place in the
/// stack.
///
/// A track supplies *stacking*, never *timing*: it has no start, no duration and no clock
/// (ADR-0004). Array order carries no meaning, for timing or for stacking — ADR-0060
/// confirmed a layer tie is never broken by declaration order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Track {
    pub name: String,
    pub layer: i64,
    pub elements: Vec<Element>,
}

/// The whole video, as a single declarative document.
///
/// Field order is canonical key order (ADR-0041), and matches the committed fixture
/// exactly. `loop` is the one field whose position no ADR states — ADR-0062 introduced it
/// and ADR-0041's rule hands the position to the introducing ADR, which did not take it.
/// It is placed next to `duration` because that is the field it talks about: `loop: true`
/// asserts that `duration` connects back to `0`. Recorded rather than decided silently.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub frame: Frame,
    pub fps: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<Colour>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,
    /// ADR-0062. Default `false`/absent; affects nothing but one `validate` check, and
    /// Montagent writes no container-level loop metadata.
    #[serde(rename = "loop", default, skip_serializing_if = "Option::is_none")]
    pub looping: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fonts: Option<BTreeMap<String, Vec<FontFile>>>,
    #[serde(
        rename = "fontVendor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub font_vendor: Option<BTreeMap<String, FontAttestation>>,
    pub tracks: Vec<Track>,
}

/// One thing placed on the timeline.
///
/// The first six fields are the universal prefix every element carries regardless of type
/// — `id, type, group, start, end, layer` — with `group` and `layer` omitted entirely
/// rather than written as `null` when the element carries neither (ADR-0041, ADR-0030).
#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    /// Required and unique. Its only job is to be a target — an anchor's `below`/`above`
    /// (ADR-0019), a transition's pair.
    pub id: String,
    /// Optional, and render-inert: it says two elements belong to one authorial unit and
    /// nothing more.
    pub group: Option<String>,
    /// Integer milliseconds on the project's single absolute timeline. The interval is
    /// **half-open** — `[start, end)` — so an element whose `end` is 7500 is not on screen
    /// at 7500 and its neighbour starting at 7500 is (ADR-0005).
    pub start: i64,
    pub end: i64,
    /// This element's own stacking position, overriding its track's (ADR-0004). Absent on
    /// almost every element: the track is where stacking normally lives, and an override
    /// is the exception that says *this one element sits somewhere else*.
    ///
    /// Last in the universal prefix, so it is the same position on every type. ADR-0041
    /// enumerated the prefix as `id, type, group, start, end` and handed `layer`'s position
    /// to "the ADR that introduces it into a given type's schema" — and neither ADR-0004,
    /// which introduced the override, nor ADR-0019, which fixed the anchor's semantics,
    /// took it. It is placed here rather than at the head of each type's tail because it is
    /// a field of *every* type, which is what the prefix means; the per-type tails are the
    /// fields that differ. Unratified surface, raised as #243 rather than left to be
    /// discovered from the struct.
    pub layer: Option<Layer>,
    /// The type-discriminated remainder. `type` is written between `id` and `group`, which
    /// is why the prefix is assembled by hand rather than by `#[serde(flatten)]`.
    pub body: Body,
}

/// The per-type field set, discriminated by `type`.
///
/// ADR-0012: the field set is a **function of `type`** — *"`x` on an audio element is a
/// schema error naming the replacement"*, never a silently-ignored field. That is also why
/// shapes are sibling types rather than one `shape` type with a discriminator inside it: a
/// second, omissible discriminator would let a malformed ellipse render as a rect, silently
/// and plausibly (ADR-0014).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Body {
    Image(Image),
    Video(Video),
    Text(TextElement),
    Rect(Rect),
    Ellipse(Ellipse),
    Path(PathElement),
    Audio(Audio),
    Transition(Transition),
}

impl Body {
    /// The `type` string, which is also the substring an agent greps under exact-string
    /// editing.
    pub fn type_name(&self) -> &'static str {
        match self {
            Body::Image(_) => "image",
            Body::Video(_) => "video",
            Body::Text(_) => "text",
            Body::Rect(_) => "rect",
            Body::Ellipse(_) => "ellipse",
            Body::Path(_) => "path",
            Body::Audio(_) => "audio",
            Body::Transition(_) => "transition",
        }
    }
}

/// `source, x, y, origin, width, height, fit, clip, scale` — ADR-0041's measured order —
/// then the two transform properties ADR-0012 declares after `scale`, then `effects`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Image {
    /// Inline, never an asset-table reference (ADR-0002).
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<Animatable<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<Animatable<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<Origin>,
    /// Required, never defaulted: the source's dimensions are not in the document, so a
    /// natural-size default would make the element's rendered rect unreadable — and it
    /// would fail *quietly*, because a centre-cropped photo looks plausible (ADR-0012).
    pub width: i64,
    pub height: i64,
    /// Required on every element carrying a raster source; omission is a schema error
    /// naming the three values (ADR-0015).
    pub fit: Fit,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clip: Option<Clip>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<Animatable<Scale>>,
    /// Degrees clockwise, and **never normalised into `[0,360)`** — `1080` is three turns,
    /// and a writer that wraps it silently renders one third of the motion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Animatable<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<Animatable<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blend: Option<Blend>,
    /// PROTOTYPE #718: ADR-0155's per-element motion blur.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motion_blur: Option<MotionBlur>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<Effect>>,
}

/// An image that also has a clock.
///
/// ADR-0041 has no measured order for `video` — the fixture has no instance — and states
/// that whichever ADR first fixes its full property set fixes its order too. No ADR has,
/// so this ticket does, following the two orders that *are* measured: `image`'s visual
/// sequence, with `audio`'s `source_start, source_end` in `audio`'s own relative position
/// directly after `source`, then the fields a video shares with audio.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Video {
    pub source: String,
    pub source_start: i64,
    pub source_end: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<Animatable<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<Animatable<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<Origin>,
    pub width: i64,
    pub height: i64,
    pub fit: Fit,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clip: Option<Clip>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<Animatable<Scale>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Animatable<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<Animatable<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blend: Option<Blend>,
    /// PROTOTYPE #718: ADR-0155's per-element motion blur.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motion_blur: Option<MotionBlur>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<Speed>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overrun: Option<Overrun>,
    /// A video element is one element with intrinsic audio, so its embedded audio reuses
    /// this same field and there is no `mute` — `volume: 0` already says silent, including
    /// at one instant via a keyframe (ADR-0055).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<Animatable<Volume>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<Effect>>,
}

/// `x, y, origin, width, height, font, size, line_height, color, align, runs` — ADR-0041's
/// measured order — then the paint ADR-0014 adds, the transform properties, `effects`, and
/// last `caption`, which draws nothing and so follows everything that does (ADR-0136).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextElement {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<Animatable<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<Animatable<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<Origin>,
    /// The box the text must fit inside — required, because an omitted `height` would be
    /// indistinguishable from a decision not to check (ADR-0014). It is a *container
    /// claim*, not drawn geometry, which is why a stroke grows into it rather than past it.
    pub width: i64,
    pub height: i64,
    /// A key into the project's `fonts` table, never a system family name.
    pub font: String,
    /// A literal number. The renderer never chooses a size and never chooses a line break
    /// (ADR-0007) — the fitted sizes in the fixture came from a pipeline that measured and
    /// wrote literals, and that loop moves to authoring time rather than disappearing.
    ///
    /// An integer, by ADR-0012's absolute-integer-pixels rule: a size is a length in the
    /// project's frame space, like `width` and `height`, and all 22 of the fixture's text
    /// elements carry one. A float would additionally give every whole size two spellings —
    /// `55` and `55.0` — which is the ambiguity `center-center` and the `scale` union were
    /// each retired for.
    pub size: i64,
    /// Restricted to one decimal digit, always exactly representable as `n/10`, so the
    /// block-height derivation is exact integer arithmetic and never IEEE double
    /// (ADR-0028). Defaults to `1.2` when omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_height: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Paint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>,
    pub runs: Vec<Run>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<Paint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<Animatable<Length>>,
    /// Space added after every grapheme of a line but the last, in **thousandths of an em**
    /// of the size of the run the grapheme sits in: `size × letter_spacing / 1000` pixels.
    /// Negative tightens. Default 0. Element-level only: a run cannot override it. None is
    /// added between two letters of the same joining script, such as Arabic. An animatable
    /// property: keyframe values are integers, and the resolved value is never rounded.
    /// Any non-zero value or keyframe switches the optional ligatures `liga`, `clig` and
    /// `dlig` off for the whole element, outside the joining scripts (ADR-0151, ADR-0153).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<Animatable<i64>>,
    /// A stagger over the element's letters, words or lines (ADR-0151 §2–§3): each unit runs
    /// the block's lists late by its delay. A `by: letter` block switches the optional
    /// ligatures off as a non-zero `letter_spacing` does.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub units: Option<Units>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<Animatable<Scale>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Animatable<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<Animatable<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blend: Option<Blend>,
    /// PROTOTYPE #718: ADR-0155's per-element motion blur.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motion_blur: Option<MotionBlur>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<Effect>>,
    /// `false` says this text is not a caption — a title, a lower-third, a logo, a
    /// kinetic word — and silences all four caption checks on it: `R-CAPTION-PACE`,
    /// `R-CAPTION-MIN-DURATION`, `R-CAPTION-NO-AUDIO` and `R-CAPTION-REPEAT-DURATION`, whose
    /// grouping of identical text leaves it out. Omitted means `true`: every text element
    /// is a caption unless it says otherwise. It changes no pixel and no other check
    /// (ADR-0136).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<bool>,
}

/// A rounded or square rectangle.
///
/// **`rect` and `ellipse` are two types rather than one with a `shape` discriminator
/// inside it** (ADR-0014): ADR-0012 already made the field set a function of `type`, so a
/// second narrowing mechanism would add a second, *omissible* one — and a malformed
/// ellipse missing its discriminator would render as a rect, silently and plausibly, which
/// is this format's named failure class.
///
/// They are also two *structs*, and the one field that differs is why: ADR-0014's heading
/// is *"`radius` is a field on `rect`"*, so `radius` on an ellipse is an unknown key rather
/// than a field that quietly does nothing. An ellipse inscribing its rect has no corners to
/// round, and ADR-0007 has ruled twice that *"a field the renderer cannot honour is worse
/// than no field."*
///
/// ADR-0041's measured order is `x, y, origin, width, height, fill`; the three fields
/// ADR-0014 adds follow, in the order that ADR's own headings introduce them — `stroke`,
/// `stroke_width`, then `radius`. ADR-0041 hands a new field's position to the ADR that
/// introduces it and ADR-0014 did not take it explicitly, so that reading is this
/// ticket's and is raised as [#274](https://github.com/MBehtemam/Montagent/issues/274)
/// rather than left to be discovered from the struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Rect {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<Animatable<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<Animatable<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<Origin>,
    pub width: Animatable<Length>,
    pub height: Animatable<Length>,
    /// Optional when a `stroke` is present, giving an outlined shape. A shape with neither
    /// is a schema error naming both, because an element that deliberately renders nothing
    /// and an element that forgot its paint must not look alike.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<Paint>,
    /// On a shape the stroke falls **inside** the declared rect, so a stroked `card-05`
    /// still occupies exactly 984×169 (ADR-0014).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<Paint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<Animatable<Length>>,
    /// A single integer, defaulting to 0 — one corner radius, not four.
    ///
    /// The fixture is measurably square and the README's *"rounded cream panel"* was wrong,
    /// which under ADR-0003's asymmetry is **not** evidence against the field: a rounded
    /// rectangle is unremarkable in the CapCut/Premiere reference class, and admitting it
    /// now costs one clause where admitting it later is a schema change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<Animatable<Length>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<Animatable<Scale>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Animatable<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<Animatable<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blend: Option<Blend>,
    /// PROTOTYPE #718: ADR-0155's per-element motion blur.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motion_blur: Option<MotionBlur>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<Effect>>,
}

/// An ellipse inscribing its declared rect — which is exactly what distinguishes it from
/// the point-list shapes ADR-0014 rejected: it needs no new placement rule.
///
/// [`Rect`]'s field set minus `radius`; see that type for why the two are separate.
///
/// **Contradicts a parenthetical in [ADR-0041], and says so rather than overriding it
/// silently** (`docs/agents/domain.md`). That ADR writes *"`video` and `ellipse` have no
/// committed instance to measure yet; per the rule above, whichever ADR first fixes their
/// full property set (they inherit `image`'s and `rect`'s shape respectively) fixes their
/// order too"* — and the shape an ellipse would inherit now carries `radius`. The same
/// sentence is what resolves it: the ADR that fixes a type's full property set fixes it,
/// and that ADR is ADR-0014, whose own heading is *"`radius` is a field on `rect`"*. So the
/// inheritance holds for every field ADR-0041 measured and stops at the one ADR-0014 gave
/// to `rect` alone. Raised as [#274](https://github.com/MBehtemam/Montagent/issues/274)
/// rather than left as a discrepancy a reader has to reconcile.
///
/// [ADR-0041]: ../../../../docs/adr/0041-canonical-key-order-is-schema-order-validate-checks-it-fmt-splits.md
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Ellipse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<Animatable<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<Animatable<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<Origin>,
    pub width: Animatable<Length>,
    pub height: Animatable<Length>,
    /// Optional when a `stroke` is present, giving an outlined shape. A shape with neither
    /// is a schema error naming both, because an element that deliberately renders nothing
    /// and an element that forgot its paint must not look alike.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<Paint>,
    /// On a shape the stroke falls **inside** the declared rect, so a stroked `card-05`
    /// still occupies exactly 984×169 (ADR-0014).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<Paint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<Animatable<Length>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<Animatable<Scale>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Animatable<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<Animatable<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blend: Option<Blend>,
    /// PROTOTYPE #718: ADR-0155's per-element motion blur.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motion_blur: Option<MotionBlur>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<Effect>>,
}

/// A drawing from a list of vertices, in integer pixels from the declared box's top-left
/// corner (ADR-0154).
///
/// A `rect`'s field set less `radius`, plus `closed` and `points`. Its box is placed exactly
/// as a `rect`'s is, and it **bounds** the drawing without scaling it: resizing the box does
/// not move a point, so `width`, `height` and `closed` are static, and a keyframe list on any
/// of them is a schema error. Its stroke is centred on the outline, with a round join and a
/// butt cap, and `validate` checks that the box contains it (`E-PATH-OUTSIDE-BOX`).
///
/// The order follows ADR-0154's own example — `x, y, origin, width, height, closed`, the
/// paint, then `points` — and the transform tail every visual type shares.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, remote = "Self")]
pub struct PathElement {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<Animatable<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<Animatable<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<Origin>,
    /// Static on a path: the box bounds the drawing and does not stretch it (ADR-0154 §2).
    /// Resize a path by editing its points, or animate `scale`.
    #[serde(deserialize_with = "static_side")]
    pub width: Length,
    #[serde(deserialize_with = "static_side")]
    pub height: Length,
    /// Whether a last segment runs from the last vertex back to the first. Static: a path
    /// that opens mid-clip is two elements (ADR-0154 §1).
    #[serde(deserialize_with = "static_closed")]
    pub closed: bool,
    /// Only on a closed path: `fill` with `"closed": false` is a schema error, because the
    /// format does not close an open path silently to fill it (ADR-0154 §5). Nonzero
    /// winding, so a self-intersecting outline fills its overlap. A gradient is measured
    /// against the declared box, as on every shape (ADR-0149).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<Paint>,
    /// Centred on the outline, with a round join and a butt cap (ADR-0154 §4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<Paint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<Animatable<Length>>,
    pub points: Animatable<Points>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<Animatable<Scale>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Animatable<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<Animatable<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blend: Option<Blend>,
    /// PROTOTYPE #718: ADR-0155's per-element motion blur.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motion_blur: Option<MotionBlur>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<Effect>>,
}

impl PathElement {
    /// ADR-0154 §5's relational rule, which the derive cannot state.
    fn checked(self) -> Result<Self, String> {
        if !self.closed && self.fill.is_some() {
            return Err(
                "`fill` on a path with `\"closed\": false`: an open path takes only `stroke`, \
                 and the format does not close a path silently to fill it — set \
                 `\"closed\": true` or drop `fill` (ADR-0154)"
                    .to_string(),
            );
        }
        Ok(self)
    }
}

// The two halves of `remote = "Self"`, as on `Transition`.
impl Serialize for PathElement {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        PathElement::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for PathElement {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        PathElement::deserialize(deserializer)?
            .checked()
            .map_err(D::Error::custom)
    }
}

/// A path's `width` or `height`: a [`Length`], and never a keyframe list (ADR-0154 §2).
fn static_side<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Length, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    if value.is_array() {
        return Err(D::Error::custom(
            "`width` and `height` are static on a path: the box bounds the drawing and does \
             not stretch it, so resize a path by editing its points, or animate `scale` \
             (ADR-0154)",
        ));
    }
    Length::deserialize(value).map_err(D::Error::custom)
}

/// A path's `closed`: a boolean, and never a keyframe list (ADR-0154 §1).
fn static_closed<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    if value.is_array() {
        return Err(D::Error::custom(
            "`closed` is static on a path: a path that opens mid-clip is two elements \
             (ADR-0154)",
        ));
    }
    bool::deserialize(value).map_err(D::Error::custom)
}

/// A pair of integer pixels: a vertex's `at`, or a handle's offset from its vertex.
pub type Pixels = [i64; 2];

/// One vertex of a `path` (ADR-0154 §1).
///
/// `at` is the vertex, in integer pixels from the declared box's top-left corner. `in` and
/// `out` are optional **handles**: integer offsets from their own vertex. `out` shapes the
/// segment leaving the vertex and `in` the segment arriving at it. A missing handle is a
/// zero offset, so a vertex with neither is a corner; a written `[0, 0]` draws the same.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Vertex {
    pub at: Pixels,
    #[serde(rename = "in", default, skip_serializing_if = "Option::is_none")]
    pub arriving: Option<Pixels>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub out: Option<Pixels>,
}

/// A path's vertex list: one value of `points`, whole, as a keyframe's value is whole
/// (ADR-0154 §3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Points(pub Vec<Vertex>);

impl JsonSchema for Points {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Points".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let vertex = generator.subschema_for::<Vertex>().to_value();
        schemars::Schema::try_from(serde_json::json!({
            "type": "array",
            "items": vertex,
            "description": "A path's vertices, in order. Each segment is a cubic Bezier from \
                            one vertex's `at` through `at + out`, then the next vertex's \
                            `at + in`, to that `at`; a closed path adds the segment from the \
                            last vertex back to the first (ADR-0154).",
        }))
        .expect("an object literal is a schema")
    }
}

/// `source, source_start, source_end` — ADR-0041's measured order — then the fields the
/// fixture appends after a type's core set.
///
/// An audio element carries no transform: `x` on it is a schema error naming the
/// replacement, and the specific trap is `opacity`, which an agent will write meaning
/// volume and which would fade nothing, forever (ADR-0012).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Audio {
    pub source: String,
    pub source_start: i64,
    pub source_end: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<Speed>,
    // No field doc: [`AudioOverrun`] carries the whole of why `hold` is absent here, and
    // `schemars` publishes a field's doc *and* its type's, so a second copy would put the
    // same sentence twice on one property.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overrun: Option<AudioOverrun>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<Animatable<Volume>>,
}

/// A transition is its own element type, with its own time range and two id references —
/// not a property on either bridged element, which would create an unprincipled ownership
/// question, and not an effect, which is element-local by construction (ADR-0059).
///
/// ADR-0059 names *"two id references"* without naming the fields. This ticket calls them
/// `from` and `to`, and records that as a spelling the ADR series may want to ratify.
///
/// `direction` and `ease` are fields of `wipe`, `slide` and `push` only (ADR-0150): a
/// crossfade travels nowhere, and its linear ramp is what keeps its two halves summing to
/// a constant. A Rust struct cannot say "required under three `kind`s and an unknown key
/// under the fourth", so [`Transition::checked`] says it on the way in and
/// `crate::schema::publish_transition_fields` says it in the published schema — the
/// arrangement `mask`'s `radius` already has.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, remote = "Self")]
pub struct Transition {
    /// `crossfade` ramps the two elements' opacity; `wipe`, `slide` and `push` move or cut
    /// them a whole frame's width or height, outside their own transforms (ADR-0150).
    pub kind: TransitionKind,
    pub from: String,
    pub to: String,
    /// The way the motion travels: `"left"` means the content (or a wipe's edge) moves
    /// leftward, so the incoming element enters from the right. Required on `wipe`,
    /// `slide` and `push`; refused on `crossfade` (ADR-0150).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<Direction>,
    /// The keyframe vocabulary, applied to the window's progress. Omitted means `linear`;
    /// refused on `crossfade` (ADR-0150).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ease: Option<Ease>,
}

impl Transition {
    /// ADR-0150's two relational rules, which the derive cannot state.
    fn checked(self) -> Result<Self, String> {
        let kind = self.kind.as_str();
        match self.kind {
            TransitionKind::Crossfade => {
                for (field, present) in [
                    ("direction", self.direction.is_some()),
                    ("ease", self.ease.is_some()),
                ] {
                    if present {
                        return Err(format!(
                            "unknown field `{field}` on a `crossfade`: a crossfade travels \
                             nowhere and its ramp is linear, so `direction` and `ease` are \
                             fields of `wipe`, `slide` and `push` only (ADR-0150) — expected \
                             one of `kind`, `from`, `to`"
                        ));
                    }
                }
            }
            TransitionKind::Wipe | TransitionKind::Slide | TransitionKind::Push => {
                if self.direction.is_none() {
                    return Err(format!(
                        "missing field `direction`: a `{kind}` names the way its motion \
                         travels, one of `left`, `right`, `up`, `down` (ADR-0150)"
                    ));
                }
            }
        }
        Ok(self)
    }
}

// The two halves of `remote = "Self"`, as on `Effect`: the wire form is the derive's, and
// the parse adds [`Transition::checked`].
impl Serialize for Transition {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Transition::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for Transition {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Transition::deserialize(deserializer)?
            .checked()
            .map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TransitionKind {
    Crossfade,
    Wipe,
    Slide,
    Push,
}

impl TransitionKind {
    /// The word a document spells this kind with.
    pub fn as_str(self) -> &'static str {
        match self {
            TransitionKind::Crossfade => "crossfade",
            TransitionKind::Wipe => "wipe",
            TransitionKind::Slide => "slide",
            TransitionKind::Push => "push",
        }
    }
}

/// A `wipe`, `slide` or `push`'s direction: the way the motion travels, never the edge
/// something enters from (ADR-0150).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    /// The word a document spells this direction with.
    pub fn as_str(self) -> &'static str {
        match self {
            Direction::Left => "left",
            Direction::Right => "right",
            Direction::Up => "up",
            Direction::Down => "down",
        }
    }
}
