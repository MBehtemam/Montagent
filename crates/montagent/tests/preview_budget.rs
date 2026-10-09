//! `preview`'s wall clock: **the one span budget that is enforced** (#218, story 61).
//!
//! `render`'s number was retired and not replaced (#217, ADR-0072), and the
//! full-resolution arm is observational by design — the caller asked for true pixels and
//! accepted the cost. The scrub preview is the arm that keeps a ceiling: `<5 s`, at the
//! 720p proxy target, on `skia-safe` (ADR-0021, ADR-0065). This is where that ceiling is
//! asserted, through [`Budget::measure`] and [`Measured::assert_within_budget`] — #189's
//! own entry points, so the number is the one in
//! [`montagent_render::budget::SCRUB_PREVIEW_LIMIT`] rather than a literal repeated here.
//!
//! **Ten seconds of the committed fixture**, which is the span ADR-0021 states the budget
//! for — *"a 10 s preview under 5 seconds"* — taken over the fixture's own opening rather
//! than a synthetic project, so what is measured includes real decode, real fonts and real
//! text.
//!
//! **What a failure here means.** Not "the machine is slow": the budget sits against
//! measurements of 2.68 s (4K) and 3.78 s (8K) at this target, and the fixture is 1080×1920
//! — comfortably inside. Those two were measured with bilinear sampling and no mipmaps; the
//! proxy now reads a shrunk image through one mip level, which ADR-0186 adopted after #552's
//! trilinear read took this test past the budget on every CI leg. A miss is either a real
//! regression in the paint path or a claim in ADR-0065 that no longer holds, and both are
//! worth stopping for. Note also that the verb
//! itself would have degraded to 540p before it returned; this test asserts the tier it
//! actually came back at, so a pass that was bought by degrading is not read as a pass.
//!
//! Release only, as every other budget test: a debug number is comparable to none of the
//! recorded ones. The guarantee is `skia-safe`-specific and `tiny-skia` is explicitly not
//! certified at 8K/720p (ADR-0065), which is why this asserts nothing about that backend.
//!
//! **Best of three attempts (ADR-0189).** The ceiling is unchanged; what changed is how CI
//! enforces it. The preview sits at ~5.0 s on shared runners, and identical code passed on
//! one run and missed by ~0.1 s on the next — runner load only ever makes a run *slower*.
//! So the same cold preview is run up to [`ATTEMPTS`] times, each in its own fresh scratch
//! and cache directory, stopping at the first attempt that meets the budget at the 720p
//! target; the test fails only if none does, and then prints every attempt. A real
//! regression slows every attempt and still fails; noise slows only some.

use std::fmt;
use std::path::PathBuf;
use std::process::Command;

use montagent_render::budget::{Budget, Measured, Work};

/// The span ADR-0021's `<5 s` is written for.
const SCRUB_MS: i64 = 10_000;

/// How many cold runs of the same preview CI may take to see one meet the budget
/// (ADR-0189). The first that does ends the test, so the common case is one run.
const ATTEMPTS: usize = 3;

