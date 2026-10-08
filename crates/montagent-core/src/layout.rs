//! Canonical key order: **one** predicate, one rewrite, and one place the rule lives.
//!
//! ADR-0041 fixed canonical key order as *"a universal prefix, then each type's property
//! order in the published schema"*, and then said the thing this module exists to make
//! true: `fmt` and `validate`'s `LAYOUT` check *"share one implementation of 'what does
//! canonical form look like' … so there is exactly one place the rule lives, not two that
//! can disagree."* The order itself is read out of [`crate::schema`] rather than written
//! down again here, so the count of places it can go stale stays at one — the types.
//!
//! The two callers want opposite things from the same rule, and getting them from two
//! functions is how they drift. So there is one function, [`reorder`], and the predicate
//! [`is_canonical`] is *defined as* "reordering changes nothing". They cannot disagree,
//! because one is the other.
//!
//! Three properties are structural rather than checked, and each discharges an ADR:
//!
//! - **No key is added and none is removed** (ADR-0030). [`reorder`] permutes the entries
//!   it was handed; there is no path in it that inserts a default or drops a key written
//!   at one.
//! - **No value is touched** (ADR-0026). Both `cover` and `contain` are true at an exact
//!   aspect match, and a formatter that picked one would be making a semantic decision it
//!   has no basis for. This one moves keys.
//! - **A shape it does not recognise is left exactly as written.** An element whose `type`
//!   is absent or unknown — which is every element in a file mid-edit — has no published
//!   order, so it keeps its own. A formatter that invented one would damage the file it
//!   was pointed at to repair (ADR-0042).
//!
//! Keys the schema does not declare — an unknown key, which ADR-0017 makes an `error` and
//! ADR-0042 nevertheless requires `fmt` to format around — have no canonical position, so
//! they keep their relative order and follow the declared ones. Appending is the only
//! choice that is stable under a second run.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde_json::{Map, Value};

/// One structure the published schema declares a key order for.
///
/// Canonical key order is a property of *where* a value sits, not of what it contains: the
/// same five keys mean different orders on an `image` and on an `audio`. Naming the
/// structure at the call site is what keeps the rule readable off one table.
///
/// Deliberately **not** called `Shape`: `CONTEXT.md` spends that word on a drawn primitive
/// — a `rect` or an `ellipse` — and `Shape::Element("rect")` would be a Shape holding a
/// shape. The same near-miss `CONTEXT.md` records for `anchor` against `origin`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Published<'a> {
    /// The document's own header — `frame, fps, background, …, tracks`.
    Project,
    /// One track — `name, layer, elements`.
    Track,
    /// One element, by its `type` string. ADR-0041's universal prefix is already the
    /// front of every branch's property list in the published schema, so there is no
    /// second rule here for it.
    Element(&'a str),
    /// One member of an element's `effects` list, by its `name` (#774). ADR-0163 §3 states
    /// the mask's: `name, shape, x, y, width, height, radius, points, invert, feather`.
    Effect(&'a str),
}

impl Published<'_> {
    /// The shape of one element, read off its own `type`.
    ///
    /// A value that is not an object, or carries no string `type`, still answers here —
    /// [`canonical_order`] answers `None` for it and everything downstream leaves it
    /// alone. That is the mid-edit file ADR-0042 insists stays formattable.
    pub fn of_element(element: &Value) -> Published<'_> {
        Published::Element(element.get("type").and_then(Value::as_str).unwrap_or(""))
    }

    /// The shape of one `effects` member, read off its own `name` — the same leniency as
    /// [`Published::of_element`]: a member with no string `name` has no published order.
    pub fn of_effect(member: &Value) -> Published<'_> {
        Published::Effect(member.get("name").and_then(Value::as_str).unwrap_or(""))
    }
}

