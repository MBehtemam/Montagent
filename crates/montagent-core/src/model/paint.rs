//! **Paint**: what a paint field holds — a flat colour, a keyframe list of colours, or a
//! gradient (ADR-0149).
//!
//! Four fields take a paint: a `rect`'s and an `ellipse`'s `fill` and `stroke`, and a `text`
//! element's element-level `color` and `stroke`. Every other colour field — run and
//! highlight paint, the project `background`, the effect colours — stays a [`Colour`].
//!
//! # Static, in this slice
//!
//! A gradient's `angle`, `center`, `radius` and `stops` are literals: none of them may be a
//! keyframe list yet, and a paint field's keyframe list holds colours, never a gradient. The
//! schema says so by typing each parameter as a plain value. ADR-0149 §3's in-place
//! animation is the next slice, and it widens these types rather than adding a second shape.
//!
//! # Telling a stop list from a keyframe list
//!
//! A stop list is an array of objects, and [`Animatable`]'s shape rule reads any array of
//! objects as keyframes. ADR-0149 does not settle it, so the rule is stated here: **a `stops`
//! array is a keyframe list when its first item carries a `t`**, and a stop list otherwise. A
//! stop has no `t`, so the two never overlap.

use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::de::{DeserializeOwned, Error as _};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Number, Value, json};

use super::{Animatable, Colour};

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
                 object itself as the field's value (ADR-0149)",
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
                            object. A keyframe list never holds a gradient, and a gradient's \
                            parameters cannot be keyed yet (ADR-0149).",
        }))
        .expect("an object literal is a schema")
    }
}

/// A `linear` or `radial` gradient, measured against the element's declared box (ADR-0149
/// §1, §2). Every parameter is required: a wrong direction, centre or size gives a frame that
/// looks plausible, so none of them may default.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "gradient", rename_all = "lowercase", deny_unknown_fields)]
pub enum Gradient {
    /// Along a line through the box's centre.
    Linear {
        /// Degrees, CSS's convention: `0` runs bottom to top, `90` left to right, turning
        /// clockwise. Any number: `370` is `10`.
        angle: Angle,
        stops: Stops,
    },
    /// Out from a centre, as a circle in box fractions — an ellipse on a box that is not
    /// square.
    Radial {
        /// `[fx, fy]`, fractions of the box from its top-left corner. Any numbers: a glow
        /// may be centred outside the box.
        center: Center,
        /// A fraction of the distance from the centre to the box's farthest corner, in box
        /// fractions: `1` just reaches it. `0` paints the last stop's colour over the box.
        radius: Radius,
        stops: Stops,
    },
}

impl Gradient {
    /// `linear` or `radial`, as the document spells it.
    pub fn kind(&self) -> &'static str {
        match self {
            Gradient::Linear { .. } => "linear",
            Gradient::Radial { .. } => "radial",
        }
    }

    pub fn stops(&self) -> &[Stop] {
        match self {
            Gradient::Linear { stops, .. } | Gradient::Radial { stops, .. } => &stops.0,
        }
    }

    fn stops_mut(&mut self) -> &mut Vec<Stop> {
        match self {
            Gradient::Linear { stops, .. } | Gradient::Radial { stops, .. } => &mut stops.0,
        }
    }

    /// ADR-0149 §4's fix, the part a static gradient can need: each offset clamped to 0..1,
    /// then, in list order, raised to the largest offset before it. A raised stop keeps its
    /// own colour, so a crossing is a hard edge rather than a failure — and the result is
    /// always a legal literal.
    ///
    /// A literal offset is already inside 0..1 (the schema's bound), so here the fix only
    /// raises: a raised stop takes the earlier stop's offset exactly as it is written.
    pub fn settled(mut self) -> Gradient {
        let mut floor: Option<Offset> = None;
        for stop in self.stops_mut() {
            match &floor {
                Some(previous) if stop.offset.fraction() < previous.fraction() => {
                    stop.offset = previous.clone();
                }
                _ => floor = Some(stop.offset.clone()),
            }
        }
        self
    }

    /// Whether this gradient paints a single colour over the whole box: its stops share one
    /// colour, or it is a radial whose `radius` is at or below 0 (ADR-0149 §4, §5).
    pub fn one_colour(&self) -> bool {
        if let Gradient::Radial { radius, .. } = self
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
        let value = Value::deserialize(deserializer)?;
        let keyed = value
            .as_array()
            .and_then(|items| items.first())
            .is_some_and(|first| first.get("t").is_some());
        if keyed {
            return Err(D::Error::custom(not_keyable("stops")));
        }
        let stops: Vec<Stop> = serde_json::from_value(value).map_err(D::Error::custom)?;
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
                            equal offsets are legal and give a hard edge. Not a keyframe list \
                            yet (ADR-0149).",
        }))
        .expect("an object literal is a schema")
    }
}

