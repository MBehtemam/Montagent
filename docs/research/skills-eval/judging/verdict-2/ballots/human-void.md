# Void: the first human sittings were too fast to be judgements

`human-void.json` holds the human's first ballots: all 45 pairs, cast between 16:55:50 and
16:59:49 UTC on 2026-10-02, with a median of 3 s between ballots and a longest gap of 25 s
(19 equal, 15 left, 11 right). `sittings-void.json` is the sittings record from that pass.
Each pair is two clips of roughly 15–60 s, and `RUBRIC-v2.md` asks for the whole video to be
judged, so these ballots cannot be judgements of the videos.

The blind was intact when they were voided: `key.json` and `sample.json` had not been
opened, and neither the human's nor the court's ballots had been tallied. The human chose to
judge again. The sittings were reset (same order, start times cleared), and the second pass is
`human.json`, the only ballots `verdict.py` reads. These are kept so the void can be checked.
