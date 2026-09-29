---
status: accepted
---

# A missing `ffmpeg`/`ffprobe` is `E-TOOL-MISSING`, not `E-INTERNAL`

> **Amended by [ADR-0115](./0115-ffmpeg-7-1-with-libx264-is-the-floor-and-a-tool-qualification-finds-out.md)**: `Missing`'s one dispatch point now chooses among **three**
> codes. A found `ffmpeg` that fails the tool qualification is **`E-TOOL-UNSUPPORTED`**, this
> code's sibling (same exit 70, same `NotAboutDocument`), because the remedy is to upgrade rather
> than to install. The painter's and encode loop's tool channel carry the `Missing` whole, so a
> found binary is no longer named `E-TOOL-MISSING` there.

**Ticket:** [#368](https://github.com/MBehtemam/Montagent/issues/368), part of
[#356](https://github.com/MBehtemam/Montagent/issues/356)'s stranger walk
([#362](https://github.com/MBehtemam/Montagent/issues/362)).

## The gap

ADR-0009 makes Montagent's shape deliberate: *"a binary, plus an `ffmpeg` the user
supplies."* A first-time user who has not installed `ffmpeg` has an unconfigured
environment — the documented, expected state of a machine that has never run Montagent
before. Rendering with no `ffmpeg` on `PATH` printed:

```
error  E-INTERNAL
  Montagent failed internally: ffmpeg was not found on PATH: no executable of that
  name in any PATH entry. Looked in: /usr/bin, /bin
```

The exit code (70) is exactly ADR-0011's table entry for this condition and is not in
question. The **finding code** said the opposite of ADR-0009: `E-INTERNAL` reads as
"Montagent broke," and an agent reading it has no reason to surface an actionable
"install `ffmpeg`" to the human it works for. This is the single most likely first-run
failure there is, so the mis-signalling lands on day one, for every stranger.

`crates/montagent-core/src/media/tools.rs` already carries the fact needed to tell the
two conditions apart: `Missing::resolved` is `None` when the program was never found on
`PATH` at all, and `Some(path)` when a path resolved and then would not run (wrong
architecture, corrupt download, a permissions error). Only the machinery was shared; nothing
downstream read the distinction.

## Decision

**A new code, `E-TOOL-MISSING`, for `Missing::resolved.is_none()`** — the program is not
on `PATH` at all. `E-INTERNAL` stays for `Missing::resolved.is_some()` — a path resolved
and the OS still could not run it, which reads as a genuine break rather than an absent
install.

Both stay:

- **Exit 70, unchanged.** ADR-0011's table entry is *"internal failure … → retry or
  report"*; nothing about the *code* the table names changes, and re-litigating the exit
  contract is out of this ticket's scope.
- **`RepairClass::NotAboutDocument` (ADR-0073).** The subject is the invocation
  environment, not a project file — the same fault line ADR-0073 already drew for
  `E-INTERNAL`. `E-TOOL-MISSING`'s fix is fully determined (install the named program),
  which would make it look like `Advise` on ADR-0043's own test, but `Advise`'s repair is
  a structured, applyable edit *to the document*, and there is no document here to edit —
  so it takes the same exemption `E-INTERNAL` does, its remedy stated as message text
  (`{reason}`, from `Missing::reason()`) rather than a `repair` value.

`Missing` gains the dispatch, in one place: `Missing::into_report()` (the constructor
`probe` already uses) and a new `Missing::fail(&mut Report)` (for `render`, `preview` and
`validate`'s probe half, which have already established other facts and must keep them)
both pick the code from `resolved`. No call site decides it itself.

## Why here, not a document-shaped answer

Reusing `E-INTERNAL` and just improving its message text was considered and rejected: an
agent (or `compare`) that wants to tell "you don't have this installed" apart from "it
crashed mid-run" needs a stable code to gate on, per ADR-0006's *"validate reports
facts"* — a fact buried in free-text `reason` is not a fact the wire format makes
checkable. A single `E-TOOL-MISSING` covering both `ffmpeg` and `ffprobe`, naming the
program in its message, was chosen over one code per program: the two tools are resolved
together (`tools::resolve`) and the actionable answer is identical either way — install
it — so a second code would be a second thing to keep in step for no distinction a
consumer needs.

## Scope

This ticket fixes the three call sites where a `Missing` cleanly becomes a whole report
or a report-in-progress's terminal finding: `probe` (via `into_report`), `render` and
`preview`, and `validate`'s probe half (`run_checks`). It does not touch `frame.rs`'s
`ffmpeg()` or `keyed.rs`'s `coverage`, which already collapse `Missing` into a bare
`String` shared with unrelated document-level failures before it reaches any `Report` —
in `measure`'s keyed path that string currently surfaces as `E-INVOCATION`/exit 3, not
even `E-INTERNAL`. That is a wrong *exit code*, not just a wrong class, and is a deeper,
separate defect in those two call sites' error plumbing. Recorded rather than folded in
here; see the map's decision log for the follow-up ticket.

## Consequences

- New registry entry: `E-TOOL-MISSING`, `Error`, `RepairClass::NotAboutDocument`,
  `ThresholdProvenance::Internal`, template `"{reason}"`.
- `E-INTERNAL`'s registry comment is narrowed: it no longer covers every missing-tool
  case, only a resolved-and-unrunnable one.
- `Report::tool_missing` / `Report::fail_tool_missing` join `Report::internal_failure` /
  `Report::fail_internally` as the pair of constructors `Missing` chooses between.
- `probe`, `render`, `preview` and `validate` all report a never-installed `ffmpeg` or
  `ffprobe` as `E-TOOL-MISSING`; a resolved-but-broken one still reports `E-INTERNAL`.
