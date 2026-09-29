# How the pack's footage was made

## 1. Presenter: three green-screen takes (done)

Generated with Azure AI Speech's text-to-speech avatar: the prebuilt avatar "Harry",
casual style, background `#00FF00`, 1920×1080 at 25 fps. Each script was pasted in as
plain text and the MP4 committed as delivered. Word timings come from
`pack-src/align_takes.py`, which refuses a take whose speech doesn't match its script
word for word.

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
