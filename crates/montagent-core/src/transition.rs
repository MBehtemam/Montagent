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

use montagent_render::canvas::AperturePath;
use serde_json::Value;

use crate::model::{Direction, Ease, IrisShape, TransitionAudio, TransitionKind};
use crate::permissive::Loose;
use crate::stack::Stack;
use crate::verbs::query::geometry::Rect;

/// The frame-space region an element is cut to by its own `clip` and the running
/// transitions (ADR-0220 §5).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Aperture {
    /// A single axis-aligned integer rectangle (`clip`, cardinal `wipe`, or rectangular `iris`).
    Rect(Rect),
    /// A union of axis-aligned integer rectangles (`blinds` slats, or the complement of a
    /// rectangular `iris`).
    MultiRect(Vec<Rect>),
    /// A non-rectangular geometric contour bounded by its conservative axis-aligned frame
    /// rectangle (`wipe` with `angle`, circular `iris`, or `radial_wipe`).
    Path {
        bounds: Rect,
        shape: AperturePath,
        invert: bool,
    },
}

impl Aperture {
    /// The conservative axis-aligned bounding box of this aperture in frame space (ADR-0220 §4).
    pub(crate) fn bounds(&self) -> Rect {
        match self {
            Aperture::Rect(rect) => *rect,
            Aperture::MultiRect(rects) => {
                let mut non_empty = rects.iter().filter(|r| r.width > 0 && r.height > 0);
                let Some(first) = non_empty.next() else {
                    return rects.first().copied().unwrap_or(Rect {
                        x: 0,
                        y: 0,
                        width: 0,
                        height: 0,
                    });
                };
                let mut left = first.x;
                let mut top = first.y;
                let mut right = first.x + first.width;
                let mut bottom = first.y + first.height;
                for rect in non_empty {
                    left = left.min(rect.x);
                    top = top.min(rect.y);
                    right = right.max(rect.x + rect.width);
                    bottom = bottom.max(rect.y + rect.height);
                }
                Rect {
                    x: left,
                    y: top,
                    width: (right - left).max(0),
                    height: (bottom - top).max(0),
                }
            }
            Aperture::Path { bounds, .. } => *bounds,
        }
    }

    /// Intersect this aperture with an axis-aligned rectangle `clip`.
    pub(crate) fn intersect_rect(self, clip: Rect) -> Aperture {
        match self {
            Aperture::Rect(rect) => Aperture::Rect(overlap(rect, clip)),
            Aperture::MultiRect(rects) => {
                let clipped: Vec<Rect> = rects
                    .into_iter()
                    .map(|rect| overlap(rect, clip))
                    .filter(|rect| rect.width > 0 && rect.height > 0)
                    .collect();
                if clipped.is_empty() {
                    Aperture::Rect(Rect {
                        x: clip.x,
                        y: clip.y,
                        width: 0,
                        height: 0,
                    })
                } else {
                    Aperture::MultiRect(clipped)
                }
            }
            Aperture::Path {
                bounds,
                shape,
                invert,
            } => Aperture::Path {
                bounds: overlap(bounds, clip),
                shape,
                invert,
            },
        }
    }

    /// Intersect two apertures when an element is bridged by two transitions simultaneously.
    fn intersect(self, other: Aperture) -> Aperture {
        match (self, other) {
            (Aperture::Rect(a), b) => b.intersect_rect(a),
            (a, Aperture::Rect(b)) => a.intersect_rect(b),
            (Aperture::MultiRect(a), Aperture::MultiRect(b)) => {
                let mut clipped = Vec::new();
                for ra in &a {
                    for rb in &b {
                        let r = overlap(*ra, *rb);
                        if r.width > 0 && r.height > 0 {
                            clipped.push(r);
                        }
                    }
                }
                if clipped.is_empty() {
                    Aperture::Rect(Rect {
                        x: 0,
                        y: 0,
                        width: 0,
                        height: 0,
                    })
                } else {
                    Aperture::MultiRect(clipped)
                }
            }
            (a, b) => a.intersect_rect(b.bounds()),
        }
    }
}

