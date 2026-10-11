---
status: accepted
amends: 0011 (`render`'s request gains an optional `profile` and its answer an optional `render.profile` block; no tenth verb)
---

# `render --profile` says where the painters' CPU went, by element, type and effect

[#931](https://github.com/MBehtemam/Montagent/issues/931), from the US-5 verdict in
[#925](https://github.com/MBehtemam/Montagent/issues/925) (map
[#913](https://github.com/MBehtemam/Montagent/issues/913)). A 49-minute production could not
tell which of its 1,000-odd elements made it slow. The only instrument was the stage split
behind the `MONTAGENT_STAGES` environment variable
([ADR-0142](0142-render-has-one-speed-target-the-benchmark-project-in-three-minutes.md),
[ADR-0144](0144-render-paints-on-k-painters-over-chunks-and-the-spy-trailer-renders-in-a-minute.md)
§4, "observed, never enforced"): four numbers for the whole run, no element in them. The
prefix cache and the video-painter lift both need to be measured with one instrument before
they are built, so this goes first.

## Decision

1. **Surface.** `render` takes an optional `profile` (`--profile` on the CLI, `profile` on MCP).
   It adds a `render.profile` block beside `render.painting`. No tenth verb, no progress-file
   field, and `MONTAGENT_STAGES` is not promoted. `preview` has no `profile`.
2. **What it holds.** CPU milliseconds, summed across painters:
   - `elements`: per element id, with its `type`, `cpu_ms` and `frames_painted`, sorted by cost
     and capped at 25, the rest folded into one `elements_remainder` row (`elements`, `cpu_ms`,
     `frames_painted`). Use it on a slice (`--from --to`).
   - `types`: the same per element type.
   - `effects`: per effect name, `cpu_ms` and `applications`. An effect's time is **inside** its
     element's cost, not added to it.
   - `stages`: the existing split (decode, paint, readback, paint wait, encode wait, seal).
3. **What "CPU" means.** The painter thread's own time: the wall clock of an element's paint call
   less the time that call spent waiting on decoded frames. The wait is the `decode` stage,
   reported beside the elements, so a video element's cost is its painting and not its feed's
   latency. An effect's cost is the time its layer takes to unwind (Skia filters a layer when it
   is restored into its parent), plus a `grain`'s pass or a `mask`'s eraser. Work done
   before the layers unwind (the bounds hint's recording of the element, the directional-blur
   crop) stays in the element's cost.
4. **The 90% criterion is against the paint stage, not wall time.** `attributed_pct` is the
   elements' share of the paint stage. The remainder is `unattributed`, labelled *unattributed /
   compositor*: the background fill, the transitions' resolution and per-frame bookkeeping. It
   is a named bucket so the figure cannot be met by leaving cost out of sight. The element
   figure counts every element, not only the 25 listed.
5. **Never speed-gated, never a byte.** Timing costs a few clock reads per element, so a profiled
   run is not a speed measurement and no gate reads one. The frames at the encoder's input hash
   equal with and without the flag, on one painter and on several
   (`tests/render_profile.rs`). The block is not part of what a test compares across runs.
6. **`elements_drawn`** (the US-7 ask in [#921](https://github.com/MBehtemam/Montagent/issues/921))
   becomes `frames_painted` per element inside this block, not a progress-file field; the
   progress file keeps its fields. `frames_reused` stays with the static-prefix cache's own ADR
   and joins the profile when the cache lands. "Frames drawn from scratch / frames reused" in
   US-5's acceptance is therefore split across two build items.

## What it is for

[ADR-0144](0144-render-paints-on-k-painters-over-chunks-and-the-spy-trailer-renders-in-a-minute.md)
§2 admits a new bottleneck class only behind *"a profile in which one distinct stage dominates
the render"*. This block is that profile. Whether an accepted story (the prefix cache, the
video-painter lift) needs a class of its own is read from it on the reporter's project, by
whoever builds that story, and is not decided here.

## Not chosen

- **A `--profile` verb or a progress-file section.** Rejected in #925: a ninth-plus verb costs a
  schema slot on every turn, and the progress file is for a caller that polls, not one that
  reads a result.
- **Thread CPU time from the OS.** It would need a platform call per element; the painter
  thread is CPU-bound apart from the feed wait, which is already measured and subtracted.
- **Per-frame rows.** Unbounded, and not what a slice asks.

## Consequences

- A painter that fails or is cancelled takes its partial profile with it, but then the render
  answers with no video and no `render` block, so a partial profile is never shown.
- Skia's lazy filters mean an effect is charged where its layer unwinds. The sum of effect rows
  can differ from "the effect's true cost" by the compositing of its parent layer; the element
  row stays exact.
- An element's `id` is required and unique, so a row is one element.
- `--profile` is read with `--json`; the text form does not print it.
