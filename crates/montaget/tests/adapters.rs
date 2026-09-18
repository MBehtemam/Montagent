//! Adapter smoke tests, and deliberately nothing more.
//!
//! Spec #168: *"One test per CLI subcommand asserting argv reaches the right core call
//! and the exit code is right; one test per MCP tool asserting the schema advertised
//! matches the one enforced. No check logic is ever tested through an adapter — if a
//! test needs the CLI to reach a rule, the rule is in the wrong crate."*

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const HEADER_ONLY: &str = r##"{
  "frame": {"width": 1080, "height": 1920},
  "fps": 25,
  "background": "#FBF3E3",
  "duration": 65216,
  "output": "out/clean.mp4",
  "tracks": []
}
"##;

// ---- CLI ----------------------------------------------------------------------------

#[test]
fn cli_validate_on_a_header_only_project_is_a_clean_report_and_exit_0() {
    let project = scratch("cli-clean", "clean.montaget.json", HEADER_ONLY);
    let out = montaget(&["validate", project.to_str().unwrap()]);

    assert_eq!(out.code, Some(0));
    assert!(out.stdout.starts_with("0 errors"), "{}", out.stdout);
    assert!(out.stdout.contains("NOT CHECKED"), "{}", out.stdout);
}

#[test]
fn cli_validate_json_replaces_the_text_report_and_never_accompanies_it() {
    let project = scratch("cli-json", "clean.montaget.json", HEADER_ONLY);
    let out = montaget(&["validate", project.to_str().unwrap(), "--json"]);

    assert_eq!(out.code, Some(0));
    let json: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout is not JSON alone ({e}):\n{}", out.stdout));
    assert_eq!(json["summary"]["error"], 0);
    assert_eq!(json["exit_code"], 0);
}

#[test]
fn cli_validate_on_a_malformed_file_is_exit_2() {
    let project = scratch(
        "cli-broken",
        "broken.montaget.json",
        "{\n  \"frame\": {\"width\": 1080},\n  \"fps\": ,\n  \"tracks\": []\n}\n",
    );
    let out = montaget(&["validate", project.to_str().unwrap()]);

    assert_eq!(out.code, Some(2), "{}{}", out.stdout, out.stderr);
    assert!(out.stdout.contains("E-PARSE"), "{}", out.stdout);
}

#[test]
fn cli_fmt_check_reaches_the_verb_without_writing_and_exits_1() {
    // One test per subcommand, asserting argv reaches the right core call and the exit
    // code is right (#168). The convention itself is tested in the core.
    let pretty = "{\n  \"frame\": {\n    \"width\": 1080,\n    \"height\": 1920\n  },\n  \"fps\": 25,\n  \"tracks\": []\n}\n";
    let project = scratch("cli-fmt-check", "p.montaget.json", pretty);
    let out = montaget(&["fmt", project.to_str().unwrap(), "--check"]);

    assert_eq!(out.code, Some(1), "{}{}", out.stdout, out.stderr);
    assert!(out.stdout.contains("L-LAYOUT"), "{}", out.stdout);
    assert_eq!(std::fs::read_to_string(&project).unwrap(), pretty);
}

