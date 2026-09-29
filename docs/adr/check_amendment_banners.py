#!/usr/bin/env python3
"""Check that every ADR amendment is discoverable from the ADR it amends.

Run from anywhere:  python3 docs/adr/check_amendment_banners.py
Exits non-zero, naming every defect, when it stops reproducing.

Why this exists
---------------
ADR-0065 was decided against a `main` that did not contain ADR-0046 or ADR-0050,
producing a live contradiction on `main` that ADR-0067 had to resolve (#178).
ADR-0067's own diagnosis was that "the rule was already written; what was missing
was anything that enforced it."

The rule is `docs/agents/domain.md`'s: an ADR is amended, never rewritten. The
amendment is recorded in the *amending* ADR's header, which means the *amended*
ADR carries no trace of it unless someone adds one by hand. An agent that opens
ADR-0006 directly and reads to the end learns nothing about the sixteen ADRs that
have since amended it.

So three things must agree, and this script checks all three:

  1. every ADR that declares an amendment names a target that exists;
  2. every amended ADR carries an "Amended by" banner naming every amender;
  3. `README.md`'s *Amended by* column agrees with both.

Two frontmatter conventions are in use and both are read here: YAML
(`amends: 0011 (gloss), 0012`) and prose (`**Amends:** [ADR-0011](...) (gloss)`).
ADR-0016, ADR-0043 and ADR-0044 use the prose form. Tooling that reads only YAML
silently drops those amendments — which is how ADR-0011's banner came to be
missing ADR-0016 and ADR-0006's missing ADR-0043 and ADR-0044.

A fourth thing is checked, added after the first three had been passing clean for
some time over a false sentence:

  4. `README.md`'s opening sentence counts ADRs, and those counts are derived
     here rather than retyped.

It claimed "74 of 92 ADRs declare an amendment: 71 in an `amends:` header" when
the true figures were 84, 102 and 81 — drifted across roughly ten ADRs — while
this script exited 0, because it checked amendment *edges* and never *totals*.
The sentence meanwhile credited this script with keeping "the three views
(header, banner, this column) in agreement". A claim with a verifier named beside
it, where the verifier does not cover the claim, is the defect this repository
keeps finding in itself; the cheap fix is to retype four numbers, and the fix
that holds is to derive them.
"""

import glob
import os
import re
import sys

ADR_DIR = os.path.dirname(os.path.abspath(__file__))


def adr_files():
    return sorted(glob.glob(os.path.join(ADR_DIR, "0*.md")))


def declared_amendments(text):
    """Return the set of 4-digit ADR numbers this document declares it amends."""
    targets = set()

    yaml = re.match(r"^---\n(.*?)\n---", text, re.S)
    if yaml:
        # The whole value, not its first line. A YAML block scalar continues onto every
        # following indented line, and the headers that run to several lines are the ones
        # on the ADRs that amend the most — ADR-0061 declares 0006 and 0034, and reading
        # one line saw only 0006, so its amendment of ADR-0034 was invisible to the check
        # that exists to make amendments discoverable (ADR-0071).
        block = re.search(
            r"^amends:[ \t]*(.*(?:\n[ \t]+.*)*)$", yaml.group(1), re.M
        )
        if block:
            targets |= set(re.findall(r"\b(\d{4})\b", block.group(1)))

    # Prose form, e.g. `**Amends:** [ADR-0011](./0011-...md) (gloss),`
    for line in re.findall(r"^\*\*Amends:\*\*\s*(.*)$", text, re.M):
        targets |= set(re.findall(r"ADR-(\d{4})", line))

    return targets


def banner_region(text):
    """The document's preamble: title plus any leading block-quote banners.

    Stops at the first substantive paragraph, so a passing mention of an ADR
    deep in the body is never mistaken for a banner entry.
    """
    body = text
    yaml = re.match(r"^---\n.*?\n---\n", text, re.S)
    if yaml:
        body = text[yaml.end():]

    kept = []
    for line in body.splitlines():
        if line.startswith(">") or line.startswith("#") or not line.strip():
            kept.append(line)
        else:
            break
    return "\n".join(kept)


def index_rows():
    """`README.md`'s *Amended by* column, keyed by ADR number."""
    rows = {}
    path = os.path.join(ADR_DIR, "README.md")
    if not os.path.exists(path):
        return rows
    for line in open(path):
        m = re.match(r"\|\s*\[(\d{4})\]\([^)]*\)\s*\|.*\|\s*([0-9,\s]*)\|\s*$", line)
        if m:
            nums = set(re.findall(r"\d{4}", m.group(2)))
            if nums:
                rows[m.group(1)] = nums
    return rows


# Small integers are spelled as words in the index's prose, so the check compares like
# with like rather than forcing a digit into a sentence that reads better without one.
NUMBER_WORDS = {
    1: "one", 2: "two", 3: "three", 4: "four", 5: "five",
    6: "six", 7: "seven", 8: "eight", 9: "nine", 10: "ten",
}

HEADLINE = re.compile(
    r"Most of this series amends itself — (\d+) of (\d+) ADRs declare an "
    r"amendment: (\d+) in an `amends:` header, ([A-Za-z]+) \((.*?)\) in an "
    r"`\*\*Amends:\*\*` line",
    re.S,
)


