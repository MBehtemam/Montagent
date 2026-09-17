---
status: accepted
amends: 0007 (discharges the fonts-vendor mechanics, licence-check and substitute-policy questions ADR-0007 left open)
---

# `fonts vendor` is a local-only file copy gated by a three-bucket licence check; attestation is keyed by file path, not by fonts-table entry; a refusal may suggest open substitutes but never auto-vendors one; the live fixture defect is a separate follow-up task

[ADR-0007](./0007-text-runs-literal-size-declared-fonts.md) settled that a project
declares fonts as vendored file paths under a top-level `fonts` table — never a
system family name — and named the shape of a `fonts` discovery tool (`fonts list`
/ `fonts vendor`) and the principle behind it (*"wherever this format refuses a
convenience, the convenience belongs in an authoring-time tool whose output is
inert"*). It flagged font licensing as *"a first-class outcome, not an edge
case"* but designed neither the check nor what happens when it fails. That gap
became a live defect: the committed sample project
(`fixtures/en-halloween-decorating/`) declares
`"brand": [{"file": "fonts/SFProRounded-Bold.ttf"}]`, that file does not exist
in the repo, and Apple's SF Pro Rounded is not redistributable outside a
narrow Apple-platform-mockup licence — the fixture is unrenderable as
committed.

## Decision

### `fonts vendor` copies from the local filesystem only — no network, no fetch-by-name

The source is always a font already resolvable as a local file — either
picked from `fonts list`'s enumeration of installed system fonts (paths and
face indices) or a plain local path the caller names directly. There is no
URL argument and no fetch-by-family-name (e.g. a Google Fonts lookup).
Acquiring a font the author doesn't have is a separate concern from vendoring
one they do — the author downloads it by whatever means they like (browser,
`curl`, a package manager), and `fonts vendor` freezes the resulting bytes
into the repo. A pure local-filesystem copy stays trivially auditable,
deterministic, and testable against fixtures, and keeps a nondeterministic,
credential-adjacent, remotely-influenced write out of a tool whose one job is
to commit exact, known bytes. If a network-sourced acquisition tool is wanted
later, it is an explicit, separately-named command (e.g. `fonts fetch`) that
lands a file on disk and stops — never an implicit capability of `vendor`.

Decided **unanimous 3/3** (Opus, Sonnet, Haiku).

### The licence check is a hard gate at copy time, resolved by a three-bucket check, not a warning or a downstream `validate` finding

`fonts vendor` refuses outright — no file lands in the repo — when it
determines a font is non-redistributable; it never copies-with-a-warning and
never defers the check to a later `validate`/`fonts` pass. The moment that
matters is the copy itself: once a font file is committed it is in git
history, has propagated to every clone and CI runner, and a warning is a
control only a human reliably heeds — the primary caller here is an agent,
which reliably acknowledges warnings and moves on. A `validate`-time check
would confirm the violation only after it is already expensive to reverse.

Research done for this ticket confirmed there is no reliable machine-readable
signal for *"may this file be redistributed"* inside a font file. The OS/2
table's `fsType` field governs *document-embedding* permission (may an
application embed this font inside a PDF or Office file that then travels to
other machines) — a different legal question from *"may I copy this file into
a git repo,"* and the two do not correlate: a font can be `fsType`
Installable while its separate EULA still forbids file redistribution, and
vice versa. The check therefore cannot be a single yes/no test; it is a
three-bucket decision that assumes some fonts are unknowable and treats that
honestly:

1. **Known non-redistributable → hard refuse, no override.** A font on
   Montaget's own blocklist (below) is refused unconditionally. No flag lifts
   this, because an override affordance is itself the thing that makes the
   project a knowing party to an illegal copy — the caller most likely to
   reach for `--yes-i-know` is exactly the caller the blocklist exists to
   stop.
