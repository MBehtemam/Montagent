//! `--at <t>` — the resolved stack at an instant.
//!
//! ADR-0011: *"`query --at <t>` — the resolved stack at an instant"*, under the verb's
//! organising rule — *"`query` returns resolved values, never echoed fields. Echoing
//! `"scale":[[3018,1.0],[18018,1.08]]` back at the agent tells it nothing it did not have;
//! `scale 1.0170` is the entire point."* This is the mode that rule was written about, and
//! [ADR-0070](../../../../docs/adr/0070-the-where-predicate-is-a-conjunction-of-whole-value-terms.md)
//! since bounded it to exactly this mode: `--where` matches what the document writes.
//!
//! Three things are answered here, and each is a thing the shell was measured getting wrong
//! or getting right for the wrong reason:
//!
//! - **Presence** — every element whose half-open range contains the instant (`CONTEXT.md`'s
//!   *Presence set*). ADR-0011's verifier found `jq` does this correctly.
//! - **Painter's order** — from [`crate::stack`], which is ticket 11's resolution and the
//!   only implementation of it. ADR-0011 found the shell one-liner *"silently invents an
//!   order at layer ties"*, and ADR-0060 closed that structurally: a tie whose boxes overlap
//!   is an `error`, so the only ties this mode can meet are ones where *"any consistent
//!   internal order is correct by definition"*. Ascending resolved layer, back to front,
//!   document order within a tie.
//! - **Resolved animated values** — from [`crate::resolve`], the keyframe resolver, which is
//!   likewise the only implementation.
//!
//! ## The four components that reach outside the document
//!
//! ADR-0011 names four more: the offset into the source, the crop rectangle, the ink box,
//! and `NOT COVERED`. [#210](https://github.com/MBehtemam/Montagent/issues/210) is what
//! builds them, each through the layer that already owns its arithmetic rather than a
//! second implementation of one:
//!
//! - **Offset into source** ([`source_offset`]) reads `source_start`/`source_end`/`speed`/
//!   `overrun` off the document and [`crate::exact`]'s ADR-0020 arithmetic — no probe,
//!   argued at [`source_offset`] itself.
//! - **The crop rectangle** ([`crop_for`], [`super::geometry::crop_rectangle`]) is the one
//!   component that opens anything: a raster element's real source dimensions are not in
//!   the document (ADR-0013's own worked example), so `at` now takes a probing
//!   [`Session`], optionally — every other component answers in full without one.
//! - **The ink box** ([`super::geometry::ink_box`]) calls the same
//!   [`montagent_text::measure`] `measure` the verb calls (#205), never a second
//!   implementation of glyph layout.
//! - **`NOT COVERED`** ([`super::geometry::not_covered`]) unions every visible element's
//!   own drawn rectangle and reports the complement, refusing outright rather than
//!   approximating where an element is rotated.
//!
//! A fifth component appears in ADR-0011's verifier table — *"previous / next boundary"* —
//! and is deliberately **not** here either, on the reading that the table is the hostile
//! consumer's analysis rather than a specification of this mode. The ADR's own sentence
//! naming what the output carries lists six things and that is not among them, and the
//! boundary want is folded into `--from`/`--to` by name: *"it must always name the boundary
//! immediately outside the range on each side, which folds in the `boundaries` want without
//! a fourth verb"*. Stated rather than left silent, because the alternative reading is
//! available to anyone who reads the table first.
//!
//! ## The surface, and where it is argued
//!
//! ADR-0011 names the components this answer must carry and not the shape it is written in,
//! so the keys below — and the readings above and below them — are this ticket's own.
//! [#269](https://github.com/MBehtemam/Montagent/issues/269) carries them for ratification,
//! rather than leaving them to be discovered from this file.
//!
//! ## Why declared properties only, and no defaults
//!
//! ADR-0012 publishes a default for every transform property — `x` and `y` to the frame
//! centre, `scale` to `[1,1]`, `rotation` to `0`, `opacity` to `1`. This view resolves what
//! the document **declares** and synthesises none of them, because ADR-0030 makes a
//! defaultable field's *presence* content: omitted and explicit-at-default are two different
//! declarations, and a row reading `opacity 1` that might mean either would collapse exactly
//! the distinction the format keeps — on the one verb an agent uses to find out what the
//! document says.

use serde::Serialize;
use serde_json::{Value, json};

use crate::animatable::Unreadable;
use crate::exact::{self, Decimal};
use crate::media::Source;
use crate::media::probe::Outcome;
use crate::media::session::Session;
use crate::model::Animatable;
use crate::permissive::Loose;
use crate::resolve::{self, Unresolvable, VertexAt};
use crate::stack::{Stack, Unresolved};

use super::Named;
use super::geometry::{self, InkBox, NotAxisAligned, Rect, clip_rect, covers_the_frame};

