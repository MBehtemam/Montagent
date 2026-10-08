//! **Nest** (prototype, #780): the one nest-chain resolver, and the flattening that gives
//! every reader of "the elements" a flat list of leaves.
//!
//! A nest is a container for space only (#632): it holds `tracks[]`, has a `start`/`end`
//! window that bounds its presence, and composes one matrix before each child's own
//! transform. It has no clock — children keep timeline ms.
//!
//! **Two views of one document.** [`flatten`] turns the written tree into a document whose
//! nests are gone and whose leaves each carry `_nest`: the chain of enclosing nests,
//! outermost first, as copies of the nest objects. Every reader that asked
//! `Loose::elements_in_tracks` for "every element" keeps working and now sees the leaves of
//! every nest; [`composed`] is the single place a chain turns into a matrix, called by the
//! painter, `query --at`, the geometry functions and `validate`.

use serde_json::{Map, Value};

use crate::verbs::query::geometry::number_at as number_at_instant;

/// The key a flattened leaf carries its chain under. Never written by a user: the schema
/// check reads the original tree, which has no such key.
pub const KEY: &str = "_nest";

/// An affine map `x' = a·x + c·y + e`, `y' = b·x + d·y + f`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Affine {
    pub const IDENTITY: Affine = Affine {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    /// `self` applied after `inner` (`self · inner`).
    pub fn then_inner(self, inner: Affine) -> Affine {
        Affine {
            a: self.a * inner.a + self.c * inner.b,
            b: self.b * inner.a + self.d * inner.b,
            c: self.a * inner.c + self.c * inner.d,
            d: self.b * inner.c + self.d * inner.d,
            e: self.a * inner.e + self.c * inner.f + self.e,
            f: self.b * inner.e + self.d * inner.f + self.f,
        }
    }

    pub fn apply(&self, (x, y): (f64, f64)) -> (f64, f64) {
        (self.a * x + self.c * y + self.e, self.b * x + self.d * y + self.f)
    }

    pub fn is_identity(&self) -> bool {
        *self == Affine::IDENTITY
    }

    pub fn as_array(&self) -> [f64; 6] {
        [self.a, self.b, self.c, self.d, self.e, self.f]
    }

    /// The rotation the matrix carries, in degrees clockwise (y down).
    pub fn rotation_degrees(&self) -> f64 {
        self.b.atan2(self.a).to_degrees()
    }

    /// The scale the matrix carries along each axis.
    pub fn scale(&self) -> (f64, f64) {
        (self.a.hypot(self.b), self.c.hypot(self.d))
    }
}

pub fn is_nest(value: &Value) -> bool {
    value.get("type").and_then(Value::as_str) == Some("nest")
}

/// `translate(x, y) · about(pivot)(rotate · scale)` for one nest at `t`.
pub fn matrix_of(nest: &Value, t: (i128, i128)) -> Affine {
    let x = number_at_instant::<i64>(nest, "x", t, 0.0);
    let y = number_at_instant::<i64>(nest, "y", t, 0.0);
    let rotation = number_at_instant::<f64>(nest, "rotation", t, 0.0);
    let [sx, sy] = number_at_instant::<[f64; 2]>(nest, "scale", t, [1.0, 1.0]);
    let (px, py) = nest
        .get("pivot")
        .and_then(Value::as_array)
        .and_then(|p| Some((p.first()?.as_f64()?, p.get(1)?.as_f64()?)))
        .unwrap_or((0.0, 0.0));
    let (sin, cos) = rotation.to_radians().sin_cos();
    // R · S about the pivot, then the offset.
    let (a, b, c, d) = (cos * sx, sin * sx, -sin * sy, cos * sy);
    Affine {
        a,
        b,
        c,
        d,
        e: px + x - (a * px + c * py),
        f: py + y - (b * px + d * py),
    }
}

