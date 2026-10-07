//! **Animatable properties**: the one list of them, and the one function that says what one
//! of them is at an instant (ADR-0146).
//!
//! # The list is the schema's
//!
//! A property is an animatable property only where the schema types it as one: a literal, or
//! a keyframe list (ADR-0146 §1). So the list is not written here. It is read out of the
//! published schema — every element branch's property whose type is an `Animatable{T}`, and
//! every `effects` member's parameter typed the same way — once per process, the way
//! [`crate::layout`] reads canonical key order. Every tool that walks keyframe lists
//! (`shift`, the checks, the contact sheet, `compare`, `timeline`, `query --at`) reads this
//! list through [`declared`] and keeps none of its own: the prototype's worst finding was
//! `shift` leaving a keyed `width` and `fill` behind because a six-name list had been copied
//! into three places.
//!
//! A paint's gradient parameters are on the list as nested paths: `fill.angle`,
//! `fill.center`, `fill.radius` and `fill.stops`, and the same under `stroke` and a text
//! element's `color` (ADR-0149 §6). A path is read by [`get`], so a tool that walks the list
//! reads a nested property as it reads a top-level one. A gradient has `angle` or
//! `center` and `radius`, not both, so a path the element's gradient does not have is
//! absent, as an undeclared property is.
//!
//! Each entry also carries its value [`Kind`], read off the same schema type, because what a
//! reader does with a list depends on it: an integer splits to the nearest integer, a colour
//! to bytes, a pair component by component.
//!
//! # Effect parameters are named by position
//!
//! An animated effect parameter is written in place inside its member (ADR-0146 §4), and a
//! tool names it by the member's zero-based position in `effects`, with the member's name in
//! the text: `effects[1].radius (blur)`. Position is the only unambiguous name, because two
//! members of one name are legal.
//!
//! # The one resolving function
//!
//! [`read`] (and [`Declared::read`] for an effect parameter) is what a property *is* at an
//! instant, and the clamps ADR-0146 §5 publishes live in it and nowhere else, so
//! `query --at`, `validate`, the contact sheet and the painter report the same value:
//!
//! - a colour blends in sRGB with premultiplied alpha, each component clamped to its range,
//!   then rounded to the nearest byte ([`crate::resolve::Blend`]);
//! - an integer-typed property resolves to a continuous value and is never rounded;
//! - a resolved `radius` clamps to half the shorter side of the box at the same instant — a
//!   `mask`'s to half the shorter side of its own rect;
//! - a resolved `stroke_width` below zero clamps to `0`;
//! - a path's resolved `points` clamps every absolute vertex and handle into the inset box
//!   (ADR-0154 §3);
//! - a paint field holding a gradient (ADR-0149) resolves every parameter at the instant
//!   (`angle`, `center`, `radius` and `stops` may each be keyed), then applies §4's fix — each
//!   offset clamped to 0..1 and raised to the largest before it — so what is printed is what
//!   is drawn;
//! - a resolved effect parameter clamps into the range the schema states on it, such as
//!   `chroma`'s `[0, 1]`;
//! - a resolved `width` or `height` is left as it is: at or below zero it draws nothing for
//!   that frame ([`painted_box`]), and between two integers it is drawn unrounded. A plain
//!   `mask`'s does the same to the whole element ([`hiding_mask`]).
//!
//! [`crate::resolve`] stays the keyframe arithmetic underneath; this module is the reading
//! of a property, by name, off an element.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Serialize;
use serde_json::Value;

use crate::model::{Animatable, Colour, Gradient, Points, ResolvedGradient, Scale, Stops};
use crate::resolve::{self, Interpolate, Unresolvable, VertexAt};

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
    /// A paint (ADR-0149): a colour, a keyframe list of colours, or a gradient object. Its
    /// keyframe values are colours, so a reader that splits or blends a list treats it as
    /// [`Kind::Colour`]; a gradient resolves to [`Resolved::Gradient`].
    Paint,
    /// A path's whole vertex list, every `at`, `in` and `out` coordinate interpolated
    /// separately: `points` (ADR-0154 §3).
    Points,
    /// A gradient's stop list (ADR-0149): each keyframe value is a whole list, blended stop
    /// by stop — an offset as a number, a colour premultiplied.
    Stops,
}

impl Kind {
    /// Whether this kind's keyframe values are colours.
    pub fn is_colour(self) -> bool {
        matches!(self, Kind::Colour | Kind::Paint)
    }
}

