//! `R-SOURCE-CUT-POP` — ADR-0033: *"two adjacent elements on one track draw the same
//! source with no gap between them … and an animated property resolves to different
//! values on either side of the cut. The viewer sees one continuous shot; the frame
//! pops."*
//!
//! The discrimination is exactly one field wide, and the fixture is the argument: of its
//! six `photo`-track cuts, four reset `scale` to `1.0` because a *new image* starts its
//! own Ken Burns move, and two reset it across one continuous still. Same track, same
//! source, no gap.
//!
//! ## Why this is not keyed on `group`
//!
//! ADR-0012's group-keyframe-time check answers a different question — *do elements
//! sharing a `group` move together?* — and the fixture shows it cannot stand in for this
//! one. `photo-05-intro`/`photo-05` share `group: item-05`, so that mechanism catches the
//! 3018 pop **by coincidence**; `photo-05-quiz` (`group: quiz`) and `photo-05-loop`
//! (`group: loop-tail`) share none, so it misses the larger pop at 64016 entirely.
//! *"Coverage that depends on whether an author happened to assign a shared `group` … is
//! not coverage."* Nothing in this module reads `group`.
//!
//! ## The wrap
//!
//! ADR-0033 declined the loop seam because nothing in the schema declared one, and
//! predicted the extension would be mechanical once a field did: *"the last element and
//! the first element become an ordinary same-source-adjacent pair, and no new mechanism is
//! needed then either."* ADR-0062 supplies `loop` and confirms the prediction, so the wrap
//! arrives here as one extra *pair* — [`Seam::Wrap`] — and not as one extra rule: the same
//! source test, the same source-time term, the same tolerance table, the same
//! `origin` trigger, the same code. A gap at either edge (`last.end < duration`, or
//! `first.start > 0`) fails the same `A.end == B.start` condition an interior gap fails,
//! so no new gap rule is needed either.
//!
//! A track holding one element that spans the whole project is its own `first` and `last`,
//! and wraps onto itself. ADR-0062 contemplates two elements and neither ADR rules on the
//! degenerate case; it is admitted here because it is the same defect — a Ken Burns move
//! that ends at 1.0064 and restarts at 1.0 pops at the seam whether or not a second
//! element is involved — and excluding it would be a rule the ADRs do not state. Raised
//! as [#285](https://github.com/MBehtemam/Montagent/issues/285).
//!
//! ## What the comparison is
//!
//! `A`'s value **as resolved at `A.end`** against `B`'s **at `B.start`**, through the one
//! resolver ([`crate::resolve`], reached via [`geometry::number`] so that the renderer's
//! own defaults apply). Under ADR-0012's clamping `A.end` is `A`'s last keyframe value
//! whether or not `A` is trimmed short of it; `A` never actually renders at `t = A.end`
//! under the half-open interval, and the comparison is between the limit approaching the
//! cut and the value entering it.
//!
//! `origin` is **not** compared as a value and is its own unconditional trigger: it is the
//! reference frame `x`/`y`/`scale`/`rotation` are read against, so comparing raw numbers
//! across two elements whose frames disagree can both miss a real pop and manufacture a
//! false one. Two elements presenting one continuous shot whose declared reference frames
//! disagree is itself the finding.
//!
//! **One finding per cut, not per property.** ADR-0033 asks for *"per property that fails
//! its tolerance: the resolved out-value, the resolved in-value, and the delta"* and shows
//! one worked line; `detail` carries that list, in the shape [`crate::checks::quantization`]
//! already uses for a finding whose substance is a list. Per-property findings would report
//! one cut three times, and `origin` — whose mismatch has no delta — would have no honest
//! row in a per-property field set. That reading is this check's own and is
//! [#285](https://github.com/MBehtemam/Montagent/issues/285) along with the wrap's
//! degenerate case.
//!
//! **Document-only**: source identity is a comparison of *resolved paths*, which ADR-0033
//! calls a fact derivable from the document, so this check opens no file and needs no
//! session.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::media::Source;
use crate::permissive::Loose;
use crate::report::Report;
use crate::verbs::query::at::frame_dimensions;
use crate::verbs::query::geometry;

