#!/usr/bin/env python3
"""Turn a code-drawn piece into lossless footage with a recipe beside it, rebuild it, and
place it in a project.

    python3 prerender.py build <spec>             render, encode, verify, write the recipe
    python3 prerender.py rebuild <recipe-dir>     re-run the recipe, compare decoded frames
    python3 prerender.py place <project> <spec>   print <project> with the footage added

Requires Python >= 3.9
Standard library only, plus ffmpeg on PATH (and `montagent` for `place`).
drift-guard: place type_on.montagent.json prerender.spec.json

`build` takes a JSON spec:

    {
      "name": "pulse",                 required: the footage is <out>/<name>.mov
      "out": "media",                  folder for the footage and its recipe (default ".")
      "width": 160, "height": 120,     required
      "fps": 25, "frames": 48,         required (fps may be a rational such as "30000/1001")
      "render": ["python3", "sample_piece.py"],  required: your code, as an argv
      "files": ["sample_piece.py"],    required: every file the code needs, copied into the recipe
      "input": "rgba",                 "rgba" (default): the code writes raw RGBA frames to stdout
                                       "png": it writes f00000.png, f00001.png ... into
                                       $PRERENDER_FRAMES_DIR
      "versions": {"pillow": ["python3", "-c", "import PIL; print(PIL.__version__)"]}
    }

Paths in the spec resolve against the spec's folder, and the code runs there. It reads its
size from the environment: PRERENDER_WIDTH, PRERENDER_HEIGHT, PRERENDER_FPS,
PRERENDER_FRAMES (and PRERENDER_FRAMES_DIR for "png"). Use integer arithmetic or a fixed
seed, so the same code draws the same frames.

The footage is PNG in MOV, `-c:v png -pix_fmt rgba`, the one codec checked to return every
byte of RGBA, alpha included. `build` decodes it and fails on the first frame that differs
from what the code drew. The recipe, <out>/<name>.recipe/, holds the source files and
recipe.json (commands, tool versions, size, fps, frame count, one hash per decoded frame).
No project names it.

`rebuild` re-runs the recipe from its own copy of the code and compares decoded-frame hashes,
never file bytes. It also decodes the placed footage and compares that. Exit 0 on a match,
1 on a difference, with the first frame that differs.

`place` takes the project and a spec {"source": "media/pulse.mov", "id": "pulse",
"track": "prerender", "layer": 10, "at": 0, "x": ..., "y": ..., "origin": "center",
"width": ..., "height": ...}; every key but `source` is optional. It probes the footage,
refuses it when `probe` does not report alpha, and prints the project with a plain `video`
element added.
"""

import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ENCODE = ["ffmpeg", "-y", "-v", "error", "{input}", "-c:v", "png", "-pix_fmt", "rgba", "{out}"]
DECODE = ["ffmpeg", "-v", "error", "{input}", "-f", "rawvideo", "-pix_fmt", "rgba", "-"]


def fail(message):
    sys.exit(f"prerender: {message}")


def run(argv, **kw):
    try:
        return subprocess.run(argv, check=True, **kw)
    except FileNotFoundError:
        fail(f"`{argv[0]}` is not on PATH")
    except subprocess.CalledProcessError as e:
        fail(f"`{' '.join(map(str, argv))}` exited {e.returncode}")


def expand(template, input, out=None):
    """Fill `{input}` (a list of arguments) and `{out}` (a path) into a command template."""
    argv = []
    for part in template:
        argv.extend(input if part == "{input}" else [out if part == "{out}" else part])
    return argv


def frames_input(recipe, work):
    """The ffmpeg input arguments that read the drawn frames from `work`."""
    fps = str(recipe["fps"])
    if recipe["input"] == "png":
        return ["-framerate", fps, "-start_number", "0", "-i", str(work / "f%05d.png")]
    size = f"{recipe['width']}x{recipe['height']}"
    return ["-f", "rawvideo", "-pix_fmt", "rgba", "-s", size, "-r", fps,
            "-i", str(work / "frames.rgba")]  # fmt: skip


