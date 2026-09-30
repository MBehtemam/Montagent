---
status: accepted
amends: 0100 (its §7 known limit, *"it does not make the shared census renderer name its members"*, is discharged: `E-TRACK-OVERLAP` is Named, so a text-only reader gets up to three ids per knot and the count of the rest, not only the knot's extent), 0043 (how a sibling census renders in the text form is now decided once per code in the registry, beside the repair form this ADR already fixes per code; the census itself, and the rule that it is never ordered or worded by size, are unchanged)
---

# A census names its members where its value cannot be searched for

> **Amended by [ADR-0120](./0120-a-fact-that-bears-on-one-branch-of-a-refuse-fork-is-a-repair-by-another-name.md)**, which discharges §6: `E-FONT-NO-GLYPH`'s
> census-shaped field is dropped rather than renamed, so the code carries no census and the
> registry has no exception to *"every code that carries a census declares a mode"*.

**Ticket:** [#470](https://github.com/MBehtemam/Montagent/issues/470), building
[#427](https://github.com/MBehtemam/Montagent/issues/427)'s
[rulings](https://github.com/MBehtemam/Montagent/issues/427#issuecomment-5887547798), on the
map [#383](https://github.com/MBehtemam/Montagent/issues/383).

## The gap

`crate::text` rendered every census as counts: `census <field>: <n> at <value>, …`. The member
ids lived only in the canonical JSON. ADR-0100 §7 recorded this as a known limit for
`E-TRACK-OVERLAP`: *"a text-only reader gets the track, the count, the knot count and each
knot's extent, and must open the canonical JSON or the document to read the fifteen names."*

Counts are enough when the value leads the reader to the members. `census height: 4 at 1597,
1 at 1537` narrows a search, because `1537` is written in the document and `grep` finds the
one element that carries it. They are not enough when the value cannot be searched for:

- `N-TEXT-INVISIBLE` groups by an invisible codepoint, and its own template says *"no diff will
  show you where"*.
- `N-TEXT-MIXED-NORMALIZATION` groups by `NFC` and `NFD`. The two spellings look identical, and
  no search the reader can type finds one without the other.
- `E-TRACK-OVERLAP` groups by `a..b`, a stretch of the clock. The document holds `start` and
  `end` on each element, never the stretch, so the value narrows the clock and not the
  document. #384 asked for *"the set, not a pointer"*, and the text form gave neither.
- `E-RETIRED-KEY` groups some keys by `clip` and `origin` objects, which `grep` cannot reliably
  find across spellings and whitespace.

For these four, `1 at NFC, 1 at NFD` tells a reader that two spellings exist and gives them
no way to find either one without leaving the report.

The canonical JSON was never short of anything: every member of every group is there. This is
a gap in the text form only.

## Decision

### 1. Naming is decided per code, by one test

**Can the reader get from a group's value to its members using only the document and a
shell?** A value written literally in the document passes, and its census is **Counted**. A
value that cannot be searched for fails, and its census is **Named**.

**Running another verb does not pass the test.** `query` can select elements by track and
span, so it can turn `0..5000` into ids. If that counted, every value would pass through some
verb, nothing would ever be Named, and the rule would be count-only again. A count narrows
only when its value is a handle on the document itself.

### 2. The choice lives in the registry

`CheckSpec` gains `census: Option<CensusMode>`, with `CensusMode::{Counted, Named}`, next to
the repair form ADR-0043 already fixes per code. The mode is decided once, by whoever writes
the check, and holds for every instance.

**It is not inferred from the value's JSON type**, because being searchable does not follow
from the type. A height is a number and can be searched for. A codepoint name is a string and
cannot, since what the document holds is the codepoint, not its name. And `E-RETIRED-KEY`
mixes objects and strings under one code, so a rule keyed on type would split one code across
both modes.

A registry test reads the crate's source and checks that **every code that attaches a census
declares a mode**, and that no code declares a mode without attaching one.

### 3. The classification

| Code | Mode | Why |
|---|---|---|
| `N-TEXT-INVISIBLE` | **Named** | the value is an invisible codepoint |
| `N-TEXT-MIXED-NORMALIZATION` | **Named** | NFC and NFD bytes look identical, so they cannot be searched for |
| `E-TRACK-OVERLAP` | **Named** | `a..b` narrows the clock, not the document |
| `E-RETIRED-KEY` | **Named** | its `clip`/`origin` values are objects. The mode is per code, so its font-key groups are named too, which is harmless redundancy |
| `R-BOX-SLACK` | Counted | the height is written literally in the document |
| `N-FONT-CENSUS` | Counted | the font key is written literally on each element or run |
| `R-FONT-SWAP` | Counted | the same, and its template prints `{font}` |
| `D-BOUNDARY-CLUSTER-DRIFT` | Counted | its template already prints `{members}` |
| `E-FONT-NO-GLYPH` | *no mode* | not a census; see §6 |

`R-FONT-SWAP`'s source carried a comment saying *"the census below carries the names, which a
sentence cannot"*. The text form never printed those names. The comment now says what is
true: the names are in the canonical JSON, and the text form does not need them, because the
font key can be searched for.

### 4. Named prints at most N = 3 members per group, and N + 1 prints all of them

A Named group renders as `<n> at <value> (<id>, <id>, <id>, +K more — see --json)`:

- The members come **in the group's own order**, the declaration or clock order the check built
  them in, and are **never sorted**. A sorted prefix would be a selection, and ADR-0043 forbids
  a census that ranks its members.
- **A group of exactly N + 1 = 4 members names all four.** `+1 more` would hide one id and save
  nothing, so the marker appears only when K ≥ 2.
- The marker names `--json` on every verb, because `--verbose` expands collapsed findings and
  does not expand a group. The canonical JSON is where the rest of the members are.

**N is unmeasured.** It is 3 because three is the smallest number of members that shows what
they have in common, such as a track or an id prefix. One member shows no pattern and reads
like a representative pick, which leans towards the ranking ADR-0043 forbids.

**This N coincides with ADR-0099's N and does not inherit it.** #388 ruling 3 made the census
and the repetition collapse two separate mechanisms, tested by *does one document fact produce
many findings, or do many document facts share a code?* The two constants are separate in the
source (`NAMED_MEMBERS` and `FULL_INSTANCES`), and changing either one does not move the other.

### 5. The bound is O(groups × N), and the group count is not capped

Naming is flat in the number of members per group and linear in the number of groups.

**This is not a new growth order, because the census line was never flat in element count.**
`E-TRACK-OVERLAP` already printed one `n at a..b` entry per knot, and a track of `e` elements
can hold up to `e / 2` knots. Naming multiplies each entry by a constant and changes nothing
else. What ADR-0099 and ADR-0100 assert is that the **line count** does not move with element
count, and it still does not: a census is one line however many groups or members it has.

**Groups are named, not capped.** Cutting groups off would hide part of the partition, and the
partition is what makes a census a narrowing. `E-TRACK-OVERLAP` is error-class, and there is no
`--verbose` route to recover a hidden group from it. A cap here would also settle the tool's
growth policy locally while the map's cross-verb growth-order question is still open. The group
count joins that question, as ADR-0099 did with `CACHE`.

### 6. `E-FONT-NO-GLYPH` is outside the rule

It attaches a `census` field, but its grouping is not a census. Every group holds the same list
of missing codepoints, so it partitions nothing (the glossary's **Census**: *"every sibling sits
in exactly one group"*). Its groups are alternative fonts, which is one inference away from
*"switch to this font"*, the repair a refuse-class finding may not state. It declares no mode,
renders as before, and whether it is reshaped is left open on the map.

### 7. `query` is deliberately outside the rule

- **`query`'s `absent` list stays unbounded.** ADR-0030 settled that the line exists, and its ids
  are the answer the reader asked for, not an unsolicited narrowing.
- **`query`'s own census stays Counted.** It has no code to declare a mode on, and the reader
  chose the field: filtering by the value is the verb they are already in.

Both are stated here so that nobody later "fixes" them for consistency, or cites them as
precedent for bounding or naming.

### 8. Note-class censuses are unchanged in reach

`N-TEXT-INVISIBLE` and `N-TEXT-MIXED-NORMALIZATION` are note-class, so they still collapse to
one counted line unless `--verbose` is passed (ADR-0006, ADR-0099). **Named means "named when
expanded".**

## Consequences

- **This is a text-form change only. The canonical JSON is unchanged**: same fields, same groups,
  same members, and no new key. Consumers of `--json` and the MCP `json` parameter see nothing
  new.
- **Counted codes render byte-identically to before.** A census line changes only on the four
  Named codes, and only by the parenthesised members appended to each group.
- **A text-only reader of `E-TRACK-OVERLAP` can now find a knot's elements without leaving the
  report**: up to three ids per knot, and the count of the rest. ADR-0100 §7's known limit is
  discharged.
- **The census line grows by a constant per group.** Report size stays flat in members per group
  and in line count, and remains linear in the number of groups, as it already was.
- **A new census-bearing check cannot ship without a mode.** The registry test reads the source,
  so attaching a census to a new code without deciding its mode fails the build.

## Evidence

Re-executable: `cargo test -p montagent-core --test registry --test time --test runs`.

- `every_code_that_carries_a_census_declares_how_it_renders`: the source scan finds exactly the
  nine codes of §3, every one except `E-FONT-NO-GLYPH` declares a mode, and no code declares a
  mode without attaching a census.
- `the_codes_whose_value_cannot_be_searched_for_are_named`: §3's classification, pinned.
- `a_fifteen_element_knot_names_three_members_and_counts_the_rest`: MONTAGENT-5's fifteen
  elements print `15 at 0..10000 (e00, e01, e02, +12 more — see --json)`.
- `a_four_member_group_names_all_four`: four members print all four with no marker, and five
  print three and `+2 more`.
- `mixed_normalization_names_an_element_in_each_form_under_verbose`: `1 at NFC (composed), 1 at
  NFD (decomposed)`.
- `the_collapsed_finding_is_flat_in_the_number_of_overlapping_elements` (ADR-0100's test,
  unchanged): 15 against 45 overlapping elements still give an identical line count.
- **Byte-identity of the Counted codes, checked against `main`.** `validate --verbose` was run with
  both binaries on 1,273 distinct projects left behind by this repo's own test suite, 126 of them
  carrying a census line (`R-BOX-SLACK` 226 lines, `E-TRACK-OVERLAP` 96, `E-RETIRED-KEY` 24,
  `N-TEXT-INVISIBLE` 7, `N-FONT-CENSUS` 5, `N-TEXT-MIXED-NORMALIZATION` 3). 1,165 reports were
  byte-identical. The other 108 differed **only** on census lines of the four Named codes, each by
  the members appended in parentheses, with the same line count. **The corpus exercises neither
  `R-FONT-SWAP` nor `D-BOUNDARY-CLUSTER-DRIFT`**. Those two render through the same Counted arm,
  which is the code path that was there before. A first run reported 54 further mismatches that
  did not reproduce on a second run over the same corpus. They are attributed to test temp
  directories being rewritten by a concurrent test run, and nothing was changed between the two
  runs.
