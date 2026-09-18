//! The budget harness, exercised against synthetic durations.
//!
//! No verb exists to time yet, so these tests are about the harness rather than
//! about Montaget's speed: that an enforced miss fails, that the one deliberately
//! unenforced arm cannot fail, that `render`'s ceiling scales with its output, and
//! that the numbers are the ones the ADRs state. The verb tickets supply the real
//! measurements through `Budget::measure`.

use montaget_render::budget::{
    Budget, FRAME_LIMIT, FULL_RESOLUTION_PREVIEW_REFERENCES, Measured, OBSERVATIONAL_DRIFT_FACTOR,
    RENDER_MS_PER_OUTPUT_SECOND, SCRUB_PREVIEW_LIMIT, Verdict, Work,
};
use std::time::Duration;

/// ADR-0021 states `frame` at `<500ms` cold, resolution-independent, and the
/// scrub preview at `<5s`. A budget nothing can regress against is not a budget,
/// so the numbers themselves are asserted.
#[test]
fn the_stated_numbers_are_the_numbers() {
    assert_eq!(FRAME_LIMIT, Duration::from_millis(500));
    assert_eq!(SCRUB_PREVIEW_LIMIT, Duration::from_secs(5));
    assert_eq!(RENDER_MS_PER_OUTPUT_SECOND, 2_000);
}

#[test]
fn frame_is_flat_and_resolution_independent() {
    // `Work::Still` is the only shape `frame` has: ADR-0021 states the number
    // independent of resolution, so there is no axis for it to vary along.
    assert_eq!(Budget::Frame.limit(Work::Still), Some(FRAME_LIMIT));
}

#[test]
fn frame_under_budget_passes_and_over_budget_fails() {
    let ok = Budget::Frame.judge(Work::Still, Duration::from_millis(270));
    assert!(!ok.is_failure(), "0.27 s is the worst cold value measured");

    let bad = Budget::Frame.judge(Work::Still, Duration::from_millis(501));
    assert!(bad.is_failure());
    assert!(matches!(bad, Verdict::Exceeded { .. }));
}

#[test]
fn the_scrub_preview_ceiling_is_flat_across_span_lengths() {
    // ADR-0021's `<5s` is an absolute wall-clock number at the proxy target, not
    // a rate: the proxy cap is what bounds the cost, so a longer scrub does not
    // buy a longer budget.
    for output_ms in [1_000, 10_000, 60_000] {
        assert_eq!(
            Budget::ScrubPreview.limit(Work::span(output_ms)),
            Some(SCRUB_PREVIEW_LIMIT),
            "scrub preview budget moved at {output_ms} ms of output"
        );
    }
}

#[test]
fn render_scales_with_the_length_of_its_output() {
    // The original budget, as one point: 60 s of video in under two minutes.
    assert_eq!(
        Budget::Render.limit(Work::span(60_000)),
        Some(Duration::from_secs(120))
    );
    assert_eq!(
        Budget::Render.limit(Work::span(10_000)),
        Some(Duration::from_secs(20))
    );
    // A zero-length span has a zero budget rather than an infinite one.
    assert_eq!(
        Budget::Render.limit(Work::span(0)),
        Some(Duration::from_secs(0))
    );
}

#[test]
fn a_negative_span_does_not_wrap_into_an_enormous_budget() {
    // Times are i64 milliseconds throughout (#168) and nothing here gets to
    // assume a caller's arithmetic was right.
    assert_eq!(
        Budget::Render.limit(Work::span(-10_000)),
        Some(Duration::from_secs(0))
    );
}

/// The load-bearing asymmetry in ADR-0021: the caller who asked for true pixels
/// accepted the cost, so there is no promise for a ceiling to encode. This arm
/// must be incapable of failing, however slow it is.
#[test]
fn the_full_resolution_preview_is_observational_and_cannot_fail() {
    assert!(!Budget::FullResolutionPreview.is_enforced());
    assert_eq!(
        Budget::FullResolutionPreview.limit(Work::span(10_000)),
        None
    );

    let verdict = Budget::FullResolutionPreview.judge(Work::span(10_000), Duration::from_secs(600));
    assert!(
        !verdict.is_failure(),
        "ten minutes must still not be a failure"
    );
    assert!(matches!(verdict, Verdict::Observed { .. }));
}

#[test]
fn every_other_arm_is_enforced() {
    for budget in [Budget::Frame, Budget::ScrubPreview, Budget::Render] {
        assert!(budget.is_enforced(), "{} should be enforced", budget.name());
    }
}