/// ADR-0033's tolerance table, per property rather than one epsilon.
///
/// *"A single epsilon does not generalize once the check covers more than `scale` — 0.0005
/// is reasonable headroom against float noise for `scale`, meaningless for `x`/`y` in
/// pixels."* None of these carries the discrimination: every fixture pop clears its column
/// by two or more orders of magnitude, and their only job is refusing to fire on noise.
const SCALE_TOLERANCE: f64 = 0.0005;
const OPACITY_TOLERANCE: f64 = 0.005;
const ROTATION_TOLERANCE: f64 = 0.01;
const PIXEL_TOLERANCE: f64 = 0.5;

/// How many decimals a resolved value is stated to.
///
/// ADR-0033's own worked example — *"`scale` 1.05419 → 1.00000 (Δ −0.05419)"* — pads to
/// five and shows the same width on both sides of the arrow, which is what makes a pop
/// readable at a glance. One width for every property rather than one per unit: a reader
/// comparing an `x` row against a `scale` row should not have to notice that the two are
/// printed differently.
const DECIMALS: usize = 5;

/// One element, as this check reads it — a candidate for either side of a cut.
///
/// Named for the relation rather than for the seam: a `Cut` holding an element would be a
/// second meaning for the word this module's own name already uses for the seam between
/// two of them.
struct Adjacent<'a> {
    id: &'a str,
    element: &'a Value,
    start: i64,
    end: i64,
    /// The `source` exactly as the document spells it — what the finding quotes. Identity
    /// is decided on the *resolved* path ([`same_source`]); this is the author's own text,
    /// which is what a reader will search the file for.
    source: &'a str,
}

/// Which seam a pair meets at.
#[derive(Clone, Copy)]
enum Seam {
    /// An ordinary hard cut inside the timeline, at this instant.
    Interior(i64),
    /// The wrap, under `loop: true` — `duration` connecting back to `0`. ADR-0062: the
    /// location *"is not one interior timestamp; it is two boundary instants on two
    /// different elements"*, and the finding names both.
    Wrap { duration: i64 },
}

impl Seam {
    /// The prose the finding locates itself with.
    fn words(self) -> String {
        match self {
            Seam::Interior(instant) => format!("at {instant} ms"),
            Seam::Wrap { duration } => {
                format!("at the loop seam (duration={duration} \u{2192} 0)")
            }
        }
    }

    /// The instant the out-value is resolved at, and the one the finding carries.
    fn instant(self) -> i64 {
        match self {
            Seam::Interior(instant) => instant,
            Seam::Wrap { duration } => duration,
        }
    }

    fn is_wrap(self) -> bool {
        matches!(self, Seam::Wrap { .. })
    }
}

