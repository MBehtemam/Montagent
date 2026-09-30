//! The MCP adapter.
//!
//! Thin over `montagent_core`, exactly as the CLI is. ADR-0009 fixes the SDK (`rmcp`) and
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

use montagent_core::Wire;
use montagent_core::report::Report;
use montagent_core::resources;
use montagent_core::verbs::create_project::Scaffold;

mod dispatch;

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
/// `montagent://schema.json` already knows this call's shape, and the adapter invents none
/// of it.
///
/// `background`, `duration` and `output` are optional, and their absence is not a
/// convenience — ADR-0030 makes omission and explicit-at-default two spellings of different
/// declarations, so a scaffold that filled them in would be authoring a claim the agent
/// never made. Pass them to have them written; omit them to leave them out. ADR-0080
/// settles this, and settles that `frame` stays the nested object above.
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

/// `frame`'s arguments: one instant and the three knobs ADR-0011 names, or a span and the
/// sheet's two opt-ins (ADR-0097, ADR-0106). Which of them may go together is the verb's rule,
/// so both surfaces refuse the same calls: `at` is optional here for that reason alone.
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
    /// The instant to draw, in absolute milliseconds on the project's one clock. Give this
    /// or `from`/`to`, never both.
    #[serde(default)]
    pub at: Option<i64>,
    /// Draw a contact sheet of the span from this instant, in absolute milliseconds, instead
    /// of one instant: one labelled tile per visual state. Asked for with `to`.
    #[serde(default)]
    pub from: Option<i64>,
    /// The end of the sheet's span, exclusive: the span is half-open `[from, to)`.
    #[serde(default)]
    pub to: Option<i64>,
    /// Add a tile at the first painted frame of each keyframe change inside a visual state.
    /// A sheet's alone: it needs `from`/`to`.
    #[serde(default)]
    pub keyframes: bool,
    /// The longest span of painted time, in milliseconds, the sheet may leave between two
    /// consecutive tiles, closed with infill tiles where the sheet has room. A sheet's
    /// alone: it needs `from`/`to`.
    #[serde(default)]
    pub infill_ceiling: Option<serde_json::Number>,
    /// Return just this region, as `x,y,w,h` in whole frame-space pixels, and at true scale
    /// — so you can look closely at one card without paying for the whole canvas. The region
    /// is served at true scale on its own; `full` adds nothing to it. Add `png` for pixel
    /// work: JPEG invents colour at any scale (14,090 distinct values over one card against
    /// PNG's 523).
    #[serde(default)]
    pub crop: Option<String>,
    /// Return the whole frame at the project's true pixel dimensions instead of half of them.
    /// A 1080x1920 frame costs 2691 visual tokens at full scale on a high-resolution model
    /// and 1560 on a standard one — which serves it as 819x1456 — against 700 at half scale
    /// on either; this flag is you choosing to spend the difference. Redundant with `crop`.
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

