//! **Time remap**: which moment of its file a `video` carrying `source_time` shows (ADR-0157).
//!
//! `source_time` is an animatable property of integer source milliseconds. A literal is a
//! freeze frame; a keyframe list is a curve whose slope is the rate, so a flat stretch
//! freezes and a falling one plays in reverse. The curve is the only author of the source.
//!
//! [`source_ms`] is **the one resolution function**. The painter (through
//! `query::at::source_offset`, which every painted frame's caption is built by), `validate`'s
//! remap arm of `E-SOURCE-OVERRUN`, `query --at`, `frame`, the contact sheet and `measure`'s
//! keyed-alpha series all call it, so no two of them can resolve one instant to two
//! milliseconds. At a painted frame instant (ADR-0077):
//!
//! 1. `source_time` is read through [`crate::animatable`], the one number interpolation every
//!    animatable property uses (ADR-0146), under each key's `ease`, holding the nearest key's
//!    value before the first key and after the last;
//! 2. the value is rounded half-up to an integer millisecond;
//! 3. the frame shown is the last source frame starting at or before it (ADR-0096), which is
//!    `decode::frame_at`'s rule and every supplier's.
//!
//! There is no exact-rational path for a linear segment (ADR-0157 §3): a second path could
//! disagree with this one by a millisecond at a rounding boundary.

use serde_json::Value;

use crate::animatable::{self, Resolved, Unreadable};
use crate::exact;

/// The field, as the document spells it.
pub const FIELD: &str = "source_time";

/// The fields the curve makes redundant on the same element (ADR-0157 §2), in the order
/// `E-REMAP-FIELD` reports them.
pub const REFUSED: [&str; 4] = ["source_start", "source_end", "speed", "overrun"];

/// Whether `element` is a `video` that names its source by `source_time`.
pub fn is_remapped(element: &Value) -> bool {
    element.get("type").and_then(Value::as_str) == Some("video") && element.get(FIELD).is_some()
}

/// **The one resolution function**: the source millisecond `element` shows at `instant`,
/// steps 1 and 2 of ADR-0157 §3. `Err` names why the curve gives no value there.
///
/// A value below `0` is returned as it resolves, never clamped: an overshooting ease that
/// dips below the file's first moment is `validate`'s `E-SOURCE-OVERRUN`, found here.
pub fn source_ms(element: &Value, instant: i64) -> Result<i64, String> {
    let resolved = animatable::at(element, FIELD, instant)
        .ok_or_else(|| "the element carries no `source_time`".to_string())?;
    let value = match resolved {
        Ok(Resolved::Number(value)) => value,
        Ok(_) => return Err("`source_time` is not a number of milliseconds".to_string()),
        Err(Unreadable::Schema(reason)) => {
            return Err(format!("`source_time` does not read: {reason}"));
        }
        Err(Unreadable::Unresolvable(_)) => {
            return Err("`source_time`'s keyframe list determines no value".to_string());
        }
    };
    round_half_up(value).ok_or_else(|| format!("`source_time` resolves to {value}"))
}

/// Round half-up to an integer: `1.5` to `2`, `-1.5` to `-1`.
fn round_half_up(value: f64) -> Option<i64> {
    let rounded = (value + 0.5).floor();
    (rounded.is_finite() && rounded.abs() < i64::MAX as f64).then_some(rounded as i64)
}

/// Every painted frame instant inside `[start, end)` at `fps`: `⌊n × 1000 / fps⌋` for each
/// frame `n` whose instant falls in the range (ADR-0077), in order.
pub fn painted_instants(start: i64, end: i64, fps: i64) -> impl Iterator<Item = i64> {
    let first = exact::frame_at_or_after(start, fps).map(|sampled| sampled.frame);
    first
        .into_iter()
        .flat_map(move |first| first..)
        .map(move |n| exact::instant_of(n, fps))
        .take_while(move |instant| *instant < end)
}

/// The painted frame instant after the frame `instant` lies in.
pub fn next_instant(instant: i64, fps: i64) -> Option<i64> {
    let frame = exact::frame_at_or_before(instant, fps)?.frame;
    Some(exact::instant_of(frame + 1, fps))
}

/// The **rate** at `instant` (ADR-0157 §5): the change in resolved source time from this
/// instant to the next painted frame instant, over the time between them, signed, to three
/// decimals with `×`: `-0.500×`, `0.000×`. What the viewer sees, so it has one value at a key
/// where the curve's slope jumps.
pub fn rate(element: &Value, instant: i64, fps: i64) -> Result<String, String> {
    let next = next_instant(instant, fps)
        .filter(|next| *next > instant)
        .ok_or_else(|| "the project states no `fps` to find the next frame by".to_string())?;
    let here = source_ms(element, instant)?;
    let there = source_ms(element, next)?;
    let rate = (there - here) as f64 / (next - instant) as f64;
    // `+ 0.0` turns a negative zero into a positive one, so a freeze never reads `-0.000×`.
    Ok(format!("{:.3}×", rate + 0.0))
}
