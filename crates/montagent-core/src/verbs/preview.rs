//! `preview` — a scrub preview under five seconds, with the caller always knowing what it
//! is looking at.
//!
//! ## The ladder
//!
//! `preview` renders a span at a **proxy resolution** rather than at the project's own:
//! 720p, the long edge capped at 1280 px, aspect preserved, rounded to even (ADR-0046).
//! A fixed cap rather than a fraction of the project, because the cost that dominates a
//! frame scales with pixel count and *"half of 16K is still 8K"* (ADR-0021).
//!
//! If that span runs past the `<5 s` scrub budget it degrades **exactly once**, to 540p,
//! and if 540p misses too it **hard-fails** rather than degrade again. Why there is no
//! third rung, and why one step cannot always rescue a miss, is
//! [`montagent_render::proxy`]'s to argue — it owns the rungs, the two floors and the
//! refusal text. **This module decides no number of its own**: what it walks, what it
//! discloses and what it puts in a refusal all come from there and from the ADRs, and
//! where a refusal quotes a figure the figure is theirs.
//!
//! **The two floors are not the same refusal** (ADR-0067). 540p is where the ladder gives
//! up on *wall-clock* grounds. 360p is where a frame stops being *readable*, measured on
//! the real fixture. The ladder stops at 540p, so the 360p guard cannot fire today — it is
//! checked on every rung anyway, so that the next rung anyone adds meets it.
//!
//! ## Disclosure is mandatory, not conditional
//!
//! Every answer carries the tier it was rendered at, whether or not it degraded (ADR-0021:
//! *"disclosure was never only a hedge against uncertainty; it is how a caller knows what
//! it is looking at"*). A preview is a lossy artefact by design, and an agent that cannot
//! tell 540p from 720p from true pixels will attribute the proxy's own softness to the
//! project.
//!
//! **The `<5 s` guarantee is `skia-safe`-specific.** `tiny-skia` is explicitly not
//! certified at 8K/720p (ADR-0065) — a single run at a 1.8% margin its own findings refuse
//! to certify.
//!
//! ## What this verb is not
//!
//! It is not `render`. Proxy degradation applies here and **never** to the deliverable
//! (ADR-0067). The two share one painter and one encode path —
//! [`crate::verbs::render::encode_span`] — so a preview is the render's picture on a
//! smaller device rather than a second picture; what differs is two fields, the surface
//! and the clock.
//!
//! It is not `frame` either. ADR-0021 keeps `frame` at true pixels unconditionally,
//! because the agent needs one tool it can trust for pixel-accurate checks *"without first
//! asking whether what it's looking at is a lie"*. `preview` answers the other question —
//! does it look right in motion — and pays resolution for it.
//!
//! ## What the ladder's ADRs do not state, ratified by ADR-0078 (#295)
//!
//! Everything below was a reading this verb had to pick and no ADR stated. **ADR-0078
//! ratifies each one**, and corrects the one place two of them met and the code named a
//! frame two ways — see [`rung_name`]. They are kept here because this is where they are
//! acted on; the ADR is what governs.
//!
//! - **`preview` is the ninth MCP verb**, and ADR-0011's table — eight MCP tools, eleven
//!   CLI commands — gains a row for it rather than the code losing one.
//! - **`preview` runs the identical check engine and refuses on any `error`**, as `render`
//!   does (ADR-0006). Stated for `render` and not for this verb, but a document the checks
//!   refuse is one no painter can be handed, and a preview that rendered what `render`
//!   would refuse would be the one artefact in the product that shows an illegal project.
//! - **A preview never lands on the deliverable, with or without a range.** `render`'s
//!   rule (story 56) is about *partial* renders; this is stronger and for an additional
//!   reason — every frame here may be proxy-scaled. The derived name is
//!   `out/<name>.preview.<from>-<to>.mp4`.
//! - **The budget is judged per attempt, on that attempt's own clock** — so a *degraded*
//!   preview can cost up to twice the budget, and the answer's `wall_ms` is the whole
//!   invocation, attempts included, because that is what the caller actually waited.
//!   ADR-0021 enforces `<5 s` for *"the common-case proxy-resolution scrub preview — the
//!   number every caller hits by default"*, which is one rung; the alternative readings
//!   are both worse. A whole-invocation clock would leave the second rung whatever was
//!   left after the first missed — usually nothing — so the degrade step ADR-0021 requires
//!   would be a step that could not land. Predicting the miss instead of measuring it is
//!   not available: the cost is what the render takes. A ladder that responds to a miss
//!   has spent the miss by definition.
//! - **A span that runs past its deadline is abandoned where it stands**, rather than
//!   finished and then judged. A ladder that ran every rung to completion would cost the
//!   sum of its rungs, which is the opposite of what a wall-clock budget is for.
//! - **The full-resolution escape hatch is never degraded.** ADR-0021 makes that arm
//!   observational — *"the caller explicitly asked for true pixels and accepted the cost,
//!   so there is no promise for a target to encode"* — so it runs with no deadline and
//!   discloses `native`.
//! - **The legibility floor governs a proxy resolution, never the author's declared
//!   frame.** A project declaring a frame below 360p is previewed at its own pixels:
//!   nothing downscaled it, and `preview` is not the verb that tells an author their
//!   project is too small. ADR-0050's *"any resolution request below 360x640-equivalent is
//!   a refusal"* reads the other way on its own; what settles it is ADR-0067, which is
//!   later and governs — it records the threshold as *"unreachable today"* and *"a guard
//!   on the ladder's future, not a live branch"*, and a declared frame that fired it would
//!   make it reachable today.
//! - **A project between the two caps is degraded to 540p even though the 720p cap never
//!   engaged for it.** Its first rung is true pixels — ADR-0046: *"no proxy applies at
//!   all"* — and its second is a real proxy. That follows from the ladder being defined on
//!   caps rather than on sizes, and neither ADR states it.
//! - **A rung whose cap never engaged is named `native`, not the rung's name** — in the
//!   tier disclosure, the attempt trace and a refusal alike ([`rung_name`]). ADR-0046 makes
//!   the field report *"native, the 720p proxy tier, or — once #117 resolves — a
//!   floor-refuse"*, and naming a frame for a cap that did nothing to it puts a number in
//!   front of the caller that was never rasterized.
//! - **A hard fail is exit 3**, not exit 1 and not exit 70: the document is legal and
//!   Montagent did not fail — what cannot be satisfied is the invocation, and the caller's
//!   levers are the ones exit 3 means, a shorter range or the escape hatch.

