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
    pub repair: Option<RepairClass>,
    pub threshold: ThresholdProvenance,
    /// The ADR this check comes from.
    pub adr: &'static str,
    /// The prose template. `{name}` interpolates a field, or one of the location keys
    /// `file`, `line`, `column`, `byte_offset`, `track`, `element`.
    pub template: &'static str,
    pub status: Status,
}

use Class::{Error, Layout, Note, Review, Unchecked};
use RepairClass::{Advise, Refuse};
use Status::{Declared, Live};
use ThresholdProvenance::{External, Internal};

const CHECKS: &[CheckSpec] = &[
    // ---- This ticket's own checks (#188). -----------------------------------------
    CheckSpec {
        code: "E-PARSE",
        classes: &[Error],
        // The bytes could not be read as JSON, so there is no document to derive a fix
        // from — the one condition under which "the fix is fully determined by the
        // document" is not merely unmet but unmeetable.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0011",
        template: "{file} is not valid JSON: {reason}, at line {line}, column {column} (byte {byte_offset}).",
        status: Live,
    },
    CheckSpec {
        code: "E-READ",
        classes: &[Error],
        // Advise, like `E-INVOCATION` and unlike `E-PARSE`. Neither of ADR-0043's two
        // classes fits a file that was never opened — there is no document whose author
        // could have meant anything — so the choice is which distortion is smaller.
        // Refuse carries a guarantee that is vacuous here (no flag was ever going to
        // lift a missing file) and tells the agent to stop and escalate to a human over
        // what is usually a mistyped path; the OS states the condition and the next move
        // follows from it.
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0011",
        template: "{file} could not be read: {reason}.",
        status: Live,
    },
    CheckSpec {
        // ADR-0042's refusal. The reported hazard is wrong-file destruction — `fmt`
        // pointed at a transcript export or a beats export will rewrite it — so the
        // proportionate fix is an identity check, not a correctness gate. The three keys
        // it names are `crate::permissive::REQUIRED`, and the template says them rather
        // than dumping a raw schema error, which is the message-quality gap the ADR found.
        code: "E-NOT-A-PROJECT",
        classes: &[Error],
        // Advise, on `E-READ`'s reasoning: there is no document whose author could have
        // meant anything, because the document is not a project. The next move — point the
        // tool at the project file — follows from the condition itself.
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0042",
        template: "{file} does not look like a Montaget project file — no {missing}.",
        status: Live,
    },
    CheckSpec {
        // `create_project` scaffolds, and a scaffold that overwrites is a scaffold that
        // deletes a project. Its own code rather than `E-WRITE` or a bare `E-INVOCATION`,
        // because this is the one failure of a write tool that is not a failure at all: the
        // file is intact, and the agent has learned that the thing it was about to create
        // already exists — which, mid-session, is usually the answer it wanted.
        //
        // No ADR says `create_project` refuses to overwrite, or what it says when it does.
        // Raised as #246 rather than left to be discovered from this table.
        code: "E-PROJECT-EXISTS",
        classes: &[Error],
        // Advise, on `E-READ`'s reasoning: the document whose author could have meant
        // something is not this call's, and the next move — write somewhere else, or edit
        // the file that is already there — follows from the condition itself.
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0011",
        template: "{file} already exists; `create_project` never overwrites. Edit it, or \
scaffold somewhere else.",
        status: Live,
    },
    CheckSpec {
        code: "E-INVOCATION",
        classes: &[Error],
        // Advise, not refuse: exit 3's next move is "fix the command" (ADR-0011), and
        // the usage text states it. Nothing about the author's intent is in question.
        repair: Some(Advise),
        threshold: Internal,
        adr: "ADR-0011",
        template: "{reason}",
        status: Live,
    },
    CheckSpec {
        code: "E-INTERNAL",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0011",
        template: "Montaget failed internally: {reason}",
        status: Live,
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
        template: "{element}: `{source}` holds {probed_duration} ms ({axis}), and the declared \
source range {source_start}..{source_end} ({declared_source_span} ms) reaches {over_by} ms past \
it.",
        status: Live,
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
    // `E-PARSE` and `E-ANCHOR-CHAIN` are already refuse-class without one, so the practice
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
        // Montaget version before removing it. Do not delete the key to make the file
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
        template: "{subject}: unknown key `{key}` — not a key this Montaget knows, and not one \
it has retired. It may belong to a newer format revision than this binary implements. Check \
your Montaget version before removing it. Do not delete the key to make the file validate. \
Here the format publishes {expected}.",
        status: Live,
    },
    CheckSpec {
        // Everything else the published types refuse: a value of the wrong type, a required
        // key absent, a value outside a closed vocabulary, a `null` where the convention
        // omits. One code rather than three, because the three are one condition — *this
        // value is not one the format publishes* — and one repair: write a value it does.
        code: "E-SCHEMA",
        classes: &[Error],
        // Refuse, on `E-PARSE`'s stated reasoning one level up. `E-PARSE` refuses because
        // *"there is no document to derive a fix from"*; here there are bytes and a tree,
        // but the part of the tree the finding is about is exactly the part the format
        // cannot read — so the fix is whatever the author meant by it, which is the
        // condition ADR-0043 reserves for refusal. The prose renderer already states the
        // guarantee in words on every refuse-class finding, so the template does not.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0017",
        // The reason is the format's own words wherever the types state one — `#FBF3E3FF is
        // the opaque form of #FBF3E3`, `not integer milliseconds`, `written as null; omit
        // the key instead` — rather than a sentence this table would have to keep in step
        // with them.
        template: "{subject} does not fit the published schema: {reason}.",
        status: Live,
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
says this with {replacement}. Surface this finding verbatim to whoever is operating Montaget; \
do not repair it by ordinary file edit.",
        status: Live,
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
    // No ADR names any of these four spellings — ADR-0004, ADR-0006 and ADR-0020 each
    // state the condition and none states a code. That is surface the ADR series has not
    // ratified, raised as #255 rather than left to be discovered from this table.
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
        template: "`{element}` ({start}..{end}) and `{other}` ({other_start}..{other_end}) \
are both in track `{track}` and overlap by {overlap} ms.",
        status: Live,
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
        template: "`{font}` calls itself `{name}`, which matches `{matched}` on Montaget's \
blocklist of known non-redistributable fonts (Apple's system fonts are licensed for UI \
mockups on Apple's own platforms and may not be embedded in other software). Nothing was \
copied, and no flag will copy it. Known open substitutes, none of them metric-compatible: \
{substitutes}.",
        status: Live,
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
        template: "`{font}` ({name}) carries no licence Montaget recognises — {detail} — so \
nothing was copied. Once a human has confirmed the file may be redistributed, re-run with \
`--licence <identifier>` to record that declaration; a false declaration is the declarer's \
liability, recorded in the file.",
        status: Live,
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
records its licence or the bytes that were vendored. Run `montaget fonts vendor` on it.",
        status: Live,
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
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0007",
        template: "{element}: nothing in the `{font}` chain has a glyph for {characters}, \
so it renders as .notdef. The chain is {chain}.",
        status: Live,
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
        // `E-PARSE`, `E-ANCHOR-CHAIN` and the two schema codes are already refuse-class
        // without one, so the practice is that the census attaches where a sibling group
        // exists. Here none does: the fault is one boundary inside one element's own
        // string, and there is no observable other elements could be grouped by that would
        // narrow it. `E-FONT-NO-GLYPH` is the contrast — its group, the project's other
        // chains that do map the characters, is real and it carries one.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0007",
        template: "{element}: the boundary before run {run} falls inside {cluster} \
({codepoints}), so the mark shapes with no base. A run boundary is style only (ADR-0007) — \
move it off the cluster.",
        status: Live,
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
({x}, {y}), and the frame is {frame_width}\u{d7}{frame_height}.",
        status: Live,
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
    },
    CheckSpec {
        code: "L-KEY-ORDER",
        classes: &[Layout],
        repair: None,
        threshold: Internal,
        adr: "ADR-0041",
        template: "{element} (line {line}): key order does not match the schema for `{type}`; expected {expected}. Run `montaget fmt`.",
        // Live as of #193, which supplies the key-order predicate and the `fmt --check`
        // that reads it. `validate`'s `LAYOUT` check calls the same predicate and fires
        // the same code; it is a later ticket, and that is a second call site rather than
        // a second check.
        status: Live,
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
        // ADR-0041 specifies one finding shape and it is the element-scoped one, so this
        // code is surface the ADR series has not ratified. Raised as #241 rather than left
        // to be discovered from the table.
        code: "L-LAYOUT",
        classes: &[Layout],
        repair: None,
        threshold: Internal,
        adr: "ADR-0041",
        template: "{file} is not written in the canonical convention: {written_lines} lines as \
written, {canonical_lines} in canonical form, first difference at line {line}. Run `montaget \
fmt`.",
        status: Live,
    },
    // ---- `shift` (#220). ------------------------------------------------------------
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
    },
    CheckSpec {
        // ADR-0032/ADR-0047: every slack is invariant by default. `shift` refuses an edit
        // that would change one's size unless the caller names it, in full, in `release`.
        code: "E-SHIFT-SLACK",
        classes: &[Error],
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0032",
        template: "the slack {from}\u{2013}{to} is {size} ms ({from_edges} \u{2192} \
{to_edges}) and this edit would change it to {new_size} ms. Release it explicitly with \
`release: [[{from}, {to}]]` if that is intended.",
        status: Live,
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