/// The resolved stack at one instant.
#[derive(Debug, Clone, Serialize)]
pub struct At {
    /// The instant asked about, in absolute milliseconds on the project's one clock.
    pub at: i64,
    /// The presence set, in painter's order: ascending resolved layer, back to front.
    ///
    /// Audio included, like the cut list's — *"audio is an element like any other; nothing
    /// owns it"* (ADR-0001), and a caller wanting only the visual stack filters one field.
    /// ADR-0074 settles that the presence set is every element in both modes, so the term
    /// cannot mean one thing here and something narrower in the cut list.
    pub stack: Vec<Present>,
    /// Elements the document does not place on the clock — no `start`, no `end`, or one of
    /// them written as something other than whole milliseconds.
    ///
    /// Named rather than dropped, for the cut list's reason: a stack silently computed over
    /// 58 of 60 elements is a wrong answer that looks like a right one, and *which* way the
    /// range is malformed is `validate`'s question and not a view's.
    pub unplaced: Vec<String>,
    /// The region of the frame no element in the presence set reaches — ADR-0011's
    /// `NOT COVERED`, as a set of rectangles whose union is exactly that region. Never the
    /// *fewest* rectangles that could say it (see [`geometry::not_covered`]).
    pub not_covered: Vec<Rect>,
    /// Why `not_covered` is empty when it should not be trusted as *"fully covered"` — a
    /// rotated element in the presence set, whose footprint this module answers in
    /// rectangles only (see [`geometry::NotAxisAligned`]). `null` where the computation
    /// ran to completion, including when it legitimately found nothing uncovered.
    pub not_covered_unresolved: Option<String>,
}

/// One element of the presence set, resolved.
#[derive(Debug, Clone, Serialize)]
pub struct Present {
    #[serde(flatten)]
    pub named: Named,
    pub start: i64,
    pub end: i64,
    /// Where it draws, as one integer, resolved in exactly one hop (ADR-0019).
    pub layer: Option<i64>,
    /// Why there is no integer, where there is none. Present and `null` otherwise, so that
    /// "resolved to nothing" is never read off the absence of a key.
    pub layer_unresolved: Option<String>,
    /// Every animated property the element **declares**, resolved at the instant — in the
    /// order the format declares them, never the order the file happens to write them in.
    pub values: Vec<Resolved>,
    /// How the element composites into what is below it (ADR-0147), in the word the file
    /// uses, `normal` included where the field is omitted: a member that blends and one that
    /// does not must not read alike. `null` on `audio` and `transition`, which draw nothing.
    pub blend: Option<String>,
    /// `motion_blur` as the file writes it (ADR-0155 §5). Absent on an element without it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motion_blur: Option<Value>,
    /// `moving` or `still` at the frame containing the instant: whether that frame paints
    /// the element's samples and averages them, or paints it once, sharp. The values above
    /// stay those at the instant asked. Absent without the field, and where the project
    /// states no `fps` to place a frame by.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motion: Option<&'static str>,
    /// Every `grain` member's resolved values (ADR-0156 §5): its position in `effects`, the
    /// static `seed`, `size` and `mono`, `amount` at the instant, and the local frame its
    /// draw is keyed on at the frame holding the instant. Absent on an element with none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grain: Option<Vec<Value>>,
    /// **Offset into source** — where in the source file this instant plays, for `audio`
    /// and `video`. `source_start` plus how far `speed` has advanced playback, or the
    /// `overrun` position past the as-played duration (ADR-0020); on a `video` carrying
    /// `source_time`, the curve's millisecond at the instant (ADR-0157). `null` on every other
    /// type, for the same reason `layer` is `null` on an element with no anchor: nothing
    /// declared it.
    pub source_offset: Option<i64>,
    pub source_offset_unresolved: Option<String>,
    /// Where the pass of the source this instant belongs to began: the timeline instant
    /// that played `source_start`, and `source_start` — the element's own `start` before a
    /// loop wraps, the instant the current pass began after one, and `None` while
    /// `overrun: "hold"` holds `source_end` or wherever `source_offset` is `None`.
    ///
    /// Not part of the answer (ADR-0141): it is how `render`'s feeds keep the render's own
    /// offsets, since a feed measured from wherever it opened would round differently. Read
    /// off the same arithmetic as `source_offset`, so the two cannot disagree.
    #[serde(skip)]
    pub(crate) source_origin: Option<(i64, i64)>,
    /// A remapped `video`'s readings (ADR-0157 §5), derived and never accepted as input:
    /// `source_time` rounded as the painter rounds it, and the `rate` the viewer sees. Absent
    /// on every element that does not carry `source_time`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derived: Option<Derived>,
    /// **The crop rectangle** — which part of the *source file's own pixels* survive onto
    /// the screen, in source pixel space, for a raster element carrying `cover`/`contain`
    /// and a `clip` (ADR-0013, ADR-0015). `null` where the element carries no raster
    /// source, or names `literal`, which claims no derivation to compute against.
    pub crop: Option<Rect>,
    pub crop_unresolved: Option<String>,
    /// **The ink box** — a `text` element's rendered extent, stroke included, in absolute
    /// frame pixels: the tight rectangle ADR-0011 measured overstating by 1.25×–1.48× under
    /// the nominal `size × line_height` reading. See [`geometry::ink_box`] for what this
    /// refuses on.
    pub ink_box: Option<InkBox>,
    pub ink_box_unresolved: Option<String>,
    /// Where a running `wipe`, `slide` or `push` has put this element (ADR-0150): moved
    /// outside its own transform, or cut to one side of a wipe's edge. `null` where no such
    /// transition bridges it at this instant — a crossfade changes only opacity, which this
    /// view reads as declared. Without it, an element mid-slide would read as sitting where
    /// its `x` and `y` say, which is not where the frame shows it.
    pub transition: Option<Moved>,
    /// A staggered `text` element's summary (ADR-0151 §5): `by`, the unit count, and the
    /// stagger window. Absent on every element with no `units` block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stagger: Option<crate::units::Stagger>,
    /// Every unit of a staggered `text` element, uncapped, with its delay, which of its
    /// lists a run overrides, the units it moves with, and its pose at the instant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<Vec<crate::units::UnitRow>>,
    /// A `path`'s resolved vertices with their **absolute** control points — `at`,
    /// `at + in` and `at + out` — in box pixels from the declared box's top-left corner
    /// (ADR-0154 §6), so a containment finding reads without adding offsets up. The
    /// resolved `points` in `values` keeps the document's relative form. Absent on every
    /// other type, and where `points` does not resolve.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Vec<VertexAt>>,
    /// prototype(#750, ADR-0158 §7): a path's inset `m` and reach factor `k` with its source;
    /// on any dashed shape the resolved `stroke_dash_offset`, raw, and an informative outline
    /// length from the painter's own path measure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<Value>,
    /// prototype(#760, ADR-0160 §7): only on an element carrying a trim field — the three
    /// trim values raw (before the overshoot clamp, the offset unwrapped) and the window
    /// drawn: `[a, b]`, `empty` or `full`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trim: Option<Value>,
}

