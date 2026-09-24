//! The font-swap census ADR-0007 has owed since #143, measured (#186).
//!
//! [#143](https://github.com/MBehtemam/Montagent/issues/143) re-vendored the fixture's
//! typeface from **SF Pro Rounded** to **Open Runde**, because SF Pro Rounded is not
//! redistributable (ADR-0057). It named the consequence and did not resolve it: the
//! fixture's 22 text elements carry sizes, box heights and line breaks measured against
//! the old face's metrics, and ADR-0057 records that *"no font claims formal metric
//! compatibility"*.
//!
//! This file is the measurement. It answers two questions that are **independent**, and
//! whose conflation is the mistake #186 was written around:
//!
//! 1. **Do the fixture's declared layouts still hold?** Measured here: yes, with margin.
//!    No line re-partitions, no block height changes, and no line overruns its declared
//!    `width` under either face. That discharges the ADR-0007 census and settles the width
//!    term ADR-0014 parks as `UNCHECKED` — for this fixture, it is now checked.
//! 2. **Can the reference-frame gate therefore include text?** No, and no measurement of
//!    this kind could ever make it so. A declared box is a *layout* claim; the gate makes
//!    a *pixel* claim. Open Runde's advances run 2.0–9.4 % wider than SF Pro Rounded's, so
//!    every glyph on a centred line lands somewhere else — `tests/reference_frames.rs`
//!    measures the consequence at SSIM 0.6743 and 0.6597 in the text regions, against
//!    0.9874 and 0.9646 in the drawn regions beside them.
//!
//! `CONTEXT.md`'s Reference frame entry says a divergence is *"masked only while its cause
//! is unfixed"*. The cause here is a licence, and it does not get fixed. **The text mask is
//! permanent**, and ADR-0085 records it as settled rather than pending.
//!
//! # Why a test and not a script
//!
//! The other committed evidence in this repository (`decode_logo_alpha.py` and the ADR
//! scans) is stdlib-only Python, deliberately, so it runs on a bare interpreter. That is
//! the right shape for arithmetic over a PNG. It is the wrong shape here: a hand-rolled
//! OTF parser would measure *a* number, not the number this build shapes and draws. So the
//! census goes through `measure` — the verb whose output *is* the text engine's output
//! (spec #168) — exactly as `tests/effects.rs` measures the mask by rendering it.
//!
//! # The SF Pro Rounded half is machine-dependent, by necessity
//!
//! The delta needs the face the published video was typeset in, and that face cannot be
//! committed — which is the whole reason #143 happened. It is present at
//! [`SF_PRO_ROUNDED_BOLD`] on a stock macOS install, and absent in CI. So the Open Runde
//! half asserts unconditionally and the delta half **skips with a printed note** when the
//! file is not there. A skip is honest; vendoring the font to avoid one would be the
//! licence violation ADR-0057 exists to prevent, and estimating the numbers instead is
//! what #186 explicitly forbids: *"if it is not obtainable, say so rather than
//! estimating."*

use std::path::{Path, PathBuf};

use montagent_core::verbs::measure::{Answer, Ask};
use serde_json::{Value, json};

mod common;
use common::{canonical, write_project};

/// Where a stock macOS keeps the face the published video was typeset in.
///
/// Not vendored and never to be: ADR-0057's licence gate is what sent the fixture to Open
/// Runde in the first place.
const SF_PRO_ROUNDED_BOLD: &str = "/Library/Fonts/SF-Pro-Rounded-Bold.otf";

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json")
        .canonicalize()
        .expect("the committed fixture")
}

fn open_runde() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/fonts/OpenRunde-Bold.otf")
        .canonicalize()
        .expect("the vendored font (#143)")
}

/// One row of the census: what the fixture declares, and what the engine derives.
struct Row {
    id: &'static str,
    /// The element's declared `width` — the box the widest line must fit inside.
    declared_width: i64,
    /// The element's declared `height`, where it has one.
    declared_height: Option<i64>,
    /// Open Runde's widest-line advance, to the hundredth of a pixel.
    open_runde: f64,
    /// SF Pro Rounded's, for the same element. Asserted only where the face is present.
    sf_pro: f64,
    lines: usize,
    /// ADR-0028's `ceil(size × line_height × line_count)`, which carries no font term —
    /// recorded so that the claim "it carries no font term" is asserted, not assumed.
    block_height: i64,
}

