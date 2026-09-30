---
status: accepted
amends: 0093 (ruling 3's containment loses its one exception: a remote source `render` declines is an `error` to `validate`, under `render`'s own codes), 0053 (relocating a project to a remote store is a rewrite for reference, not one that renders)
---

# `render` and `frame` use local sources only, and `validate` says so

[#456](https://github.com/MBehtemam/Montagent/issues/456). A project whose one `video` element
reads `"source": "http://127.0.0.1:18456/clip.mp4"`, served by `python3 -m http.server`,
reproduced on `8707a7d2`:

```
$ montagent validate p.montagent.json
0 errors, 0 reviews, 0 notes, 0 unchecked, 1 layout — p.montagent.json
exit 0
$ montagent render p.montagent.json
error  E-NOT-MIXED-REMOTE  remote-clip
  `remote-clip` is not in the mix: `http://127.0.0.1:18456/clip.mp4` is remote, and `render`
  mixes local sources.
  … Do not guess one.
exit 1
```

`validate` gave a clean pass to a document `render` is guaranteed to refuse. That is the shape
ADR-0093 ruling 3 exists to prevent, and ADR-0093 allowed it in the next sentence:

> A containment and not an equality, on purpose: `render` mixes local sources only, so it
> declines a remote one `validate` probed perfectly well — a fact about the verb, not the file.

It is a fact about the verb. That does not make it a reason for `validate` to say nothing.
An agent reads `validate` in order to find out whether `render` will work, and a verb-level
refusal is exactly what it is asking about.

## The decision

### 1. `render` and `frame` draw and mix local sources only

This is a **decision**, recorded here because nothing recorded it before. ADR-0002 permits a
URL `source` and ADR-0056 says how one is probed. Neither says a URL renders, and the code
has always refused one: `render`'s mix at `E-NOT-MIXED-REMOTE`, and the painter at
`E-NOT-PAINTED-REMOTE`, for `render` and `frame` alike.

- **A URL `source` stays legal** (ADR-0002 stands), and `validate` still probes it exactly as
  ADR-0056 says: once per session, deduplicated by URL, with no persistent cache and a
  network failure `UNCHECKED`. Nothing about the probe changes.
- **The author's remedy is a local copy**, at a path of their choosing, named by `source`.
- **Fetching a remote source inside `render` is deferred, not rejected.** It would bring its
  own questions: where the copy lives, whether it outlives the run, what *"the file
  `validate` probed"* means once the bytes are fetched twice, and ADR-0056's promise of no
  unsolicited network call beyond the probe's own range reads. None of them is answered
  here. A later ADR that takes this up amends this one.

`file:` URLs are local ([`Source::resolve`](../../crates/montagent-core/src/media/mod.rs)),
and this ruling does not touch them.

### 2. `validate` pre-flights it: an `error`, under `render`'s own codes

For every element whose `source` resolves to `Source::Remote`, `validate` states the refusal
in its disk set. An `audio` element gets `E-NOT-MIXED-REMOTE`, an `image` gets
`E-NOT-PAINTED-REMOTE`, and a `video`, which `render` both mixes and draws, gets both.

**Class: `error`.** ADR-0006 computes class from the consequence at an instant. From
`validate` the consequence is ADR-0093's *"the render is refused or is guaranteed wrong"*: the
render refuses, at every instant. It is not `UNCHECKED`, because the question was answered. The
probe may have succeeded, and the answer is still *"`render` will not use this"*. ADR-0107 set
the precedent for a `render` refusal that the check engine can state: `validate` states it
as an `error` under `render`'s code, and then `render` refuses on the check engine's report
before it reaches the mix. `E-SOURCE-OVERRUN` works the same way (ADR-0093's amendment by
ADR-0096).

**The codes are reused and no new one is minted.** The registry allows it. The two codes'
repair form is already refuse, and ADR-0043 fixes that per code, so it is the same from either
verb. `E-NOT-MIXED-REMOTE`'s one class is `error`. `E-NOT-PAINTED-REMOTE` is `error` from
`render` and now from `validate`, and still `review` from `frame` (ADR-0093 ruling 1). Both
codes now name the `disk` set (ADR-0112). That set could already raise `error` and `review`, so
its summary line is unchanged. A new code would have made one fact carry two codes. Then
`compare` would diff the same refusal as two different findings depending on which verb saw it
first.

The finding goes through one function. `media::established::local` is the only place that
turns a `Source` into a file a verb may mix or draw from. `render`'s mix, `frame`'s painter
and `validate`'s disk check all call it, and it classifies with `Source::is_remote`'s own
match. There is no parallel check against `Report::media`. ADR-0093 ruling 3 is about exactly
that fork.

`render`'s mix still asks the same function, but its remote arm is now `E-INTERNAL`, as
ADR-0107 made the arms it closed. `render` and `preview` both refuse on the check engine's
report before the mix is built, so reaching that arm means the two halves disagree about one
document. `E-NOT-MIXED-REMOTE` is therefore raised only by `validate`'s check engine.
`E-NOT-PAINTED-REMOTE` keeps its painter arm, because `frame` paints a permissively read
document and runs no checks (ADR-0093 ruling 2's asymmetry).

### 3. Ruling 3's containment has no exception

ADR-0093 ruling 3 now reads:

> **`validate` is never a clean pass on a source `render` declines to use.**

*Unchecked or `error`*, as `tests/cross_verb.rs` has always read the ruling's *"unchecked
set"*. An `error` about the same source is a louder statement of the same thing. It is still a
containment, because `validate` may say more than `render` declines. It is no longer
containment *with a carve-out*.

### 4. The prose says what to do

Both codes are refuse-class, and they stay so: the path of the local copy is the author's to
choose, and ADR-0043's advise arm needs a fix *fully determined by the document*. But the
refuse boilerplate ends *"Do not guess one"*, and without anything beside it that reads as
*"there is nothing you can do"*. That points an agent away from the obvious fix. So each
template now names the remedy and says which part of it is the author's to choose:

> `vo` is not in the mix: `http://…/vo.mp3` is remote, and `render` mixes local sources only.
> The remedy is a local copy: fetch the file and point `source` at it by path — which path is
> yours to choose, and is why no repair is stated.

The boilerplate is shared by every refuse-class code and is not touched.
`E-NOT-PAINTED-REMOTE` is now worded *"is not painted"* rather than *"was not painted"*,
because it is now stated before any frame is.

## Scope

- **Not in scope:** fetching, downloading or caching a remote source for `render` or `frame`
  (deferred above), any change to how remote sources are probed (ADR-0056), removing URL
  support from `source` (ADR-0002), and any write tool that downloads the file for the author.
- **The format rules** (`montagent://format.md`, *Sources*) and `CONTEXT.md`'s **Source** say
  that a URL `source` is probed by `validate` and refused by `render` and `frame`, and that a
  remote store is for reference, not rendering. ADR-0053's *"relocating to a remote store
  means rewriting `source` to URLs"* is still how a project **references** a remote store. It
  no longer implies that the project renders from one. The published schema's `source`
  descriptions do not mention URLs and are unchanged.

## Consequences

- **`validate` exits 1 on a project with a remote source**, where it exited 0. This is a
  behaviour change, and it is the point: exit 0 said *"proceed"* to a `render` that would
  refuse.
- **`render`'s answer on such a project comes from the check engine**, before any mix is built.
  It carries the same codes as before, plus `E-NOT-PAINTED-REMOTE` for a `video`, which the
  mix's refusal used to hide.
- **A partial render (`render --from/--to`) refuses a remote element outside its range.** The
  check engine reads the whole document, as it already does for a missing source anywhere in
  it (ADR-0121 bounds a partial render's world-effects, not its checks). Before this, the mix
  looked only inside the range.
- **`frame`'s exit code is unchanged.** It runs no checks, and its remote finding is still
  `review`.
- **A project with only local sources or `file:` URLs gets byte-identical `validate` output**,
  because neither ever reaches the new arm and the `disk` set's classes did not move.

## Evidence

- `crates/montagent-core/tests/cross_verb.rs`,
  `a_remote_source_that_answers_is_never_a_clean_pass_to_validate`. A loopback HTTP server
  serves an `mp3` and a `png`, so `ffprobe` answers and no network finding can stand in for the
  answer. The test asserts that `render` declines each one (the mix's arm for the `audio`
  element, the painter's for the `image`) and that `validate` speaks about each. It failed on
  `8707a7d2` with *"`render` declined `http://127.0.0.1:…/vo.mp3` and `validate` passed it
  clean"*.
- `crates/montagent-core/tests/validate.rs`: the codes, classes, element and URL for each type,
  exit 1, the remedy in the text, `0 errors` absent, and a `file:` URL drawing no remote
  finding.