/// prototype(#760): the `trim` block of `query --at`. It also rewrites the `trim_start` and
/// `trim_end` rows of `values` to the raw value, so the text line matches what was written.
fn trim_of(element: &Value, instant: i64, values: &mut [Resolved]) -> Option<Value> {
    use montagent_render::canvas::{Drawn, trim_window};
    let t = (i128::from(instant), 1);
    let trim = crate::verbs::frame::trim_of(element, t)?;
    let raw = |key: &str| crate::animatable::number_unclamped(element, key, t.0, t.1);
    for row in values.iter_mut() {
        if matches!(row.property.as_str(), "trim_start" | "trim_end") {
            if let Some(v) = raw(&row.property) {
                row.value = Some(json!(v));
            }
        }
    }
    let six = |v: f64| {
        let s = format!("{:.6}", v);
        let s = s.trim_end_matches('0').trim_end_matches('.');
        if s.is_empty() || s == "-" { "0".to_string() } else { s.to_string() }
    };
    let drawn = match trim_window(&trim) {
        Drawn::Empty => "empty".to_string(),
        Drawn::Full => "full".to_string(),
        Drawn::Window(a, b) => format!("[{}, {}]", six(a), six(b)),
    };
    let mut out = serde_json::Map::new();
    for key in ["trim_start", "trim_end", "trim_offset"] {
        if let Some(v) = raw(key) {
            out.insert(key.into(), json!(v));
        }
    }
    out.insert("drawn".into(), json!(drawn));
    Some(Value::Object(out))
}

/// prototype(#750): the `stroke` block of `query --at`.
fn stroke_of(element: &Value, kind: Option<&str>, instant: i64) -> Option<Value> {
    use montagent_render::canvas::{Shape, outline_length, path_outline, shape_outline};
    if !matches!(kind, Some("path" | "rect" | "ellipse")) || element.get("stroke").is_none() {
        return None;
    }
    let mut out = serde_json::Map::new();
    if kind == Some("path") {
        let reach = crate::animatable::path_reach(element);
        out.insert("inset".into(), json!(reach.inset));
        out.insert("reach_factor".into(), json!(reach.k));
        out.insert("reach_source".into(), json!(reach.source));
    }
    if element.get("stroke_dash").is_some() {
        let t = (i128::from(instant), 1);
        let offset =
            crate::animatable::number_read(element, "stroke_dash_offset", t.0, t.1, 0.0);
        out.insert("dash_offset".into(), json!(offset));
        let stroke_width =
            crate::animatable::number_read(element, "stroke_width", t.0, t.1, 0.0) as f32;
        let side = |key| crate::animatable::number_read(element, key, t.0, t.1, 0.0) as f32;
        let path = match kind {
            Some("path") => {
                let Ok(crate::animatable::Resolved::Points(vertices)) =
                    crate::animatable::at(element, "points", instant)?
                else {
                    return Some(Value::Object(out));
                };
                let closed = element.get("closed").and_then(Value::as_bool).unwrap_or(false);
                Some(path_outline(&crate::verbs::frame::outline_of(&vertices, closed)))
            }
            Some("rect") => shape_outline(
                Shape::Rect {
                    radius: f64::from(side("radius")),
                },
                side("width"),
                side("height"),
                stroke_width,
            ),
            _ => shape_outline(Shape::Ellipse, side("width"), side("height"), stroke_width),
        };
        if let Some(path) = path {
            out.insert(
                "outline_length".into(),
                json!((outline_length(&path) * 100.0).round() / 100.0),
            );
            out.insert(
                "outline_length_note".into(),
                json!("informative, not a contract: the painter's own path measure"),
            );
        }
    }
    Some(Value::Object(out))
}

/// What `query --at` derives for a remapped `video` (ADR-0157 §5).
#[derive(Debug, Clone, Serialize)]
pub struct Derived {
    /// The source millisecond shown at the instant: `source_time` resolved and rounded
    /// half-up, the number the painter decodes at ([`crate::remap::source_ms`]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_time: Option<i64>,
    /// The change in that millisecond to the next frame instant, over the frame interval,
    /// signed, with `×`: `-0.500×` plays backwards at half rate, `0.000×` is frozen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate: Option<String>,
    /// Why one of the two is missing, where one is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unresolved: Option<String>,
}

