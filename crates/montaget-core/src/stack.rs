//! Anchor resolution: where each element sits in the stack, as one integer, in one hop.
//!
//! ADR-0019 settled the anchor's three open questions — its target is an element `id` and
//! never a track name, it resolves in **exactly one hop**, and `validate` checks both a
//! target that cannot be resolved and one that can but never overlaps. The rule it states
//! for the hop is structural rather than arithmetic: *"an anchor's target's own `layer`
//! must not itself be an object. No walk, no possibility of a cycle by construction."*
//!
//! This module owns that resolution, and it is owned here rather than by either caller
//! because there are two — the keyframe resolver asks *what draws in front at this
//! instant*, and the rasterizer asks *in what order do I paint* — and a rule with two
//! implementations is the drift ADR-0041 names in its own domain and this project has now
//! found four times. [`crate::checks::anchor`] is a third caller: the checks are this
//! function asked the same question and told to report what it could not answer.
//!
//! **Named for the stack, not for the layer.** `CONTEXT.md` defines a layer as *"a place in
//! the stack"*, so either word is available — and `layer.rs` would sit one letter from
//! [`crate::layout`], which is a different subject entirely (canonical key order). The
//! near-miss `CONTEXT.md` already records between `anchor` and `origin` is the same hazard.
//!
//! Two representations reach it. [`Stack::of`] reads the permissive tree, which is what
//! every check has and what a mid-edit document can always produce; [`Stack::of_project`]
//! reads the format's types, which is what a render has. They fill the same rows, so the
//! answer cannot depend on which door a caller came through.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::model::{Anchor, Layer, Project};
use crate::permissive::Loose;

/// Which side of its target an anchor sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Below,
    Above,
}

impl Side {
    /// The layer this side resolves to, given the target's own integer layer.
    ///
    /// Saturating rather than wrapping: a project anchored below `i64::MIN` has larger
    /// problems than the arithmetic, and wrapping to the top of the stack is the one answer
    /// that would be silently wrong on a frame.
    pub fn against(self, target_layer: i64) -> i64 {
        match self {
            Side::Below => target_layer.saturating_sub(1),
            Side::Above => target_layer.saturating_add(1),
        }
    }

    /// The key the document writes, which is also what a finding names.
    pub fn as_str(self) -> &'static str {
        match self {
            Side::Below => "below",
            Side::Above => "above",
        }
    }
}

/// An element's half-open time range, `[start, end)` (ADR-0005).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: i64,
    pub end: i64,
}

impl Span {
    /// Do these two ranges share any instant?
    ///
    /// Half-open, so an element ending at 7500 and its neighbour starting at 7500 do not
    /// overlap — the boundary instant belongs to exactly one of them (ADR-0005). An empty
    /// or inverted range overlaps nothing, including itself: there is no instant at which
    /// it is on screen, so there is none at which its stacking could matter.
    pub fn overlaps(self, other: Span) -> bool {
        self.start < self.end
            && other.start < other.end
            && self.start < other.end
            && other.start < self.end
    }
}

/// What one element's document says about where it sits, before anything is resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Own<'a> {
    /// No override: the element takes its track's layer, which is the normal case.
    Track,
    Absolute(i64),
    Anchor {
        side: Side,
        target: &'a str,
    },
    /// A `layer` that is neither an integer nor a well-formed anchor. Reachable only
    /// through the permissive tree, and it is a *schema* error — resolution reports that it
    /// cannot answer and says nothing about the key, which belongs to the check that owns
    /// the schema.
    Malformed,
}

/// One element, as the stack sees it.
#[derive(Debug, Clone, Copy)]
pub struct Placement<'a> {
    pub id: &'a str,
    /// The track the element sits in, where the document names one. Carried because a
    /// finding's location does.
    pub track: Option<&'a str>,
    /// The element's own time range, where the document states one in integer
    /// milliseconds. Absent on an element mid-edit, and an overlap that cannot be computed
    /// is never reported as an overlap that is not there.
    pub span: Option<Span>,
    own: Own<'a>,
    track_layer: Option<i64>,
}