// ADR-0191: ignored on x86_64 Windows alone, where the preview sits at about 5 s on shared
// runners and still misses on the best of three. Every other leg enforces it.
#[cfg_attr(
    all(windows, target_arch = "x86_64"),
    ignore = "ADR-0191: misses the 5 s budget on shared x86_64 Windows runners; owner-disabled there"
)]
#[test]
fn a_ten_second_scrub_preview_of_the_fixture_stays_inside_the_budget() {
    if cfg!(debug_assertions) {
        eprintln!(
            "skipping: this is a debug build, and the scrub budget is stated for the binary \
             that ships. CI runs the suite with --release."
        );
        return;
    }
    if montagent_core::media::tools::resolve().is_err() {
        eprintln!("skipping: no ffmpeg/ffprobe on PATH");
        return;
    }

    let attempts = attempt_until_within_budget(ATTEMPTS, scrub_preview);
    let report = attempts
        .iter()
        .enumerate()
        .map(|(i, a)| format!("  attempt {}: {a}", i + 1))
        .collect::<Vec<_>>()
        .join("\n");
    eprintln!("scrub preview, up to {ATTEMPTS} cold attempts:\n{report}");

    let Some(best) = best_of(&attempts) else {
        // No attempt was inside the budget at the target tier. Judge the fastest, so the
        // panic names the same thing the single-run test did, with every attempt beside it.
        let fastest = attempts
            .iter()
            .min_by_key(|a| a.measured.elapsed)
            .expect("at least one attempt ran");
        // A pass bought by degrading is not a pass against this budget: the `<5 s` is
        // stated for the 720p target, and the tier is on the answer precisely so a reader
        // never has to assume which one it got (ADR-0021).
        assert_eq!(
            fastest.tier, "720p",
            "the scrub budget is the 720p target's; no attempt met it there, and the fastest \
             came back at another tier: {}\n{report}",
            fastest.disclosure
        );
        assert!(!fastest.degraded, "the fastest attempt degraded:\n{report}");
        panic!(
            "no attempt of {ATTEMPTS} met the scrub budget; the fastest: {}\n{report}",
            fastest.measured
        );
    };
    eprintln!(
        "scrub preview: {:.2?} cold for {SCRUB_MS} ms of output at {} (the 720p target, \
         from the fixture's 1080x1920)",
        best.measured.elapsed, best.size,
    );
    best.measured.assert_within_budget();
}

/// One cold run of the preview, and what the budget needs to know about it.
struct Attempt {
    measured: Measured,
    tier: String,
    degraded: bool,
    disclosure: String,
    size: String,
}

impl Attempt {
    /// Inside the ceiling **and** at the undegraded 720p target: both halves of the claim.
    fn meets_budget(&self) -> bool {
        self.tier == "720p" && !self.degraded && !self.measured.verdict.is_failure()
    }
}

impl fmt::Display for Attempt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:.2} s at {}{} ({})",
            self.measured.elapsed.as_secs_f64(),
            self.tier,
            if self.degraded { ", degraded" } else { "" },
            self.measured
        )
    }
}

/// The best attempt: the fastest that meets the budget at the target tier, if any did.
fn best_of(attempts: &[Attempt]) -> Option<&Attempt> {
    attempts
        .iter()
        .filter(|a| a.meets_budget())
        .min_by_key(|a| a.measured.elapsed)
}

/// Run up to `max` attempts, stopping at the first that meets the budget.
fn attempt_until_within_budget(max: usize, mut run: impl FnMut(usize) -> Attempt) -> Vec<Attempt> {
    let mut attempts = Vec::with_capacity(max);
    for i in 0..max {
        let attempt = run(i);
        let done = attempt.meets_budget();
        attempts.push(attempt);
        if done {
            break;
        }
    }
    attempts
}

