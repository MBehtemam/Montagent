//! **Animatable properties**: the one list of them, and the one function that says what one
//! of them is at an instant (ADR-0146).
//!
//! # The list is the schema's
//!
//! A property is an animatable property only where the schema types it as one: a literal, or
//! a keyframe list (ADR-0146 §1). So the list is not written here. It is read out of the
//! published schema — every element branch's property whose type is an `Animatable{T}` — once
//! per process, the way [`crate::layout`] reads canonical key order. Every tool that walks
//! keyframe lists (`shift`, the checks, the contact sheet, `compare`, `timeline`,
//! `query --at`) reads this list and keeps none of its own: the prototype's worst finding was
//! `shift` leaving a keyed `width` and `fill` behind because a six-name list had been copied
//! into three places.
//!
//! Each entry also carries its value [`Kind`], read off the same schema type, because what a
//! reader does with a list depends on it: an integer splits to the nearest integer, a colour
//! to bytes, a pair component by component.
//!
//! # The one resolving function
//!
//! [`read`] is what a property *is* at an instant, and the clamps ADR-0146 §5 publishes live
//! in it and nowhere else, so `query --at`, `validate`, the contact sheet and the painter
//! report the same value:
//!
//! - a colour blends in sRGB with premultiplied alpha, each component clamped to its range,
//!   then rounded to the nearest byte ([`crate::resolve::Blend`]);
//! - an integer-typed property resolves to a continuous value and is never rounded;
//! - a resolved `radius` clamps to half the shorter side of the box at the same instant;
//! - a resolved `stroke_width` below zero clamps to `0`;
//! - a resolved `width` or `height` is left as it is: at or below zero it draws nothing for
//!   that frame ([`painted_box`]), and between two integers it is drawn unrounded.
//!
//! [`crate::resolve`] stays the keyframe arithmetic underneath; this module is the reading
//! of a property, by name, off an element.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Serialize;
use serde_json::Value;

use crate::model::{Animatable, Colour, Scale};
use crate::resolve::{self, Interpolate, Unresolvable};

/// What one animatable property's values are, as the schema types them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// An integer in the document, continuous once resolved: `x`, `y`, and the lengths.
    Integer,
    /// A number: `rotation`, `opacity`, `volume`.
    Number,
    /// A fixed-size pair, each component interpolated separately: `scale`.
    Pair,
    /// A `#RRGGBB` or `#RRGGBBAA` colour.
    Colour,
}

/// One animatable property of one element type.
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    /// The key, as the document spells it.
    pub name: String,
    pub kind: Kind,
    /// The bound the schema states on every value, where it states one — `0` on a length
    /// and on `volume`. A `shift` split that would write a value past it is refused.
    pub minimum: Option<f64>,
}

struct Table {
    by_type: BTreeMap<String, Vec<Property>>,
    names: Vec<String>,
}

fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| derive(&crate::schema::generate()))
}

/// Read the list out of the published schema.
fn derive(schema: &Value) -> Table {
    let defs = &schema["$defs"];
    let mut by_type = BTreeMap::new();
    let mut names: Vec<String> = Vec::new();
    for branch in schema["$defs"]["Element"]["oneOf"]
        .as_array()
        .into_iter()
        .flatten()
    {
        let Some(element_type) = branch["properties"]["type"]["const"].as_str() else {
            continue;
        };
        let mut properties = Vec::new();
        for (name, property) in branch["properties"].as_object().into_iter().flatten() {
            let Some(def) = property["$ref"]
                .as_str()
                .and_then(|reference| reference.strip_prefix("#/$defs/"))
                .filter(|def| def.starts_with("Animatable"))
            else {
                continue;
            };
            // The first alternative of an `Animatable{T}` is the static `T`.
            let mut value = &defs[def]["anyOf"][0];
            let mut colour = false;
            if let Some(target) = value["$ref"]
                .as_str()
                .and_then(|reference| reference.strip_prefix("#/$defs/"))
            {
                colour = target == "Colour";
                value = &defs[target];
            }
            let kind = if colour {
                Kind::Colour
            } else {
                match value["type"].as_str() {
                    Some("integer") => Kind::Integer,
                    Some("number") => Kind::Number,
                    Some("array") => Kind::Pair,
                    _ => continue,
                }
            };
            if !names.iter().any(|seen| seen == name) {
                names.push(name.clone());
            }
            properties.push(Property {
                name: name.clone(),
                kind,
                minimum: value["minimum"].as_f64(),
            });
        }
        by_type.insert(element_type.to_string(), properties);
    }
    Table { by_type, names }
}

