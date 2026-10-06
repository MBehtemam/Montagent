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

1. Read `montagent://format.md` in full, then each page it lists before you write what that page covers. Together they hold the rules the schema can't express, and each one fits in one read.
2. Open [the starter project](references/starter-project.md) and build from it. It shows the keys most pieces need.
3. When you need a key the starter doesn't show, read `montagent://schema/index.json`. It lists every element type and effect with its required keys, and the URI of the piece that holds each one's rules. Read the piece you need, not the whole schema.
4. Read `montagent <verb> --help` for one verb at the moment you first use it, not up front. MCP tool descriptions say the same.

## The loop

Run every step, in order, every time the project changes.

1. **Scaffold.** `create_project`, or copy the starter into your working directory.
2. **Fonts** (only when there is text). For each face, run `montagent fonts vendor <project> <font file>`. Then write the chain entry it prints into `fonts` yourself.
3. **Build.** Write the project, or generate it with a script when the structure repeats (per-letter typing, a rig baked frame by frame, captions from word timings). Where a Job skill ships a script for the job in its `scripts/` folder, run that script rather than writing your own. Use `measure` to size text boxes. Only drawn frames reach the video. Put every cut, and every keyframe that ends a move, on an instant that `measure --at` confirms is drawn.
4. **Check.** `montagent fmt <project>`, then `validate`. Fix every error. The first time you get findings, open [the findings guide](references/findings.md). Act on every finding it doesn't list as noise.
5. **Look.** Load `montagent-craft` before judging anything. Call `montagent frame --from` and `--to` over the whole video, or a stretch of it. You get a contact sheet: one tile for each visual state. Its provenance list tells you which element drew each tile. To look closely at one tile, call `frame` at the instant its line lists. For motion, `preview` a span of a few seconds around the beat. When a span is refused, the refusal names ranges that fit. Fix what you see, then go back to step 4.
6. **Render**, once, in the foreground, as a call of its own, and wait for it to exit. A render that your shell tool moves to the background is still running: wait on it until it exits. End your turn only after it has.
7. **Verify.** Run `verify` on the project. It proves the MP4 was rendered from this version of the project and measures what is in it. On an error, render again.

The video is **delivered** when `verify` passes on the last edit.

## What Montagent does, and which skill covers it

This is the capability map. Read it before you design, so the brief does not lean on something Montagent lacks. Each line names the capability and the keys that spell it; `montagent://format.md` with its pages, and the schema, hold their values and rules.

### What it does

**Elements and placement**

- Images, video, text, rectangles and ellipses, placed and sized on a frame, layered on tracks: `image`, `video`, `text`, `rect`, `ellipse`, `x`, `y`, `width`, `height`, `origin`, `layer`.

**Motion**

- Keyframe position, scale, rotation and opacity, with named easings or a cubic bezier (overshoot included): `scale`, `rotation`, `opacity`, `ease`.
- Keyframe a shape's size, corner radius, fill and stroke, and a text element's colour and stroke: `width`, `height`, `radius`, `fill`, `stroke`, `stroke_width`, `color`.

**Paint**

- Timed colour on a word inside text, by recolouring that word's run: `highlight`.

**Text**

- Text in fonts you vendor, as runs with their own size and colour: `fonts`, `size`, `color`.
- Letter spacing on a whole text element, keyable: `letter_spacing`.

**Compositing**

- Crossfade between clips, and a cut between clips: `transition`, `step`.
- Wipe, slide and push between clips, travelling in a direction, with an ease: `kind`, `direction`, `ease`.
- Chroma key: `chroma`.
- Blend an element into what is beneath it, as screen, add, multiply or overlay: `blend`.
- Keep everything outside a mask shape instead of inside it: `mask`, `invert`.

**Effects**

- Static blur, shadow, mask, tint, saturation, brightness and contrast: `blur`, `shadow`, `mask`, `tint`, `saturation`, `brightness`, `contrast`.

**Time**

- Per-clip speed, and a check on clips that run across a loop's join: `speed`, `loop`.

**Audio**

- Audio on tracks, with per-clip volume: `audio`, `volume`.

### Which skill covers it

| The piece | Load |
|---|---|
| Graphics and text: launch spots, product and social ads, intros and outros, kinetic and typed text, logo reveals and loops, trailer effects | `montagent-motion` |
| Recorded video: talking heads, green screen, captions, lower-thirds, picture-in-picture, music under a voice | `montagent-footage` |
| A character that moves or talks: rig, poses, lip sync, blinks | `montagent-character` |
| Timing, easing, layout, and judging whether it looks right | `montagent-craft`, at every look step |

A piece can mix these: load each skill whose row it touches.

A piece can mix these: load each skill whose row it touches.

### What it can't do yet

Each item names the issue whose close retires it. Tell the user when the brief depends on one.

- Animated effect parameters, masks included. Keep these out of the design. <!-- workaround: #676 · replaced by: animated effect parameters -->
- Gradient paint. Keep it out of the design. <!-- workaround: #686 · replaced by: gradient paint -->
- Per-letter stagger. Write each letter as its own element, generated by a script (`montagent-motion`). <!-- workaround: #693 · replaced by: a per-letter stagger -->
- Mask feather. Keep it out of the design. <!-- workaround: #698 · replaced by: mask feather -->
- Paths, lines and morphs. Keep them out of the design. <!-- workaround: #710 · replaced by: a path element, and morphs -->
- Motion blur. Keep it out of the design. <!-- workaround: #719 · replaced by: motion blur -->
- Generative effects. Keep them out of the design. <!-- workaround: #724 · replaced by: generative effects -->
- 3D tilt and perspective. Keep them out of the design. <!-- workaround: #705 · replaced by: 3D tilt and perspective -->
- Speed ramps. Keep them out of the design. <!-- workaround: #706 · replaced by: speed ramps -->
- Parenting, groups and cameras. Bake the hierarchy to per-element keyframes (`montagent-character`). <!-- workaround: #499 · replaced by: parenting or a group transform -->
- Changing an element's image over time. Use one element per image, each shown for its own span (`montagent-character`). <!-- workaround: #518 · replaced by: an image that changes over time -->
- Reading a track's tempo or beats. Find the beat grid with the script in `montagent-craft`. <!-- workaround: #547 · replaced by: tempo and downbeat from probe -->

### Not by design

Montagent refuses these on purpose, so do not wait for them. Each line gives what to do instead and the decision that refused it.

- Spring easing. Use a cubic bezier with overshoot, or keyframes baked out. ([ADR-0146](https://github.com/MBehtemam/Montagent/blob/main/docs/adr/0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md))
- A repeat or copy construct. Write each copy out as its own element. ([ADR-0148](https://github.com/MBehtemam/Montagent/blob/main/docs/adr/0148-a-repeat-does-not-enter-and-a-stagger-enters-only-across-the-units-of-one-text-element.md))
- A matte taken from another element. Use the element's own `mask`. ([ADR-0150](https://github.com/MBehtemam/Montagent/blob/main/docs/adr/0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md))
- Keyframe expressions, or links between values. Write literal keyframes. ([ADR-0145](https://github.com/MBehtemam/Montagent/blob/main/docs/adr/0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md))
- An open shader or script file. Pre-render it and bring it in as footage. ([ADR-0017](https://github.com/MBehtemam/Montagent/blob/main/docs/adr/0017-closed-schema-no-escape-hatch.md))