use std::path::{Path as FilePath, PathBuf};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;

use montagent_render::budget::SCRUB_PREVIEW_LIMIT;
use montagent_render::canvas::Rgba;
use montagent_render::proxy::{self, Tier};

use crate::media::sidecar::Sidecar;
use crate::media::{attest, probe, tools};
use crate::permissive::Loose;
use crate::report::{ExitCode, Report};
use crate::verbs::render::{self, Progress, Span, Stop, Surface, Video};

const TOOL: &str = "preview";

/// What one `preview` invocation is asking.
#[derive(Debug, Clone, Default)]
pub struct Ask {
    /// The start of the scrub, in absolute milliseconds. Asked for with `to`.
    pub from: Option<i64>,
    /// The end of the scrub, exclusive: the range is half-open `[from, to)`.
    pub to: Option<i64>,
    /// Where to write the preview instead of the derived name. Refused where it names the
    /// project's own `output`.
    pub output: Option<PathBuf>,
    /// ADR-0021's explicit full-resolution escape hatch: true pixels, for a check that is
    /// precision-sensitive. That arm of the budget is observational, so it is neither
    /// enforced nor degraded — the caller asked for the cost and accepted it.
    pub full: bool,
    /// What each rung is judged against. [`Clock::Scrub`] — ADR-0021's `<5 s` — is the
    /// default and the only thing either adapter can ask for.
    pub clock: Clock,
}