impl Derived {
    fn of(document: &Loose, element: &Value, instant: i64) -> Derived {
        let source_time = crate::remap::source_ms(element, instant);
        let fps = document
            .value()
            .get("fps")
            .and_then(Value::as_i64)
            .filter(|fps| *fps > 0)
            .ok_or_else(|| "the project states no `fps` to find the next frame by".to_string());
        let rate = fps.and_then(|fps| crate::remap::rate(element, instant, fps));
        Derived {
            unresolved: source_time.as_ref().err().or(rate.as_ref().err()).cloned(),
            source_time: source_time.ok(),
            rate: rate.ok(),
        }
    }
}

/// A `path`'s resolved vertices, each handle made absolute by adding its own vertex.
fn absolute_vertices(element: &Value, instant: i64) -> Option<Vec<VertexAt>> {
    let Ok(crate::animatable::Resolved::Points(vertices)) =
        crate::animatable::at(element, "points", instant)?
    else {
        return None;
    };
    let absolute =
        |at: [f64; 2], offset: Option<[f64; 2]>| offset.map(|[dx, dy]| [at[0] + dx, at[1] + dy]);
    Some(
        vertices
            .into_iter()
            .map(|vertex| VertexAt {
                arriving: absolute(vertex.at, vertex.arriving),
                out: absolute(vertex.at, vertex.out),
                at: vertex.at,
            })
            .collect(),
    )
}

/// What a running `wipe`, `slide` or `push` does to one element's geometry (ADR-0150).
#[derive(Debug, Clone, Serialize)]
pub struct Moved {
    /// Whole frame-space pixels the element is moved by, `[dx, dy]`.
    pub offset: [i64; 2],
    /// The frame-space rectangle a wipe cuts it to, which may be empty. `null` for none.
    pub cut: Option<Rect>,
    /// The element's visible box after the offset and the cut — the rectangle `NOT
    /// COVERED` reads. `null` where it has none: a rotated element (answered in rectangles
    /// only), no readable box, or a cut that leaves nothing.
    #[serde(rename = "box")]
    pub visible: Option<Rect>,
}

/// One property's value at the instant.
#[derive(Debug, Clone, Serialize)]
pub struct Resolved {
    /// The key, as the document spells it.
    pub property: String,
    /// Whether the document writes this property as a keyframe list. Stated rather than
    /// inferred: `scale [1.0, 1.0]` at an instant is the same row whether the element holds
    /// still or is one millisecond into a fifteen-second ramp, and which of those it is
    /// changes what an author does next.
    pub animated: bool,
    /// The resolved value — **a number, a pair of them, a colour, or a gradient**, never the
    /// records. A colour or a gradient is the literal that pastes back (ADR-0146, ADR-0149).
    ///
    /// A resolved `x` is a number even where the document states it as an integer and
    /// nothing animates it. Two spellings for one value would put a shape test in every
    /// consumer, which is what ADR-0012 retired the `scale` union for; and an integer here
    /// would publish a rounding rule ADR-0035 says does not exist.
    pub value: Option<Value>,
    /// Why there is no value, where there is none — a property the format cannot read at
    /// all, or a keyframe list that does not fit the schema. The fact is `validate`'s to
    /// judge; this says only that the view could not answer.
    pub unresolved: Option<String>,
}

/// Answer `--at`, opening a probing [`Session`] itself where the project needs one.
///
/// Kept in this file rather than in `query/mod.rs`: `neither_mode_reaches_for_a_resolver_
/// the_stack_or_the_disk` (`tests/query.rs`) is the structural form of *"`cuts` and
/// `predicate` never reach for anything beyond the document"*, checked by grepping the
/// dispatcher's own source for `crate::media`. Opening the session in `mod.rs` instead
/// would make that true only by discipline — this file is the one the ADR already
/// excepts, being the mode `--at` alone resolves through.
pub fn answer(document: &Loose, instant: i64) -> Result<At, String> {
    // Narrower than `crate::verbs::validate`'s own precondition, deliberately: an
    // audio-only project — or a raster element with no `clip` — has nothing the crop
    // rectangle could ever ask a probe about (`crop_for` refuses before it reaches
    // `Session::probe` either way), and answering those without `ffmpeg`/`ffprobe` on
    // `PATH` is the whole of what this module's own doc claims: "every other component
    // answers in full without one."
    if !document.elements().any(|element| {
        matches!(
            element.get("type").and_then(Value::as_str),
            Some("image" | "video")
        ) && element.get("clip").is_some()
    }) {
        return Ok(at(document, instant, None));
    }
    match Session::open() {
        Ok(mut session) => {
            session.begin_run();
            Ok(at(document, instant, Some(&mut session)))
        }
        // No `ffmpeg`/`ffprobe`: the same exit 70 `validate` gives the identical condition
        // (ADR-0011), rather than a partial answer that silently drops the one component
        // that needed a probe.
        Err(missing) => Err(missing.reason()),
    }
}

/// Build the resolved stack at `instant`.
///
/// `session` opens the fourth `#210` component: a raster element's crop rectangle needs
/// the source's real dimensions, which no document field carries (ADR-0013's own worked
/// example). `None` is legal and answers every other component in full — it is what a
/// caller with no `ffmpeg`/`ffprobe`, or a project with no raster source at all, passes —
/// and every crop that would have needed it comes back `crop_unresolved` instead of
/// silently absent.
pub fn at(document: &Loose, instant: i64, session: Option<&mut Session>) -> At {
    build(document, instant, session, Detail::Full)
}

/// How much of the view one caller needs.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Detail {
    /// Every component: the stack, its geometry, and `NOT COVERED`.
    Full,
    /// The stack alone — who is present, in painter's order, with the offset into each
    /// source — and none of the derived geometry.
    Presence,
}