#[test]
fn cli_fmt_writes_and_exits_0() {
    let pretty = "{\n  \"frame\": {\n    \"width\": 1080,\n    \"height\": 1920\n  },\n  \"fps\": 25,\n  \"tracks\": []\n}\n";
    let project = scratch("cli-fmt-write", "p.montaget.json", pretty);
    let out = montaget(&["fmt", project.to_str().unwrap()]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    assert_eq!(
        std::fs::read_to_string(&project).unwrap(),
        "{\n  \"frame\": {\"width\": 1080, \"height\": 1920},\n  \"fps\": 25,\n  \"tracks\": []\n}\n"
    );
}

#[test]
fn cli_a_bad_invocation_is_exit_3_on_stderr() {
    let out = montaget(&["validate", "--nope"]);

    assert_eq!(out.code, Some(3));
    assert!(out.stderr.contains("E-INVOCATION"), "{}", out.stderr);
    assert!(
        out.stdout.is_empty(),
        "the report goes to stderr: {}",
        out.stdout
    );
}

#[test]
fn cli_a_source_that_grew_between_two_runs_is_announced_by_the_second() {
    // #231's criterion, at the only seam that can state it: two processes. Before the
    // sidecar (ADR-0069) the second run reported a `First` miss — "not yet in this
    // session's cache" — and the growth, the one defect class ADR-0011 says nothing else
    // can catch, went unsaid.
    let dir = scratch_dir("cli-grew-between-runs");
    let cache = dir.join("cache");
    let media = dir.join("take3.mp3");
    std::fs::copy(fixture_dir().join("audio/05-cobweb.mp3"), &media).expect("copy a real mp3");
    let source = media.to_str().unwrap().to_string();

    let first = montaget_caching(&["probe", &source], &cache);
    if first.code == Some(70) {
        eprintln!("skipping: {}", first.stderr.trim());
        return;
    }
    assert_eq!(first.code, Some(0), "{}{}", first.stdout, first.stderr);
    assert!(
        first.stdout.contains("not yet in the probe cache"),
        "the first run is a `First` miss:\n{}",
        first.stdout
    );

    // The file gains bytes — and, on this filesystem, a later mtime — between the runs.
    std::thread::sleep(std::time::Duration::from_millis(10));
    let mut grown = std::fs::read(&media).unwrap();
    grown.extend_from_slice(&std::fs::read(&media).unwrap());
    std::fs::write(&media, &grown).unwrap();

    let second = montaget_caching(&["probe", &source], &cache);
    assert_eq!(second.code, Some(0), "{}{}", second.stdout, second.stderr);
    assert!(
        second.stdout.contains("CHANGED ON DISK"),
        "the growth must be announced across the process boundary:\n{}",
        second.stdout
    );
    // Both halves of ADR-0006's key, both sides of the change.
    assert!(second.stdout.contains("bytes →"), "{}", second.stdout);
    assert!(second.stdout.contains("mtime "), "{}", second.stdout);

    // And a third run, with nothing changed, is a hit: no CACHE block at all.
    let third = montaget_caching(&["probe", &source], &cache);
    assert_eq!(third.code, Some(0), "{}{}", third.stdout, third.stderr);
    assert!(
        !third.stdout.contains("CACHE"),
        "an unchanged file is not a miss:\n{}",
        third.stdout
    );
}

#[test]
fn cli_the_probe_cache_never_lands_beside_the_project() {
    // ADR-0069: the sidecar lives under the per-user cache directory, so there is nothing
    // beside a project to commit by accident and ADR-0053's movable unit stays as it was.
    let dir = scratch_dir("cli-no-sidecar-beside-the-project");
    let cache = dir.join("cache");
    let media = dir.join("take3.mp3");
    std::fs::copy(fixture_dir().join("audio/05-cobweb.mp3"), &media).expect("copy a real mp3");

    let out = montaget_caching(&["probe", media.to_str().unwrap()], &cache);
    if out.code == Some(70) {
        eprintln!("skipping: {}", out.stderr.trim());
        return;
    }

    let beside: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name != "cache")
        .collect();
    assert_eq!(
        beside,
        vec!["take3.mp3".to_string()],
        "the run left something beside the media"
    );
    assert!(
        cache.join("probe-cache.json").exists(),
        "and wrote it here instead"
    );
}

#[test]
fn cli_probe_reaches_the_probe_verb_and_exits_0() {
    let fixture = fixture_dir();
    let out = montaget(&["probe", fixture.join("images/06.png").to_str().unwrap()]);

    if out.code == Some(70) {
        eprintln!("skipping: {}", out.stderr.trim());
        return;
    }
    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    assert!(out.stdout.contains("1536×2720"), "{}", out.stdout);
    // ADR-0006 puts the cache line at the top of the report, above the facts, and `probe`
    // answers with the same report shape every other verb does.
    let cache = out.stdout.find("CACHE").expect(&out.stdout);
    let media = out.stdout.find("MEDIA").expect(&out.stdout);
    assert!(cache < media, "{}", out.stdout);
}

