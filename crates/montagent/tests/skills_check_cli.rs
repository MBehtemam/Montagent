//! ADR-0244 / Issue #1141: CLI integration tests for `montagent skills check`.

use std::path::PathBuf;
use std::process::Command;

struct Output {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_montagent"))
}

fn montagent(args: &[&str]) -> Output {
    let out = Command::new(binary())
        .args(args)
        .output()
        .expect("run montagent");
    Output {
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

fn scratch_dir(name: &str) -> PathBuf {
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("montagent-skills-cli-tests/{}-{id}", name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn cli_skills_check_help_exposes_expected_subcommand_and_flags() {
    let out = montagent(&["skills", "check", "--help"]);
    assert_eq!(out.code, Some(0), "{}", out.stderr);
    assert!(out.stdout.contains("--json"));
    assert!(out.stdout.contains("--verbose"));
}

#[test]
fn cli_skills_check_alias_help_exposes_expected_flags() {
    let out = montagent(&["skills-check", "--help"]);
    assert_eq!(out.code, Some(0), "{}", out.stderr);
    assert!(out.stdout.contains("--json"));
    assert!(out.stdout.contains("--verbose"));
}

#[test]
fn cli_skills_check_clean_skill_returns_exit_0() {
    let dir = scratch_dir("clean");
    let skill_file = dir.join("SKILL.md");
    std::fs::write(
        &skill_file,
        r#"---
name: clean-skill
description: A clean test skill
---

Run validation: `montagent validate project.json --json`.
See `montagent://format.md`.
"#,
    )
    .unwrap();

    let out = montagent(&["skills", "check", dir.to_str().unwrap()]);
    assert_eq!(
        out.code,
        Some(0),
        "stdout: {}, stderr: {}",
        out.stdout,
        out.stderr
    );
}

#[test]
fn cli_skills_check_drift_returns_exit_1_and_reports_findings() {
    let dir = scratch_dir("drift");
    let skill_file = dir.join("SKILL.md");
    std::fs::write(
        &skill_file,
        r#"---
name: drift-skill
description: A drifting test skill
---

Run unknown flag: `montagent validate --unknown-flag-xyz project.json`.
"#,
    )
    .unwrap();

    let out = montagent(&["skills", "check", dir.to_str().unwrap()]);
    assert_eq!(
        out.code,
        Some(1),
        "stdout: {}, stderr: {}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("E-SKILL-FLAG-UNKNOWN"),
        "{}",
        out.stdout
    );
}

#[test]
fn cli_skills_check_json_returns_parseable_report() {
    let dir = scratch_dir("json");
    let skill_file = dir.join("SKILL.md");
    std::fs::write(
        &skill_file,
        r#"---
name: json-skill
description: A test skill for JSON output
---

Unknown code: `E-CODE-THAT-DOES-NOT-EXIST`.
"#,
    )
    .unwrap();

    let out = montagent(&["skills", "check", dir.to_str().unwrap(), "--json"]);
    assert_eq!(
        out.code,
        Some(1),
        "stdout: {}, stderr: {}",
        out.stdout,
        out.stderr
    );
    let parsed: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("invalid json: {e}, stdout: {}", out.stdout));
    assert_eq!(parsed["summary"]["error"], 1);
    assert_eq!(parsed["findings"][0]["code"], "E-SKILL-CODE-UNKNOWN");
}

#[test]
fn cli_skills_check_alias_works_identically() {
    let dir = scratch_dir("alias");
    let skill_file = dir.join("SKILL.md");
    std::fs::write(
        &skill_file,
        r#"---
name: alias-skill
description: A clean test skill via alias
---

Valid command: `montagent validate project.json`.
"#,
    )
    .unwrap();

    let out = montagent(&["skills-check", dir.to_str().unwrap()]);
    assert_eq!(
        out.code,
        Some(0),
        "stdout: {}, stderr: {}",
        out.stdout,
        out.stderr
    );
}

#[test]
fn cli_skills_check_nonexistent_path_returns_exit_3() {
    let out = montagent(&["skills", "check", "/path/that/does/not/exist/9999"]);
    assert_eq!(
        out.code,
        Some(3),
        "stdout: {}, stderr: {}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("E-INVOCATION") || out.stderr.contains("E-INVOCATION"),
        "stdout: {}, stderr: {}",
        out.stdout,
        out.stderr
    );
}
