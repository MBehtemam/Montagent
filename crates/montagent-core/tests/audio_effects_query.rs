//! `query --at` describes an element's `audio_effects` list (ADR-0179 Consequences; #845 E5):
//! each member with its position, its values and whether it is bypassed, on an `audio` and
//! on a `video`.

use montagent_core::verbs::query::{Ask, query};
use serde_json::{Value, json};

mod common;
use common::{canonical, write_project};

#[track_caller]
fn queried(kind: &str, list: Value) -> Value {
    let mut element = json!({"id": "bed", "type": "audio", "start": 0, "end": 1000,
        "source": "a.wav", "source_start": 0, "source_end": 1000});
    if kind == "video" {
        element = json!({"id": "bed", "type": "video", "start": 0, "end": 1000,
            "source": "v.mp4", "source_start": 0, "source_end": 1000,
            "x": 0, "y": 0, "width": 100, "height": 100, "fit": "contain"});
    }
    element["audio_effects"] = list;
    let body = canonical(
        &json!({
            "frame": {"width": 100, "height": 100}, "fps": 25, "duration": 1000,
            "output": "out/t.mp4",
            "tracks": [{"name": "t", "layer": 0, "elements": [element]}],
        })
        .to_string(),
    );
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &body);
    query(
        &path,
        &Ask {
            at: Some(500),
            ..Ask::default()
        },
    )
    .to_json()
}

fn bed(answer: &Value) -> &Value {
    answer["query"]["stack"]
        .as_array()
        .expect("a stack")
        .iter()
        .find(|member| member["id"] == "bed")
        .unwrap_or_else(|| panic!("bed is present: {answer}"))
}

#[test]
fn query_at_lists_each_member_with_its_position_and_whether_it_is_bypassed() {
    let list = json!([
        {"name": "highpass", "frequency_hz": 100, "slope_db_per_oct": 24},
        {"name": "bell", "frequency_hz": 1500, "gain_db": -6, "q": 1.4, "enabled": false},
        {"name": "shelf", "side": "high", "frequency_hz": 4000, "gain_db": 3},
    ]);
    for kind in ["audio", "video"] {
        let answer = queried(kind, list.clone());
        let listed: Vec<(i64, &str, bool)> = bed(&answer)["audio_effects"]
            .as_array()
            .unwrap_or_else(|| panic!("{kind}: {answer}"))
            .iter()
            .map(|m| {
                (
                    m["index"].as_i64().unwrap(),
                    m["name"].as_str().unwrap(),
                    m["enabled"].as_bool().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            listed,
            [
                (0, "highpass", true),
                (1, "bell", false),
                (2, "shelf", true)
            ]
        );
        let text = montagent_core::text::render(&answer, montagent_core::text::Options::default())
            .expect("renders");
        for cell in [
            "audio_effects[0] highpass 100 Hz 24 dB/oct",
            "audio_effects[1] bell 1500 Hz -6 dB q 1.4 (bypassed)",
            "audio_effects[2] shelf high 4000 Hz 3 dB",
        ] {
            assert!(text.contains(cell), "{kind}: {cell}\n{text}");
        }
    }
}

#[test]
fn query_at_says_nothing_of_an_empty_list() {
    let answer = queried("audio", json!([]));
    assert!(bed(&answer).get("audio_effects").is_none(), "{answer}");
}

#[test]
fn query_at_describes_normalize_loudness_by_its_target_in_lufs() {
    let answer = queried(
        "audio",
        json!([
            {"name": "highpass", "frequency_hz": 80, "slope_db_per_oct": 12},
            {"name": "normalize_loudness", "target_lufs": -38},
        ]),
    );
    let listed = &bed(&answer)["audio_effects"][1];
    assert_eq!(listed["name"], "normalize_loudness", "{answer}");
    assert_eq!(listed["target_lufs"], -38, "{answer}");
    assert_eq!(listed["index"], 1);
    assert_eq!(listed["enabled"], true);
    let text = montagent_core::text::render(&answer, montagent_core::text::Options::default())
        .expect("renders");
    assert!(
        text.contains("audio_effects[1] normalize_loudness -38 LUFS"),
        "{text}"
    );
}
