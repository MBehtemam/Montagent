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

    // The project's own colours, which are the only ones not on an element.
    colours(root, "the project", &None, &None, &mut out);

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
    let id = element.get("id").and_then(Value::as_str);
    let subject = id.unwrap_or("an element carrying no `id`").to_string();
    let located = id.map(str::to_string);
    let type_name = element
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    scan(element, &subject, &located, track, &type_name, out);
}

/// One object inside an element, and everything nested under it — a `run`, a `highlight`,
/// an `effect`, a keyframe record. The traversal is recursive rather than field-by-field
/// because the retired spellings it looks for are, by construction, keys the types do not
/// have: a scan written against the current field set would be a second enumeration of the
/// schema, in the one place a drift between two enumerations goes unnoticed.
fn scan(
    object: &Map<String, Value>,
    subject: &str,
    element: &Option<String>,
    track: &Option<String>,
    type_name: &str,
    out: &mut Vec<Sighting>,
) {
    let at = |key: &str, value: &Value, replacement: String, verdict: Verdict| Sighting {
        subject: subject.to_string(),
        element: element.clone(),
        track: track.clone(),
        key: key.to_string(),
        value: value.clone(),
        replacement,
        verdict,
    };

    // The aperture the refuse-class censuses group on: `clip` is the geometry fact that,
    // with the rect's own position, already determines which part of the source survives
    // — which is exactly the quantity the retired keys named.
    let clip = object.get("clip").cloned().unwrap_or(Value::Null);

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
                    census_field: "box",
                    census_value: value.clone(),
                },
            )),
            // `align` is not retired; `align` on a non-text element is. On text it keeps
            // ADR-0007's meaning exactly — how lines align to each other.
            "align" if type_name != "text" => out.push(at(
                key,
                value,
                "`x`, `y`, `origin` and the aperture's `clip`".into(),
                Verdict::Refuse {
                    census_field: "clip",
                    census_value: clip.clone(),
                },
            )),

            // ---- Advise-class (ADR-0043). ------------------------------------------
            // ADR-0068: the bare key never had accepted semantics in any document, so no
            // prior meaning exists for a repair to misread, and the param-less form is
            // now defined. The only input is the string itself — a shape outside the enum
            // transposes the same way and is then an ordinary schema question about the
            // value, which is not this check's to adjudicate.
            "mask" if value.is_string() => out.push(at(
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
            // is renderer-specific and machine-dependent (ADR-0007). The author's meaning
            // is not in question — only the file is — so this advises rather than refuses.
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
            && let Some(six) = opaque_eight_digit(value)
        {
            out.push(at(
                key,
                value,
                format!("`{six}`"),
                Verdict::Advise { repair: json!(six) },
            ));
        }

        // Nested objects carry their element's identity: a `run`'s retired spelling is
        // still that element's, and `runs[2]` is not a locus the report knows how to name.
        match value {
            Value::Object(nested) => scan(nested, subject, element, track, type_name, out),
            Value::Array(items) => {
                for item in items {
                    if let Value::Object(nested) = item {
                        scan(nested, subject, element, track, type_name, out);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Colour keys on one object alone, without recursing. Used for the project root, whose
/// children are tracks and are walked as elements.
fn colours(
    object: &Map<String, Value>,
    subject: &str,
    element: &Option<String>,
    track: &Option<String>,
    out: &mut Vec<Sighting>,
) {
    for key in COLOUR_KEYS {
        let Some(value) = object.get(key) else {
            continue;
        };
        if let Some(six) = opaque_eight_digit(value) {
            out.push(Sighting {
                subject: subject.to_string(),
                element: element.clone(),
                track: track.clone(),
                key: key.to_string(),
                value: value.clone(),
                replacement: format!("`{six}`"),
                verdict: Verdict::Advise { repair: json!(six) },
            });
        }
    }
}

/// `#RRGGBBFF` is the opaque form of a colour the six-digit spelling already says, and two
/// spellings of one value break the write-read round trip: `fmt` normalises on write and
/// the agent's next exact-string replace finds nothing (ADR-0014). Eight digits are legal
/// — it is the fully-opaque eight that are retired.
fn opaque_eight_digit(value: &Value) -> Option<String> {
    let text = value.as_str()?;
    let body = text.strip_prefix('#')?;
    if body.len() == 8 && body.bytes().all(|b| b.is_ascii_hexdigit()) && body.ends_with("FF") {
        return Some(format!("#{}", &body[..6]));
    }
    None
}

/// On an image or a video the message names the rect's own position and the aperture; on
/// text and shapes there is no aperture, and ADR-0014's clause names `origin` alone.
fn gravity_replacement(type_name: &str) -> String {
    match type_name {
        "image" | "video" => "`x`, `y`, `origin` and the aperture's `clip`".into(),
        _ => "`origin`".into(),
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
