//! `R-EASE-INERT` — ADR-0052: an `ease` attached to a segment across which the value does
//! not change, so it *"describes motion that doesn't happen"*.
//!
//! ADR-0038 made `ease` required on every keyframe record but the first, closing a schema
//! silence that had produced three different agent behaviours on the identical text, and
//! named the cost without designing the remedy: a **hold segment** — two consecutive
//! records whose `v` is unchanged — still has to carry an explicit `ease`, typically
//! `"linear"`. This check is that remedy.
//!
//! ## Literal equality, no epsilon
//!
//! The comparison is between two **author-written** `v` fields, not resolved values:
//! *"nothing sits between the two records but the file itself — no interpolation math, no
//! rasterization, no round trip through a renderer."* So unlike
//! [`crate::checks::cut`]'s `R-SOURCE-CUT-POP`, whose per-property tolerance table exists
//! to absorb the noise *resolving* a value introduces, there is no noise source here for a
//! tolerance to absorb. `1.0` against `0.9999999999` is a real, if minuscule, authored
//! difference, and a finding calling them the same would state an intent the file does not
//! carry.
//!
//! What *is* absorbed is JSON's two spellings of one number: `1` and `1.0` are the same
//! value exactly, and [`same_value`] compares numbers numerically rather than by the type
//! `serde_json` happened to park them in. That is not a tolerance — the comparison is still
//! exact — it is the difference between comparing values and comparing encodings.
//!
//! ## Whole value, not per component
//!
//! `scale` is `[sx, sy]`, and a vector-valued `ease` can be inert on one axis while doing
//! real work on the other. ADR-0052 decided 2–1 that the check fires **only when every
//! component is unchanged**: *"a scale that moves on one axis is, in the ordinary sense, a
//! segment where something eases"*, there is no way to write "ease only applies to `sx`",
//! and the rule then degenerates identically to the scalar case with no per-property
//! branching. The dissent — that a per-component variant would be more complete — is
//! recorded in that ADR as available on evidence, and is not this check.
//!
//! ## One finding per run
//!
//! *"Three keyframe records all sharing one `v` produce two inert-ease segments back to
//! back, and reporting them as two lines duplicates the same fact."* Runs are maximal, and
//! the finding names the run's record count, its value, and the span its first and last
//! record sit at.
//!
//! **No repair is proposed** — delete the `ease`? change `v`? leave it? — which is an
//! authorial-intent judgment ADR-0006 forbids a finding from making, symmetric with
//! `R-SOURCE-CUT-POP`'s refusal to say whether a pop was deliberate.
//!
//! ## Array order, not clock order
//!
//! Records are read in the order the file writes them. ADR-0052 asks about *"adjacent
//! keyframe records"* of an author-written list and ADR-0038 makes `ease` *"a pure function
//! of position in the list"* — both are statements about the array, and a check that sorted
//! by `t` first would pair records the author never wrote next to each other. ADR-0082 now
//! guarantees array order and clock order are the same order for any list that reaches this
//! check, closing the gap [`crate::resolve`] used to record as
//! [#270](https://github.com/MBehtemam/Montagent/issues/270); this module keeps the array
//! reading because that is the one the `ease` rule it is auditing is written against.

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;

/// One record, as this check reads it.
struct Record<'a> {
    t: i64,
    v: &'a Value,
    /// The `ease` this record states, where it states one. A record with none states no
    /// inert ease — ADR-0038's rule is broken there, which is `E-KEYFRAME-EASE`'s finding
    /// and not this one's, and there is nothing here to call inert.
    ease: Option<&'a Value>,
}

