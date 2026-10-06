//! The ordered `effects` list and its closed v1 vocabulary.
//!
//! ADR-0040 settled the attachment — a list field on the element, order significant,
//! because *"blur-then-shadow is a different frame from shadow-then-blur"* — and named
//! `blur`, `shadow` and `mask`. ADR-0049 added four colour scalars. ADR-0068 retired the
//! bare `mask` key that the fixture carried, and fixed where the list sits in key order:
//! **`effects` appends after the type's existing fields.**
//!
//! # Two rules the derive cannot state
//!
//! ADR-0084 gives `mask` one shape-independent rect and a `radius` whose legality depends
//! on `shape`. Neither is expressible in a `#[derive(Deserialize)]` enum variant: one is a
//! relation between four optional fields, the other makes a declared field an *unknown key*
//! under two of three `shape` values. So the derive is generated against [`Effect`] as a
//! remote (`remote = "Self"`, serde's own name for "write the functions, not the impls")
//! and the trait impls below wrap it — the parse the derive produces, then
//! [`Effect::checked`]. There is no second enumeration of the vocabulary anywhere: the
//! members are declared once, here. ADR-0156 adds `grain`, `posterize`, `glow` and
//! `directional_blur`; the last three's ranges are the fourth rule [`Effect::checked`]
//! enforces.
//!
//! ADR-0088 adds a third rule of the same kind, on `chroma`: three of its four parameters
//! are bounded to `[0, 1]`, and a Rust `f64` field cannot say so. It joins the other two in
//! [`Effect::checked`], and is published in the schema by [`Fraction`] and [`ScreenColour`]
//! for the same reason the mask rules are — a schema admitting numbers this binary refuses
//! is the two-artifact divergence, arriving through under-statement.
//!
//! The same two rules are said again in the published schema, by
//! `crate::schema::publish_mask_rect`, for the reason ADR-0041 and #168 both give: a schema
//! that admitted files this binary refuses is the two-artifact divergence, arriving through
//! under-statement rather than through drift.

use schemars::JsonSchema;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{Animatable, Colour, Length};

