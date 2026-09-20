//! The keyframe resolver, and `query --at` over the document (#208).
//!
//! The acceptance criteria are four sentences, and the sections below answer them in order:
//! `--at` returns resolved values and never the keyframe records; painter's order comes from
//! ticket 11's resolution rather than a second implementation; `ease` on a first record and
//! a missing `ease` on a later one are both schema errors (ADR-0038); and an off-grid
//! keyframe time resolves with no rounding rule anywhere (ADR-0035).
//!
//! Everything is asserted through the core verb API — seam 1 — against a project file, which
//! is the only seam an agent can ever observe.

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

/// The `query` block of one `--at` answer.
fn at(path: &Path, instant: i64) -> Value {
    query::query(
        path,
        &Ask {
            at: Some(instant),
            ..Ask::default()
        },
    )
    .to_json()["query"]
        .clone()
}

fn prose(path: &Path, instant: i64) -> String {
    wire::render_query(
        &query::query(
            path,
            &Ask {
                at: Some(instant),
                ..Ask::default()
            },
        ),
        Wire::Text { verbose: false },
    )
}

/// One member of the resolved stack, by `id`.
fn element<'a>(answer: &'a Value, id: &str) -> &'a Value {
    answer["stack"]
        .as_array()
        .expect("the stack is a list")
        .iter()
        .find(|element| element["id"] == json!(id))
        .unwrap_or_else(|| panic!("`{id}` is not in the stack: {answer}"))
}

/// One resolved property of one element.
fn value<'a>(answer: &'a Value, id: &str, property: &str) -> &'a Value {
    element(answer, id)["values"]
        .as_array()
        .expect("values is a list")
        .iter()
        .find(|value| value["property"] == json!(property))
        .unwrap_or_else(|| panic!("`{id}` resolves no `{property}`: {answer}"))
}

/// The ids of the stack, in the order the answer puts them in.
fn order(answer: &Value) -> Vec<String> {
    answer["stack"]
        .as_array()
        .expect("the stack is a list")
        .iter()
        .map(|element| element["id"].as_str().unwrap_or("?").to_string())
        .collect()
}

/// Write one project into a scratch directory, canonically, and hand back its path.
fn project(line: u32, body: Value) -> PathBuf {
    let dir = common::tempdir(line);
    common::write_project(
        &dir,
        "p.montaget.json",
        &common::canonical(&body.to_string()),
    )
}

/// One `rect` with a range, plus whatever animated properties the caller adds.
fn rect(id: &str, start: i64, end: i64, extra: Value) -> Value {
    let mut element = json!({
        "id": id, "type": "rect", "start": start, "end": end,
        "width": 10, "height": 10, "fill": "#000000"
    });
    for (key, value) in extra.as_object().expect("an object of extra keys") {
        element[key] = value.clone();
    }
    element
}

/// A project with one element per track, so that nothing here is also a track-overlap test.
fn spread(elements: Vec<Value>) -> Value {
    json!({
        "frame": {"width": 1080, "height": 1920},
        "fps": 25,
        "tracks": elements.into_iter().enumerate().map(|(i, element)| json!({
            "name": format!("t{i}"),
            "layer": i as i64,
            "elements": [element],
        })).collect::<Vec<_>>(),
    })
}

// ---- `query --at` returns resolved values, never the keyframe records ----------------

#[test]
fn an_animated_value_arrives_interpolated_and_the_records_never_appear() {
    // ADR-0011's own worked number: *"echoing `"scale":[[3018,1.0],[18018,1.08]]` back at
    // the agent tells it nothing it did not have; `scale 1.0170` is the entire point."* The
    // fixture's `photo-05` is that element, and 6205 ms is where the ramp reaches it.
    let answer = at(&fixture(), 6205);

    assert_eq!(
        value(&answer, "photo-05", "scale")["value"],
        json!([1.0169973333333333, 1.0169973333333333]),
    );
    assert_eq!(value(&answer, "photo-05", "scale")["animated"], json!(true));

    // Not a single keyframe record anywhere in the answer. `ease` and `v` are the two keys a
    // record carries that nothing else in this view does, so their absence is the assertion.
    let serialised = answer.to_string();
    assert!(!serialised.contains("\"ease\""), "{serialised}");
    assert!(!serialised.contains("\"v\""), "{serialised}");
    assert!(!prose(&fixture(), 6205).contains("ease"));
}

