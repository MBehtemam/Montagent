//! A path's stroke shape (ADR-0158): its join, miter limit and cap, read off the file, and
//! the **reach** that widens ADR-0154's inset to keep the stroke inside the declared box.
//!
//! One reading for every consumer: `validate`'s `E-PATH-OUTSIDE-BOX`, the resolver's
//! overshoot clamp ([`crate::animatable::inset_box`]), `query --at`, and the painter. Each
//! field is read from the element as written, so a malformed value — the schema check's to
//! report — reads as absent here, never as a guess.
//!
//! **Dashes** (ADR-0158 §5): a dash makes a cap draw on a closed path too, which is
//! [`caps_draw`]'s one other case; the reach is otherwise unchanged by dashes, because every
//! dash end and every join inside a dash lies on the curve. [`dash`] is the pattern's one
//! reading, for `validate`, `query --at` and the painter alike.
//!
//! **Trim** (ADR-0160): a trim makes a cap draw on a closed path as a dash does, and is
//! [`caps_draw`]'s third case; every trim end lies on the curve too. [`window`] is the one
//! arithmetic of the window drawn, and [`window_at`] its one reading off an element, for
//! `query --at` and the painter alike.

use serde_json::Value;

/// `stroke_join`: how two segments meet (ADR-0158 §2). `"round"` when absent, ADR-0154's pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Join {
    Round,
    Bevel,
    /// With its `stroke_miter_limit`, `None` where the file states none — which `validate`
    /// refuses as `E-STROKE-MITER-LIMIT`. Read as limit 1 (always beveled) by the reach and
    /// the painter alike, so the two never disagree about a file `render` will not take.
    Miter(Option<i64>),
}

/// `stroke_cap`: how an open path's two ends, and each dash's, are drawn (ADR-0158 §3).
/// `"butt"` when absent, ADR-0154's pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cap {
    Butt,
    Round,
    Square,
}

/// Where the reach factor `k` came from (ADR-0158 §4 and §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// Neither a miter join nor a square cap that draws: `k` is 1.
    Neither,
    /// A `"miter"` join: `k` is its `stroke_miter_limit`.
    MiterLimit(i64),
    /// A `"square"` cap where a cap draws: `k` is √2.
    SquareCap,
}

impl Source {
    /// `k` as `query` and `validate` print it: `1`, the limit, or `√2`.
    pub fn factor(self) -> String {
        match self {
            Source::Neither => "1".into(),
            Source::MiterLimit(limit) => limit.to_string(),
            Source::SquareCap => "√2".into(),
        }
    }

    /// Where `k` came from, in the words `validate` and `query` print.
    pub fn describe(self) -> String {
        match self {
            Source::Neither => "neither a miter join nor a square cap that draws".into(),
            Source::MiterLimit(limit) => format!("`stroke_miter_limit` {limit}"),
            Source::SquareCap => "`stroke_cap` \"square\"".into(),
        }
    }
}

/// A path's stroke reach: the inset `m = ceil(k × w / 2)` and where `k` came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reach {
    /// `m`, in box pixels.
    pub inset: i64,
    /// `w`: `stroke_width`, its largest key where keyed, `0` with no `stroke`.
    pub width: i64,
    pub source: Source,
}

/// The element's join, from `stroke_join` and `stroke_miter_limit`.
pub fn join(element: &Value) -> Join {
    match element.get("stroke_join").and_then(Value::as_str) {
        Some("bevel") => Join::Bevel,
        Some("miter") => Join::Miter(element.get("stroke_miter_limit").and_then(Value::as_i64)),
        _ => Join::Round,
    }
}

/// The element's cap, from `stroke_cap`.
pub fn cap(element: &Value) -> Cap {
    match element.get("stroke_cap").and_then(Value::as_str) {
        Some("round") => Cap::Round,
        Some("square") => Cap::Square,
        _ => Cap::Butt,
    }
}

/// Whether a cap draws anywhere on this path: at an open path's two ends, at both ends of
/// every dash, and at a trim's two ends. A closed path with no `stroke_dash`, `trim_start`
/// or `trim_end` has none (ADR-0158 §3, ADR-0160 §6). Presence decides the trim, not value:
/// a closed path with an explicit full window counts, and the containment that buys is
/// conservative, never short.
pub fn caps_draw(element: &Value) -> bool {
    element.get("closed").and_then(Value::as_bool) == Some(false)
        || element.get("stroke_dash").is_some()
        || trimmed(element)
}

/// The element's `stroke_dash` as written (ADR-0158 §5): its entries, where it is a list of
/// 2 to 16 integers each at least 0. Anything else is the schema's to report, and reads as
/// absent.
pub fn dash(element: &Value) -> Option<Vec<i64>> {
    let entries = element.get("stroke_dash")?.as_array()?;
    if !(2..=16).contains(&entries.len()) {
        return None;
    }
    entries
        .iter()
        .map(|entry| entry.as_i64().filter(|length| *length >= 0))
        .collect()
}

