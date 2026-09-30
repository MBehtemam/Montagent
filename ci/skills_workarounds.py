#!/usr/bin/env python3
"""The workaround half of the skills drift guard (#450, #513).

A **Workaround** in a skill compensates for something Montagent cannot yet do, and names the
issue whose resolution retires it:

    prose <!-- workaround: #N · replaced by: what replaces it -->     (Markdown)
    # workaround: #N · replaced by: what replaces it                  (a script's header)

A marker covers the paragraph or list item it sits in. A marker standing alone as its own
paragraph covers the whole section, down to the next heading.

Modes:

    --lint      offline: malformed markers, workaround wording no marker covers, and prose
                that points at a Workaround script without that script's marker
    --check     --lint, plus every marked issue must still be open (for PRs touching skills/)
    --report    keep one rolling "Skills: stale workarounds" issue up to date (weekly); never
                fails on a closed marker, so `main` stays green. With --dry-run, print the
                body it would write and touch nothing
    --self-test the lint and the marker reader, against inline examples

Wording a reader should not take for a workaround opts out in its own paragraph with
`<!-- guard-ok: word -->` (or `# guard-ok: word` in a script), the same opt-out the cargo
half (`crates/montagent/tests/skills.rs`) reads.
"""

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REPORT_TITLE = "Skills: stale workarounds"

MARKER = re.compile(r"workaround:\s*#(\d+)\s*·\s*replaced by:\s*(\S.*?)\s*(?:-->|$)", re.M)
ANY_MARKER = re.compile(r"(?:<!--|#)\s*workaround:", re.I)
# The wording that says "this is temporary": narrow on purpose, so it rarely fires on
# plain prose. Matched with comments stripped, so a marker's own text never trips it.
WORDING = re.compile(
    r"\b(work-?arounds?|fake[sd]?|fake it|faking|hacks?|hacky|for now)\b|\buntil\b[^.\n]*\blands?\b",
    re.I,
)
OPT_OUT = re.compile(r"guard-ok:([^\n>]*)")
COMMENT = re.compile(r"<!--.*?-->", re.S)


@dataclass
class Block:
    file: str
    line: int
    text: str
    section: int  # index of the heading this block sits under
    markers: set = field(default_factory=set)


@dataclass
class Found:
    problems: list = field(default_factory=list)
    markers: dict = field(default_factory=dict)  # issue number -> ["file:line", ...]


def blocks_of_markdown(name, text):
    """Paragraphs and list items (split at blank lines and list bullets), fences left out."""
    blocks, current, section, in_fence = [], None, 0, False
    for n, line in enumerate(text.splitlines(), 1):
        stripped = line.strip()
        if stripped.startswith("```"):
            in_fence = not in_fence
            current = None
            continue
        if in_fence or not stripped:
            current = None
            continue
        if stripped.startswith("#"):
            section += 1
            current = None
            continue
        if current is None or re.match(r"([-*]|\d+\.)\s", stripped):
            current = Block(name, n, "", section)
            blocks.append(current)
        current.text += line + "\n"
    return blocks


def blocks_of_script(name, text):
    """A script's comment and docstring lines, as one block: its header is its prose."""
    return [Block(name, 1, text, 0)]


def scan(files):
    """`files` is [(display name, text, is_script)]."""
    found = Found()
    scripts = {}  # script file name -> marker issues in its header
    all_blocks = []
    for name, text, is_script in files:
        blocks = blocks_of_script(name, text) if is_script else blocks_of_markdown(name, text)
        for block in blocks:
            for raw in ANY_MARKER.finditer(block.text):
                rest = block.text[raw.start():].split("\n", 1)[0]
                parsed = MARKER.search(rest)
                if not parsed:
                    found.problems.append(
                        f"{name}:{block.line}: malformed marker, write "
                        "`workaround: #N · replaced by: …`"
                    )
                    continue
                block.markers.add(int(parsed.group(1)))
                found.markers.setdefault(int(parsed.group(1)), []).append(f"{name}:{block.line}")
        if is_script:
            scripts[Path(name).name] = blocks[0].markers
        all_blocks.extend(blocks)

    # A marker alone in its paragraph covers its whole section.
    covered = {}
    for block in all_blocks:
        if block.markers and not COMMENT.sub("", block.text).strip():
            covered.setdefault((block.file, block.section), set()).update(block.markers)

    for block in all_blocks:
        markers = block.markers | covered.get((block.file, block.section), set())
        opted = {w for m in OPT_OUT.finditer(block.text) for w in m.group(1).split()}
        prose = COMMENT.sub("", block.text)
        if not markers:
            for word in WORDING.finditer(prose):
                if not any(o.lower() in word.group(0).lower() for o in opted):
                    found.problems.append(
                        f"{block.file}:{block.line}: \"{word.group(0)}\" reads as a workaround "
                        "but carries no `workaround: #N` marker "
                        f"(opt out with <!-- guard-ok: {word.group(0).split()[0].lower()} -->)"
                    )
        for script, needed in scripts.items():
            if block.file.endswith(script) or script not in prose:
                continue
            missing = needed - markers
            if missing:
                found.problems.append(
                    f"{block.file}:{block.line}: points at the Workaround script `{script}` "
                    f"without its marker for #{', #'.join(map(str, sorted(missing)))}"
                )
    return found


