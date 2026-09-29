#!/usr/bin/env bash
# ADR-0109's behavioural claims, asserted end to end against a built binary.
#
# Run from anywhere:  bash docs/adr/mcp_cancel_withholds_check.sh
# Exits non-zero, naming every defect, the moment one of them stops holding.
#
# Why this exists
# ---------------
# The claim is about the disk after a protocol event: when an MCP client sends
# `notifications/cancelled` mid-render, nothing is published and the deliverable already at
# the path is untouched, and the server is free for the next encode at once. Only a real
# `montagent mcp` spoken to over real pipes can show what is on disk afterwards and when the
# next call ran.
#
# Run it against the commit before ADR-0109 and assertions 1, 2 and 4 fail: the cancelled
# render runs on, replaces the deliverable, and holds the slot for its whole encode.
#
# Needs `ffmpeg`/`ffprobe` (ADR-0009) and `python3`, and skips with a message when they are
# absent.

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO" || exit 1

for tool in ffmpeg ffprobe python3; do
  if ! command -v "$tool" > /dev/null 2>&1; then
    echo "skipped: $tool not on PATH"
    exit 0
  fi
done

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo "building the binary under test"
cargo build -q -p montagent || { echo "FAIL: the binary did not build"; exit 1; }

MONTAGENT="$REPO/target/debug/montagent" WORK="$WORK" python3 - <<'PY'
import hashlib, json, os, subprocess, sys, threading, time, queue

work = os.environ["WORK"]

def project(path, name, duration, fill):
    with open(path, "w") as f:
        json.dump({"frame": {"width": 640, "height": 360}, "fps": 25, "background": "#000000",
                   "duration": duration, "output": f"out/{name}.mp4",
                   "tracks": [{"name": "only", "layer": 0, "elements": [
                       {"id": "card", "type": "rect", "start": 0, "end": duration, "x": 320,
                        "y": 180, "width": 200, "height": 100, "fill": fill}]}]}, f)

main, tiny = f"{work}/c.montagent.json", f"{work}/t.montagent.json"
project(main, "c", 400, "#FF0000")
project(tiny, "t", 40, "#00FF00")

env = dict(os.environ, MONTAGENT_MCP_HEARTBEAT_MS="150", MONTAGENT_CACHE_DIR=f"{work}/cache")
stderr = open(f"{work}/server.stderr", "w")
server = subprocess.Popen([os.environ["MONTAGENT"], "mcp"], stdin=subprocess.PIPE,
                          stdout=subprocess.PIPE, stderr=stderr, text=True, env=env)
lines = queue.Queue()
def read():
    for line in server.stdout:
        if line.strip():
            lines.put((time.monotonic(), json.loads(line)))
threading.Thread(target=read, daemon=True).start()

def send(message):
    server.stdin.write(json.dumps(message) + "\n")
    server.stdin.flush()
    return time.monotonic()

seen = []
def wait_for(pred, timeout=120):
    end = time.monotonic() + timeout
    while True:
        for at, value in seen:
            if pred(value):
                return at
        seen.append(lines.get(timeout=max(0.1, end - time.monotonic())))

def call(id_, name, args, token=None):
    params = {"name": name, "arguments": args}
    if token:
        params["_meta"] = {"progressToken": token}
    return send({"jsonrpc": "2.0", "id": id_, "method": "tools/call", "params": params})

send({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
    "protocolVersion": "2025-06-18", "capabilities": {},
    "clientInfo": {"name": "adr-0109-check", "version": "0"}}})
wait_for(lambda v: v.get("id") == 0)
send({"jsonrpc": "2.0", "method": "notifications/initialized"})

call(2, "render", {"project": main})
wait_for(lambda v: v.get("id") == 2)
digest = lambda p: hashlib.sha256(open(p, "rb").read()).hexdigest()
before = digest(f"{work}/out/c.mp4")

project(main, "c", 60000, "#0000FF")
call(3, "render", {"project": main}, token="c")
wait_for(lambda v: v.get("method") == "notifications/progress"
         and v["params"]["progressToken"] == "c" and "frames" in v["params"].get("message", ""))
cancelled_at = send({"jsonrpc": "2.0", "method": "notifications/cancelled",
                     "params": {"requestId": 3, "reason": "adr-0109-check"}})
call(4, "render", {"project": tiny})
next_at = wait_for(lambda v: v.get("id") == 4)
server.kill()
stderr.close()

failures = 0
def check(ok, what):
    global failures
    print(("  ok: " if ok else "  FAIL: ") + what)
    failures += 0 if ok else 1

check(digest(f"{work}/out/c.mp4") == before,
      "1. the deliverable already at the path is byte-identical after the cancel")
held = next_at - cancelled_at
check(held < 10, f"2. the next encode ran {held:.1f} s after the cancel (bound 10 s; the cancelled one had 1500 frames)")
check(sorted(os.listdir(f"{work}/out")) == ["c.mp4", "t.mp4"],
      f"3. no temp file left in out/: {sorted(os.listdir(f'{work}/out'))}")
check("#3 render  cancel requested" in open(f"{work}/server.stderr").read(),
      "4. stderr records the cancel against the call it cancelled")
sys.exit(1 if failures else 0)
PY
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  echo "ADR-0109 does not hold"
  exit 1
fi
echo "ADR-0109 holds"