/// The wall clock the ladder is measured against.
///
/// Two arms, and the second is **not adapter surface**: neither the CLI nor MCP offers it,
/// and both pass [`Clock::Scrub`]. It exists because the alternative way to assert a
/// degrade — or the hard fail under it — is to find a machine slow enough to produce one,
/// which is a test that passes or fails on the hardware rather than on the code. The rungs
/// it states are the ladder's behaviour under a budget; which budget is [`Clock::Scrub`]'s
/// to say, and it says the one ADR-0021 wrote.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Clock {
    /// ADR-0021's scrub-preview budget, from [`SCRUB_PREVIEW_LIMIT`].
    #[default]
    Scrub,
    /// One wall clock per rung, in ladder order. A rung past the end of the list is judged
    /// against the last entry, so a one-entry list is one number for the whole ladder.
    Stated(Vec<Duration>),
}

impl Clock {
    /// The wall clock rung `index` is judged against.
    fn at(&self, index: usize) -> Duration {
        match self {
            Clock::Scrub => SCRUB_PREVIEW_LIMIT,
            Clock::Stated(rungs) => rungs
                .get(index)
                .or_else(|| rungs.last())
                .copied()
                .unwrap_or(SCRUB_PREVIEW_LIMIT),
        }
    }
}

/// One `preview` invocation's answer.
pub struct Answer {
    preview: Option<Preview>,
    report: Report,
}

impl Answer {
    pub fn report(&self) -> &Report {
        &self.report
    }

    /// The preview, where the run produced one.
    pub fn preview(&self) -> Option<&Preview> {
        self.preview.as_ref()
    }

    /// The canonical JSON: the report's own object plus the `preview` block — `null` where
    /// the preview was refused or failed, so a consumer reads the absence off a key that
    /// is always there.
    pub fn to_json(&self) -> Value {
        self.report.to_json_with(
            "preview",
            match &self.preview {
                Some(preview) => serde_json::to_value(preview).unwrap_or(Value::Null),
                None => Value::Null,
            },
        )
    }
}

/// The machine-readable result: `render`'s own block, plus the tier that produced it and
/// every rung the ladder tried on the way.
#[derive(Debug, Clone, Serialize)]
pub struct Preview {
    /// The file and its numbers, in the same shape `render` answers with — including
    /// `width`/`height`, which here are the tier's frame rather than the declared one.
    #[serde(flatten)]
    pub video: Video,
    /// Which resolution this is, and whether it is a degradation. Mandatory, always
    /// present, never conditional on having degraded (ADR-0021).
    pub tier: Disclosure,
    /// Every rung tried, in order — one entry on the common path, two where the ladder
    /// degraded. A caller that wants to know what the budget cost reads it here.
    pub attempts: Vec<Attempt>,
}

/// The tier disclosure ADR-0021 makes mandatory on every preview.
#[derive(Debug, Clone, Serialize)]
pub struct Disclosure {
    /// `720p`, `540p` or `native`.
    pub name: &'static str,
    pub width: i64,
    pub height: i64,
    /// The project's own frame, so the caller can see what it is a proxy *of* without
    /// opening the file.
    pub declared_width: i64,
    pub declared_height: i64,
    /// The long-edge cap this tier applies, or `null` for true pixels (ADR-0046).
    pub long_edge_cap: Option<i64>,
    /// Whether the cap actually engaged. False for true pixels, and false for a project
    /// already inside the cap.
    pub proxied: bool,
    /// Whether this is a degradation from the default target — the fact the disclosure
    /// exists for.
    pub degraded: bool,
    /// The wall clock this tier was judged against, or `null` on the observational
    /// full-resolution arm, which has no ceiling to encode (ADR-0021).
    pub budget_ms: Option<u64>,
    /// The whole of it in one sentence, so a caller reading prose is told as plainly as a
    /// caller reading JSON.
    pub disclosure: String,
}

/// One rung of the ladder, as it went.
#[derive(Debug, Clone, Serialize)]
pub struct Attempt {
    pub tier: &'static str,
    pub width: i64,
    pub height: i64,
    pub wall_ms: u64,
    /// Whether it ran past the budget.
    pub missed: bool,
    /// Frames encoded before it was abandoned, where it was.
    pub frames: u64,
}