def declaration_forms(texts):
    """Which ADRs *declare* an amendment, split by the convention they use.

    Presence of the declaration, not whether a target could be parsed out of it —
    which is the sentence's own question ("ADRs declare an amendment") and differs
    from the edge check above by exactly one document. **ADR-0021** declares
    `amends: the performance budget stated in the map's Notes (never itself an ADR)`:
    a real amendment of something that is not an ADR, so it contributes no edge and
    is still an ADR that declares one. Counting parsed targets instead reports 83 of
    102 and silently drops it.
    """
    yaml_form, prose_form = set(), set()
    for num, text in texts.items():
        header = re.match(r"^---\n(.*?)\n---", text, re.S)
        if header and re.search(r"^amends:[ \t]*\S", header.group(1), re.M):
            yaml_form.add(num)
        if re.search(r"^\*\*Amends:\*\*", text, re.M):
            prose_form.add(num)
    return yaml_form, prose_form


def headline_defects(texts):
    """Check `README.md`'s opening sentence against the ADRs it counts.

    The sentence says this script keeps the index honest. Until this function existed
    it did not check the sentence at all, and the sentence was wrong in three of its
    four numbers.
    """
    path = os.path.join(ADR_DIR, "README.md")
    if not os.path.exists(path):
        return []
    m = HEADLINE.search(open(path).read())
    if not m:
        return [
            "README.md's opening sentence no longer matches the shape this script "
            "checks, so its counts are unverified. Restore the wording or update "
            "HEADLINE."
        ]

    yaml_form, prose_form = declaration_forms(texts)
    said_declaring, said_total, said_yaml, said_prose_word, said_prose_list = m.groups()

    actual = {
        "ADRs declaring an amendment": (int(said_declaring), len(yaml_form | prose_form)),
        "ADRs in total": (int(said_total), len(texts)),
        "declarations in an `amends:` header": (int(said_yaml), len(yaml_form)),
    }
    problems = [
        f"README.md's opening sentence says {said} {what}, but there are {measured}"
        for what, (said, measured) in actual.items()
        if said != measured
    ]

    word = NUMBER_WORDS.get(len(prose_form), str(len(prose_form)))
    if said_prose_word.lower() != word:
        problems.append(
            f"README.md's opening sentence says {said_prose_word} declarations in an "
            f"`**Amends:**` line, but there are {word}"
        )
    listed = set(re.findall(r"\[(\d{4})\]", said_prose_list))
    if listed != prose_form:
        problems.append(
            "README.md's opening sentence names "
            f"{', '.join('ADR-' + a for a in sorted(listed)) or 'nothing'} as using the "
            "`**Amends:**` form, but the ADRs that do are "
            f"{', '.join('ADR-' + a for a in sorted(prose_form))}"
        )
    return problems


def main():
    texts, amends = {}, {}
    for path in adr_files():
        num = os.path.basename(path)[:4]
        texts[num] = open(path).read()
        amends[num] = declared_amendments(texts[num])

    # Reverse the declarations: who amends whom.
    amended_by = {}
    for num, targets in amends.items():
        for target in targets:
            amended_by.setdefault(target, set()).add(num)

    problems = []

    for target, amenders in sorted(amended_by.items()):
        if target not in texts:
            problems.append(
                f"ADR-{target} is named as amended by "
                f"{', '.join('ADR-' + a for a in sorted(amenders))}, but no such ADR exists"
            )
            continue

        named = set(re.findall(r"ADR-(\d{4})", banner_region(texts[target])))
        missing = amenders - named
        if missing:
            problems.append(
                f"ADR-{target}'s banner does not name "
                f"{', '.join('ADR-' + a for a in sorted(missing))} "
                f"(amended by {len(amenders)}, banner names {len(amenders & named)})"
            )

    problems += headline_defects(texts)

    rows = index_rows()
    for target, amenders in sorted(amended_by.items()):
        listed = rows.get(target, set())
        if not listed:
            problems.append(f"README.md has no Amended-by entry for ADR-{target}")
            continue
        if amenders - listed:
            problems.append(
                f"README.md's row for ADR-{target} omits "
                f"{', '.join('ADR-' + a for a in sorted(amenders - listed))}"
            )
        if listed - amenders:
            problems.append(
                f"README.md's row for ADR-{target} lists "
                f"{', '.join('ADR-' + a for a in sorted(listed - amenders))}, "
                f"which declare no amendment of it"
            )

    if problems:
        print(f"{len(problems)} defect(s):\n")
        for p in problems:
            print(f"  - {p}")
        print(
            "\nEvery amendment must be discoverable from the ADR it amends. "
            "Add an\n\"Amended by\" banner under the amended ADR's title, and a row in "
            "README.md."
        )
        return 1

    total = sum(len(v) for v in amended_by.values())
    yaml_form, prose_form = declaration_forms(texts)
    print(
        f"OK — {len(amended_by)} amended ADRs, {total} amendment edges, "
        f"every one named in both the banner and the index."
    )
    # Printed on success too, so the index's opening sentence is visibly checked rather
    # than merely unreported: it went stale by ten ADRs while this script exited 0.
    print(
        f"     {len(yaml_form | prose_form)} of {len(texts)} ADRs declare an amendment "
        f"({len(yaml_form)} in an `amends:` header, {len(prose_form)} in an "
        f"`**Amends:**` line) — matching README.md's opening sentence."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
