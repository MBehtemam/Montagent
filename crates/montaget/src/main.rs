//! The Montaget binary: one CLI and one MCP server over one core library.
//!
//! ADR-0011: *"Both. One binary, one core library. The MCP server wraps the library,
//! never the CLI"* — a subprocess per tool call would pay the startup cost ADR-0009 says
//! is paid once on the stdio binding.
//!
//! Both adapters here are thin **by construction**: neither contains a check, a rule or
//! an arithmetic decision. Each one turns its own argument shape into a
//! `montaget_core::verbs` call and prints what comes back. If a test ever needs an
//! adapter to reach a rule, the rule is in the wrong crate.

mod cli;
mod mcp;

fn main() -> std::process::ExitCode {
    cli::run(std::env::args_os())
}