/// The census. Every number is measured output, pasted back as an assertion.
///
/// Looked up **by id**, never by position: `measure --all` answers in the order it walks
/// the document, and pinning the census to that order would make this file fail whenever
/// a track is reordered — a change that moves no pixel and no metric.
#[rustfmt::skip]
const CENSUS: &[Row] = &[
    Row { id: "sentence-05",    declared_width: 984, declared_height: Some(169), open_runde: 805.83984375    , sf_pro: 744.24560546875 , lines: 1, block_height:  61 },
    Row { id: "sentence-06",    declared_width: 984, declared_height: Some(169), open_runde: 509.27557373046875, sf_pro: 470.66748046875 , lines: 2, block_height: 126 },
    Row { id: "sentence-07",    declared_width: 984, declared_height: Some(169), open_runde: 528.0393676757812, sf_pro: 486.36474609375 , lines: 2, block_height: 126 },
    Row { id: "sentence-08",    declared_width: 984, declared_height: Some(169), open_runde: 709.12109375    , sf_pro: 651.728515625   , lines: 1, block_height:  61 },
    Row { id: "sentence-quiz",  declared_width: 984, declared_height: Some(169), open_runde: 805.83984375    , sf_pro: 744.24560546875 , lines: 1, block_height:  61 },
    Row { id: "intro-title",    declared_width: 984, declared_height: Some(128), open_runde: 604.8189086914062, sf_pro: 558.3349609375  , lines: 2, block_height: 128 },
    Row { id: "hook-05",        declared_width: 984, declared_height: Some(194), open_runde: 788.5625        , sf_pro: 723.63671875    , lines: 2, block_height: 194 },
    Row { id: "word-05",        declared_width: 984, declared_height: Some(97) , open_runde: 807.9375        , sf_pro: 750.01953125    , lines: 1, block_height:  97 },
    Row { id: "word-06",        declared_width: 984, declared_height: Some(97) , open_runde: 669.875         , sf_pro: 612.34765625    , lines: 1, block_height:  97 },
    Row { id: "word-07",        declared_width: 984, declared_height: Some(88) , open_runde: 784.7160034179688, sf_pro: 719.1796875     , lines: 1, block_height:  88 },
    Row { id: "word-08-target", declared_width: 984, declared_height: Some(54) , open_runde: 347.57635498046875, sf_pro: 318.404296875   , lines: 1, block_height:  54 },
    Row { id: "quiz-question",  declared_width: 984, declared_height: Some(161), open_runde: 903.66015625    , sf_pro: 839.74951171875 , lines: 2, block_height: 161 },
    Row { id: "count-5",        declared_width: 984, declared_height: Some(274), open_runde: 160.48828125    , sf_pro: 155.625         , lines: 1, block_height: 274 },
    Row { id: "count-4",        declared_width: 984, declared_height: Some(274), open_runde: 168.80007934570312, sf_pro: 161.2177734375  , lines: 1, block_height: 274 },
    Row { id: "count-3",        declared_width: 984, declared_height: Some(274), open_runde: 164.20205688476562, sf_pro: 156.8408203125  , lines: 1, block_height: 274 },
    Row { id: "count-2",        declared_width: 984, declared_height: Some(274), open_runde: 156.86293029785156, sf_pro: 150.15380859375 , lines: 1, block_height: 274 },
    Row { id: "count-1",        declared_width: 984, declared_height: Some(274), open_runde: 121.84730529785156, sf_pro: 119.51513671875 , lines: 1, block_height: 274 },
    Row { id: "word-quiz",      declared_width: 984, declared_height: Some(97) , open_runde: 807.9375        , sf_pro: 750.01953125    , lines: 1, block_height:  97 },
    Row { id: "hook-loop",      declared_width: 984, declared_height: Some(194), open_runde: 788.5625        , sf_pro: 723.63671875    , lines: 2, block_height: 194 },
    Row { id: "word-08-bridge", declared_width: 984, declared_height: Some(39) , open_runde: 248.26881408691406, sf_pro: 227.431640625   , lines: 1, block_height:  39 },
    Row { id: "chip-text",      declared_width: 238, declared_height: Some(84) , open_runde: 187.078125      , sf_pro: 172.859375      , lines: 1, block_height:  58 },
    Row { id: "handle-text",    declared_width: 472, declared_height: Some(84) , open_runde: 428.6583557128906, sf_pro: 394.2041015625  , lines: 1, block_height:  38 },
];

/// The advances above are the engine's own `f64`s, written out in full rather than rounded
/// — so this is round-trip slack, not measurement tolerance.
///
/// It is deliberately far tighter than a pixel. A loose epsilon here would have hidden the
/// transcription error that produced this file's first draft, where several literals were
/// typed from a two-decimal table and their remaining digits invented.
const EPSILON: f64 = 1e-9;