/// `R-SOURCE-CUT-POP`, over every same-source adjacency in the document.
pub fn check(document: &Loose, report: &mut Report) {
    // Without a legal `frame` there is no frame space for `x`/`y` to default into
    // (ADR-0012 puts an undeclared element at its centre), and a check that invented a
    // size would be comparing positions in a coordinate system the document does not have.
    // The schema check owns saying so.
    let Some(frame) = frame_dimensions(document) else {
        return;
    };
    let base = crate::checks::project_dir(document);
    let root = document.value();
    // ADR-0062: `loop` is a project-level boolean, default `false`/absent. Anything else
    // written there is a schema fact and not this check's to report — and is not `true`.
    let looping = root.get("loop") == Some(&Value::Bool(true));
    let duration = root.get("duration").and_then(Value::as_i64);

    for track in root
        .get("tracks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let name = track.get("name").and_then(Value::as_str);
        // Time order, never array order: a track *"has no start, no duration and no
        // clock"* and the order its elements are written in carries no meaning (ADR-0060).
        let mut elements: Vec<Adjacent<'_>> = track
            .get("elements")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(read)
            .collect();
        elements.sort_by_key(|element| (element.start, element.end, element.id));

        // ADR-0033's trigger is `A.end == B.start`, and it is asked of every pair rather
        // than of neighbours in the sorted list: a track carrying an illegal overlap —
        // `[0,100]`, `[50,60]`, `[100,200]` — sorts the short one between the two that
        // really do cut, and a sliding window would lose the seam at 100 entirely. The
        // overlap is `E-TRACK-OVERLAP`'s finding; the adjacency is still this one's.
        // Indexed by `start`, so the pairing is a lookup rather than a scan of the track
        // per element.
        let mut starting_at: BTreeMap<i64, Vec<usize>> = BTreeMap::new();
        for (i, element) in elements.iter().enumerate() {
            starting_at.entry(element.start).or_default().push(i);
        }
        for a in &elements {
            for &i in starting_at.get(&a.end).into_iter().flatten() {
                let b = &elements[i];
                // An element is not adjacent to itself: a zero-length range states no
                // instant at all, and `read` has already refused to build one.
                if std::ptr::eq(a, b) {
                    continue;
                }
                // *"A gap suppresses the check, since a gap is content, not a seam."*
                // — and by construction there is none here.
                if let Some(finding) = pop(a, b, Seam::Interior(a.end), &base, frame) {
                    report.push(located(finding, document, name, a.id));
                }
            }
        }

        if !looping {
            continue;
        }
        let Some(duration) = duration else {
            continue;
        };
        // ADR-0062's wrap-adjacency condition, the direct generalization of `A.end ==
        // B.start`: the track's last element must reach `duration` and its first must
        // start at `0`. `max_by_key`/`min_by_key` over the time-sorted list rather than
        // over the array, for the same reason the interior pass sorts.
        let (Some(first), Some(last)) = (
            elements
                .iter()
                .min_by_key(|element| (element.start, element.id)),
            elements
                .iter()
                .max_by_key(|element| (element.end, element.id)),
        ) else {
            continue;
        };
        if last.end != duration || first.start != 0 {
            continue;
        }
        if let Some(finding) = pop(last, first, Seam::Wrap { duration }, &base, frame) {
            report.push(located(finding, document, name, last.id));
        }
    }
}

/// One element, or `None` where it states no readable range, `id` or `source`.
///
/// An element carrying no `source` — a `rect`, an `ellipse`, a piece of `text` — has no
/// source identity to share with its neighbour, so the question this check asks cannot be
/// put about it at all. A malformed range is the schema check's fact, and an adjacency
/// that cannot be computed is never reported as one that is there.
fn read(element: &Value) -> Option<Adjacent<'_>> {
    Some(Adjacent {
        id: element.get("id")?.as_str()?,
        start: element.get("start")?.as_i64()?,
        end: element.get("end")?.as_i64()?,
        source: element.get("source")?.as_str()?,
        element,
    })
}

/// A finding's location: the file, the track where the document names one, and the
/// out-element.
///
/// At the out-element rather than the in-element because the seam's instant is that
/// element's `end` — the reader following the finding to a line wants the element whose
/// travel produced the out-value. Both are named in the fields.
fn located(finding: Finding, document: &Loose, track: Option<&str>, id: &str) -> Finding {
    let finding = finding.at_file(document.path()).at_element(id);
    match track {
        Some(track) => finding.at_track(track),
        None => finding,
    }
}