def skill_files():
    files = []
    for path in sorted((ROOT / "skills").rglob("*")):
        if path.suffix == ".md" or (path.suffix == ".py" and path.parent.name == "scripts"):
            files.append((str(path.relative_to(ROOT)), path.read_text(), path.suffix == ".py"))
    return files


def gh(*args, stdin=None):
    return subprocess.run(["gh", *args], check=True, capture_output=True, text=True, input=stdin).stdout


def closed(markers):
    """The marked issues that have closed, with how: {N: (title, state_reason)}."""
    out = {}
    for number in sorted(markers):
        issue = json.loads(gh("api", f"repos/{{owner}}/{{repo}}/issues/{number}"))
        if issue["state"] == "closed":
            out[number] = (issue["title"], issue.get("state_reason") or "completed")
    return out


def advice(reason):
    if reason == "not_planned":
        return "closed as not planned: drop the marker, the workaround is now the recipe"
    return "closed as completed: switch to the native feature and drop the workaround"


def report(found, stale, dry_run=False):
    lines = [
        "Kept by `ci/skills_workarounds.py --report` (weekly). Each line is a Workaround in "
        "`skills/` whose issue has closed.",
        "",
    ]
    for number, (title, reason) in stale.items():
        where = ", ".join(f"`{w}`" for w in found.markers[number])
        lines.append(f"- [ ] #{number} {title}: {advice(reason)}. Marked at {where}")
    body = "\n".join(lines) + "\n"
    if dry_run:
        print(body if stale else "No stale workarounds.")
        return
    existing = json.loads(
        gh("issue", "list", "--state", "all", "--search", f'in:title "{REPORT_TITLE}"',
           "--json", "number,title,state")
    )
    existing = next((i for i in existing if i["title"] == REPORT_TITLE), None)
    if stale:
        if existing is None:
            gh("issue", "create", "--title", REPORT_TITLE, "--body-file", "-", stdin=body)
        else:
            gh("issue", "edit", str(existing["number"]), "--body-file", "-", stdin=body)
            if existing["state"] != "OPEN":
                gh("issue", "reopen", str(existing["number"]))
    elif existing is not None and existing["state"] == "OPEN":
        gh("issue", "edit", str(existing["number"]), "--body-file", "-",
           stdin="No stale workarounds: every marked issue is open.\n")
        gh("issue", "close", str(existing["number"]), "--comment",
           "Every Workaround marker in `skills/` points at an open issue again.")
    print(body if stale else "No stale workarounds.")


def self_test():
    def problems(md, script=None):
        files = [("skills/x/SKILL.md", md, False)]
        if script:
            files.append(("skills/x/scripts/bake.py", script, True))
        return scan(files).problems

    assert problems("Fake it with opacity. <!-- workaround: #1 · replaced by: blends -->\n") == []
    assert len(problems("Fake it with opacity.\n")) == 1, "unmarked wording fires"
    assert problems("Use the marked workaround. <!-- guard-ok: workaround -->\n") == []
    assert len(problems("Do this until blend modes land.\n")) == 1
    assert problems("## H\n\n<!-- workaround: #2 · replaced by: x -->\n\nA hack here.\n\n## Next\n") == []
    assert len(problems("## H\n\n<!-- workaround: #2 · replaced by: x -->\n\n## Next\n\nA hack.\n")) == 1
    assert len(problems("Odd. <!-- workaround: 2 replaced by x -->\n")) == 1, "malformed marker"
    assert problems("- One. <!-- workaround: #3 · replaced by: y -->\n- A hack.\n") != [], "per list item"
    bake = '"""Bake.\n\n# workaround: #499 · replaced by: parenting\n"""\n'
    assert len(problems("Run `bake.py` to flatten the rig.\n", bake)) == 1, "pointer without marker"
    assert problems(
        "Run `bake.py`. <!-- workaround: #499 · replaced by: parenting -->\n", bake
    ) == []
    assert scan([("a.md", "x <!-- workaround: #7 · replaced by: z -->\n", False)]).markers == {7: ["a.md:1"]}
    assert advice("not_planned").startswith("closed as not planned")
    print("self-test passed")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    mode = parser.add_mutually_exclusive_group(required=True)
    for flag in ("--lint", "--check", "--report", "--self-test"):
        mode.add_argument(flag, action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        self_test()
        return 0
    found = scan(skill_files())
    if args.report:
        report(found, closed(found.markers), args.dry_run)
        return 0
    problems = list(found.problems)
    if args.check:
        for number, (title, reason) in closed(found.markers).items():
            problems.append(f"#{number} ({title}) is {advice(reason)}; marked at {', '.join(found.markers[number])}")
    for problem in problems:
        print(problem)
    print(f"{len(found.markers)} marked issues, {len(problems)} problems")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