#[test]
fn cli_probe_json_replaces_the_text_report_and_never_accompanies_it() {
    let fixture = fixture_dir();
    let out = montaget(&[
        "probe",
        fixture.join("images/06.png").to_str().unwrap(),
        "--json",
    ]);

    if out.code == Some(70) {
        eprintln!("skipping: {}", out.stderr.trim());
        return;
    }
    let json: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout is not JSON alone ({e}):\n{}", out.stdout));
    assert_eq!(json["media"][0]["dimensions"]["width"], 1536);
    assert_eq!(json["network_attempts"], 0);
}

#[test]
fn cli_probe_without_an_ffmpeg_on_path_is_exit_70() {
    // ADR-0011's exit 70: "internal failure (ffmpeg died, font stack failed) → retry or
    // report". The `PATH` is emptied for this child alone.
    let empty = scratch_dir("cli-no-ffmpeg");
    let out = Command::new(binary())
        .args(["probe", "anything.mp4"])
        .env("PATH", &empty)
        .output()
        .expect("run montaget");

    assert_eq!(out.status.code(), Some(70));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("ffmpeg"), "{stderr}");
    assert!(
        stderr.contains(&empty.display().to_string()),
        "the search is the actionable half: {stderr}"
    );
}

#[cfg(unix)]
#[test]
fn cli_probe_with_an_ffprobe_too_old_for_our_flags_is_exit_70_not_a_media_finding() {
    // ADR-0009 ships "a binary, plus an `ffmpeg` the user supplies", so the supplied one
    // may be older than the flags Montaget passes. Driven through a real executable on a
    // real `PATH` rather than a hand-built value, because the claim under test is that
    // some code path actually reaches exit 70 — not that the type can represent it.
    let dir = scratch_dir("cli-old-ffprobe");
    for program in ["ffprobe", "ffmpeg"] {
        let path = dir.join(program);
        std::fs::write(
            &path,
            "#!/bin/sh\necho \"Unrecognized option 'protocol_whitelist'.\" >&2\nexit 1\n",
        )
        .expect("write the stub");
        let mut mode = std::fs::metadata(&path).unwrap().permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut mode, 0o755);
        std::fs::set_permissions(&path, mode).expect("make the stub executable");
    }

    let fixture = fixture_dir();
    let out = Command::new(binary())
        .args(["probe", fixture.join("images/06.png").to_str().unwrap()])
        .env("PATH", &dir)
        // A cold cache, so the stub is actually reached: a warm sidecar (ADR-0069) would
        // answer from the last run and the broken tool would never be asked anything.
        .env(
            montaget_core::media::sidecar::CACHE_DIR_VAR,
            dir.join("cache"),
        )
        .output()
        .expect("run montaget");

    assert_eq!(
        out.status.code(),
        Some(70),
        "a broken tool is exit 70, never exit 0 with the media blamed: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("protocol_whitelist"), "{stderr}");
    assert!(
        stderr.contains(dir.join("ffprobe").to_str().unwrap()),
        "exit 70 names the resolved path, which is the thing to go and look at: {stderr}"
    );
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains("U-SOURCE-UNPROBEABLE"),
        "the media is not the thing that failed"
    );
}

#[test]
fn cli_validate_offers_no_flag_that_narrows_the_disk_checks() {
    // ADR-0006, unanimous 5 of 5: "No fast mode. No `--no-probe`. No scoping of what is
    // checked." The reason is stated as a prediction about people: "the moment a fast path
    // exists it becomes the mode used in the edit loop, so the single highest-value check
    // in the tool surface is the one that gets skipped." A flag is the easiest thing in the
    // world to add later, so the absence is asserted rather than assumed.
    let out = montaget(&["validate", "--help"]);

    assert_eq!(out.code, Some(0));
    for forbidden in [
        "--no-probe",
        "--fast",
        "--skip",
        "--only",
        "--scope",
        "--group",
        "--no-disk",
        "--offline",
    ] {
        assert!(
            !out.stdout.contains(forbidden),
            "`{forbidden}` would be a way to not run the check that exists to catch the \
             defect that changed on disk rather than in the project:\n{}",
            out.stdout
        );
    }
    // The two that do exist are about the *output*, which ADR-0006 explicitly permits
    // scoping — "a checked-but-unprinted finding still exists; an unchecked one silently
    // does not".
    assert!(out.stdout.contains("--json"), "{}", out.stdout);
    assert!(out.stdout.contains("--verbose"), "{}", out.stdout);
}

