"""Call Azure FLUX.2-pro. Usage: W=1024 H=1024 python3 flux.py out.png "prompt" [ref.png ...]"""
import base64, json, os, sys, time, urllib.error, urllib.request
from pathlib import Path

ENV = Path(__file__).resolve().parent.parent / ".env"
env = dict(l.strip().split("=", 1) for l in ENV.read_text().splitlines() if "=" in l and not l.startswith("#"))
ep, key, ver, model = env["AZURE_IMAGE_ENDPOINT"], env["AZURE_IMAGE_KEY"], env["AZURE_IMAGE_API_VERSION"], env["AZURE_IMAGE_MODEL"]

out, prompt, refs = sys.argv[1], sys.argv[2], sys.argv[3:]
body = {"prompt": prompt, "model": model.lower(), "width": int(os.environ.get("W", 1024)),
        "height": int(os.environ.get("H", 1024)), "output_format": "png"}
for i, r in enumerate(refs):
    body["input_image" + ("" if i == 0 else f"_{i + 1}")] = base64.b64encode(Path(r).read_bytes()).decode()

req = urllib.request.Request(f"{ep}?api-version={ver}", json.dumps(body).encode(),
                             {"Content-Type": "application/json", "Authorization": f"Bearer {key}"})
t = time.time()
try:
    res = json.load(urllib.request.urlopen(req, timeout=300))
except urllib.error.HTTPError as e:
    sys.exit(f"HTTP {e.code}: {e.read()[:800]!r}")


def short(v):
    if isinstance(v, dict):
        return {k: short(x) for k, x in v.items()}
    if isinstance(v, list):
        return [short(x) for x in v]
    return f"<{len(v)} chars>" if isinstance(v, str) and len(v) > 200 else v


print(round(time.time() - t, 1), "s", json.dumps(short(res))[:600])
item = res["data"][0] if "data" in res else res
b64 = item.get("b64_json") or (item.get("result") or {}).get("sample")
if b64:
    Path(out).write_bytes(base64.b64decode(b64))
    print("wrote", out)
