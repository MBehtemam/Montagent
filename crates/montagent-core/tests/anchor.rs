//! The anchor: resolution in one hop, and the two checks ADR-0019 put over it (#198).
//!
//! Every project here is built from `rect` elements. They carry no `source`, so the disk
//! half of `validate` has nothing to ask and these tests run without an `ffprobe` — the
//! question is entirely about the document, which is what ADR-0019's checks are.

use montagent_core::finding::{Class, Finding, Repair};
use montagent_core::model::{Anchor, Layer};
use montagent_core::report::ExitCode;
use montagent_core::stack::{Side, Stack, TimelineRange, Unresolved};
use montagent_core::{text, validate};

mod common;
use common::{canonical, write_project};

/// A project of whatever tracks a test needs, in the canonical convention.
fn project(tracks: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{tracks}]}}"##
    ))
}

/// One track at `layer`, holding `elements`.
fn track(name: &str, layer: i64, elements: &str) -> String {
    format!(r##"{{"name":"{name}","layer":{layer},"elements":[{elements}]}}"##)
}

/// A rect on `[start, end)`, carrying `layer` verbatim where a test gives one.
fn rect(id: &str, start: i64, end: i64, layer: Option<&str>) -> String {
    let layer = match layer {
        Some(layer) => format!(r##","layer":{layer}"##),
        None => String::new(),
    };
    format!(
        r##"{{"id":"{id}","type":"rect","start":{start},"end":{end}{layer},"x":0,"y":0,"width":100,"height":100}}"##
    )
}

/// The report `validate` gives for a project, written to a scratch file.
#[track_caller]
fn report_on(tracks: &str) -> montagent_core::report::Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &project(tracks));
    validate(&path)
}

#[track_caller]
fn codes(report: &montagent_core::report::Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

fn render(report: &montagent_core::report::Report) -> String {
    text::render(&report.to_json(), text::Options::verbose()).expect("the report renders")
}

/// The stack a project describes, with the document kept alive for it to borrow from.
#[track_caller]
fn stack_of(tracks: &str) -> (montagent_core::permissive::Loose, String) {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &project(tracks));
    let document = montagent_core::parse::read(&path).expect("the scratch project parses");
    (document, path.display().to_string())
}

// ---------------------------------------------------------------------------
// Resolution: one hop, and what it resolves to.
// ---------------------------------------------------------------------------

