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
//! and `NOT COVERED`. [#210](https://github.com/MBehtemam/Montaget/issues/210) is what
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
//!   [`montaget_text::measure`] `measure` the verb calls (#205), never a second
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
//! [#269](https://github.com/MBehtemam/Montaget/issues/269) carries them for ratification,
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
use serde_json::Value;

use crate::exact::{self, Decimal};
use crate::media::Source;
use crate::media::probe::Outcome;
use crate::media::session::Session;
use crate::model::{Animatable, Scale};
use crate::permissive::Loose;
use crate::resolve::{self, Interpolate, Unresolvable};
use crate::stack::{Stack, Unresolved};

use super::Named;
use super::geometry::{self, InkBox, NotAxisAligned, Rect};

/// The resolved stack at one instant.
#[derive(Debug, Clone, Serialize)]
pub struct At {
    /// The instant asked about, in absolute milliseconds on the project's one clock.
    pub at: i64,
    /// The presence set, in painter's order: ascending resolved layer, back to front.
    ///
    /// Audio included, like the cut list's — *"audio is an element like any other; nothing
    /// owns it"* (ADR-0001), and a caller wanting only the visual stack filters one field.
    /// The departure from ADR-0011's word *"on-screen"* is the same one, argued in the same
    /// place ([#250](https://github.com/MBehtemam/Montaget/issues/250)).
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
    /// **Offset into source** — where in the source file this instant plays, for `audio`
    /// and `video`. `source_start` plus how far `speed` has advanced playback, or the
    /// `overrun` position past the as-played duration (ADR-0020). `null` on every other
    /// type, for the same reason `layer` is `null` on an element with no anchor: nothing
    /// declared it.
    pub source_offset: Option<i64>,
    pub source_offset_unresolved: Option<String>,
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
    /// The resolved value — **always a number, or a pair of them**, never the records.
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

/// The value shape a property is written in — the one table mapping a key to the type the
/// model declares for it.
///
/// It exists because this mode reads the permissive tree (the representation that always
/// exists, and the one every other check and view already reads) rather than the strict
/// model, so the per-property type cannot come from a struct field. What it does *not*
/// duplicate is any rule: the reading is [`Animatable`]'s own deserializer, and the
/// resolution is [`crate::resolve`].
#[derive(Clone, Copy)]
enum Shape {
    /// `x`, `y` — absolute integer pixels in the document (ADR-0012).
    Pixels,
    /// `rotation`, `opacity`, `volume` — ratios and angles, floats in the document.
    Ratio,
    /// `scale` — always `[sx, sy]`, never a bare number (ADR-0012).
    Pair,
}

/// Every animated property in the format, in the order the model declares them.
///
/// One list rather than one per element type: the key is spelled the same wherever it
/// appears, and a `volume` on an image is a schema error that `validate` reports rather than
/// something this view has to have an opinion about.
const ANIMATED: [(&str, Shape); 6] = [
    ("x", Shape::Pixels),
    ("y", Shape::Pixels),
    ("scale", Shape::Pair),
    ("rotation", Shape::Ratio),
    ("opacity", Shape::Ratio),
    ("volume", Shape::Ratio),
];

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
pub fn at(document: &Loose, instant: i64, mut session: Option<&mut Session>) -> At {
    let stack = Stack::of(document);
    let mut present: Vec<Present> = Vec::new();
    let mut unplaced: Vec<String> = Vec::new();
    let frame = frame_dimensions(document);

    // Every visual element's own drawn rectangle, `clip`-intersected where one applies —
    // the input `NOT COVERED` unions over. Built in the same pass as `present` rather than
    // a second traversal, since it needs exactly the same half-open presence test.
    let mut covering: Vec<Rect> = Vec::new();
    let mut not_covered_unresolved: Option<String> = None;

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

        // An invisible element (a resolved `opacity` of exactly `0`) contributes nothing
        // to `NOT COVERED` either way, so its rotation — which would otherwise force the
        // whole computation to refuse — is not this instant's concern. One call into
        // `drawn_rect`, its result reused for both the refusal check and the rectangle
        // itself, rather than computing the same footprint twice.
        if covers_the_frame(kind)
            && geometry_number_opacity(element, instant) != 0.0
            && let Some(frame) = frame
        {
            match geometry::drawn_rect(element, instant, frame) {
                Some(Err(NotAxisAligned::Rotated(degrees))) => {
                    not_covered_unresolved.get_or_insert_with(|| {
                        format!(
                            "`{name}`'s resolved `rotation` is {degrees}°; `NOT COVERED` is \
                             answered in rectangles only"
                        )
                    });
                }
                Some(Ok(rect)) => {
                    let visible = match clip_rect(element) {
                        Some(clip) => rect.intersect(clip),
                        None => Some(rect),
                    };
                    if let Some(visible) = visible {
                        covering.push(visible);
                    }
                }
                None => {}
            }
        }

        let (source_offset, source_offset_unresolved) =
            source_offset(element, kind, start, instant);
        let (crop, crop_unresolved) = match frame {
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
            (Some("text"), Some(frame)) => {
                match geometry::ink_box(document, element, instant, frame) {
                    Ok(ink_box) => (Some(ink_box), None),
                    Err(reason) => (None, Some(reason)),
                }
            }
            (Some("text"), None) => (
                None,
                Some("the project carries no legal `frame`".to_string()),
            ),
            _ => (None, None),
        };

        present.push(Present {
            named,
            start,
            end,
            layer,
            layer_unresolved,
            values: values(element, instant),
            source_offset,
            source_offset_unresolved,
            crop,
            crop_unresolved,
            ink_box,
            ink_box_unresolved,
        });
    }

