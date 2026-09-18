//! Retired spellings — the migration mechanism, since there is no version number.
//!
//! ADR-0016: a project file carries no version number, there is no `montaget migrate`,
//! and a file written against an older vocabulary is caught by an error that **names the
//! replacement**. That naming is a property of the message text and holds for every
//! retirement; what varies is whether the finding may also state the *fix*.
//!
//! ADR-0043 settles that second question, once per check and never per instance, which is
//! why the retirements are split across **two codes** rather than one:
//!
//! - [`E-RETIRED-KEY`](crate::registry) is **refuse-class**. Its founding instance is
//!   `gravity`: of the 8 elements carrying it in the pre-ADR-0015 fixture, 6 deletions
//!   were geometric no-ops and 2 silently changed which part of the source was on screen,
//!   and the fact separating them — the source's pixel dimensions — is not in the
//!   document. So every instance refuses, including the ones that look safe, and carries
//!   a sibling census grouping the affected elements by an observable geometry fact.
//! - `E-RETIRED-SPELLING` is **advise-class**. Every member of it is a transposition
//!   whose whole input is the string already in the file: the bare `mask` key
//!   (ADR-0068), `anchor` carrying a string, `center-center`, the opaque eight-digit
//!   colour, `bold`/`weight`, and `none`/`fill` as a `fit` value.
//!
//! One code may not emit both shapes (ADR-0043's uniformity rule), so the split *is* the
//! classification — and `tests/retired_spellings.rs` asserts it holds over every instance
//! the checks can produce rather than over one of them.
//!
//! # Recorded, not decided: `box` and `align` on a non-text element
//!
//! Neither has been classified by any ADR, and both carry `gravity`'s fork:
//!
//! - `box: [x,y,w,h]` retires into `x`, `y`, `origin`, `width`, `height` (ADR-0012). The
//!   pivot the 4-array left implicit is precisely what the repair needs and the document
//!   does not carry. Its other meaning, `box: "card-05"` — *the id of the element you must
//!   fit inside* — retires into literal `width`/`height`, whose values live on an element
//!   that 15 of the fixture's 22 text elements do not have at all.
//! - `align` on an image meant *which part of the source survives the crop* (ADR-0012),
//!   which is `gravity`'s question in `align`'s spelling, and inherits `gravity`'s fork
//!   with it.
//!
//! ADR-0068 remarks in passing that *"`box` was migrated by arithmetic script"*, which
//! points toward advise; ADR-0043 is explicit that it *"classifies `gravity` and nothing
//! else"*. Until an ADR rules, both sit under ADR-0043's own standing rule — *"if any
//! instance a check can match is capable of being load-bearing, the check emits
//! `repair: "none"` for every instance it matches"* — which is the conservative half of
//! the fork and the one whose cost that ADR has already accepted. **This placement is not
//! the ruling.** The ruling belongs to an ADR, and moving either spelling to the
//! advise-class code is a one-line change here once one exists.

use serde_json::{Map, Value, json};

use crate::finding::{Census, Finding};
use crate::permissive::Loose;
use crate::report::Report;

/// The keys whose value is a colour. Named rather than sniffed: a recursive hunt for
/// anything shaped like `#RRGGBBFF` would eventually fire on a `source` path, and the
/// format's colour-valued keys are a closed list that the types already enumerate.
const COLOUR_KEYS: [&str; 4] = ["background", "color", "fill", "stroke"];

/// Every retired spelling in the document, as findings.
pub fn check(document: &Loose, report: &mut Report) {
    let sightings = sightings(document);
    let file = document.path();

    for sighting in &sightings {
        let finding = match &sighting.verdict {
            Verdict::Refuse { census_field, .. } => {
                // The class is taken from the registry by `Finding::new`; nothing here
                // asks for `repair: "none"` and nothing here could ask for anything else.
                Finding::new("E-RETIRED-KEY")
                    .census(census(&sightings, &sighting.key, census_field))
            }
            Verdict::Advise { repair } => {
                Finding::new("E-RETIRED-SPELLING").repair_value(json!({ "value": repair }))
            }
        };

        let mut finding = finding
            .at_file(file)
            .field("subject", json!(sighting.subject))
            .field("key", json!(sighting.key))
            .field("value", sighting.value.clone())
            .field("replacement", json!(sighting.replacement));
        if let Some(element) = &sighting.element {
            finding = finding.at_element(element);
        }
        if let Some(track) = &sighting.track {
            finding = finding.at_track(track);
        }
        report.push(finding);
    }
}

