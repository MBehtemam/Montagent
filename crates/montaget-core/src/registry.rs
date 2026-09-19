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
        status: Declared,
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
