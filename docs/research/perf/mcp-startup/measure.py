#!/usr/bin/env python3
"""Measure MCP stdio server cold start across TypeScript, Python and Rust.

Prototype for #16, part of the map #2. Throwaway.

Two kinds of measurement:

  exit variants     -- spawn a process that writes one line and exits. Isolates
                       runtime startup (`bare`) from SDK import cost (`import`).
                       Timed to the line, so it stops at the same point the protocol
                       variants do; interpreter teardown is reported separately
                       because it is real but is not startup.
  protocol variants -- spawn a real MCP server and drive the handshake over
                       stdio sequentially, exactly as a client does: each request
                       is written only after the previous response is read.
                       Timers start immediately before spawn, so every number
                       includes process creation.

All times are wall-clock seconds from just-before-spawn.
"""

import argparse
import json
import os
import statistics
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
VENV_PY = HERE / ".venv" / "bin" / "python"
RS_BIN = HERE / "rs" / "target" / "release"

LEGACY = "2025-11-25"   # newest revision all three SDKs implement
MODERN = "2026-07-28"   # current spec revision; TS SDK 1.30.0 does not support it


def meta(version):
    return {
        "io.modelcontextprotocol/protocolVersion": version,
        "io.modelcontextprotocol/clientInfo": {"name": "harness", "version": "0"},
        "io.modelcontextprotocol/clientCapabilities": {},
    }


