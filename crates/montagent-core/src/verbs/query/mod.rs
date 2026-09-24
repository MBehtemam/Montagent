//! `query` — *"what is true at an instant, over a range, or across a predicate"* (ADR-0011).
//!
//! All three modes are here.
//!
//! - **`--at <t>`** — the resolved stack at an instant. See [`at`].
//! - **`--from <a> --to <b>`** — the cut list. See [`cuts`].
//! - **`--where <predicate> [--census <field>]`** — the matched set and its distribution.
//!   See [`predicate`].
//!
//! ## Why the two reading modes are not the `jq` half of the verb, quite
//!
//! ADR-0011's verifier ran all eight components of the disputed `--at` output and found the
//! shell reaches the presence list, the resolved keyframe value and the previous/next
//! boundary — *"the rebuttal to the kill was itself half wrong"*. So the honest claim for
//! [`cuts`] and [`predicate`] is not that a shell cannot compute them. It is that the shell
//! computes each one **per invocation, correctly only if the invoker got the half-open
//! convention and the absent cases right**, and that getting them right is a property of one
//! tested implementation rather than of the person typing the filter.
//!
//! ## Only `--at` resolves anything
//!
//! #196's acceptance criterion was *"asserted by it working with no resolver present"*, and
//! the assertion is structural rather than a test's discipline: nothing in [`cuts`] or in
//! [`predicate`] reads a `{t,v,ease}` record, consults [`crate::stack`], or opens anything.
//! A range comes from `start` and `end`, and a predicate compares against what the document
//! writes — ADR-0070 bounds *"resolved values, never echoed fields"* to `--at` for exactly
//! that reason.
//!
//! [`at`] is the other half, and it resolves through the two modules that own those rules
//! rather than through any arithmetic of its own: [`crate::stack`] for painter's order and
//! [`crate::resolve`] for an animated value. It still opens nothing — the four components
//! that reach outside the document are
//! [#210](https://github.com/MBehtemam/Montagent/issues/210). The one file any run opens is
//! the project itself, through the same [`parse::read`] every verb reads it with.

pub mod at;
pub mod cuts;
pub(crate) mod geometry;
pub mod predicate;

use std::path::Path as FilePath;

use serde::Serialize;
use serde_json::Value;

use crate::finding::{CensusGroup, Finding};
use crate::parse;
use crate::permissive::Loose;
use crate::report::Report;

use at::At;
use cuts::Cuts;
use predicate::{Path, Predicate};

const TOOL: &str = "query";

/// What one `query` invocation is asking, as the flags arrive from either adapter.
///
/// The flags rather than a resolved mode, deliberately: which combinations are legal is a
/// rule about the verb, and ADR-0011 makes both adapters thin by construction. An adapter
/// that decided the mode would be the second place that rule lives, and the MCP surface —
/// where the arguments arrive as a JSON object with no `clap` to arrange them — would need
/// its own copy.
#[derive(Debug, Clone, Default)]
pub struct Ask {
    pub at: Option<i64>,
    pub from: Option<i64>,
    pub to: Option<i64>,
    pub predicate: Option<String>,
    pub census: Option<String>,
}

/// One `query` invocation's answer: the view, and the report every verb answers with.
pub struct Answer {
    view: Option<View>,
    report: Report,
}

impl Answer {
    pub fn report(&self) -> &Report {
        &self.report
    }

    /// The canonical JSON: the report's own object, plus the answer under `query`.
    ///
    /// Present and `null` where no answer could be built, rather than absent — `timeline`'s
    /// rule, for its reason: an absent key makes "there is no answer" indistinguishable from
    /// a version of Montagent that did not have this verb.
    pub fn to_json(&self) -> Value {
        self.report.to_json_with(
            "query",
            match &self.view {
                Some(view) => serde_json::to_value(view).unwrap_or(Value::Null),
                None => Value::Null,
            },
        )
    }
}

/// Which question was asked, and its answer.
///
/// Internally tagged, so a consumer reads `mode` and knows which other keys are there —
/// rather than sniffing for the presence of `intervals`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum View {
    /// `--at`.
    At(At),
    /// `--from --to`.
    Cuts(Cuts),
    /// `--where`, with or without `--census`.
    Matches(Matches),
}

/// The matched set, and its distribution over the censused field.
#[derive(Debug, Clone, Serialize)]
pub struct Matches {
    /// The predicate as it was written, so the answer states its own question.
    pub predicate: String,
    pub matched: Vec<Matched>,
    /// `null` unless `--census` was asked for.
    pub census: Option<Census>,
}

/// Who an element is, as all three modes name it.
///
/// The identifying fields and no more. ADR-0011's organising rule for `query` is that it
/// *"returns resolved values, never echoed fields"* — so echoing each element's whole object
/// back would be the failure that rule names, at length. The caller has the document; what
/// it did not have is which elements these are, and — under `--at` — what its animated
/// fields resolve to, which is carried beside this rather than inside it.
///
/// One struct rather than one per mode, because it is one question — a cut list's member, a
/// matched element and a member of the resolved stack are the same element, and three
/// readings of the permissive tree would be three places the answer to *"what is this
/// element called"* could drift.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Named {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub track: Option<String>,
    pub group: Option<String>,
}

