//! `query`'s two reading modes (#196).
//!
//! The acceptance criteria are four sentences, and the tests below are organised as four
//! sections answering them: the cut list is made of element boundaries and never of sampled
//! instants, the boundary immediately outside the range is named on each side, `--where
//! --census` returns both the matched set and the distribution, and neither mode resolves
//! anything.

use montaget_core::Wire;
use montaget_core::report::ExitCode;
use montaget_core::verbs::query::{self, Ask};
use montaget_core::wire;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

mod common;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json")
}

fn cuts(path: &Path, from: i64, to: i64) -> Value {
    answer(
        path,
        &Ask {
            from: Some(from),
            to: Some(to),
            ..Ask::default()
        },
    )
}

fn matching(path: &Path, predicate: &str, census: Option<&str>) -> Value {
    answer(
        path,
        &Ask {
            predicate: Some(predicate.into()),
            census: census.map(str::to_string),
            ..Ask::default()
        },
    )
}

fn answer(path: &Path, ask: &Ask) -> Value {
    query::query(path, ask).to_json()
}

fn prose(path: &Path, ask: &Ask) -> String {
    wire::render_query(&query::query(path, ask), Wire::Text { verbose: false })
}

/// Every id a view names, in the order it names them.
fn ids(list: &Value, key: &str) -> Vec<String> {
    list[key]
        .as_array()
        .unwrap_or_else(|| panic!("`{key}` is a list, got {list}"))
        .iter()
        .map(|item| match item {
            Value::String(id) => id.clone(),
            object => object["id"].as_str().unwrap_or("?").to_string(),
        })
        .collect()
}

/// A two-track project with the boundaries a range test needs, written canonically so that
/// nothing here is also a `LAYOUT` test.
fn spaced_project() -> String {
    common::canonical(
        &json!({
            "frame": {"width": 1080, "height": 1920},
            "fps": 25,
            "tracks": [
                {"name": "photo", "layer": 0, "elements": [
                    {"id": "a", "type": "rect", "start": 0, "end": 1000,
                     "x": 0, "y": 0, "width": 10, "height": 10, "fill": "#000000"},
                    {"id": "b", "type": "rect", "start": 1000, "end": 3000,
                     "x": 0, "y": 0, "width": 10, "height": 10, "fill": "#000000"},
                    {"id": "d", "type": "rect", "start": 4000, "end": 5000,
                     "x": 0, "y": 0, "width": 10, "height": 10, "fill": "#000000"}
                ]},
                {"name": "vo", "layer": 1, "elements": [
                    {"id": "c", "type": "audio", "source": "a.mp3", "start": 500, "end": 2500}
                ]}
            ]
        })
        .to_string(),
    )
}

// ---- The cut list is element boundaries, never sampled instants ----------------------

#[test]
fn the_intervals_are_cut_at_element_boundaries_and_nowhere_else() {
    // ADR-0011: "the intervals over which the set of on-screen elements is constant. Not
    // sampled instants." The test of "not sampled" is that every interior cut is an instant
    // some element actually starts or ends at — a sampler would produce a regular grid.
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "spaced.montaget.json", &spaced_project());

    let json = cuts(&path, 0, 5000);
    let edges: Vec<(i64, i64)> = json["query"]["intervals"]
        .as_array()
        .expect("intervals")
        .iter()
        .map(|interval| {
            (
                interval["start"].as_i64().unwrap(),
                interval["end"].as_i64().unwrap(),
            )
        })
        .collect();

    // 0, 500, 1000, 2500, 3000, 4000 are the element boundaries inside the range; 5000 is
    // the range's own far edge. No 2000, no 3500: nothing changes there.
    assert_eq!(
        edges,
        [
            (0, 500),
            (500, 1000),
            (1000, 2500),
            (2500, 3000),
            (3000, 4000),
            (4000, 5000),
        ]
    );
}

#[test]
fn the_intervals_partition_the_range_asked_for_with_no_gap_and_no_overlap() {
    let json = cuts(&fixture(), 17000, 31000);
    let intervals = json["query"]["intervals"].as_array().expect("intervals");

    assert_eq!(intervals.first().expect("a first interval")["start"], 17000);
    assert_eq!(intervals.last().expect("a last interval")["end"], 31000);
    for pair in intervals.windows(2) {
        assert_eq!(
            pair[0]["end"], pair[1]["start"],
            "the intervals are consecutive and gapless"
        );
    }
    for interval in intervals {
        assert_eq!(
            interval["duration_ms"].as_i64(),
            Some(interval["end"].as_i64().unwrap() - interval["start"].as_i64().unwrap()),
            "duration is derived from the range, never stored (ADR-0005)"
        );
    }
}

