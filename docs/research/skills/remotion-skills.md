# How Remotion's agent skills teach Remotion

Gathered while resolving [Research: how Remotion's agent skills teach Remotion](https://github.com/MBehtemam/Montagent/issues/442),
part of the map [End-user agents unlock Montagent through skills](https://github.com/MBehtemam/Montagent/issues/441).

**Question.** How does Remotion's agent-skill set teach an agent to use Remotion
well? How is it split, what kind of knowledge does it hold, how is it structured,
how does it stay in step with the library, how is it shipped, and is there
evidence that it helps? What of that transfers to Montagent?

**Sources read** (2026-09-29): the docs page, the published skill repo at commit
[`cf49eff`](https://github.com/remotion-dev/skills/tree/cf49eff5d4463b33966b6618c83f7295797dd028)
(2026-09-25) including its full history, and the monorepo where the skills are
written, versioned, tested and published (`remotion-dev/remotion`, `main`).
Short name for the skill repo below: **`skills@cf49eff`**.

## Verdict

Remotion's skills are **a router plus eleven topic skills, mostly Remotion idiom
and taste written as short imperative rules with code examples**. They are
maintained inside the library's own monorepo, stamped with the library version
on every release, published by a script to a mirror repo, installed by the
library's own CLI, scaffolder and Studio, and tested with a before/after eval
harness that has an agent make real videos. Current API facts are mostly *not*
copied into the skills: the `remotion-docs` skill sends the agent to live docs
fetched as Markdown. Remotion went as far as deprecating its docs MCP server for
that skill.

## 1. How the set is split

Twelve skills, all published together
([docs](https://www.remotion.dev/docs/ai/skills),
[README](https://github.com/remotion-dev/skills/blob/cf49eff5d4463b33966b6618c83f7295797dd028/README.md)):

| Skill | What it is | Size of SKILL.md | Supporting files |
|---|---|---|---|
| `remotion-best-practices` | Router: "This skill encompasses all other skills" | 59 lines | embeds all others |
| `remotion-create` | Workflow: scaffold project, design, open preview | 85 | `video-layout.md`, `tailwind.md` |
| `remotion-markup` | Core idiom + taste for writing compositions | 358 | 29 rule files |
| `remotion-studio` | One command + flag table | 24 | none |
| `remotion-render` | Render/still commands + pointer to CLI docs | 35 | `transparent-videos.md` |
| `remotion-maps` | Picks one of five map "techniques" | 36 | `techniques/*/TECHNIQUE.md`, references, scripts, asset code |
| `remotion-captions` | `Caption` type + three task files | 36 | 3 |
| `remotion-saas` | App architecture (Player, Lambda, etc.) | 33 | 3 |
| `remotion-interactivity` | How to write code the Studio can edit | 315 | none |
| `remotion-docs` | How to search and fetch live docs | 46 | none |
| `remotion-upgrade` | Upgrade procedure (packages + skills) | 31 | none |
| `remotion-multimedia` | Mediabunny metadata lookups | 20 | 3 |

Line counts from `wc -l` on `skills@cf49eff`. The 12 SKILL.md files total about
1,080 lines; all Markdown including the router's embedded copies totals ~17,900.

**The split was found by trial.** The repo history shows three shapes
([commits](https://github.com/remotion-dev/skills/commits/main)):

1. **2026-01-19, briefly: one skill per topic** — 21, then 25 separate skills
   (`skills/remotion/3d`, `animations`, `timing`, `fonts`, ...) on launch day.
2. **2026-01-19 to 2026-07: one skill + `rules/`** — a single `remotion-best-practices`
   SKILL.md that is a table of contents over ~40 `rules/*.md` files, each with a
   one-line "when to load" description (e.g. commit `d5d3955`,
   `skills/remotion/SKILL.md`).
3. **2026-07-11 onward: router + topic skills** — 15, then 9-11, settling at 12 by
   2026-08-04. The old per-rule files became supporting files under `remotion-markup`,
   and task-shaped skills (`create`, `render`, `studio`, `upgrade`, `docs`) were
   split out.

Granularity now follows **task** (create / render / preview / upgrade / look up
docs) plus **domain** (maps, captions, SaaS, multimedia), with the bulk of
craft knowledge in one large `markup` skill. Install counts on
[skills.sh](https://skills.sh/remotion-dev/skills) show that users mostly
install the router: `remotion-best-practices` has ~552K installs, and every other
skill has 70-106K.

## 2. What kinds of knowledge they hold

All four kinds are present, in different skills:

- **Idiom / "the Remotion way" (most of it).** `remotion-markup` opens with rules
  such as "Drive animations using `useCurrentFrame()` and `interpolate()`. CSS
  `transition` or `animation` will not render correctly", "Use `scale`,
  `translate`, `rotate` CSS properties over `transform`", with 👍/👎 code pairs
  ([SKILL.md](https://github.com/remotion-dev/skills/blob/cf49eff5d4463b33966b6618c83f7295797dd028/skills/remotion-markup/SKILL.md)).
  This is the knowledge an agent gets wrong by default: web habits that break
  frame-accurate rendering.
- **Workflow.** `remotion-create` is a step-by-step procedure: inspect the folder
  (and don't treat `.env`/`.git` as disposable), scaffold with exact flags,
  design, start the Studio, and **"Only render if the user explicitly asks for
  it"**. It ends with a handoff back to the router
  ([SKILL.md](https://github.com/remotion-dev/skills/blob/cf49eff5d4463b33966b6618c83f7295797dd028/skills/remotion-create/SKILL.md)).
  `remotion-upgrade` is a numbered procedure with a verification step
  ([SKILL.md](https://github.com/remotion-dev/skills/blob/cf49eff5d4463b33966b6618c83f7295797dd028/skills/remotion-upgrade/SKILL.md)).
- **Craft / taste.** `video-layout.md` is pure design judgment: "You are
  designing a video, not a webpage. Decide what the viewer should notice first
  in each scene", safe areas (80 px sides / 100 px top-bottom at 1080 wide),
  minimum type sizes (84 px headline, 44 px supporting), "Do not add redundant
  elements"
  ([file](https://github.com/remotion-dev/skills/blob/cf49eff5d4463b33966b6618c83f7295797dd028/skills/remotion-create/video-layout.md)).
  `timing.md` prescribes easing choices ("A nice push movement with no bounce:
  `Easing.spring({damping: 200})`"). The maps technique files record known
  failures and their fixes: shimmer from per-frame `map.jumpTo()` → render a
  fixed map plate once and move it with CSS
  ([render-stability.md](https://github.com/remotion-dev/skills/blob/cf49eff5d4463b33966b6618c83f7295797dd028/skills/remotion-maps/techniques/mapbox/references/render-stability.md)).
- **Recipes.** Most `remotion-markup/*.md` files are recipes (light leaks, text
  highlights, audio visualization, voiceover via ElevenLabs, silence detection
  with FFmpeg). `remotion-maps` includes runnable assets (`CesiumFlythrough.tsx`,
  `prep-geo.mjs`).
- **API facts are kept thin on purpose.** Where they appear, they are the facts
  the agent needs in order to act (the `Caption` type, the `from` /
  `durationInFrames` / `trimBefore` props, a Studio flag table). Full option lists
  are linked out: "Full list of options: https://www.remotion.dev/docs/cli/render.md"
  ([remotion-render](https://github.com/remotion-dev/skills/blob/cf49eff5d4463b33966b6618c83f7295797dd028/skills/remotion-render/SKILL.md)).

Two cross-cutting rules come up again and again and are worth noting:
**"Preserve user changes"** (the user may edit the code outside the
conversation; don't overwrite surprising changes), which appears in both the
router and `markup`; and **"write it so the Studio can edit it"** (inline
literal values, one JSX node per clip, no `.map()` over clips), which runs
through `markup`, `timing.md`, `video-editing.md` and a whole
`remotion-interactivity` skill. The second rule exists only because Remotion is
code: the skills are there to keep an agent's code within the subset of code
that the GUI can round-trip.

## 3. How they are structured

- **Frontmatter is minimal**: `name`, a very short `description` (e.g. "Router
  for all Remotion skills", "Export a Remotion video"), and `version` (the
  library version). The descriptions are too short to trigger automatically in a
  reliable way. The docs and README present every skill as a **slash command with
  an example prompt** (`/remotion-create Make a promo video for a record store`),
  so invocation is expected to be explicit or to go through the router.
- **Progressive disclosure is done by routing prose.** Each section is one
  heading plus "When X, load [file](file.md)". The router does this across
  skills and `markup` does it across its 29 rule files. `remotion-maps` adds a
  third level: "Choose exactly one technique from the intended shot, then load
  only that technique's `TECHNIQUE.md`. Every technique directory is
  self-contained"
  ([SKILL.md](https://github.com/remotion-dev/skills/blob/cf49eff5d4463b33966b6618c83f7295797dd028/skills/remotion-maps/SKILL.md)).
- **Every skill has an exit.** Non-router skills start with "If this is not
  relevant, load Remotion Best Practices instead" and hand back when they finish.
- **Embedding for portability.** The Agent Skills format requires file references
  to stay inside the skill's own directory. At publish time
  `prepare-embedded-skills.ts` copies every sibling skill *into* the router
  (renaming each `SKILL.md` to `REFERENCE.md`) and turns cross-skill links into
  plain text
  ([script](https://github.com/remotion-dev/skills/blob/cf49eff5d4463b33966b6618c83f7295797dd028/scripts/prepare-embedded-skills.ts)).
  Installing only `remotion-best-practices` therefore gets you everything. That
  explains its install count.
- **Per-agent UI metadata**: each skill has `agents/openai.yaml` (display name,
  short description, brand colour, icon, default prompt) for Codex.

## 4. How they avoid drifting from the real API and version

- **Same repo as the library.** The skills are written in
  `remotion-dev/remotion/packages/skills` (the mirror's `package.json` says so:
  `"repository": ".../remotion/tree/main/packages/skills"`). The mirror is
  regenerated by `packages/it-tests/src/templates/publish.ts`, which copies the
  folder, runs the embed step and commits "Update template". That is why every
  commit in `remotion-dev/skills` has that message
  ([publish.ts](https://github.com/remotion-dev/remotion/blob/main/packages/it-tests/src/templates/publish.ts)).
- **Version-stamped per release.** The monorepo's `set-version.ts` rewrites
  `version:` in every public SKILL.md, and in the Claude Code / Codex / Kimi plugin
  manifests, to the release version (currently `4.0.529`)
  ([set-version.ts](https://github.com/remotion-dev/remotion/blob/main/set-version.ts)).
- **Dogfooded.** Public skills are symlinked into the monorepo's own
  `.agents/skills`, so Remotion's own coding agents use them. CI checks the
  symlinks with `bun run checkskills`
  ([skill-locations](https://github.com/remotion-dev/remotion/blob/main/.agents/skills/skill-locations/SKILL.md),
  [sync-agent-skills.ts](https://github.com/remotion-dev/skills/blob/cf49eff5d4463b33966b6618c83f7295797dd028/scripts/sync-agent-skills.ts)).
  A link validator checks every relative Markdown link
  ([validate-links.ts](https://github.com/remotion-dev/skills/blob/cf49eff5d4463b33966b6618c83f7295797dd028/scripts/validate-links.ts)).
- **Defer to live docs rather than copying them.** `remotion-docs` tells the agent
  to search Algolia, append `.md` to any docs URL, and "Implement using the
  current documentation rather than memorized API knowledge"
  ([SKILL.md](https://github.com/remotion-dev/skills/blob/cf49eff5d4463b33966b6618c83f7295797dd028/skills/remotion-docs/SKILL.md)).
- **Upgrades carry the skills along.** `npx remotion upgrade` "also updates
  project-local Remotion skills", and the manual path runs `npx skills update`
  on all twelve.
- **Push version-sensitive choices into tools.** The skills say "Use
  `npx remotion add` to add new packages with the right version" instead of
  writing version numbers into prose.

## 5. How they are distributed and installed

Remotion offers every channel it can:

- `npx skills add remotion-dev/skills` (the Vercel `skills` CLI; the format is
  [agentskills.io](https://agentskills.io/home)).
- `npx remotion skills add|update`, which wraps a **pinned** `skills@1.5.26` and
  installs to `.agents/skills` with `.claude/skills` symlinked to it
  ([docs](https://www.remotion.dev/docs/cli/skills),
  [skills.ts](https://github.com/remotion-dev/remotion/blob/main/packages/cli/src/skills.ts)).
- Offered during scaffolding: `bun create video` / `create-video` runs the same
  install ([install-skills.ts](https://github.com/remotion-dev/remotion/blob/main/packages/create-video/src/install-skills.ts)).
- **In the Studio GUI**: a Skills settings panel lists every skill with
  Install / Remove buttons, served by a Studio-server route
  ([install-remotion-skill.ts](https://github.com/remotion-dev/remotion/blob/main/packages/studio-server/src/preview-server/routes/install-remotion-skill.ts)).
- **Agent plugins** that bundle the skills for Claude Code
  (`claude plugin marketplace add remotion-dev/claude-code-plugin`), Codex,
  Cursor, GitHub Copilot CLI and Kimi Code
  ([plugins](https://www.remotion.dev/docs/ai/plugins),
  [Claude Code plugin](https://www.remotion.dev/docs/ai/claude-code-plugin)).
- The getting-started flow for coding agents includes skill install as a normal
  step between `npm install` and `npm run dev`
  ([coding-agents](https://www.remotion.dev/docs/ai/coding-agents)).

**The MCP was deprecated in favour of a skill.** Remotion's docs MCP server
(`remotion-documentation`) is deprecated, and users are told to use
`/remotion-docs` instead. The reasons given: its data "can be less current than
the Remotion documentation", "Remotion pays the token costs", and "Installing MCP
servers is difficult, and agents do not invoke them reliably"
([mcp docs](https://www.remotion.dev/docs/ai/mcp),
[issue #9055](https://github.com/remotion-dev/remotion/issues/9055); the issue
gives no numbers, only qualitative reports). Note that this was a
*docs-retrieval* MCP, not an action/tool server like Montagent's.

## 6. Evidence that they help

- **An internal eval harness.** `packages/skills-evals` runs scenarios (vertical
  and landscape promo videos with required phrases, a LA→NY→Paris map trip with
  a 3D Eiffel Tower, a bar+line chart, a transparent ProRes YouTube lower third)
  with a coding agent (the [Pi](https://pi.dev) harness, model
  `openai-codex/gpt-5.5`). The skills are copied into a blank template, and the
  agent must render a final MP4. A `compare` mode runs **before/after** a skills
  git ref, repeated several times, records a hash of the skill snapshot and a
  skill diff, and collects the rendered videos for review
  ([scenarios.ts](https://github.com/remotion-dev/remotion/blob/main/packages/skills-evals/scenarios.ts),
  [run-skill-eval.ts](https://github.com/remotion-dev/remotion/blob/main/packages/skills-evals/src/run-skill-eval.ts),
  [manifest.ts](https://github.com/remotion-dev/remotion/blob/main/packages/skills-evals/src/manifest.ts)).
  I found no automated grader in the files I read. Judging appears to be a
  human looking at the output videos. No results are published.
- **Adoption, not efficacy.** skills.sh reports ~1.6M total installs for the
  pack ([skills.sh](https://skills.sh/remotion-dev/skills)). The creator's
  launch post is widely cited (a video "created ... without writing any code"),
  but only through secondary write-ups
  ([robotostudio](https://robotostudio.com/blog/how-to-use-remotion-agent-skills-with-claude-code)).
  I found no first-party number for how much the skills improve output quality.
  **Treat "they help" as asserted by Remotion's investment, not measured
  publicly.**

## What transfers to Montagent / what doesn't

Montagent's surface is different: a declarative JSON project edited with
ordinary file tools, nine MCP verbs, and two resources (`montagent://schema.json`
for shape, `montagent://format.md` for the rules a schema can't express). There
is no code for the agent to write.

### Transfers

1. **Router + a few task skills, not one skill per topic.** Remotion started
   with 25 fine-grained skills and moved away from them within hours; users
   install the router ~5× more than any leaf. For Montagent: one entry skill
   that routes, plus task-shaped skills that match real jobs (make a video from
   assets / edit an existing project / check and fix / render and deliver).
2. **Put the knowledge the agent lacks in the skill; link to the rest.** Remotion's
   skills are mostly idiom and taste that a model gets wrong by default, while
   API facts go to live docs. For Montagent the "live docs" already exist as
   the two MCP resources. A skill should say "read `schema.json` / `format.md`",
   not restate them.
3. **Craft rules with numbers.** `video-layout.md` (safe areas, minimum type
   sizes, "what should the viewer notice first") is directly portable, and it is
   the kind of knowledge Montagent's verbs can check but cannot make up.
4. **Workflow guardrails.** "Only render if the user explicitly asks" and
   "preserve user changes made outside the conversation" map straight onto
   Montagent (render is expensive; project files are hand-editable).
5. **"Choose exactly one technique, load only that file."** A useful pattern for
   any branching choice (e.g. how to verify: `frame` vs `preview` vs `measure`).
6. **Version-stamp the skills from the release script and publish from the
   product repo.** Same repo, same version, dogfooded by the maintainers' own
   agents, link-checked in CI.
7. **Before/after skill evals on real end-to-end scenarios** that must produce a
   rendered file. Montagent can go further than Remotion here: `validate`,
   `measure` and `compare` can grade a run by machine, where Remotion appears to
   rely on a human watching the videos.
8. **Ship through the product itself** (a CLI subcommand that installs the
   skills, and an offer at `create_project` time), not only through a
   third-party installer.

### Doesn't transfer

1. **The "write code the GUI can round-trip" rules** (inline `interpolate`, one
   JSX node per clip, `Interactive.*`). Montagent's format *is* the editable
   structure, so there is nothing to teach here. That removes a large part of
   Remotion's skill volume.
2. **Recipe-heavy library tours** (Lottie, Three.js, Mapbox, ElevenLabs, FFmpeg
   silence detection). They exist because Remotion can do anything React can.
   Montagent has a closed element vocabulary, so the schema already lists it.
3. **Package-management and upgrade procedures.** Montagent is a single binary.
4. **"MCP is unreliable, use a skill instead".** Remotion's deprecated MCP only
   duplicated docs. Montagent's verbs *do things a text editor cannot*
   (render, measure, frame). The lesson that does carry over is narrower:
   agents may not reach for an MCP server on their own, so a skill should name the
   verb to call at each step.
5. **Terse descriptions that rely on slash-command invocation.** Remotion can get
   away with them because users type `/remotion-create`. Montagent's skills are
   aimed at strangers' agents, which need descriptions that trigger on their own.
