//! ADR-0017, at runtime rather than in the schema: an unknown key is rejected at every
//! object level, and a defaultable field's presence survives a round trip.

use montaget_core::model::{Animatable, Body, Project};

/// One element of each shape the tests need, written the way the fixture writes them.
fn project_with(element: &str) -> String {
    format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{{"name":"t","layer":1,"elements":[{element}]}}]}}"##
    )
}

fn rejection(element: &str) -> String {
    let source = project_with(element);
    match serde_json::from_str::<Project>(&source) {
        Ok(_) => panic!("this was accepted and should not have been:\n{element}"),
        Err(e) => e.to_string(),
    }
}

#[test]
fn an_unknown_key_on_an_element_is_rejected_and_named() {
    // #48 measured agents systematically mistyping fields copied from examples — `gravty`
    // for `gravity`. Under an open schema that typo is silently accepted and the video
    // renders wrong with no signal, which is worse than a blocked render because a blocked
    // render is at least noticed.
    let message = rejection(
        r##"{"id":"a","type":"rect","start":0,"end":1,"x":0,"y":0,"width":10,"height":10,"fill":"#000000","gravty":"top"}"##,
    );
    assert!(message.contains("gravty"), "{message}");
}

#[test]
fn a_retired_key_is_rejected_like_any_other_unknown_one() {
    // `gravity` was measured inert on 8 of 8 image elements and retired by ADR-0015; the
    // schema does not remember it, which is the whole mechanism — with no version number in
    // the file, the unknown-key error is the only signal an old binary has.
    let message = rejection(
        r##"{"id":"a","type":"image","start":0,"end":1,"source":"a.png","width":10,"height":10,"fit":"literal","gravity":"top"}"##,
    );
    assert!(message.contains("gravity"), "{message}");
}

#[test]
fn the_schema_is_closed_inside_a_run() {
    // ADR-0017's scope decision, 2–1 for uniform closure: a `run` misspelling a style key is
    // exactly the silent-drift failure the policy exists to catch, and runs are authored in
    // the highest multiplicity per file.
    let message = rejection(
        r##"{"id":"a","type":"text","start":0,"end":1,"width":10,"height":10,"font":"brand","size":20,"runs":[{"text":"hi","weight":"bold"}]}"##,
    );
    assert!(message.contains("weight"), "{message}");
}

#[test]
fn the_schema_is_closed_inside_a_keyframe() {
    let message = rejection(
        r##"{"id":"a","type":"rect","start":0,"end":1,"width":10,"height":10,"fill":"#000000","opacity":[{"t":0,"v":1.0,"easing":"linear"}]}"##,
    );
    assert!(message.contains("easing"), "{message}");
}

#[test]
fn the_schema_is_closed_at_the_project_level_and_inside_a_track() {
    let project = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[],"version":1}"##;
    let message = serde_json::from_str::<Project>(project)
        .unwrap_err()
        .to_string();
    assert!(message.contains("version"), "{message}");

    let track = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"t","layer":1,"kind":"visual","elements":[]}]}"##;
    let message = serde_json::from_str::<Project>(track)
        .unwrap_err()
        .to_string();
    // A track's kind is the kind of its elements — derived, and therefore impossible to
    // forget, mistype or leave stale (ADR-0006).
    assert!(message.contains("kind"), "{message}");
}

#[test]
fn a_field_the_renderer_cannot_honour_is_an_error_naming_its_replacement() {
    // ADR-0012's specific trap: `opacity` on audio, which an agent will write meaning
    // volume and which would fade nothing, forever.
    let message = rejection(
        r##"{"id":"a","type":"audio","start":0,"end":1,"source":"a.mp3","source_start":0,"source_end":1,"opacity":0.5}"##,
    );
    assert!(message.contains("opacity"), "{message}");
}

#[test]
fn an_explicit_default_and_an_omission_stay_distinguishable() {
    // ADR-0030: they are two spellings of *different declarations*. Omission says "I have
    // no opinion, give me whatever the default is"; `opacity: 1` says "I have pinned this
    // to 1". They diverge the moment a future edit changes the field on a sibling, or the
    // default itself is revisited.
    let pinned = r##"{"id":"a","type":"rect","start":0,"end":1,"width":10,"height":10,"fill":"#000000","opacity":1.0}"##;
    let floating =
        r##"{"id":"a","type":"rect","start":0,"end":1,"width":10,"height":10,"fill":"#000000"}"##;

    let pinned: Project = serde_json::from_str(&project_with(pinned)).unwrap();
    let floating: Project = serde_json::from_str(&project_with(floating)).unwrap();

    let Body::Rect(a) = &pinned.tracks[0].elements[0].body else {
        panic!("a rect");
    };
    let Body::Rect(b) = &floating.tracks[0].elements[0].body else {
        panic!("a rect");
    };
    assert_eq!(a.opacity, Some(Animatable::Static(1.0)));
    assert_eq!(b.opacity, None);

    // And the distinction survives being written back out: a `#[serde(default)]` would have
    // collapsed both spellings into one on the way in, and materialising the omitted field
    // on the way out would invalidate any pending exact-string replace whose context window
    // touched the block.
    let written = serde_json::to_string(&floating).unwrap();
    assert!(!written.contains("opacity"), "{written}");
    let written = serde_json::to_string(&pinned).unwrap();
    assert!(written.contains(r##""opacity":1.0"##), "{written}");
}

#[test]
fn a_group_is_omitted_rather_than_written_as_null() {
    // ADR-0041: `group` omitted entirely, not written as `null`, matching what every
    // element in the committed fixture already does.
    let source = project_with(
        r##"{"id":"a","type":"rect","start":0,"end":1,"width":10,"height":10,"fill":"#000000"}"##,
    );
    let project: Project = serde_json::from_str(&source).unwrap();
    let written = serde_json::to_string(&project).unwrap();
    assert!(!written.contains("group"), "{written}");

    let explicit_null = project_with(
        r##"{"id":"a","type":"rect","group":null,"start":0,"end":1,"width":10,"height":10,"fill":"#000000"}"##,
    );
    let message = serde_json::from_str::<Project>(&explicit_null)
        .unwrap_err()
        .to_string();
    assert!(message.contains("null"), "{message}");
}

#[test]
fn a_colour_has_exactly_one_spelling() {
    // `#RRGGBBFF` is an error naming the six-digit form, and the three-digit shorthand and
    // CSS names do not exist — two spellings of one value break the write-read round trip,
    // as `center-center` does.
    for (colour, expected) in [
        ("#000000FF", "six-digit"),
        ("#fff8e8", "uppercase"),
        ("#FFF", "three-digit"),
        ("red", "not a colour"),
    ] {
        let message = rejection(&format!(
            r##"{{"id":"a","type":"rect","start":0,"end":1,"width":10,"height":10,"fill":"{colour}"}}"##
        ));
        assert!(
            message.contains(expected),
            "{colour} should be refused with `{expected}`, got: {message}"
        );
    }
}

#[test]
fn a_time_is_integer_milliseconds() {
    // Float seconds failed in practice — two agents produced `4.400000000000002` and
    // `14.832999999999998` in their own runs, and exact adjacency is the predicate the
    // overlap and gap rules are built on (ADR-0005).
    let message = rejection(
        r##"{"id":"a","type":"rect","start":0.5,"end":1,"width":10,"height":10,"fill":"#000000"}"##,
    );
    assert!(message.contains("integer milliseconds"), "{message}");
}