/// One retired spelling, found in one place.
struct Sighting {
    /// What the prose names: an element's `id`, or the project itself.
    subject: String,
    element: Option<String>,
    track: Option<String>,
    key: String,
    value: Value,
    /// What the format says now, in words. ADR-0016: naming it is a message-text property
    /// of *every* retirement, refuse-class or not — refusing to state a repair is not
    /// refusing to state the replacement.
    replacement: String,
    verdict: Verdict,
}

enum Verdict {
    /// ADR-0043 refuse-class, carrying the observable fact its census groups on.
    Refuse {
        census_field: &'static str,
        census_value: Value,
    },
    /// ADR-0043 advise-class: the content of the repair, which varies per instance where
    /// the class does not.
    Advise { repair: Value },
}

fn sightings(document: &Loose) -> Vec<Sighting> {
    let mut out = Vec::new();
    let Some(root) = document.value().as_object() else {
        return out;
    };

    // The project's own colours, which are the only ones not on an element. Its children
    // are tracks, so this reads the root's own keys and does not recurse.
    let project = Locus::project();
    out.extend(
        COLOUR_KEYS
            .iter()
            .filter_map(|key| Some((*key, root.get(*key)?)))
            .filter_map(|(key, value)| project.retired_colour(key, value)),
    );

    for track in root
        .get("tracks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let name = track
            .get("name")
            .and_then(Value::as_str)
            .map(str::to_string);
        for element in track
            .get("elements")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(element) = element.as_object() {
                scan_element(element, &name, &mut out);
            }
        }
    }
    out
}

fn scan_element(element: &Map<String, Value>, track: &Option<String>, out: &mut Vec<Sighting>) {
    scan(element, &Locus::element(element, track), out);
}

/// Where a sighting is, and what it is on. One value rather than four parameters, because
/// the four never travel apart: everything nested inside an element — a `run`, a
/// `highlight`, an effect — is reported at that element, since `runs[2]` is not a locus
/// the report knows how to name.
struct Locus {
    /// What the prose names: an element's `id`, or the project itself.
    subject: String,
    element: Option<String>,
    track: Option<String>,
    /// The element's `type`, which two of the retirements read: `align` is retired only
    /// off text, and `gravity`'s replacement differs where there is no aperture.
    type_name: String,
}

impl Locus {
    fn project() -> Self {
        Locus {
            subject: "the project".into(),
            element: None,
            track: None,
            type_name: String::new(),
        }
    }

    fn element(element: &Map<String, Value>, track: &Option<String>) -> Self {
        let id = element.get("id").and_then(Value::as_str);
        Locus {
            subject: id.unwrap_or("an element carrying no `id`").to_string(),
            element: id.map(str::to_string),
            track: track.clone(),
            type_name: element
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        }
    }

    fn sighting(&self, key: &str, value: &Value, replacement: String, verdict: Verdict) -> Sighting {
        Sighting {
            subject: self.subject.clone(),
            element: self.element.clone(),
            track: self.track.clone(),
            key: key.to_string(),
            value: value.clone(),
            replacement,
            verdict,
        }
    }

    /// The one retirement that is a value rather than a key, and the one that can appear
    /// on the project as well as on an element — so it is written once, here, rather than
    /// in both traversals.
    fn retired_colour(&self, key: &str, value: &Value) -> Option<Sighting> {
        let six = opaque_eight_digit(value)?;
        Some(self.sighting(
            key,
            value,
            format!("`{six}`"),
            Verdict::Advise { repair: json!(six) },
        ))
    }
}

