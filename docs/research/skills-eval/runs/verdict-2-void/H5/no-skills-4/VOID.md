# Void: an API error (session limit) stopped this run

The transcript's `result` event has `is_error: true` and `terminal_reason: "api_error"`:
"You've hit your session limit · resets 4:40pm (Europe/Copenhagen)", after 50 turns. It left
a video, but `RUBRIC-v2.md` treats an API error that stops a run as an isolation problem, so
the run is voided and repeated as `runs/verdict-2/H5/no-skills-4` before pairing. The batch
stopped at this run, so no other run was affected.
