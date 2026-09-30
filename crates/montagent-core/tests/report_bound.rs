//! ADR-0099: the text report's size is **O(distinct finding codes)**, not O(elements).
//!
//! The first test here is the falsifiable form of the whole decision, and it is the one
//! that would have caught the failure #388 was filed about: a report that grows with the
//! project outgrows its reader's context, and no constant in this repository can say when,
//! because the cap belongs to the client. So the claim under test is not *"the report
//! fits"* — Montagent cannot observe that — but *"the report does not grow"*, which it can.
//!
//! The bound's own hole is tested too, rather than left to the prose: `CACHE` is
//! O(sources) and may not collapse, so a report **does** grow with the number of media
//! files. That is stated in ADR-0099 and asserted here, so nobody later reads the first
//! test as a promise the tool does not make.

use montagent_core::finding::{Class, Finding};
use montagent_core::registry::CheckSet;
use montagent_core::report::Report;
use montagent_core::text;
use serde_json::{Value, json};

/// `k` captions with no audio under them — `review`-class, one code, one finding per
/// element, which is the shape 72 of the reference project's findings had.
fn captions_with_no_audio(k: usize) -> Report {
    let mut report = Report::new("validate", Some("p.json".into()));
    for i in 0..k {
        let start = i as i64 * 1000;
        report.push(
            Finding::new("R-CAPTION-NO-AUDIO")
                .at_file("p.json")
                .at_element(format!("caption-{i:04}"))
                .field("start", json!(start))
                .field("end", json!(start + 900))
                .field("duration", json!(900)),
        );
    }
    // Standing in for a whole validate run, so it records what one would (ADR-0112).
    report.record(CheckSet::Document);
    report.record(CheckSet::Disk);
    report
}

fn text_of(report: &Report) -> String {
    text::render(&report.to_json(), text::Options::default()).unwrap()
}

/// A `render` block whose element enumerations are all O(elements): every element mixed,
/// every element painted, and every element also listed under a reason it did not reach
/// the file. `sources` is held at one file deliberately — it is O(sources), which is the
/// bound's stated hole, and mixing it in would make this test measure two things.
fn render_block(elements: usize) -> Value {
    let ids: Vec<String> = (0..elements).map(|i| format!("el-{i:04}")).collect();
    let refused: Vec<Value> = ids
        .iter()
        .map(|id| json!({"element": id, "code": "E-SOURCE-AUDIO-UNREADABLE"}))
        .collect();
    json!({
        "path": "out.mp4",
        "from": 0, "to": 10_000, "partial": false,
        "duration_ms": 10_000, "frames": 250, "fps": 25,
        "width": 1080, "height": 1920, "encoded": null,
        "bytes": 1_234, "wall_ms": 1_000, "realtime": 10.0,
        "mixed": ids, "painted": ids,
        "not_mixed": refused, "not_painted": [], "painted_partially": [],
        "sources": ["clip.mp4"], "fonts": [],
    })
}

fn render_text(elements: usize, options: text::Options) -> String {
    let report = Report::new("render", Some("p.json".into()));
    text::render(
        &report.to_json_with("render", render_block(elements)),
        options,
    )
    .unwrap()
}

/// The bound's measure. **Line count is the exact claim** — it may not move at all, because
/// a line is what an enumeration costs. Characters may move by the *digits of the counts
/// that survive the collapse*, which is the price of ADR-0006's exact-count channel and is
/// O(log elements) rather than O(elements). `slack` is how many characters of digit growth
/// the report is allowed across the whole range under test.
fn flat_in_element_count(what: &str, small: &str, large: &str, slack: usize) {
    assert_eq!(
        small.lines().count(),
        large.lines().count(),
        "{what} grew by whole lines:\n--- small ---\n{small}\n--- large ---\n{large}"
    );
    let grew = large.len().saturating_sub(small.len());
    assert!(
        grew <= slack,
        "{what} grew by {grew} characters, more than the {slack} the surviving counts' own \
         digits can account for — something is still being enumerated:\n--- small ---\n\
         {small}\n--- large ---\n{large}"
    );
}

#[test]
fn the_report_does_not_grow_with_element_count() {
    // Ruling 1 of #388, in the only form that can fail. A hundredfold more captions buy
    // the report not one extra line: it is flat in the project's size, rather than merely
    // smaller than it was.
    //
    // Two counts print — the summary's and the collapsed line's — so 100 elements to
    // 10,000 can widen the report by the two extra digits each of them gains.
    flat_in_element_count(
        "the report",
        &text_of(&captions_with_no_audio(100)),
        &text_of(&captions_with_no_audio(10_000)),
        4,
    );
}

