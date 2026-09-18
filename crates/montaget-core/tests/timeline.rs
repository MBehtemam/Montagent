//! `timeline` — the human's wide view (#195).
//!
//! The acceptance criterion is one sentence: *"a human can see the shape of a 60-element
//! project without reading 155 lines of JSON."* So the tests that matter are about what the
//! view **collapses to** and what it **states explicitly**, not about byte-exact prose.

use montaget_core::Wire;
use montaget_core::report::ExitCode;
use montaget_core::verbs::timeline;
use montaget_core::wire;
use std::path::PathBuf;

mod common;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json")
}

fn prose(path: &std::path::Path) -> String {
    wire::render_timeline(&timeline::timeline(path), Wire::Text { verbose: false })
}

fn json(path: &std::path::Path) -> serde_json::Value {
    timeline::timeline(path).to_json()
}

#[test]
fn the_fixture_s_sixty_elements_collapse_to_eight_group_blocks() {
    // The round-1 finding that survived its own contamination: "`group` is the beat the
    // author thinks in. 60 elements collapse to 8 blocks", ranked first of four candidate
    // renderings by readers doing real tasks against this fixture.
    let json = json(&fixture());
    let timeline = &json["timeline"];

    assert_eq!(timeline["counts"]["elements"], 60);
    assert_eq!(timeline["counts"]["tracks"], 14);
    assert_eq!(timeline["counts"]["groups"], 8);

    let groups: Vec<&str> = timeline["groups"]
        .as_array()
        .expect("a groups array")
        .iter()
        .map(|g| g["group"].as_str().unwrap_or("(no group)"))
        .collect();
    // Ordered by the instant each group first puts something on screen, then by the instant
    // it lets go — never by array order, which ADR-0060 settled carries no meaning.
    assert_eq!(
        groups,
        [
            "intro",
            "item-05",
            "header",
            "item-06",
            "item-07",
            "item-08",
            "quiz",
            "loop-tail",
        ]
    );
}

#[test]
fn a_group_block_carries_the_earliest_start_and_the_latest_end_of_its_elements() {
    let json = json(&fixture());
    let item_05 = json["timeline"]["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["group"] == "item-05")
        .expect("the item-05 block");

    assert_eq!(item_05["start"], 0);
    assert_eq!(item_05["end"], 17472);
    assert_eq!(item_05["duration_ms"], 17472);
    assert_eq!(item_05["elements"].as_array().unwrap().len(), 11);
}

#[test]
fn every_instant_the_view_prints_is_absolute() {
    // ADR-0004's requirement, as ADR-0031 reads it: "absolute times stated explicitly, not
    // left to be inferred from position or array order". The prototype's group blocks
    // printed offsets from each group's own start — item-06's first element as `+0.000` —
    // which is exactly the inference the requirement forbids.
    let prose = prose(&fixture());

    assert!(prose.contains("17472..30603"), "{prose}");
    assert!(
        !prose.contains("+0.000"),
        "a group-relative offset is not an absolute instant:\n{prose}"
    );
}

#[test]
fn the_tracks_block_names_every_track_and_the_place_in_the_stack_it_supplies() {
    let json = json(&fixture());
    let tracks = json["timeline"]["tracks"].as_array().expect("tracks");

    assert_eq!(tracks.len(), 14);
    // Ordered by layer: stacking is what a track supplies, and back-to-front is the order a
    // reader asking "what is in front of what" needs (ADR-0004).
    assert_eq!(tracks[0]["name"], "narration");
    assert_eq!(tracks[0]["layer"], 0);
    assert_eq!(tracks[0]["elements"], 20);
    assert_eq!(tracks[13]["name"], "chip-text");
    assert_eq!(tracks[13]["layer"], 34);
}

#[test]
fn the_header_of_the_view_states_the_frame_the_rate_and_the_length() {
    let prose = prose(&fixture());

    assert!(prose.contains("1080×1920"), "{prose}");
    assert!(prose.contains("25 fps"), "{prose}");
    assert!(prose.contains("65216 ms"), "{prose}");
    assert!(prose.contains("out/en-halloween-decorating.mp4"), "{prose}");
    assert!(
        prose.contains("60 elements, 14 tracks, 8 groups"),
        "{prose}"
    );
}

#[test]
fn an_element_says_what_it_is_without_the_reader_opening_the_file() {
    // The detail column is the difference between a list of ids and a view: `photo-06` and
    // `sentence-06` tell a reader nothing about what is on screen at 17472.
    let prose = prose(&fixture());

    assert!(prose.contains("images/06.png"), "{prose}");
    assert!(
        prose.contains("I hang cobwebs over the door."),
        "a text element's own words are what it is:\n{prose}"
    );
    assert!(
        prose.contains("984×169 #1E344C"),
        "a shape is its rect and its paint:\n{prose}"
    );
    assert!(prose.contains("audio/intro-2.mp3"), "{prose}");
}

#[test]
fn a_line_break_inside_a_run_prints_as_a_mark_rather_than_breaking_the_row() {
    // ADR-0008: a line break is a `\n` inside a run's text. A view that printed it raw
    // would put one element across two rows and silently misalign every column after it.
    let prose = prose(&fixture());

    assert!(prose.contains("What is this called⏎in English?"), "{prose}");
}

#[test]
fn an_element_carrying_no_group_is_listed_under_a_block_of_its_own() {
    let dir = common::tempdir(line!());
    let project = common::write_project(
        &dir,
        "p.montaget.json",
        &common::canonical(
            r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"photo","layer":10,
                "elements":[
                  {"id":"grouped","type":"rect","group":"beat","start":0,"end":100,"width":10,"height":10,"fill":"#FFFFFF"},
                  {"id":"loose","type":"rect","start":0,"end":100,"width":10,"height":10,"fill":"#000000"}
                ]}]}"##,
        ),
    );

    let json = json(&project);
    let groups = json["timeline"]["groups"].as_array().unwrap();

    // `group` is optional (CONTEXT.md), so the view must not pretend every element has one
    // — and the ungrouped block goes last, because it is a residue rather than a beat.
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0]["group"], "beat");
    assert!(groups[1]["group"].is_null());
    assert_eq!(groups[1]["elements"][0]["id"], "loose");
    assert_eq!(json["timeline"]["counts"]["groups"], 1, "one named group");

    assert!(prose(&project).contains("(no group)"));
}