def legacy_exchange():
    """(handshake request, notifications, follow-up request) for initialize-based revisions."""
    return (
        {"jsonrpc": "2.0", "id": 1, "method": "initialize",
         "params": {"protocolVersion": LEGACY, "capabilities": {},
                    "clientInfo": {"name": "harness", "version": "0"}}},
        [{"jsonrpc": "2.0", "method": "notifications/initialized"}],
        {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
    )


def modern_exchange():
    """(handshake request, notifications, follow-up request) for per-request-metadata revisions."""
    return (
        {"jsonrpc": "2.0", "id": 1, "method": "server/discover",
         "params": {"_meta": meta(MODERN)}},
        [],
        {"jsonrpc": "2.0", "id": 2, "method": "tools/list",
         "params": {"_meta": meta(MODERN)}},
    )


def env_for(schema):
    e = dict(os.environ)
    e["SCHEMA"] = schema
    e.pop("PYTHONPATH", None)
    return e


def exit_variants():
    return {
        "ts / bare node": ["node", str(HERE / "ts" / "bare.mjs")],
        "ts / sdk import": ["node", str(HERE / "ts" / "import-only.mjs")],
        "py / bare python": [str(VENV_PY), str(HERE / "py" / "bare.py")],
        "py / sdk import": [str(VENV_PY), str(HERE / "py" / "import_only.py")],
        "rs / bare binary": [str(RS_BIN / "bare")],
    }


def server_cmds():
    return {
        "ts": ["node", str(HERE / "ts" / "server.mjs")],
        "py": [str(VENV_PY), str(HERE / "py" / "server.py")],
        "rs": [str(RS_BIN / "server")],
    }


def read_response(proc, want_id, deadline=30.0):
    """Read stdout lines until the JSON-RPC response with `want_id` arrives."""
    start = time.perf_counter()
    while True:
        if time.perf_counter() - start > deadline:
            raise TimeoutError(f"no response id={want_id} within {deadline}s")
        line = proc.stdout.readline()
        if not line:
            err = proc.stderr.read().decode("utf-8", "replace")[-2000:]
            raise RuntimeError(f"server closed stdout before id={want_id}\nstderr:\n{err}")
        line = line.strip()
        if not line:
            continue
        msg = json.loads(line)
        if msg.get("id") == want_id:
            if "error" in msg:
                raise RuntimeError(f"server returned error: {msg['error']}")
            return msg
        # notifications and unrelated messages are skipped, per the stdio binding


def time_exit(argv, env):
    """Returns (to_line, incl_exit). `to_line` is comparable with the protocol
    numbers; the difference is runtime teardown, which an stdio server pays at
    session end rather than at startup."""
    t0 = time.perf_counter()
    proc = subprocess.Popen(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env)
    proc.stdout.readline()
    to_line = time.perf_counter() - t0
    proc.wait()
    incl_exit = time.perf_counter() - t0
    proc.stdout.close()
    proc.stderr.close()
    return to_line, incl_exit


def time_protocol(argv, env, exchange):
    """Returns (t_handshake, t_tools_list) both measured from just-before-spawn."""
    handshake_req, notifs, followup_req = exchange
    t0 = time.perf_counter()
    proc = subprocess.Popen(argv, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, env=env)
    try:
        proc.stdin.write((json.dumps(handshake_req) + "\n").encode())
        proc.stdin.flush()
        read_response(proc, 1)
        t_handshake = time.perf_counter() - t0

        for n in notifs:
            proc.stdin.write((json.dumps(n) + "\n").encode())
        proc.stdin.write((json.dumps(followup_req) + "\n").encode())
        proc.stdin.flush()
        resp = read_response(proc, 2)
        t_tools = time.perf_counter() - t0

        n_tools = len(resp["result"]["tools"])
        schema_bytes = len(json.dumps(resp["result"]["tools"][0]["inputSchema"],
                                      separators=(",", ":")))
        return t_handshake, t_tools, n_tools, schema_bytes
    finally:
        try:
            proc.stdin.close()
        except Exception:
            pass
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
        proc.stdout.close()
        proc.stderr.close()


def summarize(xs):
    xs = sorted(xs)
    return {
        "n": len(xs),
        "min": xs[0],
        "p10": xs[max(0, int(0.10 * len(xs)) - 1)],
        "median": statistics.median(xs),
        "p90": xs[min(len(xs) - 1, int(0.90 * len(xs)))],
        "max": xs[-1],
    }


def ms(x):
    return f"{x * 1000:.1f}"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("-n", "--iterations", type=int, default=30)
    ap.add_argument("-o", "--out", default=str(HERE / "results.json"))
    args = ap.parse_args()
    N = args.iterations

    results = {"iterations": N, "exit": {}, "protocol": {}, "first_run": {}}

    print(f"# exit variants (n={N})\n")
    print(f"{'variant':<22} {'first':>8} {'median':>8} {'p10':>8} {'p90':>8} {'teardown':>9}")
    for name, argv in exit_variants().items():
        env = env_for("big")
        runs = [time_exit(argv, env) for _ in range(N)]
        s = summarize([r[0] for r in runs])
        td = summarize([r[1] - r[0] for r in runs])
        results["exit"][name] = {"to_line": s, "teardown": td}
        results["first_run"][name] = runs[0][0]
        print(f"{name:<22} {ms(runs[0][0]):>8} {ms(s['median']):>8} "
              f"{ms(s['p10']):>8} {ms(s['p90']):>8} {ms(td['median']):>9}")

    exchanges = {"legacy": legacy_exchange(), "modern": modern_exchange()}
    for revision, exchange in exchanges.items():
        label = LEGACY if revision == "legacy" else MODERN
        for schema in ("small", "big"):
            print(f"\n# protocol: {revision} ({label}), schema={schema} (n={N})\n")
            print(f"{'host':<6} {'handshake':>18} {'+ tools/list':>18} "
                  f"{'schema B':>9} {'first run':>10}")
            for host, argv in server_cmds().items():
                env = env_for(schema)
                key = f"{host}/{revision}/{schema}"
                try:
                    runs = [time_protocol(argv, env, exchange) for _ in range(N)]
                except Exception as e:
                    print(f"{host:<6} UNSUPPORTED: {type(e).__name__}: "
                          f"{str(e).splitlines()[0][:80]}")
                    results["protocol"][key] = {"error": f"{type(e).__name__}: {e}"[:400]}
                    continue
                hs = summarize([r[0] for r in runs])
                tl = summarize([r[1] for r in runs])
                results["protocol"][key] = {
                    "handshake": hs, "tools_list": tl,
                    "schema_bytes": runs[0][3], "n_tools": runs[0][2],
                    "first_run_tools_list": runs[0][1],
                }
                print(f"{host:<6} {ms(hs['median'])+' ms':>18} "
                      f"{ms(tl['median'])+' ms':>18} "
                      f"{runs[0][3]:>9} {ms(runs[0][1])+' ms':>10}")

    Path(args.out).write_text(json.dumps(results, indent=2) + "\n")
    print(f"\nwrote {args.out}")


if __name__ == "__main__":
    main()
