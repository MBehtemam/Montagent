//! ADR-0108's acceptance (#439): the stdio server keeps answering while it encodes, and a
//! long call is heard from until it ends.
//!
//! Before ADR-0108 every tool ran inside the poll of a `current_thread` runtime, so a
//! `frame` issued behind a `render` was not even *read* until the render returned, and the
//! render itself said nothing on the wire. Claude Code aborts a stdio call that sends *"no
//! response and no progress notification"* for 1800 s — MONTAGENT-9.
//!
//! The heartbeat is shortened through `MONTAGENT_MCP_HEARTBEAT_MS` so *"progress at least
//! every N s"* is provable in a test's run time; the 30 s production value is the same code
//! path with a different constant.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

/// The shortened heartbeat, and the gap the test tolerates. Generous against a loaded CI
/// machine: what it must rule out is *silence for the whole render*, which is seconds.
const HEARTBEAT_MS: u64 = 150;
const TOLERATED_GAP: Duration = Duration::from_millis(1500);

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_montagent"))
}

fn has_ffmpeg() -> bool {
    montagent_core::media::tools::resolve().is_ok()
}

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("montagent-mcp-concurrency/{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// A project that takes a few seconds to encode: enough frames that a `frame` issued just
/// after it has plenty of time to come back first, if the server lets it.
fn slow_project(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(format!("{name}.montagent.json"));
    std::fs::write(
        &path,
        format!(
            r##"{{"frame":{{"width":640,"height":360}},"fps":25,"background":"#000000",
  "duration":4000,"output":"out/{name}.mp4",
  "tracks":[{{"name":"only","layer":0,"elements":[
    {{"id":"card","type":"rect","start":0,"end":4000,"x":320,"y":180,"width":200,
      "height":100,"fill":"#FF0000"}}]}}]}}
"##
        ),
    )
    .expect("write project");
    path
}

/// One line the server wrote to stdout, and when it arrived.
struct Line {
    at: Instant,
    value: Value,
}

/// A running `montagent mcp`, read on its own thread so arrival times are real.
struct Server {
    child: Child,
    stdin: std::process::ChildStdin,
    lines: mpsc::Receiver<Line>,
    seen: Vec<Line>,
    stderr: PathBuf,
}