impl Attempt {
    /// This rung as a caller reads it: what it was called, and the frame it actually
    /// rasterized beside it.
    ///
    /// One spelling, because the degraded disclosure and the hard-fail refusal both name
    /// rungs and must not name the same one two ways — the failure this and [`rung_name`]
    /// were written together to close (ADR-0078).
    fn named(&self) -> String {
        format!("{} ({}x{})", self.tier, self.width, self.height)
    }
}

/// Preview the project at `path`.
///
/// `progress` is called as frames are encoded, once per rung; the CLI prints it to stderr.
/// The verb itself prints nothing.
pub fn preview(path: &FilePath, ask: &Ask, progress: &mut dyn FnMut(Progress)) -> Answer {
    preview_cancellable(path, ask, progress, None)
}

/// [`preview`], stopping where it stands and publishing nothing once `cancel` is set, on
/// `render`'s rule (ADR-0109).
pub fn preview_cancellable(
    path: &FilePath,
    ask: &Ask,
    progress: &mut dyn FnMut(Progress),
    cancel: Option<&render::Cancel>,
) -> Answer {
    let started = Instant::now();
    let project = Some(path.display().to_string());

    // The invocation is settled before the file is opened, exactly as `render` does it,
    // and through the same function: `--from 5 --to 2` is wrong whatever the document
    // says, and one spelling of that refusal serves both verbs.
    let asked = render::Ask {
        from: ask.from,
        to: ask.to,
        output: ask.output.clone(),
        // ADR-0104: `preview` runs the clobber pre-flight itself, below, on its own
        // narrower rule. It never borrows `render`'s.
        no_clobber: false,
    };
    let range = match render::request(&asked) {
        Ok(range) => range,
        Err(reason) => return refused(Report::rejected(TOOL, project, reason)),
    };

    // The identical check engine, and the refusal (ADR-0006, as `render`).
    let (document, mut report) =
        match crate::verbs::validate::checked(TOOL, path, None, Sidecar::default_path()) {
            Ok(checked) => checked,
            Err(report) => return refused(*report),
        };
    if report.exit_code() != ExitCode::Ok {
        return refused(report);
    }

    let header = match document.strict() {
        Ok(project) => project,
        Err(e) => {
            report.fail_internally(format!(
                "the project passed every check and still does not fit the model: {e}"
            ));
            return refused(report);
        }
    };
    let (width, height) = (header.frame.width, header.frame.height);
    let fps = header.fps;

    let (from, to, partial) = match range {
        Some((from, to)) => (from, to, true),
        None => match crate::exact::extent(&document) {
            Some(end) => (0, end, false),
            None => {
                return refused(Report::rejected(
                    TOOL,
                    Some(document.path().to_string()),
                    "there is nothing to preview: the project declares no `duration` and \
                     no element states a range",
                ));
            }
        },
    };

    let (Some(first), Some(last)) = (
        crate::exact::frame_at_or_after(from, fps),
        crate::exact::frame_before(to, fps),
    ) else {
        return refused(Report::rejected(
            TOOL,
            Some(document.path().to_string()),
            format!("no frame at {fps} fps falls inside {from}..{to} ms"),
        ));
    };
    if last.frame < first.frame {
        return refused(Report::rejected(
            TOOL,
            Some(document.path().to_string()),
            format!("no frame at {fps} fps falls inside {from}..{to} ms"),
        ));
    }
    let frames = (last.frame - first.frame + 1) as u64;

    let project_dir = crate::checks::project_dir(&document);
    let output = match destination(
        &document,
        &project_dir,
        header.output.as_deref(),
        ask,
        from,
        to,
    ) {
        Ok(output) => output,
        Err(reason) => {
            return refused(Report::rejected(
                TOOL,
                Some(document.path().to_string()),
                reason,
            ));
        }
    };

    // ADR-0104, and the narrower half of the rule. `preview` is exempt from ADR-0093's
    // no-promotion rule because a proxy is not a deliverable — but that exemption must not
    // become the bypass: `--output` is honoured here, and without this a preview could rename
    // itself over *another* project's finished cut, which is MONTAGENT-7 one verb across.
    //
    // Only `Foreign` refuses. An unattested file is left alone on purpose: a preview writes
    // no stamp, so the previous preview at this path is unattested by construction, and
    // refusing it would refuse the second preview of every project. A missing `ffmpeg` is
    // the next block's refusal to report (ADR-0091), not this one's to pre-empt.
    if let Ok(resolved) = tools::resolve()
        && let attest::Attestation::Foreign { project } = attest::of(
            &probe::ProcessRunner,
            &resolved,
            &output,
            FilePath::new(document.path()),
        )
    {
        return refused(Report::rejected(
            TOOL,
            Some(document.path().to_string()),
            format!(
                "{} was written by a different project ({project}), and a preview may never \
                 destroy a deliverable: it is a disposable artefact (ADR-0104)",
                output.display()
            ),
        ));
    }

    let ffmpeg = match tools::resolve() {
        Ok(tools) => tools.ffmpeg,
        Err(missing) => {
            missing.fail(&mut report);
            return refused(report);
        }
    };

    let background = header
        .background
        .as_ref()
        .and_then(crate::verbs::frame::rgba_of)
        .unwrap_or(Rgba::BLACK);

    // `None` on the escape hatch, where `Budget::FullResolutionPreview` has deliberately
    // no ceiling: that arm is observational and is neither enforced nor degraded.
    let budget = |rung: usize| match ask.full {
        true => None,
        false => Some(ask.clock.at(rung)),
    };

    let mut rung = 0usize;
    let mut tier = match ask.full {
        true => Tier::Native,
        false => Tier::Target,
    };
    let mut attempts: Vec<Attempt> = Vec::new();

    loop {
        let frame = tier.frame(width, height);

        // The 360p guard (ADR-0050), on every rung. It cannot fire as the ladder stands
        // and it is checked anyway: the next rung anyone adds meets it here rather than in
        // review, and a caller-specified proxy resolution — should one ever be admitted —
        // arrives at the same check.
        if frame.proxied
            && let Err(reason) = proxy::admit(frame.long_edge())
        {
            return refused(Report::rejected(
                TOOL,
                Some(document.path().to_string()),
                reason,
            ));
        }

        let span = Span {
            document: &document,
            established: crate::media::established::Established::of(&report),
            project_dir: &project_dir,
            ffmpeg: &ffmpeg,
            background,
            declared: (width, height),
            surface: Surface::rung(frame, width, height),
            fps,
            from,
            to,
            first: first.frame,
            last: last.frame,
            frames,
            output: &output,
            // ADR-0104: a proxy is not a deliverable and never attests to being one.
            stamp: None,
            deadline: budget(rung),
            cancel,
        };

        let attempt = Instant::now();
        match render::encode_span(&span, attempt, progress) {
            Ok(painted) => {
                attempts.push(Attempt {
                    tier: rung_name(tier, frame),
                    width: frame.width,
                    height: frame.height,
                    wall_ms: attempt.elapsed().as_millis() as u64,
                    missed: false,
                    frames,
                });
                let wall_ms = started.elapsed().as_millis() as u64;
                return Answer {
                    preview: Some(Preview {
                        // `partial` carries what it carries on `render` — whether a range
                        // was asked for. That a preview is never the deliverable is not
                        // that field's job here: it is true of every preview, range or
                        // none, and the tier disclosure is where it is said.
                        // ADR-0093 ruling 6 is about *the deliverable*, and a proxy is
                        // not one — ADR-0065 discloses it as a proxy and ADR-0021 keeps
                        // `render` the only verb that writes the declared `output`. So a
                        // preview publishes what it encoded, as it always has.
                        video: match painted.into_video(&span, partial, wall_ms) {
                            Ok(video) => video,
                            Err(reason) => {
                                report.fail_internally(reason);
                                return refused(report);
                            }
                        },
                        tier: Disclosure::of(tier, frame, (width, height), budget(rung), &attempts),
                        attempts,
                    }),
                    report,
                };
            }
            // ADR-0093: the mix's pre-flight refused, before any encoder was spawned. A
            // preview of a project whose audio cannot be mixed is not a cheaper picture of
            // it — the span decided this without touching the clock, so the ladder has
            // nothing cheaper to try and stops here.
            Err(Stop::Refused(findings)) => {
                for finding in findings {
                    report.push(finding);
                }
                return refused(report);
            }
            // ADR-0091: an unconfigured environment, not a broken project.
            Err(Stop::ToolMissing(missing)) => {
                missing.fail(&mut report);
                return refused(report);
            }
            Err(Stop::Internal(reason)) => {
                report.fail_internally(reason);
                return refused(report);
            }
            // ADR-0109: a cancelled preview does not try the next rung — nobody is waiting
            // for it.
            Err(Stop::Cancelled { done }) => {
                report.fail_cancelled(render::cancelled_after(TOOL, done, frames));
                return refused(report);
            }
            Err(Stop::Missed { elapsed, done }) => {
                attempts.push(Attempt {
                    tier: rung_name(tier, frame),
                    width: frame.width,
                    height: frame.height,
                    wall_ms: elapsed.as_millis() as u64,
                    missed: true,
                    frames: done,
                });
                let next = tier.next().map(|next| (next, next.frame(width, height)));
                match next {
                    // A rung that would rasterize the same frame is not a rung: the
                    // project is already inside the next cap, so there is nothing left to
                    // degrade and the ladder is over here rather than after a second
                    // identical attempt.
                    Some((next, smaller)) if smaller.long_edge() < frame.long_edge() => {
                        tier = next;
                        rung += 1;
                    }
                    _ => {
                        return refused(Report::rejected(
                            TOOL,
                            Some(document.path().to_string()),
                            gave_up(&attempts, budget(rung), frames, to - from),
                        ));
                    }
                }
            }
        }
    }
}

