//! The performance budgets, and the harness later tickets assert them through.
//!
//! [ADR-0021] splits the budget in two and forbids collapsing the halves into one
//! number: `frame` (a single still, the agent's per-turn self-check) and the
//! render/preview **spans** answer different questions — *is my edit right* versus
//! *does it look right in motion* — and [#34] measured them not moving together
//! (`frame` held flat at 4K while preview time regressed 4–6×).
//!
//! Three of the four arms are enforced and one is deliberately not. The
//! unenforced one is the interesting decision: ADR-0021 states that for a
//! full-resolution preview *"the caller explicitly asked for true pixels and
//! accepted the cost, so there is no promise for a target to encode"*, and that a
//! measured reference example is a **better** regression baseline than a guessed
//! ceiling — it flags a real 2× drift that a loose invented bound would pass, and
//! it does not false-alarm on legitimately heavier input. So that arm reports
//! drift against [`FULL_RESOLUTION_PREVIEW_REFERENCES`] and never fails.
//!
//! Nothing in this module measures anything yet, because no verb exists to
//! measure. It exists now so that the verb tickets have somewhere to assert
//! (#189) rather than each inventing a number, and so the numbers live in one
//! place where a change to one is a visible diff.
//!
//! [ADR-0021]: ../../../../docs/adr/0021-preview-budget-and-graceful-degradation.md
//! [#34]: https://github.com/MBehtemam/Montaget/issues/34

use std::fmt;
use std::time::{Duration, Instant};

/// `frame` must come back in under this, **cold** — a fully cold process:
/// launch, load the project, resolve fonts, decode the still, rasterize, encode.
///
/// Resolution-independent, and that independence is the claim being protected:
/// `frame`'s cost has never once tracked project size in any measurement this
/// project has taken (0.11–0.27 s cold at 1080p; 28.76 ms warm at 2160×3840).
/// The number sits ~2× above the worst cold value measured anywhere, so it will
/// not fire on noise (ADR-0021).
pub const FRAME_LIMIT: Duration = Duration::from_millis(500);

/// The scrub preview — the default every caller hits — must come back in under
/// this, at the 720p proxy target (long edge capped at 1280 px, ADR-0046).
///
/// A miss degrades exactly one tier to 540p with mandatory disclosure and
/// hard-fails if that still misses (ADR-0067). The ladder is the `preview`
/// ticket's; this constant is only the number the ladder is measured against.
pub const SCRUB_PREVIEW_LIMIT: Duration = Duration::from_secs(5);

/// Wall clock allowed per second of rendered output.
///
/// **This is the one budget stated here as a rate rather than as the pair it was
/// written as, and that is an assumption worth seeing.** The original budget is
/// *"a 60 s video renders in under two minutes"* — one point, not a curve.
/// ADR-0021 amended the preview half of the map's informal budget and left this
/// half alone, so nothing has re-derived it. Reading it linearly is what lets a
/// 10 s test assert anything at all; it is recorded here rather than buried in
/// whichever ticket first needed it, so that the moment a measurement disagrees
/// there is a single line to change and an ADR to write.
pub const RENDER_MS_PER_OUTPUT_SECOND: u64 = 2_000;

/// A measured example of what a full-resolution preview actually cost.
///
/// Not a limit. ADR-0021 makes this arm observational: the spec *"states measured
/// reference examples rather than inventing a target"*.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reference {
    /// What was rendered, and on what — enough for a later reader to tell
    /// whether their own run is comparable.
    pub conditions: &'static str,
    /// Length of the rendered span, in milliseconds.
    pub output_ms: i64,
    /// The wall clock it took, in milliseconds.
    pub elapsed_ms: u64,
    /// Where the number came from.
    pub source: &'static str,
}