impl<'a> Placement<'a> {
    /// The anchor this element carries, if it carries one.
    pub fn anchor(&self) -> Option<(Side, &'a str)> {
        match self.own {
            Own::Anchor { side, target } => Some((side, target)),
            _ => None,
        }
    }

    /// The integer this element states directly, from its own override or from its track.
    /// `None` where it states an anchor instead, or states nothing an integer can be read
    /// from — which is exactly the condition an anchor's target may not be in.
    fn stated(&self) -> Option<i64> {
        match self.own {
            Own::Absolute(layer) => Some(layer),
            Own::Track => self.track_layer,
            Own::Anchor { .. } | Own::Malformed => None,
        }
    }
}

/// Why an element's layer is not an integer.
///
/// Three of these are the errors ADR-0019 specifies and one is its `review`; the remaining
/// two are *"the document does not say, and saying so is another check's job"* — resolution
/// reports what it could not answer rather than guessing or panicking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unresolved<'a> {
    /// No element in the document carries this `id`.
    NoSuchElement,
    /// The anchor names an element that is not in the document.
    MissingTarget(&'a str),
    /// The anchor names the element that carries it.
    SelfReference(&'a str),
    /// The target's own layer is itself an anchor. ADR-0019: one hop, never a walk.
    ChainedTarget(&'a str),
    /// Neither the element nor its track states an integer layer.
    Unstated,
    /// A `layer` value that is neither an integer nor a well-formed anchor, here or on the
    /// target. A schema error, reported by whatever check owns the schema.
    Malformed,
}

/// Every element's place in the stack, indexed by `id`.
///
/// Built once per document and read many times: resolution is a lookup by construction
/// (ADR-0019's one hop), and rebuilding the index per element would turn the flat read the
/// whole decision protects into a quadratic one.
#[derive(Debug, Clone, Default)]
pub struct Stack<'a> {
    placements: Vec<Placement<'a>>,
    by_id: BTreeMap<&'a str, usize>,
}

impl<'a> Stack<'a> {
    /// The stack a permissive document describes.
    ///
    /// Reads the raw tree rather than the format's types, because a document that does not
    /// strictly parse is exactly the one an anchor check is most wanted on — and because
    /// this is the spine every check already reads.
    pub fn of(document: &'a Loose) -> Stack<'a> {
        let mut stack = Stack::default();
        for track in document
            .value()
            .get("tracks")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let name = track.get("name").and_then(Value::as_str);
            let track_layer = track.get("layer").and_then(Value::as_i64);
            for element in track
                .get("elements")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let Some(id) = element.get("id").and_then(Value::as_str) else {
                    // Nothing can name it, so nothing can anchor to it and no finding could
                    // be located at it. `id` is required, and the check that says so is the
                    // schema's.
                    continue;
                };
                stack.push(Placement {
                    id,
                    track: name,
                    span: span_of(element.get("start"), element.get("end")),
                    own: own_of(element.get("layer")),
                    track_layer,
                });
            }
        }
        stack
    }

    /// The same stack, read off the format's types — what a render has in hand.
    pub fn of_project(project: &'a Project) -> Stack<'a> {
        let mut stack = Stack::default();
        for track in &project.tracks {
            for element in &track.elements {
                stack.push(Placement {
                    id: &element.id,
                    track: Some(&track.name),
                    span: Some(Span {
                        start: element.start,
                        end: element.end,
                    }),
                    own: match &element.layer {
                        None => Own::Track,
                        Some(Layer::Absolute(layer)) => Own::Absolute(*layer),
                        Some(Layer::Relative(anchor)) => Own::Anchor {
                            side: match anchor {
                                Anchor::Below(_) => Side::Below,
                                Anchor::Above(_) => Side::Above,
                            },
                            target: anchor.target(),
                        },
                    },
                    track_layer: Some(track.layer),
                });
            }
        }
        stack
    }

    /// Every element, in document order.
    ///
    /// Document order is a traversal and never a ranking: ADR-0060 settled that array order
    /// carries no meaning, for timing or for stacking.
    pub fn placements(&self) -> impl Iterator<Item = &Placement<'a>> {
        self.placements.iter()
    }

    /// One element's placement, by `id`.
    pub fn placement(&self, id: &str) -> Option<&Placement<'a>> {
        self.by_id.get(id).map(|&i| &self.placements[i])
    }

    /// **The resolution function.** One element's layer as an integer, in exactly one hop.
    ///
    /// Every failure is named rather than folded into an `Option`, because the caller that
    /// matters most is a check whose whole job is to say *which* of them happened.
    pub fn layer_of(&self, id: &str) -> Result<i64, Unresolved<'a>> {
        let Some(placement) = self.placement(id) else {
            return Err(Unresolved::NoSuchElement);
        };

        let Some((side, target)) = placement.anchor() else {
            return match placement.own {
                Own::Malformed => Err(Unresolved::Malformed),
                _ => placement.stated().ok_or(Unresolved::Unstated),
            };
        };

        // Checked before the lookup, because an element is in its own index: resolved by
        // lookup first, a self-reference would come back as a chain — a true statement that
        // names the wrong defect and sends the author to the wrong line.
        if target == placement.id {
            return Err(Unresolved::SelfReference(target));
        }
        let Some(target_placement) = self.placement(target) else {
            return Err(Unresolved::MissingTarget(target));
        };
        if target_placement.anchor().is_some() {
            return Err(Unresolved::ChainedTarget(target));
        }
        if target_placement.own == Own::Malformed {
            return Err(Unresolved::Malformed);
        }

        // One hop, and it ends here: the target states an integer or it states nothing, and
        // neither branch looks at a third element.
        target_placement
            .stated()
            .map(|layer| side.against(layer))
            .ok_or(Unresolved::Unstated)
    }

    /// Every element's resolved layer, in document order — draw order, as far as this
    /// document determines it.
    ///
    /// What it does *not* determine is a tie: two elements resolving to one layer is legal
    /// while their boxes never meet, and an `error` once they do (ADR-0060). That check
    /// samples across keyframes and belongs to the ticket that can.
    pub fn resolved(&self) -> impl Iterator<Item = (&'a str, Result<i64, Unresolved<'a>>)> {
        self.placements
            .iter()
            .map(|placement| (placement.id, self.layer_of(placement.id)))
    }

    fn push(&mut self, placement: Placement<'a>) {
        // First writing wins. A duplicate `id` breaks the uniqueness ADR-0019 requires, and
        // the check that says so is the schema's; resolving against the first is stable and
        // is what a reader scanning the file top-down would do.
        self.by_id
            .entry(placement.id)
            .or_insert(self.placements.len());
        self.placements.push(placement);
    }
}

fn span_of(start: Option<&Value>, end: Option<&Value>) -> Option<Span> {
    Some(Span {
        start: start?.as_i64()?,
        end: end?.as_i64()?,
    })
}

/// What an element's own `layer` key says, whatever is written there.
fn own_of(layer: Option<&Value>) -> Own<'_> {
    match layer {
        None => Own::Track,
        Some(Value::Number(n)) => n.as_i64().map(Own::Absolute).unwrap_or(Own::Malformed),
        Some(Value::Object(object)) if object.len() == 1 => {
            let (key, value) = object.iter().next().expect("one entry");
            match (key.as_str(), value.as_str()) {
                ("below", Some(target)) => Own::Anchor {
                    side: Side::Below,
                    target,
                },
                ("above", Some(target)) => Own::Anchor {
                    side: Side::Above,
                    target,
                },
                _ => Own::Malformed,
            }
        }
        Some(_) => Own::Malformed,
    }
}
