//! `frame`'s range mode — the contact sheet's tracer bullet (#488).
//!
//! Asserted at spec #168's **seam 1**, the core verb call, in `tests/frame.rs`'s style: a
//! real project goes in, and what comes back is read as values — `to_json()`, the text
//! form, and the picture's pixels through an independent decoder. The sizing function has
//! its own table tests beside it (`src/verbs/frame/sizing.rs`); everything else about the
//! sheet is asserted here, through the verb. The comparisons with other verbs live in
//! `tests/cross_verb.rs`.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use montagent_core::report::ExitCode;
use montagent_core::text::{self, Options};
use montagent_core::verbs::frame::{Answer, Ask, frame};
use serde_json::{Value, json};

mod common;
use common::{
    Scratch, fixture_project, keyframe_coincidence_fixture, keyframe_cross_boundary_fixture,
    long_state_fixture, tempdir, unpainted_fixture, write_project,
};

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

fn range(from: i64, to: i64) -> Ask {
    Ask {
        from: Some(from),
        to: Some(to),
        ..Ask::default()
    }
}

/// A sheet, refusing to continue if the verb did not draw one.
#[track_caller]
fn drawn(project: &Path, ask: &Ask) -> Answer {
    let answer = frame(project, ask);
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "frame did not draw a sheet: {}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );
    answer
}

/// The plain-text form, the one an agent reaching for pixels actually reads (ADR-0097 §6).
fn prose(json: &Value) -> String {
    text::render(json, Options::default()).expect("the report renders")
}

/// The fixture's whole document, drawn once for every test that reads it: 18 full-size
/// frames are the slowest thing in this file.
fn fixture_sheet() -> &'static (Value, Vec<u8>) {
    static SHEET: OnceLock<(Value, Vec<u8>)> = OnceLock::new();
    SHEET.get_or_init(|| {
        let project = fixture_project();
        let duration = common::document(&project)["duration"].as_i64().unwrap();
        let answer = drawn(
            &project,
            &Ask {
                png: true,
                ..range(0, duration)
            },
        );
        let bytes = answer.image().expect("a drawn sheet").bytes.clone();
        (answer.to_json(), bytes)
    })
}

/// A 1080×1920 project at 25 fps with one visual state per 200 ms rect on one track, over
/// nothing: `states` states in `[0, states × 200)`.
fn states_project(dir: &Path, states: usize) -> PathBuf {
    let elements: Vec<Value> = (0..states)
        .map(|i| {
            json!({
                "id": format!("card-{i:02}"), "type": "rect",
                "start": i as i64 * 200, "end": (i as i64 + 1) * 200,
                "x": 0, "y": 0, "origin": "top-left", "width": 1080, "height": 1920,
                "fill": format!("#{:02X}{:02X}40", (i * 8) % 256, 255 - (i * 8) % 256),
            })
        })
        .collect();
    write_project(
        dir,
        "states.montagent.json",
        &json!({
            "frame": {"width": 1080, "height": 1920},
            "fps": 25,
            "background": "#000000",
            "tracks": [{"name": "cards", "layer": 0, "elements": elements}],
        })
        .to_string(),
    )
}

/// The six `blind_to` tokens and their fixed sentences: ADR-0105 §5's table, with
/// `between-keyframes` as ADR-0106 rewords it.
const BLIND_TO: [(&str, &str); 6] = [
    (
        "inside-run",
        "Each tile shows the first painted frame of its visual state; change inside a state \
         \u{2014} a source clip's own cut, motion within a still-looking element \u{2014} is \
         not on this sheet. `frame --at <instant>` looks at any other instant.",
    ),
    (
        "between-keyframes",
        "Keyframed values are shown only where a tile falls; keyframe change points are \
         untiled unless asked for, and a keyframe tile shows the first painted frame at or \
         after the change, so a wrong easing curve shows only if its endpoints are wrong.",
    ),
    (
        "below-tile-width",
        "Tiles are served at the width stated above; detail finer than that is not visible \
         here. `frame --crop --at <instant>` looks closely at one region at true scale.",
    ),
    (
        "across-sheets",
        "Only tiles on this one sheet can be compared with each other; a relation with a \
         state outside this range is not visible.",
    ),
    ("audio", "Nothing audible is on this sheet."),
    (
        "motion",
        "A sheet is stills; whether motion looks right is `preview`'s question.",
    ),
];

// ---------------------------------------------------------------------------
// The main fixture: 18 visual states, one sheet at the target rung
// ---------------------------------------------------------------------------

#[test]
fn the_whole_fixture_is_eighteen_tiles_at_the_target_rung_served_at_184_px() {
    let (json, bytes) = fixture_sheet();
    let sheet = &json["sheet"];
    let picture = &sheet["picture"];

    assert_eq!(sheet["provenance"].as_array().unwrap().len(), 18);
    assert_eq!(picture["rung"], "target");
    assert_eq!(picture["served_tile_width"], 184);
    assert_eq!(
        (&picture["columns"], &picture["rows"]),
        (&json!(6), &json!(3))
    );

    // The picture is the size the tier serves, so nothing downscales it again.
    let decoded = image::load_from_memory(bytes).expect("the sheet decodes");
    assert_eq!(
        (decoded.width() as i64, decoded.height() as i64),
        (
            picture["width"].as_i64().unwrap(),
            picture["height"].as_i64().unwrap()
        )
    );
    assert_eq!((decoded.width(), decoded.height()), (1104, 1092));
    assert_eq!(picture["encoding"], "png");
}

#[test]
fn the_whole_fixture_drops_28_audio_only_boundaries_and_skips_nothing() {
    let (json, _) = fixture_sheet();
    let sheet = &json["sheet"];

    let dropped = &sheet["audio_boundaries_dropped"];
    assert_eq!(dropped["count"], 28);
    let named = dropped["boundaries"].as_array().unwrap();
    assert_eq!(named.len(), 28, "named as well as counted");
    for boundary in named {
        let changed = boundary["entering"].as_array().unwrap().len()
            + boundary["leaving"].as_array().unwrap().len();
        assert!(
            changed > 0,
            "a dropped boundary names what changed: {boundary}"
        );
    }

    assert_eq!(sheet["skipped"], json!([]));
    assert_eq!(sheet["coverage"]["not_depicted_ms"], 0);
    assert_eq!(sheet["coverage"]["states"], 18);
    assert_eq!(sheet["coverage"]["tiled"], 18);
    assert_eq!(json["summary"]["review"], 0);
}

#[test]
fn every_provenance_line_carries_its_instant_run_class_and_full_presence_set() {
    let (json, _) = fixture_sheet();
    let provenance = json["sheet"]["provenance"].as_array().unwrap();
    let mut previous_end = 0;
    for (i, tile) in provenance.iter().enumerate() {
        assert_eq!(tile["index"], i + 1, "tiles are numbered from 1");
        assert_eq!(tile["why"], "boundary");
        assert_eq!(tile["class"], "run");
        let (start, end) = (
            tile["run"]["start"].as_i64().unwrap(),
            tile["run"]["end"].as_i64().unwrap(),
        );
        assert_eq!(start, previous_end, "the runs partition the range");
        previous_end = end;
        // The first frame the grid paints inside the run: ⌊n·1000/25⌋ = 40n.
        let instant = tile["instant_ms"].as_i64().unwrap();
        assert!(instant >= start && instant < end, "{tile}");
        assert_eq!(instant % 40, 0);
        assert!(
            instant - 40 < start,
            "no earlier frame is inside the run: {tile}"
        );
        // The full set on every line, never a delta, and never an audio member.
        assert!(!tile["present"].as_array().unwrap().is_empty(), "{tile}");
    }
    assert_eq!(
        json["sheet"]["classes"],
        json!({"run": 18, "keyframe": 0, "infill": 0})
    );
}

