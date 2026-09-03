// C/D: a minimal stdio MCP server advertising one tool.
use std::borrow::Cow;
use std::sync::Arc;

use rmcp::handler::server::ServerHandler;
use rmcp::model::*;
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServiceExt};
use serde_json::{json, Map, Value};

const BIG_SCHEMA: &str = include_str!("../../element-schema.json");

#[derive(Clone)]
struct Montaget {
    input_schema: Arc<Map<String, Value>>,
}

impl Montaget {
    fn new() -> Self {
        let big = std::env::var("SCHEMA").as_deref() != Ok("small");
        let element: Value = if big {
            serde_json::from_str(BIG_SCHEMA).expect("schema parses")
        } else {
            json!({"type":"object","properties":{"type":{"type":"string"},"start":{"type":"number"}}})
        };
        let schema = json!({
            "type": "object",
            "required": ["path", "element"],
            "properties": { "path": {"type":"string"}, "element": element }
        });
        let Value::Object(map) = schema else { unreachable!() };
        Self { input_schema: Arc::new(map) }
    }
}

impl ServerHandler for Montaget {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("montaget", "0.0.0"))
    }

    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Borrowed(ProtocolVersion::KNOWN_VERSIONS)
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult {
            tools: vec![Tool::new_with_raw(
                "add_element",
                Some(Cow::Borrowed("Append one complete element to a project file.")),
                self.input_schema.clone(),
            )],
            next_cursor: None,
            ..Default::default()
        })
    }

    async fn call_tool(
        &self,
        _request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        Ok(CallToolResult::success(vec![ContentBlock::text("ok")]).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = Montaget::new().serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