/// The canonical key order for one shape, as the published schema declares it, or `None`
/// where the schema declares none.
///
/// `None` is not a failure. It is "this is not a shape Montagent publishes an order for",
/// and every caller's correct response to it is to leave the object as written.
pub fn canonical_order(published: Published<'_>) -> Option<&'static [String]> {
    let orders = orders();
    match published {
        Published::Project => Some(&orders.project),
        Published::Track => Some(&orders.track),
        Published::Element(type_name) => orders.elements.get(type_name).map(Vec::as_slice),
        Published::Effect(name) => orders.effects.get(name).map(Vec::as_slice),
    }
}

/// The same object, its keys in canonical order.
///
/// **The one implementation.** `fmt` writes what this returns; `validate`'s `LAYOUT` check
/// asks [`is_canonical`], which is this function compared against its own input. Nothing
/// else in the tree may re-derive the order.
pub fn reorder(published: Published<'_>, object: &Map<String, Value>) -> Map<String, Value> {
    let Some(order) = canonical_order(published) else {
        return object.clone();
    };

    let mut out = Map::with_capacity(object.len());
    for key in order {
        // Present-only. ADR-0030: a key the document does not carry is a declaration —
        // "give me whatever the default is" — and materialising it here would answer a
        // question the author deliberately left open.
        if let Some(value) = object.get(key) {
            out.insert(key.clone(), value.clone());
        }
    }
    // Whatever the schema does not declare, in the order the file wrote it. ADR-0017 makes
    // an unknown key an `error`; ADR-0042 makes formatting the file around it `fmt`'s job
    // anyway, and the two hold at once only if the key survives.
    for (key, value) in object {
        if !out.contains_key(key) {
            out.insert(key.clone(), value.clone());
        }
    }
    out
}

/// Is this object's key order canonical?
///
/// Defined as "[`reorder`] would change nothing", rather than as a second traversal that
/// answers the same question — which is exactly the two-implementations-one-rule shape
/// ADR-0041 was written against.
pub fn is_canonical(published: Published<'_>, object: &Map<String, Value>) -> bool {
    reorder(published, object).keys().eq(object.keys())
}

/// The whole document in canonical convention: every structure the schema publishes an
/// order for reordered, every track's elements sorted by `start`, and everything else left
/// exactly as it was written.
///
/// Four structures: the header, each track, each element, and each member of an element's
/// `effects` list (#774) — the one nested object whose key order a decision states
/// (ADR-0163 §3 for a mask), and whose order the schema publishes branch by branch. Any
/// other object nested inside an element — a keyframe, a run, a vertex, and the keyframe
/// records inside a member — keeps the order it was written in: ADR-0041 scopes its rule to
/// *"within an element only"*, and extending it further down would be this module inventing
/// format. The members themselves keep their order in the list, which is significant
/// (ADR-0040): only the keys inside each one move.
pub fn canonicalise(document: &Value) -> Value {
    walk(document, Reach::WholeDocument)
}

/// The same, leaving each element's own key order as written — the header, the tracks, the
/// element sort, each `effects` member's keys and the line layout canonical, each element's
/// own keys untouched.
///
/// Not a second convention. It exists so a report can subtract what `L-KEY-ORDER` already
/// says: compared against the file as written, what is left is exactly the part of a
/// rewrite that no element-level finding names. A caller told both would be reading one
/// fact twice, and ADR-0006's noise budget is explicit that a check free to run and
/// expensive to report is still expensive.
pub fn canonicalise_except_elements(document: &Value) -> Value {
    walk(document, Reach::ExceptElements)
}

/// How far a canonicalisation reaches. The header, the tracks and the layout are always
/// made canonical; whether each element's own keys are is what varies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reach {
    WholeDocument,
    ExceptElements,
}

fn walk(document: &Value, reach: Reach) -> Value {
    let Some(root) = document.as_object() else {
        return document.clone();
    };

    let mut out = reorder(Published::Project, root);
    if let Some(Value::Array(tracks)) = out.get("tracks") {
        let tracks: Vec<Value> = tracks
            .iter()
            .map(|track| walk_track(track, reach))
            .collect();
        out.insert("tracks".into(), Value::Array(tracks));
    }
    Value::Object(out)
}

fn walk_track(track: &Value, reach: Reach) -> Value {
    let Some(object) = track.as_object() else {
        return track.clone();
    };

    let mut out = reorder(Published::Track, object);
    if let Some(Value::Array(elements)) = out.get("elements") {
        let mut elements: Vec<Value> = elements
            .iter()
            .map(|element| canonical_element(element, reach))
            .collect();
        sort_by_start(&mut elements);
        out.insert("elements".into(), Value::Array(elements));
    }
    Value::Object(out)
}

/// One track's elements, sorted by `start`.
///
/// ADR-0005's writing convention in full: *"Elements are written sorted by `start` within a
/// track, and formatted one element per line."* ADR-0041 restates it and says it *"does not
/// reopen"* it, so the sort is as much the canonical convention as the key order is. Moving
/// an element's line is safe in the one way that matters: the element's own text is
/// unchanged, so an exact-string replace written against it still matches — and ADR-0060
/// settled that array order carries no meaning, for timing or for stacking, so nothing
/// downstream can read anything off the move.
///
/// **Stable, and by `start` alone.** Two elements starting at the same instant keep the
/// order the file wrote them in, because the document says nothing about which comes first
/// and a tie broken on any other field would make the sort's output depend on a value the
/// author may edit next. An element whose `start` is absent or is not an integer — the
/// mid-edit element ADR-0042 insists stays formattable — sorts last rather than first, so
/// a half-typed element is never hoisted above a complete one.
fn sort_by_start(elements: &mut [Value]) {
    elements.sort_by_key(|element| {
        let start = element.get("start").and_then(Value::as_i64);
        (start.is_none(), start)
    });
}

