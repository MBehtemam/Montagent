---
status: accepted
amends: 0007 (its `index` bullet is implemented rather than changed — the field it declared "for `.ttc` collections, defaulting to 0" was parsed, validated, hashed and then dropped, which is the state its own "a field the renderer cannot honour is worse than no field" bullet forbids; the default is also restated as an *applied* default, since an omitted `index` was not face 0 either)
---

# A `.ttc` chain entry is cut down to its face before it is registered

**Ticket:** [#390](https://github.com/MBehtemam/Montagent/issues/390) (MONTAGENT-6).

## The gap

ADR-0007 declares the field and counts the reason:

> **`index`** for `.ttc` collections, defaulting to 0. 49 of the fonts in a stock macOS
> `/System/Library/Fonts` are collections; this is the normal case.

Three bullets earlier, the same ADR rules on what must never happen to such a field:

> a field the renderer cannot honour is worse than no field — it reads as declarative and is
> **silently ignored, which is the failure this format exists to prevent.**

`index` was that field. It was parsed, range-checked, hashed into `fontVendor`, round-tripped
through `fmt`, printed by `fonts list` as *"the `index` a `.ttc` entry in the `fonts` table
takes"* — and read by no code that chose a face.

**The mechanism is not the one #390 states, and the correction matters.** #390 says *"face 0
loads regardless."* It does not. `fontique::Collection::register_fonts` registers **every**
face of a collection into the one synthetic family, and `parley` then chooses between them by
a CSS attribute query — width, style, weight — which ADR-0007's format deliberately does not
carry (*"No `weight`, no `bold`. A different weight is a different file."*). So the query runs
at its defaults and **the winner is a property of the file's other faces**, not of the
document. Measured on a stock `Avenir Next.ttc`, shaping `Handgloves` at size 100:

| declared `index` | 0 | 1 | 6 | 7 | 10 | 11 |
| --- | --- | --- | --- | --- | --- | --- |
| the whole collection registered, as shipped | 546.90 | 546.90 | 546.90 | 546.90 | 546.90 | 546.90 |
| that face alone | 582.70 | 576.90 | 545.40 | **546.90** | 530.40 | 523.80 |

546.90 is face **7** (Regular). Not face 0 (Bold, 582.70), and not the declared one. A
collection carrying no 400-weight face would land somewhere else again — which is why "face
0" is the wrong diagnosis to fix against: there is no fixed wrong answer, only an answer
computed from the wrong inputs.

**The omitted `index` was wrong by the same mechanism**, and #390 does not mention this at
all. ADR-0007 defaults `index` to 0, and leaving the collection whole is not that default —
it is the same attribute query. On `Avenir Next.ttc` an entry with no `index` field resolved
to face 7 exactly as `"index": 0` did.

**The sharpest symptom is a split, not a silence.** `checks/fonts.rs` already passed the
declared index to `Charmap::of`, so ADR-0007's glyph-coverage `error` was computed against
face N while the renderer drew the face the attribute query won. The two halves of the tool
disagreed about which face the project declared: a project could validate clean on a face it
never rendered, and render tofu from a face no check ever read.

## Decision

### 1. `index` is honoured, not refused

#390 offers refusal as the alternative — make a chain entry carrying `index` an error under
[#384](https://github.com/MBehtemam/Montagent/issues/384)'s severity decision, and name
`fonts vendor` as the workaround. Refused, on three grounds.

- **ADR-0007 ratified the field on a counted reason**, and nothing has changed about the
  count. Refusing repeals an accepted decision; honouring implements it.
- **The named workaround does not exist.** `fonts vendor` is *"a pure filesystem copy"* by
  design (ADR-0057), so it cannot produce a standalone `.ttf` from a `.ttc`. Refusing `index`
  would leave an author whose only copy of a face is a collection — the macOS normal case,
  by ADR-0007's own count — with **no route at all**, until `vendor` grew the very face
  extraction this decision puts in the renderer. The refusal is the larger change, not the
  smaller one.
- **ADR-0007 rejected `weight: "bold"` *"naming the replacement"*.** On a stock macOS font
  the replacement it names is a collection index. Refusing `index` withdraws the replacement
  and leaves the rejection pointing at nothing.

### 2. The face is cut out of the collection, because it cannot be selected inside one

`Collection::unregister_font` addresses a face by `(width, style, weight)` and **not by
index**, so narrowing a registered collection to one face is not an operation that can name
which. On the stock `Avenir Next.ttc` the triple is not even unique: faces 0 and 1 (Bold,
Bold Italic) both register as `(1.0, Normal, 700)`, and faces 10 and 11 (Ultra Light, Ultra
Light Italic) both as `(1.0, Normal, 275)` — `fontique` reads neither italic's slope. The
approach fails on the exact file the field exists for, so it is ruled out by measurement
rather than by preference.

What is left is to hand `fontique` a file containing one face, so the attribute query has
nothing to choose between. A collection is a header pointing at N table directories over one
pool of tables, so the face **already is** a complete table directory: `sfnt::face` copies
that directory and the tables it names into a standalone sfnt. Per-table checksums are copied
verbatim — a table's checksum covers its own bytes, which do not change — and
`head.checkSumAdjustment`, the one field that is genuinely about the *file*, is recomputed.
Leaving the collection's value would ship a number describing a file this one is not, which is
this ADR's own defect in miniature.

**A single-face file is passed through borrowed and unparsed beyond its header**, which is
every font in this repository and most of any project. Only a collection allocates.

### 3. The default is applied, not skipped

`index.unwrap_or(0)` at the registration site: an omitted `index` on a collection means face
0 and is cut out exactly as `"index": 0` is. Ruling 2 makes the two spellings the same
request; this ruling is what stops "no field" from silently meaning "no cut".

### 4. An out-of-range `index` stays a refusal, and now says how many faces there are

`it has no face at `index` 3 (it carries 3)` — unchanged in shape from the check this
replaces, and now reached from the file's own face count rather than from the set `fontique`
happened to register. A single-face file carries face 0 and refuses any other index with the
same wording, rather than a second phrasing for the same mistake.

### 5. No new finding code, and this is not a #384 world effect

#384 rules that a render-time world effect gets one finding code per reason. This produces
none, and the reason is that **after ruling 1 there is no world effect left**: the field is
honoured, so nothing is dropped, unhonoured or clobbered at render time. The two failure modes
that remain — a bad index, a bad file — are refusals at registration, already
`E-FONT-MISSING`/`E-FONT-UNREADABLE`'s territory, and were already errors before this change.
The gap #390 logged was that the field reached no code path, and the fix is that it does.

## Consequences

- **Projects declaring a `.ttc` will render differently, and that is the point.** Every
  element whose chain entry names a collection — with or without an `index` — has been
  shaping in a face chosen by an attribute query. It now shapes in the declared face. Advance
  widths move, so **hand-tuned `size` and hand-placed breaks are unverified** for those
  elements, which is precisely the condition ADR-0007's font-swap census exists for. No
  census fires here: the `fonts` table has not changed, only what it resolves to. **Authors
  with a `.ttc` chain entry should re-run `measure`.**
- **No project in this repository is affected.** Every committed font is a single-face file
  and takes the borrowed pass-through path, which is why the workspace suite is green
  unchanged and why the fixture below had to be built rather than found.
- **No schema change, no `fontVendor` change, no sidecar change and no re-probe.** The field
  already existed and was already hashed; this decision is entirely about what reads it.
- **`fonts vendor` stays a byte copy.** Honouring `index` is what lets it: an author vendors
  the whole collection and names the face in the chain. Whether `vendor` should also be able
  to freeze a single face out of a collection is a separate question and is **not** decided
  here — the extraction now exists to serve it if it is ever wanted.
- **An extracted face is a new file that no attestation covers.** It lives only in memory for
  the length of a measurement and is never written, so `fontVendor`'s path-keyed hash still
  describes the file on disk. Anything that later *writes* an extracted face — `vendor
  --face`, were it added — acquires an attestation question this ADR does not answer.

## Evidence

Re-executable: `cargo test -p montagent-text --test ttc_face`.

The collection under test is **built by the test from committed faces**, not read from the
machine. ADR-0007 exists because a test that reads `/System/Library/Fonts` passes on its
author's machine and asserts nothing anywhere else, and five of ADR-0064's six tier-1 targets
have no `Avenir Next.ttc` at all — which is the shape of failure this ticket is about. Face 0
of the fixture is the only non-Regular face and is CFF where the other two are `glyf`: the
first so an omitted `index` has somewhere wrong to land, the second so the extraction cannot
be relying on a table set that happens to be the same in every face. It is the stock
`Avenir Next.ttc` arrangement — Bold at 0, a Regular further down.

- `each_declared_index_shapes_in_the_face_it_names` — the ruling. Each face of the built
  collection measures as the file it was built from. **Fails before the fix** (600.0 against
  581.14).
- `an_omitted_index_is_face_zero_and_not_whatever_an_attribute_query_wins` — ruling 3. **Fails
  before the fix**, and would have passed vacuously on a fixture whose face 0 were Regular,
  which is why the fixture's order is asserted rather than incidental.
- `the_fixture_collection_has_faces_worth_telling_apart` — the anti-vacuity check: every pair
  of fixture faces must measure differently, or every assertion above would hold with the
  index still ignored.
- `validates_coverage_face_and_the_renderers_shaping_face_are_the_same_face` — the split named
  in *The gap*. Coverage and metrics are two readings of *which face is this*, and they now
  agree through two different call sites, so the agreement is asserted rather than assumed.
- `an_index_past_the_end_names_no_face_and_says_how_many_there_are`,
  `a_single_face_file_takes_index_zero_and_refuses_any_other`,
  `a_file_that_is_not_a_font_is_refused_as_one` — ruling 4, and that a non-font still reads as
  a non-font rather than as a missing face.

The `Avenir Next.ttc` numbers in *The gap* are **not** re-executable in CI, by the same
argument that built the fixture: they are a reading of one machine's font book, taken to
establish the mechanism, and the portable statement of the same fact is the test list above.