fn measure_all(project: &Path) -> Answer {
    montagent_core::verbs::measure::measure(
        project,
        &Ask {
            element: None,
            at: None,
            elements: None,
            all: true,
        },
    )
}

/// Every text element the fixture declares, keyed by id, as `measure --all` answers.
fn measured(project: &Path) -> Vec<(String, Value)> {
    let json = measure_all(project).to_json();
    json["measure"]["results"]
        .as_array()
        .expect("a batch answers with results")
        .iter()
        .map(|r| {
            let id = r["id"]
                .as_str()
                .expect("a fixture element has an id")
                .to_string();
            let ok = r["ok"].clone();
            assert!(!ok.is_null(), "{id} did not measure: {r}");
            (id, ok)
        })
        .collect()
}

/// The fixture's own text elements, re-declared against `font_file`.
///
/// The elements are copied out of the committed document rather than retyped, so the
/// census cannot silently describe text the fixture no longer carries. Only the font chain
/// differs between the two runs — that is the whole experiment.
fn project_in(font_file: &Path, line: u32) -> PathBuf {
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(fixture()).expect("read"))
        .expect("the fixture parses");
    let mut elements = Vec::new();
    collect_text(&doc, &mut elements);
    assert_eq!(
        elements.len(),
        CENSUS.len(),
        "the fixture's text element count moved"
    );

    let dir = common::tempdir(line);
    write_project(
        &dir,
        "census.montagent.json",
        &canonical(
            &json!({
                "frame": doc["frame"],
                "fps": doc["fps"],
                "fonts": {"brand": [{"file": font_file.display().to_string()}]},
                "tracks": [{"name": "census", "elements": elements}],
            })
            .to_string(),
        ),
    )
}

fn collect_text(node: &Value, out: &mut Vec<Value>) {
    match node {
        Value::Object(map) => {
            if map.get("type").and_then(Value::as_str) == Some("text") {
                out.push(node.clone());
            }
            for v in map.values() {
                collect_text(v, out);
            }
        }
        Value::Array(items) => items.iter().for_each(|v| collect_text(v, out)),
        _ => {}
    }
}

/// The census row for one element id.
fn row(id: &str) -> &'static Row {
    CENSUS
        .iter()
        .find(|row| row.id == id)
        .unwrap_or_else(|| panic!("{id} is a fixture text element the census does not record"))
}

/// The census and the fixture describe the same 22 elements — neither more nor fewer.
///
/// Asserted separately from the per-row checks because the failure it catches is the one a
/// per-row loop cannot see: an element *deleted* from the fixture leaves every surviving
/// row passing.
fn assert_census_covers(rows: &[(String, Value)]) {
    let mut measured: Vec<&str> = rows.iter().map(|(id, _)| id.as_str()).collect();
    let mut recorded: Vec<&str> = CENSUS.iter().map(|row| row.id).collect();
    measured.sort_unstable();
    recorded.sort_unstable();
    assert_eq!(
        measured, recorded,
        "the fixture's text elements and the census have diverged"
    );
}

// ---------------------------------------------------------------------------
// The census itself.
// ---------------------------------------------------------------------------

/// The Open Runde half, which asserts unconditionally: this is the face the repository
/// actually ships and renders in.
#[test]
fn the_fixtures_text_measures_as_the_census_records_under_open_runde() {
    let rows = measured(&project_in(&open_runde(), line!()));
    assert_census_covers(&rows);

    for (id, got) in &rows {
        let row = row(id);
        let advance = got["advance_width"].as_f64().expect("an advance");
        assert!(
            (advance - row.open_runde).abs() < EPSILON,
            "{id}: advance {advance} is not the recorded {}",
            row.open_runde
        );
        assert_eq!(
            got["line_count"].as_u64().expect("a line count") as usize,
            row.lines,
            "{id}: the line partition moved"
        );
        assert_eq!(
            got["block_height"].as_i64().expect("a block height"),
            row.block_height,
            "{id}: the block height moved"
        );
    }
}

/// The construction the delta rests on: measuring the fixture's elements in a project of
/// their own gives the same numbers as measuring the committed fixture directly.
///
/// Without this the census would be a statement about a synthesised document rather than
/// about the fixture, and the two could drift apart without anything noticing.
#[test]
fn the_synthesised_census_project_measures_identically_to_the_fixture_itself() {
    let direct = measured(&fixture());
    let synthesised = measured(&project_in(&open_runde(), line!()));
    assert_census_covers(&direct);
    assert_census_covers(&synthesised);

    // By id, because the two documents walk their elements in different orders — which is
    // itself the reason this test exists rather than being assumed.
    for (id, a) in &direct {
        let b = synthesised
            .iter()
            .find(|(other, _)| other == id)
            .map(|(_, value)| value)
            .unwrap_or_else(|| panic!("{id} is missing from the synthesised project"));
        assert_eq!(
            a["advance_width"], b["advance_width"],
            "{id}: the synthesised project does not reproduce the fixture's own measurement"
        );
        assert_eq!(a["line_count"], b["line_count"], "{id}: line count");
        assert_eq!(a["block_height"], b["block_height"], "{id}: block height");
    }
}

