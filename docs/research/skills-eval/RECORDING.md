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

## 2. Product: one screen recording (done)

Recorded headlessly with [VHS](https://github.com/charmbracelet/vhs) 0.12.1 at 1920×1080
and 30 fps, 24 pt, Catppuccin Mocha. The tape is `pack-src/session.tape`. It is a real,
unscripted session: Claude Code 2.1.284 on Opus 5.5, in a copy of `examples/hello-text`
with its font vendored and the Montagent MCP server connected. It gets one prompt:

> Add a subtitle under the greeting that fades in over the first second. Check a frame, then render it.

The tool permissions were pre-approved in the copy's `.claude/settings.json`, so the
session runs without prompts. Claude read `format.md` and the schema, added the subtitle
on its own track, checked frames at 500 and 1500 ms, and rendered in about 40 s.
`screen/session.mp4` is the first 55 s. `stills/session-01.png` is the frame at 36 s,
and `stills/session-02.png` is the rendered video at 1.5 s.
