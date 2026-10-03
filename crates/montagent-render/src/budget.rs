//! The performance budgets, and the harness later tickets assert them through.
//!
//! [ADR-0021] splits the budget in two and forbids collapsing the halves into one
//! number: `frame` (a single still, the agent's per-turn self-check) and the
//! render/preview **spans** answer different questions — *is my edit right* versus
//! *does it look right in motion* — and [#34] measured them not moving together
//! (`frame` held flat at 4K while preview time regressed 4–6×).
//!
//! **Two of the four arms are enforced and two are deliberately not**, and the
//! unenforced pair is the interesting decision. ADR-0021 states that for a
//! full-resolution preview *"the caller explicitly asked for true pixels and
//! accepted the cost, so there is no promise for a target to encode"*, and that a
//! measured reference example is a **better** regression baseline than a guessed
//! ceiling — it flags a real 2× drift that a loose invented bound would pass, and
//! it does not false-alarm on legitimately heavier input. `render` is unenforced
//! **in CI** for a different reason: it has a target, [`RENDER_TARGET`], but the
//! target is stated for one machine and is judged by a test run on purpose there,
//! never by the suite. See [`Budget::Render`].
//!
//! An unenforced arm reports drift against its own recorded measurements —
//! [`FULL_RESOLUTION_PREVIEW_REFERENCES`] and [`RENDER_REFERENCES`] — and never
//! fails. The lists are per-arm and [`nearest_reference`] will not cross between
//! them: a 720p-capped proxy preview and a render at the declared frame answer
//! different questions at different pixel counts, so one arm's number scoring the
//! other's run would be drift against nothing.
//!
//! The numbers live in one place so that a change to one is a visible diff, and
//! so the verb tickets have somewhere to assert (#189) rather than each inventing
//! a number.
//!
//! [ADR-0021]: ../../../../docs/adr/0021-preview-budget-and-graceful-degradation.md
//! [#34]: https://github.com/MBehtemam/Montagent/issues/34

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

/// `render`'s speed target: **the benchmark project renders in at most this,
/// median wall clock, on the dev's M1 Pro** ([ADR-0142]). The one place the
/// number lives.
///
/// **The benchmark project** is what `fixtures/benchmark/make_benchmark.py`
/// builds from the committed fixtures: 6 minutes of 1920x1080 at 30 fps, three
/// `video` elements visible at once for most of the timeline (sources re-encoded
/// to a pinned codec, profile and 10 s GOP), text, rects and an audio mix. It is
/// never the "reference project": [`RENDER_REFERENCES`] already means reference
/// *measurements*.
///
/// Half of real time, and fixed: the first clean measurement confirms it and does
/// not adjust it, and only a superseding ADR changes it. It is stated for one
/// scenario on one machine (8P+2E, 16 GB), so it is **not enforced in CI** —
/// [`Budget::Render`]'s [`Budget::limit`] stays `None`. It is judged by
/// `crates/montagent/tests/render_target.rs`, an `#[ignore]`d test run on purpose
/// with `cargo test --release -- --ignored`, which follows ADR-0142's protocol
/// (one discarded warm-up, five timed whole `montagent render` runs, the median
/// judged) and refuses to judge on a debug build, a loaded machine, battery
/// power, or frame hashes that differ from a sequential render's.
///
/// Only 1080p30 is committed. The 2160p30 and one-video variants are measured
/// and recorded in [`BENCHMARK_REFERENCES`] with no ceiling.
///
/// [ADR-0142]: ../../../../docs/adr/0142-render-has-one-speed-target-the-benchmark-project-in-three-minutes.md
pub const RENDER_TARGET: Duration = Duration::from_secs(3 * 60);