/// One member of the closed vocabulary, discriminated by `name`.
///
/// Two effects of the same name are ordinary rather than forbidden (ADR-0040) — the
/// container is a list precisely so a second shadow has somewhere to go.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "name",
    rename_all = "snake_case",
    deny_unknown_fields,
    remote = "Self"
)]
pub enum Effect {
    /// Gaussian blur; one parameter.
    ///
    /// Every numeric and colour parameter of every member is an animatable property
    /// (ADR-0146 §3, slice 2): a literal, or a keyframe list written in place inside the
    /// member, whose `t` means what it means on the element.
    Blur { radius: Animatable<f64> },
    /// Drop shadow.
    Shadow {
        dx: Animatable<f64>,
        dy: Animatable<f64>,
        radius: Animatable<f64>,
        color: Animatable<Colour>,
        opacity: Animatable<f64>,
    },
    /// A shape mask, shape-only — no image source, no alpha or soft mask.
    ///
    /// **One rect, shared by all three shapes** (ADR-0084). `x`, `y`, `width`, `height`
    /// name the rect the shape is inscribed in — `circle` is the largest circle inscribed
    /// in it, `rect` is it, `ellipse` fills it — and `shape` selects which figure is drawn
    /// there, never which fields exist. The four are **all-or-none**, element-local
    /// integers measured from the element rect's top-left whatever the `origin` keyword
    /// is, and their identity value is the element's own rect: a bare
    /// `{"name": "mask", "shape": "circle"}` (ADR-0068's form) is the same declaration
    /// reached by the same arithmetic, not a legacy spelling beside this one.
    ///
    /// `radius` rounds the corners of a `rect` mask, identity `0`. On `circle` or
    /// `ellipse` it is an unknown key naming its reason, exactly as `radius` is on a drawn
    /// ellipse (ADR-0014): an inscribed ellipse has no corners to round.
    ///
    /// The mask is declared inside the element's box in unscaled units, so the element's
    /// `scale` grows it and its `rotation` turns it — the rule a `blur` radius and a
    /// `stroke_width` already follow.
    ///
    /// The rect and `radius` are animatable (ADR-0146): integer keyframe values that
    /// resolve unrounded. `width`, `height` and `radius` are lengths, so a negative one is a
    /// schema error and `0` is legal. A plain mask whose resolved `width` or `height` is at
    /// or below zero hides the whole element for that frame, which is how a reveal starts.
    Mask {
        shape: MaskShape,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        x: Option<Animatable<i64>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        y: Option<Animatable<i64>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        width: Option<Animatable<Length>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        height: Option<Animatable<Length>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        radius: Option<Animatable<Length>>,
        /// Keep the pixels **outside** the shape and erase the inside (ADR-0152 §1).
        /// Omitted means `false`; a written `false` is legal and the same picture. A
        /// boolean is not an animatable type (ADR-0146), so a keyframe list here is a
        /// schema error with no rule of its own. Masks in one list still intersect, so
        /// `[mask ellipse, mask smaller ellipse inverted]` keeps a ring.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        invert: Option<bool>,
        /// Soften the mask's edge (ADR-0152 §2): the hard coverage, 1 inside the shape and
        /// 0 outside, blurred by a Gaussian of σ = `feather` / 2 (`blur`'s own reading of a
        /// radius), centred on the edge. In unscaled element units, so it rides the
        /// transform. Omitted means `0`, the hard antialiased edge. An animatable
        /// property: a keyframe list of integers is written in place and resolves
        /// unrounded between its keys (ADR-0146, ADR-0035). A length, so a negative
        /// value is a schema error in a static value and in every keyframe record.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        feather: Option<Animatable<Length>>,
    },
    /// Pushes pixel colour toward `color` by `amount` (0–1).
    ///
    /// `color` is ADR-0049's sole grandfathered exception to *"parameters must be bounded
    /// scalars"*, and the exception is closed: a future colour operation does not get the
    /// same allowance by citing `tint` as precedent.
    Tint {
        color: Animatable<Colour>,
        amount: Animatable<f64>,
    },
    /// `0` = fully desaturated (a bare `grayscale`, which is deliberately not its own
    /// member), `1` = unchanged, `>1` = oversaturated.
    Saturation { amount: Animatable<f64> },
    /// Signed offset from unchanged at `0`.
    Brightness { amount: Animatable<f64> },
    /// Signed offset from unchanged at `0`.
    Contrast { amount: Animatable<f64> },
    /// Key a screen colour out of the element's pixels (ADR-0088).
    ///
    /// **A matte operation, not a colour one.** Its output is transparency: it decides
    /// which pixels survive, and the RGB it keeps is the RGB it was given. That is why
    /// ADR-0049's stopping rule — which opens *"a **colour operation** is admissible in v1
    /// only if…"* — does not reach the keying, and why `color` is a literal `#RRGGBB`
    /// rather than `tint.color`'s closed exception being cited a second time.
    ///
    /// `color` is the screen colour, in the one spelling ADR-0014 fixes for every colour in
    /// the format. Not `#RRGGBBAA`: an alpha on the key colour is meaningless and would be
    /// a second way to say nothing — [`Effect::checked`] refuses one.
    ///
    /// `tolerance` is the normalised distance in the chroma plane within which a pixel is
    /// keyed out, identity `0` — which keys **nothing**, and makes the whole member a
    /// no-op. `softness` is the width of the partial-alpha band at the edge of the key,
    /// identity `0` — a hard, binary matte. `spill` suppresses screen colour reflected onto
    /// the subject, identity `0`; it is the one parameter that changes RGB, and ADR-0088
    /// admits it on ADR-0049's four clauses directly rather than by any exception.
    ///
    /// All three are `0.0`–`1.0` in a static value and in every keyframe record, which
    /// [`Fraction`]'s schema states and [`Effect::checked`] enforces.
    ///
    /// All four animate (ADR-0146), `color` staying six-digit in every record, so a screen
    /// whose lighting drifts mid-take is followed by a keyed `tolerance` rather than by
    /// cutting the element at each drift. `measure`'s per-frame coverage reading is how the
    /// drift is found.
    Chroma {
        color: Animatable<ScreenColour>,
        tolerance: Animatable<Fraction>,
        softness: Animatable<Fraction>,
        spill: Animatable<Fraction>,
    },
    /// Film grain: a symmetric offset per colour channel, drawn per `size`×`size` cell by a
    /// fixed integer hash of the written `seed` and the element's local frame, so it
    /// re-rolls on every output frame (ADR-0156 §3, §4).
    ///
    /// `amount` (`0`–`1`) is the one animatable parameter. `seed`, `size` and `mono` are
    /// static: none is typed `Animatable`, so a keyframe list on any of them is a schema
    /// error and none joins the one derived list of animatable properties (ADR-0146).
    Grain {
        seed: Seed,
        amount: Animatable<Fraction>,
        size: GrainSize,
        mono: bool,
    },
    /// Quantise each colour channel to `levels` steps (ADR-0156 §4).
    ///
    /// On non-premultiplied sRGB values from 0 to 1, `q = round(v × (levels − 1)) /
    /// (levels − 1)`, ties away from zero, alpha untouched: `levels: 256` is the identity on
    /// 8-bit values. A keyed `levels` resolves to a continuous value, which the painter rounds
    /// half away from zero, the rule `shift` uses.
    Posterize { levels: Animatable<Levels> },
    /// A threshold bloom (ADR-0156 §4): the pixels brighter than `threshold` (Rec.709 luma),
    /// blurred by `radius` read as `blur` reads it, times `intensity`, added over the element
    /// inside its own layer. There is no `color`: a coloured alpha glow is a zero-offset
    /// `shadow`.
    Glow {
        threshold: Animatable<Fraction>,
        radius: Animatable<Reach>,
        intensity: Animatable<Gain>,
    },
    /// A centred smear along one direction (ADR-0156 §4). `angle` in degrees, `0`
    /// horizontal and clockwise positive, as `rotation` is, measured in the element's own
    /// space; `length` the whole smear in element pixels. Not motion blur: it smears whether
    /// or not the element moves.
    DirectionalBlur {
        angle: Animatable<f64>,
        length: Animatable<Smear>,
    },
}

