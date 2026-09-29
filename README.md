# Montagent

Montagent is a video editor whose project format is designed to be authored and edited
by an AI agent rather than dragged around in a GUI. You describe what should be on
screen to an agent — Claude Code, or any other client speaking [MCP](https://modelcontextprotocol.io),
the protocol that lets an AI assistant call a tool — and Montagent renders it.

## Why not a timeline GUI?

A GUI timeline is built for a hand on a mouse: drag a clip, nudge a keyframe, look at
the result. An agent doesn't have a hand or an eye — it has a document. A Montagent
project is a single JSON file that states, by being read, what is on screen at any given
moment: which track, which element, what time range, what it looks like. An agent edits
that file the way it edits code — read, change a value, re-render, look at the frame —
and the file is the whole authority: check it into git, diff it, review it in a pull
request. There's no separate project database, and nothing "in the app" that a diff
can't show.

Montagent itself is deterministic and contains no model: it doesn't write your video
for you, it renders the one you (or your agent) described, the same way every time.
It's also fast — a 2-second, 1920×1080 clip typically renders well under half a second.

## Install

```
cargo install montagent
```

or download a prebuilt binary for your platform from the
[Releases page](https://github.com/MBehtemam/Montagent/releases) and put it on your
`PATH`. Six desktop targets are built: macOS, Linux and Windows, each for `x86_64` and
`aarch64`.

You'll also need `ffmpeg` on `PATH`: **7.1 or newer, built with `libx264`** (ADR-0115).
Ubuntu 24.04's apt ships 6.1, which is below that; use a static build or a newer
distribution. Montagent spawns it for encoding and decoding
rather than linking it in (its codecs are GPL; Montagent's binary isn't), and it's only
asked for when you actually render — validating or scaffolding a project needs nothing
but the `montagent` binary itself.

### macOS: the binary is unsigned

If you downloaded the tarball instead of building with `cargo`, macOS quarantines the
extracted binary and refuses to run it — not with a dialog you can click through, but a
silent kill on the first attempt and, after that, a silent kill with no dialog at all.
Before running it the first time:

```
xattr -d com.apple.quarantine ./montagent
```

No sudo, no GUI, and nothing to click. `cargo install` never hits this — `cargo` builds
the binary on your machine rather than downloading a quarantined one.

## Wire it up to an agent

Montagent is a single binary that is both this CLI and an MCP server. Point your MCP
client at it with a config block like this one (Claude Code / Claude Desktop's shape —
adjust the wrapper for whatever client you use):

```json
{
  "mcpServers": {
    "montagent": {
      "command": "montagent",
      "args": ["mcp"]
    }
  }
}
```

Once it's connected, your agent has nine tools — `compare`, `create_project`, `frame`,
`measure`, `preview`, `query`, `render`, `shift`, `validate` — and two resources that
teach it the project format directly: `montagent://schema.json` and
`montagent://format.md`. That's where the agent learns the verbs; this page isn't
trying to repeat them.

## See it work, without wiring up an agent

Unpack the release archive (or clone the repo) and run, from its root:

```
montagent fonts vendor examples/hello-text/hello-text.montagent.json examples/hello-text/OpenRunde-Bold.otf --licence OFL-1.1
montagent render examples/hello-text/hello-text.montagent.json
```

That writes `out/hello-text.mp4` — 90 frames of white text on a dark background — and
only the second command needs `ffmpeg`.

The first command isn't a formality: Montagent refuses to render a font it hasn't
verified the licence of, even the Open Runde font shipped alongside this example, so
`--licence OFL-1.1` is you confirming what `OpenRunde-LICENSE.txt` (also in that
directory) already says. See `examples/hello-text/README.md` for why.

## If a source looks stale

Montagent remembers what it probed about each media file in a per-user cache, so an
unchanged file costs no `ffprobe`. It notices a file that changed on disk, including one
rewritten in place under an unchanged size and timestamp. If you ever need to start from
the disk again:

```
montagent cache clear
```

It prints the path it removed, which is platform-specific and not worth memorising.

## Learn more

`CONTEXT.md` and `docs/adr/` hold the reasoning behind the format and every non-obvious
decision in it, for anyone who wants more than this page says.

## Licence

MIT — see [`LICENSE`](LICENSE). The test fixtures under `fixtures/` and some committed
research media are not covered by it; see [`LICENSE-MEDIA.md`](LICENSE-MEDIA.md) for the
carve-out. The binary itself statically links Skia and around 128 other Rust crates;
[`THIRD-PARTY.md`](THIRD-PARTY.md) — shipped in this repo and in every release archive —
lists what's owed for them.