/// The presence set at `instant`, in painter's order, and nothing derived from it.
///
/// What `render` walks on every frame. It is [`at`] with the geometry left out rather than
/// a second traversal of the tracks, because the one thing `frame`'s painter insists on is
/// that draw order is decided in exactly one place — *"a second ordering here, however
/// carefully written, would be a second place draw order could be decided"* — and a render
/// that sorted its own stack would be that second place. What is left out is what a
/// painter never reads: the crop rectangle (the canvas applies `clip` itself), the ink box
/// (a text measurement, which re-registers every font of every text element on every
/// call), and `NOT COVERED`. Their fields come back `unresolved` with a sentence saying so,
/// never silently empty.
pub(crate) fn presence(document: &Loose, instant: i64) -> At {
    build(document, instant, None, Detail::Presence)
}

fn build(document: &Loose, instant: i64, mut session: Option<&mut Session>, detail: Detail) -> At {
    let stack = Stack::of(document);
    let mut present: Vec<Present> = Vec::new();
    let mut unplaced: Vec<String> = Vec::new();
    let frame = frame_dimensions(document);

    // Every visual element's own drawn rectangle, `clip`-intersected where one applies —
    // the input `NOT COVERED` unions over. Built in the same pass as `present` rather than
    // a second traversal, since it needs exactly the same half-open presence test.
    let mut covering: Vec<Rect> = Vec::new();
    let mut not_covered_unresolved: Option<String> = None;

    // The running transitions, resolved once for the instant through the same
    // `crate::transition` `frame`'s painter reads (ADR-0150), so a bridged element's box here
    // is where the picture puts it. Not in a presence-only view: a painter resolves its own.
    let running = match (detail, frame) {
        (Detail::Full, Some(frame)) => crate::transition::running_at(document, instant, frame),
        _ => Vec::new(),
    };

    for (index, (track, element)) in document.elements_in_tracks().enumerate() {
        let named = Named::of(element, track);
        let name = named.called(index);
        let (Some(start), Some(end)) = (
            element.get("start").and_then(Value::as_i64),
            element.get("end").and_then(Value::as_i64),
        ) else {
            unplaced.push(name);
            continue;
        };
        // Half-open, so an element ending at the instant is already out of the set and one
        // starting there is already in it (ADR-0005). An inverted or empty range contains no
        // instant at all, which this same test gives for free.
        if !(start <= instant && instant < end) {
            continue;
        }

        let (layer, layer_unresolved) = match &named.id {
            // Resolution is by `id`, so an element without one has no row in the stack's
            // index — a fact about the document, and `validate`'s to report (ADR-0019).
            None => (None, Some("the element states no `id`".to_string())),
            Some(id) => match stack.layer_of(id) {
                Ok(layer) => (Some(layer), None),
                Err(unresolved) => (None, Some(why(unresolved))),
            },
        };

        let kind = named.kind.as_deref();
        let bridge = match &named.id {
            Some(id) => crate::transition::bridge_of(&running, id),
            None => crate::transition::Bridge::default(),
        };

        // An invisible element (a resolved `opacity` of exactly `0`) contributes nothing
        // to `NOT COVERED` either way, so its rotation — which would otherwise force the
        // whole computation to refuse — is not this instant's concern. One call into
        // `drawn_rect`, its result reused for both the refusal check and the rectangle
        // itself, rather than computing the same footprint twice.
        if detail == Detail::Full
            && covers_the_frame(kind)
            && geometry_number_opacity(element, instant) != 0.0
            && let Some(frame) = frame
        {
            match geometry::bridged_visible_rect(element, instant, frame, &bridge) {
                Some(Err(NotAxisAligned::Rotated(degrees))) => {
                    not_covered_unresolved.get_or_insert_with(|| {
                        format!(
                            "`{name}`'s resolved `rotation` is {degrees}°; `NOT COVERED` is \
                             answered in rectangles only"
                        )
                    });
                }
                Some(Ok(visible)) => covering.push(visible),
                // No readable box, or a `clip` that excludes the element entirely: either
                // way it paints nothing and covers nothing.
                None => {}
            }
        }

        let (source_offset, source_offset_unresolved, source_origin) =
            source_offset(element, kind, start, instant);
        let (crop, crop_unresolved) = match frame {
            _ if detail == Detail::Presence => (None, Some(NOT_DERIVED.to_string())),
            Some(frame) => crop_for(
                document,
                element,
                kind,
                instant,
                frame,
                session.as_deref_mut(),
            ),
            None => (
                None,
                Some("the project carries no legal `frame`".to_string()),
            ),
        };
        let (ink_box, ink_box_unresolved) = match (kind, frame) {
            (Some("text"), _) if detail == Detail::Presence => {
                (None, Some(NOT_DERIVED.to_string()))
            }
            (Some("text"), Some(frame)) => {
                match geometry::ink_box(document, element, instant, frame) {
                    // Moved with the element by a slide or push (ADR-0150).
                    Ok(ink_box) => (
                        Some(InkBox {
                            x: ink_box.x + bridge.offset.0 as f64,
                            y: ink_box.y + bridge.offset.1 as f64,
                            ..ink_box
                        }),
                        None,
                    ),
                    Err(reason) => (None, Some(reason)),
                }
            }
            (Some("text"), None) => (
                None,
                Some("the project carries no legal `frame`".to_string()),
            ),
            _ => (None, None),
        };

        let blend = blend_word(element, kind);
        let (motion_blur, motion) = motion_of(document, element, kind, instant);
        let grain = grain_of(document, element, instant);
        let (stagger, units) = match kind {
            Some("text") if detail != Detail::Presence => {
                match crate::units::report(document, element, instant) {
                    Some((stagger, units)) => (Some(stagger), Some(units)),
                    None => (None, None),
                }
            }
            _ => (None, None),
        };
        let path = match kind {
            Some("path") => absolute_vertices(element, instant),
            _ => None,
        };
        let stroke = stroke_of(element, kind, instant);
        let mut values_now = values(element, instant);
        present.push(Present {
            stroke,
            stagger,
            units,
            named,
            start,
            end,
            layer,
            layer_unresolved,
            trim: trim_of(element, instant, &mut values_now),
            values: values_now,
            blend,
            motion_blur,
            motion,
            grain,
            source_offset,
            source_offset_unresolved,
            source_origin,
            derived: (detail == Detail::Full && crate::remap::is_remapped(element))
                .then(|| Derived::of(document, element, instant)),
            crop,
            crop_unresolved,
            ink_box,
            ink_box_unresolved,
            transition: match (bridge.moves_or_cuts(), frame) {
                (true, Some(frame)) => Some(Moved {
                    offset: [bridge.offset.0, bridge.offset.1],
                    cut: bridge.cut,
                    visible: geometry::bridged_visible_rect(element, instant, frame, &bridge)
                        .and_then(Result::ok),
                }),
                _ => None,
            },
            path,
        });
    }

    // Painter's order: back to front. Stable, so a tie keeps document order — which the file
    // makes no promise about and nothing checks (ADR-0060), and which is the whole of what
    // this view is permitted to do with one. An element whose layer did not resolve sorts
    // last rather than at zero: it has no place in the stack, and putting it at one would be
    // the invented order ADR-0060 refuses.
    present.sort_by_key(|element| (element.layer.is_none(), element.layer.unwrap_or_default()));

    if detail == Detail::Presence {
        not_covered_unresolved = Some(NOT_DERIVED.to_string());
    }
    let not_covered = match (frame, &not_covered_unresolved) {
        (Some(frame), None) => geometry::not_covered(frame, &covering),
        _ => Vec::new(),
    };

    At {
        at: instant,
        stack: present,
        unplaced,
        not_covered,
        not_covered_unresolved,
    }
}

