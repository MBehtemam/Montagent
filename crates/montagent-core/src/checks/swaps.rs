//! PROTOTYPE (#614): the document checks on an image's `swaps`, from the resolution of
//! #595.
//!
//! - **`E-SWAP-ORDER`** — the list is not sorted by `start`. There is no "later wins" and no
//!   silent sort: an implicit `end` is the *next* swap's `start`, so an unsorted list
//!   resolves to windows nobody wrote.
//! - **`E-SWAP-OVERLAP`** — two swaps share an instant. Only an explicit `end` can do it,
//!   since an implicit one stops at the next `start`. Half-open, so `end == next.start` is
//!   not an overlap and is not flagged: the scoring ticket's edit test between the two
//!   variants measures that hazard directly.
//! - **`E-SWAP-RANGE`** — a swap that is empty, or that falls outside the element's own
//!   `[start, end)`.
//!
//! All three refuse, on `E-HIGHLIGHT-OVERLAP`'s reasoning ([`crate::checks::highlight`]):
//! the document does not say which of the two numbers is the wrong one.
//!
//! Document-only: no I/O, so this needs no session. The disk half of a swap — the
//! missing-file check and `E-FIT-DEVIATION` on every swap's `source` — is the existing
//! checks reading [`crate::swaps::sources`].

use serde_json::{Value, json};

use crate::checks::subject_of;
use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;
use crate::swaps::{self, Window};

pub fn check(document: &Loose, report: &mut Report) {
    let file = document.path();
    for (track, element) in document.elements_in_tracks() {
        if element.get("swaps").is_none() {
            continue;
        }
        let windows = swaps::windows(element);
        let subject = subject_of(element.get("id").and_then(Value::as_str));
        let at = |finding: Finding| {
            let finding = finding
                .at_file(file)
                .at_element(subject.clone())
                .field("element", json!(subject));
            match track {
                Some(track) => finding.at_track(track),
                None => finding,
            }
        };

        let mut sorted = true;
        for pair in windows.windows(2) {
            let [earlier, later] = pair else { continue };
            if later.start < earlier.start {
                sorted = false;
                report.push(at(Finding::new("E-SWAP-ORDER")
                    .field("index", json!(later.index))
                    .field("start", json!(later.start))
                    .field("previous_start", json!(earlier.start))));
            }
        }

        let own = (
            element.get("start").and_then(Value::as_i64),
            element.get("end").and_then(Value::as_i64),
        );
        for window in &windows {
            if let Some(reason) = out_of_range(window, own) {
                report.push(at(Finding::new("E-SWAP-RANGE")
                    .field("index", json!(window.index))
                    .field("start", json!(window.start))
                    .field("end", json!(window.end))
                    .field("reason", json!(reason))));
            }
        }

        // On an unsorted list an implicit `end` is the next swap's `start` in a list nobody
        // meant, so every overlap it produced would be an echo of the order error.
        if !sorted {
            continue;
        }
        let mut ordered: Vec<&Window> = windows.iter().collect();
        ordered.sort_by_key(|w| w.start);
        let mut active: Vec<&Window> = Vec::new();
        for window in ordered {
            active.retain(|open| open.end.is_some_and(|end| end > window.start));
            for open in &active {
                let overlap = open
                    .end
                    .unwrap_or(window.start)
                    .min(window.end.unwrap_or(i64::MAX))
                    - window.start;
                report.push(at(Finding::new("E-SWAP-OVERLAP")
                    .field("index", json!(open.index))
                    .field("start", json!(open.start))
                    .field("end", json!(open.end))
                    .field("other", json!(window.index))
                    .field("other_start", json!(window.start))
                    .field("other_end", json!(window.end))
                    .field("overlap", json!(overlap))));
            }
            active.push(window);
        }
    }
}

/// Why this swap is not a window inside the element, or `None` where it is.
fn out_of_range(
    window: &Window,
    (own_start, own_end): (Option<i64>, Option<i64>),
) -> Option<String> {
    if let Some(end) = window.declared_end
        && end <= window.start
    {
        return Some(format!("its `end` {end} is not after its `start`"));
    }
    if let Some(own_start) = own_start
        && window.start < own_start
    {
        return Some(format!(
            "it starts before the element's `start` {own_start}"
        ));
    }
    if let Some(own_end) = own_end {
        if window.start >= own_end {
            return Some(format!(
                "it starts at or after the element's `end` {own_end}"
            ));
        }
        if let Some(end) = window.declared_end
            && end > own_end
        {
            return Some(format!(
                "its `end` {end} is after the element's `end` {own_end}"
            ));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissive::Loose;

    fn codes(swaps: Value) -> Vec<String> {
        let document = json!({"frame":{"width":100,"height":100},"fps":30,"tracks":[{"name":"t",
            "elements":[{"id":"m","type":"image","start":1000,"end":2000,"source":"a.png",
            "width":10,"height":10,"fit":"literal","swaps":swaps}]}]});
        let loose = Loose::new("p.montagent.json", document);
        let mut report = Report::new("validate", None);
        check(&loose, &mut report);
        report.findings.iter().map(|f| f.code.clone()).collect()
    }

    #[test]
    fn a_sorted_abutting_list_is_clean_in_either_variant() {
        assert!(
            codes(json!([{"start":1000,"source":"b.png"},{"start":1200,"source":"c.png"}]))
                .is_empty()
        );
        assert!(
            codes(json!([
                {"start":1000,"end":1200,"source":"b.png"},
                {"start":1200,"end":1300,"source":"c.png"}
            ]))
            .is_empty()
        );
    }

    #[test]
    fn an_unsorted_list_is_an_order_error() {
        assert_eq!(
            codes(json!([{"start":1500,"source":"b.png"},{"start":1200,"source":"c.png"}])),
            ["E-SWAP-ORDER"]
        );
    }

    #[test]
    fn an_explicit_end_past_the_next_start_is_an_overlap() {
        assert_eq!(
            codes(json!([
                {"start":1000,"end":1300,"source":"b.png"},
                {"start":1200,"source":"c.png"}
            ])),
            ["E-SWAP-OVERLAP"]
        );
    }

    #[test]
    fn a_swap_outside_the_element_or_empty_is_a_range_error() {
        assert_eq!(
            codes(json!([{"start":900,"source":"b.png"}])),
            ["E-SWAP-RANGE"]
        );
        assert_eq!(
            codes(json!([{"start":2000,"source":"b.png"}])),
            ["E-SWAP-RANGE"]
        );
        assert_eq!(
            codes(json!([{"start":1500,"end":2100,"source":"b.png"}])),
            ["E-SWAP-RANGE"]
        );
        assert_eq!(
            codes(json!([{"start":1500,"end":1500,"source":"b.png"}])),
            ["E-SWAP-RANGE"]
        );
    }
}