#[test]
fn a_static_value_resolves_to_a_number_and_never_to_two_shapes() {
    // A resolved `x` is a number whether or not anything animates it. Two spellings for one
    // value put a shape test in every consumer, which is what ADR-0012 retired the `scale`
    // union for — and an integer would publish the rounding rule ADR-0035 says there is not.
    let path = project(
        line!(),
        spread(vec![rect("still", 0, 1000, json!({"x": 540, "y": 100}))]),
    );
    let answer = at(&path, 500);

    assert!(value(&answer, "still", "x")["value"].is_f64());
    assert_eq!(value(&answer, "still", "x")["value"], json!(540.0));
    assert_eq!(value(&answer, "still", "x")["animated"], json!(false));
    // The prose is the place a whole number reads as one.
    assert!(
        prose(&path, 500).contains("x 540, y 100"),
        "{}",
        prose(&path, 500)
    );
}

#[test]
fn a_property_the_document_does_not_declare_is_not_invented() {
    // ADR-0012 publishes a default for every transform property, and ADR-0030 makes a
    // defaultable field's *presence* content: omitted and explicit-at-default are two
    // different declarations. A row reading `opacity 1` that might mean either would collapse
    // exactly that distinction, on the verb an agent uses to find out what the document says.
    let path = project(
        line!(),
        spread(vec![rect("bare", 0, 1000, json!({"opacity": 1.0}))]),
    );
    let answer = at(&path, 500);

    let properties: Vec<&str> = element(&answer, "bare")["values"]
        .as_array()
        .expect("values is a list")
        .iter()
        .map(|value| value["property"].as_str().unwrap_or("?"))
        .collect();
    assert_eq!(properties, ["opacity"]);
}

#[test]
fn the_value_holds_before_the_first_record_and_after_the_last() {
    // ADR-0012's clamp, at both ends: *"before the first keyframe the value is the first
    // value; after the last it is the last"*, which is what makes every one of the fixture's
    // seven trimmed moves well-defined — one of them sits 13.8 s past the end of the project.
    let path = project(
        line!(),
        spread(vec![rect(
            "ramp",
            1000,
            5000,
            json!({"x": [{"t": 2000, "v": 0}, {"t": 3000, "v": 100, "ease": "linear"}]}),
        )]),
    );

    assert_eq!(value(&at(&path, 1000), "ramp", "x")["value"], json!(0.0));
    assert_eq!(value(&at(&path, 2500), "ramp", "x")["value"], json!(50.0));
    assert_eq!(value(&at(&path, 4999), "ramp", "x")["value"], json!(100.0));
}

#[test]
fn a_name_and_its_control_points_are_one_evaluator() {
    // ADR-0012: *"A name is sugar over one evaluator, never a second mechanism."* The two
    // spellings of `ease-in-out` must resolve to the same `f64`, bit for bit.
    let keyframes =
        |ease: Value| json!({"opacity": [{"t": 0, "v": 0.0}, {"t": 1000, "v": 1.0, "ease": ease}]});
    let named = project(
        line!(),
        spread(vec![rect("e", 0, 2000, keyframes(json!("ease-in-out")))]),
    );
    let raw = project(
        line!(),
        spread(vec![rect(
            "e",
            0,
            2000,
            keyframes(json!([0.42, 0.0, 0.58, 1.0])),
        )]),
    );

    for instant in [1, 137, 250, 500, 750, 999] {
        assert_eq!(
            value(&at(&named, instant), "e", "opacity")["value"],
            value(&at(&raw, instant), "e", "opacity")["value"],
            "at {instant} ms",
        );
    }
    // And it is genuinely a curve: `ease-in-out` is symmetric about its midpoint, and slower
    // than linear on the way in.
    let half = value(&at(&named, 500), "e", "opacity")["value"]
        .as_f64()
        .expect("a number");
    assert!((half - 0.5).abs() < 1e-12, "{half}");
    let quarter = value(&at(&named, 250), "e", "opacity")["value"]
        .as_f64()
        .expect("a number");
    assert!(quarter < 0.25, "ease-in-out starts slowly, got {quarter}");
}

