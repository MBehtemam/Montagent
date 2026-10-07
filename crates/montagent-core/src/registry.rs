//! The check registry — the thing every check ticket needs and none should own.
//!
//! One table declares, for every check the ADR series specifies: its stable code, its
//! class, ADR-0043's per-check refuse-class decision, ADR-0061's threshold provenance,
//! the ADR it comes from, and the prose template its findings render through.
//!
//! **This table is a declaration, not a schedule.** An entry says what a check's
//! findings *are*; it does not say that the check runs today. [`Status`] records which
//! half a code is in, so a reader can tell a live check from one whose implementation a
//! later ticket supplies without having to grep for call sites.
//!
//! The registry exists so that no check ticket has to invent any of this, and so that
//! the invariants the ADRs state are testable in one place rather than re-argued per
//! check. See `tests/registry.rs`.

use crate::finding::Class;

/// ADR-0043's per-check decision, made once by whoever authors the check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairClass {
    /// The correct fix is fully determined by the document, the media on disk and the
    /// published rendering semantics.
    Advise,
    /// The fix depends on knowing what the author meant. Non-bypassable.
    Refuse,
    /// ADR-0073: the finding is not about a document — its subject is the invocation,
    /// the raw bytes, or Montagent's own process, so ADR-0043's refuse/advise question
    /// ("is the fix determined by the document?") does not apply. No `repair` field is
    /// emitted at all; any remedy the condition names lives in the finding's own message
    /// text instead.
    NotAboutDocument,
}

/// ADR-0061: where the number that decides whether a finding fires comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThresholdProvenance {
    /// Every deciding number is derivable from the document itself or from the format's
    /// own fixed rendering semantics.
    Internal,
    /// A number borrowed from outside both. Admissible only at `review`/`note`, with the
    /// raw measurement as the finding's substance and the source cited **inline in the
    /// finding** — this declaration records the obligation, it does not discharge it.
    /// A finding whose check is `External` and which carries no
    /// [`Citation`](crate::finding::Citation) is a defect in that check: the prose must
    /// never supply a citation the canonical JSON does not carry.
    External {
        source: &'static str,
        adr: &'static str,
    },
}

/// Whether the check behind a code runs today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The check is implemented and can fire.
    Live,
    /// The ADR series specifies the finding; a later ticket supplies the check.
    Declared,
}

/// How a check's sibling census renders in the text form. ADR-0111.
///
/// The canonical JSON carries every member of every group whatever the mode; this is a
/// text-form choice only. It is made **per code**, by one test: *can the reader get from a
/// group's value to its members using only the document and a shell?* A value written
/// literally in the document passes, because it can be searched for, and its group is
/// `Counted`. A value that cannot be searched for — an invisible codepoint, a
/// normalization form, a stretch of the clock, an object — fails it, and its group is
/// `Named`. Running another verb does not pass the test: every value would pass through
/// some verb, and the rule would be count-only again.
///
/// It is declared rather than inferred from the value's JSON type, because searchability
/// does not follow type: a height (a number) can be searched for and a codepoint name (a
/// string) cannot, and `E-RETIRED-KEY` mixes objects and strings under one code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CensusMode {
    /// `n at <value>`: the value is the reader's route to the members.
    Counted,
    /// `n at <value>` and then the members themselves, bounded per group.
    Named,
}

/// A named group of checks a run executes. ADR-0112.
///
/// A report records the sets that **completed**, never the ones its verb is declared to
/// run, so a run that stopped part-way says so on the wire. Which classes a set can raise
/// is [`CheckSet::classes`], read off the codes below that name it: the registry is the
/// only table of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckSet {
    /// `validate`'s checks that read only the file and its fonts.
    Document,
    /// `validate`'s checks that need `ffprobe`: source, fit, chroma-on-disk.
    Disk,
    /// The key-order/line-break check alone, as `fmt` runs it.
    Layout,
    /// `compare`'s own checks.
    Drift,
    /// `verify`'s measurement of the rendered file against the document (ADR-0117). A
    /// fifth set beside ADR-0112's four: `verify` runs none of `validate`'s checks, and
    /// printing it as the check-free case would drop the zeros a clean measurement earns.
    Deliverable,
}

impl CheckSet {
    /// Every set, in the order the report names them.
    pub const ALL: [CheckSet; 5] = [
        CheckSet::Document,
        CheckSet::Disk,
        CheckSet::Layout,
        CheckSet::Drift,
        CheckSet::Deliverable,
    ];

    /// The spelling on the wire.
    pub fn as_str(self) -> &'static str {
        match self {
            CheckSet::Document => "document",
            CheckSet::Disk => "disk",
            CheckSet::Layout => "layout",
            CheckSet::Drift => "drift",
            CheckSet::Deliverable => "deliverable",
        }
    }

    /// The set a wire spelling names, for the text renderer, which sees only the JSON.
    pub fn named(name: &str) -> Option<CheckSet> {
        CheckSet::ALL.into_iter().find(|set| set.as_str() == name)
    }

    /// Every class some code in this set may emit — derived, so a check that learns a new
    /// class moves the summary line with it rather than disagreeing with a second table.
    pub fn classes(self) -> impl Iterator<Item = Class> {
        [Error, Review, Note, Unchecked, Layout, Drift]
            .into_iter()
            .filter(move |class| {
                CHECKS
                    .iter()
                    .any(|spec| spec.sets.contains(&self) && spec.classes.contains(class))
            })
    }
}

/// One check's declaration.
#[derive(Debug, Clone, Copy)]
pub struct CheckSpec {
    /// The stable code. ADR-0006: it is the contract keeping the JSON and the prose in
    /// step, the handle for suppressing a class, and the identity a future `compare`
    /// diffs on.
    pub code: &'static str,
    /// The classes this check may emit, most usual first.
    ///
    /// Usually one. It is a set because ADR-0006 is explicit that **severity is computed
    /// from the consequence at an instant, not from the check**: one visual gap is
    /// `review` because nothing on any visual track covered it, and the next is a `note`.
    /// A registry that pinned one class per code would make that unrepresentable, which
    /// is the dichotomy ADR-0006 opens by calling wrong.
    pub classes: &'static [Class],
    /// Required on `error`, forbidden elsewhere. Guarded by the completeness test.
    /// `RepairClass::NotAboutDocument` (ADR-0073) satisfies the requirement while
    /// emitting no `repair` field — reserved for the process-level codes whose subject
    /// is not the document.
    pub repair: Option<RepairClass>,
    pub threshold: ThresholdProvenance,
    /// The ADR this check comes from.
    pub adr: &'static str,
    /// The prose template. `{name}` interpolates a field, or one of the location keys
    /// `file`, `line`, `column`, `byte_offset`, `track`, `element`.
    pub template: &'static str,
    pub status: Status,
    /// How the census renders in the text form, on a code that emits one; `None` on every
    /// other code. ADR-0111.
    pub census: Option<CensusMode>,
    /// The check sets whose run can raise this code (ADR-0112). Empty on a code no check
    /// raises: a refusal, a failure of Montagent's own, or a verb's answer about something
    /// other than the document's checks — `frame` raises `E-NOT-A-PROJECT` while running
    /// none. The `L-` codes sit in two, because `fmt` runs `validate`'s layout check alone.
    pub sets: &'static [CheckSet],
}

use CensusMode::{Counted, Named};
use CheckSet::{Deliverable, Disk, Document};
use Class::{Drift, Error, Layout, Note, Review, Unchecked};
use RepairClass::{Advise, NotAboutDocument, Refuse};
use Status::{Declared, Live};
use ThresholdProvenance::{External, Internal};

