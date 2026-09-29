# Research: distribution mechanics for skills that ship alongside a CLI/MCP binary

Research for [#444](https://github.com/MBehtemam/Montagent/issues/444), part of the map [#441](https://github.com/MBehtemam/Montagent/issues/441).

**Date of research:** 2026-09-29. Every version number, support claim and status below is as of that date; this area moves weekly.

**This document does not pick a distribution channel.** [#448](https://github.com/MBehtemam/Montagent/issues/448) makes that trade-off. This document surfaces the facts it needs.

**Sourcing rule:** each claim cites the source that owns it: the spec, the tool's own docs, or its source code. Where something is a proposal, a prototype or undocumented, it is flagged **[proposal]**, **[prototype]** or **[undocumented]**. Where a fact came only from a search-engine summary, it is marked **[secondary]**.

## The shape of the problem, in Montagent's terms

Montagent ships as one Rust binary (`cargo install montagent`, or a GitHub Releases tarball for six targets). It is both a CLI and a stdio MCP server (`montagent mcp`). The server already exposes nine tools, two resources (`montagent://schema.json` and `montagent://format.md`), a `serverInfo.version` taken from `CARGO_PKG_VERSION`, and an `instructions` string ([`crates/montagent/src/mcp.rs`](../../../crates/montagent/src/mcp.rs), `get_info`). It declares `tools` and `resources` capabilities and no `prompts` capability. It uses `rmcp` 3.4 ([`crates/montagent/Cargo.toml`](../../../crates/montagent/Cargo.toml)).

A skill written for strangers' agents has to answer two questions: **how does it get onto their machine or into their agent's context**, and **how does it stay in step with the binary they actually installed**. The mechanisms below differ mainly on those two axes.

## Mechanisms

### 1. The Agent Skills format itself (the common denominator)

- A skill is a directory with a `SKILL.md`: YAML frontmatter plus a Markdown body. The optional `scripts/`, `references/` and `assets/` directories sit beside it. ([spec](https://agentskills.io/specification))
- The spec defines six frontmatter fields: `name` (required, must match the directory name), `description` (required, at most 1024 characters), `license`, `compatibility` (at most 500 characters, *"Indicates environment requirements (intended product, system packages, network access, etc.)"*), `metadata` (*"a map from string keys to string values"*), and `allowed-tools` (experimental). ([spec](https://agentskills.io/specification))
- **The spec has no `version` field.** Its own example puts `version: "1.0"` under `metadata`, and the spec says only that *"Clients can use this to store additional properties"*. No client documented below reads `metadata.version` for anything. ([spec](https://agentskills.io/specification))
- The spec does not say where skills live on disk. The client-implementation guide calls `.agents/skills/` (project) and `~/.agents/skills/` (user) *"a widely-adopted convention for cross-client skill sharing"*. It notes that some clients also scan `.claude/skills/` *"for pragmatic compatibility"*. ([adding-skills-support](https://agentskills.io/client-implementation/adding-skills-support))
- The agentskills.io client showcase lists 46 products, among them Claude Code, Claude, ChatGPT & Codex, Cursor, Gemini CLI, GitHub Copilot, VS Code, Goose, OpenCode, Amp, Kiro, Roo Code, Junie and fast-agent. ([clients](https://agentskills.io/clients))

### 2. `npx skills` (vercel-labs/skills)

This is the tool that wrote this repo's [`skills-lock.json`](../../../skills-lock.json). Facts below come from the README and source at commit `3694740`, 2026-09-28, package version `1.7.0` ([repo](https://github.com/vercel-labs/skills)).

- **Install:** run `npx skills add <source>`. The source can be GitHub shorthand, a full GitHub, GitLab or Azure Repos URL, any git URL, a local path, a direct `SKILL.md` URL or archive URL, or a site that publishes `/.well-known/agent-skills/index.json`. The README lists four agents "and 75 more". For each agent it writes into that agent's directory, project-scoped by default or global with `-g`. It symlinks from one canonical copy by default, or copies with `--copy`. ([README](https://github.com/vercel-labs/skills#readme); `src/providers/wellknown.ts`, whose discovery index uses schema `https://schemas.agentskills.io/discovery/0.2.0/schema.json`)
  - This repo shows the layout it produces: `.agents/skills/court/` holds the canonical copy, and `.claude/skills/court` is a symlink to `../../.agents/skills/court`.
- **Discovery inside a source repo:** the CLI scans `skills/`, the agent directories (`.agents/skills`, `.claude/skills`, …), and any Claude Code `.claude-plugin/marketplace.json` or `plugin.json` it finds. A Claude Code plugin repo can therefore also be installed with `npx skills add` (`src/skills.ts`, `src/plugin-manifest.ts`).
- **Pinning:** a `#<ref>` fragment selects a branch or tag. A full commit SHA also works, through a fetch-by-SHA fallback after `git clone --branch` fails (`src/source-parser.ts`, `src/git.ts` `cloneAtSha`). The ref is recorded in the lock entry.
- **Lock file:** the project lock `skills-lock.json` is version 1. Each entry records `source`, optional `sourceUrl`, `ref`, `sourceType` (`github`, `node_modules`, `local`, …), `skillPath` and `computedHash`, a SHA-256 over the skill folder's files on disk. It is *"meant to be checked into version control"* (`src/local-lock.ts`). The lock records **what was installed, not which tool version it targets**. It has no semver or range concept.
- **Update and check:** `npx skills update` (aliases `check` and `upgrade`) re-resolves each entry against its recorded ref and reinstalls changed skills. `experimental_install` restores from `skills-lock.json`. `experimental_sync` installs `SKILL.md` files found in `node_modules` packages (`src/cli.ts`, `src/update.ts`, `src/sync.ts`).
- **Requirements and side effects:** it needs Node/npx on the user's machine. It sends telemetry unless `DISABLE_TELEMETRY` or `DO_NOT_TRACK` is set (`src/telemetry.ts`). Skills are discoverable on [skills.sh](https://skills.sh).

### 3. Claude Code plugins and marketplaces

All facts in this section come from the Claude Code docs as of v2.1.284: [manifest reference](https://code.claude.com/docs/en/plugins-reference), [marketplace reference](https://code.claude.com/docs/en/plugins/marketplace-reference), [loading reference](https://code.claude.com/docs/en/plugins/loading) and [create a marketplace](https://code.claude.com/docs/en/plugin-marketplaces).

- **What a plugin can bundle:** `skills/<name>/SKILL.md`, MCP servers (in `.mcp.json` or inline under `mcpServers`), hooks, agents and a `bin/` directory whose executables go on the Bash tool's `PATH`. Plugin skills are namespaced as `/plugin-name:skill-name`.
- **A plugin can declare an MCP server that is a command on `PATH`,** for example `"command": "montagent", "args": ["mcp"]`. A plugin can therefore ship a skill and the MCP wiring together. It cannot install the Rust binary itself unless the plugin carries per-platform binaries in `bin/`, and claude.ai and Cowork refuse to install a plugin that has `bin/`.
- **Marketplace:** a repo with `.claude-plugin/marketplace.json`. Users add it with `claude plugin marketplace add owner/repo` (or `@ref`/`#ref`) and install with `claude plugin install <name>@<marketplace>`. The Montagent repo itself could be the marketplace, with a relative-path plugin source.
- **Plugin source types:** relative path, `github` (`repo`, `ref`, `sha`), `url`, `git-subdir`, `npm` (`package`, `version` range, `registry`), `archive` (`url`, `sha256`) and `command`.
- **Versioning:** the resolved version is taken first from the manifest's `version`, then from the marketplace entry's `version`, and otherwise derived from the source: a 12-character commit SHA for git sources, `unknown` for npm. *"A manifest that pins `"version": "1.0.0"` keeps every user on the cached copy until its author changes the string, however many commits they push."*
- **Updates:** auto-update is **off by default for third-party marketplaces**. It is on only for Anthropic's official marketplaces and marketplaces added from claude.ai. When it is on, it runs after a random delay of up to 10 minutes in an interactive session.
- **`command` source (requires v2.1.229 or later).** This is the mechanism most directly shaped like "a skill that tracks the installed binary". A marketplace entry names a shell command, and Claude Code runs it on the user's machine. The command must print the absolute path of a plugin directory. Claude Code runs it at install and update, **once per session in the background**, and when the cache is missing. The plugin's version becomes `<manifest version>-<hash of output>`, so a changed output is picked up automatically, with no auto-update setting involved. The user reviews and accepts the command string first. Administrators can disable command sources with `disableCommandPluginSources`. Example from the docs: `"command": "my-tool claude-plugin-path"`.
- **Plain skills without a plugin:** Claude Code reads `~/.claude/skills/` and `.claude/skills/`, plus enterprise and plugin locations ([skills](https://code.claude.com/docs/en/skills)). The loading docs name no `.agents/skills/` path for Claude Code, and the `skills` CLI table maps Claude Code to `.claude/skills/` ([README](https://github.com/vercel-labs/skills#supported-agents)).
- **Dynamic context injection (Claude Code only).** A line `` !`<command>` `` in a `SKILL.md` body runs before the skill reaches the model, and the command's output replaces the line ([skills § Inject dynamic context](https://code.claude.com/docs/en/skills#inject-dynamic-context)). A skill can therefore inline `` !`montagent --version` `` at load time. The docs call this a Claude Code extension that doesn't function in claude.ai chat or the API, and it is not run for skills synced from claude.ai.

### 4. Agent Plugins (agent-plugins.org): a cross-vendor bundle format

- *"An open, vendor-neutral standard for packaging reusable components into portable plugins."* Version 1.0.0 covers two component types: Agent Skills (in `skills/`) and MCP servers (in `mcp.json`). The manifest is a root `plugin.json` carrying a `version`. ([agent-plugins.org](https://agent-plugins.org/))
- The standard was announced on 2026-08-06. The steering committee is Amazon, Cursor, Microsoft, OpenAI and Vercel. Launch clients were ChatGPT and Codex, Cursor, GitHub Copilot, Kiro and VS Code. **Anthropic and Claude Code are not named.** ([Vercel blog](https://vercel.com/blog/introducing-agent-plugins))
- OpenAI's plugin docs use this layout: root `plugin.json` (schema `https://agent-plugins.org/schemas/1.0.0/plugin.schema.json`), `mcp.json` and `skills/`, with `.codex-plugin/plugin.json` as a *"compatibility fallback"*. Marketplaces live at `.agents/plugins/marketplace.json`, and **the legacy `.claude-plugin/marketplace.json` location is also read**. Entry sources include `local`, `url`/`git-subdir` with `ref`/`sha`, and `npm` with a version range. The CLI command is `codex plugin marketplace add owner/repo`. ([OpenAI plugins](https://developers.openai.com/plugins/build/plugins))
- Cursor: *"To bring skills in from a GitHub repository, package them in a plugin and publish that plugin through a marketplace."* ([Cursor skills](https://cursor.com/docs/context/skills))

### 5. Gemini CLI extensions and `gemini skills install`

- `gemini skills install <git-url|path> [--path <subdir>] [--scope user|workspace]`. Discovery order runs from built-in, to extension, to user (`~/.gemini/skills/` or the `~/.agents/skills/` alias), to workspace (`.gemini/skills/` or the `.agents/skills/` alias). ([Gemini CLI skills](https://geminicli.com/docs/cli/skills/))
- An extension (`gemini-extension.json` with `name`, `version`, `mcpServers`, …) can bundle `skills/<name>/SKILL.md` alongside MCP servers. It installs with `gemini extensions install <source> [--ref <ref>] [--auto-update]` and updates with `gemini extensions update`. ([extension reference](https://geminicli.com/docs/extensions/reference/))

### 6. Serving skills from the MCP server itself

**Plain MCP primitives, shipped today:**

- **Resources.** Montagent already publishes two. Claude Code users can `@`-mention a resource as `@server:protocol://path` ([Claude Code MCP](https://code.claude.com/docs/en/mcp)). A resource is not a skill: no client lists resources in its skill registry or loads them by description.
- **Prompts.** These are user-invoked. Claude Code lists them as `/servername:promptname (MCP)`, or `/mcp__servername__promptname` ([Claude Code MCP](https://code.claude.com/docs/en/mcp)). The model does not auto-trigger them the way it triggers a skill. Montagent declares no `prompts` capability.
- **Server `instructions`.** Montagent uses this already. Claude Code *"truncates each tool description and each server's instructions at 2,048 characters by default"* ([Claude Code MCP](https://code.claude.com/docs/en/mcp)). SEP-2640 names this size bound as a motivation for the extension.

**The MCP Skills extension (SEP-2640, `io.modelcontextprotocol/skills`): Final, but hosts do not support it yet.**

- Status: **Final**, merged on 2026-09-13 ([PR #2640](https://github.com/modelcontextprotocol/modelcontextprotocol/pull/2640); [SEP text](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/main/seps/2640-skills-extension.md)). It is an Extensions Track SEP, so both server and client must opt in through `capabilities.extensions`.
- Mechanism: each file of a skill becomes a resource, conventionally at `skill://<skill-path>/<file>`. The server must implement `skills/list` and `skills/get`. Each entry carries the verbatim frontmatter and a complete `resources` manifest with SHA-256 `digest` and `size` for every file. `resources/directory/read` is optional. The limits are 512 files and 16 MiB per skill. The skill format is delegated entirely to agentskills.io.
- Its motivation names Montagent's situation exactly: *"A server and the skill that teaches an agent to use it are versioned, discovered, and installed separately."*
- Host-side rules that matter for us: MCP-served skill content is **untrusted input**, and its origin must be visible to the model. `allowed-tools` MUST be ignored unless the user approves. The host may not run code on the skill's behalf without per-skill approval. Approval is content-bound to the digests, so any change revokes it and forces a re-prompt. An MCP skill must not shadow a same-named local skill.
- **Client support as of 2026-09-29.** The [MCP extension support matrix](https://modelcontextprotocol.io/extensions/client-matrix) (last changed 2026-09-18, [PR #3372](https://github.com/modelcontextprotocol/modelcontextprotocol/pull/3372)) marks Skills support as:
  - full: mcpc only;
  - partial: ChatGPT, fast-agent, MCP Inspector;
  - none: Claude (web), Claude Desktop, VS Code Copilot, Cursor, Goose.
  - Claude Code, Codex and Gemini CLI are not in the matrix.
  - The SEP lists **[prototype]** hosts: forks of gemini-cli and codex (`olaservo/*`), fast-agent, and *"Claude Code: prototyped internally at Anthropic; not yet public"*.
- **ChatGPT's "partial" support is import-time only.** It supports *"a bounded, static subset"*. Skills are snapshotted when the plugin is submitted via Scan Tools (at most five skills, 256 KiB per `SKILL.md`, 5 MiB per skill), not read live at runtime. ([OpenAI MCP server docs](https://developers.openai.com/plugins/build/mcp-server#import-skills-from-the-mcp-server))
- **Claude Code: [undocumented].** Its MCP docs never mention `skill://` or the extension. Its changelog does mention *"MCP servers' skills and prompts"* (2.1.282–2.1.284), in a note that MCP servers named `anthropic-skills` or `claude-ai` *"list no skills or prompts"*. That wording implies some MCP-served skill listing exists. I could not confirm what triggers it or which protocol it uses ([CHANGELOG](https://github.com/anthropics/claude-code/blob/main/CHANGELOG.md)).
- **Rust SDK: [proposal].** `rmcp` support is an open PR, [rust-sdk#1286](https://github.com/modelcontextprotocol/rust-sdk/pull/1286) "feat(rmcp): implement SEP-2640 MCP Skills Extension" (opened 2026-09-18, not merged). The latest release is `rmcp-v3.5.0` (2026-09-28). The Python, C# and Go SDKs have reference implementations linked from the SEP. Hand-rolling `skills/list` and `skills/get` on `rmcp` 3.4 would mean custom request handling. **I did not verify that `rmcp` exposes a hook for custom methods.**
- Adoption: a secondary source counts 2 of 572 tested MCP servers declaring the extension (Hugging Face and one other) **[secondary]** ([API Evangelist, 2026-09-22](https://apievangelist.com/2026/09/22/skills-over-mcp-is-final-and-now-it-needs-servers/)).

## Portability matrix

| Client | Reads `SKILL.md` from | Bundle format with skills + MCP | Skills served over MCP (SEP-2640) | Skill-level version or dynamic check |
| --- | --- | --- | --- | --- |
| Claude Code | `.claude/skills/`, `~/.claude/skills/`, plugin `skills/`, enterprise ([docs](https://code.claude.com/docs/en/skills)) | Claude Code plugin (`.claude-plugin/`, `.mcp.json`) via marketplace; also a `command` source | [undocumented]: SEP says "prototyped internally, not yet public"; changelog hints | Plugin `version`/ref/sha; `` !`cmd` `` injection; `command` source re-runs per session |
| Codex / ChatGPT | `.agents/skills` (repo up to root), `~/.agents/skills`, `/etc/codex/skills` ([docs](https://learn.chatgpt.com/docs/build-skills)) | Agent Plugins (`plugin.json`, `mcp.json`, `skills/`); reads legacy `.claude-plugin/marketplace.json` | ChatGPT: partial, import-time snapshot; Codex: [prototype] fork only | Plugin `version`; ref/sha; npm range. `agents/openai.yaml` can declare an MCP dependency |
| Cursor | `.agents/skills`, `.cursor/skills`, `~/.agents/skills`, `~/.cursor/skills`, **plus** `.claude/skills`, `.codex/skills` and user equivalents ([docs](https://cursor.com/docs/context/skills)) | Agent Plugins via marketplace (the only documented GitHub route) | Not supported (matrix) | Plugin `version` |
| Gemini CLI | `.gemini/skills` or `.agents/skills`, `~/.gemini/skills` or `~/.agents/skills` ([docs](https://geminicli.com/docs/cli/skills/)) | Gemini extension (`gemini-extension.json`, `skills/`) | [prototype] fork only | Extension `version`; `--ref`; `--auto-update` |
| VS Code / GitHub Copilot | `.github/skills`, `.claude/skills`, `.agents/skills`, `~/.copilot/skills`, `~/.claude/skills`, `~/.agents/skills` ([docs](https://code.visualstudio.com/docs/copilot/customization/agent-skills)) | Agent plugins | Not supported (matrix) | Plugin `version` |
| Any of 79 agents via `npx skills` | Writes each agent's own directory ([table](https://github.com/vercel-labs/skills#supported-agents)) | Reads Claude plugin manifests as a source | n/a | `#ref`; `skills-lock.json` hash; `update` |

Observations from the matrix, without choosing between them:

- **`.agents/skills/` plus `.claude/skills/` covers the majority of named clients by path alone.** Claude Code reads `.claude/skills/`. Cursor and VS Code read both directories. Codex and Gemini CLI read `.agents/skills/`.
- **No single bundle format spans everyone.** Claude Code uses its own plugin format. Codex, Cursor, Copilot, Kiro and VS Code use Agent Plugins. Gemini uses extensions. The overlaps are partial: Codex reads a `.claude-plugin/marketplace.json`, and `npx skills` reads Claude plugin repos.
- **MCP-served skills would be the only channel where the binary ships its own skill.** As of this date, no major coding-agent host ships support for it publicly.

## Pinning a skill to, or checking it against, the installed binary

Options that exist today. The first four decide what gets installed; the last three detect a mismatch at run time.

1. **Git ref/tag pin at install time.** Available through `npx skills add MBehtemam/Montagent#v0.1.0`, Claude Code plugin sources (`ref` + `sha`), Codex marketplace entries (`ref`/`sha`) and `gemini extensions install --ref`. The pin selects the skill's version. It does not know which binary is installed, so the user still has to pick a tag that matches.
2. **Plugin `version` field.** Claude Code, Agent Plugins and Gemini extensions all have one. In Claude Code it controls cache and update identity, not compatibility with an external tool.
3. **npm version range.** Available for Claude Code and Codex `npm` sources. This would only help if the skill were published to npm, which Montagent is not.
4. **The binary emits its own skill directory.** A subcommand prints or writes the skill that matches the build:
   - Claude Code `command` source: `"command": "montagent <subcommand-that-prints-a-plugin-dir>"`, re-run every session, version derived from the output hash. The skill tracks the installed binary automatically. Claude Code only; the user accepts the command once; admins can disable it.
   - For other clients, the same subcommand could write into `.agents/skills/` or `~/.agents/skills/`. No client re-runs such a command automatically, so this is a one-shot install.
5. **MCP-served skill (SEP-2640).** The skill comes from the running binary, so it cannot drift from it. Digests give content integrity, and the frontmatter `metadata` can carry a version. This depends on host support and on `rmcp` support, neither of which has shipped (see above).
6. **Load-time check in the skill body.** Claude Code only: `` !`montagent --version` `` is inlined before the model reads the skill. Montagent already answers `--version` with `montagent 0.1.0`.
7. **Instructional check, portable to any client.** The skill body tells the agent to run `montagent --version`, or to read `serverInfo.version` from the MCP `initialize` result, and compare it with a range the skill states, for example in `compatibility` or `metadata`. No client enforces this. It works only as well as the agent follows instructions.

No client documented here has a machine-readable "requires tool X at version Y" field for skills. The closest is Codex's `agents/openai.yaml` `dependencies`, which names an MCP server but gives no version. The spec's `compatibility` field is free text.

## Product-gap flags (things that would require changes to the binary or the MCP server)

- **Serving skills over MCP (SEP-2640)** would require the server to declare `capabilities.extensions["io.modelcontextprotocol/skills"]`, to implement the `skills/list` and `skills/get` methods (with `resources/directory/read` optional), to serve `skill://…` resources, and to compute SHA-256 digests and sizes. `rmcp` 3.4 has none of this. The upstream PR (#1286) is open. Montagent's `resources::all()` would grow, and ADR-0011's "two resources" framing would change.
- **A binary-emitted skill** (a Claude Code `command` source, or a one-shot writer for `.agents/skills/`) needs a new CLI subcommand, and the skill files would need to be embedded in the binary, for example with `include_str!`/`include_dir`. This adds to the public CLI surface.
- **A prompts-based route** would need the server to declare the `prompts` capability, which it does not do today.
- **The pure file-based routes need no binary change:** `npx skills` from this repo, a Claude Code marketplace in this repo, an Agent Plugins package, or a Gemini extension. They need a `skills/` directory, plus a manifest per ecosystem, in the repo or in a sibling repo.

## Open uncertainties

1. **Claude Code and MCP-served skills.** The changelog implies MCP servers can "list skills" (2.1.282–2.1.284), but the public docs are silent, and the SEP says the Claude Code prototype is "not yet public". **A live test is needed.** Connect a SEP-2640 server, such as the Python SDK reference, and see whether Claude Code surfaces its skills.
2. **Does Claude Code load an Agent Plugins layout** (root `plugin.json`, `mcp.json` without the leading dot)? Claude Code docs say the manifest is optional and `skills/` loads by default. They document `.mcp.json` but not `mcp.json`, so a single dual-format package may load its skills in Claude Code while silently losing the MCP wiring. Not tested.
3. **Whether `rmcp` 3.x lets a server answer arbitrary custom methods** (`skills/list`, `skills/get`) without the upstream PR. Not verified.
4. **Codex CLI (as opposed to the ChatGPT app) and the Skills extension.** Only a fork prototype is known. The OpenAI docs describe import at plugin submission time, not runtime reads.
5. **Whether an `npx skills`-installed skill keeps `.claude/skills` as a symlink on Windows.** The README offers `--copy` "when symlinks aren't supported". This is relevant because two of Montagent's six targets are Windows.
6. **Auto-update defaults outside Claude Code.** Claude Code defaults third-party marketplaces to no auto-update. The Codex, Cursor and Gemini defaults for third-party sources were not pinned down here, beyond Gemini's opt-in `--auto-update`.
7. **The pace of change.** SEP-2640 went Final 16 days before this research, and Agent Plugins 1.0 went public 54 days before it. Both support tables should be rechecked before #448 commits.