#[test]
fn step_holds_the_previous_value_until_the_record_it_enters() {
    // `step` is *"not a bezier"* (ADR-0012), and it is named `step` rather than `hold`
    // because "hold" reads as a claim about what happens next.
    let path = project(
        line!(),
        spread(vec![rect(
            "cut",
            0,
            2000,
            json!({"opacity": [{"t": 0, "v": 0.0}, {"t": 1000, "v": 1.0, "ease": "step"}]}),
        )]),
    );

    assert_eq!(
        value(&at(&path, 999), "cut", "opacity")["value"],
        json!(0.0)
    );
    assert_eq!(
        value(&at(&path, 1000), "cut", "opacity")["value"],
        json!(1.0)
    );
}

#[test]
fn presence_is_the_half_open_range_and_nothing_else() {
    // ADR-0005: an element ending at `t` is already out of the set at `t`, and one starting
    // there is already in it. `CONTEXT.md`'s *Presence set*, audio included.
    let path = project(
        line!(),
        spread(vec![
            rect("first", 0, 1000, json!({})),
            rect("second", 1000, 2000, json!({})),
        ]),
    );

    assert_eq!(order(&at(&path, 999)), ["first"]);
    assert_eq!(order(&at(&path, 1000)), ["second"]);
    assert_eq!(order(&at(&path, 2000)), Vec::<String>::new());
}

#[test]
fn an_element_the_document_does_not_place_on_the_clock_is_named_not_dropped() {
    // The cut list's rule, for its reason: a stack silently computed over fewer elements than
    // the project has is a wrong answer that looks like a right one.
    let path = project(
        line!(),
        json!({
            "frame": {"width": 1080, "height": 1920},
            "fps": 25,
            "tracks": [{"name": "t", "layer": 0, "elements": [
                {"id": "placed", "type": "rect", "start": 0, "end": 1000,
                 "width": 10, "height": 10, "fill": "#000000"},
                {"id": "floating", "type": "rect",
                 "width": 10, "height": 10, "fill": "#000000"},
            ]}],
        }),
    );
    let answer = at(&path, 500);

    assert_eq!(order(&answer), ["placed"]);
    assert_eq!(answer["unplaced"], json!(["floating"]));
    assert!(prose(&path, 500).contains("state no integer range"));
}

// ---- Painter's order comes from ticket 11's resolution -------------------------------

#[test]
fn an_anchor_resolves_to_an_integer_and_orders_the_stack() {
    // ADR-0019's one hop, through `crate::stack` — the resolution ticket 11 owns. ADR-0011
    // recorded the shell one-liner *"silently invents an order at layer ties"*; the anchor is
    // the case it could not do at all, and the fixture *"contains no layer anchors"*, so this
    // is written rather than measured.
    let path = project(
        line!(),
        json!({
            "frame": {"width": 1080, "height": 1920},
            "fps": 25,
            "tracks": [
                {"name": "over", "layer": 40, "elements": [
                    {"id": "badge", "type": "rect", "start": 0, "end": 1000, "layer": {"below": "card"},
                     "width": 10, "height": 10, "fill": "#000000"}]},
                {"name": "base", "layer": 30, "elements": [
                    {"id": "card", "type": "rect", "start": 0, "end": 1000,
                     "width": 10, "height": 10, "fill": "#000000"}]},
            ],
        }),
    );
    let answer = at(&path, 500);

    // `below` is one less than its target's own integer, and the order is back to front — so
    // the badge is painted first and the card over it, which is not the order either the
    // tracks or the arrays are written in.
    assert_eq!(element(&answer, "badge")["layer"], json!(29));
    assert_eq!(element(&answer, "card")["layer"], json!(30));
    assert_eq!(order(&answer), ["badge", "card"]);
}

#[test]
fn a_layer_that_does_not_resolve_is_reported_and_never_invented() {
    // The one hop is structural (ADR-0019), so an anchor onto an anchor has no integer. A
    // view that placed it anyway would be inventing the order ADR-0060 refuses to define.
    let path = project(
        line!(),
        json!({
            "frame": {"width": 1080, "height": 1920},
            "fps": 25,
            "tracks": [
                {"name": "a", "layer": 30, "elements": [
                    {"id": "card", "type": "rect", "start": 0, "end": 1000,
                     "width": 10, "height": 10, "fill": "#000000"}]},
                {"name": "b", "layer": 30, "elements": [
                    {"id": "badge", "type": "rect", "start": 0, "end": 1000, "layer": {"above": "card"},
                     "width": 10, "height": 10, "fill": "#000000"}]},
                {"name": "c", "layer": 30, "elements": [
                    {"id": "chained", "type": "rect", "start": 0, "end": 1000, "layer": {"above": "badge"},
                     "width": 10, "height": 10, "fill": "#000000"}]},
            ],
        }),
    );
    let answer = at(&path, 500);

    assert_eq!(element(&answer, "chained")["layer"], Value::Null);
    let why = element(&answer, "chained")["layer_unresolved"]
        .as_str()
        .expect("a reason");
    assert!(why.contains("one hop"), "{why}");
    // Last, because it has no place in the stack — not at zero, which would be a place.
    assert_eq!(order(&answer), ["card", "badge", "chained"]);
}

