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
    /// raw measurement as the finding's substance and the source cited inline.
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
    pub class: Class,
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
        class: Error,
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
        class: Error,
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0011",
        template: "{file} could not be read: {reason}.",
        status: Live,
    },
    CheckSpec {
        code: "E-INVOCATION",
        class: Error,
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
        class: Error,
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0011",
        template: "Montaget failed internally: {reason}",
        status: Live,
    },
    // ---- Declared by the ADR series; the checks themselves are later tickets. ------
    CheckSpec {
        code: "E-SOURCE-OVERRUN",
        class: Error,
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
        class: Error,
        // ADR-0043's founding instance: 6 of 8 `gravity` deletions were geometric
        // no-ops and 2 silently changed the picture, and the fact separating them is
        // not in the document.
        repair: Some(Refuse),
        threshold: Internal,
        adr: "ADR-0043",
        template: "{element}: `{key}` is a retired spelling, carrying {value}.",
        status: Declared,
    },
    CheckSpec {
        code: "E-KEYFRAME-EASE",
        class: Error,
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
        class: Review,
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
        class: Note,
        repair: None,
        threshold: Internal,
        adr: "ADR-0058",
        template: "text \"{element}\" declares height {declared_height}; computed block height is {computed_height} — slack {slack}.",
        status: Declared,
    },
    CheckSpec {
        code: "R-CAPTION-PACE",
        class: Review,
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
        class: Note,
        repair: None,
        threshold: Internal,
        adr: "ADR-0006",
        template: "quantization changes {changed} boundaries.",
        status: Declared,
    },
    CheckSpec {
        code: "U-SOURCE-UNPROBEABLE",
        class: Unchecked,
        repair: None,
        threshold: Internal,
        adr: "ADR-0013",
        template: "{source} could not be probed, so the disk half of the question is unanswered.",
        status: Declared,
    },
    CheckSpec {
        code: "L-KEY-ORDER",
        class: Layout,
        repair: None,
        threshold: Internal,
        adr: "ADR-0041",
        template: "{element}: key order does not match the schema for `{type}`; expected {expected}. Run `montaget fmt`.",
        status: Declared,
    },
];

/// Every registered check, in declaration order.
pub fn all() -> &'static [CheckSpec] {
    CHECKS
}

/// The declaration for one code, or `None` if nothing registered it.
pub fn spec(code: &str) -> Option<&'static CheckSpec> {
    CHECKS.iter().find(|spec| spec.code == code)
}