/// `blend` as the file spells it, `normal` where it is omitted, and `None` on a type that
/// draws nothing. A value that is not one of the five words is the schema check's error,
/// and this view reports it as the `normal` the painter falls back to.
fn blend_word(element: &Value, kind: Option<&str>) -> Option<String> {
    if !covers_the_frame(kind) {
        return None;
    }
    let written = element
        .get("blend")
        .and_then(|value| serde_json::from_value::<crate::model::Blend>(value.clone()).ok());
    Some(written.unwrap_or_default().as_str().to_string())
}

/// `motion_blur` as written, and whether the frame containing `instant` paints the element
/// `moving` or `still` (ADR-0155 §5) — decided as the painter decides it, at that frame's
/// sample instants.
fn motion_of(
    document: &Loose,
    element: &Value,
    kind: Option<&str>,
    instant: i64,
) -> (Option<Value>, Option<&'static str>) {
    let Some(written) = element.get("motion_blur") else {
        return (None, None);
    };
    let fps = document
        .value()
        .get("fps")
        .and_then(Value::as_i64)
        .filter(|fps| *fps > 0);
    let motion = match (crate::motion_blur::of(element), fps) {
        (Some(blur), Some(fps)) if covers_the_frame(kind) => {
            let frame = crate::motion_blur::frame_containing(instant, fps);
            let instants =
                crate::motion_blur::sample_instants(frame, fps, blur.shutter, blur.samples);
            Some(match crate::motion_blur::still(element, &instants) {
                true => "still",
                false => "moving",
            })
        }
        _ => None,
    };
    (Some(written.clone()), motion)
}

/// Every `grain` member's resolved values, as the painter reads them at `instant`: through
/// the one resolving function, with the local frame the painter keys the draw on.
fn grain_of(document: &Loose, element: &Value, instant: i64) -> Option<Vec<Value>> {
    let fps = document
        .value()
        .get("fps")
        .and_then(Value::as_i64)
        .filter(|fps| *fps > 0)
        .unwrap_or(1);
    let frame = crate::grain::local_frame(element, instant, fps);
    let count = element.get("effects").and_then(Value::as_array)?.len();
    let grains: Vec<Value> = (0..count)
        .filter_map(|index| {
            match crate::verbs::frame::effect_of(element, index, (i128::from(instant), 1), frame)? {
                montagent_render::canvas::Effect::Grain {
                    seed,
                    amount,
                    size,
                    mono,
                    frame,
                } => Some(
                    serde_json::json!({"effect": index, "seed": seed, "amount": amount,
                                             "size": size, "mono": mono, "frame": frame}),
                ),
                _ => None,
            }
        })
        .collect();
    (!grains.is_empty()).then_some(grains)
}

/// The one sentence every geometry field carries in a presence-only view.
const NOT_DERIVED: &str = "not derived: this view was built for painting, and a painter reads none of the \
                           geometry";

/// The project's own `frame`, or `None` where it is missing or malformed — `validate`'s
/// fact to report, not this view's to guess at.
///
/// `pub(crate)` because the renderer asks the identical question: `frame` needs the surface
/// to be the size the caption was computed against, and a second reading of the same three
/// keys is a second place a malformed `frame` could be interpreted differently.
pub(crate) fn frame_dimensions(document: &Loose) -> Option<(i64, i64)> {
    let frame = document.value().get("frame")?;
    Some((
        frame.get("width")?.as_i64()?,
        frame.get("height")?.as_i64()?,
    ))
}