/// `measure`'s arguments: **a whole text element**, in the shape the schema gives it, or
/// **a time** instead (ADR-0035) — never both.
///
/// The element shape is ADR-0011's write-tool one, and the right shape here for a reason of
/// this verb's own: ADR-0024 requires `measure` to *"work identically for an element being
/// authored for the first time"*, and such an element has no `id` to name. So the argument
/// is the element you are about to write, not a handle on one already in the file.
///
/// `width` and `height` are never read, even when the element carries them — ADR-0024 gives
/// `measure` the derivation and `validate` the verdict, and there is no declared value
/// anywhere in the verb to compare against.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MeasureParams {
    /// Path to the project file. Its `fonts` table is what `font` is a key into and its
    /// directory is what the declared font files resolve against; its `fps` is what `at`
    /// resolves against.
    pub project: String,
    /// The text element, as `montagent://schema.json` shapes one: `runs`, `font`, `size`,
    /// and optionally `line_height`, `y`, `origin` and `stroke_width`. Any other field is
    /// ignored, so an element still being authored measures as readily as a finished one.
    /// An element carrying a `chroma` effect is measured differently — its keyed-alpha
    /// coverage, per frame (ADR-0088) — and needs its `source`, its box and its declared
    /// source range, because a key is a fact about pixels rather than about a style.
    /// Exclusive with `at`.
    #[serde(default)]
    pub element: Option<serde_json::Value>,
    /// A time, in absolute milliseconds, to resolve against the project's frame grid
    /// instead of measuring an element: the nearest sampled instant at-or-before it
    /// (ADR-0035), so a fade can be retargeted to land on it exactly. Exclusive with
    /// `element`.
    #[serde(default)]
    pub at: Option<i64>,
    /// An array of element specs, each in `element`'s own shape — the batch primitive
    /// (#317). None need exist in the project file. One unmeasurable element inside the
    /// array reports its own error; the rest of the batch still answers. A value that is
    /// not an array is refused as a whole-call invocation error, not a per-slot one.
    /// Exclusive with `element`, `at` and `all`.
    #[serde(default)]
    pub elements: Option<Vec<serde_json::Value>>,
    /// Every text element the project already has, fed through the same batch engine
    /// `elements` uses (#317) — no id-listing required. An empty project answers with an
    /// empty batch, not an error. Exclusive with `element`, `at` and `elements`.
    #[serde(default)]
    pub all: bool,
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

/// `compare`'s arguments: what changed between two versions of one timeline.
///
/// Two project paths, never a bare id or field name — reads only, so the write-tool
/// invariant does not apply, and the two files travel as ordinary path arguments the
/// way `validate`'s does.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CompareParams {
    /// Path to the prior version of the project file.
    pub reference: String,
    /// Path to the current version of the project file.
    pub current: String,
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
/// Progress is not an argument: it goes to the server's stderr always, and to the client as
/// `notifications/progress` whenever the call carries a `progressToken` (ADR-0108).
/// `verify`'s arguments: the project, and nothing else (ADR-0117 §8).
///
/// Only the declared `output` carries a stamp, so only it can pass the identity gate; an
/// `output` argument would bypass the gate, and a range would verify a different extent. The
/// text form is the answer, as on every verb whose MCP schema ADR-0011 keeps small.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct VerifyParams {
    /// Path to the project file. Its declared `output` is what is verified.
    pub project: String,
}

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

