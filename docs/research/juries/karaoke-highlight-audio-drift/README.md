# Court: karaoke highlight audio-drift check

Evidence backing [ticket #166](https://github.com/MBehtemam/Montagent/issues/166),
graduated from the map's fog, originally named out of scope in
[ADR-0051](../../adr/0051-word-alignment-is-external-validate-and-compare-catch-drift.md)
(on branch `domain/word-alignment-workflow`) and [#120](https://github.com/MBehtemam/Montagent/issues/120).

Three independent jurors (Opus, Sonnet, Haiku), blind to each other's ballots,
voted on three sub-questions: whether closing the gap needs a new schema-level
link between a `highlight` window and the audio source it was calibrated
against, whether `(path, size, mtime)` is a sufficient staleness signal, and
whether this should be designed now standalone or deferred to the map's
broader "recorded intent" fog entries. Split 2-1 on the mechanism (heuristic
vs. a declared link), 2-1 for a stronger-than-mtime signal, 2-1 for designing
now rather than deferring.

The dissenting juror (Opus) rejected the timeline-overlap heuristic on
correctness grounds, not cost: in a dense multi-track project a highlight-
bearing text element can overlap several audio-bearing elements, only one of
which it was actually calibrated against, so flagging on *any* overlapping
cache-miss would false-positive at a rate that rises with project density —
exactly the kind of noise that teaches an agent to ignore `validate`'s output.

`measure_overlap_ambiguity.py` tested that claim against the real,
currently-committed fixture (`fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json`),
which has zero actual karaoke highlighting (per ADR-0048) — every text
element stands in as a proxy for a would-be highlight-bearing element:

```
$ python3 measure_overlap_ambiguity.py
text elements: 22
exactly 1 overlapping source (heuristic precise): 5
0 overlapping sources: 6
2+ overlapping sources (heuristic ambiguous): 11
```

**11 of 22 text elements (50%) overlap 2+ audio-bearing elements** — the
narration-synced elements most likely to actually carry karaoke highlighting
(`word-05` through `word-08-bridge`) each overlap **4**, and two persistent
UI-chrome text elements spanning the whole project overlap all 20. This
confirms Opus's dissent empirically: the timeline-overlap heuristic is
ambiguous for the *majority* of this project's realistic text elements, not
a rare edge case.

That result flips the disputed premise underlying the Q1/Q3 split: Sonnet
and Haiku's case for "design now, standalone" was conditioned on the
heuristic being adequate. With the heuristic shown ambiguous on this
project's own shape, the correct check needs a declared link between a
`highlight` window and its calibration audio source — which is not worth
minting as a schema field yet (no implementation exists, no evidence of how
often real audio re-recording drift occurs) — which in turn makes this a
named instance of the map's broader "recorded intent" fog entries, not a
standalone fix. See [ticket #166](https://github.com/MBehtemam/Montagent/issues/166)
for the resolution.
