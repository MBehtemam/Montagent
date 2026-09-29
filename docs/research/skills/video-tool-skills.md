# How other video and motion-graphics tools teach agents

Gathered for [#443](https://github.com/MBehtemam/Montagent/issues/443), part of the
map [#441](https://github.com/MBehtemam/Montagent/issues/441) (end-user agent skills
for Montagent). Remotion is covered by its own ticket,
[#442](https://github.com/MBehtemam/Montagent/issues/442), and is mentioned here only
for comparison. Distribution mechanics ([#444](https://github.com/MBehtemam/Montagent/issues/444))
and drift ([#450](https://github.com/MBehtemam/Montagent/issues/450)) have their own
tickets too, so they get only brief notes here.

**Question.** How do other video, animation and motion-graphics tools teach agents
through skills, or through things that do the same job (MCP server instructions, MCP
prompts, `llms.txt`, agent guides)? For each one: what knowledge it encodes, how it is
structured and how finely it is split, how it stays current, and how it is
distributed. Then: which patterns recur, and which look like mistakes.

**Method.** I read the SKILL.md files, reference files, server source and CI
workflows in each repo directly, through the GitHub API and
`raw.githubusercontent.com`, on 2026-09-29. Sizes are byte counts from the git tree
API. Star counts are a rough signal of reach, not of quality. Two of the most
carefully built examples (ffmpeg-skill and the After Effects bridge) have almost no
stars.

## Summary table

| Example | Artifact | Knowledge it encodes | Granularity | How it stays current | Distribution |
|---|---|---|---|---|---|
| HyperFrames (HeyGen) | 21 SKILL.md files, ~900 files in `skills/` | routing, workflow loop, format contract, craft, recipes, CLI | an entry router, domain skills, workflow skills | CI tests pin skill content to lint codes; the CLI warns when skills are stale | `npx skills add`, the CLI's `skills update`, a Claude plugin, ClawHub sync on every push |
| ffmpeg-skill | 1 SKILL.md (30 KB), 42 scripts, 1 MCP server | workflow loop, request-to-script table, the line between what it decides and what it doesn't | one skill with a lot of scripts behind it | the `contract --json` tool manifest is the source of truth | a plugin, `npx`, or copying the folder |
| After Effects MCP bridge | 15 SKILL.md files beside the MCP server | the tool bridge (read, edit, look), craft as tool calls | one bridge skill plus craft and domain skills | the skills live in the server's repo | the same repo as the MCP server |
| Manim: adithya-s-k/manim_skill | 3 skills, `rules/*.md`, tested examples | API rules, a planning workflow | a planner skill plus a best-practices skill per fork | by hand | copying the folder |
| Manim: DominikPeters/manim-skill | 1 short SKILL.md, 2 scripts, a bundled copy of the docs | a render, contact sheet, iterate loop | one skill | CI re-syncs the docs and cuts a release | a release zip or `.skill` file |
| Anthropic `skills` (slack-gif-creator, algorithmic-art) | SKILL.md plus a helper library | platform limits, a validator, animation concepts, taste | one skill per task | by hand | the plugin marketplace |
| Blender MCP | MCP server `instructions` plus an MCP prompt | correctness rules and a verify step | 1.5 KB of server instructions | tests on the instructions text | the MCP handshake itself |
| Higgsfield MCP | server instructions that route to `get_*_instructions` tools | a routing table, workflows fetched on demand | tiny instructions, workflows served from the server | server-side, always current | the MCP handshake itself |
| Motion Canvas, Revideo | nothing | — | — | — | — |

---

## 1. HyperFrames (HeyGen): the most complete example

Repo: <https://github.com/heygen-com/hyperframes> (`skills/`). HyperFrames renders
video from HTML, and it bills itself as "Built for agents". Of everything surveyed,
it is the closest analogue to Montagent: a declarative document format, a CLI that
lints and checks it, a renderer, and skills that target strangers' agents.

**Structure: three tiers, with a router at the top.**

- **The entry skill** is
  [`skills/hyperframes/SKILL.md`](https://github.com/heygen-com/hyperframes/blob/main/skills/hyperframes/SKILL.md)
  (18 KB). Its description reads: *"Mandatory entry point: read this first for any
  request to make, create, edit, animate, or render a video…"*. The body is mostly
  first-match decision tables:
  - §1 is **project state → action**. A specific edit gets made directly. `BRIEF.md`
    exists means resume. A fresh creation runs an intent interview.
  - §2 is **deliverable → workflow**: ten routes, in priority order.
  - §5 is **need → domain skill**, together with a table of cross-domain "creator
    phrases" such as "punch-in", "match cut" and "duck the music", each mapped to the
    set of skills it needs.
- **Domain skills** own the technical knowledge: `hyperframes-core` (the composition
  contract), `-animation`, `-keyframes`, `-creative` (design, typography, story),
  `-audio`, `-cli`, `-registry`, `media-use`.
- **Workflow skills** are end-to-end recipes for one kind of deliverable:
  `product-launch-video`, `faceless-explainer`, `pr-to-video`, `music-to-video`,
  `talking-head-recut`, `slideshow` and others. They are installed lazily, only
  when routing picks them:
  [`references/skill-lifecycle.md`](https://github.com/heygen-com/hyperframes/blob/main/skills/hyperframes/references/skill-lifecycle.md).

**What it encodes.** It encodes all four kinds of knowledge the ticket asks about:

- **Capability map.** The §5 table routes each need to the skill that owns it.
- **Workflow loop.** In
  [`references/review-loop.md`](https://github.com/heygen-com/hyperframes/blob/main/skills/hyperframes/references/review-loop.md),
  work moves plan → sketch → build → final look, with explicit "checkpoint gates".
  There is a *collaborative* mode and an *autonomous* mode that "keeps exactly one
  question before render". After approval, the skill offers to "freeze" the run
  into a reusable recipe.
- **Recipes.** One per workflow skill, plus `hyperframes-core/references/creator-editing-recipes.md`.
- **Craft.** `hyperframes-creative` includes `motion-principles.md`, `typography.md`,
  `house-style.md` and `story-spine.md`.

**The format contract lives in the skill, pinned by tests.** Near the top,
[`hyperframes-core/SKILL.md`](https://github.com/heygen-com/hyperframes/blob/main/skills/hyperframes-core/SKILL.md)
has "Agent pitfalls (read first)" and "First-pass lint gotchas (a guaranteed first
build failure)". Each rule names the lint code that catches it, for example
`gsap_css_transform_conflict` and `media_missing_id`, with the framing "Rules that
`lint` **does** catch, but only after the fact. Write them right the first time."
It also warns about a quiet lie in the tool's own output: a lint error "switches
off the layout and contrast audits: `check` then reports `0 sample(s)`… which reads
like a clean file but means nothing ran."

**How it stays current.** It uses three mechanisms:

1. **Content tests in the product's CI.**
   [`packages/cli/src/commands/coreSkillContent.test.ts`](https://github.com/heygen-com/hyperframes/blob/main/packages/cli/src/commands/coreSkillContent.test.ts)
   asserts that skills name the canonical gate (`npx hyperframes check`), that the
   minimal skeleton still has a runnable root, and so on. A comment in that file
   records a lesson they learned: *"asserting exact sentences here made every docs
   correction a CI failure"*. So they now pin structure and identifiers, not prose.
2. **A freshness check in the product.** `npx hyperframes skills check` exits
   non-zero when installed skills are stale. The CLI "may print a one-line
   stale-skill reminder during `render`, `lint`, or `check`". The skill also tells
   the agent: *"Treat a failed update as a visible tool failure; do not continue from
   a remembered workflow contract."*
3. **Pin probing.** The entry skill has the agent run `npx hyperframes@latest
   upgrade --project . --check` before the first render-affecting command. If an
   upgrade happens, the skill requires naming the old and new versions in the
   summary, so a version bump is never silent.

**Distribution.** It ships through several channels:

- `npx skills add heygen-com/hyperframes`
- the CLI's own `skills update <name>`
- a Claude plugin (`.claude-plugin/`)
- a GitHub Action,
  [`sync-skills-to-clawhub.yml`](https://github.com/heygen-com/hyperframes/blob/main/.github/workflows/sync-skills-to-clawhub.yml),
  that republishes changed skills to the ClawHub registry on every push to `main`

Docs are also published as `llms.txt` (<https://hyperframes.heygen.com/llms.txt>).

**Size.** The skills are large. SKILL.md bodies range from 6.6 KB up to **66 KB**
(`talking-head-recut`), and seven of them are over 30 KB. References chain from
the entry skill to its `references/`, then to other skills' `references/`.

## 2. ffmpeg-skill (zhaikong): an execution skill with a hard edge

Repo: <https://github.com/zhaikong/ffmpeg-skill>
([`SKILL.md`](https://github.com/zhaikong/ffmpeg-skill/blob/main/SKILL.md), 30 KB).
It has 0 stars but is actively maintained, and its design is the closest in spirit
to Montagent's verbs.

- **A numbered workflow loop.** The steps are:
  - Probe before planning: "Plan from real numbers, never assumptions."
  - Prefer lossless.
  - Run `--dry-run --json`, then execute.
  - Chain steps in a fixed order.
  - `check.py` the deliverable against a platform.
  - Verify the output, reporting the probed numbers.
  - Never overwrite originals.
  - **Look at the picture.**

  The last step splits what it checks into **mechanical** checks ("the skill's own
  job to verify") and **judgement** calls ("report it, don't silently pass or fail").
  It also has an honesty rule for agents that cannot see images: *"write `Look: PATH
  (pixels not inspected; agent has no image view)` — never claim a picture was
  inspected when it wasn't."*
- **"What this skill does and does not decide."** This section is an explicit
  boundary: *"same input + same explicit parameters always producing the same
  verifiable output belongs here; anything depending on taste… belongs to whoever
  makes that judgement."* It points taste work at a separate colour-grading skill.
- **Grounded capability discovery.** A "User says → Do" table maps phrases to
  scripts, with the rule *"name only a script you have seen in one of them (there
  is no `doctor.py`, no `trim.py`…)"*. The machine-readable source of truth is
  `contract --json`.
- **Asking the user.** Ask only when the answer changes the output materially, and
  then *"propose one bundle with your defaults"* rather than asking one question per
  turn.
- **Cost-aware progressive disclosure.** *"A reference file costs as much as this
  one; open one only for a question you have"*, and `--help` is "the cheapest full
  flag list".
- It ships an MCP server alongside the skill. `tools/list` shows only 12 core tools,
  and the other 30 stay "callable by name", so the tool list costs less context.

## 3. After Effects MCP bridge (solomondivyananth): skills as the manual for the server's tools

Repo: <https://github.com/solomondivyananth/aftereffects-mcp> (`skills/`, 15 skills,
each 4–8 KB). There are many After Effects MCP servers. Most teach only through tool
descriptions, for example
[TheLlamainator/after-effects-mcp](https://github.com/TheLlamainator/after-effects-mcp).
This one ships a skill set in the same repo as its server.

- **A bridge skill that owns the loop.**
  [`ae-mcp-bridge/SKILL.md`](https://github.com/solomondivyananth/aftereffects-mcp/blob/main/skills/ae-mcp-bridge/SKILL.md)
  says: *"Never edit blind. Every job is **read → edit → look**"* (orient, read,
  edit, look, show), and adds *"Report what you *saw*, not what you intended."* Its
  "Reading traps" section includes *"A still can't show motion. A frame mid-reveal
  looks like a hard cut. Judge timing with `ae_review_motion`, never one frame."*
- **Craft written as tool calls.**
  [`ae-motion-principles/SKILL.md`](https://github.com/solomondivyananth/aftereffects-mcp/blob/main/skills/ae-motion-principles/SKILL.md)
  opens with *"Keyframes that hit the right values can still look wrong"*. It then
  gives frame-count tables for durations (a hit is 2–4 frames, a standard move 12–18,
  and a reading hold is about words ÷ 3 seconds) and an ease table. Every entry is
  written as concrete `ae_*` arguments, not prose advice.
- **Domain skills for each concern**: camera-3d, colour, expressions, transitions,
  templates, deliver, audio-sync (with a `beats.js` script), and
  reference-breakdown.

## 4. Manim: two opposite designs

**adithya-s-k/manim_skill** (<https://github.com/adithya-s-k/manim_skill>, about
1.1k stars):

- `manimce-best-practices` and `manimgl-best-practices` are API rule books. Each
  SKILL.md is an index of ~22 `rules/*.md` topic files, plus *tested* example scenes
  and templates.
  [SKILL.md](https://github.com/adithya-s-k/manim_skill/blob/main/skills/manimce-best-practices/SKILL.md)
- The descriptions use explicit trigger lists: *"Trigger when: (1)… (2) Code
  contains `from manim import *`"*. They also carry a negative trigger, *"NOT for
  ManimGL"*, to separate the two forks.
- `manim-composer` is a planning skill ("Use this BEFORE writing any Manim code").
  It researches the topic, asks about audience and scope, and writes a `scenes.md`
  storyboard.
- This layout (an index of `rules/*.md`) matches Remotion's
  [`remotion-best-practices`](https://github.com/remotion-dev/skills) (see #442).
- It has **no render-and-look loop**. Correctness rests entirely on the rules being
  right.

**DominikPeters/manim-skill** (<https://github.com/DominikPeters/manim-skill>):

- The SKILL.md is 2.5 KB and teaches **one loop**:
  - render at 2 fps to PNG
  - build a **contact sheet**: *"Always read the contact sheet first… Then read at
    least one full-size frame"*
  - raise the fps to look between frames
  - fix, and render again
- It bundles a copy of the whole Manim docs under `references/manim-docs/`.
- A CI workflow
  ([`update-docs.yml`](https://github.com/DominikPeters/manim-skill/tree/master/.github/workflows))
  re-syncs that copy and cuts a release (`manim-skill.zip` / `.skill`) whenever the
  upstream docs change.

## 5. Anthropic's `skills` repo: the reference style

Repo: <https://github.com/anthropics/skills>. There are two animation-adjacent
skills:

- [`slack-gif-creator`](https://github.com/anthropics/skills/blob/main/skills/slack-gif-creator/SKILL.md)
  (7.8 KB) contains:
  - the platform's hard limits (dimensions, fps, colour count)
  - a helper library (`GIFBuilder`, `easing.py`)
  - a **validator** (`validate_gif`, `is_slack_ready`)
  - "Animation Concepts" recipes (shake, pulse, bounce, slide)
  - a taste section, "Making Graphics Look Good"

  It also says what it does *not* provide: "Rigid animation templates…".
- [`algorithmic-art`](https://github.com/anthropics/skills/blob/main/skills/algorithmic-art/SKILL.md)
  is taste-first. The agent writes an aesthetic "philosophy" `.md` *before* writing
  code, so creative intent becomes an explicit artifact of its own.

Anthropic's
[authoring guide](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices)
sets the rules that the examples above follow or break:

- "Keep SKILL.md body under 500 lines"
- "Keep references one level deep from SKILL.md". Otherwise Claude "might use
  commands like `head -100` to preview content", which means it misses information.
- "Default assumption: Claude is already very smart"
- descriptions written in third person, stating both what the skill does and when
  to use it
- feedback loops ("Run validator → fix errors → repeat")
- "Create evaluations BEFORE writing extensive documentation"
- MCP tools referenced by their fully qualified `Server:tool` names

## 6. Blender MCP: a skill-like artifact that moved into server instructions

Repo: <https://github.com/ahujasid/blender-mcp> (about 29.6k stars). This is the
most instructive *negative* result in the survey.

- It used to teach through an MCP **prompt**, `asset_creation_strategy`. PR
  [#348](https://github.com/ahujasid/blender-mcp/pull/348) (merged 2026-09-15) found
  that the prompt mostly went unread. It *"is a **prompt** — clients only receive it
  if they call `prompts/get`, and many never do"*. It had also verified that
  `initialize → instructions: None`.
- Because of that, two bugs that had been reported as the model writing bad code
  were really the agent never receiving the rules.
- The fix moved the correctness rules into `FastMCP(instructions=...)`. The source
  now explains why
  ([`server.py`](https://github.com/ahujasid/blender-mcp/blob/main/src/blender_mcp/server.py)):
  *"This is the only guidance every client is sure to get: MCP prompts are
  user-invoked, and the model has no way to fetch one. Per-tool details belong in
  tool descriptions. Kept short because instructions are injected into every
  conversation."*
- The instructions are about 1.5 KB and cover:
  - orient first (`get_addon_status`, `get_scene_info`)
  - version-proof coding rules
  - *"After changing anything, call get_viewport_screenshot() to confirm"*
  - which asset source to use for which job
- Tests assert on API identifiers rather than prose, and enforce a size budget.
- The PR cites Blender's own [`lab/blender_mcp`](https://projects.blender.org/lab/blender_mcp)
  as doing the same thing.

## 7. Higgsfield MCP (generative video): skills served as tools

Service: <https://higgsfield.ai/mcp> (a hosted server at `mcp.higgsfield.ai`). The
following is taken from the server's `initialize` instructions and tool list, as
received by this research session's own MCP client on 2026-09-29.

- The server instructions are a dense routing table. Examples: *"for multi-step
  videos call get_workflow_instructions first"*, and *"When unsure which model fits,
  call models_explore(action:'recommend')"*.
- The workflows themselves are served on demand by tools:
  - `get_workflow_instructions({workflow: "ad-multiplier" | "character-sheet" | …})`
  - `get_workflow_bundle_file`
  - `get_preset_instructions`

  In effect these are skills, delivered as tool results. They are always current
  and they need no install.
- The cost is that the routing table is injected into every conversation, and the
  copy seen here was already truncated by the client.

## 8. Motion Canvas and Revideo: nothing

Neither has any agent-facing artifact:

- No skills directory, `AGENTS.md` or `llms.txt` in either repo
  (<https://github.com/motion-canvas/motion-canvas>,
  <https://github.com/redotvideo/revideo>).
- Both `motioncanvas.io/llms.txt` and `docs.re.video/llms.txt` return an HTML page,
  not an llms.txt.
- Agents using these tools rely on training data and third-party skills.

## 9. Community aggregators (for scale, not depth)

Many ffmpeg and Manim skills are thin wrappers or restate the docs. Examples:

- [donghaozhang/ffmpeg-claude-skill](https://github.com/donghaozhang/ffmpeg-claude-skill)
- [Yusuke710/manim-skill](https://github.com/Yusuke710/manim-skill)
- [vumichien/manim-skill](https://github.com/vumichien/manim-skill), a pipeline of 4
  sub-agent roles
- [wilwaldon/Claude-Code-Video-Toolkit](https://github.com/wilwaldon/Claude-Code-Video-Toolkit),
  which bundles Remotion, Manim and FFmpeg

I skimmed them. None adds a structural pattern that the examples above don't
already show.

---

## Recurring patterns

1. **Look at the pixels, as a named step in the loop.** Every well-built example
   makes looking a required part of the loop:
   - After Effects: "read → edit → look"
   - ffmpeg-skill: step 8, "Not finished until `Look:` names that PNG"
   - DominikPeters: "Always read the contact sheet first"
   - Blender: "call get_viewport_screenshot() to confirm"
   - HyperFrames: its review loop

   Several also warn that one still frame can't show motion, and prescribe a
   contact sheet or a motion-review tool instead. Montagent already has `frame`,
   `preview`, `compare` and the contact sheet, so its skill needs to *sequence* them,
   not invent them.
2. **An entry point that routes, then gets out of the way.** HyperFrames' entry
   skill, ffmpeg-skill's "User says → Do" table, the After Effects bridge skill and
   Higgsfield's instructions all begin with a decision table: state or request in,
   skill or tool out. The HyperFrames rule "Route once, then leave" keeps the router
   from taking over the conversation.
3. **Knowledge split along the same lines.** They separate:
   - (a) the tool/format **contract** (core, bridge, rules)
   - (b) **craft and taste** (creative, motion-principles, "Making Graphics Look
     Good")
   - (c) **recipes** for a whole deliverable (workflow skills, "Animation Concepts",
     templates)
   - (d) **planning and intent** (the intent interview, `manim-composer`,
     algorithmic-art's philosophy file)

   This lines up with the split #441 already proposes: workflow, recipes, craft,
   capability discovery.
4. **Craft written as concrete numbers and parameters, not adjectives.** The After
   Effects frame-count and ease tables, slack-gif-creator's fps and colour limits,
   and HyperFrames' house style all do this. "Snappy" comes out as `{out: 15}` →
   `{in: 85}`.
5. **Tie skill text to the tool's own diagnostics.** HyperFrames names lint codes,
   and ffmpeg-skill names error `kind`s and `check.py` rows. The skill teaches the
   agent to *act on a finding code* instead of restating the rule. This matches
   #441's "point at the tools' own findings/repairs".
6. **Plan first, as an artifact of its own.** `BRIEF.md` / `STORYBOARD.md`, `scenes.md`,
   the philosophy `.md`, and `--dry-run --json` / `--plan FILE` all put intent in a
   file that can be checked before any expensive rendering.
7. **Ask little, and in bundles.** "Propose one bundle with your defaults", "one
   routing-only question", and an autonomous mode that "keeps exactly one question
   before render".
8. **Honesty rules about verification.** "Report what you *saw*, not what you
   intended." "Never claim a picture was inspected when it wasn't." "`0 sample(s)`…
   means nothing ran." "Never leave a bumped pin unverified."
9. **The skills ship with the product and are tested by it.** HyperFrames' content
   tests, Blender's instruction tests, and the After Effects skills living in the
   server repo. The durable lesson is to assert on identifiers and structure, not on
   sentences.

## Patterns that look like mistakes

1. **Relying on MCP prompts to deliver essential guidance.** Blender MCP learned
   that prompts are user-invoked and mostly never fetched. Guidance that every agent
   must have belongs in server `instructions` or tool descriptions. A skill is
   *extra* depth, not the only channel.
2. **Giant SKILL.md bodies and chained references.** HyperFrames has seven SKILL.md
   files over 30 KB (one is 66 KB), and references two or three levels deep across
   skills. That breaks both of Anthropic's stated limits (500 lines, one level deep),
   and the guide says why it matters: partial reads with `head -100`. A router that
   has to be read in full on *every* video request is the most expensive place to be
   long.
3. **Restating the format inside the skill.** HyperFrames-core and the Manim/Remotion
   `rules/*.md` books copy large parts of the API and format into the skill.
   HyperFrames needs CI content tests to stop that copy drifting, and even then it
   had to stop pinning prose. #441 already rules this out, and this survey backs
   that choice.
4. **Rule books with no loop.** manim_skill's ~22 rule files and tested examples
   never tell the agent to render and look. All of the quality burden sits on the
   rules being right, the opposite of pattern 1.
5. **Mixing in environment-specific process.** HyperFrames' entry skill carries
   pin-probing, plugin-versus-standalone install rules and a "history (trial)" block.
   That is operational upkeep sitting inside the one file every request loads. It
   belongs in the CLI (which already warns about stale skills) or in a reference
   file that is only read when needed.
6. **Bundling a full copy of the upstream docs.** DominikPeters' `references/manim-docs/`
   is kept fresh by CI, but it duplicates what the tool itself could serve. For
   Montagent, `montagent://format.md` and `schema.json` already serve that role.
7. **Instructions injected into every conversation.** Higgsfield's routing table is
   long enough to be truncated. Server instructions should stay small (Blender keeps
   to 1.5 KB and tests the budget), with detail moved to lazily fetched resources or
   skills.
8. **Doing nothing.** Motion Canvas and Revideo publish no agent guidance. Their
   `llms.txt` URLs return HTML, which is worse than a 404 because agents may try to
   parse the page.

## Product-gap flags for Montagent

These are fixes that belong in Montagent's product surface, not in a skill. Per
#441, each needs its own issue.

- **The server instructions don't give the check loop.** Montagent's `get_info`
  instructions
  ([`crates/montagent/src/mcp.rs`](../../../crates/montagent/src/mcp.rs)) say to
  edit the file and read the two resources. They don't give the minimal loop: edit →
  `validate` → look (`frame`/`preview`, or the contact sheet) → `render`. Blender MCP
  shows that this kind of guidance is only guaranteed to arrive when it is in the
  `initialize` instructions. A skill can deepen the loop, but an agent without the
  skill should still get the one-line version. Keep it inside a tested size budget,
  as Blender does.
- **No way to check skill and binary versions against each other.** HyperFrames'
  CLI warns when installed skills are stale (`skills check`), and its skills refuse
  to "continue from a remembered workflow contract". If Montagent's skills name
  verbs, flags or finding codes, the binary should expose something a skill can
  check against, such as a version or capability listing. `server_info` already
  carries `CARGO_PKG_VERSION`; the CLI side and the finding-code set are the open
  question. This overlaps with #444 and #450, so decide it there.
