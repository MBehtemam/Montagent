//! `R-LINE-INK-COLLISION` — *"does one line's ink actually reach into the next line's?"*
//! (ADR-0087, [#325](https://github.com/MBehtemam/Montagent/issues/325)).
//!
//! # The blindness this closes
//!
//! ADR-0007 makes a line's slot *"the largest `size` among the runs on that line ×
//! `line_height`"* — a function of two numbers the document declares, and **never of the
//! font's ink**. That is a deliberate decision, and ADR-0087 does not reopen it: literal
//! sizes are what make the file readable without evaluation. `R-BOX-SLACK` (ADR-0058)
//! inherits the same blindness for the same reason, because it compares declared numbers
//! against each other and reads no font at all.
//!
//! On Latin the two readings agree closely enough that nothing notices. On a script whose
//! marks stack they do not. Thai puts an upper vowel and a tone mark above the base and a
//! lower vowel below it, and ADR-0087 measured the consequence on the two OFL faces a
//! `fonts vendor` gate would actually admit: at `line_height` 1.1 — correct for Latin in
//! the same face — consecutive lines' ink overlaps by **9.58 px** in Noto Sans Thai and
//! **23.77 px** in Sarabun, at size 55. Every field in the document is individually valid,
//! and before this check nothing in the tool said a word about it.
//!
//! # What it reports, and what it refuses to
//!
//! A fact: where the ink is, where the slot is, and by how much the first passed the
//! second. That is exactly what ADR-0006 says `validate` reports — a fact about the file
//! and the fonts on disk — and it is not a rule.
//!
//! It states **no repair**, and the reason is not squeamishness: ADR-0087 establishes that
//! two repairs are legitimate — raise `line_height`, or set the text in a face whose marks
//! fit — and *the document does not determine which*. `measure` can now show an author the
//! seam at any `line_height` they try (ADR-0087 added the ink extents for that), so the
//! information to choose is available; the choice is not Montagent's.
//!
//! **`review`, and never `error`.** A deliberately tight `line_height` is a real
//! typographic choice, and ADR-0006 reserves `error` for *guaranteed wrong*. This is ADR-0006's `review`
//! definition read literally: legal, renders, and you must look at a frame to know if it
//! was meant.
//!
//! # Why the check is not a `line_height` floor
//!
//! The obvious-looking alternative — refuse a `line_height` below some minimum for a
//! stacking script — is the candidate ADR-0087 rejected, and it rejected it on the
//! measurements rather than on principle: the floor is **1.3** on Noto Sans Thai and
//! **1.6** on Sarabun. It is a property of the *face*, not of the script, so there is no
//! constant to key on and no bound a sample of two establishes. What this check does
//! instead is measure the actual case in front of it, which needs no constant at all.
//!
//! # It reads the disk, and reaches no verdict about the fonts
//!
//! Like `crate::checks::fonts`, this opens the font files the project declares — it has to,
//! since the ink is in them — but needs no subprocess, so it runs alongside the document
//! checks rather than behind the probe session. An element whose `font` names a key the
//! `fonts` table does not declare, or whose declared file will not open, produces nothing
//! here: those are `crate::checks::fonts`' findings (`E-FONT-MISSING`) and its named gap
//! (a dangling `font` key has no code), and a second, differently-worded complaint about
//! the same missing file would be the drift ADR-0006's *"one code, one field set, one
//! template"* forbids.

use serde_json::{Value, json};

use crate::checks::{pluralised, subject_of};
use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;
use crate::verbs::measure::try_measure_element;

/// `R-LINE-INK-COLLISION`, over every `text` element in the document.
pub fn check(document: &Loose, report: &mut Report) {
    let file = document.path();

    for (track, element) in document.elements_in_tracks() {
        if element.get("type").and_then(Value::as_str) != Some("text") {
            continue;
        }
        // An element that cannot be measured is not an element with no collision — it is
        // one the question could not be asked of, and every cause is already somebody
        // else's finding. See the module doc.
        let Ok(measured) = try_measure_element(document, element) else {
            continue;
        };
        let Some(finding) = candidate(&measured) else {
            continue;
        };

        let mut finding = finding.at_file(file);
        if let Some(track) = track {
            finding = finding.at_track(track);
        }
        report.push(finding);
    }
}

/// This element's finding, if any pair of its inked lines actually overlaps.
///
/// **One finding per element, naming the worst seam and counting the rest.** A ten-line
/// Thai block collides at all nine seams by nearly the same amount, and nine findings
/// saying so is the noise ADR-0006 budgets against — the second one tells a reader nothing
/// the first did not. The count travels in a field so the report still says how much of
/// the element is affected.
fn candidate(measured: &crate::verbs::measure::Text) -> Option<Finding> {
    // A seam with no `overlap` is one where no glyph of either line shares horizontal space
    // with the other's — they cannot meet at any `line_height`, which is not the same fact
    // as meeting by a negative amount, and is why the field is nullable.
    let colliding: Vec<_> = measured
        .ink_seams
        .iter()
        .filter_map(|seam| Some((seam, seam.overlap?)))
        .filter(|(_, overlap)| *overlap > 0.0)
        .collect();
    // The worst, by overlap. `f64::total_cmp` rather than `partial_cmp`: an overlap is
    // derived from font coordinates and a NaN would silently make `max_by` return the
    // wrong seam rather than panic.
    let (worst, worst_overlap) = colliding.iter().max_by(|(_, a), (_, b)| a.total_cmp(b))?;

    let subject = measured
        .asked
        .id
        .clone()
        .unwrap_or_else(|| subject_of(None));
    // The slot both lines of the seam reserve. Read off the upper line rather than
    // recomputed from `size × line_height`, so the number in the finding is the one the
    // engine actually laid the block out with (ADR-0007's rule has one implementation).
    let slot = measured.lines.get(worst.above).map(|line| line.slot_height);

    Some(
        Finding::new("R-LINE-INK-COLLISION")
            .at_element(subject.clone())
            .field("element", json!(subject))
            .field("font", json!(measured.asked.font))
            .field("size", json!(measured.asked.size))
            .field(
                "line_height",
                json!(decimal(measured.asked.line_height_tenths)),
            )
            .field("above", json!(worst.above))
            .field("below", json!(worst.below))
            .field("overlap", json!(px(*worst_overlap)))
            .field("slot", json!(slot.map(px)))
            .field(
                "ink_bottom",
                json!(
                    measured
                        .lines
                        .get(worst.above)
                        .and_then(|l| l.ink_bottom)
                        .map(px)
                ),
            )
            .field(
                "ink_top",
                json!(
                    measured
                        .lines
                        .get(worst.below)
                        .and_then(|l| l.ink_top)
                        .map(px)
                ),
            )
            .field("collisions", json!(pluralised(colliding.len(), "seam"))),
    )
}

/// A `line_height` held as tenths, as the decimal the document writes it as.
///
/// The same one-way conversion `crate::text::tenths` makes for the prose form, made here
/// because a finding's *fields* are the canonical JSON and ADR-0006 generates the prose
/// from them — a template cannot do arithmetic, so the field has to carry the spelling the
/// author will recognise.
fn decimal(tenths: i64) -> String {
    format!("{}.{}", tenths / 10, tenths % 10)
}

/// A measurement rounded for the report, to the three decimals the rest of the surface
/// prints pixels at.
///
/// Presentational only, `crate::checks::box_slack::percent`'s posture: the comparison that
/// decided whether this finding fired was made on the unrounded value, so this number is
/// never the reason one did or did not.
fn px(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}
