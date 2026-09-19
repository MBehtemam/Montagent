//! The four caption checks: a reading-pace floor, a repeat-duration disagreement, a
//! caption with nothing spoken under it, and a caption below the display floor.
//!
//! **All four are pure document reads.** ADR-0054 is explicit about the one that looks
//! like it needs the disk — *"No file is read; this is a pure document-level check"* —
//! and the other three never had a reason to open a file. That is why this module takes
//! no probe session: the property is structural rather than a discipline, in the way
//! [`crate::checks::source`]'s signature makes the opposite structural.
//!
//! **None of them is scoped to a track named `caption`.** ADR-0054: the checks *"carry
//! no track-name restriction anywhere in their mechanics — they operate on any element's
//! `runs`, `start`, `end`"*, and a track name is a free-text label with no semantics
//! elsewhere in this domain model. Whether a decorative overlay is a caption at all is
//! [#135](https://github.com/MBehtemam/Montaget/issues/135)'s question, and whatever
//! discriminator it settles on only narrows this scope forward.
//!
//! Two of the four compare a document-derived fact against a number that is not in the
//! document, so both are [`ThresholdProvenance::External`](crate::registry::ThresholdProvenance)
//! and both carry ADR-0061's citation inline. The other two decide on numbers the
//! project itself states — a duration, and one frame at the project's own `fps`.

use serde_json::{Value, json};

use crate::finding::{Citation, Finding};
use crate::permissive::Loose;
use crate::report::Report;

/// ADR-0034's threshold: *"`cps > 20` → `review`"*, from Netflix's and the BBC's
/// published timed-text style guides.
///
/// Calibrated for Latin/Cyrillic-family scripts, and ADR-0034 states the consequence
/// rather than solving it: against CJK text, whose published guidance runs 8–10 cps, this
/// number goes **silent** on lines that are genuinely too fast. The error is
/// under-flagging, never over-flagging, which is the acceptable direction for a `review`.
const PACE_CPS: i64 = 20;

/// ADR-0054's floor: 833.33 ms (5/6 second) rounded **up** to the stricter integer
/// millisecond, so a caption at exactly 833 ms is flagged rather than passed by a
/// rounding artifact. Script-agnostic, and independent of the project's `fps` — a
/// human-reading-time constant is not a grid fact.
const MIN_DURATION_MS: i64 = 834;

/// All four, over every `text` element in the document.
pub fn check(document: &Loose, report: &mut Report) {
    let subjects = text_elements(document);

    for subject in &subjects {
        pace(subject, document, report);
        min_duration(subject, document, report);
    }
    no_audio(&subjects, document, report);
    repeat_duration(&subjects, document, report);
}

/// One `text` element, reduced to what these four questions need.
///
/// **Not called a `Span`**, though [`crate::track::Span`] is the same two instants plus a
/// name: `CONTEXT.md`'s **Run** entry retires "span" for a text element's content, and this
/// type carries that content. It is also not [`crate::track::Span`] itself, and the reason
/// is what these checks are *for* — that traversal groups by track and sorts by time, and
/// every one of the four questions below is project-wide and text-typed. Reaching through it
/// would mean re-reading `type` and `runs` off a document the sequence has already thrown
/// away, and grouping by the one thing ADR-0054 says these checks must not read.
///
/// **Not called a `Caption`**, though all four codes are. `CONTEXT.md`'s glossary has no
/// **Caption** entry, and ADR-0054 is explicit that nothing in the document distinguishes
/// "this text is a spoken-line caption" from "this text is decorative UI" — that line is
/// [#135](https://github.com/MBehtemam/Montaget/issues/135)'s question. The codes are
/// ADR-0034's and are fixed; a type that asserted the same thing inside the check would be
/// asserting what the checks are scoped *not* to claim.
struct TextElement {
    element: String,
    track: Option<String>,
    start: i64,
    end: i64,
    /// The concatenated `runs` text, NFC-normalized.
    text: String,
}

impl TextElement {
    fn duration(&self) -> i64 {
        self.end - self.start
    }