impl Server {
    fn start(dir: &Path) -> Server {
        let stderr = dir.join("server.stderr");
        let mut child = Command::new(binary())
            .arg("mcp")
            .env("MONTAGENT_MCP_HEARTBEAT_MS", HEARTBEAT_MS.to_string())
            .env("MONTAGENT_CACHE_DIR", dir.join("cache"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(std::fs::File::create(&stderr).expect("stderr file"))
            .spawn()
            .expect("spawn the MCP server");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = child.stdout.take().expect("stdout");
        let (tx, lines) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if line.trim().is_empty() {
                    continue;
                }
                let value = serde_json::from_str(&line)
                    .unwrap_or_else(|e| panic!("stdout is not JSON ({e}): {line}"));
                if tx
                    .send(Line {
                        at: Instant::now(),
                        value,
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        let mut server = Server {
            child,
            stdin,
            lines,
            seen: Vec::new(),
            stderr,
        };
        server.send(
            json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{
            "protocolVersion":"2025-06-18","capabilities":{},
            "clientInfo":{"name":"mcp-concurrency-test","version":"0"}}}),
        );
        server.wait_for(0);
        server.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
        server
    }

    fn send(&mut self, message: Value) -> Instant {
        writeln!(self.stdin, "{message}").expect("write a message");
        self.stdin.flush().expect("flush");
        Instant::now()
    }

    fn call(&mut self, id: u64, tool: &str, arguments: Value, token: Option<&str>) -> Instant {
        let mut params = json!({"name": tool, "arguments": arguments});
        if let Some(token) = token {
            params["_meta"] = json!({"progressToken": token});
        }
        self.send(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":params}))
    }

    /// Read until the response with this id has arrived, keeping everything seen.
    fn wait_for(&mut self, id: u64) -> Instant {
        loop {
            if let Some(line) = self.seen.iter().find(|l| l.value["id"] == json!(id)) {
                return line.at;
            }
            let line = self
                .lines
                .recv_timeout(Duration::from_secs(120))
                .unwrap_or_else(|_| panic!("no response for id {id} within 120 s"));
            self.seen.push(line);
        }
    }

    fn response(&self, id: u64) -> &Value {
        &self
            .seen
            .iter()
            .find(|l| l.value["id"] == json!(id))
            .expect("the response")
            .value
    }

    /// Every progress notification for one token, in arrival order.
    fn progress(&self, token: &str) -> Vec<&Line> {
        self.seen
            .iter()
            .filter(|l| {
                l.value["method"] == "notifications/progress"
                    && l.value["params"]["progressToken"] == token
            })
            .collect()
    }

    fn stderr(&self) -> String {
        std::fs::read_to_string(&self.stderr).unwrap_or_default()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn text_of(response: &Value) -> String {
    response["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

#[test]
fn a_frame_issued_behind_a_render_is_answered_while_the_render_is_still_encoding() {
    if !has_ffmpeg() {
        eprintln!("skipping: no ffmpeg/ffprobe on PATH");
        return;
    }
    let dir = scratch_dir("frame-behind-render");
    let project = slow_project(&dir, "slow");
    let mut server = Server::start(&dir);

    let project = project.display().to_string();
    let dispatched = server.call(2, "render", json!({"project": project}), Some("r"));
    server.call(3, "frame", json!({"project": project, "at": 1000}), None);

    let frame_at = server.wait_for(3);
    let render_at = server.wait_for(2);

    // The acceptance itself: the `frame` did not queue behind the encode.
    assert!(
        frame_at < render_at,
        "`frame` came back {:?} after the render finished",
        frame_at - render_at
    );
    assert!(
        text_of(server.response(2)).contains("RENDER"),
        "the render did not succeed, so the ordering proves nothing: {}",
        server.response(2)
    );

    // Progress at least every TOLERATED_GAP from dispatch to result.
    let progress = server.progress("r");
    assert!(!progress.is_empty(), "no progress was sent for the render");
    let mut previous = dispatched;
    for line in progress.iter().map(|l| l.at).chain([render_at]) {
        assert!(
            line.duration_since(previous) <= TOLERATED_GAP,
            "{:?} of silence during the render (heartbeat {HEARTBEAT_MS} ms)",
            line.duration_since(previous)
        );
        previous = line;
    }

    // The spec's rule: `progress` strictly increases, and `total` is never exceeded.
    let values: Vec<f64> = progress
        .iter()
        .map(|l| l.value["params"]["progress"].as_f64().expect("a number"))
        .collect();
    assert!(
        values.windows(2).all(|w| w[1] > w[0]),
        "progress did not strictly increase: {values:?}"
    );
    for line in &progress {
        if let Some(total) = line.value["params"]["total"].as_f64() {
            assert!(line.value["params"]["progress"].as_f64().unwrap() <= total);
        }
    }

    // The stderr record research §8 asked for: whether a token came, and when each call
    // was dispatched, started and finished.
    let stderr = server.stderr();
    assert!(
        stderr.contains("render  dispatched  progressToken: present"),
        "{stderr}"
    );
    assert!(
        stderr.contains("frame  dispatched  progressToken: absent"),
        "{stderr}"
    );
    assert!(stderr.contains("render  finished"), "{stderr}");
    assert!(stderr.contains("render  0/100 frames"), "{stderr}");
}

#[test]
fn a_second_render_waits_for_the_encode_slot_and_says_what_it_waits_behind() {
    if !has_ffmpeg() {
        eprintln!("skipping: no ffmpeg/ffprobe on PATH");
        return;
    }
    let dir = scratch_dir("render-behind-render");
    let first = slow_project(&dir, "first").display().to_string();
    let second = slow_project(&dir, "second").display().to_string();
    let mut server = Server::start(&dir);

    server.call(2, "render", json!({"project": first}), Some("a"));
    // Let the first call take the slot before the second arrives.
    std::thread::sleep(Duration::from_millis(300));
    server.call(3, "render", json!({"project": second}), Some("b"));
    server.wait_for(2);
    server.wait_for(3);

    let queued: Vec<String> = server
        .progress("b")
        .iter()
        .filter_map(|l| l.value["params"]["message"].as_str().map(str::to_string))
        .collect();
    assert!(
        queued
            .iter()
            .any(|m| m.contains("queued behind render of") && m.contains("first")),
        "the waiting render never said what it was waiting behind: {queued:?}"
    );
    // Both deliverables, each once: the slot serialised them rather than dropping one.
    assert!(dir.join("out/first.mp4").is_file());
    assert!(dir.join("out/second.mp4").is_file());
}

#[test]
fn a_call_without_a_progress_token_gets_no_notifications_and_still_its_result() {
    if !has_ffmpeg() {
        eprintln!("skipping: no ffmpeg/ffprobe on PATH");
        return;
    }
    let dir = scratch_dir("no-token");
    let project = slow_project(&dir, "slow").display().to_string();
    let mut server = Server::start(&dir);

    server.call(2, "render", json!({"project": project}), None);
    server.wait_for(2);
    assert!(
        server
            .seen
            .iter()
            .all(|l| l.value["method"] != "notifications/progress"),
        "a notification was sent against a token the client never supplied"
    );
    assert!(text_of(server.response(2)).contains("RENDER"));
}

// ---------------------------------------------------------------------------
// ADR-0109 (#440): a cancelled call withholds, and frees the slot
// ---------------------------------------------------------------------------

fn write_long_or_short(path: &Path, name: &str, duration: i64, fill: &str) {
    std::fs::write(
        path,
        format!(
            r##"{{"frame":{{"width":640,"height":360}},"fps":25,"background":"#000000",
  "duration":{duration},"output":"out/{name}.mp4",
  "tracks":[{{"name":"only","layer":0,"elements":[
    {{"id":"card","type":"rect","start":0,"end":{duration},"x":320,"y":180,"width":200,
      "height":100,"fill":"{fill}"}}]}}]}}
"##
        ),
    )
    .expect("write project");
}

#[test]
fn a_cancelled_render_publishes_nothing_and_frees_the_slot_for_the_next_call() {
    if !has_ffmpeg() {
        eprintln!("skipping: no ffmpeg/ffprobe on PATH");
        return;
    }
    let dir = scratch_dir("cancelled-render");
    let project = dir.join("c.montagent.json");
    let tiny = dir.join("t.montagent.json");
    write_long_or_short(&project, "c", 400, "#FF0000");
    write_long_or_short(&tiny, "t", 40, "#00FF00");
    let mut server = Server::start(&dir);
    let project_arg = project.display().to_string();

    // A stamped deliverable already at the path: the file the cancelled call must leave.
    server.call(2, "render", json!({"project": project_arg}), None);
    server.wait_for(2);
    let before = std::fs::read(dir.join("out/c.mp4")).expect("the first deliverable");

    // A long edit of the same project, so a completed render would replace those bytes.
    write_long_or_short(&project, "c", 60_000, "#0000FF");
    server.call(3, "render", json!({"project": project_arg}), Some("c"));
    // Wait for the encode to be under way: the first frame report.
    let deadline = Instant::now() + Duration::from_secs(60);
    while !server.progress("c").iter().any(|l| {
        l.value["params"]["message"]
            .as_str()
            .is_some_and(|m| m.contains("frames"))
    }) {
        assert!(Instant::now() < deadline, "the long render never started");
        if let Ok(line) = server.lines.recv_timeout(Duration::from_millis(100)) {
            server.seen.push(line);
        }
    }

    let cancelled_at = server.send(json!({"jsonrpc":"2.0","method":"notifications/cancelled",
        "params":{"requestId":3,"reason":"acceptance test"}}));
    // The next call needs the same encode slot, so it can only start once the cancelled
    // encode has actually stopped.
    server.call(
        4,
        "render",
        json!({"project": tiny.display().to_string()}),
        None,
    );
    let next_at = server.wait_for(4);

    assert!(
        next_at.duration_since(cancelled_at) < Duration::from_secs(10),
        "the slot was held {:?} after the cancel — the 1500-frame encode ran on",
        next_at.duration_since(cancelled_at)
    );
    assert!(text_of(server.response(4)).contains("RENDER"));
    assert_eq!(
        std::fs::read(dir.join("out/c.mp4")).expect("the first deliverable"),
        before,
        "the cancelled render touched the deliverable"
    );
    let mut beside: Vec<String> = std::fs::read_dir(dir.join("out"))
        .expect("out/")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    beside.sort();
    assert_eq!(
        beside,
        vec!["c.mp4".to_string(), "t.mp4".to_string()],
        "a temp file was left"
    );
    let stderr = server.stderr();
    assert!(stderr.contains("#3 render  cancel requested"), "{stderr}");
}