/// Kind-specific resolved parameters carried on a [`Running`] transition.
#[derive(Debug, Clone)]
enum Params {
    Crossfade,
    AudioCrossfade,
    Linear {
        direction: Direction,
    },
    WipeAngle {
        angle: f64,
    },
    Iris {
        shape: IrisShape,
        direction: Direction,
        center: [i64; 2],
    },
    RadialWipe {
        direction: Direction,
        start_angle: f64,
        center: [i64; 2],
    },
    Blinds {
        direction: Direction,
        slats: i64,
    },
}

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
    /// The eased fraction of the window.
    eased: f64,
    /// `n`, the whole pixels travelled — `None` on a crossfade or radial wipe.
    pub pixels: Option<i64>,
    /// The sound across the window as the document states it (ADR-0176): `None` is the
    /// absent key, which renders the hard cut.
    pub audio: Option<TransitionAudio>,
    params: Params,
    frame: (i64, i64),
}

/// Why a transition that claims to be running cannot be resolved.
#[derive(Debug, Clone)]
pub(crate) enum Unusable {
    /// No `kind` at all.
    NoKind,
    /// A `kind` the format does not have.
    Kind(String),
    /// A geometric transition with no readable or valid `direction` (or `angle`).
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
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Bridge {
    /// What its `opacity` is multiplied by.
    pub fade: f64,
    /// Whole frame-space pixels it is moved by, outside its own transform.
    pub offset: (i64, i64),
    /// The frame-space aperture it is cut to, which may be empty. `None` for no cut.
    pub cut: Option<Aperture>,
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
                (Some(a), Some(b)) => Some(a.intersect(b)),
                (a, b) => a.or(b),
            },
        }
    }

    /// A frame-space rectangle `clip` moved by this bridge's offset and cut by its cut: the
    /// aperture the element is painted through. `None` where neither the element nor the
    /// transition clips it.
    pub(crate) fn aperture(&self, own: Option<Rect>) -> Option<Aperture> {
        let own = own.map(|clip| Rect {
            x: clip.x + self.offset.0,
            y: clip.y + self.offset.1,
            ..clip
        });
        match (own, self.cut.clone()) {
            (Some(a), Some(b)) => Some(b.intersect_rect(a)),
            (Some(a), None) => Some(Aperture::Rect(a)),
            (None, b) => b,
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

fn read_center(element: &Value, frame: (i64, i64)) -> Option<[i64; 2]> {
    match element.get("center") {
        None => Some([frame.0 / 2, frame.1 / 2]),
        Some(value) => serde_json::from_value::<[i64; 2]>(value.clone()).ok(),
    }
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
    let (direction, params) = match kind {
        TransitionKind::Crossfade => (None, Params::Crossfade),
        TransitionKind::AudioCrossfade => (None, Params::AudioCrossfade),
        TransitionKind::Slide | TransitionKind::Push => {
            let dir = element
                .get("direction")
                .and_then(|value| serde_json::from_value::<Direction>(value.clone()).ok())
                .filter(|d| d.is_cardinal())
                .ok_or(Unusable::Direction(kind.as_str()))?;
            (Some(dir), Params::Linear { direction: dir })
        }
        TransitionKind::Wipe => {
            let has_dir = element.get("direction").is_some();
            let has_angle = element.get("angle").is_some();
            match (has_dir, has_angle) {
                (true, false) => {
                    let dir = element
                        .get("direction")
                        .and_then(|value| serde_json::from_value::<Direction>(value.clone()).ok())
                        .filter(|d| d.is_cardinal())
                        .ok_or(Unusable::Direction(kind.as_str()))?;
                    (Some(dir), Params::Linear { direction: dir })
                }
                (false, true) => {
                    let angle = element
                        .get("angle")
                        .and_then(Value::as_f64)
                        .filter(|a| a.is_finite())
                        .ok_or(Unusable::Direction(kind.as_str()))?;
                    (None, Params::WipeAngle { angle })
                }
                _ => return Err(Unusable::Direction(kind.as_str())),
            }
        }
        TransitionKind::Iris => {
            let dir = match element.get("direction") {
                None => Direction::Out,
                Some(value) => serde_json::from_value::<Direction>(value.clone())
                    .ok()
                    .filter(|d| d.is_iris())
                    .ok_or(Unusable::Direction(kind.as_str()))?,
            };
            let shape = match element.get("shape") {
                None => IrisShape::Circle,
                Some(value) => serde_json::from_value::<IrisShape>(value.clone())
                    .map_err(|_| Unusable::Direction(kind.as_str()))?,
            };
            let center =
                read_center(element, frame).ok_or(Unusable::Direction(kind.as_str()))?;
            (
                Some(dir),
                Params::Iris {
                    shape,
                    direction: dir,
                    center,
                },
            )
        }
        TransitionKind::RadialWipe => {
            let dir = match element.get("direction") {
                None => Direction::Clockwise,
                Some(value) => serde_json::from_value::<Direction>(value.clone())
                    .ok()
                    .filter(|d| d.is_radial())
                    .ok_or(Unusable::Direction(kind.as_str()))?,
            };
            let start_angle = match element.get("start_angle") {
                None => 0.0,
                Some(value) => value
                    .as_f64()
                    .filter(|a| a.is_finite())
                    .ok_or(Unusable::Direction(kind.as_str()))?,
            };
            let center =
                read_center(element, frame).ok_or(Unusable::Direction(kind.as_str()))?;
            (
                Some(dir),
                Params::RadialWipe {
                    direction: dir,
                    start_angle,
                    center,
                },
            )
        }
        TransitionKind::Blinds => {
            let dir = element
                .get("direction")
                .and_then(|value| serde_json::from_value::<Direction>(value.clone()).ok())
                .filter(|d| d.is_cardinal())
                .ok_or(Unusable::Direction(kind.as_str()))?;
            let slats = match element.get("slats") {
                None => 10,
                Some(value) => value
                    .as_i64()
                    .filter(|&s| s >= 1)
                    .ok_or(Unusable::Direction(kind.as_str()))?,
            };
            (
                Some(dir),
                Params::Blinds {
                    direction: dir,
                    slats,
                },
            )
        }
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
    let eased = match &ease {
        Some(ease) => crate::resolve::eased(ease, progress),
        None => progress,
    };
    let pixels = match &params {
        Params::Crossfade | Params::AudioCrossfade | Params::RadialWipe { .. } => None,
        Params::Linear { direction } | Params::Blinds { direction, .. } => {
            Some((eased * travel(*direction, frame) as f64).floor() as i64)
        }
        Params::WipeAngle { angle } => {
            let theta = angle.to_radians();
            let d = frame.0 as f64 * theta.cos().abs() + frame.1 as f64 * theta.sin().abs();
            Some((eased * d).floor() as i64)
        }
        Params::Iris {
            shape,
            direction,
            center,
        } => {
            let active_p = match direction {
                Direction::Out => eased,
                _ => 1.0 - eased,
            };
            let d = match shape {
                IrisShape::Circle => farthest_corner_distance(*center, frame),
                IrisShape::Rect => {
                    let max_w = center[0].max(frame.0 - center[0]);
                    let max_h = center[1].max(frame.1 - center[1]);
                    max_w.max(max_h) as f64
                }
            };
            Some((active_p * d).floor() as i64)
        }
    };
    Ok(Some(Running {
        element: crate::checks::subject_of(element.get("id").and_then(Value::as_str)),
        kind,
        direction,
        from: from.to_string(),
        to: to.to_string(),
        start,
        end,
        progress,
        eased,
        pixels,
        audio: element
            .get("audio")
            .and_then(|audio| serde_json::from_value(audio.clone()).ok()),
        params,
        frame,
    }))
}

/// `D`: the frame's width for a horizontal direction, its height for a vertical one.
fn travel(direction: Direction, (width, height): (i64, i64)) -> i64 {
    match direction {
        Direction::Left | Direction::Right => width,
        Direction::Up | Direction::Down => height,
        _ => 0,
    }
}

/// The unit step of the direction of travel, in frame space (`y` grows downward).
fn unit(direction: Direction) -> (i64, i64) {
    match direction {
        Direction::Left => (-1, 0),
        Direction::Right => (1, 0),
        Direction::Up => (0, -1),
        Direction::Down => (0, 1),
        _ => (0, 0),
    }
}

/// Euclidean distance from `center` to the farthest corner of the `frame` (`R_max` in ADR-0220).
fn farthest_corner_distance([x0, y0]: [i64; 2], (width, height): (i64, i64)) -> f64 {
    [(0, 0), (width, 0), (width, height), (0, height)]
        .into_iter()
        .map(|(cx, cy)| ((cx - x0) as f64).hypot((cy - y0) as f64))
        .fold(0.0_f64, f64::max)
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
        Some(match &self.params {
            Params::AudioCrossfade => return None,
            // A crossfade: the linear opacity ramp, unchanged since ADR-0059.
            Params::Crossfade => Bridge {
                fade: match incoming {
                    true => self.progress,
                    false => 1.0 - self.progress,
                },
                ..Bridge::default()
            },
            Params::Linear { direction } => {
                let direction = *direction;
                let n = self.pixels.unwrap_or(0);
                let d = travel(direction, self.frame);
                let (ux, uy) = unit(direction);
                let along = |distance: i64| (ux * distance, uy * distance);
                match (self.kind, incoming) {
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
                        cut: Some(Aperture::Rect(self.side(direction, n.clamp(0, d), incoming))),
                        ..Bridge::default()
                    },
                    _ => Bridge::default(),
                }
            }
            Params::WipeAngle { angle } => Bridge {
                cut: Some(self.wipe_angle(*angle, incoming)),
                ..Bridge::default()
            },
            Params::Iris {
                shape,
                direction,
                center,
            } => Bridge {
                cut: Some(self.iris(*shape, *direction, *center, incoming)),
                ..Bridge::default()
            },
            Params::RadialWipe {
                direction,
                start_angle,
                center,
            } => Bridge {
                cut: Some(self.radial_wipe(*direction, *start_angle, *center, incoming)),
                ..Bridge::default()
            },
            Params::Blinds { direction, slats } => Bridge {
                cut: Some(self.blinds(*direction, *slats, incoming)),
                ..Bridge::default()
            },
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
            _ => (0, false),
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
            _ => Rect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
        }
    }

    /// `wipe` at an arbitrary `angle` in degrees (ADR-0220 §2).
    fn wipe_angle(&self, angle: f64, passed: bool) -> Aperture {
        let (width, height) = self.frame;
        let rem = angle.rem_euclid(360.0);
        let cardinal = if (rem - 0.0).abs() < 1e-9 || (rem - 360.0).abs() < 1e-9 {
            Some(Direction::Right)
        } else if (rem - 90.0).abs() < 1e-9 {
            Some(Direction::Down)
        } else if (rem - 180.0).abs() < 1e-9 {
            Some(Direction::Left)
        } else if (rem - 270.0).abs() < 1e-9 {
            Some(Direction::Up)
        } else {
            None
        };
        if let Some(dir) = cardinal {
            let d = travel(dir, self.frame);
            let n = (self.eased * d as f64).floor() as i64;
            return Aperture::Rect(self.side(dir, n.clamp(0, d), passed));
        }

        let theta = angle.to_radians();
        let ux = theta.cos();
        let uy = theta.sin();
        let w = width as f64;
        let h = height as f64;
        let d_total = w * ux.abs() + h * uy.abs();
        let n = (self.eased * d_total).floor();

        let full = Rect {
            x: 0,
            y: 0,
            width,
            height,
        };
        let empty = Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };
        if n <= 0.0 {
            return Aperture::Rect(if passed { empty } else { full });
        }
        if n >= d_total {
            return Aperture::Rect(if passed { full } else { empty });
        }

        let d_min = (w * ux).min(0.0) + (h * uy).min(0.0);
        let d_edge = d_min + n;

        let passed_poly = clip_frame_half_plane(w, h, ux, uy, d_edge, true);
        let unpassed_poly = clip_frame_half_plane(w, h, ux, uy, d_edge, false);
        let bounds = polygon_bounds(if passed { &passed_poly } else { &unpassed_poly }, self.frame);
        Aperture::Path {
            bounds,
            shape: AperturePath::Polygon {
                points: passed_poly,
            },
            invert: !passed,
        }
    }

    /// `iris` with `shape` (`circle` or `rect`), `direction` (`out` or `in`), and `center`
    /// (ADR-0220 §2).
    fn iris(
        &self,
        shape: IrisShape,
        direction: Direction,
        center: [i64; 2],
        incoming: bool,
    ) -> Aperture {
        let (width, height) = self.frame;
        let p = self.eased.clamp(0.0, 1.0);
        let (active_p, inside) = match direction {
            Direction::Out => (p, incoming),
            _ => (1.0 - p, !incoming),
        };
        let [x0, y0] = center;
        let full = Rect {
            x: 0,
            y: 0,
            width,
            height,
        };
        let empty = Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };

        match shape {
            IrisShape::Circle => {
                let r_max = farthest_corner_distance(center, self.frame);
                let r = (active_p * r_max).max(0.0);
                if r <= 0.0 {
                    return Aperture::Rect(if inside { empty } else { full });
                }
                if r >= r_max {
                    return Aperture::Rect(if inside { full } else { empty });
                }
                let bounds = if inside {
                    let left = ((x0 as f64 - r).floor() as i64).clamp(0, width);
                    let top = ((y0 as f64 - r).floor() as i64).clamp(0, height);
                    let right = ((x0 as f64 + r).ceil() as i64).clamp(0, width);
                    let bottom = ((y0 as f64 + r).ceil() as i64).clamp(0, height);
                    Rect {
                        x: left,
                        y: top,
                        width: (right - left).max(0),
                        height: (bottom - top).max(0),
                    }
                } else {
                    full
                };
                Aperture::Path {
                    bounds,
                    shape: AperturePath::Circle {
                        center: (x0 as f64, y0 as f64),
                        radius: r,
                    },
                    invert: !inside,
                }
            }
            IrisShape::Rect => {
                let max_w = x0.max(width - x0);
                let max_h = y0.max(height - y0);
                let hw = (active_p * max_w as f64).floor() as i64;
                let hh = (active_p * max_h as f64).floor() as i64;
                let left = (x0 - hw).clamp(0, width);
                let top = (y0 - hh).clamp(0, height);
                let right = (x0 + hw).clamp(0, width);
                let bottom = (y0 + hh).clamp(0, height);
                let inner = Rect {
                    x: left,
                    y: top,
                    width: (right - left).max(0),
                    height: (bottom - top).max(0),
                };
                if inside {
                    Aperture::Rect(inner)
                } else {
                    let bands = [
                        Rect {
                            x: 0,
                            y: 0,
                            width,
                            height: top,
                        },
                        Rect {
                            x: 0,
                            y: bottom,
                            width,
                            height: height - bottom,
                        },
                        Rect {
                            x: 0,
                            y: top,
                            width: left,
                            height: bottom - top,
                        },
                        Rect {
                            x: right,
                            y: top,
                            width: width - right,
                            height: bottom - top,
                        },
                    ];
                    let rects: Vec<Rect> = bands
                        .into_iter()
                        .filter(|r| r.width > 0 && r.height > 0)
                        .collect();
                    if rects.is_empty() {
                        Aperture::Rect(empty)
                    } else if rects.len() == 1 {
                        Aperture::Rect(rects[0])
                    } else {
                        Aperture::MultiRect(rects)
                    }
                }
            }
        }
    }

    /// `radial_wipe` from `start_angle` (degrees from 12 o'clock `-y`, clockwise) around
    /// `center` (ADR-0220 §2).
    fn radial_wipe(
        &self,
        direction: Direction,
        start_angle: f64,
        center: [i64; 2],
        incoming: bool,
    ) -> Aperture {
        let (width, height) = self.frame;
        let full = Rect {
            x: 0,
            y: 0,
            width,
            height,
        };
        let empty = Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };
        let p = self.eased.clamp(0.0, 1.0);
        if p <= 0.0 {
            return Aperture::Rect(if incoming { empty } else { full });
        }
        if p >= 1.0 {
            return Aperture::Rect(if incoming { full } else { empty });
        }

        let skia_start = start_angle - 90.0;
        let sweep = match direction {
            Direction::Clockwise => p * 360.0,
            _ => -(p * 360.0),
        };
        let r_max = farthest_corner_distance(center, self.frame);
        let bounds = if incoming {
            sector_bounds(center, r_max, skia_start, sweep, self.frame)
        } else {
            let comp_start = skia_start + sweep;
            let comp_sweep = if sweep >= 0.0 {
                360.0 - sweep
            } else {
                -360.0 - sweep
            };
            sector_bounds(center, r_max, comp_start, comp_sweep, self.frame)
        };
        Aperture::Path {
            bounds,
            shape: AperturePath::Sector {
                center: (center[0] as f64, center[1] as f64),
                // Pad the wedge radius slightly past `r_max` so the cubic-Bezier arc
                // approximation lies strictly outside the farthest canvas corner.
                radius: r_max + 2.0,
                start_angle_deg: skia_start,
                sweep_deg: sweep,
            },
            invert: !incoming,
        }
    }

    /// `blinds` across `slats` integer bands in `direction` (ADR-0220 §2).
    fn blinds(&self, direction: Direction, slats: i64, passed: bool) -> Aperture {
        let (width, height) = self.frame;
        let s = slats.max(1);
        let d = travel(direction, self.frame);
        let mut rects = Vec::with_capacity(s as usize);

        for k in 0..s {
            let start_k = (k * d) / s;
            let end_k = ((k + 1) * d) / s;
            let t_k = end_k - start_k;
            let n_k = ((self.eased * t_k as f64).floor() as i64).clamp(0, t_k);
            let rect = match (direction, passed) {
                (Direction::Right, true) => Rect {
                    x: start_k,
                    y: 0,
                    width: n_k,
                    height,
                },
                (Direction::Right, false) => Rect {
                    x: start_k + n_k,
                    y: 0,
                    width: t_k - n_k,
                    height,
                },
                (Direction::Left, true) => Rect {
                    x: end_k - n_k,
                    y: 0,
                    width: n_k,
                    height,
                },
                (Direction::Left, false) => Rect {
                    x: start_k,
                    y: 0,
                    width: t_k - n_k,
                    height,
                },
                (Direction::Down, true) => Rect {
                    x: 0,
                    y: start_k,
                    width,
                    height: n_k,
                },
                (Direction::Down, false) => Rect {
                    x: 0,
                    y: start_k + n_k,
                    width,
                    height: t_k - n_k,
                },
                (Direction::Up, true) => Rect {
                    x: 0,
                    y: end_k - n_k,
                    width,
                    height: n_k,
                },
                (Direction::Up, false) => Rect {
                    x: 0,
                    y: start_k,
                    width,
                    height: t_k - n_k,
                },
                _ => Rect {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                },
            };
            if rect.width > 0 && rect.height > 0 {
                rects.push(rect);
            }
        }

        if rects.is_empty() {
            Aperture::Rect(Rect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            })
        } else if rects.len() == 1 {
            Aperture::Rect(rects[0])
        } else {
            Aperture::MultiRect(rects)
        }
    }
}

