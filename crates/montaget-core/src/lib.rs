//! Montaget's core library — the one place a check, a rule or an arithmetic decision
//! lives.
//!
//! ADR-0011: *"One binary, one core library. The MCP server wraps the library, never the
//! CLI."* Everything an adapter can do, it does by calling a verb in [`verbs`] and
//! printing what comes back.
//!
//! The pieces, in the order a run touches them:
//!
//! - [`parse`] reads the bytes, or produces the one finding that means "fix the *file*".
//! - [`permissive`] is the lenient spine every check reads from, and [`model`] is the
//!   format's types for the checks that want them.
//! - [`registry`] declares every check: its class, its refuse class, its threshold
//!   provenance and its prose template.
//! - [`checks`] is where the checks themselves live, one module per question.
//! - [`media`] is the disk half of the question: `ffmpeg` resolution, `probe`, the
//!   ADR-0023 dimensions pipeline and the probe caches.
//! - [`finding`] is the object every verb answers with — *"an error is a finding."*
//! - [`report`] collects them, summarises them, and derives the exit code.
//! - [`text`] generates the prose form **from the canonical JSON and nothing else**.
//! - [`wire`] is the one place a report becomes bytes, in one form per invocation.
//!
//! Two more sit beside that path rather than on it, and both exist because the format is
//! published as well as parsed:
//!
//! - [`schema`] is the JSON Schema, generated from [`model`] — which is what makes the
//!   types the single place canonical key order can go stale (ADR-0041).
//! - [`write`] is the canonical writing convention, the one place a project becomes bytes.

pub mod checks;
pub mod finding;
pub mod media;
pub mod model;
pub mod parse;
pub mod permissive;
pub mod registry;
pub mod report;
pub mod schema;
pub mod text;
pub mod verbs;
pub mod wire;
pub mod write;

pub use verbs::validate::validate;
pub use wire::Wire;
