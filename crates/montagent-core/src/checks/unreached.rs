//! `R-KEYFRAME-UNREACHED` — ADR-0035: *"for every animated property, whether the
//! property's stated final (or first) keyframe value is ever actually produced by a
//! sampled frame within the element's own `[start, end)` range."*
//!
//! The measured defect: fading `opacity` 1 → 0 over `[end-300, end]` *"leaves 0.010–0.171
//! residual opacity on the last sampled frame instead of reaching exactly 0 — the element
//! visibly pops off rather than fading"*, and two agents independently wrote that literal,
//! naive spelling and caught it only in self-audit. The renderer is asked to fix nothing:
//! ADR-0035 is unanimous that there is **no keyframe rounding rule**, because snapping a
//! declared `t` to the grid *"would mean a clip's opacity and a clip's visibility could be
//! computed on two different effective clocks"*. So the arithmetic is named instead — here
//! as a finding, and in `measure` as the nearest-sampled-instant output an author retargets
//! onto.
//!
//! ## What is asked, and of which keyframe
//!
//! ADR-0012 clamps at both ends, so each endpoint record declares a **plateau**: the last
//! record's value is what the element resolves to from its `t` onwards, and the first
//! record's is what it resolves to up to its `t`. Intersected with the element's own
//! half-open range, that plateau is the whole of the window in which the declared value
//! can appear on screen — and the question is whether the grid puts a frame in it.
//!
//! - **Last record**, where `t <= end`: the plateau is `[t, end)`. At `t == end` it is
//!   empty *by construction* — ADR-0005 makes `end` an instant the element never renders
//!   at — which is exactly ADR-0035's fade, and why this fires whether or not `end` is on
//!   the grid.
//! - **First record**, where `t > start`: the plateau is `[start, t]`, closed at `t`.
//!
//! ## The two cases that are deliberately *not* asked about
//!
//! **A keyframe outside the element's own range is a trimmed move**, not an unreached
//! target. `CONTEXT.md` calls it ordinary authoring and ADR-0012's clamping is the whole of
//! how it is spelled — *"and then it holds"* costs no syntax — and the committed fixture
//! writes seven of them, one sitting 13.8 s past the end of the project. The element makes
//! no claim to reach that value inside its own life, so there is no claim to test.
//!
//! **A first record at exactly the element's own `start`** is passed over. Its plateau
//! would be the single instant `start`, which holds a sampled frame precisely when `start`
//! is on the grid — so firing there would be the literal off-grid-boundary check ADR-0006
//! considered and refused: *"109 of its 120 time values … are off the 40 ms grid … a check
//! with a 98% hit rate on a correct, published project is not a check; it is the alarm
//! fatigue this ADR already identified as a safety problem."* The last record's `t == end`
//! case is **not** the same shape, and the asymmetry is ADR-0005's rather than this
//! module's: `start` is an instant inside the range that the grid may or may not sample,
//! and `end` is an instant outside it that the grid can never sample.
//!
//! Neither exclusion is stated by ADR-0035, which names the trigger and not its edges;
//! both are this module's reading, argued from ADR-0006's noise budget. The first is the
//! load-bearing one — without it the check fires on 7 of 7 of the committed fixture's
//! `photo` elements, so **the narrowing rather than the ADR is what decides this check's
//! reach**, which is a thing an ADR should settle and a module doc should not. Raised as
//! [#285](https://github.com/MBehtemam/Montagent/issues/285).
//!
//! ## Exact arithmetic
//!
//! The grid step is `1000/fps` ms and *"not necessarily integral"*: at 30 fps it is `100/3`
//! and only multiples of 100 ms are frame-exact. Frames are carried as **indices**
//! throughout ([`crate::exact`]), the sampled instant as the exact ratio
//! `n × 1000 / fps`, and the resolved value comes from
//! [`crate::resolve::at_instant`] at that ratio — never from an instant rounded to a whole
//! millisecond first, which would be the rounding rule ADR-0035 says there is not.
//!
//! **Document-only**: *"computable from the grid (`fps`), the element's `start`/`end`, and
//! the keyframe list, with no I/O"*.

use serde_json::{Value, json};

use crate::exact;
use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;
use crate::stack::TimelineRange;

/// Which end of the list a target came from.
#[derive(Clone, Copy)]
enum End {
    First,
    Last,
}

