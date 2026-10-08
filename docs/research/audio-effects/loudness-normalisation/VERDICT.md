# Verdict: per-element loudness normalisation (#816)

Owner, 2026-10-08, blind listen to `ab/X.m4a` and `ab/Y.m4a` (AAC 160k, matched at -20 LUFS),
verbatim:

> the Y has falling in the last part of narration , X is more natural to me . that the only
> difference that I got .

`ab/KEY` (sha256 `291fe469d4534d30a53ab1e500dbdf2e896eb32f54ead4cf4a1ee960ffc83fe5`), opened after
the verdict: X = `B_normalised` (member on every element), Y = `A_bypass` (`enabled: false`). The
owner preferred the normalised mix, and the only difference heard was the quieter second voice line
(9 dB down, by construction) falling away in Y. No difference was reported in the bed.

Build: ffmpeg 6.1.1-3ubuntu5 (Linux). Numbers: `measurements.json`.
