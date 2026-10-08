//! What every running transition does to the two elements it bridges at one instant: the
//! resolution that happens before painting (ADR-0059, ADR-0150).
//!
//! The rasterizer never hears of a transition. A `crossfade` multiplies the two elements'
//! opacity; a `slide` or `push` moves them by whole frame-space pixels, outside their own
//! transforms; a `wipe` cuts them to complementary sides of one frame-space edge. `frame`'s
//! painter and `query --at` both read this one resolution, so the picture and the caption
//! cannot disagree about where a bridged element is.
//!
//! **One `n` per transition per instant.** `p` is the eased fraction of the derived window,
//! `D` the frame's width (`left`/`right`) or height (`up`/`down`), and `n = floor(p × D)`.
//! Every offset and edge below is computed from that one integer, so a push's two halves
//! stay joined and a wipe's two sides abut on the same pixel column: no seam can form, and
//! integer geometry paints the same on every painter (ADR-0144).

use serde_json::Value;

use crate::model::{Direction, Ease, TransitionAudio, TransitionKind};
use crate::permissive::Loose;
use crate::stack::Stack;
use crate::verbs::query::geometry::Rect;

/// One transition running at the instant.
#[derive(Debug, Clone)]
pub(crate) struct Running {
    /// The transition element's own id.
    pub element: String,
    pub kind: TransitionKind,
    pub direction: Option<Direction>,
    pub from: String,
    pub to: String,
    /// The derived window: the intersection of the two bridged elements' ranges.
    pub start: i64,
    pub end: i64,
    /// The linear fraction of the window, `0` at its start and approaching `1`.
    pub progress: f64,
    /// `n`, the whole pixels travelled — `None` on a crossfade, which travels nowhere.
    pub pixels: Option<i64>,
    frame: (i64, i64),
}

/// Why a transition that claims to be running cannot be resolved.
#[derive(Debug, Clone)]
pub(crate) enum Unusable {
    /// No `kind` at all.
    NoKind,
    /// A `kind` the format does not have.
    Kind(String),
    /// A `wipe`, `slide` or `push` with no readable `direction`.
    Direction(&'static str),
    /// An `ease` that is not the keyframe vocabulary.
    Ease,
    /// `from` or `to` missing.
    Unpaired,
    /// `from` or `to` naming nothing with a range.
    Unresolved { from: String, to: String },
    /// The two never coexist.
    NoWindow { from: String, to: String },
}

/// What the running transitions do to one element. The identity is `fade 1`, no offset and
/// no cut.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Bridge {
    /// What its `opacity` is multiplied by.
    pub fade: f64,
    /// Whole frame-space pixels it is moved by, outside its own transform.
    pub offset: (i64, i64),
    /// The frame-space rectangle it is cut to, which may be empty. `None` for no cut.
    pub cut: Option<Rect>,
}

impl Default for Bridge {
    fn default() -> Bridge {
        Bridge {
            fade: 1.0,
            offset: (0, 0),
            cut: None,
        }
    }
}

impl Bridge {
    /// Whether this moves or cuts the element at all — a crossfade does neither.
    pub(crate) fn moves_or_cuts(&self) -> bool {
        self.offset != (0, 0) || self.cut.is_some()
    }

    /// Another transition's contribution on top of this one. An element can be the `to` of
    /// one transition and the `from` of the next, at the instant their windows meet: fades
    /// multiply, offsets add, cuts intersect.
    fn then(self, other: Bridge) -> Bridge {
        Bridge {
            fade: self.fade * other.fade,
            offset: (
                self.offset.0 + other.offset.0,
                self.offset.1 + other.offset.1,
            ),
            cut: match (self.cut, other.cut) {
                (Some(a), Some(b)) => Some(overlap(a, b)),
                (a, b) => a.or(b),
            },
        }
    }

    /// A frame-space rectangle `clip` moved by this bridge's offset and cut by its cut: the
    /// aperture the element is painted through. `None` where neither the element nor the
    /// transition clips it.
    pub(crate) fn aperture(&self, own: Option<Rect>) -> Option<Rect> {
        let own = own.map(|clip| Rect {
            x: clip.x + self.offset.0,
            y: clip.y + self.offset.1,
            ..clip
        });
        match (own, self.cut) {
            (Some(a), Some(b)) => Some(overlap(a, b)),
            (a, b) => a.or(b),
        }
    }
}

/// The overlap of two rectangles, empty (zero width or height) where they do not meet —
/// unlike [`Rect::intersect`], a cut that excludes everything is still a cut.
fn overlap(a: Rect, b: Rect) -> Rect {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);
    Rect {
        x,
        y,
        width: (right - x).max(0),
        height: (bottom - y).max(0),
    }
}