/// The message for a gradient parameter written as a keyframe list, which this slice
/// refuses.
fn not_keyable(key: &str) -> String {
    format!(
        "`{key}` is a keyframe list: a gradient's parameters cannot be keyed yet, so write \
         one value (ADR-0149)"
    )
}

/// One parameter read as a literal, or refused by name where it is written as a keyframe
/// list — an array of objects, [`Animatable`]'s own shape test.
fn unkeyed<'de, D: Deserializer<'de>, T: DeserializeOwned>(
    deserializer: D,
    key: &str,
) -> Result<T, D::Error> {
    let value = Value::deserialize(deserializer)?;
    let keyed = value
        .as_array()
        .and_then(|items| items.first())
        .is_some_and(Value::is_object);
    if keyed {
        return Err(D::Error::custom(not_keyable(key)));
    }
    serde_json::from_value(value).map_err(D::Error::custom)
}

/// A number kept exactly as the document spells it, so a tool that re-emits the file —
/// `shift`, `fmt` — writes `30` back as `30` and `0.5` as `0.5`, and `query --at` prints the
/// literal that was written: the shortest exact form, which pastes back to the same bytes.
fn literal<'de, D: Deserializer<'de>>(value: Value) -> Result<Number, D::Error> {
    match value {
        Value::Number(number) if number.as_f64().is_some() => Ok(number),
        other => Err(D::Error::custom(format!(
            "invalid type: {other}, expected a number"
        ))),
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
        literal::<D>(unkeyed(deserializer, "angle")?).map(Angle)
    }
}

impl JsonSchema for Angle {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Angle".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        number_schema(
            "Degrees, CSS's convention: `0` runs bottom to top, `90` left to right, turning \
             clockwise. Any number. Not a keyframe list yet (ADR-0149).",
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
        let pair: Vec<Value> = unkeyed(deserializer, "center")?;
        let [fx, fy]: [Value; 2] = pair.try_into().map_err(|pair: Vec<Value>| {
            D::Error::custom(format!(
                "`center` has {} numbers: it is `[fx, fy]`, two fractions of the box",
                pair.len()
            ))
        })?;
        Ok(Center([literal::<D>(fx)?, literal::<D>(fy)?]))
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
                            corner. Any numbers: a centre may sit outside the box. Not a \
                            keyframe list yet (ADR-0149).",
        }))
        .expect("an object literal is a schema")
    }
}

/// A radial gradient's size, a fraction of the centre-to-farthest-corner distance. `≥ 0`.
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
        let radius = literal::<D>(unkeyed(deserializer, "radius")?)?;
        if value_of(&radius) < 0.0 {
            return Err(D::Error::custom(format!(
                "a gradient's `radius` is {radius}: it is a fraction of the distance to the \
                 box's farthest corner, `0` or more (ADR-0149)"
            )));
        }
        Ok(Radius(radius))
    }
}

impl JsonSchema for Radius {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "GradientRadius".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        number_schema(
            "A fraction of the distance from the centre to the declared box's farthest \
             corner, in box fractions: `1` just reaches it. `0` paints the last stop's colour \
             over the whole box. Not a keyframe list yet (ADR-0149).",
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
