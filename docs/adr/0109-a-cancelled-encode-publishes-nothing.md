---
status: accepted
amends: 0093 (ruling 6's walk-away gains a third trigger beside an `error` and a missed deadline: the caller cancelling. The invariant becomes *a file at the output path is a render with zero errors that nobody cancelled*), 0108 (§4's *"an aborted call still runs to completion and publishes"* no longer holds: a cancelled `render`/`preview` stops within one frame and releases the encode slot)
---

# A cancelled encode publishes nothing

[#440](https://github.com/MBehtemam/Montagent/issues/440), the second ticket out of
[#393](https://github.com/MBehtemam/Montagent/issues/393) (MONTAGENT-9). Part of
[#383](https://github.com/MBehtemam/Montagent/issues/383). Research:
`docs/research/mcp-render-timeout.md` §4.3, §5.

## What was there

When an MCP client cancels a call, `rmcp` trips the request's `CancellationToken` and drops the
response. It does not stop the handler. No Montagent tool read the token, so a cancelled
`render` **encoded to the end and published**.

The caller was told *failed* while the disk said *succeeded*. That is the MONTAGENT-9 trap: the
source doc's files *"turned out complete and correct"* after both aborts, and nothing but a
`stat` could have said so. Since ADR-0108, the orphan also held the one encode slot, so every
`render` or `preview` behind it waited for work nobody was waiting for. Measured on the commit
before this ADR, a render queued behind a cancelled 1500-frame encode started **26.8 s** after
the cancel.

## Decision

### 1. What *"the caller was told it failed"* means for the disk: nothing changed

A cancelled `render` or `preview` **stops before its next frame and walks away**, using
ADR-0093 ruling 6's own mechanism. The sealed or part-written temp file goes with the encoder,
and nothing is written at any path. A deliverable already at `output` stays byte-identical.

The ticket was right that this is not an obvious default, because it trades finished work for
agreement between the verdict and the file. Four alternatives were weighed:

- **Publish anyway (the status quo).** It keeps the work, and it is the defect. The one party
  that asked for the file has been told it does not exist. Only a later `stat` or `verify`
  ([#436](https://github.com/MBehtemam/Montagent/issues/436)) would reveal it, and if the edit
  before the render was the mistake, nobody asked for it.
- **Finish the encode but keep it unpublished.** This spends the whole wall clock for a file no
  verb will ever promote, holds the slot for the full duration, and litters `out/` with temp
  files that read as half-finished renders.
- **Finish and publish somewhere else.** This invents an output path nobody declared, which
  ADR-0011 and ADR-0104 both exist to prevent.
- **Walk away (chosen).** The verdict and the disk agree: *failed* means *nothing was written*.
  The cost is lost work. That cost is smaller than it looks: since ADR-0108, idle aborts should
  no longer happen, so the cancels left are mostly a person pressing stop in `/tasks`, who
  wanted the work stopped. And the lost work is usually partial, since a cancel lands mid-encode.

`preview` follows the same rule, and a cancelled preview **does not descend the ladder**. A
cancel is not a missed budget, and there is nobody left to degrade for.

### 2. Where the stop is checked

The check is a flag, `render::Cancel`, that the core reads at four points:

1. after pre-flight, **before the encoder is spawned**, so a call cancelled while its checks
   ran starts no subprocess;
2. **before every frame**, which is what frees the server *"within one frame's time"*;
3. **after the seal**, so a request that arrives while `ffmpeg` finishes the file still wins;
4. **immediately before the rename that publishes.**

**A cancel after point 4 is too late, and that is correct rather than a race to close.** The
render has succeeded and the file at `output` is the zero-error render ADR-0093 promised. The
response is dropped, so the caller does not hear this, and `verify` (#436) is the after-the-fact
answer. The window between point 4 and the rename is one `rename(2)`.

A call cancelled **while queued** for the encode slot simply leaves the queue. Nothing ran, so
there is nothing to withhold.

The MCP adapter sets the flag the moment `rmcp` cancels the request, then **keeps waiting for
the work to actually stop** before it releases the slot. Releasing it early would let the next
encode start alongside a still-running one, which undoes ADR-0108 §3. The flag is also set if
the adapter's own future is ever dropped, so no path leaves an encode running for nobody. When
the session ends, `rmcp` cancels every child token, so a client that disconnects mid-render gets
the same withholding.

The flag is a plain `Arc<AtomicBool>` rather than `tokio_util`'s token, because the core is
synchronous and has no async runtime to depend on. The CLI never sets it: a CLI user stops a
render with a signal, and the process goes with it.

### 3. `E-CANCELLED`

The verb's report says `E-CANCELLED`, class `error` with `NotAboutDocument` repair. The project
did not stop the run, so the finding must not read as a fact about the project, on ADR-0091's
reasoning for `E-TOOL-MISSING`. It is a code of its own rather than `E-INTERNAL`, because "you
stopped it" and "it crashed" call for different next moves. Its sentence states what the disk
holds, since that is the whole of what this ADR decides:

> `render` was cancelled by the caller after 250 of 1500 frames, and published nothing:
> whatever was at the output path before this call is still there, untouched

The report is **usually unread**, because `rmcp` drops a cancelled request's response and the
spec tells the client to ignore a late one. The code exists so the verb's answer is true on
every path, and so tests can name the condition. Its exit code is ADR-0011's 70, which a
cancelled CLI run cannot produce.

## What this does not settle

**Whether Claude Code sends `notifications/cancelled` when its idle timer fires** is still
undocumented. If it does not, an idle-aborted render still completes and publishes, exactly as
before. ADR-0108's stderr record now makes that visible: a `dispatched` line with no `cancel
requested` before its `finished` line means no cancel arrived. After ADR-0108, idle aborts
should not happen at all, which is why this is recorded rather than worked around.

## Evidence

- `crates/montagent-core/tests/cancel.rs`, the core's promise about the disk:
  - cancelled at frame 0, a render stops there, publishes nothing, leaves no temp file, and
    leaves an earlier, **different** deliverable byte-identical;
  - cancelled before it starts, it spawns no encoder;
  - an unset flag changes nothing;
  - a cancelled preview neither publishes nor tries another rung.
- `crates/montagent/tests/mcp_concurrency.rs`: a stdio harness sends `notifications/cancelled`
  mid-render of a 1500-frame project. The deliverable already at the path is untouched, `out/`
  holds no temp file, and a `render` queued behind it completes within the bound. **Against
  the commit before this ADR it waited 26.8 s.**
- `docs/adr/mcp_cancel_withholds_check.sh` asserts the same over the built binary.