/// One animatable property of one element type, or one parameter of one `effects` member.
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    /// The key, as the document spells it.
    pub name: String,
    pub kind: Kind,
    /// The bound the schema states on every value, where it states one — `0` on a length
    /// and on `volume`. A `shift` split that would write a value past it is refused.
    pub minimum: Option<f64>,
    /// The upper bound the schema states on every value, where it states one — `1` on
    /// `chroma`'s three scalars. A `shift` split that would write a value past it is
    /// refused.
    pub maximum: Option<f64>,
}

struct Table {
    by_type: BTreeMap<String, Vec<Property>>,
    names: Vec<String>,
    /// Every `effects` member's animatable parameters, by the `name` the member is
    /// discriminated on, in the order the schema declares the members.
    by_effect: Vec<(String, Vec<Property>)>,
}

fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| derive(&crate::schema::generate()))
}

/// Read the list out of the published schema: every element branch's animatable
/// properties, and every `effects` member's (ADR-0146 §3).
fn derive(schema: &Value) -> Table {
    let defs = &schema["$defs"];
    let mut by_type = BTreeMap::new();
    let mut names: Vec<String> = Vec::new();
    for branch in defs["Element"]["oneOf"].as_array().into_iter().flatten() {
        let Some(element_type) = branch["properties"]["type"]["const"].as_str() else {
            continue;
        };
        let properties = animatable_properties(defs, branch);
        for property in &properties {
            if !names.contains(&property.name) {
                names.push(property.name.clone());
            }
        }
        by_type.insert(element_type.to_string(), properties);
    }
    let by_effect = defs["Effect"]["oneOf"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|branch| {
            let member = branch["properties"]["name"]["const"].as_str()?;
            Some((member.to_string(), animatable_properties(defs, branch)))
        })
        .collect();
    Table {
        by_type,
        names,
        by_effect,
    }
}

/// The properties of one object branch whose type is an `Animatable{T}`, in schema order. A
/// paint is followed by its gradient's parameters as nested paths — `fill.angle`,
/// `fill.stops`, `fill.center`, `fill.radius` (ADR-0149 §6).
fn animatable_properties(defs: &Value, branch: &Value) -> Vec<Property> {
    let mut properties = Vec::new();
    for (name, property) in branch["properties"].as_object().into_iter().flatten() {
        let Some(def) = property["$ref"]
            .as_str()
            .and_then(|reference| reference.strip_prefix("#/$defs/"))
            .filter(|def| def.starts_with("Animatable") || *def == "Paint")
        else {
            continue;
        };
        if def == "Paint" {
            properties.push(Property {
                name: name.clone(),
                kind: Kind::Paint,
                minimum: None,
                maximum: None,
            });
            for parameter in gradient_parameters(defs) {
                properties.push(Property {
                    name: format!("{name}.{}", parameter.name),
                    ..parameter
                });
            }
            continue;
        }
        if let Some(property) = typed(defs, name, def) {
            properties.push(property);
        }
    }
    properties
}

/// The property named `name` whose type is the `Animatable{T}` definition `def`, read off the
/// first alternative of it, which is the static `T`. `None` for a type the format does not
/// interpolate.
fn typed(defs: &Value, name: &str, def: &str) -> Option<Property> {
    let mut value = &defs[def]["anyOf"][0];
    if let Some(target) = value["$ref"]
        .as_str()
        .and_then(|reference| reference.strip_prefix("#/$defs/"))
    {
        value = &defs[target];
    }
    // The one string the format interpolates is a colour (ADR-0146 §2): `Colour`, or a
    // member's narrowing of it such as `chroma`'s screen colour. A path's vertex list is an
    // array the schema names `Points` (ADR-0154 §3), and a gradient's stop list one it names
    // `Stops` (ADR-0149 §3).
    let kind = match (property_def_name(&defs[def]), value["type"].as_str()) {
        (Some("Points"), _) => Kind::Points,
        (Some("Stops"), _) => Kind::Stops,
        (_, Some("string")) => Kind::Colour,
        (_, Some("integer")) => Kind::Integer,
        (_, Some("number")) => Kind::Number,
        (_, Some("array")) => Kind::Pair,
        _ => return None,
    };
    Some(Property {
        name: name.to_string(),
        kind,
        minimum: value["minimum"].as_f64(),
        maximum: value["maximum"].as_f64(),
    })
}