#[test]
fn the_presence_set_really_is_constant_across_each_interval() {
    // The claim the mode is named for, checked against the document rather than against the
    // view: whatever is present when an interval opens is still present one millisecond
    // before it closes, and nothing else has appeared.
    let path = fixture();
    let document: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("read"))
        .expect("the fixture is JSON");
    let present_at = |t: i64| -> Vec<String> {
        let mut here: Vec<String> = Vec::new();
        for track in document["tracks"].as_array().expect("tracks") {
            for element in track["elements"].as_array().expect("elements") {
                let (start, end) = (
                    element["start"].as_i64().expect("start"),
                    element["end"].as_i64().expect("end"),
                );
                if start <= t && t < end {
                    here.push(element["id"].as_str().expect("id").to_string());
                }
            }
        }
        here
    };

    for interval in cuts(&path, 0, 65216)["query"]["intervals"]
        .as_array()
        .expect("intervals")
    {
        let (start, end) = (
            interval["start"].as_i64().unwrap(),
            interval["end"].as_i64().unwrap(),
        );
        assert_eq!(ids(interval, "present"), present_at(start));
        assert_eq!(
            present_at(start),
            present_at(end - 1),
            "the presence set changed inside [{start}, {end})"
        );
    }
}

#[test]
fn a_range_is_half_open_at_both_the_element_and_the_query() {
    // ADR-0005: `[start, end)`. `a` ends at 1000 and `b` starts there, so the instant 1000
    // belongs to `b` alone — and a query starting at 1000 does not carry `a` into its first
    // interval.
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "spaced.montaget.json", &spaced_project());

    let first = cuts(&path, 1000, 1200)["query"]["intervals"][0].clone();
    assert_eq!(ids(&first, "present"), ["b", "c"]);
}

#[test]
fn a_stretch_with_nothing_in_it_is_one_interval_and_not_an_empty_answer() {
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "spaced.montaget.json", &spaced_project());

    // 3000..4000 is between `b` ending and `d` starting: the frame is the background.
    let json = cuts(&path, 3200, 3600);
    let intervals = json["query"]["intervals"].as_array().expect("intervals");
    assert_eq!(intervals.len(), 1);
    assert!(ids(&intervals[0], "present").is_empty());
    assert!(
        prose(
            &path,
            &Ask {
                from: Some(3200),
                to: Some(3600),
                ..Ask::default()
            }
        )
        .contains("(nothing present)"),
        "an empty presence set is said in words, never as a blank cell"
    );
}

#[test]
fn an_audio_element_is_in_the_presence_set_and_carries_its_type() {
    // The documented departure from ADR-0011's "on-screen": ADR-0001's "audio is an element
    // like any other", and a caller wanting only the visual cut list filters `type`.
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "spaced.montaget.json", &spaced_project());

    let interval = cuts(&path, 600, 700)["query"]["intervals"][0].clone();
    let present = interval["present"].as_array().expect("present");
    assert_eq!(ids(&interval, "present"), ["a", "c"]);
    let audio = present
        .iter()
        .find(|m| m["id"] == "c")
        .expect("the audio member");
    assert_eq!(audio["type"], "audio");
    assert_eq!(audio["track"], "vo");
}

#[test]
fn an_element_the_document_does_not_place_on_the_clock_is_named_rather_than_dropped() {
    let dir = common::tempdir(line!());
    let path = common::write_project(
        &dir,
        "half-written.montaget.json",
        &common::canonical(
            &json!({
                "frame": {"width": 1080, "height": 1920},
                "fps": 25,
                "tracks": [{"name": "photo", "layer": 0, "elements": [
                    {"id": "placed", "type": "rect", "start": 0, "end": 100,
                     "x": 0, "y": 0, "width": 10, "height": 10, "fill": "#000000"},
                    {"id": "half-written", "type": "rect", "start": 0,
                     "x": 0, "y": 0, "width": 10, "height": 10, "fill": "#000000"}
                ]}]
            })
            .to_string(),
        ),
    );

    let json = cuts(&path, 0, 100);
    assert_eq!(ids(&json["query"], "unplaced"), ["half-written"]);
    assert_eq!(ids(&json["query"]["intervals"][0], "present"), ["placed"]);
    assert!(
        prose(
            &path,
            &Ask {
                from: Some(0),
                to: Some(100),
                ..Ask::default()
            }
        )
        .contains("state no integer range"),
        "a cut list taken over fewer elements than the project has says so"
    );
}

// ---- The boundary immediately outside the range, on each side ------------------------