/// Every `transition` element in the document, and what it is doing at `instant`: running,
/// not running (`Ok(None)`), or unusable.
pub(crate) fn at(
    document: &Loose,
    instant: i64,
    frame: (i64, i64),
) -> Vec<(&Value, Result<Option<Running>, Unusable>)> {
    let stack = Stack::of(document);
    document
        .elements_in_tracks()
        .map(|(_, element)| element)
        .filter(|element| element.get("type").and_then(Value::as_str) == Some("transition"))
        .map(|element| (element, running(&stack, element, instant, frame)))
        .collect()
}

/// The running ones alone, for a reader that has nothing to say about the rest.
pub(crate) fn running_at(document: &Loose, instant: i64, frame: (i64, i64)) -> Vec<Running> {
    at(document, instant, frame)
        .into_iter()
        .filter_map(|(_, outcome)| outcome.ok().flatten())
        .collect()
}

/// What every running transition does to the element called `id`.
pub(crate) fn bridge_of(running: &[Running], id: &str) -> Bridge {
    running
        .iter()
        .filter_map(|transition| transition.bridge(id))
        .fold(Bridge::default(), Bridge::then)
}

fn running(
    stack: &Stack<'_>,
    element: &Value,
    instant: i64,
    frame: (i64, i64),
) -> Result<Option<Running>, Unusable> {
    let kind = match element.get("kind") {
        None => return Err(Unusable::NoKind),
        Some(kind) => serde_json::from_value::<TransitionKind>(kind.clone()).map_err(|_| {
            Unusable::Kind(
                kind.as_str()
                    .map(str::to_string)
                    .unwrap_or(kind.to_string()),
            )
        })?,
    };
    let direction = match kind {
        TransitionKind::Crossfade | TransitionKind::AudioCrossfade => None,
        _ => Some(
            element
                .get("direction")
                .and_then(|value| serde_json::from_value::<Direction>(value.clone()).ok())
                .ok_or(Unusable::Direction(kind.as_str()))?,
        ),
    };
    let ease = match (kind, element.get("ease")) {
        (TransitionKind::Crossfade | TransitionKind::AudioCrossfade, _) | (_, None) => None,
        (_, Some(ease)) => {
            Some(serde_json::from_value::<Ease>(ease.clone()).map_err(|_| Unusable::Ease)?)
        }
    };
    let (Some(from), Some(to)) = (
        element.get("from").and_then(Value::as_str),
        element.get("to").and_then(Value::as_str),
    ) else {
        return Err(Unusable::Unpaired);
    };
    let (Some(from_range), Some(to_range)) = (
        stack.placement(from).and_then(|placement| placement.range),
        stack.placement(to).and_then(|placement| placement.range),
    ) else {
        return Err(Unusable::Unresolved {
            from: from.to_string(),
            to: to.to_string(),
        });
    };

    // The window is derived, not declared (ADR-0059): `E-TRANSITION-RANGE` reports a drift,
    // and the picture is drawn over the intersection the two elements actually share.
    let start = from_range.start.max(to_range.start);
    let end = from_range.end.min(to_range.end);
    if end <= start {
        return Err(Unusable::NoWindow {
            from: from.to_string(),
            to: to.to_string(),
        });
    }
    if instant < start || instant >= end {
        return Ok(None);
    }

    let progress = (instant - start) as f64 / (end - start) as f64;
    let pixels = direction.map(|direction| {
        let eased = match &ease {
            Some(ease) => crate::resolve::eased(ease, progress),
            None => progress,
        };
        (eased * travel(direction, frame) as f64).floor() as i64
    });
    Ok(Some(Running {
        element: crate::checks::subject_of(element.get("id").and_then(Value::as_str)),
        kind,
        direction,
        from: from.to_string(),
        to: to.to_string(),
        start,
        end,
        progress,
        pixels,
        frame,
    }))
}

/// `D`: the frame's width for a horizontal direction, its height for a vertical one.
fn travel(direction: Direction, (width, height): (i64, i64)) -> i64 {
    match direction {
        Direction::Left | Direction::Right => width,
        Direction::Up | Direction::Down => height,
    }
}

/// The unit step of the direction of travel, in frame space (`y` grows downward).
fn unit(direction: Direction) -> (i64, i64) {
    match direction {
        Direction::Left => (-1, 0),
        Direction::Right => (1, 0),
        Direction::Up => (0, -1),
        Direction::Down => (0, 1),
    }
}

