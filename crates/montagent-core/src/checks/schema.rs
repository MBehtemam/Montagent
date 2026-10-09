//! The closed schema, as findings — *"does this document fit the format at all?"*
//! (ADR-0017, ADR-0016).
//!
//! [`crate::permissive::Loose::strict`] has always been able to answer this, and until
//! this check nothing in `src/` called it. ADR-0016 is the reason that mattered: the file
//! carries no version number, so the unknown-key error is *the* migration mechanism, and
//! *"an optional signal is indistinguishable from no signal"* — an unfired one is no
//! signal at all. Three other checks also lean on it. `stack.rs`'s `Unresolved::Malformed`
//! and `Unresolved::Unstated`, and the `anchor` check that matches both to `{}`, stay
//! deliberately silent on a `layer` that is neither an integer nor an object and on a
//! track that states none, *on the stated ground that the check owning the schema speaks*.
//! This is that check, so those silences stop compounding.
//!
//! # One parse per scope, not one per document
//!
//! `serde` stops at the first thing it cannot read, so a single strict parse of the whole
//! document yields exactly one fact, about whichever element happens to come first — and
//! ADR-0006 wants a report that is *"a set of facts about the project"*, located at the
//! element a reader has to open their editor at. So the document is parsed **by scope**:
//! the header with its tracks emptied, each track with its elements emptied, and each
//! element on its own. Each scope that does not fit produces one finding, located at that
//! scope, and a fault in one element hides nothing about the next.
//!
//! Emptying `tracks` and `elements` before parsing a container is what makes the scopes
//! independent, and it can only ever *remove* a reason to fail: `Project` and `Track` are
//! satisfied by an empty array, so no scope can report a fault the whole-document parse
//! would not have. The converse — a whole-document failure no scope reproduces — is not
//! ruled out by construction, so it is not assumed: [`findings`] falls back to the
//! whole-document parse when the walk found nothing, which is what makes *"`strict` errs ⇒
//! something is reported"* true rather than merely likely.
//!
//! # Two codes, and the one key this check does not name
//!
//! ADR-0016 requires `validate` to *"distinguish two unknown-key cases in the message
//! text"* — a key that may belong to a newer revision, and a key this binary has retired
//! — and its own draft message says the first out loud: *"not a key this Montagent knows,
//! **and not one it has retired**"*. So `E-SCHEMA-UNKNOWN-KEY` asks
//! `retired::named` (private to the crate, so a plain span rather than a link) and stays silent where the retired check is
//! already speaking. It asks the *predicate*, never the report: ADR-0006 puts the checks
//! in no significant order and forbids anything depending on which spoke first.
//!
//! Four of the nine retirements are retired *values* rather than keys — the opaque
//! eight-digit colour, `center-center`, `fit: none`/`fill`, and `anchor` carrying a string
//! — and ADR-0016 speaks only about keys. They are left to the retirement too, because the
//! alternative is a report that contradicts itself: the retirement is advise-class and
//! states the replacement, this check is refuse-class and states that no repair is
//! determined, and both would print about the same byte.
//!
//! How that recognition is done is the honest limit of this check. `serde` reports what it
//! refused and not *where*, so the two sides are matched on the one handle they both hold:
//! the key `serde` names for an unknown one, and the value itself for the rest. Neither is
//! a path, and a retired value that coincidentally appears in some *other* fault's message
//! at the same element would silence that fault until the retirement is fixed and the next
//! run reports it. Precision here wants an error path — `serde_path_to_error` is the usual
//! answer and is a dependency this ticket did not take unilaterally. Raised with the rest
//! as #253.

use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::checks::retired::Named;
use crate::finding::Finding;
use crate::model::{Element, Project, Track};
use crate::permissive::Loose;
use crate::report::Report;

/// Every place this document does not fit the format, as findings.
pub fn check(document: &Loose, report: &mut Report) {
    for finding in findings(document) {
        report.push(finding);
    }
}