/// Declares a bounded number type for one named-effect parameter (ADR-0156 §4), the way
/// [`Fraction`] is declared for `chroma`: the type states the bound in the published schema,
/// so it reaches the static value and every keyframe record alike, and so the one derived
/// list of animatable properties reads it as the range a resolved value clamps to (ADR-0146
/// §5). [`Effect::checked`] enforces it with a message naming the parameter.
macro_rules! bounded {
    ($(#[$doc:meta])* $name:ident($inner:ty), $type:literal, $min:expr, $max:expr, $description:literal) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub $inner);

        impl $name {
            /// The bound, inclusive at both ends where it has two.
            pub const RANGE: (f64, Option<f64>) = ($min, $max);

            fn value(self) -> f64 {
                self.0 as f64
            }
        }

        impl JsonSchema for $name {
            fn schema_name() -> std::borrow::Cow<'static, str> {
                stringify!($name).into()
            }

            fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
                // An integer type states integer bounds, as `Length` does.
                let bound = |value: f64| {
                    if $type == "integer" {
                        serde_json::json!(value as i64)
                    } else {
                        serde_json::json!(value)
                    }
                };
                let mut schema = serde_json::json!({
                    "type": $type,
                    "description": $description,
                    "minimum": bound($min),
                });
                if let Some(max) = $max {
                    schema["maximum"] = bound(max);
                }
                schemars::Schema::try_from(schema).expect("an object literal is a schema")
            }
        }

        impl crate::resolve::Interpolate for $name {
            type Out = f64;

            fn between(a: &Self, b: &Self, p: f64) -> f64 {
                f64::between(&a.value(), &b.value(), p)
            }

            fn held(value: &Self) -> f64 {
                value.value()
            }
        }
    };
}

bounded!(
    /// `posterize`'s `levels`: an integer from 2 to 256 (ADR-0156 §4).
    Levels(i64),
    "integer",
    2.0,
    Some(256.0),
    "How many steps each colour channel is quantised to, an integer from `2` to `256` in a \
     static value and in every keyframe record. `256` is the identity (ADR-0156)."
);

bounded!(
    /// `glow`'s `radius`: a number of pixels, at least 0, read as `blur`'s (ADR-0156 §4).
    Reach(f64),
    "number",
    0.0,
    None::<f64>,
    "The glow's blur radius in element pixels, at least `0`, read exactly as `blur` reads \
     its `radius` (σ = radius / 2) (ADR-0156)."
);