impl Named {
    /// Read an element's identity off the permissive tree, however malformed it is.
    ///
    /// Every field is optional because every one of them can be missing from a document
    /// that is still worth answering about (ADR-0042) — and naming *which* way an element is
    /// malformed is `validate`'s job, never a view's.
    pub fn of(element: &Value, track: Option<&str>) -> Named {
        let string = |key: &str| element.get(key).and_then(Value::as_str).map(str::to_string);
        Named {
            id: string("id"),
            kind: string("type"),
            track: track.map(str::to_string),
            group: string("group"),
        }
    }

    /// What a census or a boundary calls this element in a list of names.
    ///
    /// ADR-0019 requires a unique `id`; where the document does not carry one the element is
    /// named by where it was found, so that two id-less elements are two names. A shared
    /// placeholder would deduplicate against itself wherever names are grouped, and would
    /// undercount the one number a census is for.
    pub fn called(&self, index: usize) -> String {
        match &self.id {
            Some(id) => id.clone(),
            None => format!("(element {index}, no id)"),
        }
    }
}

/// One element in the matched set: who it is, and the range the document gives it.
#[derive(Debug, Clone, Serialize)]
pub struct Matched {
    #[serde(flatten)]
    pub named: Named,
    pub start: Option<i64>,
    pub end: Option<i64>,
}

/// The distribution of the matched set over one field — *"four of five are 1597, one is
/// 1537"*.
///
/// The groups are [`CensusGroup`]s, the same object a finding's census carries, because
/// there is one census in Montagent and a second spelling of it would be a second thing to
/// parse. Its rules come with it: groups keep the order they were first seen in, and
/// **nothing sorts by size** — ADR-0043 is explicit that a census *"must not be worded in a
/// way that implies the larger group is the correct one"*, and the fixture's own
/// `word-08-target` is an outlier that is correct.
#[derive(Debug, Clone, Serialize)]
pub struct Census {
    pub field: String,
    pub groups: Vec<CensusGroup>,
    /// Matched elements whose document does not write the field at all.
    ///
    /// Their own list rather than a group whose value is `null`. ADR-0030 makes a
    /// defaultable field's *presence* content — omitted and explicit-at-default are
    /// different declarations — so filing "writes no `y`" under the same heading as "writes
    /// `y: null`" would collapse exactly the distinction the format keeps.
    pub absent: Vec<String>,
}

/// Answer one `query`.
///
/// Reads the file once and touches nothing else beyond that — no sidecar, no resolver, no
/// probe — in this function. `--at` is the one mode that may need a probe (#210's crop
/// rectangle), and it opens its own session; see [`at::answer`] for why that lives there
/// rather than here.
pub fn query(path: &FilePath, ask: &Ask) -> Answer {
    let project = Some(path.display().to_string());

    // The invocation is settled before the file is opened. `query --from 5 --to 1` is wrong
    // whatever the document says, and ADR-0011 keeps exit 3 apart from exit 1 and 2 so that
    // "fix the command" is never read as "fix the project".
    let question = match Question::of(ask) {
        Ok(question) => question,
        Err(reason) => {
            return Answer {
                view: None,
                report: Report::rejected(TOOL, project, reason),
            };
        }
    };

    let document = match parse::read(path) {
        Ok(document) => document,
        // ADR-0011: nothing may partially process a malformed file.
        Err(finding) => {
            return Answer {
                view: None,
                report: Report::unparseable(TOOL, project, *finding),
            };
        }
    };

    let mut report = Report::new(TOOL, project);

    if let Err(not_a_project) = document.shape() {
        // ADR-0042's shared structural predicate, reused rather than reinvented: with no
        // `tracks` there is nothing to ask about, and the ADR's own finding was that such a
        // file deserves a sentence naming the likely mismatch rather than a raw schema
        // error.
        report.push(Finding::not_a_project(
            document.path(),
            &not_a_project,
            "point query at the project file",
        ));
        return Answer { view: None, report };
    }

    match question.answer(&document) {
        Ok(view) => Answer {
            view: Some(view),
            report,
        },
        // No `ffmpeg`/`ffprobe`: `--at` alone can raise this, and only where the project
        // references a raster source at all.
        Err(reason) => {
            report.fail_internally(reason);
            Answer { view: None, report }
        }
    }
}

/// One of the three modes, with its arguments already checked.
enum Question {
    At {
        instant: i64,
    },
    Cuts {
        from: i64,
        to: i64,
    },
    Matching {
        spelling: String,
        predicate: Predicate,
        census: Option<Path>,
    },
}