#[test]
fn an_anchor_resolves_to_its_targets_layer_either_side_of_it() {
    // ADR-0004's own worked example: `{"below": "title"}` is "that element's layer minus
    // one, wherever either of them lives".
    let (document, _) = stack_of(&track(
        "titles",
        20,
        &format!(
            "{},{},{}",
            rect("title", 0, 1000, None),
            rect("panel", 0, 1000, Some(r##"{"below":"title"}"##)),
            rect("badge", 0, 1000, Some(r##"{"above":"title"}"##)),
        ),
    ));
    let stack = Stack::of(&document);

    assert_eq!(stack.layer_of("title"), Ok(20), "the track supplies it");
    assert_eq!(stack.layer_of("panel"), Ok(19));
    assert_eq!(stack.layer_of("badge"), Ok(21));
}

#[test]
fn an_anchor_resolves_against_a_target_in_another_track() {
    // "wherever either of them lives" is the whole point of the feature: the alternative is
    // hand-maintaining `panel.layer = title.layer - 1` across two tracks.
    let (document, _) = stack_of(&format!(
        "{},{}",
        track("titles", 20, &rect("title", 0, 1000, None)),
        track(
            "panels",
            5,
            &rect("panel", 0, 1000, Some(r##"{"below":"title"}"##))
        ),
    ));

    assert_eq!(
        Stack::of(&document).layer_of("panel"),
        Ok(19),
        "the anchor names an element, and the element's track is what supplies the number"
    );
}

#[test]
fn an_elements_own_integer_overrides_its_tracks_and_is_what_an_anchor_reads() {
    let (document, _) = stack_of(&track(
        "titles",
        20,
        &format!(
            "{},{}",
            rect("title", 0, 1000, Some("40")),
            rect("panel", 0, 1000, Some(r##"{"below":"title"}"##)),
        ),
    ));
    let stack = Stack::of(&document);

    assert_eq!(stack.layer_of("title"), Ok(40), "the override wins");
    assert_eq!(stack.layer_of("panel"), Ok(39));
}

#[test]
fn a_chained_anchor_is_refused_rather_than_walked() {
    // ADR-0019: "an anchor's target's own `layer` must not itself be an object. No walk, no
    // possibility of a cycle by construction." `caption` would resolve to 18 if the chain
    // were followed — the assertion is that nothing follows it.
    let (document, _) = stack_of(&track(
        "titles",
        20,
        &format!(
            "{},{},{}",
            rect("title", 0, 1000, None),
            rect("panel", 0, 1000, Some(r##"{"below":"title"}"##)),
            rect("caption", 0, 1000, Some(r##"{"below":"panel"}"##)),
        ),
    ));

    assert_eq!(
        Stack::of(&document).layer_of("caption"),
        Err(Unresolved::ChainedTarget("panel")),
        "one hop, and the second is an error rather than a second lookup"
    );
}

#[test]
fn a_cycle_is_unreachable_rather_than_detected() {
    // Two anchors naming each other. Neither resolves, and neither needs a visited-set to
    // say so: the one-hop rule makes the walk that could loop nonexistent.
    let (document, _) = stack_of(&track(
        "titles",
        20,
        &format!(
            "{},{}",
            rect("a", 0, 1000, Some(r##"{"below":"b"}"##)),
            rect("b", 0, 1000, Some(r##"{"above":"a"}"##)),
        ),
    ));
    let stack = Stack::of(&document);

    assert_eq!(stack.layer_of("a"), Err(Unresolved::ChainedTarget("b")));
    assert_eq!(stack.layer_of("b"), Err(Unresolved::ChainedTarget("a")));
}

#[test]
fn a_self_reference_is_named_as_one_rather_than_as_a_chain() {
    // An element is in its own index, so a lookup-first resolution would come back
    // "chained" — true, and it would send the author looking at the wrong element.
    let (document, _) = stack_of(&track(
        "titles",
        20,
        &rect("title", 0, 1000, Some(r##"{"below":"title"}"##)),
    ));

    assert_eq!(
        Stack::of(&document).layer_of("title"),
        Err(Unresolved::SelfReference("title"))
    );
}

#[test]
fn an_anchor_never_resolves_against_a_track_name() {
    // ADR-0019: the target namespace is element `id` only. One of #8's three agents could
    // not tell whether `"title"` named the element or the track named `titles`; here a
    // track carries the exact name the anchor does, and the anchor still finds nothing.
    let (document, _) = stack_of(&track(
        "title",
        20,
        &rect("panel", 0, 1000, Some(r##"{"below":"title"}"##)),
    ));

    assert_eq!(
        Stack::of(&document).layer_of("panel"),
        Err(Unresolved::MissingTarget("title"))
    );
}

#[test]
fn a_target_whose_layer_is_an_object_the_grammar_rejects_is_still_not_a_plain_integer() {
    // ADR-0019 states the one-hop rule structurally: "an anchor's target's own `layer` must
    // not itself be an object." Not "must not be a well-formed anchor" — an object. A
    // target carrying `{"below": 3}` fails that test exactly as a good anchor does, and a
    // resolution that only looked for well-formed anchors would let the condition the ADR
    // names pass in silence.
    for bad_target_layer in [r##"{"below":3}"##, r##"{"under":"title"}"##, "{}"] {
        let (document, _) = stack_of(&track(
            "titles",
            20,
            &format!(
                "{},{}",
                rect("title", 0, 1000, Some(bad_target_layer)),
                rect("panel", 0, 1000, Some(r##"{"below":"title"}"##)),
            ),
        ));

        assert_eq!(
            Stack::of(&document).layer_of("panel"),
            Err(Unresolved::ChainedTarget("title")),
            "target layer: {bad_target_layer}"
        );
    }
}

#[test]
fn an_element_whose_own_layer_is_unreadable_reports_nothing_about_an_anchor() {
    // The element carries no anchor — it carries a broken `layer`. That is a fact about the
    // schema, there is no target to report it against, and a second voice on it would be
    // ADR-0006's noise budget spent on a defect this check did not find.
    for own in [r##"{"below":3}"##, r##""title""##, "1.5"] {
        let report = report_on(&track("titles", 20, &rect("panel", 0, 1000, Some(own))));

        assert!(
            codes(&report).iter().all(|code| !code.contains("ANCHOR")),
            "own layer {own}: {:?}",
            report.findings
        );
    }
}

#[test]
fn draw_order_is_computable_for_every_element_at_once() {
    // The ticket's own sentence: "draw order becomes computable". One pass over the
    // document answers it for every element, defects included, and document order is a
    // traversal rather than a ranking (ADR-0060).
    let (document, _) = stack_of(&format!(
        "{},{}",
        track(
            "titles",
            20,
            &format!(
                "{},{}",
                rect("title", 0, 1000, None),
                rect("badge", 0, 1000, Some(r##"{"above":"title"}"##)),
            )
        ),
        track(
            "panels",
            5,
            &format!(
                "{},{}",
                rect("panel", 0, 1000, Some(r##"{"below":"title"}"##)),
                rect("orphan", 0, 1000, Some(r##"{"below":"nobody"}"##)),
            )
        ),
    ));
    let stack = Stack::of(&document);

    assert_eq!(
        stack.resolved().collect::<Vec<_>>(),
        vec![
            ("title", Ok(20)),
            ("badge", Ok(21)),
            ("panel", Ok(19)),
            ("orphan", Err(Unresolved::MissingTarget("nobody"))),
        ]
    );
}

#[test]
fn a_half_open_range_touching_at_a_boundary_is_not_an_overlap() {
    // ADR-0005: `[start, end)`. An element ending at 7500 is not on screen at 7500.
    let earlier = TimelineRange {
        start: 0,
        end: 7500,
    };
    let later = TimelineRange {
        start: 7500,
        end: 9000,
    };

    assert!(!earlier.overlaps(later));
    assert!(!later.overlaps(earlier));
    assert!(earlier.overlaps(TimelineRange {
        start: 7499,
        end: 9000
    }));
}

#[test]
fn an_empty_range_overlaps_nothing_including_itself() {
    // There is no instant at which it is on screen, so there is none at which its stacking
    // could matter.
    let empty = TimelineRange {
        start: 500,
        end: 500,
    };
    assert!(!empty.overlaps(empty));
    assert!(!empty.overlaps(TimelineRange {
        start: 0,
        end: 1000
    }));
}

#[test]
fn a_side_resolves_one_step_and_saturates_rather_than_wrapping() {
    assert_eq!(Side::Below.against(10), 9);
    assert_eq!(Side::Above.against(10), 11);
    // Wrapping here would put an element at the very bottom of the stack on top of
    // everything, which is the one wrong answer that looks like an answer.
    assert_eq!(Side::Below.against(i64::MIN), i64::MIN);
    assert_eq!(Side::Above.against(i64::MAX), i64::MAX);
}

// ---------------------------------------------------------------------------
// The checks.
// ---------------------------------------------------------------------------

#[test]
fn a_missing_target_is_an_error_naming_both_the_side_and_the_target() {
    // One element per track. An anchor is for stacking two elements that are on screen at
    // once, and ADR-0004 requires exactly that to live in two tracks — "elements that
    // should overlap belong in different tracks" — so a one-track spelling of this project
    // is an `E-TRACK-OVERLAP` before it is anything about an anchor.
    let report = report_on(&format!(
        "{},{}",
        track("titles", 20, &rect("title", 0, 1000, None)),
        track(
            "panels",
            21,
            &rect("panel", 0, 1000, Some(r##"{"below":"titel"}"##))
        ),
    ));

    assert_eq!(codes(&report), ["E-ANCHOR-MISSING"]);
    let finding = &report.findings[0];
    assert_eq!(finding.class, Class::Error);
    assert_eq!(finding.location.element.as_deref(), Some("panel"));
    assert_eq!(finding.location.track.as_deref(), Some("panels"));
    assert_eq!(finding.fields["side"], "below");
    assert_eq!(finding.fields["target"], "titel");
    assert_eq!(report.exit_code(), ExitCode::Errors);

    let rendered = render(&report);
    assert!(rendered.contains("titel"), "{rendered}");
    assert!(
        rendered.contains("not an element in this project"),
        "{rendered}"
    );
}

#[test]
fn a_self_referential_target_is_its_own_error() {
    let report = report_on(&track(
        "titles",
        20,
        &rect("title", 0, 1000, Some(r##"{"above":"title"}"##)),
    ));

    assert_eq!(codes(&report), ["E-ANCHOR-SELF"]);
    assert_eq!(report.findings[0].fields["side"], "above");
    assert!(
        render(&report).contains("the element itself"),
        "{}",
        render(&report)
    );
}

#[test]
fn a_chained_target_is_an_error_that_states_no_repair() {
    // ADR-0043: the author wanted this element to track the target, and the two ways to
    // make the document legal are not the same edit. Pinning an integer is also exactly the
    // hand-maintained arithmetic ADR-0004 adopted anchoring to remove.
    let report = report_on(&format!(
        "{},{},{}",
        track("titles", 20, &rect("title", 0, 1000, None)),
        track(
            "panels",
            21,
            &rect("panel", 0, 1000, Some(r##"{"below":"title"}"##))
        ),
        track(
            "captions",
            22,
            &rect("caption", 0, 1000, Some(r##"{"below":"panel"}"##))
        ),
    ));

    assert_eq!(
        codes(&report),
        ["E-ANCHOR-CHAIN"],
        "only the second hop is wrong"
    );
    let finding = &report.findings[0];
    assert_eq!(finding.location.element.as_deref(), Some("caption"));
    assert_eq!(finding.repair, Some(Repair::None));
    assert!(
        render(&report).contains("exactly one hop"),
        "{}",
        render(&report)
    );
}

#[test]
fn an_error_class_anchor_finding_carries_a_repair_either_way() {
    // ADR-0043: "Every `error`-class finding carries a `repair` field." The two advise-class
    // codes state a value; the refuse-class one states `none`. `Report::push` asserts the
    // field's presence, so reaching a report at all is half the claim — this states the
    // other half, that the advise-class value is a value and not the literal `none`.
    let report = report_on(&track(
        "titles",
        20,
        &rect("panel", 0, 1000, Some(r##"{"below":"nobody"}"##)),
    ));

    match &report.findings[0].repair {
        Some(Repair::Advise(value)) => assert!(
            value["value"].as_str().unwrap().contains("integer"),
            "the next move is stated: {value}"
        ),
        other => panic!("an advise-class check supplies its value: {other:?}"),
    }
}

#[test]
fn a_target_that_never_overlaps_in_time_is_review_with_every_number_inline() {
    // ADR-0019's `review`: legal, renders, and a permanent no-op — "the hardest class of
    // error to catch by reading". The anchor is correct; it simply cannot matter.
    let report = report_on(&track(
        "titles",
        20,
        &format!(
            "{},{}",
            rect("title", 0, 5000, None),
            rect("panel", 5000, 9000, Some(r##"{"below":"title"}"##)),
        ),
    ));

    assert_eq!(codes(&report), ["R-ANCHOR-NO-OVERLAP"]);
    let finding = &report.findings[0];
    assert_eq!(finding.class, Class::Review);
    assert_eq!(finding.repair, None, "`repair` is an axis of `error` alone");
    assert_eq!(finding.fields["start"], 5000);
    assert_eq!(finding.fields["end"], 9000);
    assert_eq!(finding.fields["target_start"], 0);
    assert_eq!(finding.fields["target_end"], 5000);
    assert_eq!(
        finding.fields["layer"], 19,
        "it resolves; it just changes nothing"
    );

    // ADR-0011: exit non-zero only on `error`. A `review` that broke the edit loop is
    // ADR-0006's alarm fatigue by another route.
    assert_eq!(report.exit_code(), ExitCode::Ok);

    let rendered = render(&report);
    assert!(rendered.contains("never on screen together"), "{rendered}");
    assert!(rendered.contains("layer 19"), "{rendered}");
}

#[test]
fn a_target_that_does_overlap_is_not_reported_at_all() {
    // One millisecond of shared time is enough for the z-order to have a consequence, so
    // the check has nothing to say.
    let report = report_on(&format!(
        "{},{}",
        track("titles", 20, &rect("title", 0, 5001, None)),
        track(
            "panels",
            21,
            &rect("panel", 5000, 9000, Some(r##"{"below":"title"}"##))
        ),
    ));

    assert!(report.findings.is_empty(), "{:?}", report.findings);
}

#[test]
fn an_anchor_on_an_element_with_no_readable_range_is_not_reported_as_inert() {
    // A stated number is a measured one (ADR-0006). An element mid-edit carries no integer
    // `start`, so no window can be computed — and a window that cannot be computed must
    // never be reported as a window that is not there.
    let report = report_on(&track(
        "titles",
        20,
        &format!(
            "{},{}",
            rect("title", 0, 1000, None),
            r##"{"id":"panel","type":"rect","start":"soon","end":9000,"layer":{"below":"title"},"x":0,"y":0,"width":100,"height":100}"##,
        ),
    ));

    assert!(
        !codes(&report).contains(&"R-ANCHOR-NO-OVERLAP"),
        "{:?}",
        report.findings
    );
}

#[test]
fn an_anchor_whose_target_states_no_layer_is_left_to_the_check_that_owns_the_schema() {
    // The track carries no `layer`, so nothing in the document states an integer for the
    // target. That is a schema fact, and this check adding a second voice to it would be
    // ADR-0006's noise budget spent on a defect it did not find.
    let report = report_on(&format!(
        r##"{{"name":"titles","elements":[{},{}]}}"##,
        rect("title", 0, 1000, None),
        rect("panel", 0, 1000, Some(r##"{"below":"title"}"##)),
    ));

    assert!(
        codes(&report)
            .iter()
            .all(|code| !code.starts_with("E-ANCHOR")),
        "{:?}",
        report.findings
    );
}

#[test]
fn every_defective_anchor_in_one_file_is_reported() {
    // ADR-0006: every check runs over the whole project, every time. A check that stopped
    // at the first defect would send an agent round the loop once per anchor.
    let report = report_on(&format!(
        "{},{},{},{}",
        track("titles", 20, &rect("title", 0, 1000, None)),
        track(
            "as",
            21,
            &rect("a", 0, 1000, Some(r##"{"below":"nobody"}"##))
        ),
        track("bs", 22, &rect("b", 0, 1000, Some(r##"{"above":"b"}"##))),
        track("cs", 23, &rect("c", 0, 1000, Some(r##"{"below":"a"}"##))),
    ));

    assert_eq!(
        codes(&report),
        ["E-ANCHOR-MISSING", "E-ANCHOR-SELF", "E-ANCHOR-CHAIN"]
    );
}

// ---------------------------------------------------------------------------
// The format itself.
// ---------------------------------------------------------------------------

#[test]
fn an_anchor_round_trips_through_the_strict_model_and_the_canonical_writer() {
    // The anchor is a value of the published format, not a thing only a check knows about:
    // it parses into the types, serialises back to the same bytes, and sits where canonical
    // key order puts it.
    let written = project(&track(
        "titles",
        20,
        &format!(
            "{},{}",
            rect("title", 0, 1000, None),
            rect("panel", 0, 1000, Some(r##"{"below":"title"}"##)),
        ),
    ));
    let document = montagent_core::permissive::Loose::new(
        "p.montagent.json",
        serde_json::from_str(&written).unwrap(),
    );
    let project = document.strict().expect("an anchor is part of the format");

    let panel = &project.tracks[0].elements[1];
    assert_eq!(
        panel.layer,
        Some(Layer::Relative(Anchor::Below("title".into())))
    );
    assert_eq!(
        project.tracks[0].elements[0].layer, None,
        "omitted stays omitted"
    );

    let round_tripped = montagent_core::write::canonical(&serde_json::to_value(&project).unwrap());
    assert_eq!(round_tripped, written);
    assert!(
        written.contains(r##""end":1000,"layer":{"below":"title"}"##),
        "`layer` sits at the tail of the universal prefix (#243):\n{written}"
    );
}

#[test]
fn an_integer_layer_and_an_anchor_are_the_only_two_forms() {
    // ADR-0017's closed schema, at the one field of the format that is polymorphic. A third
    // spelling — the bare string `anchor` carried, which is a retired one — is a parse
    // failure here rather than a value some later stage has to second-guess.
    for bad in [
        r##""title""##,
        r##"{"below":3}"##,
        r##"{"below":"a","above":"b"}"##,
        "1.5",
    ] {
        let written = project(&track("titles", 20, &rect("panel", 0, 1000, Some(bad))));
        let document = montagent_core::permissive::Loose::new(
            "p.montagent.json",
            serde_json::from_str(&written).unwrap(),
        );
        assert!(
            document.strict().is_err(),
            "`layer`: {bad} is not one of the format's two forms"
        );
    }
}

// ---------------------------------------------------------------------------
// The wiring.
// ---------------------------------------------------------------------------

#[test]
fn the_registry_declares_every_code_this_check_can_fire() {
    // A check emitting an unregistered code panics in `Finding::new`, which is a bug in the
    // check rather than a condition of the project. Stated here so the declaration and the
    // four call sites cannot drift apart silently.
    for code in [
        "E-ANCHOR-MISSING",
        "E-ANCHOR-SELF",
        "E-ANCHOR-CHAIN",
        "R-ANCHOR-NO-OVERLAP",
    ] {
        let spec = montagent_core::registry::spec(code).expect("registered");
        assert_eq!(spec.adr, "ADR-0019");
        assert!(!spec.template.is_empty());
        let _ = Finding::new(code);
    }
}