bounded!(
    /// `glow`'s `intensity`: the gain on the glowing part, from 0 to 4 (ADR-0156 §4).
    Gain(f64),
    "number",
    0.0,
    Some(4.0),
    "The gain on the glowing part, from `0` to `4` in a static value and in every keyframe \
     record. `0` adds nothing (ADR-0156)."
);

bounded!(
    /// `directional_blur`'s `length`: the whole smear in element pixels, from 0 to
    /// [`Smear::MAX`] (ADR-0156 §4).
    Smear(f64),
    "number",
    0.0,
    Some(Smear::MAX),
    "The whole smear in element pixels, centred on each pixel, from `0` (the identity) to \
     `4096` in a static value and in every keyframe record. It takes `ceil(length) + 1` \
     samples, so a keyed `length` steps the sample count (ADR-0156)."
);

impl Smear {
    /// The longest smear: `ceil(length) + 1` samples, and the painter's loop takes at most
    /// 4,097 (the prototype's cap, #722).
    pub const MAX: f64 = 4096.0;
}

/// `grain`'s `seed`: an integer from `0` to `2³¹ − 1`, never keyed (ADR-0156 §4).
///
/// Its own deserializer, so a keyframe list, a fraction or a value out of range is refused
/// by a sentence naming the key and saying it is static, rather than by serde's bare type
/// error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Seed(pub u32);

/// The largest `seed`: `2³¹ − 1`, so a seed fits every signed 32-bit reader.
pub const SEED_MAX: u32 = i32::MAX as u32;

impl<'de> Deserialize<'de> for Seed {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let written = serde_json::Value::deserialize(deserializer)?;
        written
            .as_u64()
            .and_then(|seed| u32::try_from(seed).ok())
            .filter(|seed| *seed <= SEED_MAX)
            .map(Seed)
            .ok_or_else(|| {
                D::Error::custom(format!(
                    "`grain`'s `seed` is {written}: a seed is a literal integer from 0 to \
                     2147483647, and it is never keyed — change it to change the pattern \
                     (ADR-0156)"
                ))
            })
    }
}

impl JsonSchema for Seed {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Seed".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::Schema::try_from(serde_json::json!({
            "type": "integer",
            "minimum": 0,
            "maximum": SEED_MAX,
            "description": "A literal integer from `0` to `2147483647` that fixes every draw. \
                            Static: a keyframe list is a schema error (ADR-0156).",
        }))
        .expect("an object literal is a schema")
    }
}

/// `grain`'s `size`: the side of one cell in unscaled element units, `1` to `8`, never keyed
/// (ADR-0156 §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct GrainSize(pub u8);

/// The largest `grain` cell.
pub const GRAIN_SIZE_MAX: u8 = 8;

impl<'de> Deserialize<'de> for GrainSize {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let written = serde_json::Value::deserialize(deserializer)?;
        written
            .as_u64()
            .filter(|size| (1..=u64::from(GRAIN_SIZE_MAX)).contains(size))
            .map(|size| GrainSize(size as u8))
            .ok_or_else(|| {
                D::Error::custom(format!(
                    "`grain`'s `size` is {written}: a cell's side is a literal integer from 1 \
                     to 8, in element units, and it is never keyed (ADR-0156)"
                ))
            })
    }
}

impl JsonSchema for GrainSize {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "GrainSize".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::Schema::try_from(serde_json::json!({
            "type": "integer",
            "minimum": 1,
            "maximum": GRAIN_SIZE_MAX,
            "description": "The side of one grain cell in unscaled element units, `1` to `8`. \
                            Static: a keyframe list is a schema error (ADR-0156).",
        }))
        .expect("an object literal is a schema")
    }
}

/// A number from `0` to `1` inclusive: `chroma`'s three normalised scalars (ADR-0088).
///
/// The type states the bound in the published schema, so it reaches the static value and
/// every keyframe record alike, and so the one derived list of animatable properties reads
/// it as the range a resolved value clamps to and a `shift` split may not leave (ADR-0146
/// §5, §7). It is enforced by [`Effect::checked`], whose message names the parameter: a
/// bare number here cannot know which of the three it is.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Fraction(pub f64);

impl JsonSchema for Fraction {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Fraction".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::Schema::try_from(serde_json::json!({
            "type": "number",
            "format": "double",
            "description": "A normalised number from `0` to `1` inclusive, in a static value \
                            and in every keyframe record (ADR-0088, ADR-0146).",
            "minimum": 0.0,
            "maximum": 1.0,
        }))
        .expect("an object literal is a schema")
    }
}

