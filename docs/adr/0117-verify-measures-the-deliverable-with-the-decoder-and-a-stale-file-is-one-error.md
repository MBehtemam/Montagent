---
status: accepted
amends: 0011 (a tenth MCP verb and a thirteenth CLI command, `verify`; the counts move from 9 / 12 to 10 / 13, asserted in `crates/montagent/tests/adapters.rs`. ADR-0011's CLI-only rule is for rarely used verbs, and this one is the last step of every agent's workflow), 0104 (the stamp moves to `montagent/2` and gains a staleness digest and the engine version; ownership is still the project identity alone, so `render`'s pre-flight reads `montagent/1` and `montagent/2` alike as `Mine`, and render's output bytes change again), 0093 (states why ruling 3 does not forbid a second witness — one file, two different questions — and adds `verify` to `render`'s `NOT CHECKED` rather than to `render`, so *a file at the output path is a zero-error render* is untouched), 0006 (the `NOT CHECKED` block may be followed by a verb's own limits, under `not_checked_also`; the block's sentence is not edited)
---

# `verify` measures the deliverable with the decoder, and a stale file is one error

> **Amended by [ADR-0119](0119-verify-s-measurement-is-a-fifth-check-set-and-a-checkless-verb-s-finding-prints-after-its-scope.md)**: the Scope section's open question is decided. `verify`
> records a check set of its own, `deliverable`, when its measurement completes, and `[]` on
> every refusal, so a clean `verify` still prints `0 errors, 0 reviews, 0 notes`.

[#436](https://github.com/MBehtemam/Montagent/issues/436), building MONTAGENT-4 exactly as ruled
in [its design ticket](https://github.com/MBehtemam/Montagent/issues/392#issuecomment-5885476778)
([#392](https://github.com/MBehtemam/Montagent/issues/392)), the fourth of the nine findings from
the first real end-to-end build through this tool
([#383](https://github.com/MBehtemam/Montagent/issues/383)). The design ticket decided what the
verb witnesses, when it stops, what the digest covers, what the stamp cannot answer, what is
measured, and that it is on demand, an MCP tool, and about the declared `output` only. Each of
those rulings is observable behaviour, so ADR-0031 made it a spec gap until an ADR ratified it.
This is that ADR, and it records the three things the design ticket left to the build: the
finding codes, each code's repair form, and whether the two no-evidence cases are two codes.

## Why the verb exists after all

The source doc's motivating incident was a silent MP4 at `0 errors`. That can no longer reach
the output path: ADR-0093 withholds the deliverable on any `error`, and every dropped element is
now one. What is still silent is the class **no `error` can catch because the engine believed
its own output** — a mix the engine built and the encoder cut short, a frame count it pushed and
the container disagrees with. The only witness for that class is one that does not consult the
engine.

## The decision

### 1. An independent witness, and why ADR-0093 ruling 3 does not forbid it

`verify` measures the file with the decoder (`ffprobe`, `ffmpeg`) and compares it to the
document. It never reads what the engine established: not `Established`, not the probe sidecar,
not `render`'s parsed model. It probes each source's audio stream itself, uncached.

> **ADR-0093 ruling 3 is not violated. It is one file and two different questions, not one
> question asked down two data paths.** `render` asks *"did I intend to put X in?"*; `verify`
> asks *"is X in the file?"*.

That sentence is here in those words so that nobody later cites ruling 3 against this verb. What
ruling 3 killed was `validate` and `render` answering *the same* question from two structures
and disagreeing. `verify`'s value is exactly that it does not share a structure with `render`.

### 2. Identity first, measurement second

Measuring a file against a document it was not rendered from produces confident nonsense, so
the stamp is read first. The gate's arms:

| at the declared `output` | finding | class | then |
| --- | --- | --- | --- |
| nothing | `E-VERIFY-NO-OUTPUT` | `error` | stop |
| another project's stamp | `E-OUTPUT-FOREIGN` | `error` | stop |
| this project's stamp, digest differs from now | `E-VERIFY-STALE` | `error` | stop |
| this project's stamp, digest matches | — | — | measure |
| no Montagent stamp | `R-VERIFY-UNATTESTED` | `review` | measure |
| this project's `montagent/1` stamp, a `digest=none` stamp, or an input unreadable now | `R-VERIFY-STALENESS-UNKNOWN` | `review` | measure |

**A stale file is one finding, and measurement stops.** After an edit, every duration,
frame-count and energy mismatch descends from one fact, *the document changed*, and by #384 and
#388's census logic that is one finding. The census cannot carry it instead: it collapses
siblings of one code, and the stale cascade spans several. The flip side is the reason the verb
is worth having: **once the gate passes, every measurement mismatch is attributable to the engine
or the encoder.**

**`Foreign` reuses `E-OUTPUT-FOREIGN`** rather than minting a `verify` code. It is the same fact
that code already states, with the same repair form, and ADR-0107's rule is one fact, one code.
Its template stays true from `verify`: rendering this project would destroy that file.

**The no-evidence cases are `review`, and measured anyway.** This is the argument that won
#391's court: `error` means *guaranteed* wrong, and no evidence is not evidence. The measurements
keep their ordinary classes, because a 720p file where the document says 1080p is wrong as a
deliverable whoever made it.

### 3. The stamp: `montagent/2`, with a digest

```
montagent/2 engine=<version> digest=<sha256 hex | none> project=<canonical project path>
```

`project=` stays last because it is the one field that may contain spaces. `render` writes this
grammar; `media::attest::Stamp::parse` reads it and ADR-0104's `montagent/1 project=<path>`.

**The digest** (`media::digest`) is one SHA-256 over:

- the document in canonical (`fmt`) form, so a whitespace or key-order edit, which renders
  identically, does not make a deliverable stale;
- for every local source, in document order of first appearance, its spelling and its
  `sidecar::fingerprint` (size plus the first and last 64 KiB, ADR-0092);
- for every `fonts`-table file, its key, spelling and SHA-256, which is `FontEntry::sha256`'s
  definition.

The spelling goes into the hash rather than the resolved path, so a project tree that moves
does not go stale by this measure. ADR-0104 still makes it `Foreign`, which is a different
question and is answered first.

**Size and mtime are never an identity here.** That is MONTAGENT-1's defect: a renumbering
shuffle keeps mtime to the nanosecond. Both jurors who voted for content identity had proposed
size+mtime as the cheap option. The test for it
(`a_source_swapped_by_a_renumbering_shuffle_is_stale`) asserts its own fixture: size and mtime
travel with the bytes.

**The engine version is stamped and kept out of the digest.** Otherwise every upgrade makes
every deliverable stale, which is *loud on the safe case*, the failure ADR-0104 §1 rejected.

**A fingerprint that established nothing is never a mismatch.** ADR-0069's rule is that such a
failure is silence. At render time `render` stamps `digest=none` rather than a digest of less
than it rendered; at verify time the input is named in `R-VERIFY-STALENESS-UNKNOWN`'s `reason`.

**Ownership is still the project identity and nothing else.** `render`'s pre-flight reads
`montagent/1` and `montagent/2` alike as `Mine`, so the first render after an upgrade does not
read its own last deliverable as foreign, or even as unattested.

### 4. The measurements

| check | measured on | code | class |
| --- | --- | --- | --- |
| frame size | video stream `width×height` vs the declared frame, padded to even (ADR-0021's disclosed padding) | `E-VERIFY-FRAME-SIZE` | `error` |
| frame timing | **decoded PTS spacing**, one tick of slack for a non-integral `timebase / fps`, never `r_frame_rate`/`avg_frame_rate` (ADR-0011, ADR-0096) | `E-VERIFY-FRAME-TIMING` | `error` |
| frame count + duration | decoded frames vs ADR-0035's frames in `[0, extent)`, and the video **stream**'s first PTS to last PTS plus one frame, never the container's duration. One finding carrying both | `E-VERIFY-EXTENT` | `error` |
| audio stream presence | exists iff something should be heard | `E-VERIFY-NO-AUDIO` / `N-VERIFY-UNEXPECTED-AUDIO` | `error` / `note` |
| audio extent | audio stream vs video stream, within one video frame plus one coded audio frame | `E-VERIFY-AUDIO-EXTENT` | `error` |
| audible energy | §5 | `R-VERIFY-SILENT-SPAN` | `review` |

**Frame count is a real decode** (`ffprobe -show_frames`), not a packet count. That full decode is
what an independent witness costs. The audio extent is read from packets, whose durations are
coded frames. The AAC encoder's priming packet, stamped before `t = 0` and discarded by the edit
list, is not counted as extent.

**The audio-extent check is worth more than the energy check.** A truncated mixed track is at
least as likely a shape for *"silent at 0:30"* as a quiet one, and this check catches it with no
borrowed threshold. Its test cuts a real render's mix to 1 s of 3 s under the render's own stamp.

**The unexpected-audio case happens by design.** `render` mixes an element at `volume: 0`, so a
project whose every audible element is at 0 carries a stream nobody should hear. That is a
`note`: harmless as a deliverable, and worth one line.

### 5. Energy is per span, and the census splits by each source's own energy

**Per-element energy cannot be measured from the file.** The file carries one mixed track, so a
silent voice-over under music is invisible in it. The question `verify` asks instead is where the
mix is silent inside the union of spans in which something **should be heard**. Something should
be heard where:

- an `audio` or `video` element is in `[0, extent)`,
- its source has an audio stream, probed by `verify` itself, and
- its resolved `volume` is above 0, evaluated on every frame of ADR-0035's grid, the same
  instants `render` resolves a keyframed volume at.

Without `overrun: "loop"` an element stops being heard where its source runs out, at
`start + played`. ADR-0020's `hold` freezes a picture and never a sample.

**The measurement** is `ffmpeg`'s `ebur128` momentary loudness, one 400 ms block every 100 ms,
against **EBU R 128's absolute gate of −70 LUFS**. It is not `silencedetect`, which measures
dBFS: citing R128 while measuring dBFS is a citation that does not support its number. The
minimum silent length is **R128's 400 ms block**, not an unsourced one second. The block is the
shortest stretch the measurement can call silent, so a silent stretch is a union of below-gate
blocks, and one intersected with the should-be-heard spans must be at least a block long. Both
numbers are borrowed, so the finding is `review` only (ADR-0061), cites them inline, and reports
the raw quietest block.

Blocks ending before 400 ms are dropped. `ebur128` prints a placeholder there (−120.7 LUFS even
over a loud tone, which was checked against a 440 Hz sine), and a placeholder is not a
measurement.

**Each source is measured too**, once per source over its whole length. Each element's slice of
the silent stretch is mapped back into source time at its `speed` (a looping element that wraps
inside the stretch is judged over its whole source span) and read at the element's peak resolved
`volume` in the stretch. The census on `source_energy` then has up to three groups:

- **`silent in its own source`**: the author's material. A genuinely silent recording would
  otherwise read as an engine bug.
- **`audible in its own source`**: the source has energy there and the mix does not, which
  points at the engine.
- **`source not measurable`**: the source's loudness could not be read.

The census is `Named` (ADR-0111): its values are measurements, and no search of the document
finds them. Where one element alone should be heard in the stretch, `attribution` says the
attribution is exact.

Both halves of the split are tested against real renders: a project whose second line is digital
silence (the author's material), and a real render whose mix is replaced by silence under its
own stamp (the engine's). A census that only ever produced one group would be passed by a check
that never measured a source.

### 6. The build's own three decisions

**Codes.** `E-VERIFY-NO-OUTPUT`, `E-VERIFY-STALE`, `R-VERIFY-UNATTESTED`,
`R-VERIFY-STALENESS-UNKNOWN`, `E-VERIFY-FRAME-SIZE`, `E-VERIFY-FRAME-TIMING`, `E-VERIFY-EXTENT`,
`E-VERIFY-NO-AUDIO`, `N-VERIFY-UNEXPECTED-AUDIO`, `E-VERIFY-AUDIO-EXTENT` and
`R-VERIFY-SILENT-SPAN`, plus the reused `E-OUTPUT-FOREIGN`. There is one code per reason, as #384
and ADR-0093 require, because ADR-0043 fixes repair form per code.

**Repair form (ADR-0043).**

- `E-VERIFY-NO-OUTPUT` and `E-VERIFY-STALE` are **advise-class**, with the repair
  `{"run": "montagent render <project>"}`. `E-FONT-UNATTESTED` is the precedent: the next move is
  the one verb that writes the thing, and it follows from the condition. For the stale case this
  was the question the design ticket put to the build, and the answer rests on what *stale* means.
  `verify`'s question takes the document as it stands to be the one the deliverable must render.
  Which edit made the file stale does not change the move, and the author who wanted the old
  document back has a different problem, which is a revert.
- The measurement codes are **refuse-class, with no repair value.** Once the gate has passed, a
  mismatch is the engine's or the encoder's. Without a stamp it may be anybody's. In neither case
  does the document hold the answer to which, and a re-render of a deterministic engine
  reproduces an engine defect.
  `NotAboutDocument` (ADR-0073) was considered and not taken. Its subject is the invocation, the
  raw bytes or Montagent's process. These findings are about whether a file matches a document,
  and on the unattested path the file may not be Montagent's at all.

**The two no-evidence cases are two codes.** `R-VERIFY-UNATTESTED` means *not known to be
Montagent's*. `R-VERIFY-STALENESS-UNKNOWN` means *Montagent's, staleness unknown*. An agent acts
differently on each: the first may be another tool's file, and the second is at worst last
week's render of this project. A fingerprint unreadable now is the second code's third `reason`,
because it is the same fact: the stamp is ours, and whether the file is stale cannot be said.

### 7. On demand only: `render` names it and does not run it

This is unanimous from the design ticket, and it is implemented as a line in `render`'s boundary.
A `verify` `error` inside `render`'s report would sit beside a file already published, which
breaks ADR-0093's *"a file at the output path is a zero-error render"*. A witness sharing
`render`'s process and parsed model is not independent, and a full decode on every render makes
MONTAGENT-9's timeout worse. So:

- `render`'s report gains `not_checked_also`: *"Whether the file written carries what this
  document says it should … run `montagent verify <project>` … before calling it done."*
- The `render` MCP tool description ends *"Run `verify` on the result before calling a
  deliverable done."*

**`not_checked_also` amends ADR-0006.** The `NOT CHECKED` sentence every report ends with is
not edited. A verb whose answer has limits of its own states them beside it, as a list that is
absent rather than empty on every other verb, and the text form prints each as a bullet under the
block. `verify` states four limits:

- **What each frame shows.** Size, count and timing are measured; pixels are not compared, so a
  frame drawn wrong but on time passes. This is stated plainly because the design ticket named
  MONTAGENT-2's seek and MONTAGENT-6's face as the class verify exists for, and **this build
  catches neither**. Both are wrong pixels at the right instant, and nothing in §4 reads a pixel.
  They stay a stated blind spot, not a claimed catch.
- **An element masked by another audible one.**
- **Audio present but far too quiet**: −45 LUFS clears a −70 gate.
- **A remote source's bytes**: the digest covers the URL's spelling, not its content (ADR-0056).

### 8. An MCP tool whose schema is `project` only

ADR-0011's CLI-only rule is for rarely used verbs, and `verify` is the last step of every agent's
workflow. It also answers MONTAGENT-9's question: a timed-out MCP `render` leaves the caller
asking whether the output exists and is this document's render, which is `verify`'s question asked over
MCP. The schema is `project` and nothing else. No `output` (it would bypass the gate), no range
(a different extent), no `json` (the text form is the answer, as on the other verbs ADR-0011
keeps small). The CLI takes `--json` and `--verbose`, which cost nothing until typed. It runs in
the free dispatch slot and not the encode slot: it writes nothing and contends for no output
path.

### 9. Only the declared `output`

Only the deliverable carries a stamp, so only the deliverable can pass the identity gate. A
project with no `output` is refused at exit 3, naming the rule. Partial renders, previews and a
`verify --output <path>` stay out of scope (the map's *Out of scope* records it). **This ADR
leaves room for a range-aware `verify`** once partial renders carry a range stamp: the gate is
keyed on the stamp's fields, and a `range=` field would be a new `montagent/3` grammar read
beside these two.

## Scope

**`verify` runs none of `validate`'s check sets.** Its findings are about a file, not the
document's internal legality, and a document that does not fit the model is refused at exit 3
with *"run `montagent validate` first"*. ADR-0112's `check_sets` (accepted, not yet wired into the
report) will need a `verify` entry, or will print this verb's summary as the check-free case. The
implementer of ADR-0112 should decide which. Neither is decided here.

## Consequences

- **`render`'s output bytes change again.** The container's `comment` now carries the engine
  version and a 64-hex digest. Deterministic in `CONTEXT.md`'s sense still holds: the same
  project and the same files produce the same bytes. The byte-identity test in `tests/render.rs`
  (ADR-0062's `loop` test) blanks from the version prefix through the project identity, and the
  digest's fixed length keeps both files the same size.
- **`render` now fingerprints every local source and hashes every font before it encodes.** That
  is 128 KiB per source plus each font file, a cost measured in milliseconds against an encode
  measured in seconds.
- **Every deliverable rendered before this ADR verifies as `R-VERIFY-STALENESS-UNKNOWN`**, a
  `review`, measured anyway. The next render replaces the stamp.
- **`verify` costs a full decode of the deliverable and one decode per audible source.** That is
  what an independent witness costs, and why `render` does not pay it.
- **`E-OUTPUT-FOREIGN` now has two emitters**, `render` and `verify`, under one template and one
  repair form.
- **The MCP surface is ten tools** and the CLI has thirteen commands.

## Evidence

- `crates/montagent-core/tests/verify.rs` has ten tests over the verb seam, every one against a
  real render. The four the ticket names:
  - `a_truncated_audio_track_is_an_error`
  - `a_document_edited_after_the_render_is_one_stale_error_and_no_measurement`: the edit halves
    `duration`, which would otherwise raise at least two measurement errors
  - `a_source_swapped_by_a_renumbering_shuffle_is_stale`, which asserts its own fixture
  - `a_silent_source_is_censused_as_the_authors_material`

  The other six pin the gate's other arms, the engine half of the census, and the other
  direction: `a_fresh_render_of_this_document_verifies_clean` and
  `reformatting_the_document_does_not_make_the_deliverable_stale`. A `verify` that called
  everything stale would pass the first four.
  Against the base commit the file does not compile, because there is no `verify` module. That
  failure is real but uninformative, so the base-commit reproduction is the script below.
- `crates/montagent/tests/adapters.rs`: the MCP surface test now names ten tools, `verify` among
  them.
- **`docs/adr/verify_check.sh`** asserts the claims end to end against a built binary, with every
  exit code pinned exactly (ADR-0104's lesson: `clap`'s own argument error satisfies
  *"non-zero"*). It was checked in both directions before `status: accepted` was written:

  | run against | result |
  | --- | --- |
  | this branch | `ADR-0117 holds`: 12 assertions, exit 0 |
  | `bc523042` (the commit before it) | **10 failures**, exit 1 |

  On the base commit, every `verify` call exits 3 with `E-INVOCATION: unrecognized subcommand
  'verify'`, and the stamp reads `montagent/1`. The two assertions that pass there are the two
  that should: a reformat is not staleness (vacuously), and `render` reads a `montagent/1`
  deliverable as its own.
- `docs/adr/output_attestation_check.sh` (ADR-0104's script) now accepts either stamp grammar,
  since the project identity is still the stamp's last field and still the whole of that ADR's
  predicate.
