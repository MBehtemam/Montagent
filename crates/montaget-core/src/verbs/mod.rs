//! The verbs, as in-process calls taking a path and a typed argument struct and handing
//! back a report as values.
//!
//! This is spec #168's **seam 1**, and the one to prefer for everything: it is the
//! highest seam that can reach the MCP-only behaviours the CLI's process boundary
//! structurally cannot. Both adapters are thin over this module by construction —
//! neither contains a check, a rule or an arithmetic decision (ADR-0011).

pub mod fmt;
pub mod probe;
pub mod validate;
