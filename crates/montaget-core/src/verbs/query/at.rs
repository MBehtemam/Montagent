//! `--at <t>` — the resolved stack at an instant.
//!
//! ADR-0011: *"`query --at <t>` — the resolved stack at an instant"*, under the verb's
//! organising rule — *"`query` returns resolved values, never echoed fields. Echoing
//! `"scale":[[3018,1.0],[18018,1.08]]` back at the agent tells it nothing it did not have;
//! `scale 1.0170` is the entire point."* This is the mode that rule was written about, and
//! [ADR-0070](../../../../docs/adr/0070-the-where-predicate-is-a-conjunction-of-whole-value-terms.md)
//! since bounded it to exactly this mode: `--where` matches what the document writes.
//!
//! Three things are answered here, and each is a thing the shell was measured getting wrong
//! or getting right for the wrong reason:
//!
//! - **Presence** — every element whose half-open range contains the instant (`CONTEXT.md`'s
//!   *Presence set*). ADR-0011's verifier found `jq` does this correctly.
//! - **Painter's order** — from [`crate::stack`], which is ticket 11's resolution and the
//!   only implementation of it. ADR-0011 found the shell one-liner *"silently invents an
//!   order at layer ties"*, and ADR-0060 closed that structurally: a tie whose boxes overlap
//!   is an `error`, so the only ties this mode can meet are ones where *"any consistent
//!   internal order is correct by definition"*. Ascending resolved layer, back to front,
//!   document order within a tie.
//! - **Resolved animated values** — from [`crate::resolve`], the keyframe resolver, which is
//!   likewise the only implementation.
//!
//! ## What is deliberately not here
//!
//! ADR-0011 lists four more components: the offset into the source, the crop rectangle, the
//! ink box, and `NOT COVERED`. Every one of them reaches outside the document — the probed
//! source duration under `overrun`, the source's own dimensions under `fit`, a shaper — and
//! ADR-0011 records them as *"blocked on #21 and on `measure`"* rather than merely expensive.
//! They are [#210](https://github.com/MBehtemam/Montaget/issues/210), whose own acceptance
//! criteria name all four. This mode is the half that reads the document alone, and it opens
//! nothing.
//!
//! ## The surface, and where it is argued
//!
//! ADR-0011 names the components this answer must carry and not the shape it is written in,
//! so the keys below — and the two readings under them — are this ticket's own.
//! [#269](https://github.com/MBehtemam/Montaget/issues/269) carries them for ratification,
//! rather than leaving them to be discovered from this file.
//!
//! ## Why declared properties only, and no defaults
//!
//! ADR-0012 publishes a default for every transform property — `x` and `y` to the frame
//! centre, `scale` to `[1,1]`, `rotation` to `0`, `opacity` to `1`. This view resolves what
//! the document **declares** and synthesises none of them, because ADR-0030 makes a
//! defaultable field's *presence* content: omitted and explicit-at-default are two different
//! declarations, and a row reading `opacity 1` that might mean either would collapse exactly
//! the distinction the format keeps — on the one verb an agent uses to find out what the
//! document says.

use serde::Serialize;
use serde_json::Value;

use crate::model::{Animatable, Scale};
use crate::permissive::Loose;
use crate::resolve::{self, Tween};
use crate::stack::{Stack, Unresolved};

use super::Named;

/// The resolved stack at one instant.
#[derive(Debug, Clone, Serialize)]
pub struct At {
    /// The instant asked about, in absolute milliseconds on the project's one clock.
    pub at: i64,
    /// The presence set, in painter's order: ascending resolved layer, back to front.
    ///
    /// Audio included, like the cut list's — *"audio is an element like any other; nothing
    /// owns it"* (ADR-0001), and a caller wanting only the visual stack filters one field.
    /// The departure from ADR-0011's word *"on-screen"* is the same one, argued in the same
    /// place ([#250](https://github.com/MBehtemam/Montaget/issues/250)).
    pub stack: Vec<Present>,
    /// Elements the document does not place on the clock — no `start`, no `end`, or one of
    /// them written as something other than whole milliseconds.
    ///
    /// Named rather than dropped, for the cut list's reason: a stack silently computed over
    /// 58 of 60 elements is a wrong answer that looks like a right one, and *which* way the
    /// range is malformed is `validate`'s question and not a view's.
    pub unplaced: Vec<String>,
}

