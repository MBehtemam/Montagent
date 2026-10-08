//! What `grain`'s draw is keyed on: the element's **local frame** (ADR-0156 §3).
//!
//! ADR-0156 keys the draw on the element's local instant, measured from its `start`, and
//! says two grains draw alike when their starts "coincide on the frame grid". The local
//! instant is therefore counted in whole output frames: the frame being painted, less the
//! first frame the element's `start` lets it paint. Both are integers, so the hash takes no
//! float, and:
//!
//! - an element moved by N whole frames paints the same pixels N frames later, whatever
//!   millisecond its `start` rounds to (a `start` in whole milliseconds cannot sit exactly on
//!   a 30 fps frame, so a hash of `t − start` in milliseconds would change under most shifts);
//! - two elements whose starts fall on the same frame draw the same pattern, which is the
//!   condition `R-GRAIN-SEED-SHARED` reports;
//! - the re-roll rate follows the output `fps`.
//!
//! Under `motion_blur` the samples of one frame share the frame's draw: the grain re-rolls
//! per output frame, while `amount` and the transform follow each sample (ADR-0155 §3).

use serde_json::Value;

/// The first frame an element starting at `start` ms is painted on: the first frame whose
/// instant is at or after `start`.
pub(crate) fn first_frame(start: i64, fps: i64) -> i64 {
    crate::exact::frame_at_or_after(start, fps.max(1)).map_or(0, |frame| frame.frame)
}

/// The local frame `element`'s grain draws from when the frame holding `instant` ms is
/// painted at `fps`: that frame less the element's first. `0` on the first frame it paints.
pub fn local_frame(element: &Value, instant: i64, fps: i64) -> i64 {
    let start = element.get("start").and_then(Value::as_i64).unwrap_or(0);
    crate::motion_blur::frame_containing(instant, fps) - first_frame(start, fps)
}