/// The `$defs` name an `Animatable{T}`'s static alternative refers to, where it names one.
fn property_def_name(animatable: &Value) -> Option<&str> {
    animatable["anyOf"][0]["$ref"]
        .as_str()?
        .strip_prefix("#/$defs/")
}

/// A gradient's animatable parameters, read off the `Gradient` definition: every property of
/// either kind whose type is an `Animatable{T}`, each once, in schema order (ADR-0149 §6).
fn gradient_parameters(defs: &Value) -> Vec<Property> {
    let mut out: Vec<Property> = Vec::new();
    for branch in defs["Gradient"]["oneOf"].as_array().into_iter().flatten() {
        for (name, property) in branch["properties"].as_object().into_iter().flatten() {
            let Some(parameter) = property["$ref"]
                .as_str()
                .and_then(|reference| reference.strip_prefix("#/$defs/"))
                .filter(|def| def.starts_with("Animatable"))
                .and_then(|def| typed(defs, name, def))
            else {
                continue;
            };
            if !out.iter().any(|seen| seen.name == *name) {
                out.push(parameter);
            }
        }
    }
    out
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

/// What `name` is wherever it appears on an element. A key has one kind on every type that
/// has it.
pub fn property(name: &str) -> Option<&'static Property> {
    table()
        .by_type
        .values()
        .flatten()
        .find(|property| property.name == name)
}

/// Every `effects` member, by the `name` it is discriminated on, with its animatable
/// parameters in schema order (ADR-0146 §3).
pub fn effect_members() -> impl Iterator<Item = (&'static str, &'static [Property])> {
    table()
        .by_effect
        .iter()
        .map(|(member, parameters)| (member.as_str(), parameters.as_slice()))
}

/// The animatable parameters of the `effects` member named `member`, with the member's name
/// as the table holds it. `None` for a name the vocabulary does not have.
fn of_effect(member: &str) -> Option<(&'static str, &'static [Property])> {
    effect_members().find(|(name, _)| *name == member)
}

/// The name a tool calls an effect parameter by (ADR-0146 §4): its member's zero-based
/// position in `effects`, with the member's name in the text — `effects[1].radius (blur)`.
pub fn effect_path(index: usize, key: &str, member: &str) -> String {
    format!("effects[{index}].{key} ({member})")
}

/// One property's keyframe records, or `None` where it is absent or static.
///
/// ADR-0012's own shape test, as [`Animatable`] applies it on the way in
/// ([`crate::model::keyframe::is_keyframe_list`]): `scale`'s own `[sx, sy]` and a path's
/// static vertex list are static values.
pub fn records<'a>(element: &'a Value, property: &str) -> Option<&'a Vec<Value>> {
    let written = get(element, property)?;
    crate::model::is_keyframe_list(written)
        .then(|| written.as_array())
        .flatten()
}

/// Whether any of a paint's nested properties (`fill.angle`, ADR-0149 §6) is a keyframe list.
pub fn nested_keyed(element: &Value, property: &str) -> bool {
    let prefix = format!("{property}.");
    names()
        .iter()
        .filter(|name| name.starts_with(&prefix))
        .any(|name| records(element, name).is_some())
}

/// What the document writes at `path` on `element`: a key, or a nested path like
/// `fill.angle` (ADR-0149 §6), each step a key of the object before it.
pub fn get<'a>(element: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.')
        .try_fold(element, |value, key| value.get(key))
}

/// [`get`], to edit in place.
pub fn get_mut<'a>(element: &'a mut Value, path: &str) -> Option<&'a mut Value> {
    path.split('.')
        .try_fold(element, |value, key| value.get_mut(key))
}

/// One animatable property an element writes, wherever it sits: on the element itself, or
/// inside one of its `effects`.
#[derive(Debug, Clone)]
pub struct Declared<'a> {
    /// What a tool calls it: `width`, or `effects[1].radius (blur)`.
    pub path: String,
    /// The element that declares it.
    pub element: &'a Value,
    /// The object the key is written in: the element, or the effect member.
    pub owner: &'a Value,
    pub property: &'static Property,
    /// The member's position in `effects` and its name, for an effect parameter.
    pub effect: Option<(usize, &'static str)>,
}