    /// The count ADR-0034's metric is taken over: **grapheme clusters**, spaces included,
    /// `\n` excluded.
    ///
    /// A `\r` goes with it. The format's line break is a `\n` inside a run (ADR-0008), so a
    /// `\r` that reaches a run came in as half of a CRLF pair some paste carried — and
    /// segmentation treats `\r\n` as one cluster, so counting the leftover `\r` would make
    /// the measured rate depend on which line ending the author's clipboard used.
    ///
    /// Not `chars()`, and not UTF-16 code units: *"combining marks and emoji must not
    /// double-count"*. A family emoji is one cluster, seven `char`s and eleven UTF-16 code
    /// units, and a line of them would otherwise read as seven or eleven times too fast.
    fn characters(&self) -> i64 {
        use unicode_segmentation::UnicodeSegmentation;
        self.text
            .split(['\n', '\r'])
            .map(|line| line.graphemes(true).count() as i64)
            .sum()
    }
}

/// Every `text` element, in document order.
///
/// An element stating no integer `start` and `end`, or a range that is not a positive
/// half-open interval, contributes nothing — [`crate::track`]'s rule and its reason: those
/// are schema facts belonging to the check that owns the schema, and a pace that cannot be
/// computed is never reported as a pace that is not there.
fn text_elements(document: &Loose) -> Vec<TextElement> {
    document
        .elements_in_tracks()
        .filter(|(_, element)| element.get("type").and_then(Value::as_str) == Some("text"))
        .filter_map(|(track, element)| {
            let start = element.get("start")?.as_i64()?;
            let end = element.get("end")?.as_i64()?;
            (end > start).then(|| TextElement {
                element: super::subject_of(element.get("id").and_then(Value::as_str)),
                track: track.map(str::to_string),
                start,
                end,
                text: run_text(element),
            })
        })
        .collect()
}

/// An element's `runs`, concatenated and NFC-normalized.
///
/// ADR-0034 groups the repeat check on text that is byte-identical *"after NFC
/// normalization"*, and ADR-0007 already says a run's text is *"UTF-8, NFC"* — so on a
/// document Montaget wrote this normalizes nothing. It is here for the one the format
/// does not control: a caption pasted from a source that decomposes its accents reads
/// identically on screen and would otherwise group apart from its own repeat.
fn run_text(element: &Value) -> String {
    use unicode_normalization::UnicodeNormalization;
    let raw: String = element
        .get("runs")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(|run| run.get("text").and_then(Value::as_str))
        .collect();
    raw.nfc().collect()
}

/// ADR-0034's reading-pace floor, and ADR-0061's fenced exception in the flesh.
///
/// The finding's substance is the measurement, not the verdict: the cps, the character
/// count and the duration, so *"a reader can judge the margin without re-deriving it"* —
/// and so a reader who disagrees with 20 still has the number the disagreement is about.
fn pace(subject: &TextElement, document: &Loose, report: &mut Report) {
    let characters = subject.characters();
    let duration = subject.duration();
    // Integers, on both sides of the comparison. `characters / (duration / 1000) > 20` in
    // `f64` decides a marginal caption on a rounding error; this decides it on the two
    // numbers the document states. In `i128`, for `crate::exact`'s reason: both sides are
    // products of values a document may write, and a check must not answer differently
    // because one of them wrapped.
    if i128::from(characters) * 1000 <= i128::from(PACE_CPS) * i128::from(duration) {
        return;
    }
    report.push(
        located(
            Finding::new("R-CAPTION-PACE").field("measured_cps", json!(cps(characters, duration))),
            subject,
            document,
        )
        .field("characters", json!(characters))
        .field("duration", json!(duration))
        .field("threshold_cps", json!(PACE_CPS))
        .citation(Citation {
            threshold: json!(PACE_CPS),
            source: "Netflix and BBC timed-text guidance".into(),
            adr: "ADR-0034".into(),
        }),
    );
}

/// The measured rate, to one decimal place.
///
/// Rounded for reading only — the comparison above is made on the integers, so the
/// printed number can never be the reason a finding fired or did not.
fn cps(characters: i64, duration: i64) -> f64 {
    let exact = characters as f64 * 1000.0 / duration as f64;
    (exact * 10.0).round() / 10.0
}

