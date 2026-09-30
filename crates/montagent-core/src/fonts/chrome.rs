//! Montagent's own chrome face: the one face its labels are drawn in (#421, ADR-0098 §9).
//!
//! **Never the project's declared font**, because a project may declare a Thai-only face or
//! a display face with no digits while a label is mostly digits — and this repository's own
//! main fixture has declared a path that did not exist (ADR-0057). Chrome that fails on an
//! unusual document fails hardest on the projects most likely to be broken. **Never a
//! system font**, because ADR-0064 ships six targets and a fitted type size would mean a
//! different thing on each, so ADR-0098's 8 px floor would stop being one number.
//!
//! So the face is **in the binary**: `include_bytes!`, which makes its absence a build
//! failure rather than a fallback. That is the accident ADR-0098 names — #396's prototype
//! quietly drew its labels in whatever font happened to sit in the fixture directory.
//!
//! **JetBrains Mono NL, Regular, v2.304** (OFL-1.1; ADR-0122 records the choice). Three
//! properties chose it, and `tests/chrome_face.rs` asserts each against these bytes:
//!
//! - **Monospaced**, so its figures are tabular — a column of milliseconds lines up — and a
//!   label's width is its character count times [`ADVANCE`]. That makes ADR-0098's measured
//!   `295/chars` law exact for this face rather than ±9 %.
//! - **No ligatures** (the `NL` cut). The standard cut joins `->`, `--` and `==` into one
//!   glyph, which would both redraw a label's characters and make the unshaped
//!   [`metrics`] disagree with what the shaper lays out.
//! - **A visible `.notdef`** at the same pitch, so an id character outside its coverage is
//!   drawn as a box rather than silently omitted, and still fits the arithmetic.
//!
//! **Never visible to project text.** It is registered only in the registry [`fonts`]
//! returns, which holds nothing else; a project's text is measured and drawn in a registry
//! built from its own `fonts` table, which this face is never added to.

use montagent_text::{FaceMetrics, Fonts};

/// The face's file name under `crates/montagent-core/fonts/`, beside its `OFL.txt`.
pub const FILE: &str = "JetBrainsMonoNL-Regular.ttf";

/// The face itself. `include_bytes!`, so a checkout without the file does not build.
pub static BYTES: &[u8] = include_bytes!("../../fonts/JetBrainsMonoNL-Regular.ttf");

/// The key the chrome face is registered under in [`fonts`]'s registry — the `font` a
/// label's [`montagent_text::Spec`] names.
pub const KEY: &str = "montagent-chrome";

/// Font units per em.
pub const UNITS_PER_EM: u32 = 1000;

/// Every glyph's advance, in font units: the face is monospaced over printable ASCII and
/// its replacement glyph. A label of `n` characters at `size` px is `n × ADVANCE × size /
/// UNITS_PER_EM` px wide, exactly — the arithmetic label sizing is done in, before anything
/// is shaped.
pub const ADVANCE: u32 = 600;

/// The ascender, y-up, in font units.
pub const ASCENDER: i32 = 1020;

/// The descender, y-up (so negative), in font units.
pub const DESCENDER: i32 = -300;

/// A registry holding the chrome face under [`KEY`], and nothing else.
///
/// A fresh one per call, because a [`Fonts`] is shaped against mutably and the chrome is
/// drawn rarely; registering 200 KB from memory is not a cost worth sharing state over.
pub fn fonts() -> Fonts {
    let mut fonts = Fonts::new();
    fonts
        .register_bytes(KEY, BYTES, None)
        // Not a runtime condition: the bytes are fixed at build time and
        // `tests/chrome_face.rs` shapes in them, so a face that did not register would
        // fail the build's tests long before it reached a user.
        .expect("the embedded chrome face registers");
    fonts
}

/// The chrome face's own metrics, for arithmetic that runs before anything is shaped.
///
/// [`UNITS_PER_EM`], [`ADVANCE`], [`ASCENDER`] and [`DESCENDER`] are these numbers as
/// constants; this is the reader they are tested against, and the one to ask for a
/// character outside printable ASCII.
pub fn metrics() -> FaceMetrics<'static> {
    FaceMetrics::read(BYTES, None).expect("the embedded chrome face parses")
}
