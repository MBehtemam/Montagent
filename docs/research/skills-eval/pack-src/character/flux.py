"""Draw or edit one image with FLUX.2-pro on Azure AI Foundry.

Usage: `W=1024 H=1536 python flux.py out.png "prompt" [reference.png ...]`

With reference images it is an *edit*: FLUX keeps the reference and changes what the
prompt names. The keys come from the git-ignored `.env` at the repository root
(`AZURE_IMAGE_ENDPOINT`, `AZURE_IMAGE_KEY`, `AZURE_IMAGE_API_VERSION`, `AZURE_IMAGE_MODEL`).
FLUX is not deterministic, so re-running draws a different picture; the committed PNGs
are the record.
"""

import base64
import json
import os
import sys
import urllib.error
import urllib.request
from pathlib import Path

ENV = Path(__file__).resolve().parents[5] / ".env"
env = dict(l.strip().split("=", 1) for l in ENV.read_text().splitlines() if "=" in l and not l.startswith("#"))

out, prompt, refs = sys.argv[1], sys.argv[2], sys.argv[3:]
body = {"prompt": prompt, "model": env["AZURE_IMAGE_MODEL"].lower(), "output_format": "png",
        "width": int(os.environ.get("W", 1024)), "height": int(os.environ.get("H", 1024))}
for i, ref in enumerate(refs):
    body["input_image" + ("" if i == 0 else f"_{i + 1}")] = base64.b64encode(Path(ref).read_bytes()).decode()

req = urllib.request.Request(f"{env['AZURE_IMAGE_ENDPOINT']}?api-version={env['AZURE_IMAGE_API_VERSION']}",
                             json.dumps(body).encode(),
                             {"Content-Type": "application/json", "Authorization": f"Bearer {env['AZURE_IMAGE_KEY']}"})
try:
    res = json.load(urllib.request.urlopen(req, timeout=300))
except urllib.error.HTTPError as e:
    sys.exit(f"HTTP {e.code}: {e.read()[:800]!r}")

item = res["data"][0] if "data" in res else res
b64 = item.get("b64_json") or (item.get("result") or {}).get("sample")
if not b64:
    sys.exit(f"no image in the response: {json.dumps(res)[:800]}")
Path(out).write_bytes(base64.b64decode(b64))
print("wrote", out)
