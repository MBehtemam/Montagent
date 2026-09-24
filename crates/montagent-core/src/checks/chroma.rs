//! ADR-0088's four `chroma` findings — three read from the document, one from the probe.
//!
//! **Not one of them keys a frame**, and that is the module's shape rather than an
//! economy. ADR-0006 keeps `validate` to *"is this internally legal, and does it agree
//! with the media on disk"*; a check that had to run the keyer to form an opinion would be
//! the *"does it say what you meant"* verb that ADR refuses to be. The question *"did this
//! key well"* has an owner, and it is `measure`'s per-frame coverage reading.
//!
//! | finding | what it reads |
//! | --- | --- |
//! | `R-CHROMA-AFTER-COLOUR` | two positions in one `effects` list |
//! | `R-CHROMA-ON-AUTHORED-ELEMENT` | the element's own `type` |
//! | `N-CHROMA-INERT` | `tolerance`, against its identity value |
//! | `R-CHROMA-ON-ALPHA-SOURCE` | one `ffprobe` field the pipeline already has |
//!
//! Every one is `review` or `note` and none proposes a repair (ADR-0043). That is not
//! timidity: a deliberate pre-grade before keying is a real if unusual technique, and a
//! shape keyed out of an authored rect is a legal picture. Only the author knows which
//! they meant.
//!
//! **The alpha check ships with a stated gap.** ADR-0088: *"the #339 relationship is
//! coverage, not trust"* — [`crate::media::probe::Probe::alpha`] is derived from `pix_fmt`,
//! which yields false *negatives* for VP9, so this check is correct whenever it fires and
//! merely silent when it should have fired. A source it says nothing about is not a source
//! it cleared.
//!
//! Read permissively throughout: an `effects` that is not an array, a member that is not an
//! object, a `tolerance` that is not a number — all are the schema check's to name, and
//! simply are not read here.

use serde_json::{Value, json};

use crate::checks::subject_of;
use crate::finding::Finding;
use crate::media::Source;
use crate::media::session::Session;
use crate::media::tools::Missing;
use crate::model::effects::{CHROMA, COLOUR_FILTERS};
use crate::permissive::Loose;
use crate::report::Report;

/// The `type` values whose pixels the format itself authored (ADR-0014, ADR-0007).
///
/// `image` and `video` are the other side of the line and are absent deliberately: their
/// pixels come off a disk this document did not write, which is the case the keyer exists
/// for. `audio` has no pixels, and a `transition` is a bridge rather than a picture.
const AUTHORED: [&str; 3] = ["text", "rect", "ellipse"];

/// `tolerance`'s identity value, which keys nothing (ADR-0088).
const INERT: f64 = 0.0;

/// The three findings that need only the document.
pub fn check(document: &Loose, report: &mut Report) {
    for (track, element) in document.elements_in_tracks() {
        let subject = subject_of(element.get("id").and_then(Value::as_str));
        for key in keys(element) {
            for finding in [
                after_colour(element, &key),
                on_authored_element(element, &key),
                inert(&key),
            ]
            .into_iter()
            .flatten()
            {
                push(report, finding, document.path(), &subject, track);
            }
        }
    }
}

/// `R-CHROMA-ON-ALPHA-SOURCE`, which needs one field off the probe.
///
/// Takes the session `crate::checks::source` and `crate::checks::fit` already share, for
/// their reason: every referenced source is probed once per run, and a second session here
/// would either duplicate every `ffprobe` call or overwrite `report.misses` with a partial
/// view of what ran.
pub fn on_disk(
    document: &Loose,
    session: &mut Session,
    report: &mut Report,
) -> Result<(), Box<Missing>> {
    let base = crate::checks::project_dir(document);

    for (track, element) in document.elements_in_tracks() {
        let Some(source) = element.get("source").and_then(Value::as_str) else {
            continue;
        };
        let keys = keys(element);
        if keys.is_empty() {
            continue;
        }

        let outcome = session.probe(&Source::resolve(source, &base))?;
        // Existence-only, a confirmed miss, or genuinely unprobeable: nothing about the
        // source's pixel format was established, so the question is unanswered rather than
        // answered "no" — and `crate::checks::source` reports that as its own finding.
        if outcome.probe().and_then(|probe| probe.alpha) != Some(true) {
            continue;
        }

        let subject = subject_of(element.get("id").and_then(Value::as_str));
        for key in keys {
            let finding = Finding::new("R-CHROMA-ON-ALPHA-SOURCE")
                .field("index", json!(key.index))
                .field("color", json!(key.color))
                .field("source", json!(source));
            push(report, finding, document.path(), &subject, track);
        }
    }
    Ok(())
}

