//! The checks themselves — one module per question asked of a whole project.
//!
//! A check reads [`crate::permissive::Loose`] and appends findings to a
//! [`crate::report::Report`]. It never decides its own repair class, its own template or
//! its own ADR provenance: those are declared once in [`crate::registry`], and a check
//! that wanted to vary any of them per instance is the failure ADR-0043 forbids.
//!
//! Everything here is called from [`crate::verbs::validate`], which runs every check on
//! the whole project every time — there is no fast mode and no way to narrow what is
//! analysed (ADR-0006).

pub mod retired;
