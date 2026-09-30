---
name: montagent
description: "Make, edit or check a video with Montagent: the loop from project file to delivered MP4, what Montagent can and can't do yet, and which Montagent skill covers the job. Load first, before any other Montagent skill, whenever a task mentions Montagent or asks for a video with it."
---

# Montagent

A Montagent video is one JSON project file. You write it with your ordinary file tools. Montagent's tools check it, measure it, show it and render it.

## Environment

You need a shell with the `montagent` CLI on `PATH`, Python 3, and ideally the Montagent MCP server. The MCP tools and the CLI share their verbs, so use whichever is to hand. `fmt`, `fonts` and `probe` exist only on the CLI. <!-- workaround: #455 · replaced by: font vendoring, fmt and probe over MCP --> With MCP alone and no shell, a project that has text cannot be rendered: tell the user to install the CLI.

**The binary wins.** These skills describe Montagent's `main` branch. <!-- guard-ok: main workaround --> When a finding, a refusal or `--help` disagrees with a skill, do what the binary says. An unknown-key or unknown-code error on something a skill taught means the installed binary is older than the skill: report `montagent --version`, then upgrade or use the marked workaround.

## Learning the format

1. Read `montagent://format.md` in full: it holds the rules the schema can't express, and it fits in one read.
2. Open [the starter project](references/starter-project.md) and build from it. It shows the keys most pieces need.
3. When you need a key the starter doesn't show, open [the schema note](references/reading-the-schema.md) before you touch `montagent://schema.json`. <!-- workaround: #520 · replaced by: a schema served in pieces that fit a tool result -->
4. Read `montagent <verb> --help` for one verb at the moment you first use it, not up front. MCP tool descriptions say the same.

## The loop

Run every step, in order, every time the project changes.

1. **Scaffold.** `create_project`, or copy the starter into your working directory.
2. **Fonts** (only when there is text). For each face, run `montagent fonts vendor <project> <font file>`. Then write the chain entry it prints into `fonts` yourself.
3. **Build.** Write the project, or generate it with a script when the structure repeats (per-letter typing, a rig baked frame by frame, captions from word timings). Where a Job skill ships a script for the job in its `scripts/` folder, run that script rather than writing your own. Use `measure` to size text boxes. Only drawn frames reach the video. Put every cut, and every keyframe that ends a move, on an instant that `measure --at` confirms is drawn.
4. **Check.** `montagent fmt <project>`, then `validate`. Fix every error. The first time you get findings, open [the findings guide](references/findings.md). Act on every finding it doesn't list as noise.
5. **Look.** Load `montagent-craft` before judging anything. At every beat, call `frame` at that instant. Its `query` block tells you which element drew what. <!-- workaround: #493 · replaced by: frame --from/--to contact sheet --> For motion, `preview` a span of a few seconds around the beat. When a span is refused, the refusal names ranges that fit. Fix what you see, then go back to step 4.
6. **Render**, once, in the foreground, as a call of its own, and wait for it to exit. A render that your shell tool moves to the background is still running: wait on it until it exits. End your turn only after it has. If a render runs far past its expected time, stop it and render the unchanged project on the other surface (MCP ↔ CLI). <!-- workaround: #517 · replaced by: renders that finish on both surfaces -->
7. **Verify.** Run `verify` on the project. It proves the MP4 was rendered from this version of the project and measures what is in it. On an error, render again.

The video is **delivered** when `verify` passes on the last edit.

## What Montagent does, and which skill covers it

Montagent layers images, video, text, rectangles, ellipses and audio on tracks. Every element's position, scale, rotation and opacity can be keyframed, with named easings or a cubic bezier (overshoot included). It has static effects (blur, shadow, mask, tint, saturation, brightness, contrast), a chroma key, timed colour on a word inside text, crossfades, a check that a loop is seamless, and per-clip volume and speed.

| The piece | Load |
|---|---|
| Graphics and text: launch spots, product and social ads, intros and outros, kinetic and typed text, logo reveals and loops, trailer effects | `montagent-motion` |
| Recorded video: talking heads, green screen, captions, lower-thirds, picture-in-picture, music under a voice | `montagent-footage` |
| A character that moves or talks: rig, poses, lip sync, blinks | `montagent-character` |
| Timing, easing, layout, and judging whether it looks right | `montagent-craft`, at every look step |

A piece can mix these: load each skill whose row it touches.

**What it can't do yet**, and what to do instead:

- Animated size, corner radius or colour; paths and morphs; generative or 3D work; motion blur; animated masks. Keep these out of the design, and tell the user if the brief depends on one. <!-- workaround: #463 · replaced by: the missing capabilities -->
- Letter spacing. Set tracked text as one element per letter (`montagent-motion`). <!-- workaround: #510 · replaced by: a letter-spacing key -->
- Parenting, groups and cameras. Bake the hierarchy to per-element keyframes (`montagent-character`). <!-- workaround: #499 · replaced by: parenting or a group transform -->
- Changing an element's image over time. Use one element per image, each shown for its own span (`montagent-character`). <!-- workaround: #518 · replaced by: an image that changes over time -->
- Blend modes. Every element composites with normal alpha. Imitate blends with opacity and tint (`montagent-motion`). <!-- workaround: #521 · replaced by: blend modes -->