def draw(recipe, source_dir, work):
    env = dict(os.environ)
    env.update(
        PRERENDER_WIDTH=str(recipe["width"]),
        PRERENDER_HEIGHT=str(recipe["height"]),
        PRERENDER_FPS=str(recipe["fps"]),
        PRERENDER_FRAMES=str(recipe["frames"]),
        PRERENDER_FRAMES_DIR=str(work),
    )
    if recipe["input"] == "png":
        run(recipe["render"], cwd=source_dir, env=env)
    else:
        with open(work / "frames.rgba", "wb") as f:
            run(recipe["render"], cwd=source_dir, env=env, stdout=f)


def stream_hashes(argv, size):
    """sha256 of each `size`-byte frame that `argv` writes to stdout."""
    proc = subprocess.Popen(argv, stdout=subprocess.PIPE)
    hashes = []
    while True:
        chunk = proc.stdout.read(size)
        if not chunk:
            break
        if len(chunk) != size:
            fail("a partial frame came back")
        hashes.append(hashlib.sha256(chunk).hexdigest())
    if proc.wait() != 0:
        fail(f"`{' '.join(argv)}` failed")
    return hashes


def drawn_hashes(recipe, work):
    """Hashes of the frames the code drew: bytes of frames.rgba, or the PNGs decoded."""
    size = recipe["width"] * recipe["height"] * 4
    if recipe["input"] == "rgba":
        data = (work / "frames.rgba").read_bytes()
        if len(data) != size * recipe["frames"]:
            fail(f"the code wrote {len(data)} bytes, not {recipe['frames']} frames of {size}")
        return [hashlib.sha256(data[i : i + size]).hexdigest() for i in range(0, len(data), size)]
    return stream_hashes(expand(DECODE, input=["-start_number", "0", "-i", str(work / "f%05d.png")]), size)


def encode(recipe, work, out):
    run(expand(ENCODE, input=frames_input(recipe, work), out=str(out)))


def decoded_hashes(recipe, footage):
    size = recipe["width"] * recipe["height"] * 4
    return stream_hashes(expand(DECODE, input=["-i", str(footage)]), size)


def first_difference(want, got):
    for i in range(max(len(want), len(got))):
        if i >= len(want) or i >= len(got) or want[i] != got[i]:
            return i
    return None


def tool_versions(spec):
    found = {"python": sys.version.split()[0]}
    line = subprocess.run(["ffmpeg", "-version"], capture_output=True, text=True).stdout
    found["ffmpeg"] = line.splitlines()[0] if line else "unknown"
    for name, argv in spec.get("versions", {}).items():
        out = subprocess.run(argv, capture_output=True, text=True, cwd=spec["_dir"])
        text = (out.stdout.strip() or out.stderr.strip()).splitlines()
        found[name] = text[0] if text else "unknown"
    return found


def build(spec_path):
    spec_path = Path(spec_path).resolve()
    spec = json.loads(spec_path.read_text())
    spec["_dir"] = spec_path.parent
    for key in ("name", "width", "height", "fps", "frames", "render", "files"):
        if key not in spec:
            fail(f"the spec needs `{key}`")
    kind = spec.get("input", "rgba")
    if kind not in ("rgba", "png"):
        fail('`input` is "rgba" or "png"')
    out_dir = (spec["_dir"] / spec.get("out", ".")).resolve()
    recipe_dir = out_dir / f"{spec['name']}.recipe"
    footage = out_dir / f"{spec['name']}.mov"
    out_dir.mkdir(parents=True, exist_ok=True)
    if recipe_dir.exists():
        shutil.rmtree(recipe_dir)
    for rel in spec["files"]:
        target = recipe_dir / "source" / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(spec["_dir"] / rel, target)

    recipe = {
        "name": spec["name"], "width": spec["width"], "height": spec["height"],
        "fps": spec["fps"], "frames": spec["frames"], "input": kind,
        "render": spec["render"], "encode": ENCODE, "decode": DECODE,
        "footage": os.path.relpath(footage, recipe_dir),
        "versions": tool_versions(spec),
    }  # fmt: skip
    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp)
        draw(recipe, spec["_dir"], work)
        drawn = drawn_hashes(recipe, work)
        encode(recipe, work, footage)
    decoded = decoded_hashes(recipe, footage)
    bad = first_difference(drawn, decoded)
    if bad is not None:
        fail(f"the footage does not return the drawn pixels: frame {bad} differs")
    recipe["decoded_frame_sha256"] = decoded
    (recipe_dir / "recipe.json").write_text(json.dumps(recipe, indent=2) + "\n")
    print(f"footage {footage}\nrecipe  {recipe_dir}\n{len(decoded)} frames, decoded = drawn",
          file=sys.stderr)  # fmt: skip