#[test]
fn the_render_block_does_not_grow_with_element_count() {
    // #388 ruling 6: leaving this block O(elements) would make the bound above false the
    // day it shipped, whatever the findings did. At the reference project's 217 elements it
    // was ~140 lines. Three counts print here — mixed, painted, and the reason group's.
    flat_in_element_count(
        "render's block",
        &render_text(100, text::Options::default()),
        &render_text(10_000, text::Options::default()),
        6,
    );
}

#[test]
fn the_report_grows_logarithmically_at_worst_and_the_growth_is_the_counts() {
    // The one thing that does grow is stated, so the bound is not oversold: each tenfold
    // step adds the same handful of digits, which is what O(log elements) looks like from
    // outside. A step that added more than the step before it would mean something is
    // enumerating again.
    let len = |k: usize| text_of(&captions_with_no_audio(k)).len();
    let (a, b, c) = (len(10), len(100), len(1_000));
    assert_eq!(b - a, c - b, "growth per tenfold step is not constant");
    assert!(b - a <= 4, "a tenfold step cost {} characters", b - a);
}

#[test]
fn the_render_blocks_counts_and_totals_survive_the_collapse() {
    // What collapses is the list of ids under a count, never the count. An agent that
    // cannot tell "silent" from "not mixed" chases the wrong defect, and the numbers are
    // how it tells them apart.
    let printed = render_text(1_000, text::Options::default());
    assert!(
        printed.contains("audio       1000 elements mixed"),
        "the mixed count is gone:\n{printed}"
    );
    assert!(
        printed.contains("painted     1000 elements"),
        "the painted count is gone:\n{printed}"
    );
    assert!(
        printed.contains("not mixed   E-SOURCE-AUDIO-UNREADABLE  1000 elements"),
        "the reason group lost its code or its count:\n{printed}"
    );
    assert!(
        !printed.contains("el-0500"),
        "an id survived past the bound:\n{printed}"
    );
    // The file's own numbers are not enumerations and are never behind a switch.
    assert!(
        printed.contains("out.mp4") && printed.contains("250 frames"),
        "{printed}"
    );
}

#[test]
fn verbose_restores_every_element_of_the_render_block() {
    let expanded = render_text(1_000, text::Options::verbose());
    assert!(
        expanded.contains("el-0500"),
        "an id is unreachable even under --verbose"
    );
    assert!(
        expanded.len() > render_text(1_000, text::Options::default()).len() * 10,
        "--verbose did not expand the block"
    );
}

#[test]
fn a_code_prints_in_full_up_to_three_instances_and_collapses_at_four() {
    // N = 3, and it is a bound on *repetition*: at three the reader sees the prose vary,
    // at four it has stopped varying in any way a fourth sentence would show.
    let three = text_of(&captions_with_no_audio(3));
    assert!(
        three.contains("caption-0002") && !three.contains("expand with --verbose"),
        "three instances must print in full:\n{three}"
    );

    let four = text_of(&captions_with_no_audio(4));
    assert!(
        four.contains("review  R-CAPTION-NO-AUDIO  4 — expand with --verbose"),
        "four instances must collapse to one counted line:\n{four}"
    );
    assert!(
        !four.contains("caption-0000"),
        "a collapsed code prints no instance at all — a first-K sample would make the \
         printed set arbitrary:\n{four}"
    );
}

#[test]
fn the_repetition_rule_sits_on_top_of_the_class_rule_and_does_not_replace_it() {
    // Two independent bounds, on two different properties. The class rule collapses
    // `note` because it is **inert**; the repetition rule collapses any code because it
    // is **repetitive**. So a single note still collapses, and notes must not start
    // expanding in small projects just because this rule arrived.
    let mut report = Report::new("validate", Some("p.json".into()));
    report.push(
        Finding::new("N-TRACK-GAP")
            .at_file("p.json")
            .at_track("captions")
            .field("start", json!(0))
            .field("end", json!(500))
            .field("duration", json!(500)),
    );
    assert_eq!(
        montagent_core::registry::spec("N-TRACK-GAP")
            .unwrap()
            .default_class(),
        Class::Note
    );
    let printed = text_of(&report);
    assert!(
        printed.contains("note  N-TRACK-GAP  1 — expand with --verbose"),
        "one note must still collapse on the inertness rule alone:\n{printed}"
    );
}