#[test]
fn an_interval_is_as_long_as_the_presence_set_stays_the_same() {
    // "The intervals over which the presence set is constant" — so an interval that could
    // have been longer is not one. A zero-length element occupies no instant of the
    // half-open clock (ADR-0005) and still contributes two boundaries, so cutting at every
    // boundary would produce neighbours a reader cannot tell apart.
    let dir = common::tempdir(line!());
    let path = common::write_project(
        &dir,
        "zero-length.montaget.json",
        &common::canonical(
            &json!({
                "frame": {"width": 1080, "height": 1920},
                "fps": 25,
                "tracks": [
                    {"name": "photo", "layer": 0, "elements": [
                        {"id": "held", "type": "rect", "start": 0, "end": 1000,
                         "x": 0, "y": 0, "width": 10, "height": 10, "fill": "#000000"}
                    ]},
                    {"name": "marker", "layer": 1, "elements": [
                        {"id": "instant", "type": "rect", "start": 500, "end": 500,
                         "x": 0, "y": 0, "width": 10, "height": 10, "fill": "#FFFFFF"}
                    ]}
                ]
            })
            .to_string(),
        ),
    );

    let json = cuts(&path, 0, 1000);
    let intervals = json["query"]["intervals"]
        .as_array()
        .expect("intervals")
        .clone();
    assert_eq!(intervals.len(), 1, "500 changes nothing: {intervals:?}");
    assert_eq!(intervals[0]["start"], 0);
    assert_eq!(intervals[0]["end"], 1000);
    assert_eq!(intervals[0]["duration_ms"], 1000);
    assert_eq!(ids(&intervals[0], "present"), ["held"]);
    // The instant is still a boundary of the document, so it is still findable from outside
    // the range — the merge is about the intervals, not about what the document says.
    assert_eq!(cuts(&path, 600, 700)["query"]["previous"]["at"], 500);
}

#[test]
fn two_elements_with_no_id_are_two_members_and_not_one() {
    // ADR-0019 requires a unique `id`, so a document without one is already a `validate`
    // finding — but the census beneath it must still count. A shared placeholder would
    // deduplicate against itself and report a distribution that is short by one.
    let dir = common::tempdir(line!());
    let path = common::write_project(
        &dir,
        "no-ids.montaget.json",
        &common::canonical(
            &json!({
                "frame": {"width": 1080, "height": 1920},
                "fps": 25,
                "tracks": [{"name": "photo", "layer": 0, "elements": [
                    {"type": "rect", "start": 0, "end": 100,
                     "x": 0, "y": 7, "width": 10, "height": 10, "fill": "#000000"},
                    {"type": "rect", "start": 100, "end": 200,
                     "x": 0, "y": 7, "width": 10, "height": 10, "fill": "#000000"}
                ]}]
            })
            .to_string(),
        ),
    );

    let census = matching(&path, "type = rect", Some("y"))["query"]["census"].clone();
    assert_eq!(
        census["groups"][0]["members"]
            .as_array()
            .expect("members")
            .len(),
        2,
        "both elements are at y = 7: {census}"
    );
    // And each is named by where it was found, so the two reports can be read side by side.
    assert_eq!(
        ids(&census["groups"][0], "members"),
        ["(element 0, no id)", "(element 1, no id)"]
    );
    // A boundary's entering/leaving lists are names, so they use the same spelling; a
    // presence member is the element itself, so its `id` stays what the document writes —
    // `null` — rather than a name this verb made up.
    let boundary = cuts(&path, 50, 60)["query"]["next"].clone();
    assert_eq!(ids(&boundary, "entering"), ["(element 1, no id)"]);
    assert!(
        cuts(&path, 0, 50)["query"]["intervals"][0]["present"][0]["id"].is_null(),
        "a presence member echoes the document, and the document writes no id"
    );
}

#[test]
fn an_element_reaching_one_value_twice_is_one_member_of_that_group() {
    // A two-run element whose runs name the same font is one user of it, not two.
    let dir = common::tempdir(line!());
    let path = common::write_project(
        &dir,
        "same-font-twice.montaget.json",
        &common::canonical(
            &json!({
                "frame": {"width": 1080, "height": 1920},
                "fps": 25,
                "fonts": {"brand": "fonts/A.otf"},
                "tracks": [{"name": "caption", "layer": 0, "elements": [
                    {"id": "twice", "type": "text", "start": 0, "end": 100,
                     "x": 0, "y": 0, "origin": "center", "width": 10, "height": 10,
                     "font": "brand", "size": 55, "line_height": 1.1, "color": "#FFF8E8",
                     "align": "center",
                     "runs": [{"text": "a", "font": "brand"}, {"text": "b", "font": "brand"}]}
                ]}]
            })
            .to_string(),
        ),
    );

    let census = matching(&path, "type = text", Some("runs.*.font"))["query"]["census"].clone();
    assert_eq!(ids(&census["groups"][0], "members"), ["twice"]);
}