/// One object inside an element, and everything nested under it — a `run`, a `highlight`,
/// an `effect`, a keyframe record. The traversal is recursive rather than field-by-field
/// because the retired spellings it looks for are, by construction, keys the types do not
/// have: a scan written against the current field set would be a second enumeration of the
/// schema, in the one place a drift between two enumerations goes unnoticed.
fn scan(object: &Map<String, Value>, locus: &Locus, out: &mut Vec<Sighting>) {
    let at = |key: &str, value: &Value, replacement: String, verdict: Verdict| {
        locus.sighting(key, value, replacement, verdict)
    };

    // The aperture the refuse-class censuses group on: `clip` is the geometry fact that,
    // with the rect's own position, already determines which part of the source survives
    // — which is exactly the quantity the retired keys named.
    let clip = object.get("clip").cloned().unwrap_or(Value::Null);
    // What `box`'s census groups on instead. A file still carrying `box` predates `clip`
    // — ADR-0012 retired one and introduced the other in the same breath — so an aperture
    // census would group every element under "absent" and narrow nothing. The pivot is
    // the fact that repair needs and the 4-array left implicit, so the census reports
    // which of the affected elements state one. Present geometry, never old semantics.
    let origin = object.get("origin").cloned().unwrap_or(Value::Null);
    let type_name = locus.type_name.as_str();

    for (key, value) in object {
        match key.as_str() {
            // ---- Refuse-class (ADR-0043). ------------------------------------------
            "gravity" => out.push(at(
                key,
                value,
                gravity_replacement(type_name),
                Verdict::Refuse {
                    census_field: "clip",
                    census_value: clip.clone(),
                },
            )),
            // Retired in both of its meanings, which is why it cannot even produce a good
            // error message without reading its value's shape (ADR-0012).
            "box" => out.push(at(
                key,
                value,
                match value {
                    Value::String(_) => "literal `width` and `height`".into(),
                    _ => "`x`, `y`, `origin`, `width`, `height` and `clip`".into(),
                },
                Verdict::Refuse {
                    census_field: "origin",
                    census_value: origin.clone(),
                },
            )),
            // `align` is not retired; `align` on a non-text element is. On text it keeps
            // ADR-0007's meaning exactly — how lines align to each other.
            //
            // An element whose `type` is absent or misspelled is *not* a non-text element
            // — it is an element whose type is the finding, which is a schema question and
            // another ticket's. Firing here would answer it, and answer it with a
            // non-bypassable refusal.
            "align" if is_visual_non_text(type_name) => out.push(at(
                key,
                value,
                "`x`, `y`, `origin` and the aperture's `clip`".into(),
                Verdict::Refuse {
                    census_field: "clip",
                    census_value: clip.clone(),
                },
            )),

            // ---- Advise-class (ADR-0043). ------------------------------------------
            // ADR-0068: *"a bare `mask` key on any element"* — the value's shape does not
            // qualify it, so neither does this. The key never had accepted semantics in
            // any document, so no prior meaning exists for a repair to misread, and the
            // repair carries whatever was written into the `shape` slot verbatim: a value
            // outside the enum is then an ordinary schema question about that value,
            // which is not this check's to adjudicate.
            "mask" => out.push(at(
                key,
                value,
                "`effects`".into(),
                Verdict::Advise {
                    repair: json!({"effects": [{"name": "mask", "shape": value}]}),
                },
            )),
            // The collision is caught on *shape*, not presence: `anchor` carrying a
            // below/above object is ADR-0019's ordinary feature.
            "anchor" if value.is_string() => out.push(at(
                key,
                value,
                "`origin`".into(),
                Verdict::Advise {
                    repair: json!({ "origin": value }),
                },
            )),
            // A different weight is a different file. With one declared file and no
            // family to search, `bold: true` could only mean synthetic emboldening, which
            // is renderer-specific and machine-dependent (ADR-0007).
            //
            // Advise, not refuse, and the call is close enough to record: the repair is
            // an instruction rather than a value, because the file it names is not in the
            // document. But ADR-0043's refuse test is *intent* — "the fix depends on
            // knowing what the author meant" — and nothing here is in doubt about what
            // `bold: true` meant. What is missing is an asset, which ADR-0016 says is
            // "something an agent can do and a program categorically cannot", and
            // refusing would send an ordinary authoring move to a human. `E-READ` already
            // takes the same shape, for the same reason, with the same kind of value.
            "weight" | "bold" => out.push(at(
                key,
                value,
                "a font file of that weight, declared in the project's `fonts` table and named by `font`".into(),
                Verdict::Advise {
                    repair: json!(
                        "declare the weight as its own file in `fonts` and name that key in `font`"
                    ),
                },
            )),
            "origin" if value == "center-center" => out.push(at(
                key,
                value,
                "`center`".into(),
                Verdict::Advise {
                    repair: json!({"origin": "center"}),
                },
            )),
            // The CSS-habit reach: 3 of 8 consumers tried one of these before finding the
            // legal value. `fill` is spent on a shape's paint; CSS `object-fit: none`
            // means *intrinsic size*, which this format cannot express (ADR-0015).
            "fit" if value == "none" || value == "fill" => out.push(at(
                key,
                value,
                "`literal`".into(),
                Verdict::Advise {
                    repair: json!({"fit": "literal"}),
                },
            )),
            _ => {}
        }

        if COLOUR_KEYS.contains(&key.as_str())
            && let Some(sighting) = locus.retired_colour(key, value)
        {
            out.push(sighting);
        }

        // Nested objects carry their element's identity: a `run`'s retired spelling is
        // still that element's, and `runs[2]` is not a locus the report knows how to name.
        match value {
            Value::Object(nested) => scan(nested, locus, out),
            Value::Array(items) => {
                for item in items {
                    if let Value::Object(nested) = item {
                        scan(nested, locus, out);
                    }
                }
            }
            _ => {}
        }
    }
}