/// One `chroma` member of one element's `effects`, at the position it sits in.
struct Key<'a> {
    /// Its index in the list — the half of *where* a finding can point at, since ADR-0040
    /// makes two members of the same name ordinary and the name alone cannot say which one
    /// this is.
    index: usize,
    /// `color` as the document spells it. The finding quotes the author's own screen
    /// colour rather than describing it, so the sentence names something greppable in the
    /// file.
    color: &'a str,
    /// `tolerance`, where it is readable as a number.
    tolerance: Option<f64>,
}

/// Every `chroma` member this element declares, with its position.
fn keys(element: &Value) -> Vec<Key<'_>> {
    members(element)
        .enumerate()
        .filter(|(_, effect)| effect.get("name").and_then(Value::as_str) == Some(CHROMA))
        .map(|(index, effect)| Key {
            index,
            color: effect
                .get("color")
                .and_then(Value::as_str)
                .unwrap_or("the colour the schema check reports"),
            tolerance: effect.get("tolerance").and_then(Value::as_f64),
        })
        .collect()
}

/// This element's `effects`, in document order.
fn members(element: &Value) -> std::slice::Iter<'_, Value> {
    element
        .get("effects")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
}

/// The **first** colour operation ahead of this key, if there is one.
///
/// First rather than every one: the finding is *"this key is not measured against the
/// source's own colour"*, which is one fact about one key however many scalars precede it.
/// One finding per pair would report the same defect three times on a graded element.
fn after_colour(element: &Value, key: &Key) -> Option<Finding> {
    let (colour_index, colour_name) =
        members(element)
            .take(key.index)
            .enumerate()
            .find_map(|(index, effect)| {
                let name = effect.get("name").and_then(Value::as_str)?;
                COLOUR_FILTERS.contains(&name).then_some((index, name))
            })?;

    Some(
        Finding::new("R-CHROMA-AFTER-COLOUR")
            .field("index", json!(key.index))
            .field("color", json!(key.color))
            .field("colour_index", json!(colour_index))
            .field("colour_name", json!(colour_name)),
    )
}

/// A key on pixels the format itself authored.
fn on_authored_element(element: &Value, key: &Key) -> Option<Finding> {
    let declared = element.get("type").and_then(Value::as_str)?;
    AUTHORED.contains(&declared).then(|| {
        Finding::new("R-CHROMA-ON-AUTHORED-ELEMENT")
            .field("index", json!(key.index))
            .field("color", json!(key.color))
            .field("type", json!(declared))
    })
}

/// `tolerance: 0` — the identity value, declared.
///
/// The shape ADR-0052 already made a finding for with inert ease: an effect that is in the
/// list and does nothing. Read off the member rather than off the parsed model, because a
/// document that does not fit the types still reaches every check here (ADR-0016), and
/// `0` and `0.0` are the same identity either way.
fn inert(key: &Key) -> Option<Finding> {
    (key.tolerance? == INERT).then(|| {
        Finding::new("N-CHROMA-INERT")
            .field("index", json!(key.index))
            .field("color", json!(key.color))
    })
}

/// Attach the location every finding in this module carries.
///
/// `element` is both a field and the finding's location, as it is in
/// `crate::checks::mask`: the templates interpolate it as the sentence's subject, and the
/// report's own filters read the location.
fn push(report: &mut Report, finding: Finding, file: &str, subject: &str, track: Option<&str>) {
    let mut finding = finding
        .field("element", json!(subject))
        .at_file(file)
        .at_element(subject.to_string());
    if let Some(track) = track {
        finding = finding.at_track(track);
    }
    report.push(finding);
}