#[test]
fn cli_validate_that_loses_its_ffprobe_keeps_what_it_had_already_learned() {
    // A run reaches the disk half having already read the document. With no `ffprobe` it
    // has learned two things — the retired key, and that it cannot look at the media — and
    // a report carrying only the second would send an agent off to fix its `PATH` and
    // re-run before hearing about the key it could have fixed in the same turn. Exit 70
    // still, because the run did not finish (ADR-0011).
    //
    // Driven as a child process because `PATH` is process-wide: setting it in-process
    // would break every sibling test that runs at the same time.
    let dir = scratch_dir("cli-lost-ffprobe");
    let project = dir.join("p.montaget.json");
    std::fs::write(
        &project,
        r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"photos","layer":1,"elements":[{"id":"photo-06","type":"image","start":0,"end":1000,"source":"images/05.png","x":0,"y":0,"origin":"top-left","width":1080,"height":1912,"fit":"cover","gravity":"bottom"}]}]}"##,
    )
    .expect("write project");
    let empty = scratch_dir("cli-lost-ffprobe-path");

    let out = Command::new(binary())
        .args(["validate", project.to_str().unwrap()])
        .env("PATH", &empty)
        .output()
        .expect("run montaget");

    assert_eq!(out.status.code(), Some(70));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("E-RETIRED-KEY"),
        "what the document half learned survives: {stdout}"
    );
    assert!(stdout.contains("E-INTERNAL"), "{stdout}");
    assert!(stdout.contains("ffmpeg"), "{stdout}");
}

#[test]
fn cli_help_is_not_a_failure() {
    let out = montaget(&["--help"]);
    assert_eq!(out.code, Some(0));
    assert!(out.stdout.contains("validate"), "{}", out.stdout);
}

// ---- MCP ----------------------------------------------------------------------------

