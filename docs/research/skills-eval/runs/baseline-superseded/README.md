# Superseded baseline runs

The first nine Montagent-without-skills baseline runs, kept as evidence and replaced in
`../baseline/`. They were pinned to commit `92a45d16`, which predates the fix in
[Move to the ffmpeg 7.1+ argument forms and ship the tool qualification, and write the ADR](https://github.com/MBehtemam/Montagent/issues/479).
The eval machine has ffmpeg 9, so every run with sound (all of A and B) failed its first
render with `E-INTERNAL … Unrecognized option 'filter_complex_script'`, and lost calls
working around it. That cost belongs to a fixed product bug, not to missing skills, so the
runs were repeated on a build that has the fix. The reference runs do not use Montagent and
were kept.

`pair.py` and `verdict.py` read only `runs/<phase>/`, so nothing here is paired or tallied.