/// An answer with no preview: the report says why.
fn refused(report: Report) -> Answer {
    Answer {
        preview: None,
        report,
    }
}

/// What a rung is disclosed as: the tier's own name where its cap engaged, and `native`
/// where it did not.
///
/// One rule, used by the tier disclosure, the attempt trace and the refusal alike, so a
/// single invocation never names one frame two ways. ADR-0046 makes the disclosed field
/// report *"native, the 720p proxy tier, or — once #117 resolves — a floor-refuse"*, and a
/// project between the two caps degrades from true pixels to a real proxy (ADR-0078): its
/// first rung is the 720p *rung* and is not a 720p *frame*, so calling it one in the trace
/// would put a number in front of the caller that is not what was rasterized — the one
/// thing these fields exist to prevent.
fn rung_name(tier: Tier, frame: proxy::Frame) -> &'static str {
    match frame.proxied {
        true => tier.name(),
        false => Tier::Native.name(),
    }
}

impl Disclosure {
    fn of(
        tier: Tier,
        frame: proxy::Frame,
        declared: (i64, i64),
        budget: Option<Duration>,
        attempts: &[Attempt],
    ) -> Disclosure {
        let (declared_width, declared_height) = declared;
        let sentence = match (tier, frame.proxied) {
            (Tier::Degraded, _) => {
                // The rung above, named for what it actually rasterized. A project between
                // the two caps degrades from *true pixels* to a real proxy (ADR-0078), and
                // a sentence that said "720p ran to 3.0 s" there would be describing a
                // frame that was never drawn.
                let above = attempts.first();
                let ran = above
                    .map(Attempt::named)
                    .unwrap_or_else(|| "the rung above".to_string());
                let missed = above.map(|a| a.wall_ms).unwrap_or_default();
                format!(
                    "degraded: rendered at 540p ({}x{}), one rung below the 720p target, \
                     because {ran} ran to {:.1} s against the {:.1} s budget. This is \
                     the last rung — a 540p miss is a refusal, not a third tier (ADR-0065). \
                     The project's own frame is {declared_width}x{declared_height}; ask for \
                     true pixels with the full-resolution escape hatch.",
                    frame.width,
                    frame.height,
                    missed as f64 / 1000.0,
                    budget.unwrap_or(SCRUB_PREVIEW_LIMIT).as_secs_f64(),
                )
            }
            (Tier::Target, true) => format!(
                "the 720p proxy target ({}x{}), long edge capped at {} px from the \
                 project's {declared_width}x{declared_height} — not true pixels. Nothing \
                 degraded; this is the default every caller hits.",
                frame.width,
                frame.height,
                proxy::TARGET_LONG_EDGE,
            ),
            (Tier::Target, false) => format!(
                "true pixels ({}x{}): the project's frame is already inside the 720p cap of \
                 {} px, so no proxy applies. Nothing degraded.",
                frame.width,
                frame.height,
                proxy::TARGET_LONG_EDGE,
            ),
            (Tier::Native, _) => format!(
                "true pixels ({}x{}): the full-resolution escape hatch. This arm of the \
                 budget is observational — it is neither enforced nor degraded (ADR-0021).",
                frame.width, frame.height,
            ),
        };
        Disclosure {
            name: rung_name(tier, frame),
            width: frame.width,
            height: frame.height,
            declared_width,
            declared_height,
            long_edge_cap: match frame.proxied {
                true => tier.cap(),
                false => None,
            },
            proxied: frame.proxied,
            degraded: tier.is_degraded(),
            budget_ms: budget.map(|budget| budget.as_millis() as u64),
            disclosure: sentence,
        }
    }
}