#[test]
fn the_boundary_immediately_outside_the_range_is_named_on_each_side() {
    // ADR-0011: it "must always name the boundary immediately outside the range on each
    // side... without the caller guessing a window."
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "spaced.montaget.json", &spaced_project());

    let json = cuts(&path, 1200, 2000);
    let previous = &json["query"]["previous"];
    let next = &json["query"]["next"];

    assert_eq!(previous["at"], 1000);
    assert_eq!(ids(previous, "entering"), ["b"]);
    assert_eq!(ids(previous, "leaving"), ["a"]);
    assert_eq!(next["at"], 2500);
    assert_eq!(ids(next, "leaving"), ["c"]);
}

#[test]
fn a_boundary_landing_exactly_on_to_is_the_one_immediately_outside() {
    // `to` is outside a half-open range, so the first boundary the range does not contain
    // is the one at `to` itself. The two sides stay symmetric under one convention.
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "spaced.montaget.json", &spaced_project());

    let json = cuts(&path, 1200, 2500);
    assert_eq!(json["query"]["next"]["at"], 2500);
    assert_eq!(
        json["query"]["intervals"]
            .as_array()
            .expect("intervals")
            .len(),
        1,
        "nothing changes strictly inside [1200, 2500)"
    );
    // And a boundary landing exactly on `from` is *inside* — it opens the first interval.
    let json = cuts(&path, 1000, 2000);
    assert_eq!(json["query"]["previous"]["at"], 500);
}

#[test]
fn a_range_with_nothing_outside_it_says_so_rather_than_inventing_a_boundary() {
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "spaced.montaget.json", &spaced_project());

    let json = cuts(&path, -100, 6000);
    assert!(json["query"]["previous"].is_null());
    assert!(json["query"]["next"].is_null());
    assert!(
        prose(
            &path,
            &Ask {
                from: Some(-100),
                to: Some(6000),
                ..Ask::default()
            }
        )
        .contains("no boundary outside the range on this side"),
        "the prose states the absence rather than dropping the row"
    );
}

// ---- The matched set and its distribution --------------------------------------------

#[test]
fn where_and_census_return_both_the_matched_set_and_the_distribution() {
    let json = matching(&fixture(), "track = sentence-text", Some("y"));
    let view = &json["query"];

    assert_eq!(view["mode"], "matches");
    assert_eq!(
        ids(view, "matched"),
        [
            "sentence-05",
            "sentence-06",
            "sentence-07",
            "sentence-08",
            "sentence-quiz"
        ]
    );
    assert_eq!(view["census"]["field"], "y");
    let groups = view["census"]["groups"].as_array().expect("groups");
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0]["value"], 1537);
    assert_eq!(groups[0]["members"].as_array().expect("members").len(), 5);
}

#[test]
fn a_census_shows_the_one_sibling_that_disagrees_beside_the_four_that_agree() {
    // ADR-0011's own worked example: "four of five siblings agree and one does not", as one
    // call rather than a script.
    let dir = common::tempdir(line!());
    let path = common::write_project(
        &dir,
        "siblings.montaget.json",
        &common::canonical(&siblings().to_string()),
    );

    let json = matching(&path, "group = captions", Some("y"));
    let groups = json["query"]["census"]["groups"]
        .as_array()
        .expect("groups")
        .clone();

    assert_eq!(groups[0]["value"], 1597);
    assert_eq!(groups[0]["members"].as_array().unwrap().len(), 4);
    assert_eq!(groups[1]["value"], 1537);
    assert_eq!(ids(&groups[1], "members"), ["odd-one"]);
    assert!(
        prose(
            &path,
            &Ask {
                predicate: Some("group = captions".into()),
                census: Some("y".into()),
                ..Ask::default()
            }
        )
        .contains("census y: 4 at 1597, 1 at 1537"),
    );
}

#[test]
fn census_groups_keep_first_appearance_order_and_are_never_sorted_by_size() {
    // ADR-0043: a census "must not be worded in a way that implies the larger group is the
    // correct one", and the fixture's own `word-08-target` is an outlier that is correct.
    // The lone value is written first here, so size-ordering and document-ordering disagree
    // and only one of them can be what the answer shows.
    let dir = common::tempdir(line!());
    let mut project = siblings();
    let elements = project["tracks"][0]["elements"].as_array_mut().unwrap();
    let mut odd = elements.pop().expect("the odd one out");
    // Moved on the clock, not merely in the array: the canonical convention sorts a track's
    // elements by their range, so an array reordering alone would not survive being written
    // out — and array order carries no meaning anyway (ADR-0060).
    odd["start"] = json!(-1000);
    odd["end"] = json!(0);
    elements.insert(0, odd);
    let path = common::write_project(
        &dir,
        "siblings.montaget.json",
        &common::canonical(&project.to_string()),
    );

    let groups = matching(&path, "group = captions", Some("y"))["query"]["census"]["groups"]
        .as_array()
        .expect("groups")
        .clone();
    assert_eq!(groups[0]["value"], 1537, "the group of one was seen first");
    assert_eq!(groups[1]["value"], 1597);
}