#[test]
fn mcp_validate_on_a_header_only_project_is_a_clean_report() {
    let project = scratch("mcp-clean", "clean.montaget.json", HEADER_ONLY);
    let session = mcp_session(&[
        request(
            1,
            "initialize",
            serde_json::json!({
                "protocolVersion": "2026-07-28",
                "capabilities": {},
                "clientInfo": {"name": "montaget-tests", "version": "0"}
            }),
        ),
        notification("notifications/initialized"),
        request(2, "tools/list", serde_json::json!({})),
        request(
            3,
            "tools/call",
            serde_json::json!({
                "name": "validate",
                "arguments": {"project": project.to_str().unwrap()}
            }),
        ),
    ]);

    let call = session.get(&3).expect("a result for tools/call");
    let text = call["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.starts_with("0 errors"), "{text}");
    assert!(text.contains("NOT CHECKED"), "{text}");
    assert_eq!(call["result"]["isError"], false);
}

#[test]
fn mcp_does_not_advertise_probe() {
    // ADR-0011: `probe` is CLI-only, and the asymmetry is the point. "Every MCP tool
    // schema occupies the agent's context and degrades tool selection on every turn,
    // including turns with nothing to do with video. A CLI subcommand costs nothing until
    // invoked." Asserted here because the cost is paid on turns this suite cannot see.
    let session = mcp_session(&[
        request(
            1,
            "initialize",
            serde_json::json!({
                "protocolVersion": "2026-07-28",
                "capabilities": {},
                "clientInfo": {"name": "montaget-tests", "version": "0"}
            }),
        ),
        notification("notifications/initialized"),
        request(2, "tools/list", serde_json::json!({})),
    ]);

    let tools = session.get(&2).expect("a result for tools/list")["result"]["tools"]
        .as_array()
        .expect("a tool list")
        .iter()
        .map(|tool| tool["name"].as_str().unwrap_or("?").to_string())
        .collect::<Vec<_>>();

    assert!(tools.iter().any(|name| name == "validate"), "{tools:?}");
    assert!(!tools.iter().any(|name| name == "probe"), "{tools:?}");
}

#[test]
fn mcp_validate_advertises_the_schema_it_enforces() {
    let project = scratch("mcp-schema", "clean.montaget.json", HEADER_ONLY);
    let session = mcp_session(&[
        request(
            1,
            "initialize",
            serde_json::json!({
                "protocolVersion": "2026-07-28",
                "capabilities": {},
                "clientInfo": {"name": "montaget-tests", "version": "0"}
            }),
        ),
        notification("notifications/initialized"),
        request(2, "tools/list", serde_json::json!({})),
        // The schema says `project` is required and the other two are optional; calling
        // with `project` alone must therefore be accepted.
        request(
            3,
            "tools/call",
            serde_json::json!({
                "name": "validate",
                "arguments": {"project": project.to_str().unwrap()}
            }),
        ),
        // …and a call missing the one required property must be rejected rather than
        // silently validating something else.
        request(
            4,
            "tools/call",
            serde_json::json!({
                "name": "validate",
                "arguments": {}
            }),
        ),
    ]);

    let tools = &session[&2]["result"]["tools"];
    let validate = tools
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "validate")
        .expect("a `validate` tool");

    let schema = &validate["inputSchema"];
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["required"], serde_json::json!(["project"]));
    for property in ["project", "json", "verbose"] {
        assert!(
            !schema["properties"][property].is_null(),
            "the schema advertises `{property}`"
        );
    }

    assert_eq!(session[&3]["result"]["isError"], false);
    let rejected = &session[&4];
    assert!(
        rejected.get("error").is_some() || rejected["result"]["isError"] == true,
        "a call missing `project` must be rejected: {rejected}"
    );
}