/// **The census's verdict: every declared layout still holds.**
///
/// This is what #143 owed and ADR-0007 requires. The width term is the only font-dependent
/// one — ADR-0008 forbids auto-wrap, so a longer advance overflows rather than re-wrapping,
/// and ADR-0028's block height carries no font term at all.
#[test]
fn no_element_overruns_its_declared_box_under_either_face() {
    let faces: Vec<(&str, PathBuf)> = match available_sf_pro() {
        Some(sf) => vec![("Open Runde", open_runde()), ("SF Pro Rounded", sf)],
        None => vec![("Open Runde", open_runde())],
    };

    for (name, file) in faces {
        let rows = measured(&project_in(&file, line!()));
        assert_census_covers(&rows);
        let mut tightest: Option<(&str, f64)> = None;
        for (id, got) in &rows {
            let row = row(id);
            let advance = got["advance_width"].as_f64().expect("an advance");
            let width = row.declared_width as f64;
            assert!(
                advance <= width,
                "{name}: {id} overruns its declared width — {advance:.2} > {width}"
            );
            if let Some(height) = row.declared_height {
                assert!(
                    got["block_height"].as_i64().expect("a height") <= height,
                    "{name}: {id} overruns its declared height"
                );
            }
            let used = advance / width;
            if tightest.is_none_or(|(_, worst)| used > worst) {
                tightest = Some((row.id, used));
            }
        }
        let (id, used) = tightest.expect("22 elements");
        println!(
            "CENSUS  {name}: 22 of 22 elements fit their declared box. \
             Tightest is {id} at {:.1}% of its declared width — a face {:.1}% wider \
             than this one would be the first to overflow it.",
            used * 100.0,
            (1.0 / used - 1.0) * 100.0,
        );
    }
}

/// The delta #186 asked for: the same figures under the face the published video used.
///
/// Skips, loudly, where that face is not installed — see this file's header.
#[test]
fn the_delta_against_sf_pro_rounded_reproduces_where_the_face_is_installed() {
    let Some(sf) = available_sf_pro() else {
        println!(
            "CENSUS  SKIPPED the SF Pro Rounded delta — {SF_PRO_ROUNDED_BOLD} is not on this \
             machine. The face cannot be vendored (ADR-0057, and #143's whole reason), so \
             its absence is an ordinary state and not a broken checkout. The Open Runde \
             half above asserted in full."
        );
        return;
    };

    let rows = measured(&project_in(&sf, line!()));
    assert_census_covers(&rows);
    let (mut min_ratio, mut max_ratio) = (f64::MAX, f64::MIN);
    for (id, got) in &rows {
        let row = row(id);
        let advance = got["advance_width"].as_f64().expect("an advance");
        assert!(
            (advance - row.sf_pro).abs() < EPSILON,
            "{id}: SF Pro Rounded advance {advance} is not the recorded {}",
            row.sf_pro
        );
        // The line partition is the author's own `\n` count under ADR-0008, so a face
        // swap cannot move it — asserted rather than assumed, because it is the premise
        // the whole census rests on.
        assert_eq!(
            got["line_count"].as_u64().expect("a line count") as usize,
            row.lines,
            "{id}: the line partition moved under a face swap, which ADR-0008 forbids"
        );
        assert_eq!(
            got["block_height"].as_i64().expect("a block height"),
            row.block_height,
            "{id}: the block height moved under a face swap, which ADR-0028's formula forbids"
        );
        let ratio = row.open_runde / advance;
        min_ratio = min_ratio.min(ratio);
        max_ratio = max_ratio.max(ratio);
    }
    println!(
        "CENSUS  Open Runde runs {:.1}%–{:.1}% wider than SF Pro Rounded across the 22 \
         elements. No line partition and no block height moves. This is why the reference \
         frames' text regions cannot gate, and ADR-0085 is why that mask is permanent.",
        (min_ratio - 1.0) * 100.0,
        (max_ratio - 1.0) * 100.0,
    );
}

fn available_sf_pro() -> Option<PathBuf> {
    let path = PathBuf::from(SF_PRO_ROUNDED_BOLD);
    path.is_file().then_some(path)
}