/// Clip the canvas rectangle `[0, 0, w, h]` against the half-plane `ux * x + uy * y <= d_edge`
/// (when `less_or_equal` is `true`) or `ux * x + uy * y >= d_edge` (when `false`).
fn clip_frame_half_plane(
    w: f64,
    h: f64,
    ux: f64,
    uy: f64,
    d_edge: f64,
    less_or_equal: bool,
) -> Vec<(f64, f64)> {
    let corners = [(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)];
    let signed = |(x, y): (f64, f64)| {
        let dist = ux * x + uy * y - d_edge;
        if less_or_equal { dist } else { -dist }
    };
    let mut out = Vec::with_capacity(6);
    for i in 0..corners.len() {
        let curr = corners[i];
        let next = corners[(i + 1) % corners.len()];
        let d_curr = signed(curr);
        let d_next = signed(next);
        let curr_in = d_curr <= 0.0;
        let next_in = d_next <= 0.0;
        if curr_in {
            out.push(curr);
        }
        if curr_in != next_in {
            let denom = d_curr - d_next;
            if denom.abs() > 1e-12 {
                let t = (d_curr / denom).clamp(0.0, 1.0);
                out.push((
                    curr.0 + t * (next.0 - curr.0),
                    curr.1 + t * (next.1 - curr.1),
                ));
            }
        }
    }
    out
}