/// The file, and the element's track — the location half every one of these findings
/// shares. `track` is a location, never a field: no template here names it.
fn located(finding: Finding, subject: &TextElement, document: &Loose) -> Finding {
    let finding = finding
        .at_file(document.path())
        .at_element(subject.element.clone())
        .field("element", json!(subject.element));
    match &subject.track {
        Some(name) => finding.at_track(name.clone()),
        None => finding,
    }
}

/// ADR-0054's floor. The second member of ADR-0061's fenced category, and the ADR that
/// introduced it is the one that says so: the floor sits *"in the same register as
/// `R-CAPTION-PACE`'s 20 cps"*, which is the shape the fence is drawn around. So it cites,
/// and states the duration it measured rather than a verdict.
fn min_duration(subject: &TextElement, document: &Loose, report: &mut Report) {
    let duration = subject.duration();
    if duration >= MIN_DURATION_MS {
        return;
    }
    report.push(
        located(Finding::new("R-CAPTION-MIN-DURATION"), subject, document)
            .field("duration", json!(duration))
            .field("start", json!(subject.start))
            .field("end", json!(subject.end))
            .field("floor", json!(MIN_DURATION_MS))
            .citation(Citation {
                threshold: json!(MIN_DURATION_MS),
                source: "Netflix Timed Text Style Guide, General Requirements: a 5/6-second \
minimum caption duration"
                    .into(),
                adr: "ADR-0054".into(),
            }),
    );
}

/// ADR-0054's audio-backing check: *"zero elements of any kind whose `type` is a
/// time-based audio source"* overlap the caption's window.
///
/// **Two types answer to that, not one.** ADR-0055: *"a `video` element is one element
/// with intrinsic audio, not a visual paired with a separate audio element"* — it carries
/// the same `volume` field an `audio` element does — so a caption over footage has
/// something declared under it. Counting only `audio` would flag it, and ADR-0054 chose
/// the opposite error direction on purpose: *"under-flagging is the acceptable error
/// direction"*, the same posture ADR-0034 took for its CJK gap.
///
/// **This is presence, not narration coverage.** A caption under a continuous music bed
/// passes, and ADR-0054 names the trigger for revisiting that rather than leaving it to be
/// rediscovered: the first real project pairing a persistent non-narration track with
/// captions. Telling a voice from a music bed is not deferred work — `CONTEXT.md` says
/// Montaget *"contains no model"*, and that classification has no deterministic boundary.
fn no_audio(subjects: &[TextElement], document: &Loose, report: &mut Report) {
    // Just the instants: which audio element is under a caption is not this finding's
    // subject, and naming one would have the check answer a question — *which* narration
    // line is missing — that the document cannot answer. The traversal is
    // `elements()` rather than `elements_in_tracks()` for the same reason the query is
    // project-wide: a backing element's track name is not part of the question.
    let backing: Vec<(i64, i64)> = document
        .elements()
        .filter(|element| {
            matches!(
                element.get("type").and_then(Value::as_str),
                Some("audio" | "video")
            )
        })
        .filter_map(|element| {
            Some((
                element.get("start")?.as_i64()?,
                element.get("end")?.as_i64()?,
            ))
        })
        .collect();

    for subject in subjects {
        // Half-open on both sides (ADR-0005): an audio element ending at 3000 is not under
        // a caption starting at 3000, which is the same boundary rule the overlap check
        // applies within a track.
        if backing
            .iter()
            .any(|(start, end)| *start < subject.end && *end > subject.start)
        {
            continue;
        }
        report.push(
            located(Finding::new("R-CAPTION-NO-AUDIO"), subject, document)
                .field("start", json!(subject.start))
                .field("end", json!(subject.end))
                .field("duration", json!(subject.duration())),
        );
    }
}