/// **The one list**: every animatable property of `element_type`, in schema order. Empty
/// for a type that has none, or that the format does not define.
pub fn of(element_type: &str) -> &'static [Property] {
    table()
        .by_type
        .get(element_type)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

/// Every animatable property name across every element type, each once, in the order the
/// schema first declares it. For a reader that walks a permissive tree whose `type` it does
/// not trust: a key that is not animatable on this element's type is a schema error
/// `validate` reports, not a list this reader has to have an opinion about.
pub fn names() -> &'static [String] {
    &table().names
}

/// The animatable properties this element's own `type` has.
pub fn of_element(element: &Value) -> &'static [Property] {
    element
        .get("type")
        .and_then(Value::as_str)
        .map(of)
        .unwrap_or_default()
}

/// What `name` is wherever it appears. A key has one kind on every type that has it.
pub fn property(name: &str) -> Option<&'static Property> {
    table()
        .by_type
        .values()
        .flatten()
        .find(|property| property.name == name)
}

/// One property's keyframe records, or `None` where it is absent or static.
///
/// ADR-0012's own shape test, as [`Animatable`] applies it on the way in: a keyframe record
/// is an object, so an array **of objects** is a keyframe list and every other array —
/// `scale`'s own `[sx, sy]` — is a static value.
pub fn records<'a>(element: &'a Value, property: &str) -> Option<&'a Vec<Value>> {
    let records = element.get(property)?.as_array()?;
    records
        .first()
        .is_some_and(Value::is_object)
        .then_some(records)
}

/// A property's value at an instant.
///
/// Serialized as the document would write it: a number, a pair, or a colour literal that
/// can be pasted back — six digits when opaque, `#00000000` at alpha 0.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Resolved {
    Number(f64),
    Pair([f64; 2]),
    Colour(Colour),
}

impl Resolved {
    /// The number, where this is one.
    pub fn number(&self) -> Option<f64> {
        match self {
            Resolved::Number(value) => Some(*value),
            _ => None,
        }
    }
}

/// Why a declared property has no value at an instant.
#[derive(Debug, Clone, PartialEq)]
pub enum Unreadable {
    /// The written value does not read as the property's type — the schema check's fact.
    Schema(String),
    /// It reads, and the keyframe list still determines no value.
    Unresolvable(Unresolvable),
}

/// **The one resolving function**: what `property` is on `element` at `numerator /
/// denominator` ms, with ADR-0146's clamps applied.
///
/// `None` where the element does not declare the property: no default is synthesised here,
/// because a caller that wants one — the painter, a geometry check — states its own.
pub fn read(
    element: &Value,
    property: &str,
    numerator: i128,
    denominator: i128,
) -> Option<Result<Resolved, Unreadable>> {
    let written = element.get(property)?;
    let kind = property_kind(element, property);
    Some(
        raw(written, kind, numerator, denominator)
            .map(|value| clamp(element, property, value, numerator, denominator)),
    )
}

/// [`read`] at a whole millisecond.
pub fn at(element: &Value, property: &str, instant: i64) -> Option<Result<Resolved, Unreadable>> {
    read(element, property, i128::from(instant), 1)
}

/// [`at`], read as a number with the caller's own default where the element does not
/// declare it or it cannot be read — the reading a painter or a geometry check wants.
pub fn number_at(element: &Value, property: &str, instant: i64, default: f64) -> f64 {
    at(element, property, instant)
        .and_then(Result::ok)
        .and_then(|value| value.number())
        .unwrap_or(default)
}

/// [`at`], read as a colour, where the element declares a readable one.
pub fn colour_at(element: &Value, property: &str, instant: i64) -> Option<Colour> {
    match at(element, property, instant)?.ok()? {
        Resolved::Colour(colour) => Some(colour),
        _ => None,
    }
}

/// The kind the element's type gives this key, else the kind the key has anywhere.
fn property_kind(element: &Value, property: &str) -> Option<Kind> {
    of_element(element)
        .iter()
        .find(|candidate| candidate.name == property)
        .or_else(|| self::property(property))
        .map(|property| property.kind)
}