/// A measured example of what one arm's work actually cost.
///
/// Not a limit. ADR-0021 makes the observational arms report the spec's *"measured
/// reference examples rather than inventing a target"*.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reference {
    /// Which rasterizer produced the number.
    ///
    /// Load-bearing rather than descriptive: #34 measured the same span at
    /// 19.04 s through `skia-safe` and 30.0 s through `tiny-skia`, so a
    /// measurement scored against the wrong arm's baseline is off by 1.58x —
    /// enough to hide exactly the 2x drift this arm exists to flag. Only
    /// [`SHIPPED_RASTERIZER`] entries are eligible as a baseline; the other arm
    /// is recorded because ADR-0010 keeps `tiny-skia` as the named exit and a
    /// number for it is worth having if that exit is ever taken.
    pub rasterizer: &'static str,
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

/// The rasterizer Montagent ships (ADR-0010). A reference from any other arm is
/// kept as a record but is never chosen as a baseline.
pub const SHIPPED_RASTERIZER: &str = "skia-safe";

/// Every full-resolution preview number this project has actually measured.
///
/// Add to it; do not replace an entry, because an entry is a record of what was
/// once true on stated hardware rather than a current expectation. Order carries
/// no meaning — [`nearest_reference`] breaks a tie deterministically on the
/// entry's own values rather than on its position, so appending is safe.
pub const FULL_RESOLUTION_PREVIEW_REFERENCES: &[Reference] = &[
    Reference {
        rasterizer: "skia-safe",
        conditions: "2160x3840/30, one video clip on the timeline, M1 Pro",
        output_ms: 10_000,
        elapsed_ms: 19_040,
        source: "ADR-0021, from #34",
    },
    Reference {
        rasterizer: "tiny-skia",
        conditions: "2160x3840/30, one video clip on the timeline, M1 Pro",
        output_ms: 10_000,
        elapsed_ms: 30_000,
        source: "#34 (the named exit's arm, recorded for the same span)",
    },
];

/// Every `render` number this project has actually measured.
///
/// Appended to on the same terms as the preview list above, and read on the same
/// terms: a record of what was once true on stated hardware, never a promise.
/// `conditions` states the frame size and rate the number was taken at, because
/// ADR-0003 put 4K in ordinary scope and a wall clock with no frame behind it is
/// comparable to nothing (#217).
///
/// **The spread is why more than one reading is recorded.** #217 ran the harness
/// three times on one machine, minutes apart, with nothing changed between them:
/// 19.78 s, 18.64 s, 19.64 s. Its two entries below are that range's ends — the
/// middle reading says nothing the ends do not — and taken with the 17.3 s #215
/// recorded, the list spans 1.14× end to end. Any ceiling derived from one
/// reading would be derived from that noise as much as from the code.
///
/// A fourth reading, on the same machine while it was compiling something else,
/// came back at 22.10 s. It is deliberately **not** an entry: [`FIXTURE_CONDITIONS`]
/// says "cold", and a run competing for cores is not the thing the other three
/// measured. It is recorded in this sentence instead, because a 1.28× swing from
/// machine load alone is the sharpest argument available for why none of these
/// numbers is a ceiling.
///
/// [`nearest_reference`] baselines against the fastest, so ordinary run-to-run
/// variation surfaces as drift above 1.0 rather than hiding beneath it.
///
/// `budget.rs`'s `the_recorded_spread_is_the_one_the_prose_states` re-derives
/// every number in the paragraph above from the [`FIXTURE_CONDITIONS`] entries
/// themselves, so the prose cannot drift away from the list it describes.
///
/// **None of these is the target.** [`RENDER_TARGET`] is stated for the benchmark
/// project, not derived from this list. ADR-0142's observations of the benchmark
/// project are kept apart, in [`BENCHMARK_REFERENCES`], so that a reading of a
/// different project is never this list's baseline or part of its spread.
pub const RENDER_REFERENCES: &[Reference] = &[
    Reference {
        rasterizer: "skia-safe",
        conditions: FIXTURE_CONDITIONS,
        output_ms: RENDER_REFERENCE_OUTPUT_MS,
        elapsed_ms: 17_300,
        source: "#215, the first whole-fixture render",
    },
    Reference {
        rasterizer: "skia-safe",
        conditions: FIXTURE_CONDITIONS,
        output_ms: RENDER_REFERENCE_OUTPUT_MS,
        elapsed_ms: 19_780,
        source: "#217, the slowest of its three readings minutes apart",
    },
    Reference {
        rasterizer: "skia-safe",
        conditions: FIXTURE_CONDITIONS,
        output_ms: RENDER_REFERENCE_OUTPUT_MS,
        elapsed_ms: 18_640,
        source: "#217, the fastest of its three readings minutes apart",
    },
];