impl Running {
    /// This transition's contribution to the element called `id`, if it bridges it.
    fn bridge(&self, id: &str) -> Option<Bridge> {
        // An `audio_crossfade` paints nothing (ADR-0176).
        if self.kind == TransitionKind::AudioCrossfade {
            return None;
        }
        let incoming = if id == self.to {
            true
        } else if id == self.from {
            false
        } else {
            return None;
        };
        let (Some(direction), Some(n)) = (self.direction, self.pixels) else {
            // A crossfade: the linear opacity ramp, unchanged since ADR-0059.
            return Some(Bridge {
                fade: match incoming {
                    true => self.progress,
                    false => 1.0 - self.progress,
                },
                ..Bridge::default()
            });
        };
        let d = travel(direction, self.frame);
        let (ux, uy) = unit(direction);
        let along = |distance: i64| (ux * distance, uy * distance);
        Some(match (self.kind, incoming) {
            // `to` starts one whole frame back against the travel and arrives at `p = 1`.
            (TransitionKind::Slide | TransitionKind::Push, true) => Bridge {
                offset: along(-(d - n)),
                ..Bridge::default()
            },
            (TransitionKind::Slide, false) => Bridge::default(),
            // `from` leads by `n`, so the two stay joined.
            (TransitionKind::Push, false) => Bridge {
                offset: along(n),
                ..Bridge::default()
            },
            (TransitionKind::Wipe, incoming) => Bridge {
                cut: Some(self.side(direction, n.clamp(0, d), incoming)),
                ..Bridge::default()
            },
            // Unreachable: a crossfade has no direction and returned above.
            (TransitionKind::Crossfade | TransitionKind::AudioCrossfade, _) => Bridge::default(),
        })
    }

    /// A wipe's two complementary sides of the edge at `D − n` from the frame edge it
    /// travels towards: the side the edge has passed (`to`'s) or not yet reached (`from`'s).
    fn side(&self, direction: Direction, n: i64, passed: bool) -> Rect {
        let (width, height) = self.frame;
        // The edge's coordinate, and whether the passed side is the one beyond it.
        let (edge, passed_is_beyond) = match direction {
            Direction::Left => (width - n, true),
            Direction::Right => (n, false),
            Direction::Up => (height - n, true),
            Direction::Down => (n, false),
        };
        let beyond = passed == passed_is_beyond;
        match direction {
            Direction::Left | Direction::Right => match beyond {
                true => Rect {
                    x: edge,
                    y: 0,
                    width: width - edge,
                    height,
                },
                false => Rect {
                    x: 0,
                    y: 0,
                    width: edge,
                    height,
                },
            },
            Direction::Up | Direction::Down => match beyond {
                true => Rect {
                    x: 0,
                    y: edge,
                    width,
                    height: height - edge,
                },
                false => Rect {
                    x: 0,
                    y: 0,
                    width,
                    height: edge,
                },
            },
        }
    }
}

/// One side of a transition's sound on one element (ADR-0176): the `afade` the mix writes
/// for it and the gain `query --at` prints, both read from this one value.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AudioFade {
    /// The transition element's id.
    pub transition: String,
    /// `to` fades in; `from` fades out.
    pub incoming: bool,
    /// The transition's own window, on the timeline.
    pub start: i64,
    pub end: i64,
    pub curve: TransitionAudio,
}

impl AudioFade {
    /// The `afade` curve name: `qsin` is `sin(π/2·p)` and `tri` is `p`.
    pub fn afade_curve(&self) -> Option<&'static str> {
        match self.curve {
            TransitionAudio::Cut => None,
            TransitionAudio::ConstantPower => Some("qsin"),
            TransitionAudio::ConstantGain => Some("tri"),
        }
    }

    /// The gain this side applies at `instant` on the timeline, as `afade` defines it.
    #[allow(dead_code)] // read by `query --at` (S4)
    pub fn gain_at(&self, instant: i64) -> f64 {
        let p = ((instant - self.start) as f64 / (self.end - self.start) as f64).clamp(0.0, 1.0);
        let p = if self.incoming { p } else { 1.0 - p };
        match self.curve {
            TransitionAudio::Cut => 1.0,
            TransitionAudio::ConstantPower => (std::f64::consts::FRAC_PI_2 * p).sin(),
            TransitionAudio::ConstantGain => p,
        }
    }
}

/// Every audio fade of every transition in the document that names `id`, in window order.
/// A transition with `audio` absent or `cut` contributes none: the hard cut adds no stage.
pub(crate) fn audio_fades(document: &Loose, id: &str) -> Vec<AudioFade> {
    let mut fades: Vec<AudioFade> = document
        .elements_in_tracks()
        .map(|(_, element)| element)
        .filter(|element| element.get("type").and_then(Value::as_str) == Some("transition"))
        .filter_map(|element| {
            let curve = serde_json::from_value::<TransitionAudio>(element.get("audio")?.clone())
                .ok()
                .filter(|curve| *curve != TransitionAudio::Cut)?;
            let incoming = match (
                element.get("from").and_then(Value::as_str),
                element.get("to").and_then(Value::as_str),
            ) {
                (Some(from), Some(to)) if from != to && to == id => true,
                (Some(from), Some(to)) if from != to && from == id => false,
                _ => return None,
            };
            Some(AudioFade {
                transition: crate::checks::subject_of(element.get("id").and_then(Value::as_str)),
                incoming,
                start: element.get("start").and_then(Value::as_i64)?,
                end: element.get("end").and_then(Value::as_i64)?,
                curve,
            })
        })
        .filter(|fade| fade.end > fade.start)
        .collect();
    fades.sort_by_key(|fade| fade.start);
    fades
}