impl<'a> Declared<'a> {
    /// The key, as the document spells it inside its owner.
    pub fn key(&self) -> &'static str {
        self.property.name.as_str()
    }

    /// Its keyframe records, or `None` where it is static.
    pub fn records(&self) -> Option<&'a Vec<Value>> {
        records(self.owner, self.key())
    }

    /// What it is at `numerator / denominator` ms: **the one resolving function**, with
    /// ADR-0146's clamps applied.
    pub fn read(&self, numerator: i128, denominator: i128) -> Result<Resolved, Unreadable> {
        let Some(written) = get(self.owner, self.key()) else {
            return Err(Unreadable::Schema(
                "the property is not written".to_string(),
            ));
        };
        raw(written, Some(self.property.kind), numerator, denominator).map(|value| {
            match self.effect {
                None => clamp(self.element, self.key(), value, numerator, denominator),
                Some((index, member)) => {
                    clamp_parameter(self, index, member, value, numerator, denominator)
                }
            }
        })
    }

    /// [`Declared::read`] at a whole millisecond.
    pub fn at(&self, instant: i64) -> Result<Resolved, Unreadable> {
        self.read(i128::from(instant), 1)
    }
}

/// **Every animatable property this element writes**, static or keyed, in the order a tool
/// walks them: the element's own, in the order its type's schema declares them, then every
/// `effects` member's parameters, member by member, in the schema's order within one.
///
/// Read off the permissive tree: after its type's own, the element's keys include every
/// other animatable name it carries (a key not animatable on its type is the schema check's
/// to name, and a reader still walks the list), and a member whose `name` the vocabulary does
/// not have is skipped.
pub fn declared(element: &Value) -> Vec<Declared<'_>> {
    let own = of_element(element);
    let others = names()
        .iter()
        .filter(|name| !own.iter().any(|property| property.name == **name))
        .filter_map(|name| property(name));
    let mut out: Vec<Declared<'_>> = own
        .iter()
        .chain(others)
        .filter(|property| get(element, property.name.as_str()).is_some())
        .map(|property| Declared {
            path: property.name.clone(),
            element,
            owner: element,
            property,
            effect: None,
        })
        .collect();
    for (index, effect) in effects(element).iter().enumerate() {
        let Some((member, parameters)) = effect
            .get("name")
            .and_then(Value::as_str)
            .and_then(of_effect)
        else {
            continue;
        };
        for property in parameters {
            if effect.get(property.name.as_str()).is_none() {
                continue;
            }
            out.push(Declared {
                path: effect_path(index, &property.name, member),
                element,
                owner: effect,
                property,
                effect: Some((index, member)),
            });
        }
    }
    out
}

/// The element's `effects`, or none where it writes no list.
fn effects(element: &Value) -> &[Value] {
    element
        .get("effects")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

/// One parameter of the `effects` member at `index`, as [`declared`] names it.
pub fn effect_parameter<'a>(element: &'a Value, index: usize, key: &str) -> Option<Declared<'a>> {
    declared(element).into_iter().find(|declared| {
        declared.effect.is_some_and(|(at, _)| at == index) && declared.key() == key
    })
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
    /// A gradient paint, after ADR-0149 §4's fix: always a literal that can be pasted back.
    Gradient(ResolvedGradient),
    /// A path's vertices, each handle an offset from its own vertex as the document writes
    /// it (ADR-0154).
    Points(Vec<VertexAt>),
    /// A gradient's stop list, fixed as the gradient's own are.
    Stops(Stops),
}

impl Resolved {
    /// The number, where this is one.
    pub fn number(&self) -> Option<f64> {
        match self {
            Resolved::Number(value) => Some(*value),
            _ => None,
        }
    }