impl Question {
    /// Which question the flags ask, or the one sentence saying why they ask none.
    fn of(ask: &Ask) -> Result<Question, String> {
        // One question per invocation, and the three are named in the order ADR-0011 lists
        // them. Counted rather than matched over a tuple of three booleans: the message a
        // caller gets has to name *which* two they asked, and eight arms would each have to
        // spell that out again.
        let asked: Vec<&str> = [
            (ask.at.is_some(), "`--at`"),
            (ask.from.is_some() || ask.to.is_some(), "`--from`/`--to`"),
            (ask.predicate.is_some(), "`--where`"),
        ]
        .into_iter()
        .filter(|(asked, _)| *asked)
        .map(|(_, flag)| flag)
        .collect();

        match asked.len() {
            0 => {
                return Err(
                    "`query` needs a question: `--at <t>` for the resolved stack, `--from \
                     <a> --to <b>` for the cut list, or `--where <predicate>` for the \
                     matched set"
                        .into(),
                );
            }
            1 => {}
            _ => {
                return Err(format!(
                    "{} are different questions; ask one of them per invocation",
                    asked.join(" and ")
                ));
            }
        }

        // `--census` is a distribution over a matched set, so it belongs to exactly one of
        // the three. Checked once here rather than in each of the other two arms.
        if ask.census.is_some() && ask.predicate.is_none() {
            return Err(
                "`--census` is a distribution over a matched set, so it goes with `--where`".into(),
            );
        }

        if let Some(instant) = ask.at {
            // Every instant is a legal question, including one before the project starts and
            // one after it ends: the answer there is an empty stack, which is a fact about
            // the document rather than a malformed call. A negative instant is legal for the
            // same reason — the clock is absolute integer milliseconds (ADR-0005) and
            // nothing in the format says a project may not carry one.
            return Ok(Question::At { instant });
        }

        if ask.predicate.is_some() {
            let spelling = ask.predicate.clone().unwrap_or_default();
            let predicate = Predicate::parse(&spelling)
                .map_err(|reason| format!("`--where {spelling}`: {reason}"))?;
            let census = match &ask.census {
                Some(field) => Some(
                    Path::parse(field).map_err(|reason| format!("`--census {field}`: {reason}"))?,
                ),
                None => None,
            };
            return Ok(Question::Matching {
                spelling,
                predicate,
                census,
            });
        }

        // Both halves, always. A range with one end open would have this verb choose the
        // other — the document's first instant, its `duration`, zero — and the whole point of
        // naming the boundary outside the range is that the caller never has to guess a
        // window. A verb that guessed one for them would be doing the guessing instead.
        let (Some(from), Some(to)) = (ask.from, ask.to) else {
            return Err("`--from` and `--to` are asked for together".into());
        };
        if from >= to {
            return Err(format!(
                "`--from {from} --to {to}` is empty: a range is half-open `[from, to)`, so \
                 `--to` must be greater than `--from`"
            ));
        }
        Ok(Question::Cuts { from, to })
    }

    fn answer(self, document: &Loose) -> Result<View, String> {
        Ok(match self {
            Question::At { instant } => View::At(at::answer(document, instant)?),
            Question::Cuts { from, to } => View::Cuts(cuts::cuts(document, from, to)),
            Question::Matching {
                spelling,
                predicate,
                census,
            } => View::Matches(matches(document, &spelling, &predicate, census.as_ref())),
        })
    }
}

/// The matched set, in document order, and its census where one was asked for.
fn matches(
    document: &Loose,
    spelling: &str,
    predicate: &Predicate,
    census: Option<&Path>,
) -> Matches {
    // Document order is a traversal and never a ranking (ADR-0060) — which is exactly why it
    // is the order to answer in: it is the order the elements are in the file the caller is
    // about to edit, and any other would be this verb ranking them.
    let mut matched = Vec::new();
    let mut groups: Vec<CensusGroup> = Vec::new();
    let mut absent: Vec<String> = Vec::new();

    for (index, (track, element)) in document.elements_in_tracks().enumerate() {
        let track_name = track.map(|name| Value::String(name.to_string()));
        if !predicate.matches(element, track_name.as_ref()) {
            continue;
        }

        let named = Named::of(element, track);
        let member = named.called(index);
        matched.push(Matched {
            named,
            start: element.get("start").and_then(Value::as_i64),
            end: element.get("end").and_then(Value::as_i64),
        });

        let Some(field) = census else { continue };
        let values = field.values(element, track_name.as_ref());
        if values.is_empty() {
            absent.push(member);
            continue;
        }
        // A `*` path reaches several values on one element — `runs.*.font` on a two-run
        // element reaches both — and the element is a member of each group it has a value
        // for. That is the honest reading of *"23 elements use `brand`; 1 uses `brand-old`"*
        // on an element that uses both.
        for value in values {
            match groups.iter_mut().find(|group| group.value == *value) {
                Some(group) => {
                    // An element reaching one value twice — two runs of the same font — is
                    // one member of that group and not two.
                    if group.members.last() != Some(&member) {
                        group.members.push(member.clone());
                    }
                }
                None => groups.push(CensusGroup {
                    value: value.clone(),
                    members: vec![member.clone()],
                }),
            }
        }
    }

    Matches {
        predicate: spelling.to_string(),
        matched,
        census: census.map(|field| Census {
            field: field.spelling(),
            groups,
            absent,
        }),
    }
}