#[test]
fn a_matched_element_that_writes_no_censused_field_is_its_own_list_not_a_null_group() {
    // ADR-0030: omission and an explicit value are different declarations, so "writes no
    // `y`" may not be filed under the same heading as "writes `y: null`".
    let dir = common::tempdir(line!());
    let path = common::write_project(
        &dir,
        "mixed.montaget.json",
        &common::canonical(
            &json!({
                "frame": {"width": 1080, "height": 1920},
                "fps": 25,
                "tracks": [{"name": "vo", "layer": 0, "elements": [
                    {"id": "spoken", "type": "audio", "source": "a.mp3",
                     "start": 0, "end": 100, "volume": 1},
                    {"id": "quiet", "type": "audio", "source": "b.mp3", "start": 100, "end": 200}
                ]}]
            })
            .to_string(),
        ),
    );

    let census = matching(&path, "type = audio", Some("volume"))["query"]["census"].clone();
    assert_eq!(ids(&census["groups"][0], "members"), ["spoken"]);
    assert_eq!(ids(&census, "absent"), ["quiet"]);
    assert!(
        !census["groups"]
            .as_array()
            .unwrap()
            .iter()
            .any(|group| group["value"].is_null()),
        "absence is never a group whose value is null"
    );
}

#[test]
fn the_reserved_track_path_names_the_track_the_element_sits_in() {
    // An element does not carry the name of its track — the nesting is the statement — and
    // filtering by track is the most ordinary question there is.
    let json = matching(&fixture(), "track = handle-text", None);
    assert_eq!(ids(&json["query"], "matched"), ["handle-text"]);
    assert_eq!(json["query"]["matched"][0]["track"], "handle-text");
}

#[test]
fn a_wildcard_segment_asks_about_every_member_of_an_array() {
    // `runs.*.font` is the shape ADR-0007's font census question has: a run may override its
    // element's font, so "which fonts does this element use" is a question about the array
    // and not about its first member.
    let dir = common::tempdir(line!());
    let path = common::write_project(
        &dir,
        "runs.montaget.json",
        &common::canonical(
            &json!({
                "frame": {"width": 1080, "height": 1920},
                "fps": 25,
                "fonts": {"brand": "fonts/A.otf", "brand-old": "fonts/B.otf"},
                "tracks": [{"name": "caption", "layer": 0, "elements": [
                    {"id": "mixed", "type": "text", "start": 0, "end": 100,
                     "x": 0, "y": 0, "origin": "center", "width": 10, "height": 10,
                     "font": "brand", "size": 55, "line_height": 1.1, "color": "#FFF8E8",
                     "align": "center",
                     "runs": [{"text": "new "}, {"text": "old", "font": "brand-old"}]},
                    {"id": "plain", "type": "text", "start": 100, "end": 200,
                     "x": 0, "y": 0, "origin": "center", "width": 10, "height": 10,
                     "font": "brand", "size": 55, "line_height": 1.1, "color": "#FFF8E8",
                     "align": "center", "runs": [{"text": "new"}]}
                ]}]
            })
            .to_string(),
        ),
    );

    // "Has a run whose font is `brand-old`" — the any-value reading, which is what makes the
    // wildcard a question about the element.
    assert_eq!(
        ids(
            &matching(&path, "runs.*.font = brand-old", None)["query"],
            "matched"
        ),
        ["mixed"]
    );

    // And a census over a wildcard path distributes over the values, so an element using two
    // fonts is a member of both groups.
    let census = matching(&path, "type = text", Some("runs.*.font"))["query"]["census"].clone();
    assert_eq!(census["field"], "runs.*.font");
    assert_eq!(ids(&census["groups"][0], "members"), ["mixed"]);
    assert_eq!(census["groups"][0]["value"], "brand-old");
    // `plain`'s run states no font of its own, so the element states no value at this path
    // at all — which ADR-0030 keeps distinct from stating one.
    assert_eq!(ids(&census, "absent"), ["plain"]);
}