#[test]
fn verbose_restores_every_collapsed_finding() {
    let report = captions_with_no_audio(1_000);
    let expanded = text::render(&report.to_json(), text::Options::verbose()).unwrap();
    for id in ["caption-0000", "caption-0500", "caption-0999"] {
        assert!(expanded.contains(id), "--verbose did not restore {id}");
    }
}

#[test]
fn the_summary_line_keeps_the_exact_count_whatever_collapsed() {
    // The collapse hides no finding's existence and no finding's count. The summary is the
    // exact-count channel, which is the half of ADR-0006's "output may be filtered;
    // analysis may not" that licenses the collapse at all.
    let printed = text_of(&captions_with_no_audio(1_000));
    assert!(
        printed.starts_with("0 errors, 1000 reviews, 0 notes"),
        "the summary line lost its exact count:\n{printed}"
    );
}

#[test]
fn the_bound_is_on_element_count_and_the_cache_block_is_the_stated_hole() {
    // ADR-0099 states this rather than hiding it: `CACHE` is O(sources), ADR-0006/ADR-0011
    // forbid it collapsing — it is the sole mechanism announcing a source that changed on
    // disk — so the MCP token error can still recur on a project with enough media. A
    // bound that oversold itself would be the same dishonesty this map exists to fix.
    let with_misses = |n: usize| {
        let report = Report::new("validate", Some("p.json".into()));
        let mut json = report.to_json();
        json["cache_misses"] = (0..n)
            .map(|i| json!({"source": format!("clip-{i:04}.mp4"), "kind": "first"}))
            .collect();
        text::render(&json, text::Options::default()).unwrap()
    };
    assert!(
        with_misses(100).len() > with_misses(10).len(),
        "the CACHE block is expected to grow with sources; if it stopped, ADR-0099's \
         honesty clause is now wrong and wants amending rather than deleting"
    );
}

#[test]
fn a_compare_prints_every_drift_because_its_findings_are_the_answer() {
    // ADR-0099 §6 argues `render`'s block may be bounded *because* it is not the answer the
    // way a `compare` fact is. The repetition bound first shipped applying to `Drift`
    // anyway, which left four drifts of one code as a count of what changed and nothing
    // about what — the collapse `Class::prints_in_full`'s own comment warns against.
    let mut report = Report::new("compare", Some("p.json".into()));
    for i in 0..4 {
        report.push(
            Finding::new("D-SLACK-DRIFT")
                .at_file("p.json")
                .field("from", json!(format!("slack-from-{i}")))
                .field("to", json!(i * 1000 + 500))
                .field("ref_size", json!(500))
                .field("current_size", json!(700))
                .field("from_edges", json!("an end"))
                .field("to_edges", json!("a start"))
                .field("ref_project", json!("ref.json")),
        );
    }
    let printed = text_of(&report);
    for i in 0..4 {
        assert!(
            printed.contains(&format!("slack-from-{i}")),
            "drift {i} was not printed:\n{printed}"
        );
    }
    assert!(
        !printed.contains("D-SLACK-DRIFT  4"),
        "drift collapsed to a count:\n{printed}"
    );
}

#[test]
fn a_collapsed_line_names_a_route_the_verb_actually_has() {
    // `timeline`, `query`, `frame`, `measure` and `fonts list` have no `--verbose`. Before
    // the repetition bound only notes collapsed on them; now a fourth `error` or `review`
    // of one code does too, and a line saying "expand with --verbose" would send the
    // caller to a flag the verb rejects, with every instance's location gone.
    for tool in text::NO_VERBOSE {
        let mut report = captions_with_no_audio(4);
        report.tool = (*tool).to_string();
        let printed = text_of(&report);
        assert!(
            printed.contains("review  R-CAPTION-NO-AUDIO  4 — see --json"),
            "`{tool}` pointed its reader somewhere else:\n{printed}"
        );
        assert!(
            !printed.contains("--verbose"),
            "`{tool}` has no --verbose to name:\n{printed}"
        );
    }
    assert!(
        text_of(&captions_with_no_audio(4)).contains("4 — expand with --verbose"),
        "`validate` has the flag, and keeps pointing at it"
    );
}
