//! Montaget's rasterizer and encode path.
//!
//! The crate exists now because ADR-0011 places the four-crate split at the bottom of
//! the tree and #188 builds that spine; **it is empty of behaviour and later tickets
//! fill it.** What it will own, and the constraints it is born under:
//!
//! - Rasterization through `skia-safe`, with the text stack beside it rather than inside
//!   it (ADR-0010) — CI fails if a prebuilt miss triggers a Skia source build (#36).
//! - An `ffmpeg` Montaget **spawns** rather than bundles, so installing Montaget makes
//!   nobody a distributor of GPL software; a missing or unusable one is exit 70 naming
//!   the resolved path (spec #168, stories 80 and 82).
//! - Writing the deliverable to a temp path and renaming atomically, so a truncated MP4
//!   never reads as finished (ADR-0011).
//! - Proxy degradation that applies to `preview` and never to `render`, with the two
//!   floors ADR-0067 separates: a 540p wall-clock give-up point and a 360p legibility
//!   refusal.
//!
//! It refuses on `error` by running the identical checks `validate` runs (ADR-0006) —
//! which is why it depends on the core library rather than reimplementing a check, and
//! why a refuse-class finding's block is one it cannot lift (ADR-0043).
//!
//! It is tested at spec #168's **seam 2**: golden frames compared by SSIM against a
//! stated threshold, never byte equality. Note which frames can falsify what — a golden
//! frame Montaget rendered itself is self-confirming and can only catch regressions;
//! only `fixtures/en-halloween-decorating/reference/frame-*.png`, extracted from the
//! published video, can falsify the format.
