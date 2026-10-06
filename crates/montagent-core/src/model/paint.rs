//! **Paint**: what a paint field holds — a flat colour, a keyframe list of colours, or a
//! gradient (ADR-0149).
//!
//! Four fields take a paint: a `rect`'s and an `ellipse`'s `fill` and `stroke`, and a `text`
//! element's element-level `color` and `stroke`. Every other colour field — run and
//! highlight paint, the project `background`, the effect colours — stays a [`Colour`].
//!
//! # Animated in place
//!
//! A gradient's `angle`, `center`, `radius` and `stops` are each an [`Animatable`] of their
//! own type (ADR-0149 §3); the kind never animates, and a paint field's keyframe list holds
//! colours, never a gradient. What a gradient *is* at an instant is a [`ResolvedGradient`],
//! built by [`Gradient::resolve`], which every reader goes through.
//!
//! # Telling a stop list from a keyframe list
//!
//! A stop list is an array of objects, and [`Animatable`]'s shape rule reads any array of
//! objects as keyframes. ADR-0149 does not settle it, so the rule is stated at
//! [`super::is_keyframe_list`]: **a `stops` array is a keyframe list when its first item carries a
//! `t`**, and a stop list otherwise. A stop has no `t`, so the two never overlap.

use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Number, Value, json};

use super::{Animatable, Colour};
use crate::resolve::{self, Blend, Interpolate, Unresolvable};

/// A paint field's value: a colour, a keyframe list of colours, or a gradient (ADR-0149 §3).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Paint {
    /// A colour, or a keyframe list of colours (ADR-0146).
    Flat(Animatable<Colour>),
    /// A gradient object.
    Gradient(Gradient),
}

impl<'de> Deserialize<'de> for Paint {
    /// Chosen on shape, as [`Animatable`] is, so a fault inside a gradient is reported as
    /// that fault rather than as "matched no variant".
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        if value.is_object() {
            return serde_json::from_value(value)
                .map(Paint::Gradient)
                .map_err(D::Error::custom);
        }
        let holds_a_gradient = value.as_array().is_some_and(|records| {
            records
                .iter()
                .any(|record| record.get("v").is_some_and(Value::is_object))
        });
        if holds_a_gradient {
            return Err(D::Error::custom(
                "a paint's keyframe list holds colours, never a gradient: write the gradient \
                 object itself as the field's value and key its `angle`, `center`, `radius` or \
                 `stops` (ADR-0149)",
            ));
        }
        serde_json::from_value(value)
            .map(Paint::Flat)
            .map_err(D::Error::custom)
    }
}

impl JsonSchema for Paint {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Paint".into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        let flat = generator.subschema_for::<Animatable<Colour>>();
        let gradient = generator.subschema_for::<Gradient>();
        Schema::try_from(json!({
            "anyOf": [flat, gradient],
            "description": "A paint: a colour, a keyframe list of colours, or a gradient \
                            object. A keyframe list never holds a gradient; a gradient's \
                            `angle`, `center`, `radius` and `stops` may each be keyed \
                            (ADR-0149).",
        }))
        .expect("an object literal is a schema")
    }
}

/// A `linear` or `radial` gradient, measured against the element's declared box (ADR-0149
/// §1, §2). Every parameter is required: a wrong direction, centre or size gives a frame that
/// looks plausible, so none of them may default. Each takes a literal or a keyframe list
/// (§3); the kind is fixed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "gradient", rename_all = "lowercase", deny_unknown_fields)]
pub enum Gradient {
    /// Along a line through the box's centre.
    Linear {
        /// Degrees, CSS's convention: `0` runs bottom to top, `90` left to right, turning
        /// clockwise. Any number: `370` is `10`.
        angle: Animatable<Angle>,
        stops: Animatable<Stops>,
    },
    /// Out from a centre, as a circle in box fractions — an ellipse on a box that is not
    /// square.
    Radial {
        /// `[fx, fy]`, fractions of the box from its top-left corner. Any numbers: a glow
        /// may be centred outside the box.
        center: Animatable<Center>,
        /// A fraction of the distance from the centre to the box's farthest corner, in box
        /// fractions: `1` just reaches it. At or below `0` paints the last stop's colour
        /// over the box.
        radius: Animatable<Radius>,
        stops: Animatable<Stops>,
    },
}

