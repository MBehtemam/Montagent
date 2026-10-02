# Change request on the phone demo: show the error screen

**About 7 seconds · 1920×1080 · 25 fps · 16:9.** Deliver one MP4. It has no sound.

This directory already holds a finished 6-second product shot. `phone.montagent.json` is its
project: a phone mock-up that steps through four app screenshots from `screens/` (home, a tap
on the button, loading, done) while the camera slowly pushes in on the phone.

## The change

The client wants the demo to show what happens when the request fails. Insert
`screens/error.png` **between loading and done**, on screen for **1.2 seconds**. Done follows
the error screen and stays on screen exactly as long as it does now, so the video runs
1.2 seconds longer. The push-in keeps its pace and runs on through the extra time, with no
jump. Nothing else should change.