    /// The colour, where this is one.
    pub fn colour(self) -> Option<Colour> {
        match self {
            Resolved::Colour(colour) => Some(colour),
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
    let written = get(element, property)?;
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
    number_read(element, property, i128::from(instant), 1, default)
}

/// [`number_at`] at `numerator / denominator` ms — a motion-blur sample (ADR-0155 §3).
pub fn number_read(
    element: &Value,
    property: &str,
    numerator: i128,
    denominator: i128,
    default: f64,
) -> f64 {
    read(element, property, numerator, denominator)
        .and_then(Result::ok)
        .and_then(|value| value.number())
        .unwrap_or(default)
}

/// prototype(#760, ADR-0160 §7): a number as written, before ADR-0146 §5's clamp — what
/// `query --at` prints for the trim fields, so each matches the agent's literal.
pub fn number_unclamped(
    element: &Value,
    property: &str,
    numerator: i128,
    denominator: i128,
) -> Option<f64> {
    let written = get(element, property)?;
    raw(written, property_kind(element, property), numerator, denominator)
        .ok()?
        .number()
}

/// [`at`], read as a colour, where the element declares a readable one.
pub fn colour_at(element: &Value, property: &str, instant: i64) -> Option<Colour> {
    at(element, property, instant)?.ok()?.colour()
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
        Some(Kind::Stops) => one::<Stops, _>(written, numerator, denominator, |blend| {
            Resolved::Stops(blend.settle())
        }),
        // A gradient object resolves every parameter at the instant, then is fixed (§4).
        Some(Kind::Paint) if written.is_object() => {
            let gradient = serde_json::from_value::<Gradient>(written.clone())
                .map_err(|e| Unreadable::Schema(e.to_string()))?;
            gradient
                .resolve(numerator, denominator)
                .map(Resolved::Gradient)
                .map_err(Unreadable::Unresolvable)
        }
        Some(Kind::Paint) => one::<Colour, _>(written, numerator, denominator, |blend| {
            Resolved::Colour(blend.settle())
        }),
        Some(Kind::Points) => one::<Points, _>(written, numerator, denominator, Resolved::Points),
        None => Err(Unreadable::Schema(
            "the key is not an animatable property".to_string(),
        )),
    }
}

/// ADR-0146 §5's clamps on an element's own numbers, by property. The colour clamp is
/// [`Blend::settle`]'s, applied in [`raw`], because it is the colour rule rather than a
/// property's.
///
/// [`Blend::settle`]: crate::resolve::Blend::settle
fn clamp(
    element: &Value,
    property: &str,
    value: Resolved,
    numerator: i128,
    denominator: i128,
) -> Resolved {
    let number = match value {
        Resolved::Number(number) => number,
        Resolved::Points(vertices) => return Resolved::Points(inside(element, vertices)),
        other => return other,
    };
    Resolved::Number(match property {
        "stroke_width" => number.max(0.0),
        // prototype(#760, ADR-0160 §5): an overshooting ease clamps to [0, 1].
        "trim_start" | "trim_end" => number.clamp(0.0, 1.0),
        "radius" => number.clamp(0.0, half_shorter(sides(element, numerator, denominator))),
        _ => number,
    })
}

/// ADR-0154 §3's overshoot rule: every absolute vertex and handle position clamped into the
/// inset box at that instant, each handle then written back as an offset from its clamped
/// vertex. Where a path's box does not read, the vertices are left as they resolved.
fn inside(element: &Value, vertices: Vec<VertexAt>) -> Vec<VertexAt> {
    let Some(((left, top), (right, bottom))) = inset_box(element) else {
        return vertices;
    };
    let clamp = |[x, y]: [f64; 2]| [x.clamp(left, right), y.clamp(top, bottom)];
    vertices
        .into_iter()
        .map(|vertex| {
            let at = clamp(vertex.at);
            let handle = |offset: Option<[f64; 2]>| {
                offset.map(|[dx, dy]| {
                    let [x, y] = clamp([vertex.at[0] + dx, vertex.at[1] + dy]);
                    [x - at[0], y - at[1]]
                })
            };
            VertexAt {
                arriving: handle(vertex.arriving),
                out: handle(vertex.out),
                at,
            }
        })
        .collect()
}

/// A path's **inset**: `ceil(stroke_width / 2)`, or `0` with no stroke, from the largest
/// value a keyed `stroke_width` states (ADR-0154 §4).
pub fn path_inset(element: &Value) -> i64 {
    path_reach(element).inset
}

/// prototype(#750, ADR-0158 §4): a path's inset `m = ceil(k × w / 2)` and where `k` came from.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Reach {
    pub inset: i64,
    /// `w`: the largest value `stroke_width` states.
    pub stroke_width: i64,
    /// `k`, as a reader writes it: `1`, the miter limit, or `√2`.
    pub k: String,
    /// Where `k` came from.
    pub source: String,
}