fn geometry_number_opacity(element: &Value, instant: i64) -> f64 {
    let Some(written) = element.get("opacity") else {
        return 1.0;
    };
    let Ok(animatable) = serde_json::from_value::<Animatable<f64>>(written.clone()) else {
        return 1.0;
    };
    resolve::at(&animatable, instant).unwrap_or(1.0)
}

/// **Offset into source** (ADR-0020) — computed from the document's own declared
/// `source_start`/`source_end`/`speed`/`overrun` alone, never re-probed. `validate`
/// already judges whether that declared range agrees with the file on disk
/// (ADR-0006/ADR-0020's own invariant); recomputing that agreement here would be the
/// second implementation of one judgment ADR-0006 forbids carrying twice. So a document
/// whose declared range disagrees with the real file answers here exactly as it is
/// written, and `validate`'s finding is where the disagreement is reported.
pub(crate) fn source_offset(
    element: &Value,
    kind: Option<&str>,
    start: i64,
    instant: i64,
) -> (Option<i64>, Option<String>, Option<(i64, i64)>) {
    if !matches!(kind, Some("audio") | Some("video")) {
        return (None, None, None);
    }
    let unresolved = |reason: &str| (None, Some(reason.to_string()), None);
    // ADR-0157: a remapped video's offset is its curve at the instant, through the one
    // resolution function. It has no pass to measure from: every frame is decided alone.
    if crate::remap::is_remapped(element) {
        return match crate::remap::source_ms(element, instant) {
            Ok(ms) => (Some(ms), None, None),
            Err(reason) => unresolved(&reason),
        };
    }
    let (Some(source_start), Some(source_end)) = (
        element.get("source_start").and_then(Value::as_i64),
        element.get("source_end").and_then(Value::as_i64),
    ) else {
        return unresolved("the element carries no integer `source_start`/`source_end`");
    };
    let source_span = source_end - source_start;
    if source_span <= 0 {
        return unresolved("`source_end` does not exceed `source_start`");
    }

    let speed = match element.get("speed") {
        None | Some(Value::Null) => Decimal::of(&serde_json::Number::from(1)),
        Some(value) => value.as_number().and_then(Decimal::of),
    };
    let Some(speed) = speed.filter(|d| d.is_positive()) else {
        return unresolved(
            "`speed` is not a positive number, which is `validate`'s finding to make",
        );
    };

    let Some(played) = exact::played_ms(source_span, speed) else {
        return unresolved("the as-played duration could not be computed");
    };

    let elapsed = instant - start;
    if elapsed < played {
        return match exact::source_advance(elapsed, speed) {
            Some(advance) => (
                Some(source_start + advance),
                None,
                Some((start, source_start)),
            ),
            None => unresolved("the source position could not be computed"),
        };
    }

    match element.get("overrun").and_then(Value::as_str) {
        // A hold is one offset for the rest of the element, so it has no pass to measure
        // from.
        Some("hold") => (Some(source_end), None, None),
        Some("loop") => {
            let cycle = (elapsed - played) % played;
            match exact::source_advance(cycle, speed) {
                Some(advance) => (
                    Some(source_start + advance),
                    None,
                    Some((instant - cycle, source_start)),
                ),
                None => unresolved("the looped source position could not be computed"),
            }
        }
        _ => unresolved(
            "the timeline range outruns the as-played duration and the element declares no \
             `overrun`, which is `validate`'s speed-mismatch finding to report",
        ),
    }
}

/// **The crop rectangle** (ADR-0013, ADR-0015) — `None`/`None` on every element that is
/// not a raster source, `literal`, or missing a `clip`; a reason on every refusal, which is
/// this function's whole job beside the one call into [`geometry::crop_rectangle`].
fn crop_for(
    document: &Loose,
    element: &Value,
    kind: Option<&str>,
    instant: i64,
    frame: (i64, i64),
    session: Option<&mut Session>,
) -> (Option<Rect>, Option<String>) {
    if !matches!(kind, Some("image") | Some("video")) {
        return (None, None);
    }
    let Some(source) = element.get("source").and_then(Value::as_str) else {
        return (None, Some("the element carries no `source`".to_string()));
    };
    // `fit` itself is not read: ADR-0015's load-bearing sentence is that the declared
    // rect is authoritative at render *regardless* of which rule it claims to derive
    // from, so the crop rectangle is the same computation under `cover`, `contain` and
    // `literal` alike. What every one of them still needs is a `clip` to crop against.
    let Some(clip) = clip_rect(element) else {
        return (
            None,
            Some(
                "the element carries no `clip`, so there is no aperture to crop against"
                    .to_string(),
            ),
        );
    };
    let (Some(width), Some(height)) = (
        element.get("width").and_then(Value::as_i64),
        element.get("height").and_then(Value::as_i64),
    ) else {
        return (
            None,
            Some("the element carries no integer `width`/`height`".to_string()),
        );
    };

    let drawn = match geometry::drawn_rect(element, instant, frame) {
        None => {
            return (
                None,
                Some("the element's drawn rectangle could not be computed".to_string()),
            );
        }
        Some(Err(NotAxisAligned::Rotated(degrees))) => {
            return (
                None,
                Some(format!(
                    "its resolved `rotation` is {degrees}°; the crop rectangle is not \
                     derived for a rotated element"
                )),
            );
        }
        Some(Ok(rect)) => rect,
    };

    let Some(session) = session else {
        return (
            None,
            Some("no probing session is open for this run".to_string()),
        );
    };
    let base = crate::checks::project_dir(document);
    let outcome = match session.probe(&Source::resolve(source, &base)) {
        Ok(outcome) => outcome,
        Err(_missing) => {
            return (None, Some("ffmpeg/ffprobe is not on `PATH`".to_string()));
        }
    };
    let Outcome::Probed(probe) = &outcome else {
        return (
            None,
            Some(
                "the source could not be probed, so its real dimensions are not known".to_string(),
            ),
        );
    };
    let Some(dimensions) = probe.dimensions else {
        return (
            None,
            Some("the probe established no dimensions for this source".to_string()),
        );
    };

    match geometry::crop_rectangle(
        (width, height),
        drawn,
        clip,
        (i64::from(dimensions.width), i64::from(dimensions.height)),
    ) {
        Some(rect) => (Some(rect), None),
        None => (
            None,
            Some(
                "`clip` excludes the element entirely; none of the source reaches the screen"
                    .to_string(),
            ),
        ),
    }
}