impl crate::resolve::Interpolate for Fraction {
    type Out = f64;

    fn between(a: &Self, b: &Self, p: f64) -> f64 {
        f64::between(&a.0, &b.0, p)
    }

    fn held(value: &Self) -> f64 {
        value.0
    }
}

/// `chroma`'s screen colour: the format's one [`Colour`], narrowed to `#RRGGBB` (ADR-0088).
///
/// Narrowed rather than replaced, so it is still read, written and blended as any colour is
/// (ADR-0146 §2); a blend of two opaque colours is opaque, so a keyed screen colour never
/// gains an alpha between its records. Enforced by [`Effect::checked`], in every record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ScreenColour(pub Colour);

impl JsonSchema for ScreenColour {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "ScreenColour".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::Schema::try_from(serde_json::json!({
            "type": "string",
            "allOf": [generator.subschema_for::<Colour>()],
            "pattern": "^#[0-9A-F]{6}$",
            "description": "The screen colour, `#RRGGBB`. Not `#RRGGBBAA`: an alpha on the \
                            key colour is meaningless and would be a second way to say \
                            nothing — a key colour names a colour to find in the frame, \
                            never one to composite (ADR-0088, ADR-0014).",
        }))
        .expect("an object literal is a schema")
    }
}

impl crate::resolve::Interpolate for ScreenColour {
    type Out = crate::resolve::Blend;

    fn between(a: &Self, b: &Self, p: f64) -> crate::resolve::Blend {
        Colour::between(&a.0, &b.0, p)
    }

    fn held(value: &Self) -> crate::resolve::Blend {
        Colour::held(&value.0)
    }
}

/// Every value an animatable parameter states: its static value, or each record's `v`.
fn stated<T>(property: &Animatable<T>) -> Vec<&T> {
    match property {
        Animatable::Static(value) => vec![value],
        Animatable::Keyed(records) => records.iter().map(|record| &record.v).collect(),
    }
}

/// The word a document spells ADR-0088's keyer with.
///
/// Three places outside this module ask *"is this member the key"* — the check, `measure`'s
/// dispatch and the schema pass that publishes its bounds — and each asks it of a permissive
/// `Value` rather than of a parsed [`Effect`], where the variant would have answered. A
/// literal in each would be three spellings of one name, which is [`MASK_RECT`]'s objection
/// applied to a word instead of to a list.
pub(crate) const CHROMA: &str = "chroma";

/// The four **Colour filter** members' names (ADR-0049), as a document spells them.
///
/// Named here, beside the vocabulary, because `crate::checks::chroma` asks *"is a colour
/// operation listed ahead of this key"* and that question needs the sub-family rather than
/// the whole list. A copy in the check would be a second answer to "which members change
/// pixel colour" — the sub-family is load-bearing under ADR-0088, which turns on the
/// distinction between a colour operation and a matte one, so it is declared once.
///
/// `shadow` is not among them and neither is `chroma`'s own `spill`: `shadow` paints behind
/// the element rather than changing its pixels, and `spill` is a parameter of the key
/// itself, applied after the distance it is conditioned on has been measured.
pub(crate) const COLOUR_FILTERS: [&str; 4] = ["tint", "saturation", "brightness", "contrast"];

/// The four names of the mask rect, in ADR-0084's canonical order.
///
/// One array rather than four literals, because the all-or-none message has to name the
/// *other* three and a second listing would be a second answer to "which four".
pub(crate) const MASK_RECT: [&str; 4] = ["x", "y", "width", "height"];

/// `chroma`'s three bounded scalars, in the order ADR-0088 declares them.
///
/// One array rather than three literals for [`MASK_RECT`]'s reason: the bound is one rule
/// about three fields, and [`Fraction`] publishes it on the same three. A second listing would be a second answer to "which parameters are bounded".
pub(crate) const CHROMA_SCALARS: [&str; 3] = ["tolerance", "softness", "spill"];