/// The hard fail, in the caller's own terms: what was tried, what it cost, why there is no
/// rung left, and what the caller can actually do about it.
///
/// ADR-0050's argument for naming a floor applies to this one too: a bare failure is
/// indistinguishable from a render error or a missing asset, and a caller with no
/// explanation retries the identical call.
fn gave_up(
    attempts: &[Attempt],
    budget: Option<Duration>,
    frames: u64,
    duration_ms: i64,
) -> String {
    let budget = budget.unwrap_or(SCRUB_PREVIEW_LIMIT);
    let tried = attempts
        .iter()
        .map(|a| format!("{} ran to {:.1} s", a.named(), a.wall_ms as f64 / 1000.0))
        .collect::<Vec<_>>()
        .join(", then ");
    format!(
        "`preview` refused: {tried} — past the {:.1} s scrub budget, and 540p is where the \
         ladder gives up rather than degrade again (ADR-0065). One degrade step buys only \
         ~0.71 s at 8K, because what is left is decode and decode is driven by the source's \
         size, not the target's, so a miss this large cannot be rescued by a third tier. \
         Preview a shorter range than {frames} frames ({duration_ms} ms), or ask for true \
         pixels with the full-resolution escape hatch, which is observational rather than \
         budgeted. This is a wall-clock refusal, not the 360p legibility floor (ADR-0067).",
        budget.as_secs_f64(),
    )
}