/// One element: its own keys canonical where `reach` says so, and the keys of each of its
/// `effects` members canonical either way.
///
/// The members are canonical under both reaches because an out-of-order member is not what
/// `L-KEY-ORDER` names — that finding is about the element's own keys and states their
/// expected order — so it must be what `L-LAYOUT`'s comparison sees, or `fmt --check` would
/// stay silent about a rewrite `fmt` then makes.
fn canonical_element(element: &Value, reach: Reach) -> Value {
    let Some(object) = element.as_object() else {
        return element.clone();
    };

    let mut out = match reach {
        Reach::WholeDocument => reorder(Published::of_element(element), object),
        Reach::ExceptElements => object.clone(),
    };
    if let Some(Value::Array(effects)) = out.get("effects") {
        let effects: Vec<Value> = effects.iter().map(canonical_effect).collect();
        out.insert("effects".into(), Value::Array(effects));
    }
    // A nest's tracks are tracks: the same canonical form all the way down (#780).
    if crate::nest::is_nest(element)
        && let Some(Value::Array(tracks)) = out.get("tracks")
    {
        let tracks: Vec<Value> = tracks.iter().map(|track| walk_track(track, reach)).collect();
        out.insert("tracks".into(), Value::Array(tracks));
    }
    Value::Object(out)
}

/// One `effects` member, its keys in the order its `name`'s branch publishes. A member that
/// is not an object, or whose `name` is absent or unknown — a mid-edit member — is left as
/// written, by [`reorder`]'s own rule for a shape with no published order.
fn canonical_effect(member: &Value) -> Value {
    match member.as_object() {
        Some(object) => Value::Object(reorder(Published::of_effect(member), object)),
        None => member.clone(),
    }
}

/// Every published order, read once out of the generated schema.
///
/// Generating the schema costs real work and the answer cannot change within a process, so
/// it is done once. The table is a cache of the schema, never a second statement of it.
struct Orders {
    project: Vec<String>,
    track: Vec<String>,
    elements: BTreeMap<String, Vec<String>>,
    effects: BTreeMap<String, Vec<String>>,
}

fn orders() -> &'static Orders {
    static ORDERS: OnceLock<Orders> = OnceLock::new();
    ORDERS.get_or_init(|| {
        let schema = crate::schema::generate();
        Orders {
            project: keys_of(schema.get("properties")),
            track: keys_of(schema.pointer("/$defs/Track/properties")),
            elements: element_orders(&schema),
            effects: effect_orders(&schema),
        }
    })
}

fn keys_of(properties: Option<&Value>) -> Vec<String> {
    properties
        .and_then(Value::as_object)
        .map(|properties| properties.keys().cloned().collect())
        .unwrap_or_default()
}

/// One entry per branch of the element union, keyed by that branch's `type` const.
fn element_orders(schema: &Value) -> BTreeMap<String, Vec<String>> {
    branch_orders(schema, "/$defs/Element/oneOf", "type")
}

/// One entry per branch of the `Effect` union, keyed by that branch's `name` const.
///
/// The schema declares each branch's fields in the order the model declares them, which is
/// the order serde's writer emits them — with one difference, the tag. `Effect` is an
/// internally tagged enum: serde writes `name` **first**, and schemars lists it **last**,
/// after the variant's fields. So the order here is the branch's properties with `name`
/// hoisted to the front, which is the writer's order exactly (ADR-0163 §3 writes the mask's
/// as `name, shape, …`). `tests/fmt_effects.rs` holds the two together for every member.
fn effect_orders(schema: &Value) -> BTreeMap<String, Vec<String>> {
    branch_orders(schema, "/$defs/Effect/oneOf", "name")
        .into_iter()
        .map(|(name, keys)| {
            let order = std::iter::once("name".to_string())
                .chain(keys.into_iter().filter(|key| key != "name"))
                .collect();
            (name, order)
        })
        .collect()
}

/// Each branch of the discriminated union at `pointer`, keyed by its `tag` const, mapped to
/// its properties in declaration order.
fn branch_orders(schema: &Value, pointer: &str, tag: &str) -> BTreeMap<String, Vec<String>> {
    let mut orders = BTreeMap::new();
    let branches = schema
        .pointer(pointer)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("the published schema has a discriminated union at {pointer}"));

    for branch in branches {
        let Some(properties) = branch.get("properties").and_then(Value::as_object) else {
            continue;
        };
        let Some(type_name) = properties
            .get(tag)
            .and_then(|tag| tag.get("const"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        orders.insert(type_name.to_string(), properties.keys().cloned().collect());
    }
    orders
}