/// A dash pattern the painter can draw: an even number of entries with a total above 0.
/// Anything else is `validate`'s `E-DASH-SHAPE`, which `render` refuses.
pub fn drawable_dash(element: &Value) -> Option<Vec<i64>> {
    dash(element).filter(|pattern| {
        pattern.len() % 2 == 0 && !pattern.is_empty() && pattern.iter().sum::<i64>() > 0
    })
}

/// The part of an outline a trimmed stroke draws (ADR-0160 §4, §5), as fractions of the
/// outline's length from its start point, in its direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Window {
    /// Start at or past the end: no stroke at all, under any cap.
    Empty,
    /// `0` to `1` after the clamp, under any offset: the outline drawn whole, exactly as
    /// with no trim.
    Full,
    /// From `from` to `to`, each in `[0, 1]`. `from > to` means the window crosses the start
    /// point, and is drawn as one stroke through it.
    Part { from: f64, to: f64 },
}

/// How close to the start point a rotated window end snaps onto it, so that `f64`'s
/// rounding in `0.1 + (2.9 mod 1)` never leaves a window crossing into a piece of length
/// `1e-16`.
const SNAP: f64 = 1e-12;

/// **The window a trim draws** (ADR-0160 §4, §5): `trim_start` and `trim_end` clamped to
/// `[0, 1]`, empty where start ≥ end, full where they are `0` and `1`, and otherwise both
/// rotated forward by `trim_offset` wrapped into `[0, 1)`. The offset never makes a window
/// empty or full.
pub fn window(start: f64, end: f64, offset: f64) -> Window {
    let (start, end) = (start.clamp(0.0, 1.0), end.clamp(0.0, 1.0));
    if start >= end {
        return Window::Empty;
    }
    if start <= 0.0 && end >= 1.0 {
        return Window::Full;
    }
    let turn = offset.rem_euclid(1.0);
    let (mut from, mut to) = (start + turn, end + turn);
    if from >= 1.0 - SNAP {
        from = (from - 1.0).max(0.0);
        to -= 1.0;
    }
    if from < SNAP {
        from = 0.0;
    }
    if to > 1.0 + SNAP {
        to -= 1.0;
    } else {
        to = to.min(1.0);
    }
    Window::Part { from, to }
}

/// The trim fields (ADR-0160 §2), in the canonical key order.
pub const TRIM: [&str; 3] = ["trim_start", "trim_end", "trim_offset"];

/// Whether the element carries a trim window: `trim_start` or `trim_end`, whatever their
/// values. Presence decides it, so `validate` reads it from the file alone (ADR-0160 §6).
pub fn trimmed(element: &Value) -> bool {
    element.get("trim_start").is_some() || element.get("trim_end").is_some()
}

/// The element's window at `numerator / denominator` ms, through the one resolving function
/// (ADR-0146), which clamps `trim_start` and `trim_end`. `None` where the element carries no
/// trim field. On an open path the offset is not read: it is `validate`'s `E-TRIM-OFFSET`
/// there, which `render` refuses.
pub fn window_at(element: &Value, numerator: i128, denominator: i128) -> Option<Window> {
    if !TRIM.iter().any(|field| element.get(*field).is_some()) {
        return None;
    }
    let read = |field, default| {
        crate::animatable::number_read(element, field, numerator, denominator, default)
    };
    let open = element.get("closed").and_then(Value::as_bool) == Some(false);
    let offset = if open { 0.0 } else { read("trim_offset", 0.0) };
    Some(window(
        read("trim_start", 0.0),
        read("trim_end", 1.0),
        offset,
    ))
}

/// The path's reach (ADR-0158 §4): `k` is the larger of the join factor (the miter limit,
/// else 1) and the cap factor (√2 for a square cap that draws, else 1), and `w` the largest
/// `stroke_width` the file states. With no `stroke`, the inset is 0.
///
/// The limit is an integer and √2 lies strictly between 1 and 2, so the two factors never
/// tie: a limit of 2 or more is the larger, and a limit of 1 is not.
pub fn reach(element: &Value) -> Reach {
    let width = if element.get("stroke").is_some() {
        crate::animatable::greatest_length(element, "stroke_width")
            .unwrap_or(0)
            .max(0)
    } else {
        0
    };
    let limit = match join(element) {
        Join::Miter(Some(limit)) if limit > 1 => Some(limit),
        _ => None,
    };
    let square = cap(element) == Cap::Square && caps_draw(element);
    let (inset, source) = match (limit, square) {
        (Some(limit), _) => (ceil_half(limit * width), Source::MiterLimit(limit)),
        (None, true) => (least_root_half(width), Source::SquareCap),
        (None, false) => (ceil_half(width), Source::Neither),
    };
    Reach {
        inset,
        width,
        source,
    }
}

/// `ceil(n / 2)`, for `n ≥ 0`.
fn ceil_half(n: i64) -> i64 {
    (n + 1) / 2
}

/// `ceil(√2 × w / 2)`, exactly: the least `m ≥ 0` with `2m² ≥ w²`.
fn least_root_half(width: i64) -> i64 {
    let mut m = (width as f64 / std::f64::consts::SQRT_2).ceil() as i64;
    while 2 * m * m < width * width {
        m += 1;
    }
    while m > 0 && 2 * (m - 1) * (m - 1) >= width * width {
        m -= 1;
    }
    m
}