const CHECKS: &[CheckSpec] = &[
    // ---- This ticket's own checks (#188). -----------------------------------------
    CheckSpec {
        code: "E-PARSE",
        classes: &[Error],
        // ADR-0073 (#224): not about a document — the bytes could not be read as JSON,
        // so there is no document for either arm of ADR-0043's question to be asked of.
        // The caret in the template below is the only repair there is to give.
        repair: Some(NotAboutDocument),
        threshold: Internal,
        adr: "ADR-0011",
        template: "{file} is not valid JSON: {reason}, at line {line}, column {column} (byte {byte_offset}).",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        code: "E-READ",
        classes: &[Error],
        // ADR-0073 (#224): not about a document — the file was never opened, so there is
        // no document whose author could have meant anything. The OS-derived move
        // (`{advice}` below) is message text, not a structured repair.
        repair: Some(NotAboutDocument),
        threshold: Internal,
        adr: "ADR-0011",
        template: "{file} could not be read: {reason} — {advice}.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        // ADR-0042's refusal. The reported hazard is wrong-file destruction — `fmt`
        // pointed at a transcript export or a beats export will rewrite it — so the
        // proportionate fix is an identity check, not a correctness gate. The three keys
        // it names are `crate::permissive::REQUIRED`, and the template says them rather
        // than dumping a raw schema error, which is the message-quality gap the ADR found.
        code: "E-NOT-A-PROJECT",
        classes: &[Error],
        // Advise: there is no document whose author could have meant anything, because
        // the document is not a project. The next move — point the tool at the project
        // file — follows from the condition itself. Shares ADR-0073's fault line with
        // `E-READ`/`E-PARSE`/`E-INVOCATION`/`E-INTERNAL` but is not reclassified by it —
        // out of that ADR's scope (#224), and out of ADR-0080's, which moved only
        // `E-PROJECT-EXISTS`. Left for whoever next touches this code.
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0042",
        template: "{file} does not look like a Montagent project file — no {missing}.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        // `create_project` scaffolds, and a scaffold that overwrites is a scaffold that
        // deletes a project. Its own code rather than `E-WRITE` or a bare `E-INVOCATION`,
        // because this is the one failure of a write tool that is not a failure at all: the
        // file is intact, and the agent has learned that the thing it was about to create
        // already exists — which, mid-session, is usually the answer it wanted.
        //
        // ADR-0080 ratifies the refusal and replaces the reading it shipped with: this
        // is a bad invocation, not a defective project, so it exits 3 and not 1.
        code: "E-PROJECT-EXISTS",
        classes: &[Error],
        // `NotAboutDocument` (ADR-0080, on the fault line ADR-0073 left open for it): the
        // subject is the path this call was given, not any document — the file that is
        // there is intact and the project being scaffolded does not exist. The next move
        // — write somewhere else, or edit the file that is already there — is a repair to
        // the command, so it lives in the template text rather than in a `repair` field.
        repair: Some(NotAboutDocument),
        threshold: Internal,
        adr: "ADR-0080",
        template: "{file} already exists; `create_project` never overwrites. Edit it, or \
scaffold somewhere else.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        code: "E-INVOCATION",
        classes: &[Error],
        // ADR-0073 (#224): not about a document — exit 3's next move is "fix the
        // command" (ADR-0011), a repair to the invocation, not to anything Montagent can
        // write. The usage text already states it; no structured repair to add.
        repair: Some(NotAboutDocument),
        threshold: Internal,
        adr: "ADR-0011",
        template: "{reason}",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        // ADR-0105 (#412): `frame`'s range mode holds more visual states than the sheet can
        // show legibly, so it refuses rather than thin, split or reshape (ADR-0095). One code
        // for every limit that can bind — ADR-0095's 140 px tile width, ADR-0098's 8 px served
        // type, or either reached through the keyframe opt-in (#407) — because all three share
        // one next move, a narrower range; `limit` says which bound. Its own code rather than a
        // bare `E-INVOCATION` (the shape `preview`'s budget refusal takes) because the remedy is
        // a list of ranges, and an agent loops over `sub_ranges` without parsing prose.
        code: "E-SHEET-OVERFLOW",
        classes: &[Error],
        // `NotAboutDocument` (ADR-0073): the document is legal and unchanged, and the subject
        // is the range the caller typed. Reaches exit 3 through `Report::refused_invocation`.
        repair: Some(NotAboutDocument),
        threshold: Internal,
        adr: "ADR-0105",
        //
        // Live since #488, with `from`, `to`, `states` (the visual states needing a tile),
        // `fits` (the most tiles the sheet holds), `limit` and `limit_px` (ADR-0125), and
        // `sub_ranges`: the fewest `{from, to}` ranges that each fit, covering the range
        // exactly, which the text renders as a prose list (ADR-0126). `limit` is `tile-width`
        // or, since #490, `type-floor`: the tiles are wide enough and a label's numeric core
        // still cannot be drawn at 8 px beneath them (ADR-0098 §5).
        //
        // Since #491, a refusal under `--keyframes` adds `keyframe_tiles`,
        // `fits_without_keyframes` and `keyframe_tiles_admitted` (ADR-0106 D9), and only then
        // do the template's `{?…}`/`{!…}` clauses print: the cheaper remedy may be dropping
        // the flag, and nothing else in the refusal tells the two cases apart (ADR-0129).
        template: "`frame` refused [{from}, {to}): its {states} visual states need a tile each\
{?keyframe_tiles} and `--keyframes` adds {keyframe_tiles} keyframe tiles{/keyframe_tiles}, \
and the sheet holds {fits} before it passes its {limit} limit of {limit_px} px served, \
below which a tile or its label is not legible. It never thins, splits or reshapes the \
sheet (ADR-0095).{?fits_without_keyframes} Without `--keyframes` the range fits; with it, \
the sheet holds {keyframe_tiles_admitted} keyframe tiles beside the run tiles and never \
drops one to fit.{/fits_without_keyframes}{!fits_without_keyframes} It does not fit without \
`--keyframes` either; with it, the sheet holds {keyframe_tiles_admitted} keyframe tiles \
beside the run tiles and never drops one to fit.{/fits_without_keyframes} Ask for these \
instead, each a sheet that fits: {sub_ranges}.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        code: "E-INTERNAL",
        classes: &[Error],
        // ADR-0073 (#224): not about a document — Montagent itself broke. Refuse's "no
        // flag can lift it" doesn't fit a condition a retry might clear, and there is no
        // document for a repair to be determined from either way.
        //
        // Narrowed by ADR-0091 (#368): a program that was resolved and then would not
        // run — a corrupt or incompatible binary — is still this code. A program never
        // found on `PATH` at all is `E-TOOL-MISSING` instead.
        repair: Some(NotAboutDocument),
        threshold: Internal,
        adr: "ADR-0011",
        template: "Montagent failed internally: {reason}",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        // ADR-0091 (#368): a missing `ffmpeg`/`ffprobe` is `E-INTERNAL` today, and reads
        // as "Montagent broke" on the single most likely first-run failure there is.
        // ADR-0009 makes "bring your own `ffmpeg`" a deliberate, documented part of the
        // shape — a user with no `ffmpeg` on `PATH` has an unconfigured environment, not
        // a broken tool. Same exit code (70, unchanged by ADR-0011), same
        // `NotAboutDocument` class (ADR-0073) — a distinguishable code, so an agent (or
        // `compare`) can tell "you don't have this installed" apart from "it crashed."
        code: "E-TOOL-MISSING",
        classes: &[Error],
        repair: Some(NotAboutDocument),
        threshold: Internal,
        adr: "ADR-0091",
        template: "{reason}",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        // ADR-0115 (#477, #479): `E-TOOL-MISSING`'s sibling, for an `ffmpeg` that is on
        // `PATH` and runs, and fails the tool qualification — one null encode through the
        // floor's arguments (`montagent_render::floor`). Its own code because the remedy is
        // to *upgrade*, which `-MISSING` would misname and `E-INTERNAL` would blame on
        // Montagent. Same exit 70 and `NotAboutDocument` as its sibling, for its reason.
        //
        // In `validate` it is an `error` — `render` is guaranteed to refuse — and it belongs
        // to no check set: it is refusal-shaped, not a question asked of the project.
        code: "E-TOOL-UNSUPPORTED",
        classes: &[Error],
        repair: Some(NotAboutDocument),
        threshold: Internal,
        adr: "ADR-0115",
        template: "{reason}",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        // ADR-0109 (#440): the MCP caller sent `notifications/cancelled` and `render` or
        // `preview` walked away rather than publish. Its own code rather than `E-INTERNAL`,
        // for `E-TOOL-MISSING`'s reason — "you stopped it" and "it crashed" are different
        // next moves — and `NotAboutDocument` because the project did not stop the run.
        //
        // Usually unread: `rmcp` drops the response to a cancelled request and the spec tells
        // the client to ignore a late one. It exists so the report the verb *does* produce
        // states the one thing ADR-0109 decides — the disk was not touched — and so a test
        // can name the condition.
        code: "E-CANCELLED",
        classes: &[Error],
        repair: Some(NotAboutDocument),
        threshold: Internal,
        adr: "ADR-0109",
        template: "{reason}",
        status: Live,
        census: None,
        sets: &[],
    },
    // ---- The disk half of `validate`: the probe's findings (#190) and the check that
    // ---- reads them (#203). --------------------------------------------------------
    CheckSpec {
        // Not a new check: this is the code for ADR-0002/ADR-0006's standing *"every
        // `source` must resolve"* error, which ADR-0053 says "covers both a missing local
        // file and, by extension, an absolute path that doesn't resolve on the current
        // machine — no new finding code is introduced here". The check had no code until
        // something implemented it; this registers the one it fires under.
        code: "E-SOURCE-MISSING",
        classes: &[Error],
        // Advise, on E-READ's reasoning: there is no document whose author could have
        // meant a file that is not there, and the next move — correct the path, or put
        // the file where the document says — follows from the condition itself. ADR-0056
        // narrows what may reach this code: a *confirmed* absence only. A probe that
        // could not complete is `U-SOURCE-UNPROBEABLE`, never this.
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0053",
        // "Is not there", not "could not be resolved": the same code fires on a 404, where
        // nothing about path resolution failed. It names both the spelling the document
        // used — the string an agent has to edit — and the place that spelling resolved to,
        // because a relative path and the directory it resolved against are two different
        // things to get wrong (ADR-0053).
        template: "{source} is not there: {detail}. Looked for it at {resolved}.",
        status: Live,
        census: None,
        sets: &[Disk],
    },
    CheckSpec {
        // ADR-0056: an existence-only result "is a strictly weaker claim than a local
        // probe's, and reporting it identically would be exactly the false confidence
        // ADR-0006 was written to prevent". Its own code rather than a flag on the one
        // above, so the weaker claim can never occupy the slot a confirmed duration does.
        code: "U-SOURCE-EXISTENCE-ONLY",
        classes: &[Unchecked],
        repair: None,
        threshold: Internal,
        adr: "ADR-0056",
        template: "{source}: existence confirmed; duration and dimensions NOT CHECKED. {detail}",
        status: Live,
        census: None,
        sets: &[Disk],
    },
    CheckSpec {
        code: "E-SOURCE-OVERRUN",
        classes: &[Error],
        // The document says the source is one length and the disk says another; which
        // of the two is the mistake is not readable off either.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0006",
        // Names the axis it measured against, because ADR-0011 returns four durations and
        // forces the caller to pick: a finding that said "the source is 65216 ms" without
        // saying *which* 65216 would invite the reader to check it against the other one.
        // Two arms (ADR-0157 §2): a declared source range reaching past the file, or a
        // `source_time` curve resolving outside `[0, duration)` at a painted frame instant —
        // named at the first, with the side it crossed.
        template: "{element}: `{source}` holds {probed_duration} ms ({axis}){?source_end}, and \
the declared source range {source_start}..{source_end} ({declared_source_span} ms) reaches \
{over_by} ms past it{/source_end}{?instant}, and its `source_time` resolves to {source_time} ms \
at the painted instant {instant} ms, {side}{/instant}.",
        status: Live,
        census: None,
        sets: &[Disk],
    },
    // ---- The anchor's resolution and the checks over it (#198). --------------------
    //
    // ADR-0019 states one bullet — *"target does not exist, or is itself an anchor →
    // error"* — and this is three codes rather than one. Two reasons, and both are this
    // table's own rules rather than a preference. ADR-0043's uniformity rule fixes the
    // repair class **per code**, and a chained target's fix is not determined where a
    // dangling one's next move is; one code could not carry both. And a code is
    // ADR-0006's *"handle for suppressing a class"* and the identity `compare` will diff
    // on, so three conditions an author fixes three different ways are three handles.
    // The split is surface the ADR series has not ratified — raised as #243.
    CheckSpec {
        code: "E-ANCHOR-MISSING",
        classes: &[Error],
        // Advise, on `E-SOURCE-MISSING`'s reasoning: a reference that does not resolve is
        // the same shape whether it names a file or an `id`, and the next move — name an
        // element that is there, or state the layer outright — follows from the condition
        // itself rather than from knowing which element was meant.
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0019",
        template: "{element}: the anchor `{side}` names `{target}`, which is not an element \
in this project.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        code: "E-ANCHOR-SELF",
        classes: &[Error],
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0019",
        // Its own code rather than an instance of the one above, which it would otherwise
        // be — the element *is* in the document, so "no such element" would be false — and
        // because a self-reference reads as a copy-paste rather than as a typo, which is a
        // different thing to go looking for.
        template: "{element}: the anchor `{side}` names `{target}`, which is the element \
itself.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        code: "E-ANCHOR-CHAIN",
        classes: &[Error],
        // Refuse, where its two siblings advise. The author wanted this element to track
        // the target, and the two ways to make the document legal — pin this element to an
        // integer, or re-point it at what the target anchors to — are not the same edit and
        // the document does not say which was meant. Pinning also reintroduces exactly the
        // hand-maintained `panel.layer = title.layer - 1` arithmetic ADR-0004 adopted
        // anchoring to remove, so it is not the safe default it looks like.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0019",
        // "Is an object", not "is an anchor": ADR-0019 states the one-hop rule structurally
        // — *"an anchor's target's own `layer` must not itself be an object"* — and a
        // target carrying `{"below": 3}` fails that test exactly as a good anchor does.
        template: "{element}: the anchor `{side}` names `{target}`, whose own `layer` is an \
object rather than a plain integer. An anchor resolves in exactly one hop, so a target that \
is itself anchored is refused rather than walked.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0019's `review`: legal, renders, and *"the hardest class of error to catch by
        // reading"* — a completed-looking edit that is a permanent no-op. Every number the
        // reader needs to judge it is inline, because the point is that nothing on any
        // frame will show them.
        code: "R-ANCHOR-NO-OVERLAP",
        classes: &[Review],
        repair: None,
        threshold: Internal,
        adr: "ADR-0019",
        template: "{element}: the anchor `{side}` names `{target}`, and the two are never on \
screen together — {element} runs {start}..{end} ms and {target} runs {target_start}..\
{target_end} ms. It resolves to layer {layer} and can change nothing.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    // ---- The layer tie (#209, ADR-0060). -------------------------------------------
    CheckSpec {
        // ADR-0060: two elements resolving to one layer whose boxes overlap in both time
        // and space. Not `review`, which reserves itself for *"legal, renders, and a human
        // must look at a frame to know if it was meant"* — there is no single frame to
        // look at, because which one renders is implementation-defined, and two tools have
        // already disagreed about the same document.
        code: "E-LAYER-TIE",
        classes: &[Error],
        // Refuse. The fix costs one field — an integer `layer` override or an anchor
        // naming the other element — and *which* of the two goes in front is exactly the
        // fact the document does not carry. An advised value would be this table picking a
        // draw order, which is the thing ADR-0060 refuses to define.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0060",
        template: "{element} and {other} both resolve to layer {layer}, and their boxes \
overlap at {instant} ms — {overlap_width}×{overlap_height} px at ({overlap_x}, \
{overlap_y}). Nothing in the document says which draws in front. State one: give either \
element its own integer `layer`, or an anchor naming the other.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // The tie check answers in rectangles, and a rotated element's footprint is a
        // parallelogram — the refusal `crate::verbs::query::geometry` already records for
        // the same reason. Its bounding box would refuse projects whose elements never
        // touch; silence would pass a tie nobody looked at. ADR-0006 has a third answer,
        // and this is it: a check that could not run says so, and gates nothing.
        //
        // ADR-0060 does not name this case, so the code is surface the ADR series has not
        // ratified — raised as #282 rather than left to be discovered from this table.
        // #282 also records what this refusal does *not* do: it is per pair and is never
        // narrowed by a bounding-box pre-test, so a rotated element ties `unchecked` with
        // something across the frame it could not possibly touch.
        code: "U-LAYER-TIE-ROTATED",
        classes: &[Unchecked],
        repair: None,
        threshold: Internal,
        adr: "ADR-0060",
        template: "{element} and {other} both resolve to layer {layer}, and {rotated}'s \
resolved `rotation` is {degrees}° at {instant} ms. Whether their boxes overlap is \
unanswered: the tie check measures rectangles, and a rotated footprint is not one.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    // ---- The closed schema, turned into findings (#244). ---------------------------
    //
    // ADR-0017 closes the schema and ADR-0016 makes the unknown-key error the whole
    // migration mechanism, *"because an optional signal is indistinguishable from no
    // signal"*. Two codes rather than one, on this table's own rule that a code is
    // ADR-0006's handle for suppressing a class and the identity `compare` will diff on:
    // ADR-0016 settles the unknown key's message and its repair and settles neither for a
    // value that does not fit its type, and the two are acted on differently — one asks
    // whether the *binary* is stale, the other asks the author for a value. The split, and
    // the classes below, are surface the ADR series has not ratified — raised as #253.
    //
    // **Neither carries a sibling census, and ADR-0043 says a refuse-class finding does.**
    // `E-ANCHOR-CHAIN` is already refuse-class without one, so the practice
    // is that the census attaches where a sibling group exists; here it cannot honestly be
    // measured. `serde` stops at the first fault in an object, so a census counted over
    // what this check reported would group "elements whose *first* fault was this key" and
    // print it as "elements carrying this key" — and ADR-0006's whole posture is that a
    // stated number is a measured one. A second, independent scan for the key would be
    // exact at the top level of an element and blind inside a `run`, which is the
    // multiplicity ADR-0017 names as the one that matters. Recorded here and in #253 rather
    // than discharged with a number that is right about some documents.
    CheckSpec {
        code: "E-SCHEMA-UNKNOWN-KEY",
        classes: &[Error],
        // Refuse, and ADR-0016 writes the guarantee into the message itself: *"Check your
        // Montagent version before removing it. Do not delete the key to make the file
        // validate."* Deletion is the one repair an agent would reach for and the one the
        // ADR forbids, because the key may be a newer revision's and this binary cannot
        // tell. ADR-0043's uniformity rule then settles the rest: the check matches a typo
        // and a newer-format key alike, and if any instance is load-bearing every instance
        // refuses.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0016",
        // ADR-0016's own text, which is measured rather than drafted: of nine agents met by
        // a stale binary, the arm naming the binary preserved and compensated where the arm
        // printing a bare key list squashed the photos. "Not one it has retired" is the
        // second half of the ADR's two-case requirement, and it is true here because the
        // check asks `retired::named` before it speaks.
        //
        // **It does not list the keys the format publishes here.** It did, and that list
        // bore on the *typo* branch of the fork only: on the *newer format* branch it is a
        // menu for the silent rename this message forbids — a repair by another name
        // (ADR-0120, as narrowed by ADR-0123). ADR-0016's arms met 0/9 deletions without it.
        template: "{subject}: unknown key `{key}` — not a key this Montagent knows, and not one \
it has retired. It may belong to a newer format revision than this binary implements. Check \
your Montagent version before removing it. Do not delete the key to make the file validate.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // Everything else the published types refuse: a value of the wrong type, a required
        // key absent, a value outside a closed vocabulary, a `null` where the convention
        // omits. One code rather than three, because the three are one condition — *this
        // value is not one the format publishes* — and one repair: write a value it does.
        code: "E-SCHEMA",
        classes: &[Error],
        // Refuse: there are bytes and a tree here (unlike `E-PARSE`, ADR-0073's — the
        // document parsed), but the part of the tree the finding is about is exactly the
        // part the format cannot read — so the fix is whatever the author meant by it,
        // which is the condition ADR-0043 reserves for refusal. The prose renderer
        // already states the guarantee in words on every refuse-class finding, so the
        // template does not.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0017",
        // The reason is the format's own words wherever the types state one — `#FBF3E3FF is
        // the opaque form of #FBF3E3`, `not integer milliseconds`, `written as null; omit
        // the key instead` — rather than a sentence this table would have to keep in step
        // with them.
        template: "{subject} does not fit the published schema: {reason}.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    // ---- Declared by the ADR series; the checks themselves are later tickets. ------
    CheckSpec {
        code: "E-RETIRED-KEY",
        classes: &[Error],
        // ADR-0043's founding instance: 6 of 8 `gravity` deletions were geometric
        // no-ops and 2 silently changed the picture, and the fact separating them is
        // not in the document.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0043",
        template: "{subject}: `{key}` is a retired spelling, carrying {value}; the format now \
says this with {replacement}. Surface this finding verbatim to whoever is operating Montagent; \
do not repair it by ordinary file edit.",
        status: Live,
        census: Some(Named),
        sets: &[Document],
    },
    CheckSpec {
        // The advise-class half of the same mechanism, and a separate code because it has
        // to be: ADR-0043's uniformity rule and the two-class split cannot both hold
        // inside one code. Every member is a transposition whose whole input is the
        // string already in the file — the bare `mask` key never had accepted semantics
        // for a repair to misread (ADR-0068), and the rest are one value spelled two ways.
        code: "E-RETIRED-SPELLING",
        classes: &[Error],
        repair: Some(Advise),
        threshold: Internal,
        // ADR-0043, like its refuse-class twin: naming the replacement is ADR-0016's
        // mechanism and is common to both, but what separates the two codes — and what
        // makes this one a code at all — is ADR-0043's class and its uniformity rule.
        // ADR-0068 is the ADR that first classified a retirement into this half.
        adr: "ADR-0043",
        template: "{subject}: `{key}` is a retired spelling, carrying {value}. Write \
{replacement} instead.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        code: "E-KEYFRAME-EASE",
        classes: &[Error],
        // Presence is a pure function of position (ADR-0038), so the fix is the
        // position — fully determined by the document.
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0038",
        template: "{element}: `ease` presence does not match keyframe position on `{property}` at t={t}.",
        status: Declared,
        census: None,
        sets: &[],
    },
    // ---- The structural time checks, and `slack` (#197). ---------------------------
    //
    // Four codes for what reads as two rules, and the split is this table's own: ADR-0043
    // fixes the repair class **per code**, and ADR-0006 makes a code the handle an author
    // suppresses and `compare` diffs on. An overlap and a gap are one traversal and two
    // findings *because ADR-0004 says so* — "the overlap rule needs a validator, and it
    // must distinguish *overlap* from *gap*: a forgotten shift leaving a silent gap passes
    // an overlap-only check." A `speed` mismatch and an `overrun` that covers nothing are
    // two arms of ADR-0020's one invariant that repair opposite ways round, and they are
    // written as **two checks** rather than one branching function precisely so that
    // ADR-0043's "granularity is per check, not per instance" stays literally true of
    // them: `crate::checks::speed` splits at the question, and each half then answers
    // uniformly for every instance it matches.
    //
    // All four spellings, their class and their repair class are ratified by ADR-0076
    // (#255) — ADR-0004, ADR-0006 and ADR-0020 each state the condition, and ADR-0076 is
    // where the code, the class and the two settled questions (`N-TRACK-GAP` stays `note`;
    // a gap is bounded by its own track's elements, never the project's ends) became spec
    // rather than only this table's argument.
    CheckSpec {
        code: "E-TRACK-OVERLAP",
        classes: &[Error],
        // Refuse. Two elements of one track share an instant, and *which of the two is in
        // the wrong place* is not readable off the document: an edit that stretched the
        // first and an edit that dragged the second produce the identical file. ADR-0004
        // bought the constraint deliberately — "two elements that genuinely should overlap
        // now need two tracks" — and moving one to a second track is a third repair the
        // document cannot choose between either.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0004",
        // **One finding per track, not per overlapping pair** (ADR-0100). Per-pair
        // emission was quadratic in one authorial mistake — fifteen elements on one track
        // printed 105 sentences — and the census under this one carries the member set
        // that those 105 sentences spelled out pairwise. `overlap` survives the collapse
        // as contended track time, which is the per-pair field's own quantity summed over
        // the track rather than over a pair.
        template: "{count} elements in track `{track}` overlap, in {sets}; {overlap} ms \
of the track is covered more than once.",
        status: Live,
        census: Some(Named),
        sets: &[Document],
    },
    CheckSpec {
        // A gap is legal, ordinary content — "the silence between two narration lines is a
        // gap" — and ADR-0006 restates that it is *never* an error in the same breath as
        // the severity rule that could be misread as overturning it. It is reported
        // because ADR-0004 requires the validator to distinguish one from an overlap
        // rather than pass it in silence, and it collapses to one counted line because
        // ADR-0006's noise budget is a safety property.
        //
        // `note` and not `review`: whether the picture actually goes black across a gap is
        // a question about *other tracks*, which this check does not look at. ADR-0006
        // computes that severity "from the consequence at an instant" and hands the
        // computation to the cross-track coverage question — `R-VISUAL-GAP`, ADR-0018 and
        // #200. One gap, two checks, and the one that can see the whole frame owns the
        // `review`.
        code: "N-TRACK-GAP",
        classes: &[Note],
        repair: None,
        threshold: Internal,
        adr: "ADR-0004",
        template: "nothing in track `{track}` from {from} ms to {to} ms ({size} ms), \
between `{after}` and `{before}`.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        code: "E-SPEED-MISMATCH",
        classes: &[Error],
        // Advise, and ADR-0020 is the one that decides it rather than this table:
        // "`start`/`end` and `source_start`/`source_end` are authoritative and must never
        // move silently to satisfy this check ... `speed` is the free variable: when the
        // invariant fails, `validate`'s error reports the `speed` value that would satisfy
        // it, and the agent edits `speed`, never the spans." One free variable is a fully
        // determined fix.
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0020",
        // Every number inline, including the two that are not in the file: the played
        // duration and the timeline span it is supposed to equal. ADR-0045 is what makes
        // `played` trustworthy — it is evaluated in exact rational arithmetic, so the
        // number printed here is the number the invariant was decided on.
        template: "`{element}`: source range {source_start}..{source_end} ({source_span} ms) \
at `speed` {speed} plays for {played} ms, and the timeline range {start}..{end} is \
{timeline_span} ms.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // The other arm of the same invariant: "with `overrun` declared, `end - start` must
        // be strictly *greater than* that value. An `overrun` declared on an element that
        // doesn't need it — where `end - start` doesn't exceed the played duration — is
        // itself a `validate` error" (ADR-0020).
        code: "E-OVERRUN-UNNEEDED",
        classes: &[Error],
        // Refuse, where its sibling advises, and that is the reason they are two codes.
        // The element is too short for its own `overrun`, and the document does not say
        // whether the author meant the element to run longer or meant no `overrun` at all
        // — deleting the key and extending `end` are different videos.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0020",
        template: "`{element}`: `overrun` is \"{overrun}\", and there is nothing past the \
source to cover — {source_span} ms at `speed` {speed} plays for {played} ms, and the \
timeline range {start}..{end} is {timeline_span} ms.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        code: "R-VISUAL-GAP",
        // ADR-0018 **withdrew** ADR-0006's frame-wide union — it fired on zero of the
        // fixture's eleven real visual gaps — and replaced it with group-scoped pairing:
        // for every `group` carrying both an audio and a visual element, report either
        // side's time-union left uncovered by the other's, symmetric. Severity still
        // varies per instance, as ADR-0006's "computed from the consequence, not the
        // check" always meant, but the split is no longer union-membership — it is
        // direction: audio outrunning its group's visual is `review` (ADR-0018's own two
        // founding defects, and zero of them on the committed fixture), the reverse is
        // `note` (an ordinary narration pause, 21 of them on the fixture, the same kind
        // of fact `N-TRACK-GAP` already reports). See `crate::checks::coverage`.
        classes: &[Review, Note],
        repair: None,
        threshold: Internal,
        adr: "ADR-0018",
        template: "`{group}`: nothing on {missing} from {from} ms to {to} ms ({size} ms), \
while {elements} plays on {active}.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // A worked counterexample to the tempting rule that a code's prefix *is* its
        // class: this one is `R-` and `note`. ADR-0058 names the shape as
        // `R-<subject>-<symptom>` — a symptom noun, never a verdict — and then resolves
        // its severity to `note` separately, because you do not have to look at a frame
        // to know an oversized box was not meant. The class comes from this table and
        // from nowhere else.
        code: "R-BOX-SLACK",
        classes: &[Note],
        repair: None,
        threshold: Internal,
        adr: "ADR-0058",
        template: "text \"{element}\" declares height {declared_height}; computed block height is \
{computed_height} ({derivation}) — slack {slack} ({slack_percent}%).",
        status: Live,
        census: Some(Counted),
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0087's ink seam (#325). ADR-0007 makes a line's slot *"the largest `size`
        // among the runs on that line × `line_height`"* — two numbers the document
        // declares, never the font's ink — and ADR-0058's `R-BOX-SLACK` above inherits the
        // same blindness, because it compares declared numbers against each other. On a
        // script whose marks stack, the two readings come apart: Thai's base + upper vowel
        // + tone mark + lower vowel put one line's ink through the next line's at a
        // `line_height` of 1.1 that is correct for Latin in the same face, with every field
        // individually valid and nothing in the tool able to say so.
        //
        // `review`, which is ADR-0006's definition read literally: it is legal, it renders,
        // and you must look at a frame to know whether it was meant — deliberate tight
        // tight `line_height` is a real typographic choice, and ADR-0087 is explicit that this may
        // never be an `error`, which is reserved for *guaranteed wrong*.
        code: "R-LINE-INK-COLLISION",
        classes: &[Review],
        repair: None,
        // Internal. The deciding number is **zero** — one line's ink reaching past where
        // the next line's begins — and both sides of that comparison are derived from the
        // document and the font files it declares. Nothing here is borrowed from outside
        // the format: ADR-0087 rejected a script-aware `line_height` floor precisely
        // *because* it would have been an external constant, and one the measurements show
        // is a property of the face (1.3 on Noto Sans Thai, 1.6 on Sarabun) rather than of
        // the script.
        threshold: Internal,
        adr: "ADR-0087",
        // **No sibling census**, on `E-RUN-SPLIT-CLUSTER`'s reasoning that a census attaches
        // where a real sibling group exists. One could be built here — the other elements
        // set in this same chain — but it would not be inert: ADR-0087 records that two
        // repairs are legitimate and that the document does not determine which, and a
        // census grouping by `font` reads as an argument for the one that changes the face.
        template: "{element}: at line_height {line_height} in `{font}` at size {size}, line \
{above}'s ink reaches {overlap} px past where line {below}'s begins ({collisions} here). Each \
line reserves {slot} px — `size × line_height`, which is not a function of the font's ink \
(ADR-0007) — so every field is individually valid. Look at a frame: ADR-0087 leaves two \
repairs open and the document does not say which was meant.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    // ---- The `highlight` and transition document checks (#202). --------------------
    CheckSpec {
        // ADR-0051's containment check. Refuse, on `E-TRACK-OVERLAP`'s reasoning: the
        // document does not say whether the run's window or the element's own range is
        // the one that is wrong — a whole-document offset mixup could be either, and the
        // two repairs are not the same edit.
        code: "E-HIGHLIGHT-RANGE",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0051",
        template: "{element}: the highlight window on run \"{run}\" ({start}..{end}) falls \
outside the element's own range {element_start}..{element_end}.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0051's non-overlap check. Its own code rather than an instance of the one
        // above — a window sliced against the wrong sentence is legal containment and
        // still wrong — and refuse-class for the same reason `E-TRACK-OVERLAP` is: two
        // windows disagree about which word lights up at an instant, and nothing in the
        // document says which of the two is the copy-paste.
        code: "E-HIGHLIGHT-OVERLAP",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0051",
        template: "{element}: the highlight window on run \"{run}\" ({start}..{end}) \
overlaps run \"{other}\" ({other_start}..{other_end}) by {overlap} ms.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0134 (#542): a window that holds no painted frame — the document lights the
        // word and the video never shows it, ADR-0006's "rounds out of existence" one level
        // down from an element. `review`, as every declared-but-never-painted fact is: an
        // author can mean it. Its own code rather than a fourth `N-QUANTIZATION` condition,
        // because every condition there is about presence and a highlight changes paint.
        // Zero frames is the format's own arithmetic, so the threshold is internal; a
        // fewer-than-N floor would be borrowed, and #553 found no source to cite for one.
        code: "R-HIGHLIGHT-UNPAINTED",
        classes: &[Review],
        repair: None,
        threshold: Internal,
        adr: "ADR-0134",
        // One finding per document, `N-QUANTIZATION`'s shape for vanished elements: one bad
        // words file leaves many windows unpainted, and the list says where to look.
        template: "{count} highlight windows hold no painted frame at {fps} fps: {detail}.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0059: a transition's range is derived, redundant data that must track the
        // two elements it bridges. Advise, unlike its two siblings above — the ADR states
        // the check as "a closed-form function of the two referenced elements' own
        // ranges", so unlike an overlap or a highlight mixup, the correct value is fully
        // determined by the document and the repair is exactly that recomputed window.
        code: "E-TRANSITION-RANGE",
        classes: &[Error],
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0059",
        template: "{element}: the transition's range {start}..{end} does not match the \
intersection of `{from}` and `{to}`, which is {derived_start}..{derived_end}.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0059's closed-form function — `max(start)..min(end)` — has no legitimate
        // domain when `from` and `to` never coexist: the "intersection" collapses to an
        // empty or inverted range, and `E-TRANSITION-RANGE`'s own repair ("set `start` to
        // X and `end` to Y") would recommend that same inverted pair back, an advise-class
        // finding claiming a fix is "fully determined" when it is not — the document does
        // not say whether the transition should move, one of the bridged elements should
        // move, or the transition should not exist at all. Its own code and refuse-class,
        // on `E-TRACK-OVERLAP`'s reasoning, rather than a branch inside `E-TRANSITION-RANGE`
        // that would have to state one class for two differently-determined outcomes.
        // Surface the ADR series has not ratified, found reviewing #202's own merge.
        code: "E-TRANSITION-NO-OVERLAP",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0059",
        template: "{element}: the transition bridges `{from}` ({from_start}..{from_end}) and \
`{to}` ({to_start}..{to_end}), which never overlap — there is no window for a crossfade \
between them.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0150: `E-ANCHOR-MISSING`'s sibling, one reference over, and advise for its
        // reason — the next move follows from the condition rather than from knowing which
        // element was meant. `frame` keeps `E-NOT-PAINTED-UNRESOLVED-REF`, since it runs
        // without `validate`.
        code: "E-TRANSITION-REF-MISSING",
        classes: &[Error],
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0150",
        template: "{element}: the transition's `{side}` names `{target}`, which is not a \
visual element in this project.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0150: `E-ANCHOR-SELF`'s sibling — the element is there, so "no such element"
        // would be false, and a self-reference reads as a copy-paste rather than a typo.
        code: "E-TRANSITION-REF-SELF",
        classes: &[Error],
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0150",
        template: "{element}: the transition's `from` and `to` both name `{target}`; a \
transition hands over between two elements.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0150: in a slide only `to` moves, over a still `from`, and the transition never
        // changes a layer. Refuse, because more than one layer change fixes it — raise `to`,
        // lower `from`, or swap the tracks — and the document does not say which was meant.
        code: "E-TRANSITION-SLIDE-UNDER",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0150",
        template: "{element}: the slide's `to`, `{to}` (layer {to_layer}), paints beneath \
its `from`, `{from}` (layer {from_layer}). Only `to` moves in a slide, over a still `from`, \
so `to` must paint above `from`.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0154 §6. Refuse: a vertex is missing, and where it goes is the author's.
        code: "E-PATH-TOO-FEW-POINTS",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0154",
        template: "`{element}`.{field}{?record} in keyframe record {record} (`t` {t}){/record}: \
on a `{kind}`, an {shape} path needs at least {least} vertices, and this value has {count}.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0154 §6. Advise: the handle shapes no segment, so dropping it changes nothing
        // drawn — the fix is fully determined.
        code: "E-PATH-DANGLING-HANDLE",
        classes: &[Error],
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0154",
        template: "`{element}`.{field}{?record} in keyframe record {record} (`t` {t}){/record}: \
on a `{kind}`, vertex {vertex} carries an `{handle}` that shapes no segment of an open path. Drop it.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0154 §6. Refuse: whether the first value or this one is the intended shape is a
        // question about intent, and a morph between unlike paths is a later question.
        code: "E-PATH-KEYFRAME-SHAPE",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0154",
        template: "`{element}`.{field}: on a `{kind}`, keyframe record {record} (`t` {t}) differs from the \
first record at vertex {vertex}: {detail}. Every value of one keyframe list has the same \
vertex count, and each vertex the same handles.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0154 §4, with ADR-0158 §4's reach. Refuse: moving the point, growing the box,
        // thinning the stroke and lowering the reach each fix it, and which was meant is the
        // author's.
        code: "E-PATH-OUTSIDE-BOX",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0154",
        // ADR-0158 §7's own example names `k` and where it came from, and says the bound is
        // worst-case, so an agent does not hunt for an overlap that is not there.
        template: "`{element}`.{field}{?record} in keyframe record {record} (`t` {t}){/record}: \
vertex {vertex}'s `{handle}` sits at {position} in box pixels, outside the inset box \
[{inset}, {right}] × [{inset}, {bottom}]{?text} of this `text`: inset {derivation}. The inset \
keeps a plain glyph sitting on the curve inside the box; it is a frame convention, not a \
containment guarantee, and `measure` reports the bent line's ink.{/text}{!text} that keeps the \
stroke inside the declared box: inset {inset} = ceil({k} × {width} / 2), from {source}. This \
bound is worst-case, not a measured overlap.{/text}",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // prototype(#764), ADR-0161 §3. Refuse: whether to drop the break or split the
        // line into a second text element is the author's.
        code: "E-TEXT-PATH-BREAK",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0161",
        template: "`{element}`.runs[{run}]: its text holds a line break, and a text carrying \
`path` sets one line only. Remove the break, or put the second line in a second text element \
with its own `path`.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // prototype(#764), ADR-0161 §8. Advise: the field places nothing, so dropping it
        // changes nothing drawn.
        code: "E-TEXT-PATH-OFFSET-ORPHAN",
        classes: &[Error],
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0161",
        template: "`{element}`.path_offset: this text carries no `path`, so `path_offset` places \
nothing. Drop it, or give the text a `path`.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0158 §6. Advise: the field shapes nothing that draws, so dropping it changes
        // nothing drawn — the fix is fully determined.
        code: "E-STROKE-NO-STROKE",
        classes: &[Error],
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0158",
        template: "`{element}`.{field}: {?zero}its `stroke_width` is absent or 0 on every \
value, so the stroke never draws{/zero}{!zero}it has no `stroke`{/zero}, and `{field}` shapes \
nothing. Drop it, or give the element a `stroke` and a `stroke_width` above 0.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0158 §2. Refuse: whether the join or the limit is the intended one is the
        // author's, and the limit sets the box's inset.
        code: "E-STROKE-MITER-LIMIT",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0158",
        template: "`{element}`.{field}: {?missing}`\"miter\"` needs a `stroke_miter_limit`, an \
integer from 1 to 10, because the box's inset is computed from it{/missing}{!missing}a limit \
applies only to `\"miter\"`, and this path's `stroke_join` is `\"{join}\"`. Drop the limit, or \
set `\"miter\"`{/missing}.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0158 §3. Refuse: dropping the cap, opening the path and dashing it are all
        // fixes, and which was meant is the author's. It names `closed` and `stroke_dash`,
        // because an edit to either can make a cap draw or stop drawing.
        code: "E-STROKE-CAP-UNDRAWN",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0158",
        template: "`{element}`.stroke_cap: a `\"{cap}\"` cap draws nowhere, because the path is \
`closed` and has no `stroke_dash`, and a cap draws only at an open path's two ends and at each \
dash's ends. Drop `stroke_cap`, set `closed` to false, or add a `stroke_dash`.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0158 §5. Refuse: whether the doubled list or a shorter one was meant is the
        // author's.
        code: "E-DASH-SHAPE",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0158",
        template: "`{element}`.stroke_dash: {?odd}it has {count} entries, an odd number; the \
format does not repeat an odd list as SVG does, so write the doubled list out{/odd}{!odd}its \
entries add up to 0, so there is no pattern to repeat{/odd}. A pattern alternates dash, gap, \
dash, gap, starting with a dash.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0158 §5. Refuse: a round cap, a square cap and a longer dash each draw
        // something different, and which was meant is the author's.
        code: "E-DASH-ZERO-BUTT",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0158",
        template: "`{element}`.stroke_dash[{index}]: a zero-length dash draws nothing under a \
butt cap. {?shape}A `{kind}`'s dash ends are always butt; draw dots with a `path` with \
`\"stroke_cap\": \"round\"`{/shape}{!shape}Set `stroke_cap` to `\"round\"` for a dot, or \
`\"square\"` for a square{/shape}.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0158 §5, which makes it an error and names no code for it. Advise: the offset
        // moves nothing, so dropping it changes nothing drawn.
        code: "E-DASH-OFFSET-ALONE",
        classes: &[Error],
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0158",
        template: "`{element}`.stroke_dash_offset: there is no `stroke_dash` for it to move. \
Drop it, or add a `stroke_dash`.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        code: "R-CAPTION-PACE",
        classes: &[Review],
        repair: None,
        // The first member of ADR-0061's fenced exception, and until #199 its only one —
        // `R-CAPTION-MIN-DURATION` below is the second. ADR-0071 retires the count in
        // ADR-0061's prose and makes this table the live answer instead.
        threshold: External {
            source: "Netflix and BBC timed-text guidance",
            adr: "ADR-0034",
        },
        adr: "ADR-0034",
        // The three numbers ADR-0034 asks the finding to carry — "the computed cps, the
        // character count and the duration so a reader can judge the margin without
        // re-deriving it" — and they are also what ADR-0061's second condition requires:
        // the raw measured fact as the substance, never a pass/fail.
        template: "{element}: {characters} characters in {duration} ms is {measured_cps} \
characters per second, against a threshold of {threshold_cps}.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // Internal, and ADR-0061 says why in this check's own words: "`R-CAPTION-REPEAT-DURATION`'s
        // one-frame tolerance comes from the project's own declared `fps`, so the check
        // passes even though 'captions shouldn't visibly shrink on repeat' is itself a
        // human intuition." What is tested is the *deciding* number, not the subject matter.
        code: "R-CAPTION-REPEAT-DURATION",
        classes: &[Review],
        repair: None,
        threshold: Internal,
        adr: "ADR-0034",
        // Deliberately directionless. ADR-0034 rejects "shorter on repeat" as an
        // authorial-intent claim ADR-0006 forbids a finding from making, so the template
        // says the durations disagree, names both ends and the tolerance they cleared, and
        // leaves which one is right to the reader. `detail` is the ADR's own
        // "(id, start, duration)" listing, one entry per occurrence.
        template: "{count} elements carry the text \"{text}\", and their on-screen \
durations disagree: {shortest} ms to {longest} ms, a spread of {spread} ms against one \
frame at {fps} fps — {detail}.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // Internal, and the contrast with its two siblings is the point ADR-0061 makes:
        // "what matters is the *deciding* number". This check has none — it asks whether a
        // set is empty over an interval the document itself states — so nothing here is
        // borrowed, however much "a caption should have something spoken under it" sounds
        // like a judgment.
        code: "R-CAPTION-NO-AUDIO",
        classes: &[Review],
        repair: None,
        threshold: Internal,
        adr: "ADR-0054",
        // Says what it looked for and what it found, because the check's silence has a
        // shape a reader must be able to see: it is presence of *any* audio-bearing
        // element, never narration coverage (ADR-0054).
        template: "{element}: no `audio` or `video` element overlaps {start}..{end} ms \
({duration} ms), so nothing is declared to be heard under it.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // The second member of ADR-0061's fenced category, which ADR-0071 records as this
        // table's to answer rather than that ADR's prose. Not a contradiction of it but the rule it wrote: ADR-0054 puts this floor "in the same register as
        // `R-CAPTION-PACE`'s 20 cps — an externally documented constant about human reading
        // capacity, not a property of the render", and ADR-0061's citation requirement is
        // "binding policy for future checks of this shape, not best-effort". Strip 834 and
        // the check has no predicate left, and 834 is nowhere in any document.
        code: "R-CAPTION-MIN-DURATION",
        classes: &[Review],
        repair: None,
        threshold: External {
            source: "Netflix Timed Text Style Guide, General Requirements: a 5/6-second \
minimum caption duration",
            adr: "ADR-0054",
        },
        adr: "ADR-0054",
        // The measured duration first and the floor second, in the same shape as
        // `R-CAPTION-PACE`: a reader who disagrees with 834 still has the number. The range
        // is named because a duration with no instants is a number you cannot go and look at.
        template: "{element}: on screen for {duration} ms ({start}..{end} ms), below the \
{floor} ms floor.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0006 restated ADR-0005's instruction because the literal one is wrong:
        // "109 of its 120 time values — 47 of 48 distinct instants — are off the 40 ms grid
        // at the project's own 25 fps ... A check with a 98% hit rate on a correct,
        // published project is not a check; it is the alarm fatigue this ADR already
        // identified as a safety problem." So this reports what quantization *changes*,
        // and on a project where it changes nothing it says nothing at all.
        code: "N-QUANTIZATION",
        // `review`, and the code's `N-` prefix is not the objection — "the prefix is a
        // convention and not a rule" (`CONTEXT.md`). ADR-0006 escalates exactly the two
        // conditions this check fires on: "Escalate to `review` only for the cases in (1)
        // and (2)" — an element that rounds out of existence, and a rounding that
        // manufactures an overlap or a gap. Both mean the rendered frames do not show what
        // the document declares, which is what `review` is: legal, renders, and you must
        // look at a frame.
        //
        // `note` is declared beside it because ADR-0006's third derived fact — the twelve
        // instants where a picture cut and a speech boundary land up to 18 ms apart — is
        // one, and is the alignment detail the same ADR puts "behind `--verbose`". Nothing
        // emits it yet; ADR-0035 gave the grid arithmetic to `measure` (#205), and where
        // that fact belongs is #257.
        //
        // ADR-0105 (#412) adds a third `review` condition on the same reason: a *visual
        // state* the grid never paints — two boundaries on different elements landing on one
        // frame, so no element vanishes and no track gains a gap, yet the combination the
        // document declares is never on screen. `validate`'s quantization check raises it
        // (#437) through `checks::quantization::unpainted_states`, over the one selection of
        // visual states; `frame`'s range mode raises it for each `no-grid-frame` skipped run
        // through the same function (#488), so the state has one identity whichever verb saw
        // it. One finding
        // per state, carrying `from`, `to`, `present` and `boundaries` (each `at` with the
        // elements `entering` and `leaving`, by element and track) beside the template's
        // three fields; `changed` is always 2, the state's own boundaries. ADR-0118 ratifies
        // the fields, and that the state finding fires even where (1) or (2) already did.
        classes: &[Review, Note],
        repair: None,
        threshold: Internal,
        adr: "ADR-0006",
        // The template ADR-0006's own summary line implies — a count — plus the `fps` the
        // grid is derived from and the list of what actually changed. Without the list the
        // count is a number the reader cannot act on or check, which is the anti-vagueness
        // rule the same ADR states.
        template: "quantization at {fps} fps changes {changed} boundaries: {detail}.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    // ---- `fit`'s only consumer (#204). ---------------------------------------------
    CheckSpec {
        // ADR-0013 shipped this as a `note`; ADR-0015's court overturned that
        // unanimously, on the same behavioural evidence #44 was founded on — three of
        // six authoring tasks under the `note`'s preferred tolerance produced two
        // different, equally legal files for one input. Strict equality breaks 0 of 8
        // committed elements, the same price the loose rule paid.
        code: "E-FIT-DEVIATION",
        classes: &[Error],
        // Advise, on `E-SPEED-MISMATCH`'s reasoning: the document names a rule and a
        // source, and the rule's value is fully determined by the document plus the
        // media on disk — there is no second reading of what the author meant.
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0015",
        template: "{element}: `fit:\"{fit}\"` from `{source}` ({source_width}x\
{source_height}) derives {rule_width}x{rule_height}; the declared rect is \
{declared_width}x{declared_height}. Write {rule_width}x{rule_height}, or `fit:\"literal\"` \
if deliberate.",
        status: Live,
        census: None,
        sets: &[Disk],
    },
    // ---- Font vendoring: the gate in `fonts vendor`, and the attestation checks in
    // ---- `validate` (#207, ADR-0057). ------------------------------------------------
    //
    // ADR-0057 names the conditions and none of the codes; every spelling below is surface
    // the ADR series has not ratified, and the split into six is this table's own rule
    // (ADR-0043 fixes the repair class per code) applied to conditions that repair
    // differently.
    CheckSpec {
        // Bucket 1. Refuse, and the refusal is the whole point of the bucket: "an override
        // affordance is itself the thing that makes the project a knowing party to an
        // illegal copy". Fired by `fonts vendor` before any bytes are copied; `validate`
        // never re-adjudicates licence law (ADR-0057).
        code: "E-FONT-BLOCKLISTED",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0057",
        template: "`{font}` calls itself `{name}`, which matches `{matched}` on Montagent's \
blocklist of known non-redistributable fonts (Apple's system fonts are licensed for UI \
mockups on Apple's own platforms and may not be embedded in other software). Nothing was \
copied, and no flag will copy it. Known open substitutes, none of them metric-compatible: \
{substitutes}.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        // Bucket 3. Advise, and the tension is recorded rather than hidden: the fix is
        // *shaped* by the document — re-run with `--licence` — but its value is a fact a
        // human verifies outside it. Refuse-class was rejected because its printed
        // guarantee, "no flag can lift it", is exactly false here: ADR-0057 designed
        // `--licence` as the way through this bucket, so a finding that said no flag exists
        // would send an agent to a human with the wrong question. The repair value says
        // the human step in words instead.
        code: "E-FONT-LICENCE-UNKNOWN",
        classes: &[Error],
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0057",
        template: "`{font}` ({name}) carries no licence Montagent recognises — {detail} — so \
nothing was copied. Once a human has confirmed the file may be redistributed, re-run with \
`--licence <identifier>` to record that declaration; a false declaration is the declarer's \
liability, recorded in the file.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        // `validate`'s attestation check, missing half. Advise: the next move is the one
        // verb that writes attestations, and it follows from the condition itself.
        code: "E-FONT-UNATTESTED",
        classes: &[Error],
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0057",
        template: "`{file}` is in the `fonts` table and has no `fontVendor` entry: nothing \
records its licence or the bytes that were vendored. Run `montagent fonts vendor <path to \
the font> --as {file}` — `--as` must match `{file}` exactly, or the table still won't \
resolve to an attestation.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // `validate`'s attestation check, mismatched half. Refuse, on `E-SOURCE-OVERRUN`'s
        // reasoning: the document records one hash and the disk holds another, and which
        // of the two is the mistake — a font swapped in place, or an attestation edited by
        // hand — is not readable off either. ADR-0007 is why it is an `error` at all: a
        // font swapped in place "is a silent whole-project render change that no census
        // sees".
        code: "E-FONT-HASH-MISMATCH",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0057",
        template: "`{file}` hashes to {actual} on disk, and its `fontVendor` entry records \
{recorded}. The file was changed after it was vendored, or the entry was, and the document \
does not say which; every measured size and break in this project was taken in the font \
that was attested, not the one on disk.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // A `fonts`-table path with no file behind it. Not in ADR-0057's list, and needed
        // by its check: a hash cannot be compared against bytes that are not there, and
        // reporting that as a mismatch would be the false confidence ADR-0006 forbids.
        // ADR-0007 already names "font-file resolution" as a `validate` `error`; this is
        // the code it fires under. Advise, on `E-SOURCE-MISSING`'s reasoning.
        code: "E-FONT-MISSING",
        classes: &[Error],
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0007",
        template: "`{file}` is in the `fonts` table and is not there: {detail}. Looked for \
it at {resolved}.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0057: "a path referenced by no remaining chain ... is a `validate` warning,
        // never a silent drop — an orphaned attestation may still be wanted, and pruning
        // it is a decision an author makes, not one `fmt` makes for them." #207 resolves
        // the ADR's "warning" to `note`: nothing on any frame changes, and a fact you may
        // want and will not act on today is what a note is.
        code: "N-FONT-ATTESTATION-ORPHANED",
        classes: &[Note],
        repair: None,
        threshold: Internal,
        adr: "ADR-0057",
        template: "`fontVendor` attests `{file}`, which no `fonts` chain references. It is \
kept, never pruned: remove it yourself if the file is gone for good.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    // ---- ADR-0007's own five text checks, and the font-swap census (#206). -----------
    //
    // ADR-0007's consequence list names the conditions and none of the codes; every
    // spelling below is surface the ADR series has not ratified, on the same footing as
    // ADR-0057's six above.
    CheckSpec {
        // ADR-0007: "A character with no glyph in any chain entry renders `.notdef` and is
        // a `validate` **error** — under ADR-0006's definition it is *guaranteed wrong*,
        // and unlike a gap there is no intent it could express."
        code: "E-FONT-NO-GLYPH",
        classes: &[Error],
        // Refuse. Two documents repair this and the disagreement between them is exactly
        // what ADR-0043 fences off: the chain is short a file, or the text carries a
        // character it was never meant to. Nothing in the document says which, and a check
        // that advised one would be guessing at the author's meaning — the per-instance
        // triage that ADR forbids.
        //
        // **No census, and no list of the other chains that do map the characters.** That
        // list rode here once, shaped like a census, and it partitioned nothing (#427 ruling
        // 6). Worse, it bore on one branch of the fork only, which is a repair by another
        // name (ADR-0120). The census this code *could* carry is recorded there and not
        // built: the elements on this key, fully mapped against short.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0007",
        template: "{element}: nothing in the `{font}` chain has a glyph for {characters}, \
so it renders as .notdef. The chain is {chain}.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0007's font census — *"23 elements use `brand`; 1 uses `brand-old`"* — and
        // spec #168's story 102: "a fact I can see rather than a query I have to write".
        //
        // A `note`: nothing on any frame is wrong, and two declared fonts in one project
        // is ordinary (ADR-0007's own worked table declares `brand` and `brand+fa`). What
        // the census is for is the *outlier*, and ADR-0043 forbids saying which group that
        // is — so the finding states the distribution and stops.
        code: "N-FONT-CENSUS",
        classes: &[Note],
        repair: None,
        threshold: Internal,
        adr: "ADR-0007",
        template: "The project's text is set in more than one declared font: {distribution}.",
        status: Live,
        census: Some(Counted),
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0007: a "**font-swap** finding naming which measured layouts are now
        // unverified", and "font files join ADR-0006's `(path, size, mtime)` probe cache —
        // a font swapped in place is a silent whole-project render change that no census
        // sees". Spec #168 asks for both halves under one story (54) and one mechanism
        // answers both: a chain's identity is the ordered `(file, size, mtime)` of its
        // entries, so a table edit and a file rewritten in place are one comparison.
        //
        // `review`, which is ADR-0006's definition read literally: it is legal, it renders,
        // and the sizes and breaks ADR-0007 requires the author to have measured by hand
        // were measured in the other font — so you must look at a frame to know whether it
        // was meant.
        code: "R-FONT-SWAP",
        classes: &[Review],
        repair: None,
        threshold: Internal,
        adr: "ADR-0007",
        template: "The `{font}` chain is not the one this project was last validated \
against: {detail}. {elements} measured in the old chain are unverified — every size and \
every hand-placed break in them was taken against different metrics.",
        status: Live,
        census: Some(Counted),
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0153 §4: letter spacing adds nothing between two letters of one joining
        // script, so a tracked Arabic word stays as it was. `review`: the spacing still
        // spreads everything else on the line, and that is a legitimate thing to write.
        code: "R-SPACING-SUPPRESSED",
        classes: &[Review],
        repair: None,
        // Internal: which pairs are suppressed is a rule over Unicode properties of the
        // text, with no number borrowed from outside the format.
        threshold: Internal,
        adr: "ADR-0153",
        template: "{element}: its `letter_spacing` adds nothing between two letters of the {script} script, first in `{word}`. A joining script's letters take no letter spacing between them (ADR-0153), so the spacing spreads only its spaces, punctuation, digits and other scripts.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0151 §4: a run's `unit` replaces one unit's values, so the run must hold
        // exactly one unit. Refuse: which unit the author meant — or whether they meant to
        // split the run differently — is the one fact the document does not carry.
        code: "E-UNIT-RUN-NOT-ONE-UNIT",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0151",
        template: "{element}: run {run} (`{text}`) carries a `unit` override{?no_block}, but the element has no `units` block, so there is no unit for it to single out{/no_block}{!no_block}, but it holds {found} units' graphemes, not exactly one. A run singling out a unit holds all of that unit and nothing else: with `by: word` the word and its punctuation, with `by: line` the whole line{/no_block} (ADR-0151).",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0151 §4: an override replaces a list the block declares. Naming another would
        // add motion to one unit, which is not what the replace rule says it does.
        code: "E-UNIT-RUN-UNDECLARED",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0151",
        template: "{element}: run {run} (`{text}`) overrides `{property}`, which the `units` block does not declare. An override replaces a list for its unit and never adds one: declare `{property}` on the block (ADR-0151).",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0151 §3–§4, widened by ADR-0153 §2: a unit that moves with a neighbour, by a
        // shaping merge or a cursive join, is drawn on the first one's timing, so an
        // override on it could never be its own.
        code: "E-UNIT-RUN-MERGED",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0153",
        template: "{element}: run {run} (`{text}`) singles out unit {unit}, which is {how} with units {merged_with} and moves on the first one's timing, so its override can never be its own. Single out the whole word with `by: word`, or a letter that joins nothing (ADR-0151, ADR-0153).",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0151 §3, ADR-0153 §2: units that share a glyph or a joined piece move as one
        // body, and the next unit waits out every step they keep. `review`: legal, and the
        // agent should look at the cascade.
        code: "R-UNIT-MERGED",
        classes: &[Review],
        repair: None,
        threshold: Internal,
        adr: "ADR-0153",
        template: "{element}: under `by: {by}`, {detail} each move as one body ({how}), on their first unit's timing. Every unit keeps its index and its step, so the unit after a body waits out its steps. `\"by\": \"word\"` is one edit away if the waiting steps are unwanted (ADR-0151, ADR-0153).",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0151 §6: `R-CAPTION-PACE` is unchanged, and a staggered caption gets the
        // instant its last unit lands. `note`: readers follow letters as they arrive.
        code: "N-CAPTION-SETTLES",
        classes: &[Note],
        repair: None,
        threshold: Internal,
        adr: "ADR-0151",
        template: "{element}: its stagger lands at {settles} ms{?never}, at or after the element's end at {end} ms, so the caption never settles{/never}{!never}: its text is fully readable from then until {end} ms{/never}.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0007's grapheme-cluster check: "no run boundary splits a base from its
        // combining mark". A run boundary is a *style* boundary and each run shapes on its
        // own, so a mark that starts a run has no base to attach to and renders on a dotted
        // circle — guaranteed wrong, which is ADR-0006's `error`.
        code: "E-RUN-SPLIT-CLUSTER",
        classes: &[Error],
        // Refuse, on `E-FONT-NO-GLYPH`'s reasoning: the cluster belongs whole to one run
        // or the other, and which of the two styles the author meant it to wear is the one
        // fact the document does not carry — the boundary is there *because* the styles
        // differ.
        //
        // **No sibling census, and ADR-0043 says a refuse-class finding carries one.**
        // `E-ANCHOR-CHAIN`, `E-FONT-NO-GLYPH` and the two schema codes are already
        // refuse-class without one, so the practice is that the census attaches where a
        // sibling group exists. Here none does: the fault is one boundary inside one
        // element's own string, and there is no observable other elements could be grouped
        // by that would narrow it.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0007",
        template: "{element}: the boundary before run {run} falls inside {cluster} \
({codepoints}), so the mark shapes with no base. A run boundary is style only (ADR-0007) — \
move it off the cluster.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0007's invisible-character census, spec #168 story 104: "a character I cannot
        // see is one I am told about". Never an error — ZWJ is how an emoji sequence is
        // spelled and RLM is how ADR-0007's own bidi story is told without a `dir` override
        // — so the finding counts and locates them and judges none of them.
        code: "N-TEXT-INVISIBLE",
        classes: &[Note],
        repair: None,
        threshold: Internal,
        adr: "ADR-0007",
        // The verb agrees with the count, so it travels in the field rather than in the
        // template: "1 character … occupies", "3 characters … occupy".
        template: "{occurrences}, and no diff will show you where: {summary}.",
        status: Live,
        census: Some(Named),
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0007's mixed-normalization finding, spec #168 story 105: "two strings that
        // look identical and compare unequal are named". The harm is editing, not
        // rendering — canonical equivalence is a rendering-neutrality guarantee by design,
        // and ADR-0007 forbids any writer from normalising the difference away ("no tidying
        // pass, ever") — so it is a `note` and never a `LAYOUT` finding, whose prose would
        // send the reader to `fmt` for a change `fmt` must refuse to make.
        code: "N-TEXT-MIXED-NORMALIZATION",
        classes: &[Note],
        repair: None,
        threshold: Internal,
        adr: "ADR-0007",
        template: "{spellings} of {text} are the same string under Unicode and different \
bytes on disk, so an exact-string edit finds one of them and not the other.",
        status: Live,
        census: Some(Named),
        sets: &[Document],
    },
    // ---- The motion and geometry checks (#211). ------------------------------------
    CheckSpec {
        // ADR-0033's same-source cut, and ADR-0062's wrap — one code, because the
        // comparison at the wrap is byte-identical to the one at an interior cut and "a
        // second code would encode information the location already carries". `{seam}` is
        // that location: an interior instant, or the two boundary instants of the wrap.
        //
        // Keyed on same track + canonicalized source + adjacency and **never on `group`**:
        // ADR-0012's group-keyframe-time check answers a different question, and the
        // fixture proves it cannot stand in for this one — `photo-05-quiz` and
        // `photo-05-loop` share no `group` and carry the larger of the two pops, at 64016.
        code: "R-SOURCE-CUT-POP",
        classes: &[Review],
        repair: None,
        // The per-property tolerance table is float and sub-pixel noise headroom, not a
        // borrowed constant: every number in it is about this format's own resolved
        // arithmetic, and ADR-0033 is explicit that "none of these ever carries the
        // discrimination — every fixture pop clears its column by two or more orders of
        // magnitude". ADR-0061's fenced exception is for a threshold from outside the
        // document *and* the rendering semantics, which this is not.
        threshold: Internal,
        adr: "ADR-0033",
        // One finding per cut rather than per property, with `{detail}` carrying each
        // failing property's out-value, in-value and delta — ADR-0033's "per property that
        // fails its tolerance" list, in the shape `N-QUANTIZATION` already uses for a
        // finding whose substance is a list. Per-property findings would report one cut
        // three times, and `origin` — whose mismatch is its own trigger and has no delta —
        // has no honest row in a per-property field set.
        template: "`{element}` \u{2192} `{other}` {seam}, both `{source}`: {detail}.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0035: a keyframe's declared target value that no sampled frame inside the
        // element's own range ever produces — "the rendered picture does not match what
        // the file declares". Never off-grid `t` by itself, which is legal and which the
        // fixture does 109 times.
        code: "R-KEYFRAME-UNREACHED",
        classes: &[Review],
        repair: None,
        // The grid is `1000/fps`, evaluated in exact rational arithmetic — the format's own
        // sampling semantics and nothing borrowed.
        threshold: Internal,
        adr: "ADR-0035",
        template: "`{element}`.{property}: the keyframe at t={declared_t} declares \
{target}, and no frame sampled inside {start}..{end} ms reaches it \u{2014} the nearest is \
frame {frame} at {sampled_t} ms, where it resolves to {sampled}.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0044: a **standing** invariant, never conditioned on "a `frame` edit just
        // happened", and whole-range rather than per-instant — a slide-in that starts at
        // `x:-500` is ordinary animation vocabulary and fires nothing.
        code: "R-OFF-CANVAS",
        classes: &[Review],
        repair: None,
        threshold: Internal,
        adr: "ADR-0044",
        template: "`{element}`: its declared rect never meets the frame over {start}..{end} \
ms \u{2014} the union of its resolved rect across that range is {width}\u{d7}{height} px at \
({x}, {y}), and the frame is {frame_width}\u{d7}{frame_height}.{?widened} That rect is \
widened by the furthest its stagger's unit offsets reach, edge by the list that sets it: \
{widened}.{/widened}{?unchecked} Its units' {unchecked} were not checked (ADR-0151).{/unchecked}",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0146 §6: a run's paint is static and beats the element's, so a keyed
        // element-level text paint field that every run overrides reaches no glyph — and,
        // by ADR-0149 §5, neither does a gradient one.
        code: "R-TEXT-PAINT-OVERRIDDEN",
        classes: &[Review],
        // No repair: whether the keyframes or the run overrides are the mistake is a
        // question about intent, and a review finding declares no refuse class.
        repair: None,
        threshold: Internal,
        adr: "ADR-0146",
        template: "`{element}`.{property} is {written}, and every one of its {runs} runs \
states its own `{property}`, which beats the element's: the element's `{property}` is never \
drawn.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0149 §5: offsets never decrease in a literal stop list. The schema can bound
        // each offset but cannot relate two of them. Equal offsets are legal (a hard edge).
        code: "E-GRADIENT-STOP-ORDER",
        classes: &[Error],
        // Refuse: whether the offset or the order is wrong is the author's to say.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0149",
        template: "`{element}`.{property}, in {in}: stop {stop} is at offset {offset}, before \
stop {previous} at {previous_offset}. Offsets never decrease; equal offsets are legal and make a \
hard edge.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0149 §5: every `v` in one `stops` keyframe list has the same number of stops,
        // because stop i blends with stop i. The schema cannot relate two records.
        code: "E-GRADIENT-STOP-COUNT",
        classes: &[Error],
        // Refuse: whether a stop is missing or one too many is the author's to say.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0149",
        template: "`{element}`.{property}: keyframe {record} (t={t}) holds {count} stops, but \
the first holds {expected}. Every keyframe of one `stops` list holds the same number of stops, \
because stop i blends with stop i.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0157 §2: `source_start`, `source_end`, `speed` or `overrun` beside a
        // `source_time`, one finding per field. Advise: the curve is the only author of the
        // source, so the field says nothing the curve does not, and removing it changes no
        // frame — the fix is determined by the format's own semantics.
        code: "E-REMAP-FIELD",
        classes: &[Error],
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0157",
        template: "`{element}` carries `source_time`, which alone names the moment of its file \
on screen, and `{field}` beside it is one more place an edit could leave stale.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0157 §4: a remapped `video` whose `volume` is not the literal `0` — any other
        // number, any keyframe list, or none (the default is `1`). Advise: the format admits
        // one value here, so the fix is determined; the sound the author wanted belongs on a
        // separate `audio` element, which the repair says.
        code: "E-REMAP-AUDIBLE",
        classes: &[Error],
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0157",
        template: "`{element}` carries `source_time` and its `volume` is {volume}: a remapped \
video is silent, because a varying rate has no exact spelling in the mix.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0157 §5: a keyed `source_time` whose element holds a painted frame instant
        // before its first key or after its last, each end reported on its own. A literal
        // never fires it.
        code: "R-REMAP-HELD-END",
        classes: &[Review],
        repair: None,
        threshold: Internal,
        adr: "ADR-0157",
        template: "`{element}`'s `source_time` holds its value for {held_ms} ms {end} \
({from}..{to} ms), so the picture freezes there. A deliberate freeze is written as a flat \
pair of keys, which silences this.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0155 §5: a `motion_blur` that paints nothing a sharp element would not — no
        // value the element's keyframes resolve differs inside its range, a `units` stagger
        // counting as motion. A review, never a refusal; the repair is to remove the field.
        code: "R-MOTION-BLUR-STILL",
        classes: &[Review],
        repair: None,
        threshold: Internal,
        adr: "ADR-0155",
        template: "{element} carries `motion_blur`, and no value its keyframes resolve changes \
over {start}..{end} ms, so every frame paints it once, sharp. Remove the `motion_blur`.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0149 §5: a gradient whose resolved paint is a single colour, decided by the
        // resolving function — a forgotten second colour, or a radial that never grows.
        code: "R-GRADIENT-ONE-COLOUR",
        classes: &[Review],
        repair: None,
        threshold: Internal,
        adr: "ADR-0149",
        template: "`{element}`.{property} is a gradient that paints one colour, {colour}: \
{cause}.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0052, discharging ADR-0038's acknowledged cost: an `ease` required on a
        // record whose `v` does not change describes motion that does not happen. Literal
        // exact equality of author-written `v`, no tolerance — there is no resolution step
        // between the two records for a tolerance to absorb.
        code: "R-EASE-INERT",
        classes: &[Review],
        repair: None,
        threshold: Internal,
        adr: "ADR-0052",
        // One finding per run of consecutive holds, not one per pair: "three keyframe
        // records all sharing one `v` produce two inert-ease segments back to back, and
        // reporting them as two lines duplicates the same fact".
        template: "`{element}`.{property}: {records} consecutive keyframes hold v={value} \
from t={from} to t={to}; ease={ease} describes no motion.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0086's one instance of the recorded-intent pattern: a `t_from` whose rule
        // re-derives an instant that is not the `t` written beside it. The declaration is
        // renderer-ignored, so this finding is never about a frame — it is about a
        // relationship the document asserts and the document no longer satisfies.
        //
        // **`error`, which is the overturn ADR-0086's second jury round adopted**: a
        // violated declaration has already told you it was not meant, and `review` means
        // "you must look at a frame to know whether it was meant". No frame can say which of
        // two numbers is stale.
        code: "R-DERIVED-T",
        classes: &[Error],
        // Advise, and ADR-0086 decides it rather than this table: both v1 rules are
        // **directional** — the document names which value is the source — so the author
        // supplied the missing determinant and exactly one integer is legal. That is
        // `E-FIT-DEVIATION`'s logic transplanted off the raster axis. The ADR's symmetric
        // row, which would be refuse-class, has no v1 member to declare.
        repair: Some(Advise),
        // Every deciding number is the document's own: the element's `start`, the previous
        // record's `t`, and the `ms` the author wrote. There is no borrowed constant, and
        // no disk.
        threshold: Internal,
        adr: "ADR-0086",
        // Names the derivation as well as the number, because the number alone is a fact the
        // reader cannot check. Both repairs are stated: ADR-0086 makes the declaration
        // optional and its absence *no claim*, so dropping it is a real move and not a way
        // of silencing a checker.
        template: "`{element}`.{property}: the keyframe at t={declared_t} declares \
`t_from` `{rule}`, and {derivation}, which derives {derived}. Write {derived}, or drop the \
`t_from`.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0147. A blend mode reads what is under the element, so one with nothing under
        // it blends with the `background` alone: the light leak with no footage beneath.
        // A partial overhang is ordinary design and silent. Decided from bounding boxes at
        // the frames `render` paints; where only a rotated box keeps it quiet, the element
        // is named beneath NOT CHECKED rather than passed in silence.
        code: "R-BLEND-BACKGROUND-ONLY",
        classes: &[Review],
        repair: None,
        // Every deciding number is the document's own geometry.
        threshold: Internal,
        adr: "ADR-0147",
        template: "{element} blends in `{blend}` over {start}..{end} ms, and at {instant} ms \
no element lower in the stack meets its box, so it blends with the `background` alone. Put \
what it should blend with beneath it, or drop the `blend`.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0084. `circle` is the one mask shape whose meaning *discards* part of its
        // rect, so it is the one that gets a finding: `rect` is the rect and `ellipse`
        // fills it, both total, and an oval in a non-square rect surprises nobody.
        // `review` rather than `error` because the behaviour is determinate, ADR-0068
        // ratified it, and the committed fixture legitimately relies on it.
        code: "R-MASK-CIRCLE-NON-SQUARE",
        classes: &[Review],
        repair: None,
        // Every deciding number is the document's own: the rect's two sides, written or
        // inherited from the element. There is no borrowed constant here at all — the
        // predicate is `width != height`.
        threshold: Internal,
        adr: "ADR-0084",
        // Names both repairs, which is what makes the finding honest rather than noise:
        // they exist only because ADR-0084 admits explicit geometry. Under a
        // param-less-only vocabulary the single available move would have been "resize the
        // element" — moving the picture to satisfy a checker.
        template: "{element}: `effects[{index}]` is a `circle` mask on {rect_source}, \
{width}×{height}, so its diameter is {diameter} and {discarded} px of the long axis fall \
outside it. Use `ellipse` to fill the rect, or give the mask a square rect.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0152 §5. An inverted `rect` mask with no corner radius and no feather whose
        // rect contains the element's rect keeps no pixel on any frame, and the likely slip
        // is reading `invert` as "flip the mask" while the omitted rect is the whole
        // element. Decided from the file alone: a keyed rect field, keyed radius, keyed
        // feather or keyed element size is silent, so the finding never fires on a wipe.
        // A feather above `0` keeps a ramp at the edge (#698). `review` rather than `error`
        // because the construct is determinate and a static full erase is a legal halfway
        // state while a wipe is being written.
        code: "R-MASK-ERASES-ALL",
        classes: &[Review],
        repair: None,
        threshold: Internal,
        adr: "ADR-0152",
        template: "{element}: `effects[{index}]` is an inverted `rect` mask with no `radius` \
or `feather` whose rect contains the whole element, so it erases every pixel of {element} on every \
frame. Write the rect you meant, or drop `invert`.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    // ---- `chroma` (#342, ADR-0088). -------------------------------------------------
    //
    // Four findings, and **not one of them keys a frame**. ADR-0006 keeps `validate` to
    // "is this internally legal, and does it agree with the media on disk"; a check that
    // had to run the keyer to form an opinion would be the "does it say what you meant"
    // verb that ADR refuses to be. Three read the document alone and the fourth reads one
    // `ffprobe` field the pipeline already has.
    //
    // All four state facts and none proposes a repair (ADR-0043), which is also why none
    // is `error`-class: every condition below is a real if unusual technique, and the
    // author is the only one who knows which.
    CheckSpec {
        // A colour operation ahead of the key in the same ordered list. `effects` is
        // ordered and order is semantically real (ADR-0040), so the scalar changes the
        // pixels the key is measured against and the author's `color` no longer names what
        // is in the frame by the time the key reads it.
        code: "R-CHROMA-AFTER-COLOUR",
        classes: &[Review],
        repair: None,
        // The deciding fact is two positions in one list. There is no number here at all.
        threshold: Internal,
        adr: "ADR-0088",
        template: "{element}: `effects[{index}]` keys `{color}` out of pixels that \
`effects[{colour_index}]`, a `{colour_name}`, has already changed — `effects` is ordered, so the key is measured against the graded frame rather than the source's own colour.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // A key on pixels this format authored. The colour being keyed out is one the
        // document itself put there, so the author is asking for a shape they could have
        // declared.
        code: "R-CHROMA-ON-AUTHORED-ELEMENT",
        classes: &[Review],
        repair: None,
        threshold: Internal,
        adr: "ADR-0088",
        template: "{element}: `effects[{index}]` keys `{color}` out of a `{type}`, whose \
pixels this format authored — the colour being keyed is one the document itself states.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // A key on a source that already carries alpha. One `ffprobe` field, from the probe
        // `crate::checks::source` runs anyway.
        //
        // **The #339 relationship is coverage, not trust** (ADR-0088): `pix_fmt`-derived
        // detection yields false *negatives*, so this check is correct whenever it fires
        // and merely silent when it should have fired. It ships with that gap stated.
        code: "R-CHROMA-ON-ALPHA-SOURCE",
        classes: &[Review],
        repair: None,
        threshold: Internal,
        adr: "ADR-0088",
        template: "{element}: `effects[{index}]` keys `{color}` out of {source}, which the \
probe reports as already carrying an alpha channel.",
        status: Live,
        census: None,
        sets: &[Disk],
    },
    CheckSpec {
        // `tolerance: 0`, the identity value, which keys nothing. The shape ADR-0052
        // already made a finding for with inert ease: a declared effect that does nothing.
        code: "N-CHROMA-INERT",
        classes: &[Note],
        repair: None,
        threshold: Internal,
        adr: "ADR-0088",
        template: "{element}: `effects[{index}]` is a `chroma` with `tolerance: 0`, the \
identity value, so it keys nothing.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        // ADR-0156 §5: two `grain` members with the same `seed`, `size` and `mono`, visible
        // at the same instant, whose elements' starts fall on the same frame. Their draws
        // are the same on every frame, which shows as one locked texture. Decided from the
        // file without painting a frame. A review: copying a grain on purpose is legal.
        code: "R-GRAIN-SEED-SHARED",
        classes: &[Review],
        repair: None,
        // The deciding facts are the written seed, size and mono, the two `start`s and the
        // project's own frame grid.
        threshold: Internal,
        adr: "ADR-0156",
        template: "{element}: `effects[{index}]` and {other}'s `effects[{other_index}]` are \
`grain` with seed {seed}, size {size} and mono {mono}, and both elements start on frame \
{frame}, so they draw the same pattern on every frame they share. Change one `seed`.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        code: "U-SOURCE-UNPROBEABLE",
        classes: &[Unchecked],
        repair: None,
        threshold: Internal,
        adr: "ADR-0013",
        template: "{source} could not be probed, so the disk half of the question is unanswered: \
{detail}",
        // Live as of #190, which supplies the probe. ADR-0056 gave it its structured
        // reason: present when the network was what failed, absent when nothing was
        // attempted — the distinction that keeps "I could not look" from reading as
        // "it is not there".
        status: Live,
        census: None,
        sets: &[Disk],
    },
    // ---- ADR-0093: render's world-effects are findings (#386). ---------------------
    //
    // ADR-0093 ruling 1: no new severity tier. `CONTEXT.md` already defines `error` as
    // *"the render is refused or is guaranteed wrong"*, and a legal element the render
    // declined to mix or draw is the second clause with nothing left over. A sixth class
    // would encode only *where in the pipeline the fact surfaced*, and ADR-0006 fixes class
    // as computed from the consequence — pipeline position is not a consequence.
    //
    // ADR-0093 ruling 2 is why this is a block of codes rather than one `E-NOT-MIXED` whose
    // class is computed per instance. ADR-0043 decides repair form **once, when the check is
    // written**, and it holds for every instance the check matches. One code cannot be a
    // `note` for *"its source carries no audio stream"* and refuse-class for *"the engine
    // established nothing about its source"*. So: **one code per reason.**
    //
    // The reason set is closed here, where the prose it replaces was unassertable and
    // untestable. Two things bound it, and both are worth stating because they are what
    // keeps the set this small:
    //
    // - **`render` reaches none of these arms until `document.strict()` has succeeded.**
    //   Every type-level reason is therefore unreachable: `source_start` is a required
    //   `i64`, `Speed`'s `Deserialize` refuses `<= 0`, `Volume`'s refuses negatives, and
    //   `AudioOverrun` has no `hold` variant at all. Reaching one means the check engine and
    //   the renderer disagree about one document, which is `E-INTERNAL` (ADR-0073) and not a
    //   finding about the project. ADR-0093 ruling 2 guessed this for `overrun: "hold"`; the
    //   type system confirms it, and extends it to every sibling arm.
    // - **A reason's open-ended sub-prose is a `{detail}` field, not its own code.** An
    //   `ffmpeg` message and an `io::Error` are not a closed set and never will be. What is
    //   closed is the *condition* — "this source did not decode" — and that is the code.
    CheckSpec {
        code: "E-NOT-MIXED-REMOTE",
        classes: &[Error],
        // Refuse. The document names a URL, `render` mixes local sources, and the fix is a
        // local copy at a path only the author knows. There is no value to state: ADR-0043's
        // advise arm needs the fix *fully determined by the document*, and the one thing
        // missing here is precisely not in it.
        //
        // Refuse is not *"nothing you can do"*, and the refuse boilerplate's *"Do not guess
        // one"* reads that way on its own — so the template names the remedy and says
        // which part of it is the author's to choose (ADR-0131).
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0131",
        template: "`{element}` is not in the mix: `{source}` is remote, and `render` mixes \
local sources only. The remedy is a local copy: fetch the file and point `source` at it by \
path — which path is yours to choose, and is why no repair is stated.",
        status: Live,
        census: None,
        // ADR-0131: `validate` states it from the disk half, through the same
        // `media::established::local` `render` asks, so a URL is never a clean pass to one
        // verb and a refusal from the other.
        sets: &[Disk],
    },
    CheckSpec {
        code: "E-NOT-MIXED-UNREADABLE",
        classes: &[Error],
        // Refuse, for the same reason and not for a filesystem one: whether the author meant
        // to fix the permissions, move the file or name a different one is not readable off
        // the document. `E-SOURCE-MISSING` advises because a *confirmed absence* has exactly
        // one move; a file that is there and will not open does not.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0093",
        template: "`{element}` is not in the mix: {resolved} could not be opened — {detail}.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        code: "E-NOT-MIXED-UNESTABLISHED",
        classes: &[Error],
        // Refuse. This is the MONTAGENT-1 finding itself, and the reason it is an `error`
        // rather than a line of prose: the audible element was dropped from the mix, the
        // deliverable was a silent video, and the report said `0 errors`. ADR-0092 makes the
        // observed identity the only admissible answer to *"is this the same file?"*, so a
        // source the engine matched nothing for cannot be mixed — and ADR-0093 ruling 3 makes
        // `validate` say `UNCHECKED` about the same file in the same session, so the two verbs
        // can no longer disagree about it.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0093",
        template: "`{element}` is not in the mix: the check engine established nothing it can \
match to {resolved}, so `render` has no facts to mix it from — the `UNCHECKED` findings above \
say why.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        code: "N-NO-AUDIO-STREAM",
        classes: &[Note],
        repair: None,
        threshold: Internal,
        adr: "ADR-0093",
        // ADR-0093 ruling 2 reclassifies this one on the way through, and it is the reason
        // the ruling insists on a code per reason: a video with no audio track is an
        // ordinary video, not a defect. It is still not in the mix, and a reader who cannot
        // tell *"silent by nature"* from *"dropped"* chases the wrong thing — so the fact is
        // reported, at the class that says *"you will not act on this today"*.
        template: "`{element}` is not in the mix: {resolved} carries no audio stream.",
        status: Live,
        census: None,
        sets: &[],
    },
    // ---- #391 / ADR-0104: what is already at the output path. --------------------
    //
    // Two codes and not one, because ADR-0043 fixes repair form per code and these two
    // conditions do not share one. A *foreign* stamp is a fact: another project wrote that
    // file, Montagent observed it, and which of the two projects is meant to own the path is
    // not in either document — refuse. An *absent* stamp is an absence of evidence: the file
    // may be an irreplaceable cut or may be last week's scratch, and the tool cannot tell.
    // One code cannot be refuse-class for the first and `review` for the second.
    CheckSpec {
        code: "E-OUTPUT-FOREIGN",
        // ADR-0093: a world-effect of the promotion, so `error` — the deliverable is not
        // wrong, the *world* is about to be, and `error` covers "the render is refused".
        // Withholding here is not the absence of a promotion but the whole point of one.
        classes: &[Error],
        // Refuse. The repair is not derivable: whether the author meant to change this
        // project's `output` or to move the other project's deliverable out of the way is a
        // question about intent, and the document holds neither answer.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0104",
        template: "{output} was written by a different project ({project}), and rendering \
this one would destroy it. Give this project its own `output`, or move that file.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        code: "R-OUTPUT-UNATTESTED",
        // `review` **usually**, which is the one place this ticket's jury split. An unstamped
        // file is *true-but-uncertain*, and `error` is defined as "refused or **guaranteed**
        // wrong" — a guarantee the tool cannot make about a file it has no evidence about.
        // Classing it `error` outright would refuse the first render of every project that
        // existed before this check, charging the migration cost to the wrong party.
        //
        // `error` when `--no-clobber` was given, which is ADR-0006's per-instance class rule
        // doing exactly its job: the *condition* is identical and the *consequence* is not,
        // because the caller has said that no evidence is reason enough. It stays one code
        // rather than borrowing `E-OUTPUT-FOREIGN`, which would name a foreign project for a
        // file that has none — the report would be wrong about what it observed.
        classes: &[Review, Error],
        // ADR-0043: declared once for the code, and read only on the `error` instances. The
        // repair is not derivable for the same reason `E-OUTPUT-FOREIGN`'s is not.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0104",
        // Worded to be true at both classes: it states the observation and the two moves,
        // and does not promise which way the run went — the finding's own class says that.
        template: "{output} already exists and carries no Montagent attestation, so this \
project did not produce it. Render elsewhere, or move it aside; without `--no-clobber` it \
is replaced.",
        status: Live,
        census: None,
        sets: &[],
    },
    // ---- `verify` (ADR-0117) ---------------------------------------------------------------
    //
    // The identity gate first. `Foreign` has no code of its own here: it is the same fact
    // `E-OUTPUT-FOREIGN` states, with the same repair form, and ADR-0107's rule is one fact,
    // one code.
    CheckSpec {
        code: "E-VERIFY-NO-OUTPUT",
        classes: &[Error],
        // Advise, on `E-FONT-UNATTESTED`'s reasoning: the next move is the one verb that
        // writes the deliverable, and it follows from the condition itself.
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0117",
        template: "Nothing is at {output}, the project's declared `output`: there is no \
deliverable to verify. Render it first.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        code: "E-VERIFY-STALE",
        classes: &[Error],
        // Advise. `verify`'s question takes the document as it stands to be the one the
        // deliverable must render — that is what "stale" means — so the repair is fully
        // determined: render it again. Which edit made it stale does not change the move.
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0117",
        template: "{output} is this project's render, of a different document: the project \
file, or a source or font it names, has changed since it was rendered. Nothing about the file \
was measured, because every mismatch would descend from that one change. Render it again.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        code: "R-VERIFY-UNATTESTED",
        // `review`: #391's court's argument. `error` means *guaranteed* wrong, and no
        // evidence is not evidence. The measurements below still run at their own classes —
        // a 720p file where the document says 1080p is wrong as a deliverable whoever made it.
        classes: &[Review],
        repair: None,
        threshold: Internal,
        adr: "ADR-0117",
        template: "{output} carries no Montagent attestation, so it is not known to be this \
project's render. It was measured anyway; any mismatch below may mean it was never rendered \
from this document at all.",
        status: Live,
        census: None,
        sets: &[Deliverable],
    },
    CheckSpec {
        code: "R-VERIFY-STALENESS-UNKNOWN",
        // A separate code from `R-VERIFY-UNATTESTED` and not a second wording of it: that one
        // is *not known to be Montagent's*, this one is *Montagent's, staleness unknown*, and
        // an agent acts differently on each. A fingerprint that could not be read lands here
        // too, with its own `reason`: ADR-0069's rule is that such a failure is silence, never
        // a mismatch.
        classes: &[Review],
        repair: None,
        threshold: Internal,
        adr: "ADR-0117",
        template: "{output} is this project's render, and whether it is stale is unknown: \
{reason}. It was measured anyway; any mismatch below may be staleness rather than a defect.",
        status: Live,
        census: None,
        sets: &[Deliverable],
    },
    // The measurements. Once the gate has passed, a mismatch is attributable to the engine or
    // the encoder; without a stamp it may be anybody's. Either way the document holds no
    // answer to which, so each is refuse-class with no repair value.
    CheckSpec {
        code: "E-VERIFY-FRAME-SIZE",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0117",
        template: "{output} is {measured}, and this project renders {expected} (its declared \
`frame`, padded to even).",
        status: Live,
        census: None,
        sets: &[Deliverable],
    },
    CheckSpec {
        code: "E-VERIFY-FRAME-TIMING",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0117",
        template: "{output}'s decoded frames are not evenly spaced at {fps} fps: {irregular} \
intervals differ from {expected_ms} ms, the first before frame {frame} ({interval_ms} ms).",
        status: Live,
        census: None,
        sets: &[Deliverable],
    },
    CheckSpec {
        // One finding carrying both measurements: a missing frame shortens the stream by one
        // frame, and two findings for one cause is the cascade #384 collapsed.
        code: "E-VERIFY-EXTENT",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0117",
        template: "{output} holds {measured_frames} frames over {measured_ms} ms of video \
stream, and this project renders {expected_frames} frames over {expected_ms} ms.",
        status: Live,
        census: None,
        sets: &[Deliverable],
    },
    CheckSpec {
        code: "E-VERIFY-NO-AUDIO",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0117",
        template: "{output} has no audio stream, and something should be heard from {from} ms \
to {to} ms: an `audio` or `video` element with an audio stream and a `volume` above 0.",
        status: Live,
        census: None,
        sets: &[Deliverable],
    },
    CheckSpec {
        // `note`: a stream nobody should hear is harmless as a deliverable. It happens by
        // design where every element is at `volume: 0` — `render` still mixes them.
        code: "N-VERIFY-UNEXPECTED-AUDIO",
        classes: &[Note],
        repair: None,
        threshold: Internal,
        adr: "ADR-0117",
        template: "{output} has an audio stream, and nothing in this project should be heard.",
        status: Live,
        census: None,
        sets: &[Deliverable],
    },
    CheckSpec {
        code: "E-VERIFY-AUDIO-EXTENT",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0117",
        template: "{output}'s audio stream lasts {audio_ms} ms and its video stream {video_ms} \
ms: more apart than one video frame plus one audio frame ({slack_ms} ms).",
        status: Live,
        census: None,
        sets: &[Deliverable],
    },
    CheckSpec {
        // `review` only, because both numbers that decide it are borrowed (ADR-0061): the
        // gate and the block. The census is the point: *silent in its own source* is the
        // author's material, *audible in its own source* is the engine.
        code: "R-VERIFY-SILENT-SPAN",
        classes: &[Review],
        repair: None,
        threshold: External {
            source: "EBU R 128 absolute gate (−70 LUFS) over ITU-R BS.1770-4 400 ms momentary blocks",
            adr: "ADR-0117",
        },
        adr: "ADR-0117",
        template: "{output}'s mix is below the {gate_lufs} LUFS gate from {from} ms to {to} ms \
(quietest block {quietest_lufs} LUFS), where something should be heard. Attribution: \
{attribution}.",
        status: Live,
        // Named: "silent in its own source" is a measurement, not a value the document spells.
        census: Some(Named),
        sets: &[Deliverable],
    },
    CheckSpec {
        code: "E-EMPTY-RANGE",
        classes: &[Error],
        // Refuse: `start`, `end` and the source span are all authoritative under ADR-0020,
        // and which of them the author meant to move is not in the document.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0107",
        // One condition: an element whose `end` does not exceed its `start` — or whose
        // source range does not — occupies no instant, so it is neither mixed nor painted.
        //
        // ADR-0093 registered it as a `render` finding, because no `validate` check stated it
        // and a project with `"start": 0, "end": 0` validated at zero errors while `render`
        // refused it. ADR-0107 (#410) gives it to `crate::checks::range` under the **same**
        // code — one fact, one repair form (ADR-0043) — so `render` refuses on the check
        // engine's report and its own two arms are `E-INTERNAL`, as ADR-0093 makes every
        // arm the check engine closes. `error` and never `review`: there is no frame at
        // which to look at an element that holds no instant.
        template: "`{element}` occupies no instant: {field} {from}..{to} is empty, so there \
is nothing to render for it.",
        status: Live,
        census: None,
        sets: &[Document],
    },
    CheckSpec {
        code: "E-NOT-PAINTED-REMOTE",
        // ADR-0093: `error` from `render`, where the deliverable is guaranteed wrong, and
        // `review` from `frame`, where the consequence is *"look at this frame — the element
        // you asked about is not in it."* One code, two consequences, which is exactly what
        // ADR-0006 means by computing class from the consequence at an instant rather than
        // from the check. Repair form stays fixed per code (ADR-0043), because that is the
        // axis that may not vary.
        classes: &[Error, Review],
        // Refuse, exactly as its audio sibling: the missing fact is a local path.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0131",
        template: "`{element}` is not painted: `{source}` is remote, and `render` and `frame` \
draw local sources only. The remedy is a local copy: fetch the file and point `source` at it \
by path — which path is yours to choose, and is why no repair is stated.",
        status: Live,
        census: None,
        // `error` from `validate` as from `render` (ADR-0131): the deliverable is guaranteed
        // wrong, whichever verb says so first.
        sets: &[Disk],
    },
    CheckSpec {
        code: "E-NOT-PAINTED-UNREADABLE",
        // ADR-0093: `error` from `render`, where the deliverable is guaranteed wrong, and
        // `review` from `frame`, where the consequence is *"look at this frame — the element
        // you asked about is not in it."* One code, two consequences, which is exactly what
        // ADR-0006 means by computing class from the consequence at an instant rather than
        // from the check. Repair form stays fixed per code (ADR-0043), because that is the
        // axis that may not vary.
        classes: &[Error, Review],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0093",
        template: "`{element}` was not painted: {resolved} could not be read — {detail}.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        code: "E-NOT-PAINTED-UNDECODABLE",
        // ADR-0093: `error` from `render`, where the deliverable is guaranteed wrong, and
        // `review` from `frame`, where the consequence is *"look at this frame — the element
        // you asked about is not in it."* One code, two consequences, which is exactly what
        // ADR-0006 means by computing class from the consequence at an instant rather than
        // from the check. Repair form stays fixed per code (ADR-0043), because that is the
        // axis that may not vary.
        classes: &[Error, Review],
        // Refuse. A source that will not decode might want a different file, a different
        // codec or a different range, and the document does not say which.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0093",
        // MONTAGENT-2's failed seek no longer surfaces here: ADR-0096 made `frame_at`
        // answer with the frame the source is showing at the instant, so an off-grid
        // instant inside the last frame paints it rather than decoding nothing. ADR-0093
        // ruling 6 condition 1 held that the seek predicate was computable before the frame
        // loop; it is not, because the frame grid is a property of the source's timestamps
        // and not of either frame rate `probe` reports (ADR-0096 §2). What remains
        // computable — and so still pre-flightable — is the coarser question the clamp
        // leaves: whether the source ends more than a window before the declared range.
        template: "`{element}` was not painted: {resolved} did not decode — {detail}.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        code: "E-NOT-PAINTED-NO-EXTENT",
        // ADR-0093: `error` from `render`, where the deliverable is guaranteed wrong, and
        // `review` from `frame`, where the consequence is *"look at this frame — the element
        // you asked about is not in it."* One code, two consequences, which is exactly what
        // ADR-0006 means by computing class from the consequence at an instant rather than
        // from the check. Repair form stays fixed per code (ADR-0043), because that is the
        // axis that may not vary.
        classes: &[Error, Review],
        // Refuse: an element with no resolvable box could be missing a `width`, a `height`,
        // a `fit` or a probe, and ADR-0013's fitted extents make which one a question about
        // intent.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0093",
        template: "`{element}` was not painted: {detail}",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        code: "E-NOT-PAINTED-NO-PAINT",
        // ADR-0093: `error` from `render`, where the deliverable is guaranteed wrong, and
        // `review` from `frame`, where the consequence is *"look at this frame — the element
        // you asked about is not in it."* One code, two consequences, which is exactly what
        // ADR-0006 means by computing class from the consequence at an instant rather than
        // from the check. Repair form stays fixed per code (ADR-0043), because that is the
        // axis that may not vary.
        classes: &[Error, Review],
        // Refuse. ADR-0007's *"a field that is honoured sometimes is worse than no field"*
        // reasoning applies to the absence too: which of `fill` and `stroke` the author
        // meant, and in what colour, is not in the document.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0093",
        template: "`{element}` was not painted: it carries neither a `fill` nor a `stroke` \
to paint with.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        code: "E-NOT-PAINTED-FONT-CHAIN",
        // ADR-0093: `error` from `render`, where the deliverable is guaranteed wrong, and
        // `review` from `frame`, where the consequence is *"look at this frame — the element
        // you asked about is not in it."* One code, two consequences, which is exactly what
        // ADR-0006 means by computing class from the consequence at an instant rather than
        // from the check. Repair form stays fixed per code (ADR-0043), because that is the
        // axis that may not vary.
        classes: &[Error, Review],
        // Refuse. ADR-0007 puts the declared chain in charge of what the renderer may open,
        // so a chain that will not resolve is not something the picture may substitute its
        // way out of — which is the whole point of `R-FONT-SWAP` being a separate,
        // disclosed event rather than a silent fallback.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0093",
        template: "`{element}` was not painted: its font chain did not resolve — {detail}.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        code: "E-NOT-PAINTED-TEXT-LAYOUT",
        // ADR-0093: `error` from `render`, where the deliverable is guaranteed wrong, and
        // `review` from `frame`, where the consequence is *"look at this frame — the element
        // you asked about is not in it."* One code, two consequences, which is exactly what
        // ADR-0006 means by computing class from the consequence at an instant rather than
        // from the check. Repair form stays fixed per code (ADR-0043), because that is the
        // axis that may not vary.
        classes: &[Error, Review],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0093",
        template: "`{element}` was not painted: its text could not be laid out — {detail}.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        code: "E-NOT-PAINTED-UNDRAWABLE",
        // ADR-0093: `error` from `render`, where the deliverable is guaranteed wrong, and
        // `review` from `frame`, where the consequence is *"look at this frame — the element
        // you asked about is not in it."* One code, two consequences, which is exactly what
        // ADR-0006 means by computing class from the consequence at an instant rather than
        // from the check. Repair form stays fixed per code (ADR-0043), because that is the
        // axis that may not vary.
        classes: &[Error, Review],
        // Refuse: what the author meant by a member the format does not have is exactly the
        // thing not in the document.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0093",
        // **The painter's arms are reachable where the mix's are not, and the asymmetry is
        // the whole reason this code exists.** `render` and `preview` refuse before painting
        // and then call `document.strict()`, so no document that reaches `Mix::of` can carry
        // a closed-vocabulary member the model lacks. `frame` does neither: it reads the
        // document *permissively* and draws it, which is its job — an agent asks `frame` what
        // a picture looks like precisely when the document is not yet right.
        //
        // So a `type` the format does not have, a `transition` with no `kind`, and a source
        // offset that would not resolve all genuinely arrive here, and in `frame` they are
        // facts about the project rather than Montagent contradicting itself. One code:
        // the *condition* is one condition — the element cannot be drawn as declared — and
        // which key carries the undrawable value is a field, on `E-FIELD-UNHONOURED`'s
        // reasoning.
        template: "`{element}` was not painted: {detail}",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        code: "E-EFFECT-UNKNOWN",
        // ADR-0093: `error` from `render`, where the deliverable is guaranteed wrong, and
        // `review` from `frame`, where the consequence is *"look at this frame — the element
        // you asked about is not in it."* One code, two consequences, which is exactly what
        // ADR-0006 means by computing class from the consequence at an instant rather than
        // from the check. Repair form stays fixed per code (ADR-0043), because that is the
        // axis that may not vary.
        classes: &[Error, Review],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0093",
        // Its own code and not `E-NOT-PAINTED-UNDRAWABLE`, because the consequence is
        // different in the way the report is organised around: the element *was* drawn, and
        // one thing it asked for was not done. That is `painted_partially`, the list
        // `Picture` keeps apart from `not_painted` — and ADR-0043 would not let one code
        // cover both if their repair forms ever diverged.
        template: "`{element}` was painted without `effects[{index}]`: `{effect}` is not a \
member of the effect vocabulary, and it was drawn as though it were not there.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        code: "E-NOT-PAINTED-UNRESOLVED-REF",
        // ADR-0093: `error` from `render`, where the deliverable is guaranteed wrong, and
        // `review` from `frame`, where the consequence is *"look at this frame — the element
        // you asked about is not in it."* One code, two consequences, which is exactly what
        // ADR-0006 means by computing class from the consequence at an instant rather than
        // from the check. Repair form stays fixed per code (ADR-0043), because that is the
        // axis that may not vary.
        classes: &[Error, Review],
        // Refuse: whether the author meant to rename the reference, restore the element or
        // delete the transition is not in the document — `E-TRANSITION-NO-OVERLAP`'s own
        // reasoning, which this is the dangling-reference sibling of.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0093",
        // Until ADR-0150 no check claimed a dangling `from`/`to`, so the render was where it
        // surfaced, and until ADR-0093 as a line of prose. `validate` now says it as
        // `E-TRANSITION-REF-MISSING`; this code stays for `frame`, which runs without
        // `validate`.
        template: "`{element}` bridges `{from}` and `{to}`, and they do not both resolve to an element with a range — so the transition was not applied.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        code: "E-FIELD-UNHONOURED",
        // ADR-0093: `error` from `render`, where the deliverable is guaranteed wrong, and
        // `review` from `frame`, where the consequence is *"look at this frame — the element
        // you asked about is not in it."* One code, two consequences, which is exactly what
        // ADR-0006 means by computing class from the consequence at an instant rather than
        // from the check. Repair form stays fixed per code (ADR-0043), because that is the
        // axis that may not vary.
        classes: &[Error, Review],
        // Refuse. ADR-0093 ruling 1 names this case directly — *"an `index` parsed and
        // discarded"* is `error` — and the fix is not in the document: an author who wrote
        // the field meant something by it, and deleting it and waiting for the build to
        // honour it are different videos.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0093",
        // **One code, and the field name is a field.** The *reason* is one reason — "this
        // build parsed a declared field, validated it, and drew without it" — and ADR-0093
        // ruling 2's per-reason rule is about the condition, not about how many document
        // keys can meet it.
        //
        // **No field meets it today.** Its first instance, `runs[].dir`, is drawn as
        // ADR-0007's isolate since ADR-0133 (#457), and the font chain's `index` MONTAGENT-6
        // (#390) was to reuse it for was honoured instead (ADR-0102). Kept registered
        // because ADR-0093 makes it *the* code for the next field a build parses and does not
        // draw: a later one is raised here, not under a new code.
        template: "`{element}`: `{field}` was parsed and validated, and this build drew \
without it — the picture is not what the document declares.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        code: "L-KEY-ORDER",
        classes: &[Layout],
        repair: None,
        threshold: Internal,
        adr: "ADR-0041",
        template: "{element} (line {line}): key order does not match the schema for `{type}`; expected {expected}. Run `montagent fmt`.",
        // Live as of #193, which supplies the key-order predicate and the `fmt --check`
        // that reads it. `validate`'s `LAYOUT` check calls the same predicate and fires
        // the same code; it is a later ticket, and that is a second call site rather than
        // a second check.
        status: Live,
        census: None,
        sets: &[Document, CheckSet::Layout],
    },
    CheckSpec {
        // The rest of the convention: one element per line, the element sort ADR-0005
        // states, two-space indent, the header's and each track's own key order, one
        // trailing newline. Its own code rather than a flag on `L-KEY-ORDER`, because the
        // two are independently reachable — the incident agent pretty-printed the fixture
        // from 154 lines to 1595 **without disturbing a single key's position**, so a
        // `--check` carrying only `L-KEY-ORDER` would have reported nothing on the very
        // file ADR-0041 was written about.
        //
        // Ratified by ADR-0079 as a second, whole-file `LAYOUT` finding shape alongside
        // the element-scoped `L-KEY-ORDER` ADR-0041 specifies.
        code: "L-LAYOUT",
        classes: &[Layout],
        repair: None,
        threshold: Internal,
        adr: "ADR-0041",
        template: "{file} is not written in the canonical convention: {written_lines} lines as \
written, {canonical_lines} in canonical form, first difference at line {line}. Run `montagent \
fmt`.",
        status: Live,
        census: None,
        sets: &[Document, CheckSet::Layout],
    },
    // ---- `shift` (#220). ------------------------------------------------------------
    CheckSpec {
        // ADR-0151 §5: a cut inside a stagger window would split the unit lists right for
        // one unit only, and a split for every unit would turn the block into one run per
        // unit. `shift` refuses what it cannot write as legal literals (ADR-0146).
        code: "E-SHIFT-UNITS-WINDOW",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0151",
        template: "`{element}` in track `{track}` staggers its units from {from} to {to} ms, \
and the shift point {at} falls inside that window: a split of the unit lists at {at} would be \
right for one unit only. Shift at or before {from}, or at or after {to}.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        // ADR-0005's straddler refusal: a time-based element (audio, video) whose source
        // range cannot be stretched or relocated without either violating the timeline/
        // source invariant or moving speech that has already begun.
        code: "E-SHIFT-STRADDLE",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0005",
        template: "`{element}` in track `{track}` is time-based and straddles the shift \
point {at}: it runs {start}..{end} ms, so stretching or moving it whole would misalign its \
source. The nearest legal boundaries are {start} and {end}.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        // ADR-0146 §7: a split writes a new keyframe, so its value must be a legal literal
        // for the field. An overshoot can carry a size below zero, or a colour out of
        // range, at the split instant; clamping the written value would bend both halves of
        // the curve without saying so, so the edit is refused instead.
        code: "E-SHIFT-SPLIT-UNWRITABLE",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0146",
        template: "`{element}`.{property} cannot be split at {at} ms: the curve passes \
through {value} there, which is not a legal `{property}`. Shift at another instant, or change \
the keyframes so the curve stays in range at {at}.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        // ADR-0032/ADR-0047: every slack is invariant by default. `shift` refuses an edit
        // that would change one's size unless the caller names it, in full, in `release`.
        // ADR-0124: the caller may do so only when its instruction decides the slack's fate.
        code: "E-SHIFT-SLACK",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0032",
        template: "the slack {from}\u{2013}{to} is {size} ms ({from_edges} \u{2192} \
{to_edges}) and this edit would change it to {new_size} ms. Release it with \
`release: [[{from}, {to}]]` only if the instruction you were given decides this slack's fate; \
otherwise surface this finding verbatim to whoever is operating Montagent.",
        status: Live,
        census: None,
        sets: &[],
    },
    CheckSpec {
        // ADR-0047: a `release` entry naming a pair that does not currently bound a real,
        // protected slack this edit would change is itself a refusal — `release` cannot be
        // populated speculatively or reused stale across calls.
        code: "E-SHIFT-RELEASE-INVALID",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0047",
        template: "`release` names {from}\u{2013}{to}, which is not a slack this edit would \
change \u{2014} release only pairs this call's own refusal reports, and only for the call \
that reported them.",
        status: Live,
        census: None,
        sets: &[],
    },
    // ---- `compare` (#221). -----------------------------------------------------------
    //
    // Four codes, one per predicate the ADR series names, and every one is `Drift`-class:
    // ADR-0063/ADR-0066 are explicit that these are "facts with no severity... it
    // describes what changed, it does not judge it" — `render` never consults a `compare`
    // finding, which is what a fifth report category rather than a fourth severity buys.
    // `D-` for "drift", on this table's own `E-`/`R-`/`N-`/`U-`/`L-` prefix convention.
    CheckSpec {
        // ADR-0032: a slack that exists at a nonzero distance in the reference document
        // and changed size in the current one (including appearing or disappearing).
        // ADR-0066's exception lives in the check, not here: a slack that was at *zero*
        // distance in the reference is suppressed from this code and belongs to
        // `D-BOUNDARY-CLUSTER-DRIFT` alone, so the same edit is never reported twice.
        code: "D-SLACK-DRIFT",
        classes: &[Drift],
        repair: None,
        threshold: Internal,
        adr: "ADR-0032",
        template: "the slack {from}\u{2013}{to} ({from_edges} \u{2192} {to_edges}) was \
{ref_size} ms in {ref_project} and is {current_size} ms in the current one.",
        status: Live,
        census: None,
        sets: &[CheckSet::Drift],
    },
    CheckSpec {
        // ADR-0063: two instants numerically equal in the reference document that are no
        // longer equal in the current one. Exact equality only, never a preserved offset
        // — the whole reason this predicate needs no keyframe resolver. `{kind}` names
        // which of the three candidate populations produced the line: `a` (an element's
        // own keyframe against its own boundary), `b` (a keyframe against a *different*
        // element's boundary) or `c` (two different elements' same-property keyframe
        // times, keyed per-property — ADR-0039's retraction is why never per-`group`).
        code: "D-KEYFRAME-INSTANT-DRIFT",
        classes: &[Drift],
        repair: None,
        threshold: Internal,
        adr: "ADR-0063",
        template: "{left} and {right} both sat at {ref_at} in {ref_project} (case \
{kind}); in the current one they no longer coincide ({current_left} vs \
{current_right}).",
        status: Live,
        census: None,
        sets: &[CheckSet::Drift],
    },
    CheckSpec {
        // ADR-0066: a sibling predicate to the one above, not an extension of it — this
        // one requires no keyframe on either side. Scoped to all N\u{2265}2 boundary
        // coincidences, reporting one fact per destroyed cluster as a moved-set versus a
        // stayed-set, never pairwise: pairwise would give up to 55\u{00d7} the facts on the
        // real fixture for no more information. This is also the code that absorbs
        // `D-SLACK-DRIFT`'s zero-distance exception — a slack whose both ends coincide
        // with nothing else is a cluster of exactly two, and a cluster of one boundary
        // moving alone still gets a census with an empty stayed-set.
        code: "D-BOUNDARY-CLUSTER-DRIFT",
        classes: &[Drift],
        repair: None,
        threshold: Internal,
        adr: "ADR-0066",
        template: "boundaries {members} are coincident at {at} in the {which} version; \
in the {other} version, {moved}; {stayed}. ({ref_project} is the reference version.)",
        status: Live,
        census: Some(Counted),
        sets: &[CheckSet::Drift],
    },
    CheckSpec {
        // ADR-0051: karaoke drift gets an owner. A run whose `text` changed between the
        // two documents while its `highlight` object stayed byte/value-identical — the
        // one hardcoded fact this predicate looks for, not a generic text-diff mechanism.
        // Runs are correlated by index within an element matched by `id`; an element
        // whose run count differs between the two documents has no run identity to diff
        // against and is skipped for this check (see `compare::highlight_text_drift`).
        code: "D-HIGHLIGHT-TEXT-DRIFT",
        classes: &[Drift],
        repair: None,
        threshold: Internal,
        adr: "ADR-0051",
        template: "{element}: run {run_index}'s text changed from \"{ref_text}\" (in \
{ref_project}) to \"{current_text}\" while its highlight window did not.",
        status: Live,
        census: None,
        sets: &[CheckSet::Drift],
    },
];

impl CheckSpec {
    /// The class a finding takes unless the check computes a different one.
    pub fn default_class(&self) -> Class {
        self.classes[0]
    }

    /// Whether this check is allowed to emit `class`.
    pub fn may_emit(&self, class: Class) -> bool {
        self.classes.contains(&class)
    }

    /// Whether any class this check may emit is `error` — the condition ADR-0043's
    /// repair field attaches to.
    pub fn may_error(&self) -> bool {
        self.may_emit(Class::Error)
    }
}

/// Every registered check, in declaration order.
pub fn all() -> &'static [CheckSpec] {
    CHECKS
}

/// The declaration for one code, or `None` if nothing registered it.
pub fn spec(code: &str) -> Option<&'static CheckSpec> {
    CHECKS.iter().find(|spec| spec.code == code)
}