#[test]
fn a_validate_that_ran_and_found_errors_is_not_a_tool_failure() {
    // ADR-0006: the findings *are* the result. An `error` finding means the project is
    // wrong, not that the call failed, and a client that retries on `isError` must not
    // be told to retry a correct answer.
    let project = scratch("mcp-errors", "broken.montaget.json", "{\n  \"fps\": ,\n}\n");
    let session = mcp_session(&[
        request(
            1,
            "initialize",
            serde_json::json!({
                "protocolVersion": "2026-07-28",
                "capabilities": {},
                "clientInfo": {"name": "montaget-tests", "version": "0"}
            }),
        ),
        notification("notifications/initialized"),
        request(
            2,
            "tools/call",
            serde_json::json!({
                "name": "validate",
                "arguments": {"project": project.to_str().unwrap()}
            }),
        ),
    ]);

    let call = &session[&2];
    assert_eq!(call["result"]["isError"], false, "{call}");
    assert!(
        call["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("E-PARSE")
    );
}

#[test]
fn mcp_rejects_a_bad_call_with_a_finding_like_every_other_surface() {
    // ADR-0011: "An error is a finding. Same objects and same stable codes as ADR-0006,
    // including for invocation errors, so there is exactly one thing to parse across the
    // surface." Asserting only that a bad call is *rejected* is what let the SDK's own
    // raw deserialisation message through here while the CLI answered with a finding.
    let session = mcp_session(&[
        request(
            1,
            "initialize",
            serde_json::json!({
                "protocolVersion": "2026-07-28",
                "capabilities": {},
                "clientInfo": {"name": "montaget-tests", "version": "0"}
            }),
        ),
        notification("notifications/initialized"),
        request(
            2,
            "tools/call",
            serde_json::json!({
                "name": "validate",
                "arguments": {"project": 123}
            }),
        ),
    ]);

    let call = &session[&2];
    let text = call["result"]["content"][0]["text"]
        .as_str()
        .expect("a rendered report, not a bare SDK error");
    assert!(text.contains("E-INVOCATION"), "{text}");
    assert!(text.contains("NOT CHECKED"), "{text}");
    assert_eq!(
        call["result"]["isError"], true,
        "the client is also told the call did not run: {call}"
    );
}

#[test]
fn mcp_validate_json_returns_the_canonical_json_instead_of_the_text() {
    let project = scratch("mcp-json", "clean.montaget.json", HEADER_ONLY);
    let session = mcp_session(&[
        request(
            1,
            "initialize",
            serde_json::json!({
                "protocolVersion": "2026-07-28",
                "capabilities": {},
                "clientInfo": {"name": "montaget-tests", "version": "0"}
            }),
        ),
        notification("notifications/initialized"),
        request(
            2,
            "tools/call",
            serde_json::json!({
                "name": "validate",
                "arguments": {"project": project.to_str().unwrap(), "json": true}
            }),
        ),
    ]);

    let text = session[&2]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    let json: serde_json::Value = serde_json::from_str(text).expect("the canonical JSON alone");
    assert_eq!(json["exit_code"], 0);
    assert!(
        !text.contains("NOT CHECKED\n  This file"),
        "not both forms: {text}"
    );
}

// ---- harness ------------------------------------------------------------------------

struct Output {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn binary() -> PathBuf {
    // `CARGO_BIN_EXE_<name>` is set by cargo for integration tests of a binary crate.
    PathBuf::from(env!("CARGO_BIN_EXE_montaget"))
}

/// One `montaget` run, against a probe cache no other run shares.
///
/// Cold by construction: the sidecar (ADR-0069) is real, and a test that asserted a `CACHE`
/// block would otherwise pass once and then never again on the same machine.
fn montaget(args: &[&str]) -> Output {
    static RUNS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    montaget_caching(
        args,
        &scratch_dir(&format!("cache/{}-{n}", std::process::id())),
    )
}

/// The same, against a cache directory the caller owns — which is how two runs share one,
/// and the only way to ask whether anything survived between them.
fn montaget_caching(args: &[&str], cache: &Path) -> Output {
    let out = Command::new(binary())
        .args(args)
        .env(montaget_core::media::sidecar::CACHE_DIR_VAR, cache)
        .output()
        .expect("run montaget");
    Output {
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

fn request(id: u64, method: &str, params: serde_json::Value) -> String {
    serde_json::json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string()
}

fn notification(method: &str) -> String {
    serde_json::json!({"jsonrpc": "2.0", "method": method}).to_string()
}

/// Drive the stdio server through one session, returning every response by its id.
fn mcp_session(messages: &[String]) -> std::collections::BTreeMap<u64, serde_json::Value> {
    let expected = messages.iter().filter(|m| m.contains("\"id\":")).count();

    let mut child = Command::new(binary())
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn the MCP server");

    let mut stdin = child.stdin.take().expect("stdin");
    for message in messages {
        writeln!(stdin, "{message}").expect("write a request");
    }
    stdin.flush().expect("flush");

    let mut responses = std::collections::BTreeMap::new();
    let reader = BufReader::new(child.stdout.take().expect("stdout"));
    for line in reader.lines() {
        let line = line.expect("read a response");
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(&line)
            .unwrap_or_else(|e| panic!("response is not JSON ({e}): {line}"));
        if let Some(id) = value["id"].as_u64() {
            responses.insert(id, value);
        }
        if responses.len() == expected {
            break;
        }
    }

    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();

    assert_eq!(responses.len(), expected, "every request got a response");
    responses
}

/// The committed fixture's own directory — the only media this suite probes.
fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating")
        .canonicalize()
        .expect("the committed fixture")
}

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("montaget-adapter-tests/{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn scratch(name: &str, file: &str, body: &str) -> PathBuf {
    let path = scratch_dir(name).join(file);
    std::fs::write(&path, body).expect("write project");
    assert!(Path::new(&path).exists());
    path
}