/// ADR-0034's repeat-duration disagreement: elements whose text is identical after NFC
/// normalization are grouped project-wide, and a group whose longest and shortest
/// on-screen durations differ by more than one frame is one finding.
///
/// **Symmetric, and not "shorter on repeat".** The ticket's framing bakes in a claim about
/// what a repeat is *for* — that a callback deserves at least as much time as the original
/// — and the opposite convention is equally ordinary. So this states that the durations
/// disagree, names every value, and claims nothing about which one is right; the same
/// shape ADR-0033's same-source check takes.
///
/// **One finding per group, never pairwise.** Five occurrences are one finding listing
/// five, not ten comparisons — ADR-0006's "scope the output, never the analysis".
fn repeat_duration(subjects: &[TextElement], document: &Loose, report: &mut Report) {
    // The tolerance's own denominator. No `fps` means no frame and so no tolerance, and
    // ADR-0034 is explicit that the tolerance is required rather than optional — a bare
    // inequality would flag two elements independently snapped to frame boundaries. The
    // check that says `fps` is missing owns that finding; this one goes quiet.
    let Some(fps) = document
        .value()
        .get("fps")
        .and_then(Value::as_i64)
        .filter(|fps| *fps > 0)
    else {
        return;
    };

    // Groups in first-appearance order, and members within a group in document order, so
    // the report is byte-identical across runs — and so nothing here sorts by size, which
    // would be the implication ADR-0043 forbids a finding to carry: that the larger group
    // is the correct one.
    let mut groups: Vec<(&str, Vec<&TextElement>)> = Vec::new();
    // An element whose `runs` carry no text at all is not a repeat of anything. ADR-0034
    // recognises a caption "by its text, not by where it sits", and the empty string is the
    // one value that is shared by elements which have nothing in common — two blank
    // elements of unequal length would otherwise be reported as one line shown two ways.
    for subject in subjects.iter().filter(|subject| !subject.text.is_empty()) {
        match groups.iter_mut().find(|(text, _)| *text == subject.text) {
            Some((_, members)) => members.push(subject),
            None => groups.push((&subject.text, vec![subject])),
        }
    }

    for (text, members) in groups {
        let durations: Vec<i64> = members.iter().map(|member| member.duration()).collect();
        let (shortest, longest) = match (durations.iter().min(), durations.iter().max()) {
            (Some(shortest), Some(longest)) => (*shortest, *longest),
            _ => continue,
        };
        let spread = longest - shortest;
        // One frame at the project's `fps` — a number the document itself states, which is
        // what keeps this check inside ADR-0061's fact-only compartment even though
        // "captions shouldn't visibly shrink on repeat" is itself a human intuition. The
        // grid step is `1000/fps` ms and "not necessarily integral" (ADR-0035), so the
        // comparison is made on the two integers rather than through a division.
        if i128::from(spread) * i128::from(fps) <= 1000 {
            continue;
        }

        // The listing ADR-0034 asks the finding to carry — "one finding listing all five
        // (id, start, duration)" — composed the way `N-QUANTIZATION` composes its own
        // per-instance detail, and named by the template so a reader of the rendered report
        // sees every occurrence rather than a count. There is deliberately no census beside
        // it: ADR-0006's census is the majority-and-outlier move ("four of five are 1597"),
        // and this check is written to have no majority — a second, ranked grouping of the
        // same five values is the implication ADR-0043 forbids a census to carry, arrived at
        // from the other direction.
        let detail = members
            .iter()
            .map(|member| {
                format!(
                    "`{}` {}..{} ms ({} ms)",
                    member.element,
                    member.start,
                    member.end,
                    member.duration()
                )
            })
            .collect::<Vec<_>>()
            .join("; ");

        report.push(
            // No element location: the subject is a group, and naming one of its members
            // in the location would be the finding picking a side of a disagreement it is
            // written not to take.
            Finding::new("R-CAPTION-REPEAT-DURATION")
                .at_file(document.path())
                .field("text", json!(text))
                .field("count", json!(members.len()))
                .field("shortest", json!(shortest))
                .field("longest", json!(longest))
                .field("spread", json!(spread))
                .field("fps", json!(fps))
                .field("detail", json!(detail)),
        );
    }
}