    // Painter's order: back to front. Stable, so a tie keeps document order — which the file
    // makes no promise about and nothing checks (ADR-0060), and which is the whole of what
    // this view is permitted to do with one. An element whose layer did not resolve sorts
    // last rather than at zero: it has no place in the stack, and putting it at one would be
    // the invented order ADR-0060 refuses.
    present.sort_by_key(|element| (element.layer.is_none(), element.layer.unwrap_or_default()));

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

/// The project's own `frame`, or `None` where it is missing or malformed — `validate`'s
/// fact to report, not this view's to guess at.
fn frame_dimensions(document: &Loose) -> Option<(i64, i64)> {
    let frame = document.value().get("frame")?;
    Some((
        frame.get("width")?.as_i64()?,
        frame.get("height")?.as_i64()?,
    ))
}

/// Whether this element type has a frame-space footprint at all — audio and `transition`
/// do not, and are excluded from both `NOT COVERED`'s union and its refusal: a rotated
/// `rect` blocks the computation, but an audio element playing underneath never could.
fn covers_the_frame(kind: Option<&str>) -> bool {
    matches!(kind, Some("image" | "video" | "text" | "rect" | "ellipse"))
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

/// `clip`, as a [`Rect`] — only `image` and `video` carry the field.
fn clip_rect(element: &Value) -> Option<Rect> {
    let clip = element.get("clip")?.as_array()?;
    if clip.len() != 4 {
        return None;
    }
    Some(Rect {
        x: clip[0].as_i64()?,
        y: clip[1].as_i64()?,
        width: clip[2].as_i64()?,
        height: clip[3].as_i64()?,
    })
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
) -> (Option<i64>, Option<String>) {
    if !matches!(kind, Some("audio") | Some("video")) {
        return (None, None);
    }
    let (Some(source_start), Some(source_end)) = (
        element.get("source_start").and_then(Value::as_i64),
        element.get("source_end").and_then(Value::as_i64),
    ) else {
        return (
            None,
            Some("the element carries no integer `source_start`/`source_end`".to_string()),
        );
    };
    let source_span = source_end - source_start;
    if source_span <= 0 {
        return (
            None,
            Some("`source_end` does not exceed `source_start`".to_string()),
        );
    }

    let speed = match element.get("speed") {
        None | Some(Value::Null) => Decimal::of(&serde_json::Number::from(1)),
        Some(value) => value.as_number().and_then(Decimal::of),
    };
    let Some(speed) = speed.filter(|d| d.is_positive()) else {
        return (
            None,
            Some(
                "`speed` is not a positive number, which is `validate`'s finding to make"
                    .to_string(),
            ),
        );
    };

    let Some(played) = exact::played_ms(source_span, speed) else {
        return (
            None,
            Some("the as-played duration could not be computed".to_string()),
        );
    };

    let elapsed = instant - start;
    if elapsed < played {
        return match exact::source_advance(elapsed, speed) {
            Some(advance) => (Some(source_start + advance), None),
            None => (
                None,
                Some("the source position could not be computed".to_string()),
            ),
        };
    }

    match element.get("overrun").and_then(Value::as_str) {
        Some("hold") => (Some(source_end), None),
        Some("loop") => {
            let cycle = (elapsed - played) % played;
            match exact::source_advance(cycle, speed) {
                Some(advance) => (Some(source_start + advance), None),
                None => (
                    None,
                    Some("the looped source position could not be computed".to_string()),
                ),
            }
        }
        _ => (
            None,
            Some(
                "the timeline range outruns the as-played duration and the element declares \
                 no `overrun`, which is `validate`'s speed-mismatch finding to report"
                    .to_string(),
            ),
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

/// Every animated property this element declares, resolved.
fn values(element: &Value, instant: i64) -> Vec<Resolved> {
    ANIMATED
        .iter()
        .filter_map(|(property, shape)| {
            let written = element.get(*property)?;
            Some(match shape {
                Shape::Pixels => resolved::<i64>(property, written, instant),
                Shape::Ratio => resolved::<f64>(property, written, instant),
                Shape::Pair => resolved::<Scale>(property, written, instant),
            })
        })
        .collect()
}

/// One property, read as the format's own type and resolved at the instant.
fn resolved<T>(property: &str, written: &Value, instant: i64) -> Resolved
where
    T: serde::de::DeserializeOwned + Interpolate,
    T::Out: Serialize,
{
    let unreadable = |animated: bool, reason: String| Resolved {
        property: property.to_string(),
        animated,
        value: None,
        unresolved: Some(reason),
    };

    // The format's own reader, which is also where ADR-0038's positional `ease` rule is
    // enforced — so a keyframe list missing an `ease` on a later record arrives here as
    // unreadable rather than as a value this module had to invent a default to produce.
    let animatable: Animatable<T> = match serde_json::from_value(written.clone()) {
        Ok(animatable) => animatable,
        Err(e) => return unreadable(written.is_array(), e.to_string()),
    };
    let animated = matches!(animatable, Animatable::Keyed(_));

    let value = match resolve::at(&animatable, instant) {
        Ok(value) => value,
        Err(unresolvable) => return unreadable(animated, unanswered(unresolvable)),
    };
    match serde_json::to_value(value) {
        Ok(value) => Resolved {
            property: property.to_string(),
            animated,
            value: Some(value),
            unresolved: None,
        },
        // A value JSON cannot carry — an infinity or a NaN reached by interpolating one.
        Err(e) => unreadable(animated, e.to_string()),
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