impl Effect {
    /// This effect, if the three relational rules the derive cannot state hold of it — or
    /// the sentence saying which one does not.
    ///
    /// Two are ADR-0084's, about `mask`, and neither can be a field's own type: one relates
    /// four optional fields to each other, the other makes a declared field unknown under
    /// two of three `shape` values. The third is ADR-0088's, about `chroma`: its three
    /// scalars are bounded to `[0, 1]` and its `color` is `#RRGGBB` with no alpha, and an
    /// `f64` field and a [`Colour`] say neither.
    fn checked(self) -> Result<Self, String> {
        // ADR-0156 §4: `amount` runs from 0 to 1, in a static value and in every record.
        if let Effect::Grain { amount, .. } = &self {
            if let Some(Fraction(value)) = stated(amount)
                .into_iter()
                .find(|Fraction(value)| !(0.0..=1.0).contains(value))
            {
                return Err(format!(
                    "`grain`'s `amount` is {value}: the offset per channel runs from 0 to 1, \
                     with its identity at 0 (ADR-0156)"
                ));
            }
            return Ok(self);
        }

        // ADR-0156 §5: every range of the named effects is a schema error, in a static value
        // and in every keyframe record, and the message names the member and the parameter.
        let named: Vec<Ranged> = match &self {
            Effect::Posterize { levels } => vec![(
                "posterize",
                "levels",
                stated(levels).into_iter().map(|v| v.value()).collect(),
                Levels::RANGE,
            )],
            Effect::Glow {
                threshold,
                radius,
                intensity,
            } => vec![
                (
                    "glow",
                    "threshold",
                    stated(threshold).into_iter().map(|v| v.0).collect(),
                    (0.0, Some(1.0)),
                ),
                (
                    "glow",
                    "radius",
                    stated(radius).into_iter().map(|v| v.value()).collect(),
                    Reach::RANGE,
                ),
                (
                    "glow",
                    "intensity",
                    stated(intensity).into_iter().map(|v| v.value()).collect(),
                    Gain::RANGE,
                ),
            ],
            Effect::DirectionalBlur { length, .. } => vec![(
                "directional_blur",
                "length",
                stated(length).into_iter().map(|v| v.value()).collect(),
                Smear::RANGE,
            )],
            _ => Vec::new(),
        };
        for (member, parameter, values, (min, max)) in named {
            if let Some(value) = values
                .into_iter()
                .find(|value| !(*value >= min && max.is_none_or(|max| *value <= max)))
            {
                let range = match max {
                    Some(max) => format!("from `{min}` to `{max}`"),
                    None => format!("at least `{min}`"),
                };
                return Err(format!(
                    "`{member}`'s `{parameter}` is {value}: it must be {range}, in a static \
                     value and in every keyframe record (ADR-0156)"
                ));
            }
        }

        if let Effect::Chroma {
            color,
            tolerance,
            softness,
            spill,
        } = &self
        {
            // Bounded, and the message says what the bound is *for* — a distance and two
            // widths, all normalised — rather than only that the number is out of range.
            // In a static value and in every keyframe record alike (ADR-0146 §5).
            for (name, property) in CHROMA_SCALARS.iter().zip([tolerance, softness, spill]) {
                if let Some(Fraction(value)) = stated(property)
                    .into_iter()
                    .find(|Fraction(value)| !(0.0..=1.0).contains(value))
                {
                    return Err(format!(
                        "`chroma`'s `{name}` is {value}: the key's three scalars are \
                         normalised and must be within `[0, 1]`, each with its \
                         identity at `0` (ADR-0088)"
                    ));
                }
            }
            // ADR-0088: *"Not `#RRGGBBAA`: an alpha on the key colour is meaningless and
            // would be a second way to say nothing."* `Colour` admits both spellings —
            // `shadow.color` and every `fill` may carry an alpha — so the narrowing is
            // this member's own, and it is refused rather than ignored (ADR-0007: a field
            // the renderer cannot honour is worse than no field).
            //
            // **Length is the whole question, because `Colour` has already answered every
            // other part of it**: its own deserializer fixes the leading `#`, uppercase hex
            // digits and exactly one of two arities (ADR-0014), and refuses `#RRGGBBFF` as a
            // second spelling of the six-digit form. So the only colour that can reach here
            // and still be wrong is a genuinely translucent one, and that is what this
            // measures. The published `^#[0-9A-F]{6}$` says the same thing to a reader who
            // has no `Colour` to lean on.
            if let Some(ScreenColour(written)) = stated(color)
                .into_iter()
                .find(|ScreenColour(written)| written.as_str().len() != "#RRGGBB".len())
            {
                let written = written.as_str();
                return Err(format!(
                    "`chroma`'s `color` is {written}: the screen colour is `#RRGGBB`, and an \
                     alpha on it is meaningless — a key colour names a colour to \
                     find in the frame, never one to composite (ADR-0088, \
                     ADR-0014)"
                ));
            }
            return Ok(self);
        }

        let Effect::Mask {
            shape,
            x,
            y,
            width,
            height,
            radius,
            invert: _,
            feather: _,
        } = &self
        else {
            return Ok(self);
        };

        // All-or-none, named in both directions — what was written and what it still needs.
        // "The four are all-or-none" on its own leaves the author counting.
        let present: Vec<&str> = MASK_RECT
            .iter()
            .zip([x.is_some(), y.is_some(), width.is_some(), height.is_some()])
            .filter(|(_, written)| *written)
            .map(|(name, _)| *name)
            .collect();
        if !present.is_empty() && present.len() != MASK_RECT.len() {
            let missing: Vec<&str> = MASK_RECT
                .iter()
                .copied()
                .filter(|name| !present.contains(name))
                .collect();
            return Err(format!(
                "`mask` states {} and not {}: the mask rect's four fields are all-or-none \
                 — write all of `x`, `y`, `width`, `height`, or none of them and take the \
                 element's own rect (ADR-0084)",
                quoted(&present),
                quoted(&missing),
            ));
        }

        // `radius` is a field of `shape: "rect"` only. On the other two it is an unknown
        // key that says why, rather than one silently ignored (ADR-0084, ADR-0014).
        //
        // **Phrased in serde's own unknown-field form, deliberately.** ADR-0084 does not
        // merely call it an error: it says the key "is an **unknown key** … following
        // ADR-0014's rule for the same word on the drawn `shape` element", and CONTEXT.md
        // promises it behaves "exactly as it is on a drawn ellipse". A drawn ellipse's
        // `radius` is refused by the derive, so it reads `unknown field \`radius\`,
        // expected one of …` and `crate::checks::schema` classifies it as
        // `E-SCHEMA-UNKNOWN-KEY` — which is what carries ADR-0016's guarantee text ("it may
        // belong to a newer format revision … do not delete the key to make the file
        // validate"). A message of this check's own devising would have been `E-SCHEMA`
        // instead, and the two words would have named two different reports.
        //
        // The reason sits between the two markers that check reads, so it is in front of
        // anyone holding the raw parse error while the finding stays the shared one.
        if radius.is_some() && *shape != MaskShape::Rect {
            return Err(format!(
                "unknown field `radius` on a `{shape}` mask: an inscribed {shape} has no \
                 corners to round, so `radius` is a field of `shape: \"rect\"` only \
                 (ADR-0084, ADR-0014) — expected one of `name`, `shape`, {rect}",
                shape = shape.as_str(),
                rect = quoted(&MASK_RECT),
            ));
        }

        Ok(self)
    }
}