impl Gradient {
    /// What this gradient is at `numerator / denominator` ms: every parameter resolved, then
    /// ADR-0149 §4's fix applied to the stops — each offset clamped to 0..1 and raised to the
    /// largest before it, a raised stop keeping its own colour — so the result is always a
    /// legal literal, and what is printed is what is drawn.
    pub fn resolve(
        &self,
        numerator: i128,
        denominator: i128,
    ) -> Result<ResolvedGradient, Unresolvable> {
        let settled = |list: &Animatable<Stops>| {
            resolve::at_instant(list, numerator, denominator).map(StopsBlend::settle)
        };
        Ok(match self {
            Gradient::Linear { angle, stops } => ResolvedGradient::Linear {
                angle: Angle(number_of(resolve::at_instant(
                    angle,
                    numerator,
                    denominator,
                )?)),
                stops: settled(stops)?,
            },
            Gradient::Radial {
                center,
                radius,
                stops,
            } => {
                let [fx, fy] = resolve::at_instant(center, numerator, denominator)?;
                ResolvedGradient::Radial {
                    center: Center([number_of(fx), number_of(fy)]),
                    radius: Radius(number_of(resolve::at_instant(
                        radius,
                        numerator,
                        denominator,
                    )?)),
                    stops: settled(stops)?,
                }
            }
        })
    }
}

/// A gradient at one instant: every parameter a literal, the stops fixed by §4. Serialized as
/// the document would write it, so it pastes back.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "gradient", rename_all = "lowercase")]
pub enum ResolvedGradient {
    Linear {
        angle: Angle,
        stops: Stops,
    },
    Radial {
        center: Center,
        radius: Radius,
        stops: Stops,
    },
}

impl ResolvedGradient {
    /// `linear` or `radial`, as the document spells it.
    pub fn kind(&self) -> &'static str {
        match self {
            ResolvedGradient::Linear { .. } => "linear",
            ResolvedGradient::Radial { .. } => "radial",
        }
    }

    pub fn stops(&self) -> &[Stop] {
        match self {
            ResolvedGradient::Linear { stops, .. } | ResolvedGradient::Radial { stops, .. } => {
                &stops.0
            }
        }
    }

    /// Whether this gradient paints a single colour over the whole box: its stops share one
    /// colour, or it is a radial whose `radius` is at or below 0 (ADR-0149 §4, §5).
    pub fn one_colour(&self) -> bool {
        if let ResolvedGradient::Radial { radius, .. } = self
            && radius.fraction() <= 0.0
        {
            return true;
        }
        let stops = self.stops();
        stops
            .iter()
            .all(|stop| stop.color.bytes() == stops[0].color.bytes())
    }
}

/// One colour stop: where along the gradient, and what colour. `color`'s alpha is the
/// stop's opacity — there are no separate opacity stops (ADR-0149 §1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Stop {
    pub offset: Offset,
    pub color: Colour,
}

/// At least two stops, in order. No upper limit (ADR-0149 §1).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Stops(pub Vec<Stop>);

impl<'de> Deserialize<'de> for Stops {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let stops = Vec::<Stop>::deserialize(deserializer)?;
        if stops.len() < 2 {
            return Err(D::Error::custom(format!(
                "`stops` holds {} stop{}: a gradient needs at least two (ADR-0149); for one \
                 colour, write the colour itself",
                stops.len(),
                if stops.len() == 1 { "" } else { "s" }
            )));
        }
        Ok(Stops(stops))
    }
}

impl JsonSchema for Stops {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Stops".into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        let stop = generator.subschema_for::<Stop>();
        Schema::try_from(json!({
            "type": "array",
            "items": stop,
            "minItems": 2,
            "description": "At least two stops. Offsets never decrease (`E-GRADIENT-STOP-ORDER`); \
                            equal offsets are legal and give a hard edge. As a keyframe list, \
                            every record's `v` holds the same number of stops \
                            (`E-GRADIENT-STOP-COUNT`) and stop i blends with stop i.",
        }))
        .expect("an object literal is a schema")
    }
}

/// A stop list on its way between two keyframes: each offset as a number and each colour as
/// a premultiplied [`Blend`], neither yet clamped (ADR-0149 §3, §4).
///
/// Unclamped on purpose, as [`Blend`] is: a bezier that overshoots can carry an offset past
/// its neighbour, and `shift` must see that it did, so it refuses to write a split there
/// rather than writing the fixed list into a literal that bends the curve past the cut.
///
/// Each stop carries, beside its blend, the colour exactly as written where the list is held
/// rather than blended — a static list, or either end of a keyframe list — because a
/// premultiplied blend at alpha 0 has no hue left, and a literal must print as it was written.
#[derive(Debug, Clone, PartialEq)]
pub struct StopsBlend(pub Vec<(f64, Blend, Option<Colour>)>);