2. **Known redistributable → copy silently, record the licence.** `fonts
   vendor` reads the font's `name` table records for IDs 13 (License
   Description) and 14 (License Info URL); when that text matches a
   recognisable open-licence pattern (SIL Open Font License, Apache License
   2.0, Ubuntu Font Licence, and similarly unambiguous cases), it vendors the
   file and records the matched identifier as the attestation. This is a
   heuristic string match, not a legal determination — it exists to make the
   common, genuinely-open case (most of Google Fonts, most icon/display
   fonts) frictionless, and it fails safe: anything it doesn't confidently
   recognise falls to bucket 3.
3. **Unknown or unparsable (the common case) → refuse until the caller
   declares a licence.** Most fonts carry no usable license metadata at all.
   `fonts vendor` refuses by default and requires an explicit
   `--licence <identifier>` argument to proceed — turning detector ignorance
   into a recorded, attributed assertion rather than a silent guess. A false
   declaration is the declarer's liability, recorded in the file; a silent
   copy with no declaration would have been the project's.

Decided **unanimous 3/3** on hard-refuse-at-copy-time; the three-bucket shape
adopted from one juror's (Opus's) amendment, which the fsType/redistribution
research made necessary rather than optional — a binary check has nothing
reliable to test against for the common (bucket 3) case.

### The blocklist (bucket 1) is small, hardcoded, shipped by Montaget, and not user-removable

The blocklist is a short, versioned, code-reviewed list of known-non-
redistributable font families, seeded with the one confirmed case in hand —
**Apple's system fonts**: SF Pro, SF Pro Text, SF Pro Display, SF Pro
Rounded, SF Compact, SF Mono, New York, and their private on-disk names
(`.SF NS *`, `.AppleSystemUIFont`) — matched on family name and PostScript
name, case- and whitespace-insensitive. Apple's font licence (its Design
Resources / SF font licence terms) restricts these fonts to creating UI
mockups for software running on Apple's own platforms and explicitly
forbids embedding them in other software products; they are not a
redistributable asset by any reading in general use.

The list grows only by PR against a citable licence clause — never
speculatively, never from an external database, community registry, or
runtime fetch. A user or project config may **add** stricter entries (an
organisation's own "never vendor these" policy) but may never remove or
disable a shipped one; a user-removable hard gate is not a gate. Bucket 1
does not need to be comprehensive to be sound, because bucket 3's
default-refuse-until-declared already catches everything not explicitly
known either way — bucket 1 exists only to convert the handful of cases
where the honest answer isn't "refuse until declared" but "refuse, full
stop, there is no declaration that makes this legal."

Decided **unanimous 3/3**, independently converging on the same seed list.

### A refusal may name known open substitutes; it never auto-vendors one

When `fonts vendor` refuses (bucket 1, or bucket 3 with no declaration
supplied), it may additionally name fonts it knows of that are open and
broadly similar in style — advisory output only, never an action the tool
takes on the author's behalf. The author still makes the call, and the
fonts-table edit that follows is their declared change, keeping every entry
in the `fonts` table traceable to a human decision rather than a tool
picking a fallback under cover of a refusal.

This was the panel's one real split (2–1 for auto-vendoring a substitute
under the same fonts-table key, forcing the mandatory font-swap
re-verification ADR-0007 already requires). It is resolved for
suggest-only because the research done for this ticket makes the majority's
premise weaker than it looked: **no font claims formal metric compatibility
with SF Pro Rounded** — unlike the classic Liberation-Fonts-for-Arial case,
the two candidates found (Open Runde, Roboto-Round) are explicitly
style-alikes with no advance-width guarantee. Auto-vendoring a
non-metric-compatible font under an unchanged key would silently invalidate
every hand-tuned size and line-break the original font's metrics produced —
exactly the failure mode ADR-0007's font-swap census exists to catch, now
triggered *by the licence tool itself* rather than by an author's deliberate
choice. A suggestion must therefore be labelled honestly by its actual
basis — metric-compatible, same-classification, or merely visually
similar — and must never be phrased in language that implies a swap is safe
by default.

Decided **2–1** (Sonnet and Haiku for auto-vendor-under-same-key; Opus for
suggest-only); resolved by the map author for suggest-only, for the research
reason above.