/// Where the preview goes.
///
/// `out/<name>.preview.<from>-<to>.mp4` by default, and an explicit path is taken as given
/// — except the project's own `output`, which is refused whatever the range. `render`'s
/// story-56 rule is about a *partial* render landing on the deliverable; this is the same
/// rule with one more reason behind it, since every frame here may be proxy-scaled.
fn destination(
    document: &Loose,
    project_dir: &FilePath,
    declared: Option<&str>,
    ask: &Ask,
    from: i64,
    to: i64,
) -> Result<PathBuf, String> {
    let declared = declared.map(|output| project_dir.join(output));
    match &ask.output {
        Some(explicit) => {
            if let Some(declared) = &declared
                && render::same_path(explicit, declared)
            {
                return Err(format!(
                    "`--output {}` is the project's own `output`, and a preview may never \
                     land on the deliverable: it is a disposable artefact, and at the proxy \
                     target it is not even the project's frame size",
                    explicit.display()
                ));
            }
            Ok(explicit.clone())
        }
        None => {
            let name = declared
                .as_deref()
                .and_then(FilePath::file_stem)
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_else(|| {
                    let file = FilePath::new(document.path())
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "project".to_string());
                    file.trim_end_matches(".json")
                        .trim_end_matches(".montagent")
                        .to_string()
                });
            Ok(project_dir
                .join("out")
                .join(format!("{name}.preview.{from}-{to}.mp4")))
        }
    }
}
