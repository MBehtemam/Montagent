//! PROTOTYPE (#614): an image's `swaps`, read once for every consumer.
//!
//! The resolution of #595 shortlisted a list of timed swaps over a base `source`: each
//! swap is `{start, end?, source}` in absolute milliseconds, sorted, never overlapping,
//! stepped, and drawn in the element's one declared box. Every consumer that used to read
//! `source` with `as_str()` and stop — the renderer, the missing-file and fit checks, the
//! media digest, `R-SOURCE-CUT-POP`, `query --at` — asks this module instead, so that a
//! swap's file is never skipped silently.
//!
//! Read off the permissive [`Value`] rather than the model, because every one of those
//! consumers already reads that way. A malformed swap (no integer `start`, no string
//! `source`) is the schema check's fact and is skipped here, as every other reader in the
//! crate skips a value it cannot read.

use serde_json::Value;

/// One swap, with its end resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window<'a> {
    /// Its position in the element's `swaps` list, for findings that name it.
    pub index: usize,
    pub start: i64,
    /// The `end` the document wrote, if any.
    pub declared_end: Option<i64>,
    /// The instant it stops showing: its own `end`, else the next swap's `start`, else the
    /// element's own `end`. `None` only where none of those is a readable integer.
    pub end: Option<i64>,
    pub source: &'a str,
}

/// The element's swaps, in document order, each with its end resolved.
///
/// "The next swap" is the next one in *document* order: an unsorted list is
/// `E-SWAP-ORDER`, so the order a reader sees is the order this resolves in rather than a
/// sort that would make the error invisible in the picture.
pub fn windows(element: &Value) -> Vec<Window<'_>> {
    let element_end = element.get("end").and_then(Value::as_i64);
    let read: Vec<(usize, i64, Option<i64>, &str)> = listed(element)
        .iter()
        .enumerate()
        .filter_map(|(index, swap)| {
            Some((
                index,
                swap.get("start")?.as_i64()?,
                swap.get("end").and_then(Value::as_i64),
                swap.get("source")?.as_str()?,
            ))
        })
        .collect();
    read.iter()
        .enumerate()
        .map(|(i, &(index, start, declared_end, source))| Window {
            index,
            start,
            declared_end,
            end: declared_end.or_else(|| {
                read.get(i + 1)
                    .map(|&(_, next_start, _, _)| next_start)
                    .or(element_end)
            }),
            source,
        })
        .collect()
}

/// The file this element draws at `instant`: the swap whose `[start, end)` holds it, else
/// the base `source`. The first matching swap wins, which only matters on a document
/// `E-SWAP-OVERLAP` already refuses.
pub fn showing(element: &Value, instant: i64) -> Option<&str> {
    windows(element)
        .into_iter()
        .find(|w| w.start <= instant && w.end.is_some_and(|end| instant < end))
        .map(|w| w.source)
        .or_else(|| element.get("source").and_then(Value::as_str))
}

/// Every file the element references: the base `source` first, then each swap's, in
/// document order and with repeats — a consumer that probes once per file dedupes itself.
pub fn sources(element: &Value) -> Vec<&str> {
    element
        .get("source")
        .and_then(Value::as_str)
        .into_iter()
        .chain(
            listed(element)
                .iter()
                .filter_map(|swap| swap.get("source").and_then(Value::as_str)),
        )
        .collect()
}

/// The raw `swaps` list, on an `image` only. On any other type the key is
/// `E-SCHEMA-UNKNOWN-KEY`, and reading it anyway would add a second finding — a missing
/// file nothing draws — to the one mistake.
fn listed(element: &Value) -> &[Value] {
    if element.get("type").and_then(Value::as_str) != Some("image") {
        return &[];
    }
    element
        .get("swaps")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn mouth() -> Value {
        json!({"type":"image","start":1000,"end":2000,"source":"closed.png","swaps":[
            {"start":1000,"source":"small.png"},
            {"start":1200,"source":"open.png"},
            {"start":1500,"end":1600,"source":"round.png"}
        ]})
    }

    #[test]
    fn an_endless_swap_runs_to_the_next_start_and_the_last_to_the_element_end() {
        let element = mouth();
        let ends: Vec<_> = windows(&element).iter().map(|w| w.end).collect();
        assert_eq!(ends, [Some(1200), Some(1500), Some(1600)]);
    }

    #[test]
    fn the_base_shows_outside_every_swap_and_after_an_explicit_end() {
        let element = mouth();
        assert_eq!(showing(&element, 1000), Some("small.png"));
        assert_eq!(showing(&element, 1199), Some("small.png"));
        assert_eq!(showing(&element, 1200), Some("open.png"));
        assert_eq!(showing(&element, 1599), Some("round.png"));
        assert_eq!(showing(&element, 1600), Some("closed.png"));
        assert_eq!(showing(&element, 1999), Some("closed.png"));
    }

    #[test]
    fn an_endless_last_swap_holds_to_the_element_end() {
        let element = json!({"type":"image","start":0,"end":500,"source":"a.png",
            "swaps":[{"start":100,"source":"b.png"}]});
        assert_eq!(showing(&element, 50), Some("a.png"));
        assert_eq!(showing(&element, 499), Some("b.png"));
    }

    #[test]
    fn sources_lists_the_base_then_every_swap() {
        assert_eq!(
            sources(&mouth()),
            ["closed.png", "small.png", "open.png", "round.png"]
        );
        assert_eq!(
            sources(&json!({"type":"image","source":"a.png"})),
            ["a.png"]
        );
        let rect = json!({"type":"rect","swaps":[{"start":0,"source":"b.png"}]});
        assert!(sources(&rect).is_empty());
    }
}