#[test]
fn exists_and_missing_ask_about_presence_rather_than_about_a_value() {
    let dir = common::tempdir(line!());
    let path = common::write_project(
        &dir,
        "clip.montaget.json",
        &common::canonical(
            &json!({
                "frame": {"width": 1080, "height": 1920},
                "fps": 25,
                "tracks": [{"name": "photo", "layer": 0, "elements": [
                    {"id": "cropped", "type": "image", "source": "a.png", "start": 0, "end": 100,
                     "width": 10, "height": 10, "fit": "cover",
                     "clip": {"x": 0, "y": 0, "width": 5, "height": 5}},
                    {"id": "whole", "type": "image", "source": "b.png", "start": 100, "end": 200,
                     "width": 10, "height": 10, "fit": "contain"}
                ]}]
            })
            .to_string(),
        ),
    );

    assert_eq!(
        ids(&matching(&path, "clip exists", None)["query"], "matched"),
        ["cropped"]
    );
    assert_eq!(
        ids(&matching(&path, "clip missing", None)["query"], "matched"),
        ["whole"]
    );
    // A path reaching nothing matches neither `=` nor `!=`: absence is asked about with
    // `missing`, never by falling out of a comparison.
    assert!(
        ids(
            &matching(&path, "clip.width != 5", None)["query"],
            "matched"
        )
        .is_empty()
    );
}

#[test]
fn the_comparison_operators_read_numbers_numerically_and_quoting_forces_a_string() {
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "spaced.montaget.json", &spaced_project());

    assert_eq!(
        ids(&matching(&path, "start >= 1000", None)["query"], "matched"),
        ["b", "d"]
    );
    assert_eq!(
        ids(&matching(&path, "end < 3000", None)["query"], "matched"),
        ["a", "c"]
    );
    // `1` and `1.0` are different `serde_json` numbers and the same quantity.
    let path = common::write_project(
        &dir,
        "opacity.montaget.json",
        &common::canonical(
            &json!({
                "frame": {"width": 1080, "height": 1920},
                "fps": 25,
                "tracks": [{"name": "photo", "layer": 0, "elements": [
                    {"id": "solid", "type": "rect", "start": 0, "end": 100,
                     "x": 0, "y": 0, "width": 10, "height": 10, "fill": "#000000", "opacity": 1.0}
                ]}]
            })
            .to_string(),
        ),
    );
    assert_eq!(
        ids(&matching(&path, "opacity = 1", None)["query"], "matched"),
        ["solid"]
    );
    // Quoting is what separates the word from the boolean, and the two are both askable.
    assert!(ids(&matching(&path, "id = \"solid\"", None)["query"], "matched") == ["solid"]);
}

#[test]
fn a_predicate_matching_nothing_is_an_empty_answer_and_not_a_failure() {
    let json = matching(&fixture(), "type = ellipse", None);
    assert!(
        json["query"]["matched"]
            .as_array()
            .expect("matched")
            .is_empty()
    );
    assert_eq!(json["exit_code"], 0);
}

#[test]
fn there_is_no_or_and_the_refusal_says_what_to_do_instead() {
    // ADR-0070 ratifies the absence rather than the implementation happening to lack it, so
    // the refusal teaches the rule: an agent that types `or` learns in one turn that two
    // calls answer it, rather than reading `is left over` and guessing at the spelling.
    let answer = query::query(
        &fixture(),
        &Ask {
            predicate: Some("type = text or type = image".into()),
            ..Ask::default()
        },
    );
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
    let reason = answer.report().findings[0].fields["reason"]
        .as_str()
        .expect("a reason")
        .to_string();
    assert!(reason.contains("no `or`"), "{reason}");
    assert!(reason.contains("two calls"), "{reason}");
    // The spellings an agent reaches for next are the ones the refusal has to catch, or it
    // has taught nothing: `OR` and `||` land on the same sentence.
    for spelling in ["type = text OR type = image", "type = text || type = image"] {
        let answer = query::query(
            &fixture(),
            &Ask {
                predicate: Some(spelling.into()),
                ..Ask::default()
            },
        );
        let reason = answer.report().findings[0].fields["reason"]
            .as_str()
            .expect("a reason")
            .to_string();
        assert!(reason.contains("no `or`"), "{spelling}: {reason}");
    }

    // A bare word that merely begins with `or` is a value, not the keyword.
    assert_eq!(
        query::query(
            &fixture(),
            &Ask {
                predicate: Some("id = orange".into()),
                ..Ask::default()
            },
        )
        .report()
        .exit_code(),
        ExitCode::Ok
    );
}