/// Every observed `render` of the **benchmark project** (ADR-0142): its 2160p30
/// and one-video variants, recorded with no ceiling.
///
/// Kept apart from [`RENDER_REFERENCES`] rather than appended to it. Those are
/// the committed fixture's readings, and [`nearest_reference`] scores a run by
/// output length alone, so a six-minute benchmark reading there would become the
/// baseline for any long render and would move the fixture's recorded spread.
/// Here a reading is a record, appended on the same terms and never consulted as
/// a baseline: the benchmark project is judged only by `render_target.rs`
/// against [`RENDER_TARGET`]. Each entry's `conditions` names its variant, frame
/// size and rate.
pub const BENCHMARK_REFERENCES: &[Reference] = &[Reference {
    rasterizer: "skia-safe",
    conditions: "1920x1080/30 fps, the benchmark project (ADR-0142), cold with an empty \
        probe sidecar, release build, M1 Pro; OBSERVED under load 6.0 rising to 9.7, one run \
        and no warm-up, so an upper bound and not the protocol's median",
    output_ms: 360_000,
    elapsed_ms: 135_360,
    source: "ADR-0142 and ADR-0141 §8, the streaming PR's observed after-run",
}];

/// What every entry in [`RENDER_REFERENCES`] was taken over.
///
/// One fixture, and that is worth seeing: it is a single frame size, so these
/// numbers say nothing about the 4K [`Budget::Render`] names as the rest of the
/// gap.
pub const FIXTURE_CONDITIONS: &str = "1080x1920/25 fps, the committed \
    `en-halloween-decorating` fixture (60 elements, 22 carrying text, 20 narration \
    elements mixed), cold with an empty probe sidecar, release build, M1 Pro";

/// The fixture's declared `duration`, and so the span every render reference
/// above was taken over. Named here because `render_budget.rs` asserts the run it
/// times produced exactly this much output — a number measured over a different
/// span is not the number recorded.
pub const RENDER_REFERENCE_OUTPUT_MS: i64 = 65_216;

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
    /// **Observational in CI: `limit() == None` means not enforced in CI, not "no
    /// target".**
    ///
    /// The target is [`RENDER_TARGET`], stated by [ADR-0142] for one scenario —
    /// the benchmark project — on one machine, and judged by the `#[ignore]`d
    /// `render_target.rs` run on purpose there. CI gets no wall-clock gate: its
    /// `macos-15` runner is a shared 3-vCPU M1, and ADR-0021 keeps the expensive
    /// arms observational. ADR-0142 gives CI a deterministic spawn-count test
    /// instead, which catches the structural regression (one `ffmpeg` per frame)
    /// and lands with the streaming feeds.
    ///
    /// The budget this arm inherited was *"a 60 s video renders in under two
    /// minutes"*. ADR-0021 records that pair as *"written for 1080x1920/30 and
    /// never re-derived"* after [ADR-0003] generalised the scope, and ADR-0072
    /// retired it. **It does not come back as a rate**: `RENDER_TARGET` is an
    /// absolute number for one project, not seconds per output second, because
    /// render cost is not linear in length, frame size or element count.
    ///
    /// No measurement in [`RENDER_REFERENCES`] or [`BENCHMARK_REFERENCES`] is
    /// entitled to become a ceiling by sitting in this file; the 2160p30 and
    /// one-video observations ADR-0142 records carry none.
    ///
    /// [ADR-0003]: ../../../../docs/adr/0003-general-video-editor-not-channel-tooling.md
    /// [ADR-0142]: ../../../../docs/adr/0142-render-has-one-speed-target-the-benchmark-project-in-three-minutes.md
    Render,
}

