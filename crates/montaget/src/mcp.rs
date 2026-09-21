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

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
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

/// `create_project`'s arguments: **the project header, in the shape the schema gives it.**
///
/// ADR-0011's write-tool invariant is that a write tool *"may only take a complete element,
/// as a schema-shaped object. No tool takes a field name"*. The invariant names elements,
/// and this verb writes a header rather than an element — but the rule it protects is that
/// the argument shape and the file shape are one thing an agent has to learn, not two. So
/// `frame` is the nested `{"width": …, "height": …}` object the published schema defines,
/// and not a flattened `width`/`height` pair: an agent that has read
/// `montaget://schema.json` already knows this call's shape, and the adapter invents none
/// of it.
///
/// `background`, `duration` and `output` are optional, and their absence is not a
/// convenience — ADR-0030 makes omission and explicit-at-default two spellings of different
/// declarations, so a scaffold that filled them in would be authoring a claim the agent
/// never made (#246). Pass them to have them written; omit them to leave them out.
///
/// `project`, `json` and `verbose` are the call's own, not the document's: where to write,
/// and which wire form to answer in.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateProjectParams {
    /// Path of the project file to create. It must not already exist.
    pub project: String,
    /// The frame size, `{"width": 1080, "height": 1920}`, in pixels.
    pub frame: FrameParam,
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

/// `query`'s arguments: ADR-0011's three modes, all of which read the document alone.
///
/// One question per call. `at` asks for the resolved stack at an instant; `from`/`to` ask
/// for the cut list; `where` asks for the matched set, and `census` for its distribution.
/// Which combinations are legal is the verb's rule and is enforced there — this adapter
/// carries the arguments and decides nothing (ADR-0011).
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct QueryParams {
    /// Path to the project file.
    pub project: String,
    /// The instant to resolve the stack at, in absolute milliseconds.
    #[serde(default)]
    pub at: Option<i64>,
    /// The start of the range, in absolute milliseconds. Asked for with `to`.
    #[serde(default)]
    pub from: Option<i64>,
    /// The end of the range, exclusive: a range is half-open `[from, to)`.
    #[serde(default)]
    pub to: Option<i64>,
    /// Which elements to match, e.g. `type = text and group = item-05`.
    ///
    /// Terms are `<field> <op> <value>`, `<field> exists` or `<field> missing`, joined with
    /// `and`; there is no `or`, and two calls answer one. `<op>` is one of `=`, `!=`, `<`,
    /// `<=`, `>`, `>=`. A field is a dotted path into the element — `clip.width`,
    /// `runs.*.font`, `runs.0.font` — plus the reserved `track`, which is the name of the
    /// track the element sits in. A bare value reads as the JSON scalar it spells and
    /// quoting forces a string (`loop = true` the boolean, `id = "true"` the word). Values
    /// match **whole** — no substrings — **as the document writes them**; nothing is
    /// resolved. A term holds if any value the path reaches satisfies it.
    #[serde(default, rename = "where")]
    pub predicate: Option<String>,
    /// Distribute the matched set over this field, as a dotted path.
    #[serde(default)]
    pub census: Option<String>,
    /// Return the canonical JSON *instead of* the text answer, never alongside it.
    #[serde(default)]
    pub json: bool,
}

/// `frame`'s arguments: one instant, and the three knobs ADR-0011 names.
///
/// There is deliberately **no argument that suppresses the caption**. ADR-0011: *"`frame`
/// must print the `query --at` block alongside the image, unconditionally"* — an agent
/// looking at a picture without knowing which elements produced it attributes the defect to
/// the wrong element, so the block is the picture's caption rather than an alternative to
/// it, and there is nothing to turn off.
///
/// There is also no `out`: the CLI writes a file because a terminal cannot show a picture,
/// and this surface hands the image back in the result.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FrameParams {
    /// Path to the project file.
    pub project: String,
    /// The instant to draw, in absolute milliseconds on the project's one clock.
    pub at: i64,
    /// Return just this region, as `x,y,w,h` in whole frame-space pixels at true scale —
    /// so you can look closely at one card without paying for the whole canvas.
    #[serde(default)]
    pub crop: Option<String>,
    /// Return the frame at the project's true pixel dimensions instead of half of them.
    /// A 1080x1920 frame costs 2691 visual tokens at full scale and 700 at half; this flag
    /// is you choosing to spend the difference.
    #[serde(default)]
    pub full: bool,
    /// Return PNG instead of JPEG. It costs the same tokens — they are a function of
    /// decoded pixel dimensions, not of bytes — and buys lossless pixels for more latency.
    #[serde(default)]
    pub png: bool,
    /// Return the canonical JSON *instead of* the text caption, never alongside it. The
    /// image comes back either way.
    #[serde(default)]
    pub json: bool,
}

