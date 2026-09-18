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
use rmcp::model::{
    CallToolResult, ContentBlock, Implementation, ListResourcesResult, PaginatedRequestParams,
    ReadResourceRequestParams, ReadResourceResponse, ReadResourceResult, Resource,
    ResourceContents, ServerCapabilities, ServerConfig,
};
use rmcp::service::RequestContext;
use rmcp::transport::stdio;
use rmcp::{ErrorData, RoleServer, ServerHandler, ServiceExt, tool, tool_handler, tool_router};

use montaget_core::Wire;
use montaget_core::report::Report;
use montaget_core::resources;
use montaget_core::verbs::create_project::Scaffold;

/// The advertised schema **is** the enforced one: `schemars` derives the document the
/// tool publishes from `T`, and `T` is what the arguments are deserialised into below.
/// There is one declaration per tool, so the two cannot drift.
fn advertised<T: schemars::JsonSchema>()
-> std::sync::Arc<serde_json::Map<String, serde_json::Value>> {
    let schema = schemars::schema_for!(T);
    let object = serde_json::to_value(schema)
        .ok()
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    std::sync::Arc::new(object)
}

fn validate_schema() -> std::sync::Arc<serde_json::Map<String, serde_json::Value>> {
    advertised::<ValidateParams>()
}

fn create_project_schema() -> std::sync::Arc<serde_json::Map<String, serde_json::Value>> {
    advertised::<CreateProjectParams>()
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

/// `create_project`'s arguments.
///
/// `frame` is two arguments rather than a nested object because the tool schema an agent
/// reads is a flat list of names and descriptions, and `{"width": …, "height": …}` nested
/// one level deep is a shape it has to reconstruct from prose. The *file* keeps the nested
/// `frame` object the format defines; that mapping is this adapter's whole job.
///
/// `background`, `duration` and `output` are optional, and their absence is not a
/// convenience — ADR-0030 makes omission and explicit-at-default two spellings of different
/// declarations, so a scaffold that filled them in would be authoring a claim the agent
/// never made. Pass them to have them written; omit them to leave them out of the file.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateProjectParams {
    /// Path of the project file to create. It must not already exist.
    pub project: String,
    /// Frame width in pixels.
    pub width: i64,
    /// Frame height in pixels.
    pub height: i64,
    /// Frames per second.
    pub fps: i64,
    /// Background colour, `#RRGGBB` or `#RRGGBBAA`, uppercase.
    #[serde(default)]
    pub background: Option<String>,
    /// The project's intended length, in whole milliseconds.
    #[serde(default)]
    pub duration: Option<i64>,
    /// Where `render` writes the video, relative to the project file.
    #[serde(default)]
    pub output: Option<String>,
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
            Err(e) => return Ok(rejected("validate", &e)),
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

    #[tool(
        name = "create_project",
        description = "Scaffold a legal, empty project file so you never start from a \
                       blank document: the header you ask for, plus an empty `tracks` \
                       array, written in the canonical convention. It never overwrites an \
                       existing file, and it never writes a field you did not state. It \
                       returns the new file's findings rather than `ok` — read \
                       `montaget://schema.json` and `montaget://format.md` before editing \
                       what it gives you.",
        input_schema = create_project_schema()
    )]
    fn create_project(
        &self,
        Parameters(raw): Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, ErrorData> {
        let params: CreateProjectParams = match serde_json::from_value(raw) {
            Ok(params) => params,
            Err(e) => return Ok(rejected("create_project", &e)),
        };

        let report = montaget_core::verbs::create_project::create_project(
            &PathBuf::from(&params.project),
            &Scaffold {
                width: params.width,
                height: params.height,
                fps: params.fps,
                background: params.background,
                duration: params.duration,
                output: params.output,
            },
        );
        let form = Wire::from_flags(params.json, params.verbose);

        // ADR-0011's write-tool invariant, at the surface it was argued for: the return
        // value *is* the findings, which is what converts an opt-in check into a structural
        // one. A scaffold that did not land is still an answer about the project and still
        // `success` here; only a failure of Montaget itself would be an MCP error.
        Ok(CallToolResult::success(vec![ContentBlock::text(
            montaget_core::wire::render(&report, form),
        )]))
    }
}

/// A call whose arguments did not match the advertised schema.
///
/// Arguments are taken as a raw value and checked here rather than by the macro's own
/// deserialisation, so that a bad call answers the way every other bad call does. ADR-0011:
/// *"An error is a finding. Same objects and same stable codes as ADR-0006, including for
/// invocation errors, so there is exactly one thing to parse across the surface."* Letting
/// the SDK reject it would honour that on the CLI and break it on the surface an agent
/// actually uses.
///
/// Both signals, deliberately. The rendered finding is what an agent parses; `isError` is
/// what tells its client the call did not run at all. A verb that *did* run and found
/// errors is the opposite case and stays `success` — there, ADR-0006 is explicit that the
/// findings **are** the result, not a failure.
fn rejected(tool: &str, e: &serde_json::Error) -> CallToolResult {
    let report = Report::bad_invocation(format!("`{tool}`: {e}"));
    CallToolResult::error(vec![ContentBlock::text(montaget_core::wire::render(
        &report,
        Wire::Text { verbose: false },
    ))])
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
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_server_info(Implementation::new("montaget", env!("CARGO_PKG_VERSION")))
        .with_instructions(
            "Montaget reads, checks and renders a declarative video project. Edit the \
             project file with your ordinary file tools — there is no CRUD API — and call \
             these tools for the things a text editor cannot do. Read the resources \
             `montaget://schema.json` (the format's shape) and `montaget://format.md` \
             (the rules the schema cannot express) to learn the format, the same way you \
             would read a `package.json` schema.",
        )
    }

    /// The two resources ADR-0011 publishes.
    ///
    /// Resources rather than tools, for a cost reason: *"every MCP tool schema occupies the
    /// agent's context and degrades tool selection on every turn"*, and a resource occupies
    /// no tool slot at all. This is *"the schema and format docs as resources"* — one of
    /// the three places the ADR says the MCP server is more than a wrapper.
    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        Ok(ListResourcesResult {
            resources: resources::all()
                .iter()
                .map(|resource| {
                    Resource::new(resource.uri, resource.name)
                        .with_title(resource.title)
                        .with_description(resource.description)
                        .with_mime_type(resource.mime_type)
                })
                .collect(),
            ..Default::default()
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        // What is published, and what its bytes are, is the library's (`montaget_core::
        // resources`). This adapter owns the protocol and nothing else — including the fact
        // that the schema is *generated* on each read rather than loaded from the committed
        // copy, which is what keeps the published schema and the enforced one one artifact.
        let Some(body) = resources::read(&request.uri) else {
            return Err(ErrorData::resource_not_found(
                format!("no resource at {}", request.uri),
                None,
            ));
        };
        let mime = resources::all()
            .iter()
            .find(|resource| resource.uri == request.uri)
            .map(|resource| resource.mime_type)
            .unwrap_or("text/plain");

        Ok(ReadResourceResult::new(vec![
            ResourceContents::text(body, &request.uri).with_mime_type(mime),
        ])
        .into())
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