#[test]
fn an_index_segment_reaches_one_member_of_an_array_and_a_comparison_never_matches_a_substring() {
    let dir = common::tempdir(line!());
    let path = common::write_project(
        &dir,
        "indexed.montaget.json",
        &common::canonical(
            &json!({
                "frame": {"width": 1080, "height": 1920},
                "fps": 25,
                "fonts": {"brand": "fonts/A.otf", "brand-old": "fonts/B.otf"},
                "tracks": [{"name": "caption", "layer": 0, "elements": [
                    {"id": "mixed", "type": "text", "start": 0, "end": 100,
                     "x": 0, "y": 0, "origin": "center", "width": 10, "height": 10,
                     "font": "brand", "size": 55, "line_height": 1.1, "color": "#FFF8E8",
                     "align": "center",
                     "runs": [{"text": "new "}, {"text": "old", "font": "brand-old"}]},
                    {"id": "plain", "type": "text", "start": 100, "end": 200,
                     "x": 0, "y": 0, "origin": "center", "width": 10, "height": 10,
                     "font": "brand", "size": 55, "line_height": 1.1, "color": "#FFF8E8",
                     "align": "center", "runs": [{"text": "new"}]}
                ]}]
            })
            .to_string(),
        ),
    );

    // An all-digit segment indexes the array. It is there because a digit segment has to
    // mean *something*, and a path that silently matches nothing is the worse answer: read
    // as a key, `runs.1.font` would reach nothing on every array the format has.
    assert_eq!(
        ids(
            &matching(&path, "runs.1.font = brand-old", None)["query"],
            "matched"
        ),
        ["mixed"]
    );
    // The second run is the only one that states a font, so the first run states none.
    assert_eq!(
        ids(
            &matching(&path, "runs.0.font missing", None)["query"],
            "matched"
        ),
        ["mixed", "plain"]
    );

    // Whole values, never substrings: `brand` is not `brand-old`, which is what keeps a
    // census beneath a predicate readable — every group is a value some element states.
    assert_eq!(
        ids(&matching(&path, "font = brand", None)["query"], "matched"),
        ["mixed", "plain"]
    );
    assert!(
        ids(
            &matching(&path, "runs.*.font = brand", None)["query"],
            "matched"
        )
        .is_empty()
    );
}

// ---- Neither mode resolves anything ---------------------------------------------------

#[test]
fn a_predicate_matches_what_the_document_writes_and_never_an_interpolated_value() {
    // #196: "Neither mode resolves a keyframe." `x` here is a keyframe list passing through
    // 100 at 500 ms; a mode that resolved would match it and this mode does not.
    let dir = common::tempdir(line!());
    let path = common::write_project(
        &dir,
        "animated.montaget.json",
        &common::canonical(
            &json!({
                "frame": {"width": 1080, "height": 1920},
                "fps": 25,
                "tracks": [{"name": "photo", "layer": 0, "elements": [
                    {"id": "moving", "type": "rect", "start": 0, "end": 1000,
                     "x": [{"t": 0, "v": 0}, {"t": 1000, "v": 200, "ease": "linear"}],
                     "y": 0, "width": 10, "height": 10, "fill": "#000000"}
                ]}]
            })
            .to_string(),
        ),
    );

    assert!(ids(&matching(&path, "x = 100", None)["query"], "matched").is_empty());
    // The element is still reachable, by what the document does write about it.
    assert_eq!(
        ids(&matching(&path, "x exists", None)["query"], "matched"),
        ["moving"]
    );
    // And its range is still a range, so it is in the cut list like anything else.
    assert_eq!(
        ids(&cuts(&path, 400, 600)["query"]["intervals"][0], "present"),
        ["moving"]
    );
}

#[test]
fn neither_mode_reaches_for_a_resolver_the_stack_or_the_disk() {
    // The structural form of #196's "asserted by it working with no resolver present". A
    // test that only ran the verb would pass on a build that had grown a dependency on one
    // and happened not to need it for the fixture.
    //
    // The project file itself is read, through the same `parse::read` every verb reads it
    // with — which is why the file that calls it is not in this list. What these three must
    // not reach for is anything *beyond* the document: a probe, a session, a sidecar, or the
    // one hop that turns an anchor into an integer (ADR-0019), which is a resolved value and
    // belongs to `--at`.
    for (name, source) in [
        ("query/mod.rs", include_str!("../src/verbs/query/mod.rs")),
        ("query/cuts.rs", include_str!("../src/verbs/query/cuts.rs")),
        (
            "query/predicate.rs",
            include_str!("../src/verbs/query/predicate.rs"),
        ),
    ] {
        for forbidden in ["crate::stack", "crate::media", "std::fs"] {
            assert!(
                !source
                    .lines()
                    .filter(|line| !line.trim_start().starts_with("//"))
                    .any(|line| line.contains(forbidden)),
                "{name} reaches for `{forbidden}`; neither mode may resolve or probe anything"
            );
        }
    }
}

// ---- Invocation, and the file that is not a project ----------------------------------

