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
//! - [`document`] is the lenient spine every check reads from.
//! - [`registry`] declares every check: its class, its refuse class, its threshold
//!   provenance and its prose template.
//! - [`finding`] is the object every verb answers with — *"an error is a finding."*
//! - [`report`] collects them, summarises them, and derives the exit code.
//! - [`text`] generates the prose form **from the canonical JSON and nothing else**.
//! - [`wire`] is the one place a report becomes bytes, in one form per invocation.

pub mod document;
pub mod finding;
pub mod parse;
pub mod registry;
pub mod report;
pub mod text;
pub mod verbs;
pub mod wire;

pub use verbs::validate::validate;
pub use wire::Wire;
