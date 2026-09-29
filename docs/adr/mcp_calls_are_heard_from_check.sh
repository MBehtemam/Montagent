#!/usr/bin/env bash
# ADR-0108's behavioural claims, asserted end to end against a built binary.
#
# Run from anywhere:  bash docs/adr/mcp_calls_are_heard_from_check.sh
# Exits non-zero, naming every defect, the moment one of them stops holding.
#
# Why this exists
# ---------------
# MONTAGENT-9 was a transport fact: a call issued behind a `render` over MCP waited unread,
# with nothing on the wire, until Claude Code's 1800 s idle window aborted it. A test at the
# verb seam cannot see that, because the verbs were never wrong. Only a real `montagent mcp`,
# spoken to over real pipes, shows what a client sees and when.
#
# Run it against the commit before ADR-0108 and assertions 1, 2 and 4 fail: the `frame`
# comes back after the render, the render sends no progress, and nothing records the call.
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

cat > "$WORK/p.montagent.json" <<'JSON'
{"frame":{"width":640,"height":360},"fps":25,"background":"#000000","duration":4000,
 "output":"out/p.mp4",
 "tracks":[{"name":"only","layer":0,"elements":[
   {"id":"card","type":"rect","start":0,"end":4000,"x":320,"y":180,"width":200,"height":100,
    "fill":"#FF0000"}]}]}
JSON

MONTAGENT="$REPO/target/debug/montagent" WORK="$WORK" python3 - <<'PY'
import json, os, subprocess, sys, threading, time, queue

work = os.environ["WORK"]
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
def wait_for(id_):
    while True:
        for at, value in seen:
            if value.get("id") == id_:
                return at
        seen.append(lines.get(timeout=120))

send({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
    "protocolVersion": "2025-06-18", "capabilities": {},
    "clientInfo": {"name": "adr-0108-check", "version": "0"}}})
wait_for(0)
send({"jsonrpc": "2.0", "method": "notifications/initialized"})

project = f"{work}/p.montagent.json"
dispatched = send({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
    "name": "render", "arguments": {"project": project}, "_meta": {"progressToken": "r"}}})
send({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {
    "name": "frame", "arguments": {"project": project, "at": 1000}}})
frame_at = wait_for(3)
render_at = wait_for(2)
server.kill()
stderr.close()

failures = 0
def check(ok, what):
    global failures
    print(("  ok: " if ok else "  FAIL: ") + what)
    failures += 0 if ok else 1

check(frame_at < render_at,
      f"1. `frame` answered while the render was encoding ({(render_at - frame_at) * 1000:.0f} ms before it)")
progress = [(at, v) for at, v in seen
            if v.get("method") == "notifications/progress" and v["params"]["progressToken"] == "r"]
gaps = [b - a for a, b in zip([dispatched] + [at for at, _ in progress],
                              [at for at, _ in progress] + [render_at])]
check(bool(progress) and max(gaps) <= 1.5,
      f"2. render progress never silent over 1.5 s ({len(progress)} notifications, "
      f"longest gap {max(gaps) * 1000:.0f} ms)")
values = [v["params"]["progress"] for _, v in progress]
check(all(b > a for a, b in zip(values, values[1:])), "3. `progress` strictly increases")
log = open(f"{work}/server.stderr").read()
check("render  dispatched  progressToken: present" in log
      and "frame  dispatched  progressToken: absent" in log,
      "4. stderr records whether each call carried a progressToken")
check(os.path.isfile(f"{work}/out/p.mp4"), "5. the deliverable was published")
sys.exit(1 if failures else 0)
PY
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  echo "ADR-0108 does not hold"
  exit 1
fi
echo "ADR-0108 holds"
