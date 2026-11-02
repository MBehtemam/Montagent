//! Issue #1138 / #1142: CLI integration tests for `montagent validate` review filtering (`--quiet`, `--quiet-reviews`, `--no-quiet`).

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
    let dir = std::env::temp_dir().join(format!("montagent-quiet-cli-tests/{}-{id}", name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn cli_validate_supports_quiet_flags_and_no_quiet_and_rejects_quieting_error_codes() {
    let dir = scratch_dir("cli-quiet");
    let project_path = dir.join("hold.montagent.json");
    std::fs::write(
        &project_path,
        r##"{
  "frame": {"width": 1080, "height": 1920},
  "fps": 30,
  "background": "#000000",
  "duration": 2000,
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
"##,
    )
    .unwrap();
    let project_str = project_path.to_str().unwrap();

    // 1. `--quiet R-EASE-INERT` suppresses detailed review and reports `(1 quieted)`
    let out = montagent(&["validate", project_str, "--quiet", "R-EASE-INERT"]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    assert!(
        out.stdout.contains("1 review (1 quieted)"),
        "expected `(1 quieted)` in summary, got:\n{}",
        out.stdout
    );
    assert!(
        !out.stdout.contains("R-EASE-INERT"),
        "expected R-EASE-INERT to be omitted from body:\n{}",
        out.stdout
    );

    // 2. `--quiet E-TRACK-OVERLAP` fails with exit 1 and `E-QUIET-FORBIDDEN`
    let forbidden = montagent(&["validate", project_str, "--quiet", "E-TRACK-OVERLAP"]);
    assert_eq!(forbidden.code, Some(1), "stderr: {}", forbidden.stderr);
    assert!(
        forbidden.stdout.contains("E-QUIET-FORBIDDEN"),
        "expected E-QUIET-FORBIDDEN, got:\n{}",
        forbidden.stdout
    );

    // 3. In-project `"quiet": ["R-EASE-INERT"]` is overridden by `--no-quiet`
    let in_project_path = dir.join("in_project.montagent.json");
    std::fs::write(
        &in_project_path,
        r##"{
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
"##,
    )
    .unwrap();
    let in_project_str = in_project_path.to_str().unwrap();

    let default_out = montagent(&["validate", in_project_str]);
    assert_eq!(default_out.code, Some(0));
    assert!(default_out.stdout.contains("1 review (1 quieted)"));
    assert!(!default_out.stdout.contains("R-EASE-INERT"));

    let no_quiet_out = montagent(&["validate", in_project_str, "--no-quiet"]);
    assert_eq!(no_quiet_out.code, Some(0));
    assert!(!no_quiet_out.stdout.contains("quieted)"));
    assert!(no_quiet_out.stdout.contains("R-EASE-INERT"));
}