impl StopsBlend {
    /// The first thing that stops this list being written as it is, as a sentence: an offset
    /// outside 0..1, an offset before the one ahead of it, or a colour outside its range.
    pub fn first_fault(&self) -> Option<String> {
        let mut previous = f64::NEG_INFINITY;
        for (index, (offset, blend, _)) in self.0.iter().enumerate() {
            let stop = index + 1;
            if !(0.0..=1.0).contains(offset) {
                return Some(format!(
                    "stop {stop} at offset {}, outside 0 to 1",
                    round6(*offset)
                ));
            }
            if *offset < previous {
                return Some(format!(
                    "stop {stop} at offset {}, before stop {index} at {}",
                    round6(*offset),
                    round6(previous)
                ));
            }
            if !blend.in_range() {
                let [r, g, b, a] = blend.0.map(round6);
                return Some(format!(
                    "stop {stop} with a colour outside its range (premultiplied red {r}, \
                     green {g}, blue {b}, alpha {a})"
                ));
            }
            previous = *offset;
        }
        None
    }

    /// ADR-0149 §4's fix: each offset clamped to 0..1, then, in list order, raised to the
    /// largest offset before it. A raised stop keeps its own colour, so a crossing is a hard
    /// edge rather than a failure. Colours settle as ADR-0146 §2 says.
    pub fn settle(self) -> Stops {
        let mut floor = 0.0_f64;
        Stops(
            self.0
                .into_iter()
                .map(|(offset, blend, written)| {
                    let offset = offset.clamp(0.0, 1.0).max(floor);
                    floor = offset;
                    Stop {
                        offset: Offset(number_of(offset)),
                        color: written.unwrap_or_else(|| blend.settle()),
                    }
                })
                .collect(),
        )
    }
}

fn round6(v: f64) -> f64 {
    (v * 1_000_000.0).round() / 1_000_000.0
}

impl Interpolate for Stops {
    type Out = StopsBlend;

    /// Stop *i* against stop *i*. A list whose count differs from its neighbour's is
    /// `E-GRADIENT-STOP-COUNT`'s to report; here the stops the two share are blended.
    fn between(a: &Self, b: &Self, p: f64) -> StopsBlend {
        StopsBlend(
            a.0.iter()
                .zip(&b.0)
                .map(|(a, b)| {
                    (
                        f64::between(&a.offset.fraction(), &b.offset.fraction(), p),
                        Colour::between(&a.color, &b.color, p),
                        None,
                    )
                })
                .collect(),
        )
    }

    fn held(value: &Self) -> StopsBlend {
        StopsBlend(
            value
                .0
                .iter()
                .map(|stop| {
                    (
                        stop.offset.fraction(),
                        Colour::held(&stop.color),
                        Some(stop.color.clone()),
                    )
                })
                .collect(),
        )
    }
}

/// A number kept exactly as the document spells it, so a tool that re-emits the file —
/// `shift`, `fmt` — writes `30` back as `30` and `0.5` as `0.5`.
fn literal<'de, D: Deserializer<'de>>(value: Value) -> Result<Number, D::Error> {
    match value {
        Value::Number(number) if number.as_f64().is_some() => Ok(number),
        other => Err(D::Error::custom(format!(
            "invalid type: {other}, expected a number"
        ))),
    }
}

/// A resolved number as the document would write it: a whole value without a fraction
/// (`45`, not `45.0`), any other in its shortest exact form, which pastes back to the same
/// bytes.
fn number_of(value: f64) -> Number {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        Number::from(value as i64)
    } else {
        Number::from_f64(value).unwrap_or_else(|| Number::from(0))
    }
}

fn number_schema(description: &str, bounds: Value) -> Schema {
    let mut schema = json!({"type": "number", "format": "double", "description": description});
    if let (Some(schema), Some(bounds)) = (schema.as_object_mut(), bounds.as_object()) {
        schema.extend(bounds.clone());
    }
    Schema::try_from(schema).expect("an object literal is a schema")
}

fn value_of(number: &Number) -> f64 {
    number.as_f64().unwrap_or(0.0)
}

