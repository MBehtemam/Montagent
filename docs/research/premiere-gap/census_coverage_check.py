#!/usr/bin/env python3
"""Re-executable completeness check for the Premiere gap inventory (#895, map #882).

Every page in CENSUS.md must be accounted for: it appears in the `## Pages covered`
table of some area file in this directory, or (for `not-a-feature` and `whats-new`
pages) in INVENTORY.md's own "Pages that describe no feature" table. Exits non-zero
and names each unaccounted page otherwise. Also fails if an area file's Pages covered
table names a URL the census does not contain, so a typo cannot pass as coverage.
"""
import pathlib, re, sys

HERE = pathlib.Path(__file__).resolve().parent
URL = re.compile(r"https://helpx\.adobe\.com/premiere/desktop/[^\s|)>\]]+\.html")


def census():
    rows = {}
    for line in (HERE / "CENSUS.md").read_text().splitlines():
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) == 7 and cells[0].isdigit():
            rows[cells[4]] = cells[6]
    return rows


def covered_in(path, heading):
    m = re.search(r"^" + re.escape(heading) + r"\s*$", path.read_text(), re.M)
    if not m:
        return set()
    section = path.read_text()[m.end():]
    j = section.find("\n## ")
    if j >= 0:
        section = section[:j]
    return set(URL.findall(section))


def main():
    pages = census()
    if not pages:
        sys.exit("CENSUS.md has no rows")
    seen = set()
    for f in sorted(HERE.glob("*.md")):
        if f.name in ("CENSUS.md",):
            continue
        heading = "## Pages that describe no feature" if f.name == "INVENTORY.md" else "## Pages covered"
        seen |= covered_in(f, heading)
    missing = sorted(u for u in pages if u not in seen)
    stray = sorted(u for u in seen if u not in pages)
    for u in missing:
        print(f"UNACCOUNTED [{pages[u]}] {u}")
    for u in stray:
        print(f"NOT IN CENSUS {u}")
    print(f"{len(pages)} census pages, {len(pages) - len(missing)} accounted for, "
          f"{len(missing)} unaccounted, {len(stray)} stray")
    sys.exit(1 if missing or stray else 0)


if __name__ == "__main__":
    main()