### The attestation (licence identifier, source, file hash) is keyed by file path, in a table separate from the `fonts` reference structure — not inlined on each chain entry

`fonts vendor` records what it learned — the matched or declared licence
identifier, where the file came from, and its content hash — so `validate`
can later perform a cheap, purely local integrity check (does the committed
file's hash still match what was vendored) without re-adjudicating licence
law on every run. The question was where that record lives, given the
`fonts` table's own worked example already reuses one file across two keys:

```json
"fonts": {"brand": [{"file": "fonts/Inter-SemiBold.ttf"}],
          "brand+fa": [{"file": "fonts/Inter-SemiBold.ttf"}, {"file": "fonts/Vazirmatn-SemiBold.ttf"}]}
```

Attaching the attestation once per fonts-table *key* was rejected 3/3 outright
— a fallback chain mixes independently-licensed files by design (`brand+fa`
pairs an OFL Inter release with an independent Vazirmatn one), so a
per-key attestation cannot state a single truth for a chain that has none.

Attaching it *per chain entry*, inline on each `{"file": ...}` object, is
where the panel split 2–1: correct on granularity, but the fonts table is a
*reference* structure, not a storage structure, and `Inter-SemiBold.ttf`
already appears under two keys in ADR-0007's own example. Inlining
duplicates the same hash and licence at every point of reference; duplicated
facts drift the moment one copy is re-vendored and the other isn't, and
`validate` would then have to arbitrate between two disagreeing attestations
for one path — precisely the adjudication this whole design exists to avoid.

The decision is a **separate, top-level table keyed by file path**:

```json
"fontVendor": {
  "fonts/Inter-SemiBold.ttf":    {"licence": "OFL-1.1", "source": "Google Fonts", "sha256": "…"},
  "fonts/Vazirmatn-SemiBold.ttf": {"licence": "OFL-1.1", "source": "Google Fonts", "sha256": "…"}
}
```

One path, one hash, one licence — the invariant is structural rather than a
convention chain entries must remember to uphold. `fonts vendor` writes one
entry per vendored file regardless of how many `fonts`-table keys later
reference it; `validate`'s check becomes *"every path any chain references
resolves to a `fontVendor` entry whose hash matches the bytes on disk,"* a
purely local, deterministic comparison. A path referenced by no remaining
chain (its last referencing key deleted) is a `validate` **warning**, never a
silent drop — an orphaned attestation may still be wanted, and pruning it is
a decision an author makes, not one `fmt` makes for them.

Decided **2–1** for per-file granularity (Sonnet, Haiku voted plain per-entry
inlining; Opus's dissent on *placement*, not granularity, is adopted) —
resolved for the path-keyed table because the majority's own worked example
already falsifies naive inlining before this design would even ship.

### The licence check runs only at `fonts vendor` time; `validate` re-checks integrity, never licence law

Once vendored, a `fontVendor` entry is evidence of a decision made at a
point in time, not a claim the tool re-adjudicates on every run. `validate`
performs the local, deterministic half of that — the recorded `sha256`
still matches the committed file's bytes, and every `fonts`-table reference
resolves to an attestation — and reports a **warning**, not a licence
re-verdict, when a hash mismatches or an entry is missing (most likely a
font hand-dropped into the repo bypassing `fonts vendor` entirely, which is
this project's own accepted authoring-model bypass — an agent editing with
ordinary file tools already bypasses `validate` and `render` in exactly the
way ADR-0006 accepts). `validate`/`render` never re-verify the licence claim
itself: doing so has only two possible shapes, and both are worse than not
checking — a hash-to-known-font whitelist fails closed on every legitimately
licensed font not in it (a purchased commercial font, a corporate brand
face), and a live licence-oracle call breaks offline, deterministic
rendering and lets a third party's unrelated data change silently turn a
previously-working committed project into a failing one tomorrow.

If licence drift genuinely needs catching later (a licence is revoked, or a
vendor-time judgement turns out wrong), that is a separately-evidenced,
explicit, opt-in audit — not the hot path of every `validate`/`render` run.

Decided **2–1** (Opus and Haiku for vendor-time-only with the hash-check
refinement above; Sonnet dissented for re-verifying licence status on every
run, reasoning that a one-time check lets the file drift into shipping an
unlicensed font under a green `validate`). Resolved for vendor-time-only
because Sonnet's own drift concern is answered by the hash-integrity check,
which is deterministic and local — the part of "re-verify" that is actually
achievable without either a stale whitelist or a live network call.

### Fixing the live fixture defect is a separate follow-up task, not part of this ticket

This ticket settles the design; it does not execute it. Re-vendoring a real,
legally redistributable font into `fixtures/en-halloween-decorating/` and
updating its committed project file is concrete, checklist-executable work
that requires no further design conversation once this ADR lands — exactly
what the map's `task` ticket type exists to hold, distinct from the
`grilling` type that resolved this one. The defect is pre-existing and
independently discoverable regardless of this ticket's existence; folding
its resolution into this ticket's closure would make "the design is settled"
and "the fixture got fixed" the same event; they are not, and the map's own
`prototype` type — not `grilling` — is where exercising an unproven design
against real material belongs.

Decided **unanimous 3/3**.

## Consequences

- `fonts vendor <path-or-system-pick> [--licence <identifier>]` is the whole
  surface: a local file, an optional explicit licence declaration for bucket
  3, and blocklist/heuristic checks running before any bytes are copied.
  `fonts list` additionally surfaces each enumerated system font's detected
  licence status (blocklisted / recognised-open / unknown) so an author sees
  a doomed vendor attempt before making it.
- The project schema gains one new top-level table, `fontVendor`, keyed by
  the same file paths the `fonts` table's chains reference. It is written
  only by `fonts vendor`, never hand-edited under this project's own
  authoring model (though nothing enforces that beyond convention, same as
  every other write-tool-shaped part of the format).
- `validate` gains: a **font-attestation** check (every `fonts`-table path
  resolves to a `fontVendor` entry whose `sha256` matches the file on disk —
  `error` when it does not, since a silent post-vendor edit or hash mismatch
  is exactly the drift ADR-0006 exists to catch) and an **orphaned-attestation**
  finding (`fontVendor` entry with no remaining `fonts`-table reference —
  `note`, never auto-pruned).
- The committed sample fixture remains a known, tracked defect —
  unrenderable as committed — until the follow-up `task` ticket lands.
- ADR-0007's *"Font licensing is a first-class outcome, not an edge case"*
  consequence is now fully discharged; nothing about fonts is left open on
  this map except what future evidence surfaces.

## Evidence

Two rounds, three jurors each (Claude Opus 5, Claude Sonnet 5, Claude Haiku
4.5), independent, blind to each other's ballots, no assigned stance. Full
ballots at
[`docs/research/juries/font-vendoring/BALLOTS.md`](../research/juries/font-vendoring/BALLOTS.md).
Round one: unanimous 3/3 on local-filesystem-only vendoring and hard-refuse
licence gating; split 2–1 on substitute policy (suggest-only vs.
auto-vendor-under-same-key) and on re-verification scope (vendor-time-only
vs. re-verify-on-every-run). Round two: unanimous 3/3 on the blocklist's
shape and seed list, and on the fixture fix being a separate follow-up task;
split 2–1 on attestation placement (per-chain-entry vs. a separate
path-keyed table) — all three jurors agreed on per-file granularity, the
split was placement only. A companion research pass established that OS/2
`fsType` governs document-embedding, not file-redistribution rights; that
Apple's SF Pro / SF Pro Rounded licence forbids general redistribution; that
no font claims formal metric compatibility with SF Pro Rounded (only
style-alikes, e.g. Open Runde, Roboto-Round); and that existing tools
(Fontsource, google-webfonts-helper) rely on curating already-open catalogs
rather than performing automated licence detection — informing the
three-bucket check design and the suggest-only resolution above.
