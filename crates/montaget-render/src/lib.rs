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
//! **It knows nothing about the document, and that is the direction of the dependency.**
//! Spec #168 puts *"the twelve verb entry points"* in `montaget-core` and makes seam 1 —
//! the core verb API — the seam the rasterizer and the FFmpeg subprocess are exercised
//! *through*. So `montaget-core` depends on this crate, not the other way round, and the
//! rule that `render` *"refuses on `error` by running the identical checks `validate`
//! runs"* (ADR-0006) is enforced in the verb, which is where the checks already are.
//! Nothing is reimplemented on either side of that line: this crate takes numbers that are
//! already resolved and paints them.
//!
//! It is tested at spec #168's **seam 2**: frames compared by SSIM against a stated
//! threshold, never byte equality. Which frames can falsify what is the whole of the
//! distinction, and #213 built the two halves as two files with two names — a *golden*
//! frame Montaget rendered itself is self-confirming and can only catch a regression
//! (`montaget-core/tests/golden_frames.rs`), while only the frames extracted from the
//! published video can say the format is wrong
//! (`montaget-core/tests/reference_frames.rs`).
//!
//! What is live today, and what is a later ticket:
//!
//! - [`canvas`] and [`decode`] are #212's — the rasterizer, the transform, `rect` and
//!   `ellipse`, the image resample and the video seek — plus
//!   [#213](https://github.com/MBehtemam/Montaget/issues/213)'s [`canvas::Canvas::text`],
//!   which fills glyph **outlines**: this crate still links no font crate and knows no
//!   string, because ADR-0010 puts the text stack beside it rather than inside it.
//! - The effect vocabulary, colour filters, transitions and highlight are
//!   [#214](https://github.com/MBehtemam/Montaget/issues/214).
//! - The encode path to a deliverable, the atomic rename and the proxy ladder are #215
//!   and #218.

pub mod budget;
pub mod canvas;
pub mod decode;