impl Budget {
    /// Whether this budget is judged over a span of output. `frame` is the one
    /// that is not: ADR-0021 keeps the two halves of the budget separate and
    /// forbids collapsing them.
    pub fn is_span(self) -> bool {
        !matches!(self, Budget::Frame)
    }

    /// The wall-clock ceiling CI enforces for this much work, or `None` when the
    /// budget is observational there. For [`Budget::Render`] that is not the
    /// absence of a target: see [`RENDER_TARGET`].
    ///
    /// # Panics
    ///
    /// If the budget and the work disagree about whether there is a span. A still
    /// scored against a span arm has an output length of zero: on an enforced arm
    /// that is a measurement judged against a ceiling written for video, and on an
    /// observational one it is drift scored from a reference for a span it never
    /// rendered. Either way the answer is a number about the wrong thing, which is
    /// worse than no answer. It is a caller bug and says so.
    #[track_caller]
    pub fn limit(self, work: Work) -> Option<Duration> {
        assert_eq!(
            self.is_span(),
            matches!(work, Work::Span { .. }),
            "`{}` is judged over {}, and was handed {work:?}",
            self.name(),
            if self.is_span() {
                "a span of output"
            } else {
                "one still"
            }
        );
        match self {
            Budget::Frame => Some(FRAME_LIMIT),
            Budget::ScrubPreview => Some(SCRUB_PREVIEW_LIMIT),
            Budget::FullResolutionPreview | Budget::Render => None,
        }
    }

    /// Whether a miss is a failure. False for the two observational arms, each
    /// for its own reason — see [`Budget::FullResolutionPreview`] and
    /// [`Budget::Render`].
    pub fn is_enforced(self) -> bool {
        !matches!(self, Budget::FullResolutionPreview | Budget::Render)
    }

    /// The measurements recorded for this arm, which is where its drift is scored
    /// from. Empty for an enforced arm: it judges against its stated number, and a
    /// reference it never consults would be a number with no reader.
    pub fn references(self) -> &'static [Reference] {
        match self {
            Budget::Frame | Budget::ScrubPreview => &[],
            Budget::FullResolutionPreview => FULL_RESOLUTION_PREVIEW_REFERENCES,
            Budget::Render => RENDER_REFERENCES,
        }
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
                let reference = nearest_reference(self, work);
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

/// The nearest recorded reference for a piece of work, from `budget`'s own arm.
///
/// Drawn only from [`Budget::references`], never from another arm's list: the two
/// observational arms measure different pixel counts of different things, so a
/// preview number scoring a render would be drift against nothing.
///
/// Nearest by output length, since that is the axis the references vary along,
/// and restricted to [`SHIPPED_RASTERIZER`] so a `tiny-skia` record can never
/// become a `skia-safe` measurement's baseline.
///
/// A remaining tie is broken on the **fastest** entry rather than on array
/// position. Two properties come out of that. Appending a reference cannot
/// silently re-baseline an existing comparison unless the new run was faster than
/// everything recorded — and a run that fast is the one worth baselining against,
/// because it is the best this code has been observed to do. And the error it can
/// make is to report more drift than a slower record would, never less: an arm
/// that can only over-report is one whose silence means something.
///
/// (This paragraph said *slower* until #217, while the code and its test had
/// always said faster. The contradiction was invisible while the only list with a
/// baseline held one eligible entry; `render`'s two made it live.)
///
/// The distance is `saturating_sub` rather than `-`: nothing clamps a negative
/// span here, and a caller's bad arithmetic should not be an overflow panic.
pub fn nearest_reference(budget: Budget, work: Work) -> Option<Reference> {
    let target = work.output_ms();
    budget
        .references()
        .iter()
        .copied()
        .filter(|r| r.rasterizer == SHIPPED_RASTERIZER)
        .min_by_key(|r| {
            (
                r.output_ms.saturating_sub(target).saturating_abs(),
                r.elapsed_ms,
            )
        })
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