/// `#RRGGBBFF` is the opaque form of a colour the six-digit spelling already says, and two
/// spellings of one value break the write-read round trip: `fmt` normalises on write and
/// the agent's next exact-string replace finds nothing (ADR-0014). Eight digits are legal
/// — it is the fully-opaque eight that are retired.
///
/// Matched case-insensitively and answered in uppercase, because `#fbf3e3ff` is what the
/// CSS habit actually types and it is two retired spellings at once: hex digits are
/// uppercase (ADR-0014), and the opaque alpha is a second spelling of six digits. The
/// six-digit form this names is the one the schema will accept.
fn opaque_eight_digit(value: &Value) -> Option<String> {
    let body = value.as_str()?.strip_prefix('#')?;
    if body.len() == 8 && body.bytes().all(|b| b.is_ascii_hexdigit()) && body[6..].eq_ignore_ascii_case("FF")
    {
        return Some(format!("#{}", body[..6].to_ascii_uppercase()));
    }
    None
}

/// The element types `align` is retired on. Named positively rather than as "not text",
/// so that an element whose `type` is absent or misspelled — a schema question, and
/// another ticket's — does not collect a non-bypassable refusal on the way past.
fn is_visual_non_text(type_name: &str) -> bool {
    matches!(type_name, "image" | "video" | "rect" | "ellipse")
}

/// On an image or a video the message names the rect's own position and the aperture; on
/// text and shapes there is no aperture, and ADR-0014's clause names `origin` alone.
/// `gravity` is retired on *every* element type (ADR-0015), so an element whose type
/// cannot be read still gets the finding — and gets both halves of the replacement,
/// rather than a confident half that may be the wrong one.
fn gravity_replacement(type_name: &str) -> String {
    match type_name {
        "image" | "video" => "`x`, `y`, `origin` and the aperture's `clip`".into(),
        "text" | "rect" | "ellipse" => "`origin`".into(),
        _ => "`x`, `y`, `origin` and the aperture's `clip` — or `origin` alone, on a text \
or shape element"
            .into(),
    }
}

/// The sibling census for one retired key: every element the same key was found on,
/// grouped by one observable, document-derived fact.
///
/// ADR-0043 is explicit that this narrows attention and must not be worded — or ordered —
/// so that the larger group reads as the correct one. Groups therefore keep the order the
/// document declared them in, and nothing here sorts by size.
fn census(sightings: &[Sighting], key: &str, field: &'static str) -> Census {
    let mut census = Census::on(field);
    let mut groups: Vec<(Value, Vec<String>)> = Vec::new();

    for sighting in sightings {
        let Verdict::Refuse { census_value, .. } = &sighting.verdict else {
            continue;
        };
        if sighting.key != key {
            continue;
        }
        match groups.iter_mut().find(|(value, _)| value == census_value) {
            Some((_, members)) => members.push(sighting.subject.clone()),
            None => groups.push((census_value.clone(), vec![sighting.subject.clone()])),
        }
    }

    for (value, members) in groups {
        census = census.group(value, members);
    }
    census
}