/// The same findings, as a list.
pub fn findings(document: &Loose) -> Vec<Finding> {
    let mut faults: Vec<(Fault, Locus<'_>)> = Vec::new();
    // A fault another check already reports, which the whole-document fallback must not
    // find again.
    let mut spoken_for = false;

    // The header, with its tracks emptied: what is left is every key and value the project
    // itself states.
    if let Some(fault) = Fault::of::<Project>(&emptied(document.value(), "tracks")) {
        faults.push((fault, Locus::project()));
    }

    for track in tracks(document.value()) {
        let name = track.get("name").and_then(Value::as_str);
        if let Some(fault) = Fault::of::<Track>(&emptied(track, "elements")) {
            faults.push((fault, Locus::track(name)));
        }
        // Walked whether or not the track itself fits: a track that states no `layer` and
        // an element that misspells a key are two facts, and a reader told only the first
        // would fix it, re-run, and only then hear the second.
        for element in elements(track) {
            if let Some(fault) = Fault::of::<Element>(element) {
                // A member written in the other effect list's vocabulary is the audio
                // effects check's to report, with the list it belongs in (ADR-0169).
                if crate::checks::audio_effects::speaks_for(element, &fault.reason) {
                    spoken_for = true;
                    continue;
                }
                faults.push((fault, Locus::element(element, name)));
            }
        }
    }

    // Keyed on whether the walk found a **fault**, not on whether one survived: a retired
    // key is a fault this check deliberately leaves to the retirement, and a fallback that
    // counted findings would answer that silence by reporting the same key a second time,
    // unlocated, as the project's.
    if faults.is_empty()
        && !spoken_for
        && let Err(e) = document.strict()
    {
        // The scopes between them did not reproduce what the whole document refuses. Not
        // reachable by any construction known here, and kept anyway: this check's whole
        // reason to exist is that `strict` must never fail silently, and a guarantee that
        // holds only as far as one author's enumeration of the cases is the guarantee
        // ADR-0016 calls indistinguishable from none.
        faults.push((Fault::new(e.to_string()), Locus::project()));
    }

    let retired = crate::checks::retired::named(document);
    faults
        .into_iter()
        .filter_map(|(fault, locus)| fault.into_finding(document, &locus, &retired))
        .collect()
}

/// One thing `serde` refused, already stripped of the prefix that would repeat the locus.
struct Fault {
    reason: String,
}

impl Fault {
    fn new(reason: String) -> Self {
        Fault { reason }
    }

    /// Parse one scope's value as `T`, and keep what it refused.
    fn of<'de, T: Deserialize<'de>>(value: &'de Value) -> Option<Fault> {
        T::deserialize(value)
            .err()
            .map(|e| Fault::new(e.to_string()))
    }

    /// The finding, or nothing where the retired check already names this spelling here.
    fn into_finding(
        self,
        document: &Loose,
        locus: &Locus<'_>,
        retired: &[Named],
    ) -> Option<Finding> {
        let reason = locus.strip_prefix(&self.reason);
        let here = || {
            retired
                .iter()
                .filter(|named| named.element.as_deref() == locus.element)
        };

        let finding = match unknown_key(reason) {
            Some(key) => {
                // Exact: `serde` names the key it refused, and the retirement names the key
                // it found. The comparison is on those two strings and never on the message
                // as a whole, whose "expected one of" list is full of published key names a
                // retirement may also mention.
                if here().any(|named| named.key == key) {
                    return None;
                }
                // Not the keys `serde` names instead: they serve the *typo* branch of this
                // refusal's fork only, and on the *newer format* branch they offer the silent
                // rename the message forbids. ADR-0123.
                Finding::new("E-SCHEMA-UNKNOWN-KEY").field("key", json!(key))
            }
            None => {
                // Four of the nine retirements are retired *values* rather than keys — the
                // opaque eight-digit colour, `center-center`, `fit: none`/`fill`, and
                // `anchor` carrying a string — and for those the format's own message names
                // the value and not the key it sits under. Matched on the value, therefore,
                // which is the only handle both sides hold.
                //
                // It has to be matched on something, because the alternative is a report
                // that contradicts itself: the retirement is advise-class and states the
                // replacement, this check is refuse-class and states that no repair is
                // determined, and both would be printed about the same byte. A reader given
                // the two would be right to conclude that Montagent does not know.
                if here().any(|named| named.value.as_str().is_some_and(|v| reason.contains(v))) {
                    return None;
                }
                Finding::new("E-SCHEMA").field("reason", json!(reason))
            }
        };

        Some(locus.locate(finding.field("subject", json!(locus.subject())), document))
    }
}

/// Which scope a fault was found in, and everything a finding needs to say where.
struct Locus<'a> {
    scope: Scope,
    element: Option<&'a str>,
    track: Option<&'a str>,
}