/// Every animatable property this element declares, resolved through the one resolving
/// function ([`crate::animatable::Declared::read`]) — in the order the schema declares them,
/// never the order the file happens to write them in: the element's own, then every
/// `effects` member's parameters, each named by its member's position with the member's name
/// in the text, `effects[1].radius (blur)` (ADR-0146 §4). The list is the schema's
/// (ADR-0146): a property joins this answer by being typed as animatable, never by being
/// added here.
///
/// A colour is printed as the literal that can be pasted back: six digits when opaque,
/// `#00000000` at alpha 0.
fn values(element: &Value, instant: i64) -> Vec<Resolved> {
    let own = crate::animatable::of_element(element);
    crate::animatable::declared(element)
        .into_iter()
        // A typed element answers for its own type's properties; a key another type has is
        // the schema check's to name.
        .filter(|declared| {
            declared.effect.is_some()
                || own.is_empty()
                || own.iter().any(|property| property.name == declared.path)
        })
        // A gradient's parameters are on the list as nested paths (`fill.angle`), and are
        // printed inside the paint they belong to, resolved and fixed together (ADR-0149 §6).
        .filter(|declared| declared.effect.is_some() || !declared.path.contains('.'))
        .map(|declared| {
            let property = declared.path.as_str();
            let animated = declared.records().is_some()
                || (declared.property.kind == crate::animatable::Kind::Paint
                    && crate::animatable::nested_keyed(declared.owner, property));
            match declared.at(instant) {
                Ok(value) => match serde_json::to_value(value) {
                    Ok(value) => Resolved {
                        property: property.to_string(),
                        animated,
                        value: Some(value),
                        unresolved: None,
                    },
                    // A value JSON cannot carry — an infinity or a NaN reached by
                    // interpolating one.
                    Err(e) => unreadable(property, animated, e.to_string()),
                },
                // The format's own reader, which is also where ADR-0038's positional `ease`
                // rule is enforced — so a keyframe list missing an `ease` on a later record
                // arrives here as unreadable rather than as a value this module had to
                // invent a default to produce.
                Err(Unreadable::Schema(reason)) => {
                    unreadable(property, declared.owner[declared.key()].is_array(), reason)
                }
                Err(Unreadable::Unresolvable(unresolvable)) => {
                    unreadable(property, animated, unanswered(unresolvable))
                }
            }
        })
        .collect()
}

fn unreadable(property: &str, animated: bool, reason: String) -> Resolved {
    Resolved {
        property: property.to_string(),
        animated,
        value: None,
        unresolved: Some(reason),
    }
}

/// Why the resolver could not answer, in a sentence.
///
/// It refuses rather than holding the previous value, and the sentence says which document
/// state produced the refusal — because ADR-0011 asks this verb for resolved values, and a
/// number the document does not determine, printed in the column resolved values live in,
/// is worse than no number at all.
fn unanswered(unresolvable: Unresolvable) -> String {
    match unresolvable {
        Unresolvable::Empty => "its keyframe list carries no records".to_string(),
        Unresolvable::NoEase { t } => format!(
            "the record at `t` {t} states no `ease`, so how the value travels into it is not \
             in the document — the list is written out of clock order, and `ease` is required \
             by position in the array",
        ),
    }
}

/// Why an element's layer is not an integer, in a sentence.
///
/// The wording is this view's own. [`crate::checks::anchor`] answers the same question with
/// findings carrying stable codes and repairs, which is what a report is for; a view says
/// what it could not answer and sends the reader to `validate` for the verdict.
fn why(unresolved: Unresolved<'_>) -> String {
    match unresolved {
        Unresolved::NoSuchElement => "the project carries no element with this `id`".to_string(),
        Unresolved::MissingTarget(target) => {
            format!("its anchor names `{target}`, which no element in the project carries")
        }
        Unresolved::SelfReference(target) => {
            format!("its anchor names itself (`{target}`)")
        }
        Unresolved::ChainedTarget(target) => format!(
            "its anchor names `{target}`, whose own `layer` is not an integer — an anchor \
             resolves in exactly one hop"
        ),
        Unresolved::Unstated => {
            "neither the element nor its track states an integer `layer`".to_string()
        }
        Unresolved::Malformed => {
            "its `layer` is neither an integer nor an anchor object".to_string()
        }
    }
}