/// One fresh, cold run: its own scratch directory and its own empty cache, as the
/// single-run test always had. What does not depend on the clock (that `preview`
/// answered, and over the 10 s span) is asserted on every attempt, never best-of.
fn scrub_preview(index: usize) -> Attempt {
    let dir = scratch(&format!("preview-budget-{}", index + 1));
    let output = dir.join("scrub.mp4");
    let project = fixture();
    let work = Work::span(SCRUB_MS);
    let (out, measured) = Budget::ScrubPreview.measure(work, || {
        Command::new(binary())
            .args([
                "preview",
                project.to_str().expect("a fixture path"),
                "--from",
                "0",
                "--to",
                &SCRUB_MS.to_string(),
                "--output",
                output.to_str().expect("a scratch path"),
                "--json",
            ])
            .env(
                montagent_core::media::sidecar::CACHE_DIR_VAR,
                dir.join("cache"),
            )
            .output()
            .expect("run montagent preview")
    });
    assert!(
        out.status.success(),
        "preview did not answer (attempt {}):\n{}{}",
        index + 1,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("the machine-readable result on stdout");
    assert_eq!(
        json["preview"]["duration_ms"].as_i64(),
        Some(SCRUB_MS),
        "the budget is stated for a 10 s scrub; a number over another span is not it"
    );
    let tier = &json["preview"]["tier"];
    Attempt {
        measured,
        tier: tier["name"]
            .as_str()
            .unwrap_or("<no tier name>")
            .to_string(),
        degraded: tier["degraded"]
            .as_bool()
            .expect("the tier says whether it degraded"),
        disclosure: tier["disclosure"].to_string(),
        size: format!("{}x{}", json["preview"]["width"], json["preview"]["height"]),
    }
}

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_montagent"))
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json")
        .canonicalize()
        .expect("the committed fixture")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("montagent-preview-budget/{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// The decision on synthetic timings, so it runs in every build: a preview slowed on every
/// attempt must still fail, and noise on some attempts must not.
mod best_of_three {
    use std::time::Duration;

    use super::*;

    fn at(secs: f64, tier: &str) -> Attempt {
        let elapsed = Duration::from_secs_f64(secs);
        let work = Work::span(SCRUB_MS);
        Attempt {
            measured: Measured {
                budget: Budget::ScrubPreview,
                work,
                elapsed,
                verdict: Budget::ScrubPreview.judge(work, elapsed),
            },
            tier: tier.to_string(),
            degraded: tier != "720p",
            disclosure: String::new(),
            size: String::new(),
        }
    }

    fn at_720p(secs: &[f64]) -> Vec<Attempt> {
        secs.iter().map(|&s| at(s, "720p")).collect()
    }

    /// Feed `timings` through the real loop; how many runs it took, and the best.
    fn run(timings: &[f64]) -> (usize, Option<Duration>) {
        let mut runs = 0;
        let attempts = attempt_until_within_budget(ATTEMPTS, |i| {
            runs += 1;
            at(timings[i], "720p")
        });
        (runs, best_of(&attempts).map(|a| a.measured.elapsed))
    }

    #[test]
    fn three_near_misses_fail() {
        assert!(best_of(&at_720p(&[5.4, 5.3, 5.2])).is_none());
        assert_eq!(run(&[5.4, 5.3, 5.2]), (3, None));
    }

    #[test]
    fn one_attempt_inside_the_budget_passes_and_is_the_best() {
        let attempts = at_720p(&[5.4, 4.7, 5.3]);
        let best = best_of(&attempts).map(|a| a.measured.elapsed);
        assert_eq!(best, Some(Duration::from_secs_f64(4.7)));
        assert_eq!(run(&[5.4, 4.7, 5.3]), (2, best), "stops at the first pass");
    }

    #[test]
    fn a_regression_that_slows_every_attempt_fails() {
        // #552's trilinear read took every leg past the ceiling on every run.
        assert_eq!(run(&[7.9, 7.6, 8.1]), (3, None));
        // A slowdown of only a few percent that every run pays is caught too.
        assert_eq!(run(&[5.05, 5.02, 5.08]), (3, None));
    }

    #[test]
    fn a_fast_attempt_bought_by_degrading_is_not_a_pass() {
        assert!(best_of(&[at(5.6, "720p"), at(3.1, "540p"), at(5.4, "720p")]).is_none());
    }

    #[test]
    fn the_common_case_costs_one_run_and_there_is_never_a_fourth() {
        assert_eq!(
            run(&[4.8, 5.4, 5.3]),
            (1, Some(Duration::from_secs_f64(4.8)))
        );
        assert_eq!(run(&[5.4, 5.3, 5.2, 4.0]), (3, None));
    }

    #[test]
    fn exactly_the_ceiling_is_inside_it() {
        assert!(best_of(&at_720p(&[5.0])).is_some());
    }
}