/// One named-effect parameter's stated values beside its range: the member, the parameter,
/// every value it states, and the bound, inclusive, with no upper end where it has none.
type Ranged = (&'static str, &'static str, Vec<f64>, (f64, Option<f64>));

/// `` `x`, `width` `` — a list of field names, for a sentence.
fn quoted(names: &[&str]) -> String {
    names
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

// The two halves of `remote = "Self"`: the derive above wrote `Effect::serialize` and
// `Effect::deserialize` as inherent functions, and these are the impls that call them. The
// wire form is entirely the derive's — the only thing added is the parse-time check.
impl Serialize for Effect {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Effect::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for Effect {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Effect::deserialize(deserializer)?
            .checked()
            .map_err(D::Error::custom)
    }
}

/// The figure an [`Effect::Mask`] draws in its rect (ADR-0084).
///
/// Not a schema discriminator: all three shapes take the same fields, and `shape` says
/// which figure is cut in them. ADR-0049's two-level-lookup objection is why — an agent
/// that has learned `mask` has learned all of `mask`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum MaskShape {
    Circle,
    Rect,
    Ellipse,
}

impl MaskShape {
    /// The word a document spells this shape with — the one a message has to use.
    pub fn as_str(self) -> &'static str {
        match self {
            MaskShape::Circle => "circle",
            MaskShape::Rect => "rect",
            MaskShape::Ellipse => "ellipse",
        }
    }
}