/// The three levels the document is parsed at. Held as its own value rather than inferred
/// from which of `element`/`track` is present, because an element with no readable `id`
/// sitting in a track with no readable `name` has neither — and it is still an element,
/// which is the whole of what its finding has to say about where it is.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Scope {
    Project,
    Track,
    Element,
}

impl<'a> Locus<'a> {
    fn project() -> Self {
        Locus {
            scope: Scope::Project,
            element: None,
            track: None,
        }
    }

    fn track(name: Option<&'a str>) -> Self {
        Locus {
            scope: Scope::Track,
            element: None,
            track: name,
        }
    }

    fn element(element: &'a Value, track: Option<&'a str>) -> Self {
        Locus {
            scope: Scope::Element,
            element: element.get("id").and_then(Value::as_str),
            track,
        }
    }

    /// What the prose names — ADR-0016's own draft message, whose subject is a bare element
    /// id. The element spellings come from [`super::subject_of`] rather than from here, so
    /// that this check and the retirement cannot come to call one element two things. The
    /// track spellings are this check's own: it is the only check that reports at a track.
    fn subject(&self) -> String {
        match (self.scope, self.element, self.track) {
            (Scope::Element, id, _) => super::subject_of(id),
            (Scope::Track, _, Some(name)) => format!("the track `{name}`"),
            (Scope::Track, _, None) => "a track carrying no `name`".into(),
            (Scope::Project, _, _) => super::PROJECT.into(),
        }
    }

    /// `Element`'s deserializer prefixes its messages with the element's own id — "`a`:
    /// `layer` is 1.5, …" — so that a bare `serde` error still says which element it is
    /// about. The template already states the subject, so the prefix is dropped here
    /// rather than printed twice.
    fn strip_prefix<'m>(&self, message: &'m str) -> &'m str {
        match self.element {
            Some(id) => message
                .strip_prefix(&format!("`{id}`: "))
                .unwrap_or(message),
            None => message,
        }
    }

    fn locate(&self, finding: Finding, document: &Loose) -> Finding {
        let mut finding = finding.at_file(document.path());
        if let Some(track) = self.track {
            finding = finding.at_track(track);
        }
        if let Some(element) = self.element {
            finding = finding.at_element(element);
            if let Some(line) = document.line_of_element(element) {
                finding = finding.at_line(line);
            }
        }
        finding
    }
}

/// The key `serde` refused, where the fault is an unknown one.
///
/// Read off the message rather than out of a typed error, because `serde` has no typed
/// error: `unknown field \`x\`, expected one of \`a\`, \`b\`` is what
/// `serde::de::Error::unknown_field` writes, at every object level and for every one of
/// this format's types. The shape is asserted against real parses in
/// `tests/schema_check.rs` rather than trusted.
fn unknown_key(message: &str) -> Option<&str> {
    const MARKER: &str = "unknown field `";
    let rest = &message[message.find(MARKER)? + MARKER.len()..];
    Some(&rest[..rest.find('`')?])
}

/// The document's tracks, or nothing where it states none the walk can read. A `tracks`
/// that is not an array is the header's own fault and is reported there.
fn tracks(root: &Value) -> &[Value] {
    root.get("tracks")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

fn elements(track: &Value) -> &[Value] {
    track
        .get("elements")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

/// `value` with `key`'s array emptied, so that a container is parsed for its own keys and
/// its children are parsed as their own scopes.
///
/// Only an array is emptied. Where `key` is absent, or is not an array, the value is
/// handed over untouched — those are faults of the container itself (*"missing field
/// `tracks`"*, *"invalid type: string, expected a sequence"*) and emptying would be
/// inventing a document the file does not contain.
fn emptied(value: &Value, key: &str) -> Value {
    let Some(object) = value.as_object() else {
        return value.clone();
    };
    if !object.get(key).is_some_and(Value::is_array) {
        return value.clone();
    }
    let mut copy: Map<String, Value> = object.clone();
    copy.insert(key.to_string(), Value::Array(Vec::new()));
    Value::Object(copy)
}