/// `R-EASE-INERT`, over every keyframe list in the document.
pub fn check(document: &Loose, report: &mut Report) {
    for (track, element) in document.elements_in_tracks() {
        let subject = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
        // Every animated property, not [`crate::checks::MOVES_THE_BOX`]: that list is
        // narrowed for the geometric checks, which have nothing to learn from an instant
        // where only a fade changes. An inert `ease` is a fact about the *file*, and an
        // `opacity` or `volume` hold carries the same inert ceremony a `scale` hold does.
        // A stagger's unit lists too, the block's and each run override's (ADR-0151 §5).
        for (name, container, property) in crate::checks::keyframe_lists(element) {
            for run in runs(&records(container, property)) {
                let finding = inert(&subject, &name, run)
                    .at_file(document.path())
                    .at_element(&subject);
                report.push(match track {
                    Some(track) => finding.at_track(track),
                    None => finding,
                });
            }
        }
    }
}

/// One property's keyframe records, in the order the file writes them.
///
/// Empty where the property is absent or static ([`crate::checks::keyframe_records`] holds
/// ADR-0012's shape test). A record with no readable `t` or no `v` is the schema check's
/// finding and is dropped here rather than paired with its neighbour across the hole it
/// leaves.
fn records<'a>(element: &'a Value, property: &str) -> Vec<Record<'a>> {
    let Some(list) = crate::checks::keyframe_records(element, property) else {
        return Vec::new();
    };
    list.iter()
        .filter_map(|record| {
            Some(Record {
                t: record.get("t")?.as_i64()?,
                v: record.get("v")?,
                ease: record.get("ease").filter(|ease| !ease.is_null()),
            })
        })
        .collect()
}

/// Every maximal run of consecutive records holding one value, as index ranges into
/// `records`.
///
/// A run is at least two records. Its *segments* are the pairs inside it, and each of
/// those contributes an inert `ease` only if the later record actually states one — ADR-
/// 0012 puts `ease` on the record a segment *enters*, so the first record of a run is
/// never the one carrying the inert field.
fn runs<'a, 'b>(records: &'b [Record<'a>]) -> Vec<&'b [Record<'a>]> {
    let mut out = Vec::new();
    let mut start = 0;
    for i in 1..=records.len() {
        let continues = i < records.len()
            && same_value(records[i - 1].v, records[i].v)
            && records[i].ease.is_some();
        if continues {
            continue;
        }
        if i - start >= 2 {
            out.push(&records[start..i]);
        }
        start = i;
    }
    out
}

/// Are these two author-written values the same value?
///
/// Exact, with no epsilon (ADR-0052). Numbers are compared numerically so that `1` and
/// `1.0` — two JSON spellings of one value — are equal, and arrays element-wise so that
/// ADR-0052's whole-value rule for `scale` needs no per-property branch. Anything else
/// falls back to the tree's own equality, which is the literal comparison for values this
/// check has no arithmetic reading of.
fn same_value(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => match (a.as_f64(), b.as_f64()) {
            (Some(a), Some(b)) => a == b,
            _ => a == b,
        },
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| same_value(a, b))
        }
        (a, b) => a == b,
    }
}

/// The finding for one run: the property, the held value, the span, and the ease that
/// describes no motion.
fn inert(subject: &str, property: &str, run: &[Record<'_>]) -> Finding {
    // The eases actually carried, deduplicated in first-seen order: a run whose segments
    // all say `"linear"` says it once, and a run mixing two spellings names both rather
    // than picking one to print. The first record's own `ease` is excluded — it describes
    // the segment arriving *into* the run from whatever came before, which is real travel.
    let mut eases: Vec<String> = Vec::new();
    for record in &run[1..] {
        let spelling = record
            .ease
            .expect("a run continues only through records stating an `ease`")
            .to_string();
        if !eases.contains(&spelling) {
            eases.push(spelling);
        }
    }

    Finding::new("R-EASE-INERT")
        .field("element", json!(subject))
        .field("property", json!(property))
        .field("records", json!(run.len()))
        .field("value", run[0].v.clone())
        .field("from", json!(run[0].t))
        .field("to", json!(run[run.len() - 1].t))
        .field("ease", json!(eases.join(", ")))
}