fn raw(
    written: &Value,
    kind: Option<Kind>,
    numerator: i128,
    denominator: i128,
) -> Result<Resolved, Unreadable> {
    fn one<T, F>(
        written: &Value,
        numerator: i128,
        denominator: i128,
        wrap: F,
    ) -> Result<Resolved, Unreadable>
    where
        T: serde::de::DeserializeOwned + Interpolate,
        F: Fn(T::Out) -> Resolved,
    {
        let animatable = serde_json::from_value::<Animatable<T>>(written.clone())
            .map_err(|e| Unreadable::Schema(e.to_string()))?;
        resolve::at_instant(&animatable, numerator, denominator)
            .map(wrap)
            .map_err(Unreadable::Unresolvable)
    }

    match kind {
        // Read as an integer, without the length bound: a negative value is the schema
        // check's to name, and the view still answers what the document writes.
        Some(Kind::Integer) => one::<i64, _>(written, numerator, denominator, Resolved::Number),
        Some(Kind::Number) => one::<f64, _>(written, numerator, denominator, Resolved::Number),
        Some(Kind::Pair) => one::<Scale, _>(written, numerator, denominator, Resolved::Pair),
        Some(Kind::Colour) => one::<Colour, _>(written, numerator, denominator, |blend| {
            Resolved::Colour(blend.settle())
        }),
        None => Err(Unreadable::Schema(
            "the key is not an animatable property".to_string(),
        )),
    }
}

/// ADR-0146 §5's clamps on the numbers, by property. The colour clamp is [`Blend::settle`]'s,
/// applied in [`raw`], because it is the colour rule rather than a property's.
///
/// [`Blend::settle`]: crate::resolve::Blend::settle
fn clamp(
    element: &Value,
    property: &str,
    value: Resolved,
    numerator: i128,
    denominator: i128,
) -> Resolved {
    let Resolved::Number(number) = value else {
        return value;
    };
    Resolved::Number(match property {
        "stroke_width" => number.max(0.0),
        "radius" => {
            let half = match sides(element, numerator, denominator) {
                Some((width, height)) => (width.min(height) / 2.0).max(0.0),
                None => f64::INFINITY,
            };
            number.clamp(0.0, half)
        }
        _ => number,
    })
}

/// The resolved `width` and `height`, unclamped, where both read as numbers.
fn sides(element: &Value, numerator: i128, denominator: i128) -> Option<(f64, f64)> {
    let side = |key| read(element, key, numerator, denominator)?.ok()?.number();
    Some((side("width")?, side("height")?))
}

/// The declared box at an instant, before `scale`, where it is drawn at all.
///
/// `None` where the element states no readable `width`/`height`, or where either resolves at
/// or below zero — which draws nothing for that frame, as `opacity` 0 does (ADR-0146 §5). An
/// in-between box is returned unrounded.
pub fn painted_box(element: &Value, numerator: i128, denominator: i128) -> Option<(f64, f64)> {
    sides(element, numerator, denominator).filter(|(width, height)| *width > 0.0 && *height > 0.0)
}

/// Whether the element states a box at all — a `width` and a `height` that read as numbers
/// at its own start, whatever their sign.
pub fn states_a_box(element: &Value) -> bool {
    let start = element.get("start").and_then(Value::as_i64).unwrap_or(0);
    sides(element, i128::from(start), 1).is_some()
}

/// **Never painted** (ADR-0146 §6): the element states a box, and it has no positive size at
/// any frame its own range holds.
///
/// Decided at the frame instants `render` paints — [`crate::exact::instant_of`] of every
/// frame inside `[start, end)` — through [`painted_box`], the same reading the painter
/// takes, so `validate` and `render` agree. An element whose range holds no frame at all is
/// not answered here: that is `N-QUANTIZATION`'s fact.
pub fn never_painted(element: &Value, fps: i64) -> bool {
    let (Some(start), Some(end)) = (
        element.get("start").and_then(Value::as_i64),
        element.get("end").and_then(Value::as_i64),
    ) else {
        return false;
    };
    if fps <= 0 || end <= start || !states_a_box(element) {
        return false;
    }
    let (Some(first), Some(last)) = (
        crate::exact::frame_at_or_after(start, fps),
        crate::exact::frame_before(end, fps),
    ) else {
        return false;
    };
    if first.frame > last.frame {
        return false;
    }
    // A static box is one answer for every frame.
    if records(element, "width").is_none() && records(element, "height").is_none() {
        return painted_box(element, i128::from(start), 1).is_none();
    }
    !(first.frame..=last.frame).any(|n| {
        let instant = crate::exact::instant_of(n, fps);
        painted_box(element, i128::from(instant), 1).is_some()
    })
}

/// The largest value a length property ever states — its static value, or the greatest of
/// its keyframe values. What a text element's layout is measured with, where its
/// `stroke_width` is keyed: the stroke never moves a glyph, and the extent a declared box
/// is checked against is the one the stroke reaches at its widest.
pub fn greatest_length(element: &Value, property: &str) -> Option<i64> {
    let written = element.get(property)?;
    match serde_json::from_value::<Animatable<i64>>(written.clone()).ok()? {
        Animatable::Static(value) => Some(value),
        Animatable::Keyed(records) => records.iter().map(|record| record.v).max(),
    }
}
