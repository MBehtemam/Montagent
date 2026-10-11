---
status: accepted
amends: 0192 (§3's table gains one derived field, `wall_clock_gap_ms`; the stale rule, the `state` values and every other field are unchanged)
---

# The progress file says how much wall clock its elapsed time missed

[#930](https://github.com/MBehtemam/Montagent/issues/930), from the US-7 verdict in
[#921](https://github.com/MBehtemam/Montagent/issues/921) (map
[#913](https://github.com/MBehtemam/Montagent/issues/913)). A 49-minute production on v0.2.0
was left running while the laptop slept. When it woke, the file read `elapsed_s` 1,250 against
a `started_at` nine hours old. Both were true. `elapsed_s` is monotonic `Instant` time, which
stops while a macOS machine sleeps. `updated_at` and `started_at` are wall clock, which does
not. The agent had two honest numbers that disagreed and no rule for what the disagreement
meant. [ADR-0192](0192-a-render-writes-a-progress-file-and-says-its-phase-and-eta-or-why-it-has-none.md)
§1 says Montagent states the status and the agent infers nothing, so the file should say it.

## Decision

`render`'s progress file gains one field, in every record (`running` and terminal):

| field | meaning |
| --- | --- |
| `wall_clock_gap_ms` | about `(updated_at − started_at) − elapsed_s`, in whole milliseconds, never negative |

"About": it is read from the same two clocks at the moment of the write, unrounded, so a reader's own subtraction of the file's rounded `elapsed_s` and millisecond timestamps can differ by up to ~50 ms. A wall clock stepped backwards (NTP, a manual change) reads 0, and one stepped forwards reads as a gap: the field says the two clocks disagree, which is all it can know. It is derived from the two clocks the file already carries, at the moment of the write. It
needs no signal handling, no sleep notification and no new thread. `0` (or a few ms of
read-order noise) means the run's elapsed time covers its wall-clock span. A large value means
the wall clock moved on while the run's own clock did not: the machine slept. The ETA is not
adjusted. Its window is read off the same monotonic clock, so a sleep does not corrupt the
rate, and the file is not asked to guess when the run would have finished.

## Considered and not chosen

- **A `paused` state.** A suspended process cannot report itself. Nothing is running to write
  `paused`, and the first write after resume would already be `running` again. The gap field
  says it after the resume, which is the only time anything can. The `state` set is closed
  ([ADR-0192](0192-a-render-writes-a-progress-file-and-says-its-phase-and-eta-or-why-it-has-none.md)
  §3) and stays closed.
- **A signal handler or power-notification hook.** Platform-specific and hard to test, for a
  fact the two clocks already give.
- **A smoothed whole-run ETA.** Rejected in #921 under ADR-0072; the question belongs to
  `render --estimate` (#873).

## Consequences

- The field is not a stall detector. A process stopped with `SIGSTOP` keeps `Instant`
  advancing (only a system sleep stops it), so its gap stays near 0. A stopped or dead
  process is read from §5, an `updated_at` that does not change across polls. The gap says
  *the machine slept and woke*; staleness says *nothing is writing*. A reader that sees a large
  gap on a `running` file whose `updated_at` is changing knows the run survived a sleep and is
  alive. A reader that sees a large gap on a file whose `updated_at` is frozen still applies §5.
- #921 asked for a test that suspends the process with `SIGSTOP`/`SIGCONT`. That test is not written, on purpose: under `SIGSTOP` both clocks advance, so it could only assert a gap of 0, and the premise that the gap "says it after resume" holds for a system sleep, not for `SIGSTOP`.
- Tested by the pure function with injected clocks, since a test cannot put the machine to
  sleep, and by a real run's record carrying a small gap.
- `--progress-file`, the MCP `render` description and `SKILL.md` list the field.
