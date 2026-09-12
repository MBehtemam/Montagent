# Five questions about a video project

`project.json` in this directory is a real published 9:16 YouTube short: 14 tracks,
60 elements, 65216 ms, 1080x1920 @ 25 fps. Elements carry absolute integer-millisecond
`start` and `end`, half-open `[start, end)`. Each track has an integer `layer`
(higher draws in front). Each element may carry a `group` label. Eight elements form
a static `header` overlay present for the whole runtime.

Answer all five questions. Use whatever tools and materials you have. Be exact —
give the numbers asked for, not approximations.

**Q1.** Rank the four segments `item-05`, `item-06`, `item-07`, `item-08` by total
duration, longest first. Give each duration in ms, and the difference in ms between
the longest and the shortest.

**Q2.** Divide the runtime into aligned 5-second buckets (0–5000, 5000–10000, …).
**Excluding the 8 always-on `header` elements**, which single bucket contains the most
element `start` times, and how many does it contain?

**Q3.** This short is designed to loop — the end is meant to run back into the
beginning. Compare the opening of the video with its closing and report every
inconsistency you can find between them, with numbers.

**Q4.** Is every instant of the runtime covered by at least one visual (non-audio)
element? And how many separate spans have no audio element active at all, totalling
how many ms?

**Q5.** Considering only the `narration` track: how many gaps are there between
consecutive elements, and how long is the shortest gap in ms?

## Output format

Answer each question. Then, as the last line, report exactly:

`STEPS: <the number of tool calls you made in total>`

Do not explain your reasoning at length. Do not comment on the materials, the
tooling, or how the questions were chosen. Just answer them.