#[test]
fn a_perfect_answer_carries_every_disclosure_in_the_json_and_in_the_text() {
    let (json, _) = fixture_sheet();
    let sheet = &json["sheet"];
    let text = prose(json);

    assert_eq!(
        sheet["rule"]["name"],
        "first-painted-frame-of-each-visual-state"
    );
    assert_eq!(sheet["rule"]["version"], 1);
    assert!(
        text.contains(sheet["rule"]["sentence"].as_str().unwrap()),
        "{text}"
    );

    let blind_to: Vec<(String, String)> = sheet["blind_to"]
        .as_array()
        .unwrap()
        .iter()
        .map(|spot| {
            (
                spot["token"].as_str().unwrap().to_string(),
                spot["sentence"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(
        blind_to,
        BLIND_TO
            .iter()
            .map(|(t, s)| (t.to_string(), s.to_string()))
            .collect::<Vec<_>>()
    );
    for (token, sentence) in BLIND_TO {
        assert!(
            text.contains(token) && text.contains(sentence),
            "{token}: {text}"
        );
    }

    // Width and rung, the coverage, and every dropped boundary, in the text in full.
    assert!(text.contains("served 184 px wide"), "{text}");
    assert!(text.contains("target rung"), "{text}");
    assert!(text.contains("28 audio-only boundaries dropped"), "{text}");
    for boundary in sheet["audio_boundaries_dropped"]["boundaries"]
        .as_array()
        .unwrap()
    {
        assert!(
            text.contains(&format!("{} ms", boundary["at"])),
            "boundary {boundary} is not in the text"
        );
    }
    assert!(text.contains("0 ms not depicted"), "{text}");
    assert!(text.contains("skipped     none"), "{text}");

    // Every provenance line is in the text with its instant and its presence set.
    for tile in sheet["provenance"].as_array().unwrap() {
        let present: Vec<&str> = tile["present"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| id.as_str().unwrap())
            .collect();
        let line = format!(
            "{:>3}  run  boundary  at {} ms  state {}..{}  label `{}`  {}",
            tile["index"].as_i64().unwrap(),
            tile["instant_ms"],
            tile["run"]["start"],
            tile["run"]["end"],
            tile["label"].as_str().unwrap(),
            present.join(", ")
        );
        assert!(text.contains(&line), "missing `{line}`:\n{text}");
    }

    // `frame` runs no check set, and says so first (ADR-0112).
    assert_eq!(json["check_sets"], json!([]));
    assert!(
        text.starts_with("no checks run (validate runs them) — "),
        "{text}"
    );
}

// ---------------------------------------------------------------------------
// #437's constructed document: an unpainted visual state
// ---------------------------------------------------------------------------

#[test]
fn an_unpainted_state_is_skipped_as_no_grid_frame_and_raises_validates_n_quantization() {
    let path = unpainted_fixture();
    let answer = drawn(&path, &range(0, 2000));
    let json = answer.to_json();
    let sheet = &json["sheet"];

    assert_eq!(
        sheet["skipped"],
        json!([{
            "run": {"start": 1010, "end": 1030},
            "reason": "no-grid-frame",
            "present": ["bg"],
        }])
    );
    assert_eq!(sheet["provenance"].as_array().unwrap().len(), 2);
    assert_eq!(sheet["coverage"]["not_depicted_ms"], 20);

    // The same finding `validate` raises: one identity whichever verb saw it (ADR-0118).
    let from_frame: Vec<&Value> = json["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["code"] == "N-QUANTIZATION")
        .collect();
    let validated = montagent_core::validate(&path).to_json();
    let from_validate: Vec<&Value> = validated["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["code"] == "N-QUANTIZATION" && finding["fields"]["from"].is_i64())
        .collect();
    assert_eq!(from_frame.len(), 1);
    assert_eq!(from_frame, from_validate);
    assert_eq!(from_frame[0]["class"], "review");

    // ADR-0119 §2: a checkless verb's finding prints after its scope.
    let text = prose(&json);
    assert!(
        text.starts_with("no checks run (validate runs them); 1 review — "),
        "{text}"
    );
    assert!(text.contains("1010..1030  no-grid-frame"), "{text}");
    assert!(text.contains("20 ms not depicted"), "{text}");
}

#[test]
fn more_than_three_unpainted_states_collapse_to_one_line_pointing_at_the_json() {
    // At 25 fps frames paint at 0, 40, 80, … — so each `[40k + 10, 40k + 30)` card is a
    // visual state no frame falls in, between states that do paint.
    let dir = tempdir(line!());
    let cards: Vec<Value> = (0..4)
        .map(|k| {
            json!({"id": format!("flash-{k}"), "type": "rect", "start": 40 * k + 10,
                   "end": 40 * k + 30, "x": 0, "y": 0, "origin": "top-left",
                   "width": 20, "height": 20, "fill": "#FFFFFF"})
        })
        .collect();
    let path = write_project(
        &dir,
        "flashes.montagent.json",
        &json!({
            "frame": {"width": 200, "height": 200}, "fps": 25, "background": "#000000",
            "tracks": [
                {"name": "bg", "layer": 0, "elements": [{"id": "bg", "type": "rect",
                  "start": 0, "end": 200, "x": 0, "y": 0, "origin": "top-left",
                  "width": 200, "height": 200, "fill": "#202020"}]},
                {"name": "flashes", "layer": 1, "elements": cards},
            ],
        })
        .to_string(),
    );

    let json = drawn(&path, &range(0, 200)).to_json();
    assert_eq!(json["sheet"]["skipped"].as_array().unwrap().len(), 4);
    assert_eq!(json["summary"]["review"], 4);
    let text = prose(&json);
    assert!(
        text.contains("review  N-QUANTIZATION  4 — see --json"),
        "{text}"
    );
}

// ---------------------------------------------------------------------------
// Budget: one degrade step, then a refusal that never thins
// ---------------------------------------------------------------------------

#[test]
fn nineteen_states_degrade_once_and_say_so() {
    let dir = tempdir(line!());
    let path = states_project(&dir, 19);
    let json = drawn(&path, &range(0, 19 * 200)).to_json();
    let picture = &json["sheet"]["picture"];
    assert_eq!(picture["rung"], "degraded");
    assert_eq!(picture["served_tile_width"], 173);
    assert_eq!(json["sheet"]["provenance"].as_array().unwrap().len(), 19);
    assert!(prose(&json).contains("degraded rung"));
}

#[test]
fn a_range_past_the_tile_width_refusal_is_refused_never_thinned() {
    let dir = tempdir(line!());
    let path = states_project(&dir, 31);
    let out = dir.join("sheet.jpg");
    let answer = frame(
        &path,
        &Ask {
            out: Some(out.clone()),
            ..range(0, 31 * 200)
        },
    );

    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
    assert!(answer.image().is_none(), "no sheet, thinned or otherwise");
    assert!(!out.exists(), "nothing written");
    let json = answer.to_json();
    assert!(
        json.get("sheet").is_none(),
        "a refusal carries the report alone"
    );
    let findings = json["findings"].as_array().unwrap();
    assert_eq!(findings.len(), 1);
    let refusal = &findings[0];
    assert_eq!(refusal["code"], "E-SHEET-OVERFLOW");
    assert_eq!(refusal["fields"]["states"], 31);
    assert_eq!(refusal["fields"]["fits"], 30);
    assert_eq!(refusal["fields"]["limit"], "tile-width");
    assert!(refusal.get("repair").is_none(), "{refusal}");

    // The fewest sub-ranges, their tiles shared evenly: 16 and 15, not 30 and 1 (ADR-0126).
    assert_eq!(
        refusal["fields"]["sub_ranges"],
        json!([{"from": 0, "to": 3200}, {"from": 3200, "to": 6200}])
    );

    let text = prose(&json);
    for words in [
        "31 visual states",
        "holds 30",
        "tile-width",
        "140 px",
        "each a sheet that fits: [0, 3200) and [3200, 6200).",
    ] {
        assert!(text.contains(words), "`{words}` missing: {text}");
    }

    // Thirty fit, at the degraded rung.
    let thirty = drawn(&path, &range(0, 30 * 200)).to_json();
    assert_eq!(thirty["sheet"]["picture"]["rung"], "degraded");
}

/// Every sub-range `E-SHEET-OVERFLOW` names for `[from, to)`, each requested in turn: every
/// one draws a sheet, and together they tile every state the refusal counted, cover the
/// range exactly and skip what it would have skipped. Returns the tiles on each sheet.
#[track_caller]
fn every_sub_range_draws(path: &Path, from: i64, to: i64) -> Vec<u64> {
    let json = frame(path, &range(from, to)).to_json();
    let refusal = &json["findings"][0];
    assert_eq!(refusal["code"], "E-SHEET-OVERFLOW", "{json}");
    let ranges = refusal["fields"]["sub_ranges"].as_array().unwrap();
    let mut next = from;
    let mut tiles = Vec::new();
    let mut skipped = 0;
    for sub_range in ranges {
        let (start, end) = (
            sub_range["from"].as_i64().unwrap(),
            sub_range["to"].as_i64().unwrap(),
        );
        assert_eq!(start, next, "consecutive: {ranges:?}");
        next = end;
        let sheet = drawn(path, &range(start, end)).to_json()["sheet"].clone();
        tiles.push(sheet["coverage"]["tiled"].as_u64().unwrap());
        skipped += sheet["coverage"]["skipped"].as_u64().unwrap();
    }
    assert_eq!(next, to, "the last sub-range ends the range");
    assert_eq!(
        tiles.iter().sum::<u64>(),
        refusal["fields"]["states"].as_u64().unwrap()
    );
    let states = common::document(path)["tracks"][0]["elements"]
        .as_array()
        .unwrap()
        .len();
    assert_eq!(
        tiles.iter().sum::<u64>() + skipped,
        states as u64,
        "no state is lost"
    );
    tiles
}

/// A 1080×1920 project at `fps` with three rects a second for `seconds` seconds: `[0, 960)`,
/// `[960, 990)` and `[990, 1000)` of each. The 10 ms state holds no frame at 24, 25 or 30
/// fps, and the 30 ms one holds a frame at 25 and 30 fps but none at 24.
fn seconds_project(dir: &Path, seconds: i64, fps: i64) -> PathBuf {
    let elements: Vec<Value> = (0..seconds)
        .flat_map(|i| {
            [(0, 960), (960, 990), (990, 1000)].map(|(a, b)| (i * 1000 + a, i * 1000 + b))
        })
        .enumerate()
        .map(|(n, (start, end))| {
            json!({
                "id": format!("card-{n:03}"), "type": "rect", "start": start, "end": end,
                "x": 0, "y": 0, "origin": "top-left", "width": 1080, "height": 1920,
                "fill": format!("#{:02X}{:02X}40", (n * 8) % 256, 255 - (n * 8) % 256),
            })
        })
        .collect();
    write_project(
        dir,
        "seconds.montagent.json",
        &json!({
            "frame": {"width": 1080, "height": 1920},
            "fps": fps,
            "background": "#000000",
            "tracks": [{"name": "cards", "layer": 0, "elements": elements}],
        })
        .to_string(),
    )
}

#[test]
fn every_sub_range_an_overflow_names_draws_a_sheet() {
    let dir = tempdir(line!());
    assert_eq!(
        every_sub_range_draws(&states_project(&dir, 31), 0, 31 * 200),
        [16, 15]
    );
}

#[test]
fn every_sub_range_draws_a_sheet_whatever_the_grid_leaves_unpainted() {
    // (fps, tiles on each sheet): two tiles a second at 30 fps, one at 24.
    for (fps, tiles) in [(30, &[21_u64, 21, 20][..]), (24, &[16, 15][..])] {
        let dir = tempdir(line!() * 100 + fps as u32);
        let path = seconds_project(&dir, 31, fps);
        assert_eq!(every_sub_range_draws(&path, 0, 31_000), tiles, "{fps} fps");
    }
}

// ---------------------------------------------------------------------------
// Refusals: every one bare `E-INVOCATION`, in the verb
// ---------------------------------------------------------------------------

#[track_caller]
fn refused(ask: &Ask) -> String {
    let answer = frame(&unpainted_fixture(), ask);
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
    assert!(answer.image().is_none());
    let json = answer.to_json();
    assert_eq!(json["findings"][0]["code"], "E-INVOCATION", "{json}");
    // ADR-0125 §5: a refusal carries the report alone — no sheet, so no tile 1 to quote and
    // no `blind_to` for a sheet that was not drawn.
    assert!(json.get("sheet").is_none(), "{json}");
    json["findings"][0]["fields"]["reason"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn at_with_a_range_is_refused() {
    let reason = refused(&Ask {
        at: Some(500),
        ..range(0, 2000)
    });
    assert!(
        reason.contains("`--at`") && reason.contains("`--from`/`--to`"),
        "{reason}"
    );
    // A half range with `--at` is the same mix of modes.
    let half = refused(&Ask {
        at: Some(500),
        from: Some(0),
        ..Ask::default()
    });
    assert_eq!(half, reason);
}

#[test]
fn a_call_naming_neither_mode_is_refused_naming_both() {
    // ADR-0097: nothing in the argument list says "sheet", so the refusal an agent meets
    // with no instant — including over MCP, where an argument the schema does not know is
    // dropped rather than refused — is where the range mode is taught.
    let reason = refused(&Ask::default());
    assert!(reason.contains("`--at <t>`"), "{reason}");
    assert!(reason.contains("`--from <t> --to <t>`"), "{reason}");
}

#[test]
fn a_range_over_a_file_that_is_no_project_carries_the_report_alone() {
    // ADR-0125 §5: the shape every refusal has holds where the document never parsed, and
    // where it parsed as something other than a project.
    let dir = tempdir(line!());
    let unparseable = frame(
        &write_project(
            &dir,
            "unparseable.montagent.json",
            "{\"frame\": {\"width\": 200,",
        ),
        &range(0, 2000),
    );
    assert_eq!(unparseable.report().exit_code(), ExitCode::Unparseable);
    let not_a_project = frame(
        &write_project(&dir, "not-a-project.montagent.json", "[1, 2, 3]"),
        &range(0, 2000),
    );
    assert_eq!(
        not_a_project.to_json()["findings"][0]["code"],
        "E-NOT-A-PROJECT"
    );
    for answer in [unparseable, not_a_project] {
        assert!(answer.image().is_none());
        let json = answer.to_json();
        assert!(json.get("sheet").is_none(), "{json}");
    }
}

#[test]
fn full_with_a_range_is_refused_with_the_tier_fact() {
    let reason = refused(&Ask {
        full: true,
        ..range(0, 2000)
    });
    assert!(reason.contains("`--full`"), "{reason}");
    assert!(reason.contains("true project pixels"), "{reason}");
    assert!(reason.contains("1568"), "the tier fact: {reason}");
}

#[test]
fn crop_with_a_range_is_refused_permanently_naming_the_two_step_loop() {
    let reason = refused(&Ask {
        crop: Some("0,0,100,100".into()),
        ..range(0, 2000)
    });
    assert!(reason.contains("`frame --from/--to`"), "{reason}");
    assert!(reason.contains("`frame --crop --at <instant>`"), "{reason}");
    assert!(reason.contains("whole frames by design"), "{reason}");

    // With `--full` as well, the crop's reason is the one given (ADR-0125).
    let both = refused(&Ask {
        crop: Some("0,0,100,100".into()),
        full: true,
        ..range(0, 2000)
    });
    assert_eq!(both, reason);
}

#[test]
fn a_range_missing_a_half_or_running_backwards_is_refused() {
    let only_from = refused(&Ask {
        from: Some(0),
        ..Ask::default()
    });
    assert!(only_from.contains("`--from` needs a `--to`"), "{only_from}");
    let only_to = refused(&Ask {
        to: Some(1000),
        ..Ask::default()
    });
    assert!(only_to.contains("`--to` needs a `--from`"), "{only_to}");
    let empty = refused(&range(1000, 1000));
    assert!(empty.contains("is no range"), "{empty}");
    let backwards = refused(&range(1000, 500));
    assert!(backwards.contains("is no range"), "{backwards}");
    let negative = refused(&range(-40, 1000));
    assert!(negative.contains("before the project starts"), "{negative}");
}

// ---------------------------------------------------------------------------
// The picture
// ---------------------------------------------------------------------------

#[test]
fn the_sheet_is_written_to_out_and_its_path_disclosed() {
    let dir = tempdir(line!());
    let out = dir.join("sheet.jpg");
    let answer = drawn(
        &unpainted_fixture(),
        &Ask {
            out: Some(out.clone()),
            ..range(0, 2000)
        },
    );
    let written = std::fs::read(&out).expect("the sheet is on disk");
    assert_eq!(written, answer.image().unwrap().bytes);
    let json = answer.to_json();
    assert_eq!(json["sheet"]["picture"]["encoding"], "jpeg");
    assert_eq!(json["sheet"]["picture"]["path"], out.display().to_string());
    assert_eq!(json["sheet"]["picture"]["bytes"], written.len());
}

#[test]
fn a_range_with_no_painted_frame_answers_with_no_picture_and_says_why() {
    // `[1010, 1030)` is the constructed document's unpainted state and nothing else.
    let answer = drawn(&unpainted_fixture(), &range(1010, 1030));
    assert!(answer.image().is_none());
    let json = answer.to_json();
    assert_eq!(json["sheet"]["picture"], Value::Null);
    assert_eq!(json["sheet"]["provenance"], json!([]));
    assert_eq!(json["sheet"]["skipped"].as_array().unwrap().len(), 1);
    assert_eq!(json["summary"]["review"], 1);
    // No tile 1, so nothing for a READER CHECK to quote (ADR-0128 §6).
    assert_eq!(json["sheet"]["reader_check"], Value::Null);
    let text = prose(&json);
    assert!(text.contains("no tile: no frame is painted in [1010, 1030)"));
    assert!(!text.contains("READER CHECK"), "{text}");
}

#[test]
fn frame_without_a_range_answers_as_it_always_has() {
    let json = frame(
        &unpainted_fixture(),
        &Ask {
            at: Some(0),
            ..Ask::default()
        },
    )
    .to_json();
    assert!(json.get("sheet").is_none(), "{json}");
    assert_eq!(json["frame"]["at"], 0);
}

#[test]
fn a_translucent_background_shows_the_same_tile_whatever_came_before_it() {
    // Two states, red then blue, over a half-transparent background. The second tile must
    // not show the first through it: each is its own frame over black, as `frame --at` is.
    let dir = tempdir(line!());
    let path = write_project(
        &dir,
        "translucent.montagent.json",
        &json!({
            "frame": {"width": 200, "height": 200}, "fps": 25, "background": "#00000080",
            "tracks": [{"name": "t", "layer": 0, "elements": [
                {"id": "red", "type": "rect", "start": 0, "end": 400, "x": 0, "y": 0,
                 "origin": "top-left", "width": 200, "height": 200, "fill": "#FF000080"},
                {"id": "blue", "type": "rect", "start": 400, "end": 800, "x": 0, "y": 0,
                 "origin": "top-left", "width": 200, "height": 200, "fill": "#0000FF80"},
            ]}],
        })
        .to_string(),
    );
    let answer = drawn(
        &path,
        &Ask {
            png: true,
            ..range(0, 800)
        },
    );
    let json = answer.to_json();
    let sheet = image::load_from_memory(&answer.image().unwrap().bytes)
        .unwrap()
        .to_rgba8();
    let tile_w = json["sheet"]["picture"]["served_tile_width"]
        .as_u64()
        .unwrap() as u32;
    let columns = json["sheet"]["picture"]["columns"].as_u64().unwrap() as u32;
    let second = if columns > 1 {
        (tile_w + tile_w / 2, 50)
    } else {
        (tile_w / 2, 250)
    };
    let [r, _, b, _] = sheet.get_pixel(second.0, second.1).0;
    assert_eq!(r, 0, "the red tile shows through the blue one");
    assert!(b > 0);
}

// ---------------------------------------------------------------------------
// Labels and the READER CHECK (#490)
// ---------------------------------------------------------------------------

/// The fixture's 18 labels at the target rung: `docs/research/tile-label/OUTPUT.txt`'s
/// instants, offsets and topmost-layer picks, which ADR-0098 §8 chose.
const FIXTURE_LABELS: [&str; 18] = [
    "1 0ms +0 +intro-title",
    "2 3040ms +22 +hook-05",
    "3 5320ms +4 +word-05",
    "4 10480ms +12 +sentence-05",
    "5 17480ms +8 +word-06",
    "6 22640ms +18 +sentence-06",
    "7 30640ms +37 +word-07",
    "8 35760ms +7 +sentence-07",
    "9 42800ms +37 +word-08-bridge",
    "10 47360ms +17 +sentence-08",
    "11 53880ms +24 +quiz-question",
    "12 56120ms +4 +count-5",
    "13 57120ms +4 +count-4",
    "14 58120ms +4 +count-3",
    "15 59120ms +4 +count-2",
    "16 60120ms +4 +count-1",
    "17 61120ms +4 +word-quiz",
    "18 64040ms +24 +hook-loop",
];

/// The fixed text of the READER CHECK around `label` (ADR-0114 §3).
fn reader_check(label: &str) -> String {
    format!(
        "Each tile's label is the line in the strip beneath it, outside the video frame; \
         text inside a tile is the video's own. Tile 1's label reads exactly `{label}`. The \
         provenance list below is the complete record of this range, and this sheet is a \
         picture of it. If the strip beneath tile 1 does not read exactly that, this sheet \
         is below what you can see, and `frame --at <instant>` shows any listed instant at \
         full scale. Reading the labels is necessary for seeing the pictures, not sufficient."
    )
}

fn labels(json: &Value) -> Vec<String> {
    json["sheet"]["provenance"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tile| tile["label"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn the_fixtures_labels_name_the_change_at_each_boundary_with_its_id_at_the_target_rung() {
    let (json, _) = fixture_sheet();
    assert_eq!(labels(json), FIXTURE_LABELS);
    let picture = &json["sheet"]["picture"];
    assert_eq!(picture["ids"], "carried");
    // 29 characters in 180 px of room: 10 px, over the 8 px floor.
    assert_eq!(
        (&picture["label_px"], &picture["label_floor_px"]),
        (&json!(10), &json!(8))
    );
    let text = prose(json);
    assert!(
        text.contains("labels      in the strip beneath each tile, outside the video frame, at 10 px (never under 8 px); ids carried"),
        "{text}"
    );
}

#[test]
fn a_perfect_answer_carries_the_reader_check_first_quoting_tile_ones_label_exactly() {
    let (json, _) = fixture_sheet();
    let check = &json["sheet"]["reader_check"];
    assert_eq!(check["tile"], 1);
    assert_eq!(check["label"], json["sheet"]["provenance"][0]["label"]);
    assert_eq!(check["label"], "1 0ms +0 +intro-title");
    assert_eq!(check["sentence"], reader_check("1 0ms +0 +intro-title"));

    // Directly after the header, before the provenance list, on one line.
    let text = prose(json);
    let mut lines = text.lines();
    lines.next().expect("the header");
    assert_eq!(lines.next(), Some(""));
    assert_eq!(
        lines.next(),
        Some(format!("READER CHECK  {}", reader_check("1 0ms +0 +intro-title")).as_str())
    );
    assert!(text.find("READER CHECK") < text.find("PROVENANCE"));
}

#[test]
fn nothing_a_range_answer_prints_names_a_model() {
    // ADR-0114 §4: the readers #422 measured are dated evidence in the ADRs, never words
    // in an answer. Montagent's own words only: the answer also echoes file paths, and
    // where the checkout happens to live is no part of what `frame` says.
    let (json, _) = fixture_sheet();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the workspace root")
        .display()
        .to_string();
    let everything = format!("{json}\n{}", prose(json))
        .replace(&root, "<root>")
        .replace(&root.replace('\\', "\\\\"), "<root>")
        .to_lowercase();
    assert!(
        everything.contains("<root>"),
        "the paths were found to strip"
    );
    for name in [
        "claude", "haiku", "sonnet", "opus", "fable", "gpt", "gemini", "model",
    ] {
        assert!(!everything.contains(name), "`{name}` in the answer");
    }
}

#[test]
fn the_fixture_at_the_refusal_width_elides_its_ids_on_every_tile() {
    // The fixture with six small rects on a bottom track, each landing inside a long state:
    // every one splits a state in three, so 18 states become 30, drawn at 141 px — where
    // the 29-character label no longer fits at 8 px, and every tile gives up its id.
    let mut document = common::document(&fixture_project());
    let pads: Vec<Value> = [1000, 12000, 20000, 25000, 38000, 45000]
        .iter()
        .enumerate()
        .map(|(i, start)| {
            json!({"id": format!("pad-{i}"), "type": "rect", "start": start, "end": start + 200,
                   "x": 0, "y": 0, "origin": "top-left", "width": 10, "height": 10,
                   "fill": "#FFFFFF"})
        })
        .collect();
    document["tracks"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name": "pad", "layer": -1000, "elements": pads}));
    let scratch = Scratch::beside_the_fixture("frame-range-elided", &document.to_string());
    let duration = document["duration"].as_i64().unwrap();
    let json = drawn(scratch.path(), &range(0, duration)).to_json();

    let picture = &json["sheet"]["picture"];
    assert_eq!(
        (&picture["rung"], &picture["served_tile_width"]),
        (&json!("degraded"), &json!(141))
    );
    assert_eq!(picture["ids"], "elided");
    let drawn = labels(&json);
    assert_eq!(drawn.len(), 30);
    for label in &drawn {
        assert_eq!(
            label.split(' ').count(),
            3,
            "index, instant and offset only: {label}"
        );
    }
    // The fixture's own tiles keep their cores; only the id went.
    assert_eq!(drawn[0], "1 0ms +0");
    assert_eq!(
        drawn[18], "19 42800ms +37",
        "the fixture's busiest tile, its id gone"
    );
    assert_eq!(json["sheet"]["reader_check"]["label"], "1 0ms +0");
    assert!(prose(&json).contains("ids elided on every tile"));
}

/// A 200×200 project at 25 fps whose boundaries exercise the id rule: `bg` spans the whole
/// document; at 1000 `a`, `b` and `aa` enter on one layer; at 2000 `b` departs alone; at
/// 3000 `lo` and `hi` enter on two layers while `a` departs.
fn changes_project(dir: &Path) -> PathBuf {
    let rect = |id: &str, x: i64, start: i64, end: i64| {
        json!({"id": id, "type": "rect", "start": start, "end": end, "x": x, "y": 0,
               "origin": "top-left", "width": 20, "height": 20, "fill": "#FFFFFF"})
    };
    write_project(
        dir,
        "changes.montagent.json",
        &json!({
            "frame": {"width": 200, "height": 200}, "fps": 25, "background": "#000000",
            "duration": 4000,
            "tracks": [
                {"name": "bg", "layer": 0, "elements": [{"id": "bg", "type": "rect",
                 "start": 0, "end": 4000, "x": 0, "y": 0, "origin": "top-left",
                 "width": 200, "height": 200, "fill": "#202020"}]},
                // Three on one layer, and the greatest id is neither first nor last in
                // array order: the tie-break reads ids, never positions.
                {"name": "a", "layer": 5, "elements": [rect("a", 0, 1000, 3000)]},
                {"name": "b", "layer": 5, "elements": [rect("b", 40, 1000, 2000)]},
                {"name": "aa", "layer": 5, "elements": [rect("aa", 80, 1000, 4000)]},
                {"name": "lo", "layer": 1, "elements": [rect("lo", 120, 3000, 4000)]},
                {"name": "hi", "layer": 9, "elements": [rect("hi", 160, 3000, 4000)]},
            ],
        })
        .to_string(),
    )
}

#[test]
fn the_id_is_the_highest_layer_change_signed_by_its_side_and_tie_broken_on_id() {
    let dir = tempdir(line!());
    let json = drawn(&changes_project(&dir), &range(0, 4000)).to_json();
    assert_eq!(
        labels(&json),
        [
            // Only `bg` entered, and it spans the document: nothing is left to name.
            "1 0ms +0 =",
            // `a`, `b` and `aa` tie on layer 5: the greatest id wins.
            "2 1000ms +0 +b",
            // Nothing entered: the departure is named, and its sign says it is gone.
            "3 2000ms +0 -b",
            // `a` departs from layer 5, but what entered is named first, highest layer on top.
            "4 3000ms +0 +hi",
        ]
    );
}

#[test]
fn a_range_opening_inside_a_state_names_the_change_that_state_opened_with() {
    // The id is a function of the document and the boundary, never of the range asked
    // for (ADR-0098 §8); the offset is from the run the provenance line records.
    let dir = tempdir(line!());
    let json = drawn(&changes_project(&dir), &range(1510, 2500)).to_json();
    assert_eq!(labels(&json), ["1 1520ms +10 +b", "2 2000ms +0 -b"]);
    assert_eq!(json["sheet"]["provenance"][0]["run"]["start"], 1510);
}

#[test]
fn whole_document_span_is_measured_against_the_document_never_the_range() {
    // ADR-0128 §2: "The requested range plays no part". `a`, `b` and `aa` each span all of
    // `[1000, 2000)` and none spans the document, so they stay candidates and `b` is named;
    // a span measured against the range would leave nothing and print `=`.
    let dir = tempdir(line!());
    let project = changes_project(&dir);
    let json = drawn(&project, &range(1000, 2000)).to_json();
    assert_eq!(labels(&json), ["1 1000ms +0 +b"]);
    // The other way round: `bg` spans the document and is never named, though a range
    // running past the document's end is wider than `bg`.
    let json = drawn(&project, &range(0, 5000)).to_json();
    assert_eq!(labels(&json)[0], "1 0ms +0 =");
}

#[test]
fn a_label_core_the_type_floor_cannot_hold_is_refused_on_the_type_floor() {
    // 30 states of a 1920×200 frame: 588 px tiles over 7 px strips. The tile width is
    // nowhere near its refusal; the label is.
    let dir = tempdir(line!());
    let elements: Vec<Value> = (0..30)
        .map(|i| {
            json!({"id": format!("c{i}"), "type": "rect", "start": i * 200, "end": (i + 1) * 200,
                   "x": 0, "y": 0, "origin": "top-left", "width": 1920, "height": 200,
                   "fill": "#404040"})
        })
        .collect();
    let path = write_project(
        &dir,
        "letterbox.montagent.json",
        &json!({"frame": {"width": 1920, "height": 200}, "fps": 25, "background": "#000000",
                "tracks": [{"name": "c", "layer": 0, "elements": elements}]})
        .to_string(),
    );
    let answer = frame(&path, &range(0, 6000));
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
    let json = answer.to_json();
    let refusal = &json["findings"][0];
    assert_eq!(refusal["code"], "E-SHEET-OVERFLOW");
    assert_eq!(refusal["fields"]["limit"], "type-floor");
    assert_eq!(refusal["fields"]["limit_px"], 8);
    assert_eq!(refusal["fields"]["fits"], 22);
    let text = prose(&json);
    assert!(text.contains("type-floor limit of 8 px served"), "{text}");
    for sub_range in refusal["fields"]["sub_ranges"].as_array().unwrap() {
        let (from, to) = (
            sub_range["from"].as_i64().unwrap(),
            sub_range["to"].as_i64().unwrap(),
        );
        let sheet = drawn(&path, &range(from, to)).to_json();
        assert!(sheet["sheet"]["picture"]["label_px"].as_i64().unwrap() >= 8);
    }
}

// ---------------------------------------------------------------------------
// Keyframe tiles: `--keyframes` (#491, ADR-0106, ADR-0129)
// ---------------------------------------------------------------------------

fn keyframed(from: i64, to: i64) -> Ask {
    Ask {
        keyframes: true,
        ..range(from, to)
    }
}

/// Each provenance line as `(index, class, why, instant, label, keyframes)`.
fn tiles(json: &Value) -> Vec<(i64, String, String, i64, String, Vec<String>)> {
    json["sheet"]["provenance"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tile| {
            (
                tile["index"].as_i64().unwrap(),
                tile["class"].as_str().unwrap().to_string(),
                tile["why"].as_str().unwrap().to_string(),
                tile["instant_ms"].as_i64().unwrap(),
                tile["label"].as_str().unwrap().to_string(),
                serde_json::from_value(tile["keyframes"].clone()).unwrap(),
            )
        })
        .collect()
}

fn tile(
    index: i64,
    class: &str,
    why: &str,
    instant: i64,
    label: &str,
    keyframes: &[&str],
) -> (i64, String, String, i64, String, Vec<String>) {
    (
        index,
        class.into(),
        why.into(),
        instant,
        label.into(),
        keyframes.iter().map(|point| point.to_string()).collect(),
    )
}

#[test]
fn keyframe_tiles_sample_the_first_painted_frame_and_share_it_and_a_run_tile_keeps_its_class() {
    // ADR-0106 D10–11 on the constructed coincidence document.
    let json = drawn(&keyframe_coincidence_fixture(), &keyframed(0, 2000)).to_json();
    assert_eq!(
        tiles(&json),
        [
            tile(1, "run", "boundary", 0, "1 0ms +0 +a", &[]),
            // b.x@1020 samples at 1040, the run tile's own frame: it joins that line, and
            // the tile stays a run tile with its boundary and its label.
            tile(2, "run", "boundary", 1040, "2 1040ms +37 +b", &["b.x@1020"]),
            // Two change points on two elements, one painted frame: one tile.
            tile(
                3,
                "keyframe",
                "keyframe",
                1440,
                "3 K 1440ms +30",
                &["b.x@1410", "c.y@1425"]
            ),
            tile(
                4,
                "keyframe",
                "keyframe",
                1600,
                "4 K 1600ms +0",
                &["b.x@1600"]
            ),
        ]
    );
    let sheet = &json["sheet"];
    assert_eq!(
        sheet["classes"],
        json!({"run": 2, "keyframe": 2, "infill": 0})
    );
    assert_eq!(sheet["keyframes"]["tiled"], 4);
    assert_eq!(sheet["keyframes"]["untiled"], 0);
    assert_eq!(sheet["keyframes"]["untiled_points"], json!([]));
    // A keyframe tile's run is the visual state it samples inside.
    assert_eq!(
        sheet["provenance"][2]["run"],
        json!({"start": 1003, "end": 2000})
    );
    assert_eq!(sheet["coverage"]["tiled"], 2, "coverage counts states");

    let text = prose(&json);
    for words in [
        "keyframe  keyframe  at 1440 ms",
        "keyframes b.x@1410, c.y@1425",
        "classes     2 run, 2 keyframe, 0 infill",
        "keyframes   4 tiled, 0 untiled",
    ] {
        assert!(text.contains(words), "`{words}` missing: {text}");
    }
}

#[test]
fn without_the_flag_the_census_is_still_asserted_and_interior_points_are_untiled() {
    let json = drawn(&keyframe_coincidence_fixture(), &range(0, 2000)).to_json();
    let sheet = &json["sheet"];
    assert_eq!(sheet["classes"]["keyframe"], 0);
    assert_eq!(sheet["provenance"].as_array().unwrap().len(), 2);
    // The run tile's change point is tiled without the flag (ADR-0106 D8's split is a
    // fact about the sheet).
    assert_eq!(sheet["provenance"][1]["keyframes"], json!(["b.x@1020"]));
    assert_eq!(sheet["keyframes"]["tiled"], 1);
    assert_eq!(sheet["keyframes"]["untiled"], 3);
    let reasons: Vec<(String, String)> = sheet["keyframes"]["untiled_points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|point| {
            (
                point["point"].as_str().unwrap().to_string(),
                point["reason"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(
        reasons,
        [
            ("b.x@1410".to_string(), "not-requested".to_string()),
            ("c.y@1425".to_string(), "not-requested".to_string()),
            ("b.x@1600".to_string(), "not-requested".to_string()),
        ]
    );
    assert_eq!(
        json["findings"],
        json!([]),
        "an untiled point is not a finding"
    );
    let text = prose(&json);
    assert!(text.contains("keyframes   1 tiled, 3 untiled"), "{text}");
    assert!(
        text.contains("b.x@1410 at 1440 ms, state 1003..2000 \u{2014} not-requested"),
        "{text}"
    );
}

#[test]
fn a_change_point_with_no_frame_left_in_its_run_is_tiled_by_the_next_run_only_where_visible() {
    // ADR-0106 D12 on the constructed cross-boundary document.
    let path = keyframe_cross_boundary_fixture();
    let json = drawn(&path, &keyframed(0, 2000)).to_json();
    assert_eq!(
        tiles(&json),
        [
            tile(1, "run", "boundary", 0, "1 0ms +0 +a", &[]),
            tile(2, "run", "boundary", 520, "2 520ms +20 +p", &[]),
            // p.y@1015 has no painted frame left in 500..1030; 1040 is the next run's tile,
            // and p is on screen there.
            tile(3, "run", "boundary", 1040, "3 1040ms +10 +c", &["p.y@1015"]),
        ]
    );
    let sheet = &json["sheet"];
    assert_eq!(sheet["classes"]["keyframe"], 0, "the rule adds no tile");
    assert_eq!(sheet["keyframes"]["tiled"], 1);
    assert_eq!(sheet["keyframes"]["untiled"], 1);
    // a.x@1020 would sample at 1040 too, where `a` has gone.
    assert_eq!(
        sheet["keyframes"]["untiled_points"],
        json!([{
            "point": "a.x@1020", "element": "a", "property": "x", "at": 1020,
            "sample_ms": 1040, "run": {"start": 500, "end": 1030}, "reason": "no-grid-frame",
        }])
    );
    // Disclosure only, never a finding: neither in the report nor in `skipped[]`.
    assert_eq!(json["findings"], json!([]));
    assert_eq!(sheet["skipped"], json!([]));

    // With the next run outside the range, the visible one is untiled too.
    let json = drawn(&path, &keyframed(0, 1030)).to_json();
    let sheet = &json["sheet"];
    assert_eq!(sheet["provenance"].as_array().unwrap().len(), 2);
    assert_eq!(sheet["keyframes"]["tiled"], 0);
    assert_eq!(sheet["keyframes"]["untiled"], 2);
    let points: Vec<&str> = sheet["keyframes"]["untiled_points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|point| {
            assert_eq!(point["reason"], "no-grid-frame");
            point["point"].as_str().unwrap()
        })
        .collect();
    assert_eq!(points, ["p.y@1015", "a.x@1020"]);
    assert_eq!(json["findings"], json!([]));
}

#[test]
fn across_an_unpainted_state_the_next_painted_run_decides_whether_a_point_is_tiled() {
    // At 25 fps: `a` leaves at 1010 and `c` enters at 1030, so `1010..1030` is a state no
    // frame paints. `e` and `f` each change at 1003/1005, with no frame left in `500..1010`,
    // and both sample at 1040: `f` is on screen there and `e` left at 1030.
    let dir = tempdir(line!());
    let rect = |id: &str, start: i64, end: i64| {
        json!({"id": id, "type": "rect", "start": start, "end": end, "x": 0, "y": 0,
               "origin": "top-left", "width": 200, "height": 200, "fill": "#808080"})
    };
    let mut e = rect("e", 500, 1030);
    e["y"] = json!([{"t": 500, "v": 0}, {"t": 1005, "v": 400, "ease": "linear"}]);
    let mut f = rect("f", 500, 2000);
    f["x"] = json!([{"t": 500, "v": 0}, {"t": 1003, "v": 400, "ease": "linear"}]);
    let path = write_project(
        &dir,
        "middle.montagent.json",
        &json!({
            "frame": {"width": 1080, "height": 1920}, "fps": 25, "background": "#000000",
            "tracks": [
                {"name": "a", "layer": 0, "elements": [rect("a", 0, 1010)]},
                {"name": "e", "layer": 1, "elements": [e]},
                {"name": "f", "layer": 2, "elements": [f]},
                {"name": "c", "layer": 3, "elements": [rect("c", 1030, 2000)]},
            ],
        })
        .to_string(),
    );

    let json = drawn(&path, &keyframed(0, 2000)).to_json();
    let sheet = &json["sheet"];
    assert_eq!(
        sheet["skipped"][0]["run"],
        json!({"start": 1010, "end": 1030})
    );
    let last = &sheet["provenance"][2];
    assert_eq!(
        (last["instant_ms"].as_i64(), last["class"].as_str()),
        (Some(1040), Some("run"))
    );
    assert_eq!(last["keyframes"], json!(["f.x@1003"]));
    assert_eq!(sheet["keyframes"]["tiled"], 1);
    assert_eq!(sheet["keyframes"]["untiled_points"][0]["point"], "e.y@1005");
    assert_eq!(
        sheet["keyframes"]["untiled_points"][0]["reason"],
        "no-grid-frame"
    );
    // The one finding is the unpainted state's, and no change point adds one.
    let codes: Vec<&str> = json["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["code"].as_str().unwrap())
        .collect();
    assert_eq!(codes, ["N-QUANTIZATION"]);
}

#[test]
fn the_fixture_declares_no_interior_change_point_with_or_without_the_flag() {
    let (json, _) = fixture_sheet();
    assert_eq!(
        json["sheet"]["keyframes"],
        json!({"tiled": 0, "untiled": 0, "untiled_points": []})
    );
    assert!(prose(json).contains("keyframes   0 tiled, 0 untiled"));

    let project = fixture_project();
    let duration = common::document(&project)["duration"].as_i64().unwrap();
    let flagged = drawn(&project, &keyframed(0, duration)).to_json();
    assert_eq!(flagged["sheet"]["keyframes"], json["sheet"]["keyframes"]);
    assert_eq!(flagged["sheet"]["provenance"].as_array().unwrap().len(), 18);
}

/// `states_project`'s 18 cards, the first `keyed` of which move once at 100 ms into their
/// own 200 ms: an interior change point each, sampled at 120 ms in.
fn keyed_project(dir: &Path, keyed: usize) -> PathBuf {
    let path = states_project(dir, 18);
    let mut document = common::document(&path);
    for (i, card) in document["tracks"][0]["elements"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
        .take(keyed)
    {
        let start = i as i64 * 200;
        card["x"] = json!([{"t": start, "v": 0}, {"t": start + 100, "v": 40, "ease": "linear"}]);
    }
    write_project(dir, "keyed.montagent.json", &document.to_string())
}

#[test]
fn a_thirteenth_keyframe_tile_on_eighteen_states_is_refused_naming_the_flag() {
    let dir = tempdir(line!());

    // Twelve fit: 30 tiles, the most the refusal width admits.
    let twelve = keyed_project(&dir, 12);
    let json = drawn(&twelve, &keyframed(0, 3600)).to_json();
    assert_eq!(json["sheet"]["classes"]["keyframe"], 12);
    assert_eq!(json["sheet"]["picture"]["rung"], "degraded");

    let thirteen = keyed_project(&dir, 13);
    let answer = frame(&thirteen, &keyframed(0, 3600));
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
    let json = answer.to_json();
    let refusal = &json["findings"][0];
    assert_eq!(refusal["code"], "E-SHEET-OVERFLOW", "{json}");
    let fields = &refusal["fields"];
    assert_eq!(fields["states"], 18);
    assert_eq!(fields["keyframe_tiles"], 13);
    assert_eq!(fields["fits"], 30);
    assert_eq!(fields["fits_without_keyframes"], true);
    assert_eq!(fields["keyframe_tiles_admitted"], 12);
    let text = prose(&json);
    for words in [
        "18 visual states",
        "13 keyframe tiles",
        "Without `--keyframes` the range fits",
        "12 keyframe tiles",
    ] {
        assert!(text.contains(words), "`{words}` missing: {text}");
    }

    // Keyframe tiles are never dropped to fit: every sub-range, asked with the flag, draws
    // every keyframe tile counted for it.
    let ranges = fields["sub_ranges"].as_array().unwrap();
    assert!(ranges.len() >= 2, "{fields}");
    let mut keyframe_tiles = 0;
    for sub_range in ranges {
        let (from, to) = (
            sub_range["from"].as_i64().unwrap(),
            sub_range["to"].as_i64().unwrap(),
        );
        let sheet = drawn(&thirteen, &keyframed(from, to)).to_json()["sheet"].clone();
        keyframe_tiles += sheet["classes"]["keyframe"].as_u64().unwrap();
        assert_eq!(sheet["keyframes"]["untiled"], 0, "[{from}, {to})");
    }
    assert_eq!(keyframe_tiles, 13);

    // Without the flag the range draws, and the census still counts all thirteen.
    let json = drawn(&thirteen, &range(0, 3600)).to_json();
    assert_eq!(json["sheet"]["provenance"].as_array().unwrap().len(), 18);
    assert_eq!(json["sheet"]["keyframes"]["untiled"], 13);
}

#[test]
fn an_overflow_without_the_flag_carries_no_keyframe_field() {
    let dir = tempdir(line!());
    let path = states_project(&dir, 31);
    let json = frame(&path, &range(0, 31 * 200)).to_json();
    let fields = &json["findings"][0]["fields"];
    assert_eq!(json["findings"][0]["code"], "E-SHEET-OVERFLOW");
    for key in [
        "fits_without_keyframes",
        "keyframe_tiles_admitted",
        "keyframe_tiles",
    ] {
        assert!(
            fields.get(key).is_none(),
            "`{key}` without the flag: {fields}"
        );
    }
    assert!(!prose(&json).contains("--keyframes"));
}

#[test]
fn a_keyframe_tile_is_marked_by_an_inverted_strip() {
    let json_and_bytes = {
        let answer = drawn(
            &keyframe_coincidence_fixture(),
            &Ask {
                png: true,
                ..keyframed(0, 2000)
            },
        );
        (answer.to_json(), answer.image().unwrap().bytes.clone())
    };
    let (json, bytes) = json_and_bytes;
    let picture = &json["sheet"]["picture"];
    let number = |key: &str| picture[key].as_i64().unwrap();
    let (columns, tile_w, tile_h) = (
        number("columns"),
        number("served_tile_width"),
        number("served_tile_height"),
    );
    let cell_h = number("height") / number("rows");
    let drawn = image::load_from_memory(&bytes).unwrap().to_rgba8();
    // The strip's corner, past the inset and clear of any glyph: the ground it was drawn on.
    let ground = |index: i64| {
        let (x, y) = (
            (index % columns) * tile_w,
            (index / columns) * cell_h + tile_h,
        );
        drawn
            .get_pixel((x + tile_w - 1) as u32, (y + cell_h - tile_h - 1) as u32)
            .0
    };
    assert_eq!(
        ground(0),
        [0x1A, 0x1A, 0x1A, 0xFF],
        "a run tile is unmarked"
    );
    assert_eq!(
        ground(1),
        [0x1A, 0x1A, 0x1A, 0xFF],
        "and stays so with a change point"
    );
    assert_eq!(
        ground(2),
        [0xE6, 0xE6, 0xE6, 0xFF],
        "a keyframe tile's strip is inverted"
    );
    assert_eq!(ground(3), [0xE6, 0xE6, 0xE6, 0xFF]);
}

#[test]
fn keyframes_without_a_range_is_refused() {
    let reason = refused(&Ask {
        at: Some(500),
        keyframes: true,
        ..Ask::default()
    });
    assert!(reason.contains("`--keyframes`"), "{reason}");
    assert!(reason.contains("`--from`/`--to`"), "{reason}");
}

// ---------------------------------------------------------------------------
// Infill tiles: `--infill-ceiling` (#492, ADR-0106, ADR-0130)
// ---------------------------------------------------------------------------

fn infilled(from: i64, to: i64, ceiling: &str) -> Ask {
    Ask {
        infill_ceiling: Some(ceiling.into()),
        ..range(from, to)
    }
}

/// Every span a sheet leaves, in painted time: between consecutive tiles of any class, and
/// from the last to `to`, which is on the 25 fps grid in every test here.
fn spans(json: &Value, to: i64) -> Vec<i64> {
    let mut instants: Vec<i64> = json["sheet"]["provenance"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tile| tile["instant_ms"].as_i64().unwrap())
        .collect();
    instants.push(to);
    instants.windows(2).map(|pair| pair[1] - pair[0]).collect()
}

#[test]
fn infill_tiles_close_every_span_to_the_ceiling_marked_and_listed_with_why_infill() {
    let json = drawn(&long_state_fixture(), &infilled(0, 9000, "2000")).to_json();
    assert_eq!(
        tiles(&json),
        [
            tile(1, "run", "boundary", 0, "1 0ms +0 +a", &[]),
            tile(2, "run", "boundary", 1000, "2 1000ms +0 +b", &[]),
            // The latest painted frame no more than 2000 ms after the tile before it, with
            // the offset from its state's start and no identifying field.
            tile(3, "infill", "infill", 3000, "3 I 3000ms +2000", &[]),
            tile(4, "infill", "infill", 5000, "4 I 5000ms +4000", &[]),
            tile(5, "infill", "infill", 7000, "5 I 7000ms +6000", &[]),
        ]
    );
    assert!(spans(&json, 9000).iter().all(|&span| span <= 2000));
    let sheet = &json["sheet"];
    assert_eq!(
        sheet["classes"],
        json!({"run": 2, "keyframe": 0, "infill": 3})
    );
    assert_eq!(
        sheet["infill"],
        json!({"requested_ms": 2000, "achieved_ms": 2000})
    );
    assert_eq!(sheet["skipped"], json!([]));
    assert_eq!(
        sheet["provenance"][2]["run"],
        json!({"start": 1000, "end": 9000})
    );
    assert_eq!(sheet["provenance"][2]["present"], json!(["bg", "b"]));
    assert_eq!(sheet["coverage"]["tiled"], 2, "coverage counts states");
    // b.x@4010 samples at 4040, where no tile of this sheet sits.
    assert_eq!(sheet["keyframes"]["untiled"], 1);

    let text = prose(&json);
    for words in [
        "infill  infill  at 3000 ms",
        "classes     2 run, 0 keyframe, 3 infill",
        "infill      requested 2000 ms, achieved 2000 ms",
    ] {
        assert!(text.contains(words), "`{words}` missing: {text}");
    }
}

#[test]
fn an_infill_tile_is_marked_by_an_inverted_strip() {
    let answer = drawn(
        &long_state_fixture(),
        &Ask {
            png: true,
            ..infilled(0, 9000, "2000")
        },
    );
    let json = answer.to_json();
    let picture = &json["sheet"]["picture"];
    let number = |key: &str| picture[key].as_i64().unwrap();
    let (columns, tile_w, tile_h) = (
        number("columns"),
        number("served_tile_width"),
        number("served_tile_height"),
    );
    let cell_h = number("height") / number("rows");
    let drawn = image::load_from_memory(&answer.image().unwrap().bytes)
        .unwrap()
        .to_rgba8();
    let ground = |index: i64| {
        let (x, y) = (
            (index % columns) * tile_w,
            (index / columns) * cell_h + tile_h,
        );
        drawn
            .get_pixel((x + tile_w - 1) as u32, (y + cell_h - tile_h - 1) as u32)
            .0
    };
    assert_eq!(
        ground(1),
        [0x1A, 0x1A, 0x1A, 0xFF],
        "a run tile is unmarked"
    );
    for index in 2..5 {
        assert_eq!(
            ground(index),
            [0xE6, 0xE6, 0xE6, 0xFF],
            "tile {}",
            index + 1
        );
    }
}

#[test]
fn a_ceiling_that_only_partly_fits_is_honoured_uniformly_and_the_rest_is_infill_evicted() {
    // 200 ms would add 43 tiles; the target rung has 16 slots beside the two run tiles.
    // 520 ms is the smallest ceiling 16 tiles honour on this grid, in every span at once.
    let json = drawn(&long_state_fixture(), &infilled(0, 9000, "200")).to_json();
    let sheet = &json["sheet"];
    assert_eq!(
        sheet["infill"],
        json!({"requested_ms": 200, "achieved_ms": 520})
    );
    assert_eq!(sheet["classes"]["infill"], 16);
    assert_eq!(sheet["picture"]["rung"], "target");
    let spans = spans(&json, 9000);
    assert_eq!(spans.iter().max(), Some(&520), "{spans:?}");
    // What 200 ms would have added and the sheet does not draw, by the run it is in: 3600,
    // 6200 and 8800 are on both grids, and drawn.
    assert_eq!(
        sheet["skipped"],
        json!([
            {"run": {"start": 0, "end": 1000}, "reason": "infill-evicted",
             "present": ["bg", "a"], "evicted": 4},
            {"run": {"start": 1000, "end": 9000}, "reason": "infill-evicted",
             "present": ["bg", "b"], "evicted": 36},
        ])
    );
    // An evicted infill tile is the instrument's choice, never a finding (ADR-0105 §4).
    assert_eq!(json["findings"], json!([]));
    assert_eq!(sheet["coverage"]["skipped"], 0, "no state went untiled");

    let text = prose(&json);
    for words in [
        "infill      requested 200 ms, achieved 520 ms",
        "0..1000  infill-evicted, 4 tiles",
        "1000..9000  infill-evicted, 36 tiles",
    ] {
        assert!(text.contains(words), "`{words}` missing: {text}");
    }
}

#[test]
fn infill_with_keyframes_still_bounds_the_span_between_tiles_of_any_class() {
    let json = drawn(
        &long_state_fixture(),
        &Ask {
            keyframes: true,
            ..infilled(0, 9000, "3000")
        },
    )
    .to_json();
    assert_eq!(
        tiles(&json),
        [
            tile(1, "run", "boundary", 0, "1 0ms +0 +a", &[]),
            tile(2, "run", "boundary", 1000, "2 1000ms +0 +b", &[]),
            tile(3, "infill", "infill", 4000, "3 I 4000ms +3000", &[]),
            tile(
                4,
                "keyframe",
                "keyframe",
                4040,
                "4 K 4040ms +30",
                &["b.x@4010"]
            ),
            // From the keyframe tile, not from the infill tile before it.
            tile(5, "infill", "infill", 7040, "5 I 7040ms +6040", &[]),
        ]
    );
    assert!(spans(&json, 9000).iter().all(|&span| span <= 3000));
    assert_eq!(
        json["sheet"]["classes"],
        json!({"run": 2, "keyframe": 1, "infill": 2})
    );
}

#[test]
fn an_infill_tile_at_a_change_points_sample_frame_tiles_it_and_keeps_its_class() {
    // 3040 ms after the run tile at 1000 is 4040, where b.x@4010 first paints.
    let json = drawn(&long_state_fixture(), &infilled(0, 9000, "3040")).to_json();
    assert_eq!(
        tiles(&json)[2],
        tile(
            3,
            "infill",
            "infill",
            4040,
            "3 I 4040ms +3040",
            &["b.x@4010"]
        )
    );
    let census = &json["sheet"]["keyframes"];
    assert_eq!(
        (&census["tiled"], &census["untiled"]),
        (&json!(1), &json!(0))
    );
}

#[test]
fn a_ceiling_longer_than_the_range_is_legal_and_adds_no_tile() {
    let json = drawn(&long_state_fixture(), &infilled(0, 9000, "60000")).to_json();
    let sheet = &json["sheet"];
    assert_eq!(sheet["classes"]["infill"], 0);
    assert_eq!(
        sheet["infill"],
        json!({"requested_ms": 60000, "achieved_ms": 60000})
    );
    assert_eq!(sheet["skipped"], json!([]));
    // The sheet is the one drawn without the flag.
    let without = drawn(&long_state_fixture(), &range(0, 9000)).to_json();
    assert_eq!(sheet["picture"], without["sheet"]["picture"]);
    assert_eq!(sheet["provenance"], without["sheet"]["provenance"]);
    assert!(without["sheet"].get("infill").is_none(), "only when asked");
    assert!(!prose(&without).contains("infill      requested"));
}

#[test]
fn the_whole_fixture_gains_no_infill_tile_at_any_ceiling() {
    // Eighteen states fill the target rung's eighteen slots (ADR-0106 §5–6).
    let project = fixture_project();
    let duration = common::document(&project)["duration"].as_i64().unwrap();
    let (whole, _) = fixture_sheet();
    for (ceiling, achieved) in [("40", Value::Null), ("1000000", json!(1_000_000))] {
        let json = drawn(&project, &infilled(0, duration, ceiling)).to_json();
        let sheet = &json["sheet"];
        assert_eq!(sheet["classes"]["infill"], 0, "{ceiling}");
        assert_eq!(labels(&json), FIXTURE_LABELS, "{ceiling}");
        assert_eq!(sheet["picture"]["served_tile_width"], 184);
        assert_eq!(sheet["infill"]["achieved_ms"], achieved, "{ceiling}");
        assert_eq!(sheet["provenance"], whole["sheet"]["provenance"]);
        let evicted = sheet["skipped"].as_array().unwrap();
        assert_eq!(evicted.is_empty(), achieved.is_number(), "{ceiling}");
        assert!(
            evicted
                .iter()
                .all(|entry| entry["reason"] == "infill-evicted")
        );
        if achieved.is_null() {
            let text = prose(&json);
            assert!(
                text.contains("infill      requested 40 ms, achieved: none"),
                "{text}"
            );
        }
    }
}

#[test]
fn infill_ceiling_without_a_range_is_refused() {
    let reason = refused(&Ask {
        at: Some(500),
        infill_ceiling: Some("1000".into()),
        ..Ask::default()
    });
    assert!(reason.contains("`--infill-ceiling`"), "{reason}");
    assert!(reason.contains("`--from`/`--to`"), "{reason}");
}

#[test]
fn a_ceiling_that_is_not_a_positive_whole_number_of_milliseconds_is_refused_not_clamped() {
    for (ceiling, words) in [
        ("0", "more than zero"),
        ("-40", "more than zero"),
        ("40.5", "whole number of milliseconds"),
        ("1e400", "whole number of milliseconds"),
        ("soon", "whole number of milliseconds"),
    ] {
        let reason = refused(&infilled(0, 2000, ceiling));
        assert!(
            reason.contains(&format!("`--infill-ceiling {ceiling}`")) && reason.contains(words),
            "{ceiling}: {reason}"
        );
    }
    // A whole number written with a fraction is still that number.
    drawn(&unpainted_fixture(), &infilled(0, 2000, "40.0"));
}

#[test]
fn a_ceiling_below_one_frame_period_is_refused_and_one_period_is_legal() {
    // 25 fps paints every 40 ms.
    let reason = refused(&infilled(0, 2000, "39"));
    assert!(reason.contains("`--infill-ceiling 39`"), "{reason}");
    assert!(reason.contains("one frame period"), "{reason}");
    assert!(reason.contains("25 fps"), "{reason}");
    drawn(&unpainted_fixture(), &infilled(0, 2000, "40"));

    // 30 fps paints every 33.3 ms: 33 is under it, and 34 the least whole ceiling over.
    let dir = tempdir(line!());
    let mut document = common::document(&long_state_fixture());
    document["fps"] = json!(30);
    let thirty = write_project(&dir, "thirty.montagent.json", &document.to_string());
    let answer = frame(&thirty, &infilled(0, 9000, "33"));
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
    assert_eq!(answer.to_json()["findings"][0]["code"], "E-INVOCATION");
    let json = drawn(&thirty, &infilled(0, 500, "34")).to_json();
    assert_eq!(json["sheet"]["infill"]["achieved_ms"], 34);
}
