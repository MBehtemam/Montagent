//! Montaget's text stack.
//!
//! The crate exists now because ADR-0011 places the four-crate split at the bottom of
//! the tree and #188 builds that spine; **it is empty of behaviour and later tickets
//! fill it.** What it will own, and what it must never own:
//!
//! - Shaping and per-line metrics in the fonts the project declares, opening nothing
//!   outside the declared chain (ADR-0007).
//! - Break opportunities, with the segmenter and its data version named (ADR-0008) —
//!   Montaget never places a line break itself.
//! - Each line's resolved `baseline_y` (ADR-0029), and the **stroked** extent rather
//!   than the typographic one (ADR-0014).
//! - Block arithmetic in exact integer tenths, never IEEE double (ADR-0028).
//!
//! It is tested through `measure` rather than directly, since its output *is*
//! `measure`'s output (spec #168, "Modules under test"). And it derives: ADR-0024 is
//! explicit that `measure` writes the fit repair and never the verdict, so no judgment
//! about a document belongs in this crate — that authority is `validate`'s alone.