/// A linear gradient's direction in degrees. Any number.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Angle(Number);

impl Angle {
    pub fn degrees(&self) -> f64 {
        value_of(&self.0)
    }
}

impl<'de> Deserialize<'de> for Angle {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        literal::<D>(Value::deserialize(deserializer)?).map(Angle)
    }
}

impl Interpolate for Angle {
    type Out = f64;

    fn between(a: &Self, b: &Self, p: f64) -> f64 {
        f64::between(&a.degrees(), &b.degrees(), p)
    }

    fn held(value: &Self) -> f64 {
        value.degrees()
    }
}

impl JsonSchema for Angle {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Angle".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        number_schema(
            "Degrees, CSS's convention: `0` runs bottom to top, `90` left to right, turning \
             clockwise. Any number.",
            json!({}),
        )
    }
}

/// A radial gradient's centre, `[fx, fy]` in box fractions. Any numbers.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Center([Number; 2]);

impl Center {
    pub fn fractions(&self) -> [f64; 2] {
        [value_of(&self.0[0]), value_of(&self.0[1])]
    }
}

impl<'de> Deserialize<'de> for Center {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let pair = Vec::<Value>::deserialize(deserializer)?;
        let [fx, fy]: [Value; 2] = pair.try_into().map_err(|pair: Vec<Value>| {
            D::Error::custom(format!(
                "`center` has {} numbers: it is `[fx, fy]`, two fractions of the box",
                pair.len()
            ))
        })?;
        Ok(Center([literal::<D>(fx)?, literal::<D>(fy)?]))
    }
}

impl Interpolate for Center {
    type Out = [f64; 2];

    fn between(a: &Self, b: &Self, p: f64) -> [f64; 2] {
        <[f64; 2]>::between(&a.fractions(), &b.fractions(), p)
    }

    fn held(value: &Self) -> [f64; 2] {
        value.fractions()
    }
}

impl JsonSchema for Center {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Center".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        Schema::try_from(json!({
            "type": "array",
            "items": {"type": "number", "format": "double"},
            "minItems": 2,
            "maxItems": 2,
            "description": "`[fx, fy]`, fractions of the declared box from its top-left \
                            corner. Any numbers: a centre may sit outside the box.",
        }))
        .expect("an object literal is a schema")
    }
}

/// A radial gradient's size, a fraction of the centre-to-farthest-corner distance. `≥ 0` as
/// written; a resolved value that a bezier carries to or below 0 paints the last stop's
/// colour over the box.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Radius(Number);

impl Radius {
    pub fn fraction(&self) -> f64 {
        value_of(&self.0)
    }
}

impl<'de> Deserialize<'de> for Radius {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let radius = literal::<D>(Value::deserialize(deserializer)?)?;
        if value_of(&radius) < 0.0 {
            return Err(D::Error::custom(format!(
                "a gradient's `radius` is {radius}: it is a fraction of the distance to the \
                 box's farthest corner, `0` or more (ADR-0149)"
            )));
        }
        Ok(Radius(radius))
    }
}

impl Interpolate for Radius {
    type Out = f64;

    fn between(a: &Self, b: &Self, p: f64) -> f64 {
        f64::between(&a.fraction(), &b.fraction(), p)
    }

    fn held(value: &Self) -> f64 {
        value.fraction()
    }
}

impl JsonSchema for Radius {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "GradientRadius".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        number_schema(
            "A fraction of the distance from the centre to the declared box's farthest \
             corner, in box fractions: `1` just reaches it. At or below `0` paints the last \
             stop's colour over the whole box.",
            json!({"minimum": 0}),
        )
    }
}

/// Where a stop sits along the gradient, `0` to `1`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Offset(Number);

impl Offset {
    pub fn fraction(&self) -> f64 {
        value_of(&self.0)
    }
}

impl<'de> Deserialize<'de> for Offset {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let offset = literal::<D>(Value::deserialize(deserializer)?)?;
        if !(0.0..=1.0).contains(&value_of(&offset)) {
            return Err(D::Error::custom(format!(
                "a stop's `offset` is {offset}: it runs from `0` to `1` (ADR-0149)"
            )));
        }
        Ok(Offset(offset))
    }
}

impl JsonSchema for Offset {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Offset".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        number_schema(
            "Where the stop sits along the gradient, from `0` to `1` (ADR-0149).",
            json!({"minimum": 0, "maximum": 1}),
        )
    }
}