/// prototype(#750): ADR-0158 §4. The join factor is the miter limit under `"miter"`, else 1;
/// the cap factor is √2 for `"square"` where a cap draws (an open path, or any dashed path),
/// else 1. `ceil(√2 × w / 2)` is computed exactly as the least `m` with `2m² ≥ w²`.
pub fn path_reach(element: &Value) -> Reach {
    let none = |w| Reach {
        inset: 0,
        stroke_width: w,
        k: "1".into(),
        source: "no stroke".into(),
    };
    if element.get("stroke").is_none() {
        return none(0);
    }
    let w = greatest_length(element, "stroke_width").unwrap_or(0).max(0);
    let join_miter = element.get("stroke_join").and_then(Value::as_str) == Some("miter");
    let limit = element
        .get("stroke_miter_limit")
        .and_then(Value::as_i64)
        .filter(|_| join_miter)
        .unwrap_or(1)
        .max(1);
    let closed = element.get("closed").and_then(Value::as_bool).unwrap_or(false);
    // prototype(#760, ADR-0160 §6): a cap also draws at a trim's ends, decided by the
    // presence of `trim_start` or `trim_end`, not their values.
    let cap_draws = !closed
        || element.get("stroke_dash").is_some()
        || element.get("trim_start").is_some()
        || element.get("trim_end").is_some();
    let square = cap_draws && element.get("stroke_cap").and_then(Value::as_str) == Some("square");
    let by_join = (limit * w + 1) / 2;
    let by_cap = if square {
        let mut m = (w + 1) / 2;
        while 2 * m * m < w * w {
            m += 1;
        }
        m
    } else {
        (w + 1) / 2
    };
    if by_join >= by_cap && limit > 1 {
        Reach {
            inset: by_join,
            stroke_width: w,
            k: limit.to_string(),
            source: format!("`stroke_miter_limit` {limit}"),
        }
    } else if square && by_cap > (w + 1) / 2 {
        Reach {
            inset: by_cap,
            stroke_width: w,
            k: "√2".into(),
            source: "`stroke_cap` \"square\"".into(),
        }
    } else {
        Reach {
            inset: (w + 1) / 2,
            stroke_width: w,
            k: "1".into(),
            source: "neither a miter join nor a square cap".into(),
        }
    }
}

/// A path's **inset box**, `[m, width − m] × [m, height − m]`, as `((left, top), (right,
/// bottom))`. `None` where the box does not read, or the inset leaves no box at all.
pub fn inset_box(element: &Value) -> Option<((f64, f64), (f64, f64))> {
    let side = |key| element.get(key)?.as_i64();
    let (width, height) = (side("width")?, side("height")?);
    let m = path_inset(element);
    (2 * m <= width && 2 * m <= height).then(|| {
        (
            (m as f64, m as f64),
            ((width - m) as f64, (height - m) as f64),
        )
    })
}

/// ADR-0146 §5's clamps on an effect parameter: a `mask`'s `radius` to half the shorter side
/// of the rect it rounds, its `width` and `height` left as they are (at or below zero the
/// mask hides the element, [`hiding_mask`]), and every other parameter into the range the
/// schema states on it.
fn clamp_parameter(
    declared: &Declared<'_>,
    index: usize,
    member: &str,
    value: Resolved,
    numerator: i128,
    denominator: i128,
) -> Resolved {
    let Resolved::Number(number) = value else {
        return value;
    };
    let key = declared.key();
    if member == "mask" {
        match key {
            "width" | "height" => return Resolved::Number(number),
            "radius" => {
                let rect = mask_sides(declared.element, index, numerator, denominator)
                    .or_else(|| sides(declared.element, numerator, denominator));
                return Resolved::Number(number.clamp(0.0, half_shorter(rect)));
            }
            _ => {}
        }
    }
    let property = declared.property;
    let number = property
        .minimum
        .map_or(number, |minimum| number.max(minimum));
    Resolved::Number(
        property
            .maximum
            .map_or(number, |maximum| number.min(maximum)),
    )
}

/// Half the shorter of two sides, at least `0`; unbounded where there are no sides to read.
fn half_shorter(sides: Option<(f64, f64)>) -> f64 {
    match sides {
        Some((width, height)) => (width.min(height) / 2.0).max(0.0),
        None => f64::INFINITY,
    }
}

/// The resolved `width` and `height`, unclamped, where both read as numbers.
fn sides(element: &Value, numerator: i128, denominator: i128) -> Option<(f64, f64)> {
    let side = |key| read(element, key, numerator, denominator)?.ok()?.number();
    Some((side("width")?, side("height")?))
}