/// Conservative integer bounding box of `points` clamped to `[0, 0, width, height]`.
fn polygon_bounds(points: &[(f64, f64)], (width, height): (i64, i64)) -> Rect {
    if points.is_empty() {
        return Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };
    }
    let min_x = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let min_y = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let max_x = points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let max_y = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    let left = (min_x.floor() as i64).clamp(0, width);
    let top = (min_y.floor() as i64).clamp(0, height);
    let right = (max_x.ceil() as i64).clamp(0, width);
    let bottom = (max_y.ceil() as i64).clamp(0, height);
    Rect {
        x: left,
        y: top,
        width: (right - left).max(0),
        height: (bottom - top).max(0),
    }
}

/// Conservative integer bounding box of a circular sector centered at `center` with radius
/// `radius`, sweeping from `start_deg` by `sweep_deg` (in Skia degrees), clamped to the canvas.
fn sector_bounds(
    [x0, y0]: [i64; 2],
    radius: f64,
    start_deg: f64,
    sweep_deg: f64,
    frame: (i64, i64),
) -> Rect {
    let cx = x0 as f64;
    let cy = y0 as f64;
    let (a0, a1) = if sweep_deg >= 0.0 {
        (start_deg, start_deg + sweep_deg)
    } else {
        (start_deg + sweep_deg, start_deg)
    };
    let mut points = vec![
        (cx, cy),
        (
            cx + radius * a0.to_radians().cos(),
            cy + radius * a0.to_radians().sin(),
        ),
        (
            cx + radius * a1.to_radians().cos(),
            cy + radius * a1.to_radians().sin(),
        ),
    ];
    // Include any cardinal angle (multiples of 90°) that falls within `[a0, a1]`.
    let first_k = (a0 / 90.0).ceil() as i64;
    let last_k = (a1 / 90.0).floor() as i64;
    for k in first_k..=last_k {
        let deg = (k * 90) as f64;
        points.push((
            cx + radius * deg.to_radians().cos(),
            cy + radius * deg.to_radians().sin(),
        ));
    }
    polygon_bounds(&points, frame)
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

/// Every audio fade of every transition in the document, by the id of the element it fades,
/// each list in window order. A transition with `audio` absent or `cut` contributes none: the
/// hard cut adds no stage. Built once per render, not once per element.
pub(crate) fn audio_fades(document: &Loose) -> std::collections::HashMap<String, Vec<AudioFade>> {
    let mut fades: std::collections::HashMap<String, Vec<AudioFade>> = Default::default();
    for (_, element) in document.elements_in_tracks() {
        if element.get("type").and_then(Value::as_str) != Some("transition") {
            continue;
        }
        let Some(curve) = element
            .get("audio")
            .and_then(|audio| serde_json::from_value::<TransitionAudio>(audio.clone()).ok())
            .filter(|curve| *curve != TransitionAudio::Cut)
        else {
            continue;
        };
        let (Some(from), Some(to), Some(start), Some(end)) = (
            element.get("from").and_then(Value::as_str),
            element.get("to").and_then(Value::as_str),
            element.get("start").and_then(Value::as_i64),
            element.get("end").and_then(Value::as_i64),
        ) else {
            continue;
        };
        if from == to || end <= start {
            continue;
        }
        for (id, incoming) in [(from, false), (to, true)] {
            fades.entry(id.to_string()).or_default().push(AudioFade {
                transition: crate::checks::subject_of(element.get("id").and_then(Value::as_str)),
                incoming,
                start,
                end,
                curve,
            });
        }
    }
    for list in fades.values_mut() {
        list.sort_by_key(|fade| fade.start);
    }
    fades
}

/// What a running transition does to the sound of the element `id` at `instant`, as the
/// `volume 1.0 × transition t1 constant_power 0.63 → 0.63` line `query --at` prints
/// (ADR-0176 §6), and its parts. `None` where no running transition names the element.
///
/// The gain is [`AudioFade::gain_at`], the function the render's `afade` stage is tested
/// against, so the number printed is the number applied. A gain is never printed without the
/// id of the transition the agent edits.
pub(crate) fn sound_at(running: &[Running], id: &str, volume: f64, instant: i64) -> Option<Value> {
    let mut parts = Vec::new();
    let mut gains = Vec::new();
    let mut factors = Vec::new();
    for transition in running {
        let incoming = if transition.to == id {
            true
        } else if transition.from == id {
            false
        } else {
            continue;
        };
        match transition.audio {
            None | Some(TransitionAudio::Cut) => {
                let absent = transition.audio.is_none();
                parts.push(format!(
                    "transition {} audio: cut{}",
                    transition.element,
                    if absent { " (absent)" } else { "" }
                ));
                factors.push(serde_json::json!({
                    "transition": transition.element,
                    "audio": "cut",
                    "absent": absent,
                }));
            }
            Some(curve) => {
                let gain = AudioFade {
                    transition: transition.element.clone(),
                    incoming,
                    start: transition.start,
                    end: transition.end,
                    curve,
                }
                .gain_at(instant);
                parts.push(format!(
                    "transition {} {} {gain:.2}",
                    transition.element,
                    curve.as_str()
                ));
                gains.push(gain);
                factors.push(serde_json::json!({
                    "transition": transition.element,
                    "audio": curve.as_str(),
                    "gain": gain,
                }));
            }
        }
    }
    if factors.is_empty() {
        return None;
    }
    let total = gains.iter().fold(volume, |product, gain| product * gain);
    let text = match gains.is_empty() {
        true => parts.join(", "),
        false => format!("volume {volume} × {} → {total:.2}", parts.join(" × ")),
    };
    Some(serde_json::json!({
        "volume": volume,
        "transitions": factors,
        "gain": total,
        "text": text,
    }))
}
