//! Montaget's text stack: shaping, line partitioning, break opportunities and extents.
//!
//! ADR-0011 places the four-crate split at the bottom of the tree and #188 built that
//! spine; #205 fills this crate with the engine `measure` is. What it owns, and what it
//! must never own:
//!
//! - **Shaping and per-line metrics in the fonts the project declares**, opening nothing
//!   outside the declared chain (ADR-0007). See [`fonts`] — the discipline is structural,
//!   because `fontique`'s system-font discovery is not compiled into this binary.
//! - **The line partition**, on UAX #14's mandatory breaks rather than `split('\n')`
//!   (ADR-0008). See [`lines`].
//! - **Break opportunities**, with the segmenter and its data version named (ADR-0008) —
//!   Montaget never places a line break itself. See [`breaks`].
//! - **Each line's resolved `baseline_y`** (ADR-0029), and the **stroked** extent rather
//!   than the typographic one (ADR-0014). See [`engine`].
//! - **What a font file says about itself** — its family and PostScript names and its
//!   licence strings, per face — for `fonts list` and the gate in `fonts vendor`
//!   (ADR-0057). See [`names`]. Read, never judged: which names are blocklisted is a rule,
//!   and rules live in the core.
//! - **Block arithmetic in exact integer tenths, never IEEE double** (ADR-0028) — as far
//!   as the tenths themselves: the `ceil` that turns them into the integer a `height`
//!   field takes is `montaget_core::exact`'s, so the formula has one implementation
//!   rather than two that can disagree.
//!
//! It is tested through `measure` rather than directly, since its output *is* `measure`'s
//! output (spec #168, *"Modules under test"*). And it **derives**: ADR-0024 is explicit
//! that `measure` writes the fit repair and never the verdict, so no judgment about a
//! document belongs in this crate — that authority is `validate`'s alone. Nothing here
//! ever sees a declared `width` or `height`, which is the structural form of that rule.

pub mod breaks;
pub mod engine;
pub mod fonts;
pub mod lines;
pub mod names;

pub use breaks::{SEGMENTER, Segmenter};
pub use engine::{Extent, MeasuredLine, Measurement, Run, Spec, VerticalOrigin, measure};
pub use fonts::{FontError, FontFile, Fonts};
pub use names::FaceNames;
