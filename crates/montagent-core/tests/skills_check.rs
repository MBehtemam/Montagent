//! ADR-0244 / Issue #1141: montagent skills check verification engine tests.

use montagent_core::report::ExitCode;
use montagent_core::skills_check::{CheckOptions, check};

mod common;
use common::tempdir;

#[test]
fn skills_check_detects_clean_skill_dir() {
    let dir = tempdir(line!());
    let skill_dir = dir.join("clean_skill");
    std::fs::create_dir_all(&skill_dir).unwrap();

    let content = r##"# Clean Skill

Use `montagent validate` and `montagent preview --output test.mp4`.
Reported findings may include `E-PARSE`.
See `montagent://format.md` for format specifications.

```json
{
  "frame": {"width": 1920, "height": 1080},
  "fps": 30,
  "duration": 1000,
  "output": "out.mp4",
  "tracks": [
    {
      "name": "v1",
      "layer": 1,
      "elements": [
        {
          "id": "rect1",
          "type": "rect",
          "start": 0,
          "end": 1000,
          "width": 100,
          "height": 100,
          "fill": "#FFFFFF"
        }
      ]
    }
  ]
}
```
"##;
    std::fs::write(skill_dir.join("SKILL.md"), content).unwrap();

    let options = CheckOptions {
        target: Some(skill_dir),
        root: Some(dir),
    };
    let report = check(&options);

    assert_eq!(report.summary().error, 0, "findings: {:?}", report.findings);
    assert_eq!(report.exit_code(), ExitCode::Ok);
}

#[test]
fn skills_check_reports_unknown_command_and_flag() {
    let dir = tempdir(line!());
    let skill_dir = dir.join("bad_cmd_skill");
    std::fs::create_dir_all(&skill_dir).unwrap();

    let content = r##"# Bad Command Skill

Run `montagent nonexistent_verb` to do something.
Or run `montagent create-project --out invalid.json`.
"##;
    std::fs::write(skill_dir.join("SKILL.md"), content).unwrap();

    let options = CheckOptions {
        target: Some(skill_dir),
        root: Some(dir),
    };
    let report = check(&options);

    assert_eq!(report.exit_code(), ExitCode::Errors);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-SKILL-COMMAND-UNKNOWN"),
        "expected E-SKILL-COMMAND-UNKNOWN, got: {:?}",
        report.findings
    );
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-SKILL-FLAG-UNKNOWN"),
        "expected E-SKILL-FLAG-UNKNOWN, got: {:?}",
        report.findings
    );
}

#[test]
fn skills_check_reports_unknown_finding_code() {
    let dir = tempdir(line!());
    let skill_dir = dir.join("bad_code_skill");
    std::fs::create_dir_all(&skill_dir).unwrap();

    let content = r##"# Bad Code Skill

This might emit `E-NONEXISTENT-CODE-12345`.
"##;
    std::fs::write(skill_dir.join("SKILL.md"), content).unwrap();

    let options = CheckOptions {
        target: Some(skill_dir),
        root: Some(dir),
    };
    let report = check(&options);

    assert_eq!(report.exit_code(), ExitCode::Errors);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-SKILL-CODE-UNKNOWN"),
        "expected E-SKILL-CODE-UNKNOWN, got: {:?}",
        report.findings
    );
}

#[test]
fn skills_check_reports_unknown_resource_uri() {
    let dir = tempdir(line!());
    let skill_dir = dir.join("bad_uri_skill");
    std::fs::create_dir_all(&skill_dir).unwrap();

    let content = r##"# Bad URI Skill

Read `montagent://nonexistent_page.md` for more info.
"##;
    std::fs::write(skill_dir.join("SKILL.md"), content).unwrap();

    let options = CheckOptions {
        target: Some(skill_dir),
        root: Some(dir),
    };
    let report = check(&options);

    assert_eq!(report.exit_code(), ExitCode::Errors);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-SKILL-RESOURCE-UNKNOWN"),
        "expected E-SKILL-RESOURCE-UNKNOWN, got: {:?}",
        report.findings
    );
}

#[test]
fn skills_check_reports_invalid_snippet() {
    let dir = tempdir(line!());
    let skill_dir = dir.join("bad_snippet_skill");
    std::fs::create_dir_all(&skill_dir).unwrap();

    let content = r##"# Bad Snippet Skill

Here is invalid project JSON:
```json
{
  "fps": 30
}
```
"##;
    std::fs::write(skill_dir.join("SKILL.md"), content).unwrap();

    let options = CheckOptions {
        target: Some(skill_dir),
        root: Some(dir),
    };
    let report = check(&options);

    assert_eq!(report.exit_code(), ExitCode::Errors);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-SKILL-SNIPPET-INVALID"),
        "expected E-SKILL-SNIPPET-INVALID, got: {:?}",
        report.findings
    );
}

#[test]
fn skills_check_missing_directory_returns_invocation_error() {
    let dir = tempdir(line!());
    let missing_path = dir.join("does_not_exist");

    let options = CheckOptions {
        target: Some(missing_path),
        root: Some(dir),
    };
    let report = check(&options);

    assert_eq!(report.exit_code(), ExitCode::BadInvocation);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-INVOCATION"),
        "expected E-INVOCATION, got: {:?}",
        report.findings
    );
}