/// Every full-resolution preview number this project has actually measured.
///
/// Add to it; do not replace an entry, because an entry is a record of what was
/// once true on stated hardware rather than a current expectation.
pub const FULL_RESOLUTION_PREVIEW_REFERENCES: &[Reference] = &[
    Reference {
        conditions: "2160x3840/30, one video clip on the timeline, M1 Pro, skia-safe",
        output_ms: 10_000,
        elapsed_ms: 19_040,
        source: "ADR-0021, from #34",
    },
    Reference {
        conditions: "2160x3840/30, one video clip on the timeline, M1 Pro, tiny-skia",
        output_ms: 10_000,
        elapsed_ms: 30_000,
        source: "#34 (the named exit's arm, recorded for the same span)",
    },
];

/// How far an observational measurement may drift from its reference before
/// [`Verdict::is_notable`] says so.
///
/// ADR-0021's own worked example of what the reference is for is *"a real 2×
/// drift"*, so that is the factor.
pub const OBSERVATIONAL_DRIFT_FACTOR: f64 = 2.0;

/// The unit of work a budget is judged over.
///
/// `frame` is a still and has no span; everything else is a span, and two of the
/// three span budgets are flat rather than proportional — a scrub preview is
/// capped in absolute wall clock regardless of how long the scrub is, because
/// that is the number ADR-0021 states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// One frame, at true project resolution (ADR-0021: `frame` is never
    /// proxy-scaled).
    Still,
    /// A rendered span, by the length of the output it produces.
    Span { output_ms: i64 },
}

impl Work {
    /// A span, from its output length in milliseconds.
    pub fn span(output_ms: i64) -> Self {
        Work::Span { output_ms }
    }

    fn output_ms(self) -> i64 {
        match self {
            Work::Still => 0,
            Work::Span { output_ms } => output_ms,
        }
    }
}

/// Which budget a measurement is being judged against.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Budget {
    /// `frame`, cold, at true pixels. The agent's per-turn loop.
    Frame,
    /// `preview` at the proxy target — the default path.
    ScrubPreview,
    /// `preview` with the explicit full-resolution escape hatch. Observational.
    FullResolutionPreview,
    /// `render` — the deliverable, always at full declared resolution.
    Render,
}

impl Budget {
    /// The wall-clock ceiling for this much work, or `None` when the budget is
    /// observational and there is deliberately no ceiling to encode.
    pub fn limit(self, work: Work) -> Option<Duration> {
        match self {
            Budget::Frame => Some(FRAME_LIMIT),
            Budget::ScrubPreview => Some(SCRUB_PREVIEW_LIMIT),
            Budget::FullResolutionPreview => None,
            Budget::Render => {
                let ms = (work.output_ms().max(0) as u64)
                    .saturating_mul(RENDER_MS_PER_OUTPUT_SECOND)
                    / 1_000;
                Some(Duration::from_millis(ms))
            }
        }
    }

    /// Whether a miss is a failure. False only for the full-resolution preview.
    pub fn is_enforced(self) -> bool {
        !matches!(self, Budget::FullResolutionPreview)
    }

    /// A short stable name, used in failure messages.
    pub fn name(self) -> &'static str {
        match self {
            Budget::Frame => "frame",
            Budget::ScrubPreview => "scrub preview",
            Budget::FullResolutionPreview => "full-resolution preview",
            Budget::Render => "render",
        }
    }

    /// Judge an already-taken measurement.
    pub fn judge(self, work: Work, elapsed: Duration) -> Verdict {
        match self.limit(work) {
            Some(limit) if elapsed <= limit => Verdict::Within { limit, elapsed },
            Some(limit) => Verdict::Exceeded { limit, elapsed },
            None => {
                let reference = nearest_reference(work);
                let drift = reference.map(|r| {
                    let scale = if r.output_ms > 0 && work.output_ms() > 0 {
                        work.output_ms() as f64 / r.output_ms as f64
                    } else {
                        1.0
                    };
                    let expected = r.elapsed_ms as f64 * scale;
                    if expected > 0.0 {
                        elapsed.as_secs_f64() * 1_000.0 / expected
                    } else {
                        f64::NAN
                    }
                });
                Verdict::Observed {
                    elapsed,
                    reference,
                    drift,
                }
            }
        }
    }

    /// Time `f`, judge it, and hand back both its value and the measurement.
    ///
    /// The seam later tickets plug into: a verb test calls this around the verb
    /// and then calls [`Measured::assert_within_budget`].
    pub fn measure<T>(self, work: Work, f: impl FnOnce() -> T) -> (T, Measured) {
        let start = Instant::now();
        let value = f();
        let elapsed = start.elapsed();
        (
            value,
            Measured {
                budget: self,
                work,
                elapsed,
                verdict: self.judge(work, elapsed),
            },
        )
    }
}