/// ADR-0021's argument for the reference example over an invented ceiling is
/// that it *"flags a real 2x drift that a loose invented bound would still
/// pass"*. That is the behaviour being pinned here.
#[test]
fn an_observed_run_is_compared_against_the_nearest_reference() {
    let reference = FULL_RESOLUTION_PREVIEW_REFERENCES
        .iter()
        .find(|r| r.source.starts_with("ADR-0021"))
        .expect("ADR-0021's measured example is recorded");

    let at_reference = Budget::FullResolutionPreview.judge(
        Work::span(reference.output_ms),
        Duration::from_millis(reference.elapsed_ms),
    );
    match at_reference {
        Verdict::Observed { drift: Some(d), .. } => {
            assert!((d - 1.0).abs() < 1e-9, "drift at the reference was {d}")
        }
        other => panic!("expected an observation, got {other:?}"),
    }
    assert!(!at_reference.is_notable());

    let doubled = Budget::FullResolutionPreview.judge(
        Work::span(reference.output_ms),
        Duration::from_millis(reference.elapsed_ms * 2),
    );
    assert!(
        doubled.is_notable(),
        "a {OBSERVATIONAL_DRIFT_FACTOR}x drift is the example ADR-0021 names"
    );
    assert!(!doubled.is_failure(), "notable is still not a failure");
}

#[test]
fn drift_is_scaled_to_the_length_of_the_span_being_compared() {
    let reference = FULL_RESOLUTION_PREVIEW_REFERENCES
        .iter()
        .find(|r| r.source.starts_with("ADR-0021"))
        .expect("ADR-0021's measured example is recorded");

    // Half the output length for half the wall clock is the same speed, not a
    // 2x improvement.
    let half = Budget::FullResolutionPreview.judge(
        Work::span(reference.output_ms / 2),
        Duration::from_millis(reference.elapsed_ms / 2),
    );
    match half {
        Verdict::Observed { drift: Some(d), .. } => {
            assert!((d - 1.0).abs() < 1e-6, "drift over half a span was {d}")
        }
        other => panic!("expected an observation, got {other:?}"),
    }
}

#[test]
fn the_recorded_references_are_measurements_rather_than_targets() {
    assert!(
        !FULL_RESOLUTION_PREVIEW_REFERENCES.is_empty(),
        "an observational arm with no reference cannot catch drift"
    );
    for r in FULL_RESOLUTION_PREVIEW_REFERENCES {
        assert!(r.output_ms > 0, "{}: a reference needs a span", r.source);
        assert!(r.elapsed_ms > 0, "{}: a reference needs a number", r.source);
        assert!(
            !r.conditions.is_empty() && !r.source.is_empty(),
            "a number with no stated conditions or provenance is not evidence"
        );
    }
}

#[test]
fn measure_times_the_closure_and_returns_its_value() {
    let (value, measured) = Budget::Frame.measure(Work::Still, || 40 + 2);
    assert_eq!(value, 42);
    assert_eq!(measured.budget, Budget::Frame);
    assert_eq!(measured.work, Work::Still);
    assert_eq!(measured.elapsed, measured.verdict.elapsed());
    measured.assert_within_budget();
}

/// The failure message is the whole point of the harness: a CI failure must be
/// readable without re-running anything.
#[test]
fn a_miss_names_the_budget_the_ceiling_and_the_overshoot() {
    let elapsed = Duration::from_millis(750);
    let measured = Measured {
        budget: Budget::Frame,
        work: Work::Still,
        elapsed,
        verdict: Budget::Frame.judge(Work::Still, elapsed),
    };
    let rendered = measured.to_string();
    assert!(rendered.contains("frame"), "{rendered}");
    assert!(rendered.contains("750.0 ms"), "{rendered}");
    assert!(rendered.contains("500.0 ms"), "{rendered}");
    assert!(rendered.contains("over by 250.0 ms"), "{rendered}");

    let panicked = std::panic::catch_unwind(|| measured.assert_within_budget());
    assert!(panicked.is_err(), "an enforced miss must fail the test");
}

#[test]
fn an_observation_reports_its_reference_rather_than_a_ceiling() {
    let elapsed = Duration::from_millis(19_040);
    let measured = Measured {
        budget: Budget::FullResolutionPreview,
        work: Work::span(10_000),
        elapsed,
        verdict: Budget::FullResolutionPreview.judge(Work::span(10_000), elapsed),
    };
    let rendered = measured.to_string();
    assert!(rendered.contains("unenforced"), "{rendered}");
    assert!(rendered.contains("1.00x"), "{rendered}");
    assert!(rendered.contains("2160x3840"), "{rendered}");
    measured.assert_within_budget();
}