#[test]
fn a_question_that_is_not_one_of_the_two_modes_is_exit_3_and_a_finding() {
    // ADR-0011: "an error is a finding... so there is exactly one thing to parse across the
    // surface", and exit 3's next move is "fix the command".
    let path = fixture();
    for ask in [
        Ask::default(),
        Ask {
            from: Some(1000),
            ..Ask::default()
        },
        Ask {
            from: Some(1000),
            to: Some(1000),
            ..Ask::default()
        },
        Ask {
            from: Some(2000),
            to: Some(1000),
            ..Ask::default()
        },
        Ask {
            from: Some(0),
            to: Some(1),
            predicate: Some("type = text".into()),
            census: None,
        },
        Ask {
            from: Some(0),
            to: Some(1),
            census: Some("y".into()),
            ..Ask::default()
        },
        Ask {
            predicate: Some("type ~ text".into()),
            ..Ask::default()
        },
        Ask {
            predicate: Some("type = text and".into()),
            ..Ask::default()
        },
        Ask {
            predicate: Some("type = 'text".into()),
            ..Ask::default()
        },
        Ask {
            predicate: Some("type = text --census y".into()),
            ..Ask::default()
        },
    ] {
        let answer = query::query(&path, &ask);
        assert_eq!(
            answer.report().exit_code(),
            ExitCode::BadInvocation,
            "{ask:?} should not have been accepted"
        );
        assert_eq!(answer.report().findings[0].code, "E-INVOCATION");
        assert!(answer.to_json()["query"].is_null());
    }
}

#[test]
fn the_invocation_is_settled_before_the_file_is_opened() {
    // `--from 5 --to 1` is wrong whatever the document says, and ADR-0011 keeps exit 3 apart
    // from exit 2 so that "fix the command" is never read as "fix the file".
    let answer = query::query(
        Path::new("a-file-that-does-not-exist.montaget.json"),
        &Ask {
            from: Some(5),
            to: Some(1),
            ..Ask::default()
        },
    );
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
}

#[test]
fn a_malformed_file_is_exit_2_with_no_partial_answer() {
    // ADR-0011: "nothing may partially process a malformed file."
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "broken.montaget.json", "{\n  \"fps\": ,\n}\n");

    let answer = query::query(
        &path,
        &Ask {
            from: Some(0),
            to: Some(1),
            ..Ask::default()
        },
    );
    assert_eq!(answer.report().exit_code(), ExitCode::Unparseable);
    assert_eq!(answer.report().findings[0].code, "E-PARSE");
    assert!(answer.to_json()["query"].is_null());
}

#[test]
fn a_file_that_is_not_a_project_gets_adr_0042_s_shared_predicate_and_no_second_one() {
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "transcript.json", "{\n  \"segments\": []\n}\n");

    let answer = query::query(
        &path,
        &Ask {
            predicate: Some("type = text".into()),
            ..Ask::default()
        },
    );
    assert_eq!(answer.report().findings[0].code, "E-NOT-A-PROJECT");
    assert!(answer.to_json()["query"].is_null());
}

// ---- The wire ------------------------------------------------------------------------

#[test]
fn json_replaces_the_prose_and_never_accompanies_it() {
    // ADR-0006's wire rule, transplanted onto this verb by ADR-0011 by name.
    let answer = query::query(
        &fixture(),
        &Ask {
            from: Some(17000),
            to: Some(19000),
            ..Ask::default()
        },
    );
    let json = wire::render_query(&answer, Wire::Json);
    let text = wire::render_query(&answer, Wire::Text { verbose: false });

    serde_json::from_str::<Value>(&json).expect("the JSON form is JSON alone");
    assert!(text.starts_with("0 errors"), "{text}");
    assert!(
        text.contains("QUERY  cut list over [17000, 19000)"),
        "{text}"
    );
    assert!(
        !text.contains('{'),
        "the prose form carries no JSON:\n{text}"
    );
}

/// Four captions that agree and one that does not — ADR-0011's worked example.
fn siblings() -> Value {
    let caption = |id: &str, start: i64, y: i64| {
        json!({
            "id": id, "type": "text", "group": "captions", "start": start, "end": start + 1000,
            "x": 540, "y": y, "origin": "center", "width": 984, "height": 169,
            "font": "brand", "size": 55, "line_height": 1.1, "color": "#FFF8E8",
            "align": "center", "runs": [{"text": id}]
        })
    };
    json!({
        "frame": {"width": 1080, "height": 1920},
        "fps": 25,
        "fonts": {"brand": "fonts/OpenRunde-Bold.otf"},
        "tracks": [{"name": "caption", "layer": 0, "elements": [
            caption("one", 0, 1597),
            caption("two", 1000, 1597),
            caption("three", 2000, 1597),
            caption("four", 3000, 1597),
            caption("odd-one", 4000, 1537)
        ]}]
    })
}