/// The nearest recorded reference for a piece of work — nearest by output
/// length, since that is the axis the references vary along.
fn nearest_reference(work: Work) -> Option<Reference> {
    let target = work.output_ms();
    FULL_RESOLUTION_PREVIEW_REFERENCES
        .iter()
        .copied()
        .min_by_key(|r| (r.output_ms - target).abs())
}

/// What a measurement came to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Verdict {
    /// Inside an enforced ceiling.
    Within { limit: Duration, elapsed: Duration },
    /// Outside an enforced ceiling. A failure.
    Exceeded { limit: Duration, elapsed: Duration },
    /// An observational arm: recorded, compared to the nearest reference, never
    /// failed. `drift` is the ratio of this run to that reference scaled to the
    /// same output length — `None` when there is no comparable reference.
    Observed {
        elapsed: Duration,
        reference: Option<Reference>,
        drift: Option<f64>,
    },
}

impl Verdict {
    /// Whether this verdict should fail a test.
    pub fn is_failure(self) -> bool {
        matches!(self, Verdict::Exceeded { .. })
    }

    /// Whether this verdict is worth a human's attention even though it is not a
    /// failure — an observational arm that has drifted past
    /// [`OBSERVATIONAL_DRIFT_FACTOR`] from its reference.
    pub fn is_notable(self) -> bool {
        match self {
            Verdict::Observed {
                drift: Some(drift), ..
            } => drift >= OBSERVATIONAL_DRIFT_FACTOR,
            _ => false,
        }
    }

    /// How long it actually took.
    pub fn elapsed(self) -> Duration {
        match self {
            Verdict::Within { elapsed, .. }
            | Verdict::Exceeded { elapsed, .. }
            | Verdict::Observed { elapsed, .. } => elapsed,
        }
    }
}

/// One timed piece of work, with its verdict attached.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measured {
    pub budget: Budget,
    pub work: Work,
    pub elapsed: Duration,
    pub verdict: Verdict,
}

impl Measured {
    /// Fail the calling test if an enforced budget was missed.
    ///
    /// The panic carries the budget, the ceiling, the measurement and the
    /// overshoot, so a CI failure is readable without re-running anything.
    #[track_caller]
    pub fn assert_within_budget(self) {
        if self.verdict.is_failure() {
            panic!("{self}");
        }
    }
}

impl fmt::Display for Measured {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ms = |d: Duration| d.as_secs_f64() * 1_000.0;
        match self.verdict {
            Verdict::Within { limit, elapsed } => write!(
                f,
                "{} within budget: {:.1} ms of {:.1} ms",
                self.budget.name(),
                ms(elapsed),
                ms(limit)
            ),
            Verdict::Exceeded { limit, elapsed } => write!(
                f,
                "{} over budget: {:.1} ms against a ceiling of {:.1} ms (over by {:.1} ms)",
                self.budget.name(),
                ms(elapsed),
                ms(limit),
                ms(elapsed) - ms(limit)
            ),
            Verdict::Observed {
                elapsed,
                reference,
                drift,
            } => match (reference, drift) {
                (Some(r), Some(d)) => write!(
                    f,
                    "{} observed (unenforced): {:.1} ms, {:.2}x the reference {} ms for {} [{}]",
                    self.budget.name(),
                    ms(elapsed),
                    d,
                    r.elapsed_ms,
                    r.conditions,
                    r.source
                ),
                _ => write!(
                    f,
                    "{} observed (unenforced): {:.1} ms, no comparable reference",
                    self.budget.name(),
                    ms(elapsed)
                ),
            },
        }
    }
}
