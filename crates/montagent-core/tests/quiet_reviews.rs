//! Review filtering and quieting in `validate` (#1138, #1142).

use montagent_core::report::ExitCode;
use montagent_core::validate;

mod common;
use common::write_project;

fn tempdir() -> std::path::PathBuf {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("montagent-quiet-reviews-{id}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn project_quiet_accepts_review_codes_without_skipping_analysis_and_rejects_error_codes_with_e_quiet_forbidden() {
    let dir = tempdir();

    // 1. Valid `quiet` with `R-EASE-INERT` on a project that triggers `R-EASE-INERT`
    let valid_project = r##"{
  "frame": {"width": 1080, "height": 1920},
  "fps": 30,
  "background": "#000000",
  "duration": 2000,
  "quiet": ["R-EASE-INERT"],
  "tracks": [
    {
      "name": "main",
      "layer": 1,
      "elements": [
        {
          "id": "box-1",
          "type": "rect",
          "start": 0,
          "end": 2000,
          "x": [
            {"t": 0, "v": 100},
            {"t": 1000, "v": 100, "ease": "ease-out"}
          ],
          "y": 100,
          "width": 200,
          "height": 200,
          "fill": "#FFFFFF"
        }
      ]
    }
  ]
}
"##;
    let valid_path = write_project(&dir, "valid_quiet.montagent.json", valid_project);
    let report = validate(&valid_path);

    assert_eq!(
        report.exit_code(),
        ExitCode::Ok,
        "unexpected errors: {:?}",
        report.findings
    );
    // Analysis never skips: the engine still records R-EASE-INERT in the raw findings
    assert!(
        report.findings.iter().any(|f| f.code == "R-EASE-INERT"),
        "analysis must still run and find R-EASE-INERT: {:?}",
        report.findings
    );
    assert_eq!(report.quiet, vec!["R-EASE-INERT".to_string()]);

    // 2. Attempting to quiet an error-class code (`E-TRACK-OVERLAP`) is rejected with `E-QUIET-FORBIDDEN`
    let forbidden_project = r##"{
  "frame": {"width": 1080, "height": 1920},
  "fps": 30,
  "quiet": ["E-TRACK-OVERLAP"],
  "tracks": []
}
"##;
    let forbidden_path = write_project(&dir, "forbidden_quiet.montagent.json", forbidden_project);
    let forbidden_report = validate(&forbidden_path);

    assert_eq!(forbidden_report.exit_code(), ExitCode::Errors);
    assert!(
        forbidden_report
            .findings
            .iter()
            .any(|f| f.code == "E-QUIET-FORBIDDEN"),
        "expected E-QUIET-FORBIDDEN, got: {:?}",
        forbidden_report.findings
    );
}

#[test]
fn wire_and_text_render_omit_quieted_reviews_report_quieted_count_in_summary_and_hint_on_collapsed_lines() {
    use montagent_core::finding::Finding;
    use montagent_core::registry::CheckSet;
    use montagent_core::report::Report;
    use montagent_core::{Wire, wire};
    use serde_json::json;

    let make_report = || {
        let mut report = Report::new("validate", Some("sizzle.montagent.json".into()));
        report.record(CheckSet::Document);
        report.record(CheckSet::Disk);
        for i in 0..4 {
            report.push(
                Finding::new("R-EASE-INERT")
                    .at_file("sizzle.montagent.json")
                    .at_track("main")
                    .at_element(format!("hold-{i}"))
                    .field("element", json!(format!("hold-{i}")))
                    .field("property", json!("x"))
                    .field("records", json!(2))
                    .field("from", json!(0))
                    .field("to", json!(1000))
                    .field("value", json!(100))
                    .field("ease", json!("ease-out")),
            );
        }
        report.push(
            Finding::new("R-CAPTION-NO-AUDIO")
                .at_file("sizzle.montagent.json")
                .at_track("captions")
                .at_element("cap-1")
                .field("start", json!(0))
                .field("end", json!(1000))
                .field("duration", json!(1000)),
        );
        report
    };

    // 1. Unquieted report with 4 R-EASE-INERT collapses with `--quiet R-EASE-INERT` hint
    let unquieted = make_report();
    let unquieted_text = wire::render(&unquieted, Wire::Text { verbose: false });
    assert!(
        unquieted_text.contains(
            "review  R-EASE-INERT  4 — expand with --verbose or quiet with --quiet R-EASE-INERT"
        ),
        "expected collapsed line with --quiet hint, got:\n{unquieted_text}"
    );

    // 2. Quieting R-EASE-INERT omits R-EASE-INERT from prose and JSON findings while keeping R-CAPTION-NO-AUDIO
    // and showing `5 reviews (4 quieted)` in the summary line and `review_quieted: 4` in JSON.
    let mut quieted = make_report();
    quieted.apply_quiet(&["R-EASE-INERT".to_string()], false, false);

    let quieted_text = wire::render(&quieted, Wire::Text { verbose: false });
    let first_line = quieted_text.lines().next().unwrap();
    assert!(
        first_line.contains("0 errors, 5 reviews (4 quieted), 0 notes"),
        "expected summary line to report (4 quieted), got: {first_line}"
    );
    assert!(
        !quieted_text.contains("R-EASE-INERT"),
        "quieted code R-EASE-INERT must not print in findings block:\n{quieted_text}"
    );
    assert!(
        quieted_text.contains("R-CAPTION-NO-AUDIO"),
        "unquieted review R-CAPTION-NO-AUDIO must still print:\n{quieted_text}"
    );

    let quieted_json: serde_json::Value =
        serde_json::from_str(&wire::render(&quieted, Wire::Json)).unwrap();
    assert_eq!(quieted_json["summary"]["review"], json!(5));
    assert_eq!(quieted_json["summary"]["review_quieted"], json!(4));
    let findings_arr = quieted_json["findings"].as_array().unwrap();
    assert_eq!(findings_arr.len(), 1);
    assert_eq!(findings_arr[0]["code"], json!("R-CAPTION-NO-AUDIO"));

    // 3. `--quiet-reviews` quiets all review codes
    let mut all_quiet = make_report();
    all_quiet.apply_quiet(&[], true, false);
    let all_quiet_text = wire::render(&all_quiet, Wire::Text { verbose: false });
    assert!(
        all_quiet_text
            .lines()
            .next()
            .unwrap()
            .contains("0 errors, 5 reviews (5 quieted), 0 notes"),
        "got:\n{all_quiet_text}"
    );
    assert!(!all_quiet_text.contains("R-EASE-INERT"));
    assert!(!all_quiet_text.contains("R-CAPTION-NO-AUDIO"));

    // 4. `--no-quiet` overrides project-level `quiet`
    let mut overridden = make_report();
    overridden.quiet = vec!["R-EASE-INERT".to_string()];
    overridden.apply_quiet(&[], false, true);
    let overridden_text = wire::render(&overridden, Wire::Text { verbose: false });
    assert!(
        !overridden_text.contains("quieted)"),
        "--no-quiet must not quiet any reviews:\n{overridden_text}"
    );
    assert!(
        overridden_text.contains("R-EASE-INERT"),
        "--no-quiet must restore R-EASE-INERT:\n{overridden_text}"
    );
}