/// The pop across one adjacency, or `None` where there is none to state.
fn pop(
    a: &Adjacent<'_>,
    b: &Adjacent<'_>,
    seam: Seam,
    base: &std::path::Path,
    frame: (i64, i64),
) -> Option<Finding> {
    if !same_source(a, b, base) {
        return None;
    }

    let (out_at, in_at) = (seam.instant(), b.start);
    let mut rows: Vec<String> = Vec::new();
    let mut properties: Vec<&str> = Vec::new();

    // `origin` first, and unconditionally: it decides what every row below *means*, so a
    // reader who is told the reference frames disagree has been told why the numbers are
    // not comparable before being shown any.
    let (out_origin, in_origin) = (origin_of(a.element), origin_of(b.element));
    if !same_origin(a.element, b.element) {
        rows.push(format!(
            "`origin` {out_origin} \u{2192} {in_origin} \u{2014} the reference frames disagree"
        ));
        properties.push("origin");
    }

    for (name, tolerance, out, into) in [
        (
            "x",
            PIXEL_TOLERANCE,
            geometry::number::<i64>(a.element, "x", out_at, frame.0 as f64 / 2.0),
            geometry::number::<i64>(b.element, "x", in_at, frame.0 as f64 / 2.0),
        ),
        (
            "y",
            PIXEL_TOLERANCE,
            geometry::number::<i64>(a.element, "y", out_at, frame.1 as f64 / 2.0),
            geometry::number::<i64>(b.element, "y", in_at, frame.1 as f64 / 2.0),
        ),
        (
            "rotation",
            ROTATION_TOLERANCE,
            geometry::number::<f64>(a.element, "rotation", out_at, 0.0),
            geometry::number::<f64>(b.element, "rotation", in_at, 0.0),
        ),
        (
            "opacity",
            OPACITY_TOLERANCE,
            geometry::number::<f64>(a.element, "opacity", out_at, 1.0),
            geometry::number::<f64>(b.element, "opacity", in_at, 1.0),
        ),
    ] {
        if (out - into).abs() > tolerance {
            rows.push(row(&format!("`{name}`"), out, into));
            properties.push(name);
        }
    }

    // `scale` is `[sx, sy]` and ADR-0033 compares the axes independently. Where both fail
    // and both carry the same pair of numbers the finding shows one axis and says so —
    // the ADR's own worked line — because two identical rows state one fact twice.
    let out_scale = geometry::number::<[f64; 2]>(a.element, "scale", out_at, [1.0, 1.0]);
    let in_scale = geometry::number::<[f64; 2]>(b.element, "scale", in_at, [1.0, 1.0]);
    let axes: Vec<usize> = (0..2)
        .filter(|&i| (out_scale[i] - in_scale[i]).abs() > SCALE_TOLERANCE)
        .collect();
    if !axes.is_empty() {
        properties.push("scale");
        if axes.len() == 2 && out_scale[0] == out_scale[1] && in_scale[0] == in_scale[1] {
            rows.push(row_noting(
                "`scale`",
                out_scale[0],
                in_scale[0],
                ", one axis shown, both equal",
            ));
        } else {
            for i in axes {
                let axis = if i == 0 { "sx" } else { "sy" };
                rows.push(row(&format!("`scale` {axis}"), out_scale[i], in_scale[i]));
            }
        }
    }

    if rows.is_empty() {
        return None;
    }

    Some(
        Finding::new("R-SOURCE-CUT-POP")
            .field("element", json!(a.id))
            .field("other", json!(b.id))
            // The out-element's own spelling. Identity was decided on the resolved path,
            // which may differ from either spelling; what a reader searches the file for
            // is the text that is in it.
            .field("source", json!(a.source))
            .field("seam", json!(seam.words()))
            .field("instant", json!(seam.instant()))
            .field("wrap", json!(seam.is_wrap()))
            .field("properties", json!(properties))
            .field("detail", json!(rows.join("; "))),
    )
}

/// One `detail` row: the resolved out-value, the resolved in-value, and the delta.
fn row(label: &str, out: f64, into: f64) -> String {
    row_noting(label, out, into, "")
}

