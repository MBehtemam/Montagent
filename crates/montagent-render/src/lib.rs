//! Montagent's rasterizer and encode path.
//!
//! The crate exists now because ADR-0011 places the four-crate split at the bottom of
//! the tree and #188 builds that spine; **it is empty of behaviour and later tickets
//! fill it.** What it will own, and the constraints it is born under:
//!
//! - Rasterization through `skia-safe`, with the text stack beside it rather than inside
//!   it (ADR-0010) — CI fails if a prebuilt miss triggers a Skia source build (#36).
//! - An `ffmpeg` Montagent **spawns** rather than bundles, so installing Montagent makes
//!   nobody a distributor of GPL software; a missing or unusable one is exit 70 naming
//!   the resolved path (spec #168, stories 80 and 82).
//! - Writing the deliverable to a temp path and renaming atomically, so a truncated MP4
//!   never reads as finished (ADR-0011).
//! - Proxy degradation that applies to `preview` and never to `render`, with the two
//!   floors ADR-0067 separates: a 540p wall-clock give-up point and a 360p legibility
//!   refusal.
//!
//! **It knows nothing about the document, and that is the direction of the dependency.**
//! Spec #168 puts *"the twelve verb entry points"* in `montagent-core` and makes seam 1 —
//! the core verb API — the seam the rasterizer and the FFmpeg subprocess are exercised
//! *through*. So `montagent-core` depends on this crate, not the other way round, and the
//! rule that `render` *"refuses on `error` by running the identical checks `validate`
//! runs"* (ADR-0006) is enforced in the verb, which is where the checks already are.
//! Nothing is reimplemented on either side of that line: this crate takes numbers that are
//! already resolved and paints them.
//!
//! It is tested at spec #168's **seam 2**: frames compared by SSIM against a stated
//! threshold, never byte equality. Which frames can falsify what is the whole of the
//! distinction, and #213 built the two halves as two files with two names — a *golden*
//! frame Montagent rendered itself is self-confirming and can only catch a regression
//! (`montagent-core/tests/golden_frames.rs`), while only the frames extracted from the
//! published video can say the format is wrong
//! (`montagent-core/tests/reference_frames.rs`).
//!
//! What is live today, and what is a later ticket:
//!
//! - [`canvas`] and [`decode`] are #212's — the rasterizer, the transform, `rect` and
//!   `ellipse`, the image resample and the video seek — plus
//!   [#213](https://github.com/MBehtemam/Montagent/issues/213)'s [`canvas::Canvas::text`],
//!   which fills glyph **outlines**: this crate still links no font crate and knows no
//!   string, because ADR-0010 puts the text stack beside it rather than inside it.
//! - The ordered effect vocabulary and the four colour scalars are
//!   [#214](https://github.com/MBehtemam/Montagent/issues/214)'s
//!   [`canvas::Effect`], applied in element space in the order the list is written.
//!   `crossfade` and a run's `highlight` window landed with the same ticket but are
//!   *resolutions* rather than paint rules, so they live in `montagent-core` and reach
//!   this crate as an `opacity` and a paint like any other.
//! - The encode path to a deliverable and the atomic rename are
//!   [#215](https://github.com/MBehtemam/Montagent/issues/215)'s [`encode`]: raw frames
//!   piped to a spawned `ffmpeg`, written to a sibling temp path and renamed into place.
//! - The proxy ladder and its two floors are
//!   [#218](https://github.com/MBehtemam/Montagent/issues/218)'s [`proxy`]: the arithmetic of
//!   a rung and the refusal text, with `montagent-core`'s `preview` verb walking it.
//! - The `ffmpeg` floor and the arguments that decide it are
//!   [#479](https://github.com/MBehtemam/Montagent/issues/479)'s [`floor`] (ADR-0115): defined
//!   once, built from by [`decode`] and [`encode`], and exercised by the tool qualification.

pub mod budget;
pub mod canvas;
pub mod decode;
pub mod encode;
pub mod floor;
pub mod prof;
pub mod proxy;
