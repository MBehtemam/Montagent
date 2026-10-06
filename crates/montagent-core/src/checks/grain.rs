//! **`R-GRAIN-SEED-SHARED`** (`review`, ADR-0156 §5): two `grain` members with the same
//! `seed`, `size` and `mono`, visible at the same instant, whose elements' starts fall on the
//! same frame. Their draws are keyed on the same local frame ([`crate::grain`]), so they draw
//! the same pattern on every frame they share, which shows as one locked texture. The repair
//! is to change one seed.
//!
//! Two such members in one element's list count too: they draw the same pattern over itself.
//! `amount` is not compared, because it scales the draw rather than choosing it. Decided from
//! the file without painting a frame; a project with no `fps` has no frame grid to ask on.

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::model::Effect;
use crate::permissive::Loose;
use crate::report::Report;

/// One `grain` member, and when its element is present.
struct Member {
    subject: String,
    track: Option<String>,
    index: usize,
    seed: u32,
    size: u8,
    mono: bool,
    start: i64,
    end: i64,
    first: i64,
}

pub fn check(document: &Loose, report: &mut Report) {
    let Some(fps) = document
        .value()
        .get("fps")
        .and_then(Value::as_i64)
        .filter(|fps| *fps > 0)
    else {
        return;
    };
    let mut members = Vec::new();
    for (track, element) in document.elements_in_tracks() {
        let (Some(start), Some(end)) = (
            element.get("start").and_then(Value::as_i64),
            element.get("end").and_then(Value::as_i64),
        ) else {
            continue;
        };
        let effects = element.get("effects").and_then(Value::as_array);
        for (index, effect) in effects.into_iter().flatten().enumerate() {
            let Ok(Effect::Grain {
                seed, size, mono, ..
            }) = serde_json::from_value::<Effect>(effect.clone())
            else {
                continue;
            };
            members.push(Member {
                subject: crate::checks::subject_of(element.get("id").and_then(Value::as_str)),
                track: track.map(str::to_string),
                index,
                seed: seed.0,
                size: size.0,
                mono,
                start,
                end,
                first: crate::grain::first_frame(start, fps),
            });
        }
    }
    for (i, a) in members.iter().enumerate() {
        for b in &members[i + 1..] {
            let alike = (a.seed, a.size, a.mono, a.first) == (b.seed, b.size, b.mono, b.first);
            if !alike || !together(a, b, fps) {
                continue;
            }
            let finding = Finding::new("R-GRAIN-SEED-SHARED")
                .field("element", json!(a.subject))
                .field("index", json!(a.index))
                .field("other", json!(b.subject))
                .field("other_index", json!(b.index))
                .field("seed", json!(a.seed))
                .field("size", json!(a.size))
                .field("mono", json!(a.mono))
                .field("frame", json!(a.first))
                .at_file(document.path())
                .at_element(&a.subject);
            report.push(match &a.track {
                Some(track) => finding.at_track(track),
                None => finding,
            });
        }
    }
}

/// Whether some frame `render` paints holds both elements.
fn together(a: &Member, b: &Member, fps: i64) -> bool {
    let (from, until) = (a.start.max(b.start), a.end.min(b.end));
    crate::exact::instant_of(crate::grain::first_frame(from, fps), fps) < until
}