/// `measure`'s arguments: **a whole text element, in the shape the schema gives it.**
///
/// The same shape ADR-0011 fixes for the write side, and the right shape here for a reason
/// of this verb's own: ADR-0024 requires `measure` to *"work identically for an element
/// being authored for the first time"*, and such an element has no `id` to name. So the
/// argument is the element you are about to write, not a handle on one already in the file.
///
/// `width` and `height` are never read, even when the element carries them — ADR-0024 gives
/// `measure` the derivation and `validate` the verdict, and there is no declared value
/// anywhere in the verb to compare against.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MeasureParams {
    /// Path to the project file. Its `fonts` table is what `font` is a key into, and its
    /// directory is what the declared font files resolve against.
    pub project: String,
    /// The text element, as `montaget://schema.json` shapes one: `runs`, `font`, `size`,
    /// and optionally `line_height`, `y`, `origin` and `stroke_width`. Any other field is
    /// ignored, so an element still being authored measures as readily as a finished one.
    pub element: serde_json::Value,
    /// Return the canonical JSON *instead of* the text answer, never alongside it.
    #[serde(default)]
    pub json: bool,
}

/// `shift`'s arguments: the one edit that is arithmetic rather than authorship.
///
/// `at`, `delta` and `scope` are ADR-0005's, taking a timestamp and an offset — never a
/// field name, never an element id — so the call satisfies the standing write-tool
/// invariant. `release` is ADR-0047's: every slack this edit would otherwise change,
/// named as its own full boundary-instant pair, individually — there is no bulk form.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ShiftParams {
    /// Path to the project file.
    pub project: String,
    /// The instant to shift at or after, in absolute milliseconds.
    pub at: i64,
    /// The offset, in milliseconds. Must be positive.
    pub delta: i64,
    /// Narrow the edit to one track's elements. Omit for the whole project.
    #[serde(default)]
    pub scope: Option<String>,
    /// Every slack this edit would otherwise change, as `[from, to]` — its full
    /// boundary-instant pair, exactly as a refusal reports it. Naming one that does not
    /// currently bound a real, protected slack this edit would change is itself refused.
    #[serde(default)]
    pub release: Vec<[i64; 2]>,
    /// Return the canonical JSON *instead of* the text report, never alongside it.
    #[serde(default)]
    pub json: bool,
    /// Expand the informational classes that collapse to one counted line.
    #[serde(default)]
    pub verbose: bool,
}

/// `render`'s arguments: the whole project by default, or one half-open range of it.
///
/// There is no argument that skips the checks, narrows them, or renders at a proxy
/// resolution: ADR-0006 makes the check engine the thing `render` runs before it draws
/// anything, and ADR-0021 makes the deliverable the one output that is never downsampled.
/// Progress goes to the server's stderr — this surface hands back the result.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RenderParams {
    /// Path to the project file.
    pub project: String,
    /// Render only from this instant, in absolute milliseconds. Asked for with `to`. A
    /// partial render is written to `out/<name>.<from>-<to>.mp4`, never to the project's
    /// `output`.
    #[serde(default)]
    pub from: Option<i64>,
    /// The end of the partial range, exclusive: the range is half-open `[from, to)`.
    #[serde(default)]
    pub to: Option<i64>,
    /// Write here instead of the project's `output`. Refused while a range is set if it
    /// names the project's own `output`.
    #[serde(default)]
    pub output: Option<String>,
    /// Return the canonical JSON *instead of* the text result, never alongside it.
    #[serde(default)]
    pub json: bool,
    /// Expand the informational classes that collapse to one counted line.
    #[serde(default)]
    pub verbose: bool,
}