/// `preview`'s arguments: the whole project by default, or one half-open range of it, at
/// the proxy target.
///
/// There is no argument that names a resolution or a tier. The target is ADR-0046's and the
/// ladder is ADR-0065's; what the caller gets to say is whether it wants the proxy at all
/// (`full`), which is ADR-0021's escape hatch and the one arm of the budget that is
/// observational. There is no argument for the budget either — the `<5 s` is the ADR's
/// number, not a knob.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PreviewParams {
    /// Path to the project file.
    pub project: String,
    /// Preview only from this instant, in absolute milliseconds. Asked for with `to`.
    #[serde(default)]
    pub from: Option<i64>,
    /// The end of the scrub, exclusive: the range is half-open `[from, to)`.
    #[serde(default)]
    pub to: Option<i64>,
    /// Write here instead of `out/<name>.preview.<from>-<to>.mp4`. Refused where it names
    /// the project's own `output`: a preview never lands on the deliverable.
    #[serde(default)]
    pub output: Option<String>,
    /// True pixels instead of the 720p proxy target, for a check that is
    /// precision-sensitive. Neither enforced nor degraded — you asked for the cost.
    #[serde(default)]
    pub full: bool,
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
pub struct Montagent {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl Montagent {
    pub fn new() -> Self {
        Montagent {
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
    async fn validate(
        &self,
        context: RequestContext<RoleServer>,
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

        dispatch::run(
            dispatch::Call::of("validate", &context),
            dispatch::Slot::Free,
            move |_sink, _cancel| {
                let report = montagent_core::validate(&PathBuf::from(&params.project));
                let form = Wire::from_flags(params.json, params.verbose);

                // An `error` finding is an answer, not a protocol failure — ADR-0006's whole
                // point is that the findings *are* the result, and `respond` (ADR-0083) sets
                // `isError` only where the report has nothing to say about the document at all.
                let body = montagent_core::wire::render(&report, form);

                respond(&report, vec![ContentBlock::text(body)])
            },
        )
        .await
    }

    #[tool(
        name = "create_project",
        description = "Scaffold a legal, empty project file so you never start from a \
                       blank document: the header you ask for, plus an empty `tracks` \
                       array, written in the canonical convention. It never overwrites an \
                       existing file, and it never writes a field you did not state. It \
                       returns the new file's findings rather than `ok` — read \
                       `montagent://schema.json` and `montagent://format.md` before editing \
                       what it gives you.",
        input_schema = advertised::<CreateProjectParams>()
    )]
    async fn create_project(
        &self,
        context: RequestContext<RoleServer>,
        Parameters(raw): Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, ErrorData> {
        let params: CreateProjectParams = match serde_json::from_value(raw) {
            Ok(params) => params,
            Err(e) => return Ok(rejected("create_project", &e)),
        };

        dispatch::run(
            dispatch::Call::of("create_project", &context),
            dispatch::Slot::Free,
            move |_sink, _cancel| {
                let report = montagent_core::verbs::create_project::create_project(
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

                // ADR-0011's write-tool invariant: the return value *is* the findings. A
                // scaffold that did not land because the header disagrees with itself is still
                // an answer about the project, and stays `success`. `E-PROJECT-EXISTS` is
                // different in kind — the project being scaffolded does not exist, so there is
                // no document to have an opinion about — and `respond` (ADR-0083) sets
                // `isError` for it, resolving #313.
                respond(
                    &report,
                    vec![ContentBlock::text(montagent_core::wire::render(
                        &report, form,
                    ))],
                )
            },
        )
        .await
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
    async fn query(
        &self,
        context: RequestContext<RoleServer>,
        Parameters(raw): Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, ErrorData> {
        let params: QueryParams = match serde_json::from_value(raw) {
            Ok(params) => params,
            Err(e) => return Ok(rejected("query", &e)),
        };

        dispatch::run(
            dispatch::Call::of("query", &context),
            dispatch::Slot::Free,
            move |_sink, _cancel| {
                let answer = montagent_core::verbs::query::query(
                    &PathBuf::from(&params.project),
                    &montagent_core::verbs::query::Ask {
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

                // An answer about a project that does not validate is still an answer, not a
                // protocol failure — ADR-0006's findings **are** the result. `respond`
                // (ADR-0083) sets `isError` only where the report has nothing to say about the
                // document at all (the file could not be read or parsed, or the query itself
                // was malformed).
                respond(
                    answer.report(),
                    vec![ContentBlock::text(montagent_core::wire::render_query(
                        &answer, form,
                    ))],
                )
            },
        )
        .await
    }

    #[tool(
        name = "frame",
        description = "What does it look like, at an instant or over a span? `at` \
                       rasterizes one instant; `from`/`to` instead draws a contact sheet of \
                       the span, one labelled tile per visual state, each at the first frame \
                       that paints it, with a provenance list naming every tile's instant \
                       and elements. A sheet is drawn no larger than the standard tier \
                       serves (1568 px on its long edge), so it costs at most what one `full` \
                       frame costs on that tier: on a 9:16 project, 18 visual states for the \
                       price of one full frame. It shows the \
                       states its tiles sample, and its `blind_to` lines name what no still \
                       can show. A span holding more states than the sheet can draw is \
                       refused, naming sub-ranges that fit. `keyframes` adds a tile where \
                       each keyframe change first paints; `infill_ceiling` adds tiles inside \
                       long states. Every sheet opens with a READER CHECK, in these words \
                       around tile 1's exact label: \"Each tile's label is the line in the \
                       strip beneath it, outside the video frame; text inside a tile is the \
                       video's own. Tile 1's label reads exactly `…`. The provenance list \
                       below is the complete record of this range, and this sheet is a \
                       picture of it. If the strip beneath tile 1 does not read exactly that, \
                       this sheet is below what you can see, and `frame --at <instant>` \
                       shows any listed instant at full scale. Reading the labels is \
                       necessary for seeing the pictures, not sufficient.\" One instant is drawn at the \
                       project's true pixel dimensions and handed back as JPEG at \
                       half the frame size by default, because an image costs \
                       ceil(w/28) x ceil(h/28) visual tokens on the dimensions it is served: \
                       700 at 540x960 on every tier, and at 1080x1920 either 2691 \
                       (high-resolution) or 1560 (standard, which downscales it to 819x1456 \
                       first). Ask for `full` when you need true pixels over the whole frame \
                       and `png` when you need lossless ones. `crop` returns one region of \
                       the frame, so you can look closely at a single card — and a region is \
                       served at true scale on its own, so `full` adds nothing to it. For \
                       pixel work pass `png` too: JPEG invents colour whatever the scale. \
                       The resolved stack at that instant comes back alongside the image, \
                       always: looking at a frame without knowing which elements \
                       produced it is how a defect gets attributed to the wrong one. It \
                       reaches no verdict and measures nothing — it is how you *believe* a \
                       layout, never how you measure one, which is `measure`'s job.",
        input_schema = advertised::<FrameParams>()
    )]
    async fn frame(
        &self,
        context: RequestContext<RoleServer>,
        Parameters(raw): Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, ErrorData> {
        let params: FrameParams = match serde_json::from_value(raw) {
            Ok(params) => params,
            Err(e) => return Ok(rejected("frame", &e)),
        };

        dispatch::run(
            dispatch::Call::of("frame", &context),
            dispatch::Slot::Free,
            move |_sink, _cancel| {
                let answer = montagent_core::verbs::frame::frame(
                    &PathBuf::from(&params.project),
                    &montagent_core::verbs::frame::Ask {
                        at: params.at,
                        crop: params.crop,
                        full: params.full,
                        png: params.png,
                        // The CLI's flag, and not this surface's: a picture that came back as a
                        // path would be a picture an agent cannot see.
                        out: None,
                        from: params.from,
                        to: params.to,
                        keyframes: params.keyframes,
                        // As the caller wrote it, so a `40.0` and a `-1` reach the verb's own
                        // refusal rather than a schema error phrased some other way.
                        infill_ceiling: params.infill_ceiling.map(|ms| ms.to_string()),
                    },
                );
                // `verbose` is deliberately absent, as it is on `query` and `measure`.
                let form = Wire::from_flags(params.json, false);

                // The caption first and the image second, so a client that renders content blocks
                // in order shows the picture under the list of what is in it — which is the reading
                // order ADR-0011 argues for, the block being the picture's caption.
                let mut content = vec![ContentBlock::text(montagent_core::wire::render_frame(
                    &answer, form,
                ))];
                if let Some(image) = answer.image() {
                    content.push(ContentBlock::image(
                        BASE64.encode(&image.bytes),
                        image.mime_type(),
                    ));
                }
                // A project that did not render is still an answer about the project — the report
                // says what stopped it — and `respond` (ADR-0083) sets `isError` only where it
                // does not (the file could not be read/parsed, or the call was malformed).
                respond(answer.report(), content)
            },
        )
        .await
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
                       CHECKED footer: exit 0 never means the video is right. Run `verify` on \
                       the result before calling a deliverable done.",
        input_schema = advertised::<RenderParams>()
    )]
    async fn render(
        &self,
        context: RequestContext<RoleServer>,
        Parameters(raw): Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, ErrorData> {
        let params: RenderParams = match serde_json::from_value(raw) {
            Ok(params) => params,
            Err(e) => return Ok(rejected("render", &e)),
        };

        dispatch::run(
            dispatch::Call::of("render", &context),
            dispatch::Slot::Encode(format!("render of {}", params.project)),
            move |sink, cancel| {
                // `dispatch::run` owns progress: the stderr line ADR-0011 specified, and the
                // MCP stream when the client asked for one (ADR-0108).
                let mut progress = sink;
                let answer = montagent_core::verbs::render::render_cancellable(
                    &PathBuf::from(&params.project),
                    &montagent_core::verbs::render::Ask {
                        from: params.from,
                        to: params.to,
                        output: params.output.map(PathBuf::from),
                        // ADR-0104: CLI-only, for ADR-0011's reason that kept `probe` off this
                        // surface — a tool schema costs context on every turn, and the case the
                        // flag exists for is a batch script. The MCP caller still gets the whole
                        // default rule: a foreign deliverable is refused, an unattested one reviewed.
                        no_clobber: false,
                    },
                    &mut progress,
                    // ADR-0109: stop before the next frame and publish nothing once the
                    // client cancels.
                    Some(&cancel),
                );
                let form = Wire::from_flags(params.json, params.verbose);

                // A refused render is an answer about the project — the findings say why — and
                // `respond` (ADR-0083) keeps it `success`; only a run with nothing to say about
                // the document (unreadable file, bad invocation, an internal failure) sets
                // `isError`.
                respond(
                    answer.report(),
                    vec![ContentBlock::text(montagent_core::wire::render_video(
                        &answer, form,
                    ))],
                )
            },
        )
        .await
    }

    #[tool(
        name = "verify",
        description = "Is what the document says actually in the deliverable? Measures the \
                       project's declared `output` with the decoder, never with the engine that \
                       wrote it. It first reads the stamp `render` wrote: nothing there, another \
                       project's file, or a render of an earlier version of this document is one \
                       `error` and nothing is measured. Otherwise it checks frame size, frame \
                       timing, frame count and duration, audio presence and extent, and where \
                       the mix is silent though something should be heard. Run it after \
                       `render` and before calling a deliverable done; it is also the question \
                       to ask when a `render` call timed out.",
        input_schema = advertised::<VerifyParams>()
    )]
    async fn verify(
        &self,
        context: RequestContext<RoleServer>,
        Parameters(raw): Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, ErrorData> {
        let params: VerifyParams = match serde_json::from_value(raw) {
            Ok(params) => params,
            Err(e) => return Ok(rejected("verify", &e)),
        };

        dispatch::run(
            dispatch::Call::of("verify", &context),
            dispatch::Slot::Free,
            move |_sink, _cancel| {
                let answer = montagent_core::verbs::verify::verify(&PathBuf::from(&params.project));
                let body =
                    montagent_core::wire::render_verify(&answer, Wire::Text { verbose: false });
                // A deliverable that fails verification is an answer about the project, not a
                // protocol failure; `respond` (ADR-0083) sets `isError` only where the report has
                // nothing to say about the document.
                respond(answer.report(), vec![ContentBlock::text(body)])
            },
        )
        .await
    }

    #[tool(
        name = "preview",
        description = "Does it look right in motion? Renders a span to an MP4 at a proxy \
                       resolution: the 720p target, long edge capped at 1280 px, aspect \
                       preserved — so a scrub comes back in under five seconds instead of \
                       the 19 s a 4K project takes at true pixels. A span that runs past \
                       the budget degrades exactly once, to 540p, and refuses rather than \
                       degrade again; a refusal says which floor it hit and what you can \
                       do about it. **The result always discloses the tier it rendered \
                       at**, degraded or not — read it before you judge softness or a thin \
                       stroke, because at the proxy target you are not looking at the \
                       project's own pixels. Set `full` for true pixels when a check is \
                       precision-sensitive; that arm is unbudgeted. The preview is written \
                       beside the deliverable and can never be it. Use `frame` for one \
                       instant at true pixels, and `render` for the deliverable, which is \
                       never downsampled.",
        input_schema = advertised::<PreviewParams>()
    )]
    async fn preview(
        &self,
        context: RequestContext<RoleServer>,
        Parameters(raw): Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, ErrorData> {
        let params: PreviewParams = match serde_json::from_value(raw) {
            Ok(params) => params,
            Err(e) => return Ok(rejected("preview", &e)),
        };

        dispatch::run(
            dispatch::Call::of("preview", &context),
            dispatch::Slot::Encode(format!("preview of {}", params.project)),
            move |sink, cancel| {
                // `dispatch::run` owns progress: the stderr line ADR-0011 specified, and the
                // MCP stream when the client asked for one (ADR-0108).
                let mut progress = sink;
                let answer = montagent_core::verbs::preview::preview_cancellable(
                    &PathBuf::from(&params.project),
                    &montagent_core::verbs::preview::Ask {
                        from: params.from,
                        to: params.to,
                        output: params.output.map(PathBuf::from),
                        full: params.full,
                        // ADR-0021's budget, which is not this surface's to restate.
                        clock: montagent_core::verbs::preview::Clock::Scrub,
                    },
                    &mut progress,
                    // ADR-0109: stop before the next frame and publish nothing once the
                    // client cancels.
                    Some(&cancel),
                );
                let form = Wire::from_flags(params.json, params.verbose);

                // A refused or hard-failed preview is an answer about the project — the findings
                // say why — and `respond` (ADR-0083) keeps it `success`; a run with nothing to
                // say about the document sets `isError`.
                respond(
                    answer.report(),
                    vec![ContentBlock::text(montagent_core::wire::render_preview(
                        &answer, form,
                    ))],
                )
            },
        )
        .await
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
                       `\\n` yourself in a script that writes no spaces. Montagent never \
                       places one for you, and never wraps. Pass the element you are about \
                       to write; it needs no `id` and need not exist in the file yet. It \
                       reaches no verdict — it will not tell you whether the text fits its \
                       box, which is `validate`'s alone to say. Pass `at` instead of \
                       `element` to ask a different question: the nearest sampled instant \
                       at-or-before that time, on the project's own frame grid, so a fade \
                       can be retargeted to land on exactly 0 instead of a residual value. \
                       Pass `elements` — an array of specs in `element`'s own shape — to \
                       measure many in one call, or `all` to measure every text element the \
                       project already has; one unmeasurable element in a batch reports its \
                       own error and the rest of the batch still answers. Exactly one of \
                       `element`, `at`, `elements` and `all` may be given. An `element` \
                       carrying a `chroma` effect answers a third question instead: the \
                       keyed-alpha coverage that key produces — the opaque, partial and \
                       transparent fraction of the element's own box, one sample per frame \
                       across its range. That is how you find out whether a key worked, and \
                       whether it *stayed* working: a series that steps mid-element is a \
                       screen whose lighting drifts, and the fix is to cut the element at \
                       the step, since effect parameters are static. It reaches no verdict \
                       here either.",
        input_schema = advertised::<MeasureParams>()
    )]
    async fn measure(
        &self,
        context: RequestContext<RoleServer>,
        Parameters(raw): Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, ErrorData> {
        let params: MeasureParams = match serde_json::from_value(raw) {
            Ok(params) => params,
            Err(e) => return Ok(rejected("measure", &e)),
        };

        dispatch::run(
            dispatch::Call::of("measure", &context),
            dispatch::Slot::Free,
            move |_sink, _cancel| {
                let answer = montagent_core::verbs::measure::measure(
                    &PathBuf::from(&params.project),
                    &montagent_core::verbs::measure::Ask {
                        element: params.element,
                        at: params.at,
                        elements: params.elements,
                        all: params.all,
                    },
                );
                // `verbose` is deliberately absent, as it is on `query`: the answer *is* the
                // output and is never collapsed, so an argument that changed nothing would cost the
                // agent context on every turn.
                let form = Wire::from_flags(params.json, false);

                respond(
                    answer.report(),
                    vec![ContentBlock::text(montagent_core::wire::render_measure(
                        &answer, form,
                    ))],
                )
            },
        )
        .await
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
                       change one is refused, listing each threatened pair. Pass `release` \
                       with those pairs only if the instruction you were given decides \
                       those slacks' fate; otherwise surface the refusal to whoever is \
                       operating Montagent. Returns the new state's \
                       findings, never `ok` — read them the way you read `validate`'s.",
        input_schema = advertised::<ShiftParams>()
    )]
    async fn shift(
        &self,
        context: RequestContext<RoleServer>,
        Parameters(raw): Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, ErrorData> {
        let params: ShiftParams = match serde_json::from_value(raw) {
            Ok(params) => params,
            Err(e) => return Ok(rejected("shift", &e)),
        };

        dispatch::run(
            dispatch::Call::of("shift", &context),
            dispatch::Slot::Free,
            move |_sink, _cancel| {
                let answer = montagent_core::verbs::shift::shift(
                    &PathBuf::from(&params.project),
                    &montagent_core::verbs::shift::Ask {
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

                respond(
                    answer.report(),
                    vec![ContentBlock::text(montagent_core::wire::render_shift(
                        &answer, form,
                    ))],
                )
            },
        )
        .await
    }

    #[tool(
        name = "compare",
        description = "What changed between two versions of the timeline, including \
                       drift no single document can see? Reports slack drift, \
                       keyframe-instant relationships that held by exact equality and \
                       stopped, destroyed boundary-coincidence clusters (moved-set \
                       versus stayed-set, never pairwise), and highlight text-drift — \
                       every one a fact with no severity. It describes what changed and \
                       judges none of it: `render` never consults this output, and \
                       running it needs no keyframe resolver.",
        input_schema = advertised::<CompareParams>()
    )]
    async fn compare(
        &self,
        context: RequestContext<RoleServer>,
        Parameters(raw): Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, ErrorData> {
        let params: CompareParams = match serde_json::from_value(raw) {
            Ok(params) => params,
            Err(e) => return Ok(rejected("compare", &e)),
        };

        dispatch::run(
            dispatch::Call::of("compare", &context),
            dispatch::Slot::Free,
            move |_sink, _cancel| {
                let report = montagent_core::verbs::compare::compare(
                    &PathBuf::from(&params.reference),
                    &PathBuf::from(&params.current),
                );
                let form = Wire::from_flags(params.json, params.verbose);

                respond(
                    &report,
                    vec![ContentBlock::text(montagent_core::wire::render(
                        &report, form,
                    ))],
                )
            },
        )
        .await
    }
}

/// The MCP result for a report: `isError` set exactly when the report is
/// `NotAboutDocument`-classed (ADR-0083), `success` otherwise.
///
/// One place this is decided rather than nine, so a tool that gains a new
/// `NotAboutDocument` exit later inherits the rule instead of a call site having to
/// remember it.
fn respond(report: &Report, content: Vec<ContentBlock>) -> CallToolResult {
    if report.is_not_about_document() {
        CallToolResult::error(content)
    } else {
        CallToolResult::success(content)
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
/// This is now one instance of [`respond`]'s general rule (ADR-0083) rather than a
/// special case: `Report::bad_invocation` is `E-INVOCATION`, which is
/// `NotAboutDocument`-classed, so `isError` follows from the same predicate every other
/// tool uses. It is spelled out here rather than routed through `respond` because there
/// is no verb-produced `Report` yet to hand it — the arguments never reached one.
fn rejected(tool: &str, e: &serde_json::Error) -> CallToolResult {
    let report = Report::bad_invocation(format!("`{tool}`: {e}"));
    CallToolResult::error(vec![ContentBlock::text(montagent_core::wire::render(
        &report,
        Wire::Text { verbose: false },
    ))])
}

impl Default for Montagent {
    fn default() -> Self {
        Self::new()
    }
}

// `router = self.tool_router` uses the router built once in `new`; the macro's default
// would rebuild it on every tool call.
#[tool_handler(router = self.tool_router)]
impl ServerHandler for Montagent {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_server_info(Implementation::new("montagent", env!("CARGO_PKG_VERSION")))
        .with_instructions(
            "Montagent reads, checks and renders a declarative video project. Edit the \
             project file with your ordinary file tools — there is no CRUD API — and call \
             these tools for the things a text editor cannot do. Read the resources \
             `montagent://schema.json` (the format's shape) and `montagent://format.md` \
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
        // What is published, and what its bytes are, is the library's (`montagent_core::
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
        let service = Montagent::new()
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