#[test]
fn a_layer_anchor_prints_as_the_integer_it_resolves_to() {
    // ADR-0019 resolves an anchor in one hop, and `crate::stack` is the one place that
    // happens. A view that echoed `{"below": "card"}` would hand the reader the arithmetic
    // it exists to have already done.
    let dir = common::tempdir(line!());
    let project = common::write_project(
        &dir,
        "p.montaget.json",
        &common::canonical(
            r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"card","layer":20,
                "elements":[
                  {"id":"card","type":"rect","start":0,"end":100,"width":10,"height":10,"fill":"#FFFFFF"},
                  {"id":"shadow","type":"rect","layer":{"below":"card"},"start":0,"end":100,"width":10,"height":10,"fill":"#000000"}
                ]}]}"##,
        ),
    );

    let json = json(&project);
    let elements = &json["timeline"]["groups"][0]["elements"];

    let shadow = elements
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["id"] == "shadow")
        .expect("the anchored element");
    assert_eq!(shadow["layer"], 19);
    assert!(prose(&project).contains("L19"), "{}", prose(&project));
}

#[test]
fn an_unparseable_file_is_exit_2_and_no_view_is_printed() {
    // ADR-0011: "Nothing may partially process a malformed file." A view assembled from
    // half a parse is a picture of a document that does not exist.
    let dir = common::tempdir(line!());
    let project = common::write_project(&dir, "p.montaget.json", r#"{"fps": ,}"#);

    let answer = timeline::timeline(&project);

    assert_eq!(answer.report().exit_code(), ExitCode::Unparseable);
    assert!(answer.to_json()["timeline"].is_null());
    let prose = wire::render_timeline(&answer, Wire::Text { verbose: false });
    assert!(prose.contains("E-PARSE"), "{prose}");
    assert!(!prose.contains("GROUPS"), "{prose}");
}

#[test]
fn a_file_that_is_not_a_project_is_refused_rather_than_viewed() {
    // The same structural predicate ADR-0042 gives `fmt`, and for the same reason: with no
    // `tracks` there is nothing to lay out, and a raw schema error is the message-quality
    // gap that ADR found.
    let dir = common::tempdir(line!());
    let project = common::write_project(&dir, "p.montaget.json", r#"{"name": "not a project"}"#);

    let answer = timeline::timeline(&project);

    assert_eq!(answer.report().exit_code(), ExitCode::Errors);
    assert_eq!(answer.report().findings[0].code, "E-NOT-A-PROJECT");
    assert!(answer.to_json()["timeline"].is_null());
}

#[test]
fn the_json_form_replaces_the_prose_and_never_accompanies_it() {
    // ADR-0006, as `Wire` makes structural: one form per invocation.
    let answer = timeline::timeline(&fixture());

    let json = wire::render_timeline(&answer, Wire::Json);
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("the JSON form parses");
    assert_eq!(parsed["tool"], "timeline");
    assert_eq!(parsed["timeline"]["counts"]["elements"], 60);
    assert!(!json.contains("GROUPS"));

    let text = wire::render_timeline(&answer, Wire::Text { verbose: false });
    assert!(text.contains("GROUPS"));
    assert!(!text.contains("\"counts\""));
}

#[test]
fn the_view_reads_the_project_and_writes_nothing() {
    // ADR-0011 lists `timeline` among the reads. Asserted rather than assumed: it is the
    // one property a reader of a "wide view" would never think to check.
    let dir = common::tempdir(line!());
    let body = common::canonical(
        r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"photo","layer":10,
            "elements":[{"id":"a","type":"rect","start":0,"end":100,"width":10,"height":10,"fill":"#FFFFFF"}]}]}"##,
    );
    let project = common::write_project(&dir, "p.montaget.json", &body);

    let answer = timeline::timeline(&project);

    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    assert_eq!(std::fs::read_to_string(&project).unwrap(), body);
    assert_eq!(
        std::fs::read_dir(&dir).unwrap().count(),
        1,
        "no sidecar, no temp file, nothing beside the project"
    );
}

