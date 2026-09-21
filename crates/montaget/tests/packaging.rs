//! #222's "no unsolicited network call" acceptance criterion, checked against the
//! *packaged binary itself* rather than against the report it prints.
//!
//! `montaget_core::media::probe::probe_remote` carries its own doc comment: *"The only
//! function in Montaget that can cause a network call."* The only way a run reaches it is
//! a `Source::Remote` entry the project document names itself (ADR-0056) — so a project
//! with zero remote sources should need no network device at all. `adapters.rs` already
//! checks the self-reported `network_attempts` counter is `0`; this test does not trust
//! that counter and instead removes the network namespace out from under the built
//! binary, on Linux, before it ever runs.

use std::path::PathBuf;
use std::process::Command;

#[test]
fn validate_succeeds_with_no_network_namespace_at_all() {
    if !cfg!(target_os = "linux") {
        eprintln!("skipping: network namespaces (`unshare --net`) are Linux-only");
        return;
    }
    if Command::new("unshare").arg("--help").output().is_err() {
        eprintln!("skipping: no `unshare` on this machine");
        return;
    }

    let cache = scratch("no-net-validate");
    let out = Command::new("unshare")
        .args(["--net", "--map-root-user", "--"])
        .arg(binary())
        .args(["validate", fixture().to_str().unwrap(), "--json"])
        .env(montaget_core::media::sidecar::CACHE_DIR_VAR, &cache)
        .output()
        .expect("run `unshare`");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    // `unshare --net --map-root-user` itself needs unprivileged user namespaces, which
    // some CI sandboxes and containers disable. That is a capability of *this machine*,
    // not a finding about the binary, so it is a skip rather than a failure — distinguished
    // from a real defect by whether `montaget` ever got to run at all: a project failure
    // still comes back as our own JSON on stdout, while `unshare` failing to set up the
    // namespace writes nothing there and complains on stderr instead.
    if !out.status.success() && stdout.trim().is_empty() {
        eprintln!("skipping: could not create a network namespace here:\n{stderr}");
        return;
    }

    assert!(
        out.status.success(),
        "validate did not answer with no network namespace:\n{stdout}{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout is not JSON ({e}):\n{stdout}"));
    // The project has no `Source::Remote` entries at all — every element in the fixture
    // resolves to a local path — so a clean report here means every local probe
    // succeeded with no network device present to fall back on.
    assert_eq!(json["summary"]["error"], 0, "{stdout}");
}

#[test]
fn probe_of_a_local_file_reports_zero_network_attempts_with_no_network_namespace() {
    if !cfg!(target_os = "linux") {
        eprintln!("skipping: network namespaces (`unshare --net`) are Linux-only");
        return;
    }
    if Command::new("unshare").arg("--help").output().is_err() {
        eprintln!("skipping: no `unshare` on this machine");
        return;
    }

    let cache = scratch("no-net-probe");
    let image = fixture()
        .parent()
        .expect("the fixture directory")
        .join("images/06.png");
    let out = Command::new("unshare")
        .args(["--net", "--map-root-user", "--"])
        .arg(binary())
        .args(["probe", image.to_str().unwrap(), "--json"])
        .env(montaget_core::media::sidecar::CACHE_DIR_VAR, &cache)
        .output()
        .expect("run `unshare`");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    if !out.status.success() && stdout.trim().is_empty() {
        eprintln!("skipping: could not create a network namespace here:\n{stderr}");
        return;
    }

    assert!(
        out.status.success(),
        "probe did not answer with no network namespace:\n{stdout}{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout is not JSON ({e}):\n{stdout}"));
    assert_eq!(json["network_attempts"], 0, "{stdout}");
}

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_montaget"))
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json")
        .canonicalize()
        .expect("the committed fixture")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("montaget-packaging-tests/{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}
