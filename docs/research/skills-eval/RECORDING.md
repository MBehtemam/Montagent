# Recording checklist for the eval asset pack

Two things only the maintainer can record, done in one sitting: three green-screen
presenter takes, and one screen recording. Drop the raw files in `assets/incoming/`
(any names). The agent then trims and encodes them, derives word timings, and pulls
the stills.

## 1. Presenter: three green-screen takes

**Setup**

- [ ] A green backdrop filling the frame behind you: cloth, paper or a painted wall. Pull it taut; creases become shadows.
- [ ] Stand about 1 m in front of it, so you don't cast a shadow on it and less green bounces onto you.
- [ ] Light the backdrop evenly and separately from yourself if you can. A key that has to cope with a gradient across the backdrop is a harder test than intended.
- [ ] Wear nothing green, and nothing shiny.
- [ ] Phone **vertical**, 1080×1920 or larger, 30 fps, locked exposure and focus (long-press on iPhone).
- [ ] Frame from mid-chest up, with some headroom. Leave space on your right for a picture-in-picture and at the bottom for captions.
- [ ] Record the sound on the phone, or a lav into it. A quiet room matters more than the mic.

**Each take**

- [ ] One second of silence, still, looking at the lens, before you speak and after you finish.
- [ ] Say the script as written. The captions are checked against these exact words. If you change a word, note it.
- [ ] 8–10 s of speech. Do it again if it runs long; don't rush.

**Take 1** (development brief B)

> I stopped dragging clips around a timeline. Now my agent writes the video as a file, checks every frame, and renders it. This is Montagent.

**Take 2** (spare, for the held-out briefs)

> Here's the trick. Every element in the video is one line of text, so my agent can change exactly one thing and prove nothing else moved.

**Take 3** (spare, for the held-out briefs)

> Most editors hide the video behind a mouse. Montagent hands it to your agent, and tells it the truth about every frame before it renders.

## 2. Product: one screen recording

- [ ] Terminal with a dark theme and a **large** font (18 pt or more), so it reads when scaled down in a picture-in-picture.
- [ ] Hide anything personal: other windows, notifications (Focus mode on), your home path in the prompt if you mind it showing.
- [ ] Record a region of about 1920×1080 with ⌘⇧5 → *Record Selected Portion*.
- [ ] A real session, 30–60 s: Claude Code with the Montagent MCP server, asked to make a small change to `examples/hello-text` (for example, "add a subtitle line under the greeting that fades in"). Let it edit the file, run `validate`, look at a `frame`, and `render`, then open the rendered MP4 in QuickTime and let it play for a few seconds.
- [ ] No need to narrate. The sound from this recording is not used.

From the screen recording the agent takes two stills for the launch spot: one of the agent at work in the terminal, and one of the rendered frame.