/// The chain a flattened leaf carries, outermost first.
pub fn chain(element: &Value) -> &[Value] {
    element
        .get(KEY)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

/// **The one resolver.** The composed matrix of every nest enclosing `element`, at `t`:
/// `None` for an element in no nest (so its paint is exactly what it was before nests).
pub fn composed(element: &Value, t: (i128, i128)) -> Option<Affine> {
    let chain = chain(element);
    if chain.is_empty() {
        return None;
    }
    Some(
        chain
            .iter()
            .fold(Affine::IDENTITY, |outer, nest| outer.then_inner(matrix_of(nest, t))),
    )
}

/// Every nest in the written tree, parent first, with the ids of its enclosing nests.
pub fn nests(root: &Value) -> Vec<(Vec<String>, &Value)> {
    fn walk<'a>(tracks: &'a Value, chain: &mut Vec<String>, out: &mut Vec<(Vec<String>, &'a Value)>) {
        for track in tracks.as_array().map(Vec::as_slice).unwrap_or_default() {
            for element in track["elements"].as_array().map(Vec::as_slice).unwrap_or_default() {
                if is_nest(element) {
                    out.push((chain.clone(), element));
                    chain.push(element["id"].as_str().unwrap_or("(nest with no id)").to_string());
                    walk(&element["tracks"], chain, out);
                    chain.pop();
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(&root["tracks"], &mut Vec::new(), &mut out);
    out
}

/// Whether the written document holds any nest.
pub fn any(root: &Value) -> bool {
    !nests(root).is_empty()
}

/// Every element of the written tree including nests, document order, depth first.
pub fn all_elements<'a>(root: &'a Value) -> Vec<(Option<&'a str>, &'a Value)> {
    fn walk<'a>(tracks: &'a Value, out: &mut Vec<(Option<&'a str>, &'a Value)>) {
        for track in tracks.as_array().map(Vec::as_slice).unwrap_or_default() {
            let name = track.get("name").and_then(Value::as_str);
            for element in track["elements"].as_array().map(Vec::as_slice).unwrap_or_default() {
                out.push((name, element));
                if is_nest(element) {
                    walk(&element["tracks"], out);
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(&root["tracks"], &mut out);
    out
}

/// The document with its nests expanded: every nested track becomes a track of the root and
/// every leaf below a nest carries `_nest`. `None` where there is no nest to expand.
pub fn flatten(root: &Value) -> Option<Value> {
    if !any(root) {
        return None;
    }
    fn expand(tracks: &Value, chain: &[Value], out: &mut Vec<Value>) {
        for track in tracks.as_array().map(Vec::as_slice).unwrap_or_default() {
            let mut kept = Vec::new();
            let mut inner: Vec<(Vec<Value>, &Value)> = Vec::new();
            for element in track["elements"].as_array().map(Vec::as_slice).unwrap_or_default() {
                if is_nest(element) {
                    let mut below = chain.to_vec();
                    let mut descriptor = element.clone();
                    if let Some(map) = descriptor.as_object_mut() {
                        map.remove("tracks");
                    }
                    below.push(descriptor);
                    inner.push((below, &element["tracks"]));
                } else {
                    let mut leaf = element.clone();
                    if !chain.is_empty()
                        && let Some(map) = leaf.as_object_mut()
                    {
                        map.insert(KEY.to_string(), Value::Array(chain.to_vec()));
                    }
                    kept.push(leaf);
                }
            }
            let mut flat = Map::new();
            for (key, value) in track.as_object().into_iter().flatten() {
                if key != "elements" {
                    flat.insert(key.clone(), value.clone());
                }
            }
            flat.insert("elements".to_string(), Value::Array(kept));
            out.push(Value::Object(flat));
            for (below, tracks) in inner {
                expand(tracks, &below, out);
            }
        }
    }
    let mut tracks = Vec::new();
    expand(&root["tracks"], &[], &mut tracks);
    let mut flat = root.clone();
    flat["tracks"] = Value::Array(tracks);
    Some(flat)
}
