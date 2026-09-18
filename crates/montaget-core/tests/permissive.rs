//! The permissive path: what `fmt` needs and the strict model cannot give it.

use montaget_core::permissive::{Loose, NotAProject};

/// A project carrying an unknown key, a retired spelling, keys out of canonical order, and
/// a field written explicitly at its default. Every one of those is a thing `validate` has
/// something to say about and `fmt` must still be able to format.
const AWKWARD: &str = r##"{
  "fps": 25,
  "frame": {"width": 1080, "height": 1920},
  "duration": 4000,
  "tracks": [
    {
      "name": "photo",
      "layer": 10,
      "elements": [
        {"type":"image","id":"photo-01","start":0,"end":4000,"source":"images/01.png","gravity":"top","width":1080,"height":1912,"fit":"cover","clip":[0,0,1080,1300],"opacity":1.0,"reviewedBy":"nobody"}
      ]
    }
  ]
}
"##;

#[test]
fn a_document_with_an_unknown_key_and_a_retired_one_round_trips_losslessly() {
    // ADR-0042: `fmt` proceeds on any file recognisably a project, and ADR-0017 makes an
    // unknown key an `error`. Both hold at once, which is why this path exists — the strict
    // model by construction cannot hold `reviewedBy`, and refusing to format the file would
    // make `fmt` unusable exactly when it is most wanted, mid-authoring.
    let loose = Loose::new("awkward.montaget.json", serde_json::from_str(AWKWARD).unwrap());

    assert_eq!(loose.canonical(), AWKWARD);
}

#[test]
fn key_order_survives_the_round_trip_rather_than_being_repaired() {
    // The document above writes `fps` before `frame` and `type` before `id`, neither of
    // which is canonical order. Reading and writing it back must not silently fix that:
    // `fmt` reorders keys and says so, and `validate` reports it as `LAYOUT` — a path that
    // quietly normalised on read would take that finding away from both of them.
    let loose = Loose::new("awkward.montaget.json", serde_json::from_str(AWKWARD).unwrap());
    let written = loose.canonical();

    let fps = written.find("\"fps\"").unwrap();
    let frame = written.find("\"frame\"").unwrap();
    assert!(fps < frame, "the header's own order is preserved");

    let kind = written.find("\"type\":\"image\"").unwrap();
    let id = written.find("\"id\":\"photo-01\"").unwrap();
    assert!(kind < id, "the element's own order is preserved");
}

#[test]
fn field_presence_survives_the_round_trip() {
    // ADR-0030: `fmt` must not insert a default for an omitted field, and must not strip a
    // field explicitly written at its default. Stripping `opacity: 1.0` would silently
    // delete a line the agent just added; materialising the omitted `x`/`y`/`origin` would
    // insert up to seven lines into the element and invalidate any pending exact-string
    // replace whose context window touched the block.
    let written = Loose::new("awkward.montaget.json", serde_json::from_str(AWKWARD).unwrap()).canonical();

    assert!(written.contains(r#""opacity":1.0"#), "an explicit default stays");
    assert!(!written.contains(r#""x":"#), "an omitted field stays omitted");
    assert!(!written.contains(r#""origin":"#));
}

#[test]
fn a_document_missing_the_keys_that_make_it_a_project_is_refused_by_name() {
    // The reported hazard is wrong-file destruction — `fmt` pointed at a transcript export
    // or a beats export will rewrite it — so the proportionate fix is an identity check,
    // not a correctness gate (ADR-0042).
    let transcript = serde_json::json!({"segments": [{"start": 0.0, "text": "hello"}]});

    let refusal = Loose::new("transcript.json", transcript).shape().expect_err("not a project");
    assert_eq!(
        refusal,
        NotAProject {
            missing: vec!["tracks", "fps", "frame"]
        }
    );
    // ADR-0042 asked for a message naming the likely mismatch rather than a raw schema dump.
    assert_eq!(
        refusal.to_string(),
        "this does not look like a Montaget project file — no `tracks`/`fps`/`frame`"
    );
}

#[test]
fn a_project_with_findings_against_it_is_still_a_project() {
    // The precondition is narrow: structural shape, never severity. A `note`- or
    // `review`-level finding does not make a file any less a legitimate, safely-formattable
    // project, and gating on a clean `validate` would let "does it say what you meant" hold
    // a byte-level rewrite hostage.
    let nonsense = serde_json::json!({
        "frame": "not a frame",
        "fps": "abc",
        "tracks": "not an array",
    });

    assert!(
        Loose::new("nonsense.json", nonsense).shape().is_ok(),
        "the three keys are present, so there is a project here to talk about"
    );
}

#[test]
fn the_committed_fixture_round_trips_through_the_permissive_path_too() {
    // One convention, one writer. If the two paths could disagree about layout, the file
    // `fmt` produced and the file the strict model produced would differ, and every
    // exact-string replace written against one of them would miss against the other.
    let source =
        std::fs::read_to_string("../../fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json")
            .unwrap();
    let loose = Loose::new("fixture", serde_json::from_str(&source).unwrap());

    assert_eq!(loose.canonical(), source);
}
