//! Montagent's text stack: shaping, line partitioning, break opportunities and extents.
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
//!   Montagent never places a line break itself. See [`breaks`].
//! - **Each line's resolved `baseline_y`** (ADR-0029), and the **stroked** extent rather
//!   than the typographic one (ADR-0014). See [`engine`].
//! - **Where each line's ink actually is**, and the seam between adjacent lines' ink
//!   (ADR-0087) — the measurement ADR-0007's declared-numbers slot rule cannot see. See
//!   [`ink`]. A measurement only: whether a seam is acceptable is `validate`'s.
//! - **Where every glyph goes**, as outlines in the block's own coordinates, off the same
//!   shaping pass the measurement came from — never a second layout (#213). See [`place`].
//! - **What a font file can draw** — the `cmap` of one face, so ADR-0007's glyph-coverage
//!   `error` is asked of the file rather than guessed from a layout. See [`glyphs`].
//! - **One face's metrics as the file stores them** — its em, vertical metrics and unshaped
//!   advances — for arithmetic that runs before anything is shaped, such as `frame`'s
//!   label sizing over Montagent's chrome face (#421). See [`metrics`].
//! - **What a font file says about itself** — its family and PostScript names and its
//!   licence strings, per face — for `fonts list` and the gate in `fonts vendor`
//!   (ADR-0057). See [`names`]. Read, never judged: which names are blocklisted is a rule,
//!   and rules live in the core.
//! - **Block arithmetic in exact integer tenths, never IEEE double** (ADR-0028) — as far
//!   as the tenths themselves: the `ceil` that turns them into the integer a `height`
//!   field takes is `montagent_core::exact`'s, so the formula has one implementation
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
pub mod glyphs;
pub mod ink;
pub mod lines;
pub mod metrics;
pub mod names;
pub mod place;
pub mod sfnt;
pub mod spacing;

pub use breaks::{SEGMENTER, Segmenter};
pub use engine::{Dir, Extent, MeasuredLine, Measurement, Run, Spec, VerticalOrigin, measure};
pub use fonts::{FontError, FontFile, Fonts};
pub use glyphs::Charmap;
pub use ink::InkSeam;
pub use metrics::FaceMetrics;
pub use names::FaceNames;
pub use place::{Align, Glyph, PathEl, Placement, place};
pub use spacing::{Suppressed, first_suppressed};