/// One element of the presence set, resolved.
#[derive(Debug, Clone, Serialize)]
pub struct Present {
    #[serde(flatten)]
    pub named: Named,
    pub start: i64,
    pub end: i64,
    /// Where it draws, as one integer, resolved in exactly one hop (ADR-0019).
    pub layer: Option<i64>,
    /// Why there is no integer, where there is none. Present and `null` otherwise, so that
    /// "resolved to nothing" is never read off the absence of a key.
    pub layer_unresolved: Option<String>,
    /// Every animated property the element **declares**, resolved at the instant — in the
    /// order the format declares them, never the order the file happens to write them in.
    pub values: Vec<Resolved>,
}

/// One property's value at the instant.
#[derive(Debug, Clone, Serialize)]
pub struct Resolved {
    /// The key, as the document spells it.
    pub property: String,
    /// Whether the document writes this property as a keyframe list. Stated rather than
    /// inferred: `scale [1.0, 1.0]` at an instant is the same row whether the element holds
    /// still or is one millisecond into a fifteen-second ramp, and which of those it is
    /// changes what an author does next.
    pub animated: bool,
    /// The resolved value — **always a number, or a pair of them**, never the records.
    ///
    /// A resolved `x` is a number even where the document states it as an integer and
    /// nothing animates it. Two spellings for one value would put a shape test in every
    /// consumer, which is what ADR-0012 retired the `scale` union for; and an integer here
    /// would publish a rounding rule ADR-0035 says does not exist.
    pub value: Option<Value>,
    /// Why there is no value, where there is none — a property the format cannot read at
    /// all, or a keyframe list that does not fit the schema. The fact is `validate`'s to
    /// judge; this says only that the view could not answer.
    pub unresolved: Option<String>,
}

/// The value shape a property is written in — the one table mapping a key to the type the
/// model declares for it.
///
/// It exists because this mode reads the permissive tree (the representation that always
/// exists, and the one every other check and view already reads) rather than the strict
/// model, so the per-property type cannot come from a struct field. What it does *not*
/// duplicate is any rule: the reading is [`Animatable`]'s own deserializer, and the
/// resolution is [`crate::resolve`].
#[derive(Clone, Copy)]
enum Shape {
    /// `x`, `y` — absolute integer pixels in the document (ADR-0012).
    Pixels,
    /// `rotation`, `opacity`, `volume` — ratios and angles, floats in the document.
    Ratio,
    /// `scale` — always `[sx, sy]`, never a bare number (ADR-0012).
    Pair,
}

/// Every animated property in the format, in the order the model declares them.
///
/// One list rather than one per element type: the key is spelled the same wherever it
/// appears, and a `volume` on an image is a schema error that `validate` reports rather than
/// something this view has to have an opinion about.
const ANIMATED: [(&str, Shape); 6] = [
    ("x", Shape::Pixels),
    ("y", Shape::Pixels),
    ("scale", Shape::Pair),
    ("rotation", Shape::Ratio),
    ("opacity", Shape::Ratio),
    ("volume", Shape::Ratio),
];