#[test]
fn a_tie_keeps_document_order_and_claims_nothing() {
    // ADR-0060: an overlapping tie is an `error` and the only ties this view can meet are
    // ones where *"any consistent internal order is correct by definition"*. The fixture's
    // own two clusters are exactly that, and they must stay in the file's order rather than
    // be reordered by anything this view decided.
    let answer = at(&fixture(), 1000);
    let tied: Vec<String> = answer["stack"]
        .as_array()
        .expect("the stack is a list")
        .iter()
        .filter(|element| element["layer"] == json!(31))
        .map(|element| element["id"].as_str().unwrap_or("?").to_string())
        .collect();

    assert_eq!(tied, ["flag-field", "handle-logo", "handle-text"]);
}

// ---- `ease` is a pure function of position (ADR-0038) --------------------------------

#[test]
fn an_ease_on_the_first_record_is_a_schema_error_naming_the_convention() {
    let path = project(
        line!(),
        spread(vec![rect(
            "wrong",
            0,
            1000,
            json!({"opacity": [
                {"t": 0, "v": 0.0, "ease": "linear"},
                {"t": 500, "v": 1.0, "ease": "linear"},
            ]}),
        )]),
    );
    let report = montaget_core::validate(&path);

    let schema: Vec<&montaget_core::finding::Finding> = report
        .findings
        .iter()
        .filter(|finding| finding.code == "E-SCHEMA")
        .collect();
    assert_eq!(schema.len(), 1, "{:?}", report.findings);
    let reason = schema[0].fields["reason"].as_str().expect("a reason");
    assert!(reason.contains("first keyframe record"), "{reason}");
    assert!(reason.contains("arriving at"), "{reason}");
}

#[test]
fn a_missing_ease_on_a_later_record_is_a_schema_error_and_no_default_is_published() {
    // ADR-0038 rejected *"optional with a published default (`linear`)"* outright: a default
    // is a fact every reader must independently know and correctly apply, and the file does
    // not display it. So the absence is an error rather than a value.
    let path = project(
        line!(),
        spread(vec![rect(
            "wrong",
            0,
            1000,
            json!({"opacity": [{"t": 0, "v": 0.0}, {"t": 500, "v": 1.0}]}),
        )]),
    );
    let report = montaget_core::validate(&path);

    let reason = report
        .findings
        .iter()
        .find(|finding| finding.code == "E-SCHEMA")
        .unwrap_or_else(|| panic!("{:?}", report.findings))
        .fields["reason"]
        .as_str()
        .expect("a reason");
    assert!(reason.contains("keyframe record 2"), "{reason}");
    assert!(reason.contains("no default"), "{reason}");

    // And the view says it could not answer rather than assuming `linear`.
    let answer = at(&path, 250);
    let opacity = value(&answer, "wrong", "opacity");
    assert_eq!(opacity["value"], Value::Null);
    assert!(
        opacity["unresolved"]
            .as_str()
            .is_some_and(|reason| reason.contains("ease")),
        "{opacity}",
    );
}

#[test]
fn the_committed_fixture_carries_the_ease_the_rule_requires() {
    // The negative half of the pair, and the one that matters most: the fixture is a real
    // published video, and *"a check that fires on it is wrong unless an ADR says otherwise"*.
    // All fourteen of its keyframes are `linear`, on the second record of seven lists.
    let report = montaget_core::validate(&fixture());

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.code == "E-SCHEMA"),
        "{:?}",
        report.findings,
    );
}

