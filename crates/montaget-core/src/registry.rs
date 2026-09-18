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
    // ---- Declared by the ADR series; the checks themselves are later tickets. ------
    CheckSpec {
        code: "E-SOURCE-OVERRUN",
        classes: &[Error],
        // The document says the source is one length and the disk says another; which
        // of the two is the mistake is not readable off either.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0006",
        template: "{element}: the source is {probed_duration} ms on disk; the declared source span of {declared_source_span} ms at speed {speed} needs {timeline_span} ms of timeline.",
        status: Declared,
    },
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
    CheckSpec {
        code: "R-VISUAL-GAP",
        // ADR-0006's own worked example of severity computed per instance: "a gap whose
        // interval is uncovered in that union is `review`, and every other visual gap is
        // a note." (What "uncovered" means is #23's, not this ticket's.)
        classes: &[Review, Note],
        repair: None,
        threshold: Internal,
        adr: "ADR-0006",
        template: "nothing on any visual track from {from} ms to {to} ms.",
        status: Declared,
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
        // The first and, to date, only member of ADR-0061's fenced exception.
        threshold: External {
            source: "Netflix and BBC timed-text guidance",
            adr: "ADR-0034",
        },
        adr: "ADR-0034",
        template: "{element}: {measured_cps} characters per second, against a threshold of {threshold_cps}.",
        status: Declared,
    },
    CheckSpec {
        code: "N-QUANTIZATION",
        classes: &[Note],
        repair: None,
        threshold: Internal,
        adr: "ADR-0006",
        template: "quantization changes {changed} boundaries.",
        status: Declared,
    },
    CheckSpec {
        code: "U-SOURCE-UNPROBEABLE",
        classes: &[Unchecked],
        repair: None,
        threshold: Internal,
        adr: "ADR-0013",
        template: "{source} could not be probed, so the disk half of the question is unanswered.",
        status: Declared,
    },
    CheckSpec {
        code: "L-KEY-ORDER",
        classes: &[Layout],
        repair: None,
        threshold: Internal,
        adr: "ADR-0041",
        template: "{element} (line {line}): key order does not match the schema for `{type}`; expected {expected}. Run `montaget fmt`.",
        status: Declared,
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
