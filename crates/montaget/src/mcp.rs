//! The MCP adapter.
//!
//! Thin over `montaget_core`, exactly as the CLI is. ADR-0009 fixes the SDK (`rmcp`) and
//! records that the stdio binding pays startup once per session, not per tool call.
//!
//! The asymmetry between this surface and the CLI's is deliberate (ADR-0011): every MCP
//! tool schema occupies the agent's context and degrades tool selection on *every* turn,
//! including turns with nothing to do with video, while a CLI subcommand costs nothing
//! until invoked. So tools are added here only when an agent needs them in the loop.

use std::path::PathBuf;

use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig};
use rmcp::transport::stdio;
use rmcp::{ErrorData, ServerHandler, ServiceExt, tool, tool_handler, tool_router};

use montaget_core::Wire;
use montaget_core::report::Report;

/// The advertised schema **is** the enforced one: `schemars` derives the document the
/// tool publishes from this type, and this same type is what the arguments are
/// deserialised into below. There is one declaration, so the two cannot drift.
fn validate_schema() -> std::sync::Arc<serde_json::Map<String, serde_json::Value>> {
    let schema = schemars::schema_for!(ValidateParams);
    let object = serde_json::to_value(schema)
        .ok()
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    std::sync::Arc::new(object)
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ValidateParams {
    /// Path to the project file.
    pub project: String,
    /// Return the canonical JSON *instead of* the text report, never alongside it.
    #[serde(default)]
    pub json: bool,
    /// Expand the informational classes that collapse to one counted line.
    #[serde(default)]
    pub verbose: bool,
}

#[derive(Clone)]
pub struct Montaget {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl Montaget {
    pub fn new() -> Self {
        Montaget {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        name = "validate",
        description = "Does this project document agree with itself and with the media on \
                       disk? Reports findings with stable codes and severities, and prints \
                       its own boundary: it cannot tell you whether the file says what you \
                       meant it to say.",
        input_schema = validate_schema()
    )]
    fn validate(
        &self,
        Parameters(raw): Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, ErrorData> {
        // Arguments are taken as a raw value and checked here rather than by the
        // macro's own deserialisation, so that a bad call answers the way every other
        // bad call does. ADR-0011: "An error is a finding. Same objects and same stable
        // codes as ADR-0006, including for invocation errors, so there is exactly one
        // thing to parse across the surface." Letting the SDK reject it would honour
        // that on the CLI and break it on the surface an agent actually uses.
        let params: ValidateParams = match serde_json::from_value(raw) {
            Ok(params) => params,
            Err(e) => {
                // Both signals, deliberately. The rendered finding is what an agent
                // parses, per ADR-0011; `isError` is what tells its client the call did
                // not run at all. A `validate` that *did* run and found errors is the
                // opposite case and stays `success` — there, ADR-0006 is explicit that
                // the findings **are** the result, not a failure.
                let report = Report::bad_invocation(format!("`validate`: {e}"));
                return Ok(CallToolResult::error(vec![ContentBlock::text(
                    montaget_core::wire::render(&report, Wire::Text { verbose: false }),
                )]));
            }
        };

        let report = montaget_core::validate(&PathBuf::from(&params.project));
        let form = Wire::from_flags(params.json, params.verbose);

        // An `error` finding is an answer, not a protocol failure — ADR-0006's whole
        // point is that the findings *are* the result. The tool result carries the
        // report whatever it says; only a failure of Montaget itself would be an MCP
        // error, and this call has none to raise.
        let body = montaget_core::wire::render(&report, form);

        Ok(CallToolResult::success(vec![ContentBlock::text(body)]))
    }
}

impl Default for Montaget {
    fn default() -> Self {
        Self::new()
    }
}

// `router = self.tool_router` uses the router built once in `new`; the macro's default
// would rebuild it on every tool call.
#[tool_handler(router = self.tool_router)]
impl ServerHandler for Montaget {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("montaget", env!("CARGO_PKG_VERSION")))
            .with_instructions(
                "Montaget reads, checks and renders a declarative video project. Edit the \
             project file with your ordinary file tools — there is no CRUD API — and call \
             these tools for the things a text editor cannot do.",
            )
    }
}

/// Serve the MCP tools over stdio until the client disconnects.
pub fn serve() -> Result<(), String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("could not start the async runtime: {e}"))?;

    runtime.block_on(async {
        let service = Montaget::new()
            .serve(stdio())
            .await
            .map_err(|e| format!("could not bind the MCP stdio transport: {e}"))?;
        service
            .waiting()
            .await
            .map_err(|e| format!("the MCP session ended in error: {e}"))?;
        Ok(())
    })
}