#[test]
fn a_mid_edit_element_missing_its_range_is_shown_rather_than_dropped() {
    // The view is wanted exactly when the document is half-written. An element the types
    // cannot hold is still an element a human needs to see.
    let dir = common::tempdir(line!());
    let project = common::write_project(
        &dir,
        "p.montaget.json",
        &common::canonical(
            r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"photo","layer":10,
                "elements":[{"id":"half-written","type":"rect","group":"beat","start":0,"width":10,"height":10,"fill":"#FFFFFF"}]}]}"##,
        ),
    );

    let json = json(&project);
    let element = &json["timeline"]["groups"][0]["elements"][0];

    assert_eq!(element["id"], "half-written");
    assert!(element["end"].is_null());
    assert!(element["duration_ms"].is_null());
    // The block still states what it knows: the earliest instant is stated, the latest is
    // not, and the length between them is therefore not a number anyone may print.
    assert_eq!(json["timeline"]["groups"][0]["start"], 0);
    assert!(json["timeline"]["groups"][0]["end"].is_null());
    assert!(json["timeline"]["groups"][0]["duration_ms"].is_null());
    assert!(prose(&project).contains("half-written"));
}

#[test]
fn an_element_that_moves_is_marked_and_a_still_one_is_not() {
    // The prototype this view is built on carried the mark (`05.png ~anim`). Without it the
    // fixture's Ken Burns push and a still photograph are the same row — and every one of
    // its seven photos is keyframed while none of its cards is.
    let prose = prose(&fixture());

    assert!(prose.contains("images/05.png ~anim"), "{prose}");
    assert!(
        !prose.contains("984×169 #1E344C ~anim"),
        "a static rect carries no motion:\n{prose}"
    );
    // The trap: `runs` is also an array of objects, so a shape test over every key marks
    // all 22 of the fixture's text elements as moving. None of them is.
    assert!(
        !prose.contains("I hang cobwebs over the door. ~anim"),
        "`runs` is not a keyframe list:\n{prose}"
    );
    assert!(
        prose.matches("~anim").count() == 7,
        "exactly the seven keyframed photos:\n{prose}"
    );
}

#[test]
fn a_header_field_the_document_states_survives_into_the_prose_whatever_its_type() {
    // ADR-0030: presence is content. A `background` the author wrote is a declaration, and a
    // view that showed it only when it happened to be a string would let the prose and the
    // canonical JSON disagree about what the file says — which is the one thing ADR-0006's
    // "text is generated from it" exists to prevent.
    let dir = common::tempdir(line!());
    let project = common::write_project(
        &dir,
        "p.montaget.json",
        &common::canonical(
            r##"{"frame":{"width":1080,"height":1920},"fps":25,"background":12,"tracks":[]}"##,
        ),
    );

    let json = json(&project);
    assert_eq!(json["timeline"]["background"], 12);
    assert!(
        prose(&project).contains("background  12"),
        "{}",
        prose(&project)
    );
}