/// The resolved `width` and `height` of the `mask` at `index`, unclamped, where its rect
/// states both — the element's own rect is the bare form's, and is [`sides`].
fn mask_sides(
    element: &Value,
    index: usize,
    numerator: i128,
    denominator: i128,
) -> Option<(f64, f64)> {
    let side = |key| {
        effect_parameter(element, index, key)?
            .read(numerator, denominator)
            .ok()?
            .number()
    };
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

/// **A mask that hides the whole element** at an instant (ADR-0146 §5): the position in
/// `effects` of the first plain `mask` whose rect states a `width` or `height` that resolves
/// at or below zero.
///
/// A plain mask keeps only what is inside its shape, and a shape with no positive size has
/// no inside, so it keeps nothing — however its edge is feathered, since a blurred nothing is
/// still nothing. An inverted mask keeps what is outside the shape (ADR-0152 §1), which is
/// everything, and hides nothing. The bare form's rect is the element's own, and is
/// [`painted_box`]'s question. A mask that is non-zero but wholly outside the element also
/// keeps nothing, and is not answered here: that is a picture to look at, not a fact the
/// file states.
pub fn hiding_mask(element: &Value, numerator: i128, denominator: i128) -> Option<usize> {
    effects(element)
        .iter()
        .enumerate()
        .filter(|(_, effect)| {
            effect.get("name").and_then(Value::as_str) == Some("mask")
                && effect.get("invert").and_then(Value::as_bool) != Some(true)
        })
        .find(|(index, _)| {
            mask_sides(element, *index, numerator, denominator)
                .is_some_and(|(width, height)| width <= 0.0 || height <= 0.0)
        })
        .map(|(index, _)| index)
}

/// Why an element draws nothing at one instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Hidden {
    /// Its box — its `width` by its `height` — has no positive size.
    Box,
    /// The plain `mask` at this position in `effects` has no positive size.
    Mask(usize),
}

/// Whether the element's own declared box is what it draws: not on `text`, whose box is a
/// container claim its lines are checked against (ADR-0135), and not where it states none.
fn draws_its_box(element: &Value) -> bool {
    element.get("type").and_then(Value::as_str) != Some("text") && states_a_box(element)
}

/// Why the element draws nothing at `numerator / denominator` ms, or `None` where it draws
/// something: the box first, then the first plain mask that hides it.
pub fn hidden_at(element: &Value, numerator: i128, denominator: i128) -> Option<Hidden> {
    if draws_its_box(element) && painted_box(element, numerator, denominator).is_none() {
        return Some(Hidden::Box);
    }
    hiding_mask(element, numerator, denominator).map(Hidden::Mask)
}

/// **Never painted** (ADR-0146 §6): every frame its own range holds is hidden, and why —
/// each cause once, the box before the masks.
///
/// A frame is hidden where the box has no positive size or a plain mask with a rect of no
/// positive size hides the element ([`hidden_at`]). Decided at the frame instants `render`
/// paints — [`crate::exact::instant_of`] of every frame inside `[start, end)` — through the
/// same reading the painter takes, so `validate` and `render` agree. An element whose range
/// holds no frame at all is not answered here: that is `N-QUANTIZATION`'s fact.
pub fn never_painted(element: &Value, fps: i64) -> Option<Vec<Hidden>> {
    let (Some(start), Some(end)) = (
        element.get("start").and_then(Value::as_i64),
        element.get("end").and_then(Value::as_i64),
    ) else {
        return None;
    };
    if fps <= 0 || end <= start {
        return None;
    }
    let (first, last) = (
        crate::exact::frame_at_or_after(start, fps)?,
        crate::exact::frame_before(end, fps)?,
    );
    if first.frame > last.frame {
        return None;
    }
    // A static box and static masks are one answer for every frame.
    let keyed =
        |owner: &Value| records(owner, "width").is_some() || records(owner, "height").is_some();
    let frames: Vec<i64> = if keyed(element) || effects(element).iter().any(keyed) {
        (first.frame..=last.frame)
            .map(|n| crate::exact::instant_of(n, fps))
            .collect()
    } else {
        vec![start]
    };
    let mut causes: Vec<Hidden> = Vec::new();
    for instant in frames {
        let cause = hidden_at(element, i128::from(instant), 1)?;
        if !causes.contains(&cause) {
            causes.push(cause);
        }
    }
    causes.sort();
    Some(causes)
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