/// Build the resolved stack at `instant`.
pub fn at(document: &Loose, instant: i64) -> At {
    let stack = Stack::of(document);
    let mut present: Vec<Present> = Vec::new();
    let mut unplaced: Vec<String> = Vec::new();

    for (index, (track, element)) in document.elements_in_tracks().enumerate() {
        let named = Named::of(element, track);
        let name = named.called(index);
        let (Some(start), Some(end)) = (
            element.get("start").and_then(Value::as_i64),
            element.get("end").and_then(Value::as_i64),
        ) else {
            unplaced.push(name);
            continue;
        };
        // Half-open, so an element ending at the instant is already out of the set and one
        // starting there is already in it (ADR-0005). An inverted or empty range contains no
        // instant at all, which this same test gives for free.
        if !(start <= instant && instant < end) {
            continue;
        }

        let (layer, layer_unresolved) = match &named.id {
            // Resolution is by `id`, so an element without one has no row in the stack's
            // index — a fact about the document, and `validate`'s to report (ADR-0019).
            None => (None, Some("the element states no `id`".to_string())),
            Some(id) => match stack.layer_of(id) {
                Ok(layer) => (Some(layer), None),
                Err(unresolved) => (None, Some(why(unresolved))),
            },
        };

        present.push(Present {
            named,
            start,
            end,
            layer,
            layer_unresolved,
            values: values(element, instant),
        });
    }

    // Painter's order: back to front. Stable, so a tie keeps document order — which the file
    // makes no promise about and nothing checks (ADR-0060), and which is the whole of what
    // this view is permitted to do with one. An element whose layer did not resolve sorts
    // last rather than at zero: it has no place in the stack, and putting it at one would be
    // the invented order ADR-0060 refuses.
    present.sort_by_key(|element| (element.layer.is_none(), element.layer.unwrap_or_default()));

    At {
        at: instant,
        stack: present,
        unplaced,
    }
}

/// Every animated property this element declares, resolved.
fn values(element: &Value, instant: i64) -> Vec<Resolved> {
    ANIMATED
        .iter()
        .filter_map(|(property, shape)| {
            let written = element.get(*property)?;
            Some(match shape {
                Shape::Pixels => resolved::<i64>(property, written, instant),
                Shape::Ratio => resolved::<f64>(property, written, instant),
                Shape::Pair => resolved::<Scale>(property, written, instant),
            })
        })
        .collect()
}

/// One property, read as the format's own type and resolved at the instant.
fn resolved<T>(property: &str, written: &Value, instant: i64) -> Resolved
where
    T: serde::de::DeserializeOwned + Tween,
    T::Out: Serialize,
{
    let unreadable = |animated: bool, reason: String| Resolved {
        property: property.to_string(),
        animated,
        value: None,
        unresolved: Some(reason),
    };

    // The format's own reader, which is also where ADR-0038's positional `ease` rule is
    // enforced — so a keyframe list missing an `ease` on a later record arrives here as
    // unreadable rather than as a value this module had to invent a default to produce.
    let animatable: Animatable<T> = match serde_json::from_value(written.clone()) {
        Ok(animatable) => animatable,
        Err(e) => return unreadable(written.is_array(), e.to_string()),
    };
    let animated = matches!(animatable, Animatable::Keyed(_));

    let Some(value) = resolve::at(&animatable, instant) else {
        return unreadable(animated, "the keyframe list is empty".to_string());
    };
    match serde_json::to_value(value) {
        Ok(value) => Resolved {
            property: property.to_string(),
            animated,
            value: Some(value),
            unresolved: None,
        },
        // A value JSON cannot carry — an infinity or a NaN reached by interpolating one.
        Err(e) => unreadable(animated, e.to_string()),
    }
}

/// Why an element's layer is not an integer, in a sentence.
///
/// The wording is this view's own. [`crate::checks::anchor`] answers the same question with
/// findings carrying stable codes and repairs, which is what a report is for; a view says
/// what it could not answer and sends the reader to `validate` for the verdict.
fn why(unresolved: Unresolved<'_>) -> String {
    match unresolved {
        Unresolved::NoSuchElement => "the project carries no element with this `id`".to_string(),
        Unresolved::MissingTarget(target) => {
            format!("its anchor names `{target}`, which no element in the project carries")
        }
        Unresolved::SelfReference(target) => {
            format!("its anchor names itself (`{target}`)")
        }
        Unresolved::ChainedTarget(target) => format!(
            "its anchor names `{target}`, whose own `layer` is not an integer — an anchor \
             resolves in exactly one hop"
        ),
        Unresolved::Unstated => {
            "neither the element nor its track states an integer `layer`".to_string()
        }
        Unresolved::Malformed => {
            "its `layer` is neither an integer nor an anchor object".to_string()
        }
    }
}