/// The project's `frame` object, as the schema shapes it.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FrameParam {
    /// Frame width in pixels.
    pub width: i64,
    /// Frame height in pixels.
    pub height: i64,
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
        input_schema = advertised::<ValidateParams>()
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
        input_schema = advertised::<CreateProjectParams>()
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
                width: params.frame.width,
                height: params.frame.height,
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
    #[tool(
        name = "query",
        description = "What is true at an instant, over a range, or across a predicate? \
                       `at` returns the resolved stack at one instant — who is present, in \
                       painter's order with anchors resolved, and every animated value \
                       interpolated rather than echoed back as its keyframe records. \
                       `from`/`to` returns the cut list — the intervals over which the set \
                       of elements present is constant, with the boundary immediately \
                       outside the range named on each side, so you never have to guess a \
                       window. `where` returns the matched set, and `census` its \
                       distribution over one field, so \"four of five siblings agree and one \
                       does not\" is one call rather than a script. All three read the \
                       document alone; only `at` resolves anything.",
        input_schema = advertised::<QueryParams>()
    )]
    fn query(
        &self,
        Parameters(raw): Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, ErrorData> {
        let params: QueryParams = match serde_json::from_value(raw) {
            Ok(params) => params,
            Err(e) => return Ok(rejected("query", &e)),
        };

        let answer = montaget_core::verbs::query::query(
            &PathBuf::from(&params.project),
            &montaget_core::verbs::query::Ask {
                at: params.at,
                from: params.from,
                to: params.to,
                predicate: params.predicate,
                census: params.census,
            },
        );
        // `verbose` is deliberately absent from this tool's schema: `query` has no
        // informational findings to expand, and an argument that changed nothing would cost
        // the agent context on every turn for no answer it could get back.
        let form = Wire::from_flags(params.json, false);

        // An answer about a project that does not parse is still an answer, not a protocol
        // failure — ADR-0006's findings **are** the result. Only a failure of Montaget
        // itself would be an MCP error, and this call has none to raise.
        Ok(CallToolResult::success(vec![ContentBlock::text(
            montaget_core::wire::render_query(&answer, form),
        )]))
    }

    #[tool(
        name = "frame",
        description = "What does it look like right now? Rasterizes one instant at the \
                       project's true pixel dimensions and hands back the picture — JPEG at \
                       half the frame size by default, because an image costs \
                       ceil(w/28) x ceil(h/28) visual tokens whatever it is encoded as, and \
                       that is 2691 at 1080x1920 against 700 at 540x960. Ask for `full` when \
                       you need true pixels and `png` when you need lossless ones. `crop` \
                       returns one region of the frame, so you can look closely at a single \
                       card. The resolved stack at that instant comes back alongside the \
                       image, always: looking at a frame without knowing which elements \
                       produced it is how a defect gets attributed to the wrong one. It \
                       reaches no verdict and measures nothing — it is how you *believe* a \
                       layout, never how you measure one, which is `measure`'s job.",
        input_schema = advertised::<FrameParams>()
    )]
    fn frame(
        &self,
        Parameters(raw): Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, ErrorData> {
        let params: FrameParams = match serde_json::from_value(raw) {
            Ok(params) => params,
            Err(e) => return Ok(rejected("frame", &e)),
        };

        let answer = montaget_core::verbs::frame::frame(
            &PathBuf::from(&params.project),
            &montaget_core::verbs::frame::Ask {
                at: Some(params.at),
                crop: params.crop,
                full: params.full,
                png: params.png,
                // The CLI's flag, and not this surface's: a picture that came back as a
                // path would be a picture an agent cannot see.
                out: None,
            },
        );
        // `verbose` is deliberately absent, as it is on `query` and `measure`.
        let form = Wire::from_flags(params.json, false);

        // The caption first and the image second, so a client that renders content blocks
        // in order shows the picture under the list of what is in it — which is the reading
        // order ADR-0011 argues for, the block being the picture's caption.
        let mut content = vec![ContentBlock::text(montaget_core::wire::render_frame(
            &answer, form,
        ))];
        if let Some(image) = answer.image() {
            content.push(ContentBlock::image(
                BASE64.encode(&image.bytes),
                image.mime_type(),
            ));
        }
        // A project that did not render is still an answer about the project — the report
        // says what stopped it. Only a failure of Montaget itself would be an MCP error.
        Ok(CallToolResult::success(content))
    }

    #[tool(
        name = "render",
        description = "Files in, video out. Runs the identical check engine `validate` \
                       runs and refuses on any `error` — the check cannot be skipped by not \
                       running it — then writes the project's declared `output` via a temp \
                       path and an atomic rename, so a truncated MP4 never reads as \
                       finished. Always at the declared frame size; never downsampled. \
                       `from`/`to` render one half-open range to `out/<name>.<from>-<to>.mp4` \
                       and can never land on the deliverable. The result carries the path, \
                       duration, frame count, wall time and realtime factor, and beneath it \
                       the `review` findings the render did not refuse on plus the NOT \
                       CHECKED footer: exit 0 never means the video is right.",
        input_schema = advertised::<RenderParams>()
    )]
    fn render(
        &self,
        Parameters(raw): Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, ErrorData> {
        let params: RenderParams = match serde_json::from_value(raw) {
            Ok(params) => params,
            Err(e) => return Ok(rejected("render", &e)),
        };

        // Progress to the server's own stderr, which is where an MCP host collects a
        // server's log: the protocol has no stream for it, and the result is the answer.
        let mut progress = |p: montaget_core::verbs::render::Progress| {
            eprintln!(
                "render  {}/{} frames  {:.1} s",
                p.done,
                p.of,
                p.elapsed.as_secs_f64()
            );
        };
        let answer = montaget_core::verbs::render::render(
            &PathBuf::from(&params.project),
            &montaget_core::verbs::render::Ask {
                from: params.from,
                to: params.to,
                output: params.output.map(PathBuf::from),
            },
            &mut progress,
        );
        let form = Wire::from_flags(params.json, params.verbose);

        // A refused render is an answer about the project — the findings say why — and
        // stays `success`; only a failure of Montaget itself would be an MCP error.
        Ok(CallToolResult::success(vec![ContentBlock::text(
            montaget_core::wire::render_video(&answer, form),
        )]))
    }

    #[tool(
        name = "measure",
        description = "What does this text actually occupy, in the fonts the project \
                       declares? Returns the advance width, ascent and descent, the line \
                       count, each line's resolved baseline_y, and the block height — the \
                       integer a text element's required `height` field takes. The extent \
                       it reports is the **stroked** one, so you are never adding 2 x \
                       stroke_width by hand. It also returns the break opportunities: the \
                       byte offsets at which a line may legally break, so you can place a \
                       `\\n` yourself in a script that writes no spaces. Montaget never \
                       places one for you, and never wraps. Pass the element you are about \
                       to write; it needs no `id` and need not exist in the file yet. It \
                       reaches no verdict — it will not tell you whether the text fits its \
                       box, which is `validate`'s alone to say.",
        input_schema = advertised::<MeasureParams>()
    )]
    fn measure(
        &self,
        Parameters(raw): Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, ErrorData> {
        let params: MeasureParams = match serde_json::from_value(raw) {
            Ok(params) => params,
            Err(e) => return Ok(rejected("measure", &e)),
        };

        let answer = montaget_core::verbs::measure::measure(
            &PathBuf::from(&params.project),
            &montaget_core::verbs::measure::Ask {
                element: Some(params.element),
            },
        );
        // `verbose` is deliberately absent, as it is on `query`: the answer *is* the
        // output and is never collapsed, so an argument that changed nothing would cost the
        // agent context on every turn.
        let form = Wire::from_flags(params.json, false);

        Ok(CallToolResult::success(vec![ContentBlock::text(
            montaget_core::wire::render_measure(&answer, form),
        )]))
    }

    #[tool(
        name = "shift",
        description = "Move every time at or after an instant, project-scoped by default. \
                       Refuses a straddling time-based element (audio, video) rather than \
                       stretching or relocating it, naming the nearest legal boundaries. A \
                       time-invariant straddler's transform keyframes are carried with it \
                       and split rather than dragged by the raw at-or-after rule, so a shot \
                       that finished before the edit point does not move. Prints what it \
                       will do to every record sitting exactly at `at`, unconditionally. \
                       Every slack in the file is invariant by default: an edit that would \
                       change one is refused, listing each threatened pair; pass `release` \
                       with exactly those pairs to consume them. Returns the new state's \
                       findings, never `ok` — read them the way you read `validate`'s.",
        input_schema = advertised::<ShiftParams>()
    )]
    fn shift(
        &self,
        Parameters(raw): Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, ErrorData> {
        let params: ShiftParams = match serde_json::from_value(raw) {
            Ok(params) => params,
            Err(e) => return Ok(rejected("shift", &e)),
        };

        let answer = montaget_core::verbs::shift::shift(
            &PathBuf::from(&params.project),
            &montaget_core::verbs::shift::Ask {
                at: params.at,
                delta: params.delta,
                scope: params.scope,
                release: params
                    .release
                    .into_iter()
                    .map(|[from, to]| (from, to))
                    .collect(),
            },
        );
        let form = Wire::from_flags(params.json, params.verbose);

        Ok(CallToolResult::success(vec![ContentBlock::text(
            montaget_core::wire::render_shift(&answer, form),
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
        let Some(resource) = resources::find(&request.uri) else {
            return Err(ErrorData::resource_not_found(
                format!("no resource at {}", request.uri),
                None,
            ));
        };
        Ok(ReadResourceResult::new(vec![
            ResourceContents::text(resource.body(), resource.uri)
                .with_mime_type(resource.mime_type),
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