#[test]
fn the_published_schema_states_the_positional_rule_too() {
    // The published schema and the enforced schema must be the same thing (#168). A schema
    // that left `ease` optional everywhere would admit files the binary refuses.
    let schema = montaget_core::schema::generate();
    let defs = schema["$defs"].as_object().expect("the definitions");

    assert!(
        defs["Keyframe"]["required"]
            .as_array()
            .expect("required")
            .contains(&json!("ease")),
        "{}",
        defs["Keyframe"],
    );
    assert!(
        defs["FirstKeyframe"]["properties"]
            .as_object()
            .expect("properties")
            .get("ease")
            .is_none(),
        "{}",
        defs["FirstKeyframe"],
    );
}

// ---- Off-grid keyframe times, and no rounding rule (ADR-0035) ------------------------

#[test]
fn an_off_grid_keyframe_time_resolves_with_no_snapping_anywhere() {
    // ADR-0035, unanimous 3/3: *"the renderer needs no keyframe-specific rounding rule"* —
    // evaluation is the general continuous-property rule. At `fps: 30` the grid step is
    // `100/3` ms and only multiples of 100 ms are frame-exact, the non-obvious fact one agent
    // found only by building a private checker. Both keyframe times below are off that grid,
    // and both resolve as ordinary inputs.
    let path = project(
        line!(),
        json!({
            "frame": {"width": 1080, "height": 1920},
            "fps": 30,
            "tracks": [{"name": "t", "layer": 0, "elements": [
                {"id": "fade", "type": "rect", "start": 0, "end": 10000,
                 "width": 10, "height": 10, "fill": "#000000",
                 "opacity": [{"t": 333, "v": 0.0}, {"t": 1777, "v": 1.0, "ease": "linear"}]},
            ]}],
        }),
    );

    // The continuous value, exactly — not the value at the nearest sampled instant, and not a
    // value rounded to any number of places on the way out.
    let expected = (1000.0 - 333.0) / (1777.0 - 333.0);
    assert_eq!(
        value(&at(&path, 1000), "fade", "opacity")["value"],
        json!(expected),
    );

    // 1000 ms is frame-exact at 30 fps and 1001 ms is not — they are the same sampled frame.
    // A resolver that snapped `t` to the grid would answer both identically; this one does
    // not, because there is no grid in it.
    assert_ne!(
        value(&at(&path, 1000), "fade", "opacity")["value"],
        value(&at(&path, 1001), "fade", "opacity")["value"],
    );
}

// ---- The invocation, and the wire ----------------------------------------------------

#[test]
fn at_is_one_question_and_never_two() {
    // ADR-0011 keeps exit 3 apart from exit 1 and 2 so that "fix the command" is never read
    // as "fix the project".
    for ask in [
        Ask {
            at: Some(0),
            from: Some(0),
            to: Some(1000),
            ..Ask::default()
        },
        Ask {
            at: Some(0),
            predicate: Some("type = rect".into()),
            ..Ask::default()
        },
        Ask {
            at: Some(0),
            census: Some("y".into()),
            ..Ask::default()
        },
    ] {
        let answer = query::query(&fixture(), &ask);
        assert_eq!(
            answer.report().exit_code(),
            ExitCode::BadInvocation,
            "{ask:?} should not have been accepted",
        );
        assert_eq!(answer.report().findings[0].code, "E-INVOCATION");
        assert!(answer.to_json()["query"].is_null());
    }
}

#[test]
fn an_instant_outside_the_project_is_an_empty_stack_and_not_a_refusal() {
    let answer = at(&fixture(), 10_000_000);

    assert_eq!(order(&answer), Vec::<String>::new());
    assert_eq!(
        query::query(
            &fixture(),
            &Ask {
                at: Some(10_000_000),
                ..Ask::default()
            },
        )
        .report()
        .exit_code(),
        ExitCode::Ok,
    );
}

#[test]
fn the_prose_states_the_instant_the_order_and_the_precision() {
    let text = prose(&fixture(), 6205);

    assert!(text.contains("QUERY  the resolved stack at 6205"), "{text}",);
    assert!(text.contains("painter's order, back to front"), "{text}");
    // ADR-0012's own precision for the three continuous properties, disclosed rather than
    // left for a reader to find by comparing the two wire forms.
    assert!(text.contains("6 dp"), "{text}");
    assert!(text.contains("scale [1.016997, 1.016997]"), "{text}");
    // ADR-0006's wire decision: `--json` replaces the prose and never accompanies it.
    assert!(!text.contains('{'), "{text}");
}
