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
//! - [`stack`] resolves an anchor into an integer layer, in one hop — the one place draw
//!   order is computed, for the checks that report on it and the render that paints it.
//! - [`finding`] is the object every verb answers with — *"an error is a finding."*
//! - [`report`] collects them, summarises them, and derives the exit code.
//! - [`text`] generates the prose form **from the canonical JSON and nothing else**.
//! - [`wire`] is the one place a report becomes bytes, in one form per invocation.
//!
//! Four more sit beside that path rather than on it, and all four exist because the format
//! is published as well as parsed:
//!
//! - [`schema`] is the JSON Schema, generated from [`model`] — which is what makes the
//!   types the single place canonical key order can go stale (ADR-0041).
//! - [`layout`] reads canonical key order back out of that schema, as the one predicate
//!   `fmt` and `validate`'s `LAYOUT` check share rather than each reimplementing.
//! - [`resources`] is what the MCP surface publishes for discovery (ADR-0011): that same
//!   generated schema, and the format docs carrying the rules a schema cannot express.
//! - [`write`] is the canonical writing convention, the one place a project becomes bytes,
//!   and the atomic whole-file write every write tool reuses.

pub mod checks;
pub mod finding;
pub mod layout;
pub mod media;
pub mod model;
pub mod parse;
pub mod permissive;
pub mod registry;
pub mod report;
pub mod resources;
pub mod schema;
pub mod stack;
pub mod text;
pub mod verbs;
pub mod wire;
pub mod write;

pub use verbs::validate::validate;
pub use wire::Wire;
