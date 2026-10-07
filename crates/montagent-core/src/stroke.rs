//! A path's stroke shape (ADR-0158): its join, miter limit and cap, read off the file, and
//! the **reach** that widens ADR-0154's inset to keep the stroke inside the declared box.
//!
//! One reading for every consumer: `validate`'s `E-PATH-OUTSIDE-BOX`, the resolver's
//! overshoot clamp ([`crate::animatable::inset_box`]), `query --at`, and the painter. Each
//! field is read from the element as written, so a malformed value — the schema check's to
//! report — reads as absent here, never as a guess.
//!
//! **Where dashes go** (ADR-0158 §5, the next slice): a dash makes a cap draw on a closed
//! path too, which is [`caps_draw`]'s one other case; the reach itself is unchanged by
//! dashes, because every dash end and every join inside a dash lies on the curve.

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

/// Whether a cap draws anywhere on this path: at an open path's two ends. A closed path has
/// none until it is dashed (ADR-0158 §3; dashes are the next slice).
pub fn caps_draw(element: &Value) -> bool {
    element.get("closed").and_then(Value::as_bool) == Some(false)
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
