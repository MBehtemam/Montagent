---
status: accepted
amends: 0011 (remotely, a tool's `project` is a path inside a workspace, `render`'s result also carries a download URL, and a plain-HTTP byte door stands beside MCP; the MCP verbs are unchanged), 0006 (remotely, `validate` also reports each arrived file's sha256 and the total bytes still to upload, and it starts URL downloads, the one side effect `validate` has; over stdio it has none), 0131 (remotely, a URL source renders from the instance's download cache; over stdio it is still refused), 0053 (remotely, an absolute path or a `..` that leaves the workspace is refused)
---

# A remote instance moves bytes through a plain-HTTP door into named workspaces

[#660](https://github.com/MBehtemam/Montagent/issues/660), on the map
[#656](https://github.com/MBehtemam/Montagent/issues/656), which is finding its way to a spec
for running Montagent as a remote MCP server (Streamable HTTP) that anyone can self-host.

Locally the agent edits the project with its own file tools, and the tools read the same disk
([ADR-0011](0011-tool-surface-reads-checks-renders.md)). A remote agent shares no disk with
the instance. The throwaway prototype
([#659](https://github.com/MBehtemam/Montagent/issues/659#issuecomment-5995125476)) showed
that project JSON goes up fine as text, but media, fonts and the mp4 needed a raw HTTP side
channel. It also showed that each HTTP request costs 1–3 s, that an MCP session ends after
300 s with no messages, and that n8n opens a fresh session on every run.

Settled by three `/court` rounds of three jurors each (Opus, Sonnet, Fable), unanimous on every
question, with the owner ruling with the Judge's read each time. A fourth round, on who the
design is for, was unanimous on where to say so and split on whether bucket sources belong in
this decision. The owner again ruled with the Judge.

## The decision

Everything below applies to a **remote instance only**. Local stdio and the CLI are unchanged:
no doors, no workspaces, no downloads, no tokens.

### 1. The project's authority stays with the agent

The project lives where the agent already keeps it: its disk and git, or n8n's workflow
state. The agent edits it with its own tools, as ADR-0011 has it. The instance's copy is a
disposable working copy that nobody reads back as the truth. There is still no CRUD API.

### 2. Two doors, one guard

- **`/mcp`** is the only *control* surface: the same MCP tools as stdio, over Streamable HTTP.
  Tool calls pass **paths**, never the document inline. A tool argument is written out by the
  model token by token, so an inline project would cost its whole length in tokens on every call.
- **`/files/*`** is the **byte door**: plain HTTP with no verbs beyond moving bytes.
  - `PUT` one file, which is how the project JSON and small top-ups go up.
  - `PUT` an **archive chunk**, unpacked in place. Each file is written under a temporary name
    and renamed into place, so a cut-off chunk never leaves a truncated file that looks
    present. The instance advertises its maximum upload size, and the agent chooses a chunk
    size up to it.
  - `GET` a file, which is how the mp4 comes back.

The map's rule "MCP is the only network surface" is restated as **MCP is the only control
surface; bytes move over plain HTTP**. Both doors sit behind the same auth, which
[#661](https://github.com/MBehtemam/Montagent/issues/661) decides. Base64 inside tool calls is
refused for media and fonts alike: two ways to fill one `source` field is worse than one.

### 3. Named workspaces

Files live in a **workspace**, a named folder the agent chooses, such as `my-film/`. Every MCP
path is inside one. A workspace outlives sessions and runs. It does not outlive cleanup, or a
disk the host wipes, and an agent treats "workspace gone" as normal: `validate` lists what is
missing, and the agent uploads it. If tenancy goes multi-user, names are scoped per token.

### 4. Paths keep today's rule

The agent uploads its folder layout as-is. `source` and `output` resolve relative to the
project file's own folder ([ADR-0053](0053-asset-path-resolution-no-assetroot.md)). An
absolute path, or a `..` that leaves the workspace, is refused as a **named `validate`
finding** that quotes the path and says to make it relative. Absolute paths are never
rewritten silently.

### 5. Sync: upload what is missing or different

Remote `validate` already reports `E-SOURCE-MISSING` and `E-FONT-MISSING`. It also reports:

- the **sha256** of each file that has fully arrived. The hash is computed once per file,
  keyed on the same observed identity as the probe cache
  ([ADR-0092](0092-a-probe-is-matched-on-an-observed-identity-and-guarded-by-its-contents.md)),
  so a repeat `validate` never re-reads the files;
- the **total bytes still to upload**, as a number in the report, not a warning with a
  threshold.

The agent hashes its own files, compares them, and uploads only what is missing or different.
A file re-exported under the same name is caught; a file that only exists is not taken as
current.

### 6. URL sources render remotely, downloaded at `validate` time

This amends [ADR-0131](0131-render-and-frame-use-local-sources-only-and-validate-says-so.md)
for remote only. On a remote instance there is no "local" except the instance itself, so its
cache is the local copy.

- `validate` **starts** downloading URL sources in the background and returns straight away,
  with findings such as fetching 40% of 3.1 GB, or fetch failed: 403. Calling it again is how
  the agent checks progress. This is the one side effect `validate` has, and only remotely.
  [ADR-0006](0006-validate-reports-facts-and-render-enforces.md)'s `validate` reports facts and
  does nothing, and over stdio that stays true.
- The cache is keyed on the URL plus its ETag or content hash, so a changed file is never
  served stale.
- Each download has a size limit and a timeout, and the fetcher refuses private and internal
  addresses.
- `render` **refuses** while anything is still downloading. It never downloads inside its own
  call.
- Over stdio, `validate` still says plainly that a URL source does not render there.

### 7. The mp4 comes back as a URL

`output` lands in the workspace under the path rule above. `render`'s result carries **both**
the workspace path and a download URL. The URL is built from a **public base URL the
self-hoster configures**, not from the request's Host header. A relative `/files/…` link is
allowed when none is configured. Whether links are signed or expire is
[#661](https://github.com/MBehtemam/Montagent/issues/661)'s to decide.

## Who this is for (guidance, not a rule)

The protocol is not what makes 50 GB slow. The bytes go through the byte door, never through
MCP messages. **Distance** is what's slow: the bytes have to cross from wherever they are to
wherever Montagent runs, at least once. 50 GB takes about 1.4 h at 100 Mbit/s and about 5.5 h
at 20 Mbit/s up, and a datacenter instance pulls it from a nearby bucket in about 7 minutes.
So, usually, **run Montagent where the media is**:

| where the media is | where Montagent runs |
| --- | --- |
| on the agent's machine, a large library | locally, over stdio or the CLI. No bytes move |
| in the cloud | a remote instance near it |
| anywhere, but the agent has no disk | a remote instance |
| local, but the machine is too weak to render | a remote instance: upload once, render many times |

Compute matters too: 4 vCPU renders 3840×2160@60 at 0.34× realtime. So "where the media is"
can lose to "where the CPU is".

## Considered and not chosen

- **The project's authority on the instance**, edited through server-side write tools. That is
  the road back to the CRUD API ADR-0011 rejected, and it costs the agent its diff.
- **Shared object storage as where the authority lives.** It is not host-neutral, and it puts
  credentials on both sides just to move a few KB of JSON.
- **A working area per MCP session.** Sessions end after 300 s with no messages, and n8n opens
  one per run, so 1000 uploaded files would vanish between runs.
- **Silently rewriting absolute paths into the workspace.** The same JSON would mean two
  things, and the agent would have to mirror the rewrite to know where to `PUT`.
- **Downloading at `render` time.** That puts minutes of network inside the longest call, and
  a bad URL shows up late.
- **One giant archive.** A dropped connection at 90% of 23 GB loses all of it.
- **Content-addressed project sources.** That changes the project format for a remote-only
  concern.
- **Resumable uploads (tus).** Robust, but it puts offsets and state on a door meant to have
  none. Left open for single huge files.
- **Presigned URLs as the answer for media in a bucket.** They expire, and they would be
  written into the project file in git. Reading straight from a bucket is real, open work, but
  it brings credentials and differences between providers, so it is its own ticket on the map,
  [#753](https://github.com/MBehtemam/Montagent/issues/753), blocked by [#661](https://github.com/MBehtemam/Montagent/issues/661).

## Consequences

- The MCP verbs and the core are the code stdio runs. What is new is the transport, the byte
  door, workspaces, the URL fetcher, and three remote-only report fields: the sha256, the bytes
  to upload, and the download URL.
- `validate` is no longer side-effect free everywhere. A reader of ADR-0006 must know the
  remote exception, and its banner says so.
- A temporary disk makes a workspace live only until the container restarts. Retention,
  deletion, disk sizing and volumes are open on the map under *Persistence & cleanup*.