/// `R-KEYFRAME-UNREACHED`, over every animated property in the document.
pub fn check(document: &Loose, report: &mut Report) {
    // No grid, so no question. `fps` is required, and the check that says so owns the
    // schema.
    let Some(fps) = document.value().get("fps").and_then(Value::as_i64) else {
        return;
    };
    if fps <= 0 {
        return;
    }

    for (track, element) in document.elements_in_tracks() {
        let (Some(start), Some(end)) = (
            element.get("start").and_then(Value::as_i64),
            element.get("end").and_then(Value::as_i64),
        ) else {
            continue;
        };
        let range = TimelineRange { start, end };
        if end <= start {
            continue;
        }
        // The frames the element's own half-open range holds. An element holding none
        // *"rounds out of existence"* and is `N-QUANTIZATION`'s finding (ADR-0006); every
        // one of its targets is unreached, and saying so once per property would report
        // one fact many times.
        let (Some(first_frame), Some(last_frame)) = (
            exact::frame_at_or_after(start, fps),
            exact::frame_before(end, fps),
        ) else {
            continue;
        };
        if first_frame.frame > last_frame.frame {
            continue;
        }

        let subject = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
        for property in crate::animatable::names() {
            let Some(records) = crate::checks::keyframe_records(element, property) else {
                continue;
            };
            let Some((first_t, last_t)) = endpoints(records) else {
                continue;
            };

            for (which, declared_t, nearest) in [
                // The last record's plateau is `[t, end)`, and the frame nearest its
                // declared instant inside the range is the range's own last.
                (End::Last, last_t, last_frame),
                (End::First, first_t, first_frame),
            ] {
                if !misses(which, declared_t, range, first_frame, fps) {
                    continue;
                }
                let Some(finding) =
                    unreached(element, &subject, property, declared_t, nearest, range)
                else {
                    continue;
                };
                let finding = finding.at_file(document.path()).at_element(&subject);
                report.push(match track {
                    Some(track) => finding.at_track(track),
                    None => finding,
                });
            }
        }
    }
}

/// The `t` of the earliest and latest record of a keyframe list.
///
/// **Clock order, not array order**: a keyframe's `t` is the whole of what places it on the
/// clock (ADR-0012), which is the reading [`crate::resolve`] takes and the one a question
/// about *what the render shows* has to take. A single-record list is excluded — it is a
/// constant, so its value is produced at every instant the element is on screen, and
/// whether the element is on screen at all is `N-QUANTIZATION`'s question.
///
/// `None` too where no record states a readable `t`, which is the schema check's fact.
fn endpoints(records: &[Value]) -> Option<(i64, i64)> {
    if records.len() < 2 {
        return None;
    }
    let times: Vec<i64> = records
        .iter()
        .filter_map(|record| record.get("t")?.as_i64())
        .collect();
    Some((*times.iter().min()?, *times.iter().max()?))
}

/// Does the plateau this endpoint declares inside the element's range hold no sampled
/// frame?
///
/// See the module doc for both exclusions: a `t` outside the range declares no plateau
/// inside it, and a first record at exactly `start` declares only the element's own entry
/// instant, which is the off-grid-boundary question ADR-0006 refused.
fn misses(
    which: End,
    declared_t: i64,
    range: TimelineRange,
    first_frame: exact::Sampled,
    fps: i64,
) -> bool {
    match which {
        End::Last => {
            if declared_t > range.end {
                return false;
            }
            // `[max(start, t), end)`: a record before the element's own start still holds
            // its value across the whole range, and the range is where the question is
            // asked.
            exact::holds_a_sampled_frame(declared_t.max(range.start), range.end, fps) != Some(true)
        }
        End::First => {
            if declared_t <= range.start || declared_t > range.end {
                return false;
            }
            // `[start, t]`, closed at `t`: the plateau ends *at* the record, and a frame
            // landing exactly on it shows the declared value.
            !first_frame.is_at_or_before(declared_t)
        }
    }
}

/// The finding, or `None` where the nearest frame turns out to resolve to the target after
/// all — or where the property cannot be read as the format's types.
///
/// ADR-0035: it *"fires only when a sampled value provably diverges from a stated
/// target"*. The comparison is made rather than assumed, so a `step` segment — whose value
/// holds at the previous record until this one's own instant, and which can therefore
/// reach a target the interpolation arithmetic would not — is answered by the same
/// resolver the renderer uses rather than by a special case here.
fn unreached(
    element: &Value,
    subject: &str,
    property: &str,
    declared_t: i64,
    nearest: exact::Sampled,
    range: TimelineRange,
) -> Option<Finding> {
    let declared = resolved(element, property, i128::from(declared_t), 1)?;
    let (numerator, denominator) = nearest.ratio();
    let sampled = resolved(element, property, numerator, denominator)?;
    if declared == sampled {
        return None;
    }

    Some(
        Finding::new("R-KEYFRAME-UNREACHED")
            .field("element", json!(subject))
            .field("property", json!(property))
            .field("declared_t", json!(declared_t))
            .field("target", declared)
            .field("frame", json!(nearest.frame))
            // The instant is a ratio, and `Sampled::ms` is the one place in the crate it
            // is divided — for prose, after every decision has been made on integers.
            // `frame` above is the exact form.
            .field("sampled_t", json!(nearest.ms()))
            .field("sampled", sampled)
            .field("start", json!(range.start))
            .field("end", json!(range.end)),
    )
}

/// One property, resolved at `numerator / denominator` ms and rendered as JSON.
///
/// Through [`crate::animatable::read`], the one resolving function, so that what this check
/// calls the value at an instant is what `query --at` and the rasterizer call it — a colour
/// included, compared as the bytes it paints. `None` where the property does not fit the
/// format's types — a schema fact, and the schema check's to report.
fn resolved(element: &Value, property: &str, numerator: i128, denominator: i128) -> Option<Value> {
    let resolved = crate::animatable::read(element, property, numerator, denominator)?.ok()?;
    serde_json::to_value(resolved).ok()
}