/// The same row, with `note` inside the parenthesis the delta sits in.
///
/// One formatter rather than a row taken apart and reassembled: the only row that carries
/// a note is `scale`'s collapsed pair — ADR-0033's own *"one axis shown, both equal"* —
/// and gluing that on after the fact would mean the two spellings of a row could drift.
fn row_noting(label: &str, out: f64, into: f64, note: &str) -> String {
    format!(
        "{label} {out:.DECIMALS$} \u{2192} {into:.DECIMALS$} (\u{394} {:+.DECIMALS$}{note})",
        into - out
    )
}

/// Do these two elements draw the same source?
///
/// **The canonicalized path, not a raw string match** (ADR-0033): `images/05.png` and
/// `./images/05.png` name one file, and [`Source::resolve`] is the same ADR-0053
/// resolution every disk-reading check already uses — a relative path against the project
/// file's own directory, a `file:` URL as the local path it spells, everything else as the
/// remote string it is. `Path` compares by components, so the `.` segment normalises away
/// without a call to the filesystem.
///
/// **For a time-based source, same path is necessary but not sufficient** (ADR-0033). Two
/// clips of one file whose source times are not contiguous across the cut are a deliberate
/// cut *within* the media, and a reset there is exactly as correct as a different-source
/// cut. An image has no time axis, so the term is vacuously satisfied — which is why the
/// fixture, being all stills, never exercises it, and why it is written now rather than
/// discovered on the first project with video in it.
fn same_source(a: &Adjacent<'_>, b: &Adjacent<'_>, base: &std::path::Path) -> bool {
    if Source::resolve(a.source, base) != Source::resolve(b.source, base) {
        return false;
    }
    if !is_time_based(a.element) && !is_time_based(b.element) {
        return true;
    }
    // A missing or unreadable `source_end`/`source_start` is a schema fact and not this
    // check's; continuity that cannot be computed is never asserted.
    match (
        a.element.get("source_end").and_then(Value::as_i64),
        b.element.get("source_start").and_then(Value::as_i64),
    ) {
        (Some(out), Some(into)) => out == into,
        _ => false,
    }
}

/// Does this element's source carry a clock? `video` and `audio` do; an image does not.
fn is_time_based(element: &Value) -> bool {
    matches!(
        element.get("type").and_then(Value::as_str),
        Some("video" | "audio")
    )
}

/// The element's declared reference frame, as the keyword it resolves to.
///
/// An absent `origin` is `center` (ADR-0012), and it is compared as the resolved keyword
/// rather than as the written field: the question is whether the two elements read their
/// numbers against the same frame, and an element that omits the key reads against exactly
/// the same frame as one that writes `"center"`. ADR-0030's presence-is-content rule is
/// about what the *document declares*, and this comparison is about where the pixels land.
///
/// Two keywords compare as keywords, as they always have. Once either side is a free point
/// `[px, py]` (#612), the comparison is of the **box-local resolved point**, each keyword
/// resolved against its own element's `width`/`height`: `"top-left"` against `[0, 0]` is
/// the same reference frame written as two different claims, and not a trigger. That
/// extends the absent-equals-`center` rule above rather than replacing it.
fn same_origin(a: &Value, b: &Value) -> bool {
    let is_point = |element: &Value| element.get("origin").is_some_and(Value::is_array);
    if !is_point(a) && !is_point(b) {
        return origin_of(a) == origin_of(b);
    }
    let resolved = |element: &Value| {
        let width = element.get("width").and_then(Value::as_i64)? as f64;
        let height = element.get("height").and_then(Value::as_i64)? as f64;
        geometry::origin_offset(element, width, height)
    };
    match (resolved(a), resolved(b)) {
        (Some(out), Some(into)) => out == into,
        _ => origin_of(a) == origin_of(b),
    }
}

fn origin_of(element: &Value) -> String {
    match element.get("origin") {
        None | Some(Value::Null) => "\"center\"".to_string(),
        Some(value) => value.to_string(),
    }
}
