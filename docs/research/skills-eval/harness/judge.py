"""The human's blind judging page.

    uv run docs/research/skills-eval/harness/judge.py [--phase verdict] [--port 8765]

Serves `judging/<phase>/pairs.json` one pair at a time on localhost: the brief, and the two
clips side by side under their random names. Each answer (left better / equal / right better)
is written to `judging/<phase>/ballots/human.json` the moment it is given, so judging can
stop and resume. Pairs decided by a missing video (`"auto"`) are skipped.

Clips are served by random name. The page never learns which run a name is; only the server
reads `key.json`, to find the file. Do not open `key.json` until every pair is judged.
"""

# /// script
# requires-python = ">=3.11"
# ///

from __future__ import annotations

import argparse
import datetime as dt
import html
import json
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import JUDGING, PHASES, REPO, read_json, write_json  # noqa: E402

VOTES = ("left", "right", "equal")

PAGE = """<!doctype html>
<meta charset="utf-8">
<title>Skills eval: blind judging</title>
<style>
  :root { color-scheme: light dark; --bg: #f5f0e6; --fg: #101418; --muted: #6b6b6b; --line: #d8d2c6; }
  @media (prefers-color-scheme: dark) { :root { --bg: #101418; --fg: #f5f0e6; --muted: #9a9a9a; --line: #2a3036; } }
  body { margin: 0; padding: 16px 24px; background: var(--bg); color: var(--fg); font: 15px/1.45 system-ui, sans-serif; }
  header { display: flex; justify-content: space-between; align-items: baseline; }
  .muted { color: var(--muted); }
  .pair { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin: 12px 0; }
  figure { margin: 0; } figcaption { font-weight: 600; margin-bottom: 4px; }
  video { width: 100%%; max-height: 62vh; background: #000; border-radius: 6px; }
  .votes { display: flex; gap: 12px; justify-content: center; margin: 8px 0 16px; }
  button { font: inherit; padding: 10px 22px; border-radius: 8px; border: 1px solid var(--line);
           background: transparent; color: inherit; cursor: pointer; }
  button:hover { border-color: currentColor; }
  details { border-top: 1px solid var(--line); padding-top: 8px; }
  pre { white-space: pre-wrap; font: 13px/1.5 ui-monospace, monospace; }
  .controls { display: flex; gap: 8px; justify-content: center; }
</style>
<header><h2>Pair %(done)d of %(total)d · brief %(brief)s</h2>
<span class="muted">%(pair_id)s</span></header>
<p class="muted">Which is the better finished piece for this brief? Judge the whole video:
the brief's beats, then craft (timing, motion, type, composition, polish) and defects.
Pick equal when you cannot honestly prefer one.</p>
<div class="controls">
  <button onclick="both('play')">Play both</button>
  <button onclick="both('restart')">Restart both</button>
  <button onclick="sound(0)">Sound: left</button>
  <button onclick="sound(1)">Sound: right</button>
  <button onclick="sound(-1)">Mute</button>
</div>
<div class="pair">
  <figure><figcaption>Left</figcaption><video src="/clip/%(left)s" controls loop muted playsinline></video></figure>
  <figure><figcaption>Right</figcaption><video src="/clip/%(right)s" controls loop muted playsinline></video></figure>
</div>
<div class="votes">
  <button onclick="vote('left')">← Left better</button>
  <button onclick="vote('equal')">↓ Equal</button>
  <button onclick="vote('right')">Right better →</button>
</div>
<details open><summary>The brief</summary><pre>%(brief_text)s</pre></details>
<script>
  const vids = [...document.querySelectorAll('video')];
  function both(what) { vids.forEach(v => { if (what === 'restart') v.currentTime = 0; v.play(); }); }
  function sound(i) { vids.forEach((v, j) => v.muted = j !== i); }
  async function vote(v) {
    await fetch('/ballot', {method: 'POST', body: JSON.stringify({pair: '%(pair_id)s', vote: v})});
    location.reload();
  }
  addEventListener('keydown', e => {
    const k = {ArrowLeft: 'left', ArrowRight: 'right', ArrowDown: 'equal'}[e.key];
    if (k) vote(k);
  });
</script>
"""

DONE = """<!doctype html><meta charset="utf-8"><title>Skills eval: done</title>
<body style="font: 16px system-ui; padding: 40px"><h2>All %(total)d pairs judged.</h2>
<p>Ballots are in <code>%(path)s</code>. Commit them, then run <code>verdict.py --write</code>.</p>"""


def make_handler(phase: str, briefs: dict[str, str]):
    out = JUDGING / phase
    key = read_json(out / "key.json")
    pairs = [p for p in read_json(out / "pairs.json") if "auto" not in p]
    ballots_path = out / "ballots" / "human.json"

    def ballots() -> dict:
        return read_json(ballots_path) if ballots_path.exists() else {}

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *a):
            pass

        def send(self, code, body: bytes, ctype="text/html; charset=utf-8"):
            self.send_response(code)
            self.send_header("Content-Type", ctype)
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_GET(self):
            if self.path.startswith("/clip/"):
                name = self.path.removeprefix("/clip/")
                if name not in key:
                    return self.send(404, b"no such clip")
                data = (REPO / key[name] / "render.mp4").read_bytes()
                # Browsers seek video by byte range, and Safari will not play without it.
                rng = self.headers.get("Range", "")
                if rng.startswith("bytes="):
                    start_s, _, end_s = rng.removeprefix("bytes=").partition("-")
                    start = int(start_s or 0)
                    end = min(int(end_s) if end_s else len(data) - 1, len(data) - 1)
                    self.send_response(206)
                    self.send_header("Content-Type", "video/mp4")
                    self.send_header("Accept-Ranges", "bytes")
                    self.send_header("Content-Range", f"bytes {start}-{end}/{len(data)}")
                    self.send_header("Content-Length", str(end - start + 1))
                    self.end_headers()
                    self.wfile.write(data[start:end + 1])
                    return
                return self.send(200, data, "video/mp4")
            cast = ballots()
            todo = [p for p in pairs if p["id"] not in cast]
            if not todo:
                page = DONE % {"total": len(pairs), "path": ballots_path.relative_to(REPO)}
                return self.send(200, page.encode())
            p = todo[0]
            page = PAGE % {
                "done": len(pairs) - len(todo) + 1, "total": len(pairs), "brief": html.escape(p["brief"]),
                "pair_id": p["id"], "left": p["left"], "right": p["right"],
                "brief_text": html.escape(briefs.get(p["brief"], "(brief text not supplied: pass --briefs)")),
            }
            self.send(200, page.encode())

        def do_POST(self):
            body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
            if body.get("vote") not in VOTES or body.get("pair") not in {p["id"] for p in pairs}:
                return self.send(400, b"bad ballot")
            cast = ballots()
            cast[body["pair"]] = {"vote": body["vote"],
                                  "at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")}
            write_json(ballots_path, cast)
            self.send(204, b"")

    return Handler


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--phase", choices=PHASES, default="verdict")
    ap.add_argument("--port", type=int, default=8765)
    ap.add_argument("--briefs", type=Path, nargs="*", default=[],
                    help="brief files to show beside each pair (the held-out ones live outside the repo)")
    args = ap.parse_args()
    briefs = {p.stem: p.read_text() for p in args.briefs}
    server = ThreadingHTTPServer(("127.0.0.1", args.port), make_handler(args.phase, briefs))
    print(f"judging {args.phase} at http://127.0.0.1:{args.port}/  (Ctrl-C to stop; progress is saved)")
    server.serve_forever()


if __name__ == "__main__":
    main()
