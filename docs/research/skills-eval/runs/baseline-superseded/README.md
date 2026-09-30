# Superseded baseline runs

The first nine Montagent-without-skills baseline runs, kept as evidence and replaced in
`../baseline/`. They were pinned to commit `92a45d16`, which predates the fix in
[Move to the ffmpeg 7.1+ argument forms and ship the tool qualification, and write the ADR](https://github.com/MBehtemam/Montagent/issues/479).
The eval machine has ffmpeg 9, so every run with sound (all of A and B) failed its first
render with `E-INTERNAL … Unrecognized option 'filter_complex_script'`, and lost calls
working around it. That cost belongs to a fixed product bug, not to missing skills, so the
runs were repeated on a build that has the fix. The reference runs do not use Montagent and
were kept.

`B-talking-head/no-skills-2-shared-tmp` is from the repeated batch. Claude Code's sandbox
points every session's temp directory at one shared per-user directory, and the runs then
went three at a time: this run executed a `gen.py` that a concurrent logo-loop run had
written there, and abandoned its own generator. It was repeated after the harness made runs
wait for each other and sweep that directory after each run.

`pair.py` and `verdict.py` read only `runs/<phase>/`, so nothing here is paired or tallied.