def rebuild(recipe_dir):
    recipe_dir = Path(recipe_dir).resolve()
    recipe = json.loads((recipe_dir / "recipe.json").read_text())
    want = recipe["decoded_frame_sha256"]
    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp)
        draw(recipe, recipe_dir / "source", work)
        encode(recipe, work, work / "rebuilt.mov")
        ok = report("rebuild", want, decoded_hashes(recipe, work / "rebuilt.mov"))
    placed = (recipe_dir / recipe["footage"]).resolve()
    if placed.exists():
        ok &= report("footage", want, decoded_hashes(recipe, placed))
    else:
        print(f"footage: {placed} is missing")
        ok = False
    sys.exit(0 if ok else 1)


def report(label, want, got):
    bad = first_difference(want, got)
    if bad is None:
        print(f"{label}: match, {len(want)} frames")
        return True
    print(f"{label}: differs, first at frame {bad} ({len(want)} recorded, {len(got)} decoded)")
    return False


def place(project_path, spec_path):
    project_path = Path(project_path).resolve()
    project = json.loads(project_path.read_text())
    spec = json.loads(Path(spec_path).read_text())
    source = spec["source"]
    probed = subprocess.run(["montagent", "probe", "--json", str(project_path.parent / source)],
                            capture_output=True, text=True)  # fmt: skip
    if not probed.stdout:
        fail(f"`montagent probe` printed nothing: {probed.stderr.strip()}")
    media = json.loads(probed.stdout)["media"][0]
    if not media["alpha"]["carries"]:
        fail(f"`probe` reports no alpha on {source}: encode it with `-pix_fmt rgba`")
    ms = media["quad"]["video_stream_ms"]
    at = spec.get("at", 0)
    frame = project["frame"]
    element = {
        "id": spec.get("id", Path(source).stem), "type": "video",
        "start": at, "end": at + ms, "source": source, "source_start": 0, "source_end": ms,
        "volume": 0,
        "x": spec.get("x", frame["width"] // 2), "y": spec.get("y", frame["height"] // 2),
        "origin": spec.get("origin", "center"), "fit": spec.get("fit", "contain"),
        "width": spec.get("width", media["dimensions"]["width"]),
        "height": spec.get("height", media["dimensions"]["height"]),
    }  # fmt: skip
    name = spec.get("track", "prerender")
    track = next((t for t in project["tracks"] if t["name"] == name), None)
    if track is None:
        track = {"name": name, "layer": spec.get("layer", 10), "elements": []}
        project["tracks"].append(track)
    track["elements"] = [e for e in track["elements"] if e["id"] != element["id"]] + [element]
    project["duration"] = max(project.get("duration", 0), element["end"])
    print(json.dumps(project, indent=2))


def main(argv):
    if not argv or argv[0] in ("--help", "-h"):
        print(__doc__)
    elif argv[0] == "build" and len(argv) == 2:
        build(argv[1])
    elif argv[0] == "rebuild" and len(argv) == 2:
        rebuild(argv[1])
    elif argv[0] == "place" and len(argv) == 3:
        place(argv[1], argv[2])
    else:
        fail("usage: build <spec> | rebuild <recipe-dir> | place <project> <spec>")


if __name__ == "__main__":
    main(sys.argv[1:])
