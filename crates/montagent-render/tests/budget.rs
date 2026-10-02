//! The budget harness, exercised against synthetic durations.
//!
//! These tests are about the harness rather than about Montagent's speed: that an
//! enforced miss fails, that the two deliberately unenforced arms cannot fail
//! whatever they measure, that a reference is only ever drawn from its own arm,
//! and that the numbers are the ones the ADRs state. The verb tickets supply the
//! real measurements through `Budget::measure` and record them here as
//! references.

use montagent_render::budget::{
    Budget, FRAME_LIMIT, FULL_RESOLUTION_PREVIEW_REFERENCES, Measured, OBSERVATIONAL_DRIFT_FACTOR,
    RENDER_REFERENCE_OUTPUT_MS, RENDER_REFERENCES, RENDER_TARGET, SCRUB_PREVIEW_LIMIT,
    SHIPPED_RASTERIZER, Verdict, Work, nearest_reference,
};
use std::time::Duration;

/// ADR-0021 states `frame` at `<500ms` cold, resolution-independent, and the
/// scrub preview at `<5s`. A budget nothing can regress against is not a budget,
/// so the numbers themselves are asserted.
#[test]
fn the_stated_numbers_are_the_numbers() {
    assert_eq!(FRAME_LIMIT, Duration::from_millis(500));
    assert_eq!(SCRUB_PREVIEW_LIMIT, Duration::from_secs(5));
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

/// `render`'s target is ADR-0142's: the benchmark project in at most 3 minutes,
/// median wall clock, on the dev's M1 Pro. One number, for one project — not a
/// rate, and not a CI ceiling.
#[test]
fn the_render_target_is_three_minutes_for_the_benchmark_project() {
    assert_eq!(RENDER_TARGET, Duration::from_secs(180));
}

/// Why `render` has no CI ceiling (#217, ADR-0142).
///
/// *"A 60 s video in under two minutes"* was written for 1080x1920/30 and never
/// re-derived after ADR-0003 generalised the scope to an editor where 4K is
/// ordinary, and ADR-0072 retired it. ADR-0142 states a target, [`RENDER_TARGET`],
/// for one project on one machine, which the shared CI runner cannot judge — so
/// `limit()` is `None`, meaning *not enforced in CI*, and a run of any length here
/// is an observation. The target is judged by the `#[ignore]`d `render_target.rs`.
#[test]
fn render_is_not_enforced_in_ci() {
    assert_eq!(Budget::Render.limit(Work::span(60_000)), None);
    assert_eq!(Budget::Render.limit(Work::span(10_000)), None);
    assert!(!Budget::Render.is_enforced());

    // Twenty minutes for 60 s of output is a number worth a human's attention and
    // is still not a CI failure: the target is a number for one project on one
    // machine, not a rate this harness may apply to any span.
    let verdict = Budget::Render.judge(Work::span(60_000), Duration::from_secs(1_200));
    assert!(!verdict.is_failure(), "{verdict:?}");
    assert!(matches!(verdict, Verdict::Observed { .. }));
}

/// Every arm that *is* enforced is flat rather than proportional, so no arm can
/// quietly become a rate again the way `render`'s did.
#[test]
fn no_enforced_ceiling_is_derived_from_the_length_of_the_output() {
    // The scrub preview is the only span arm left with a ceiling, and the proxy cap
    // rather than the span length is what bounds it.
    assert_eq!(
        Budget::ScrubPreview.limit(Work::span(1_000)),
        Budget::ScrubPreview.limit(Work::span(600_000)),
        "the scrub preview's ceiling moved with its span"
    );
    assert_eq!(Budget::Frame.limit(Work::Still), Some(FRAME_LIMIT));

    // And nothing else has one to move.
    for budget in [Budget::Render, Budget::FullResolutionPreview] {
        assert_eq!(budget.limit(Work::span(600_000)), None, "{}", budget.name());
    }
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

/// The two arms that state a number are the two ADR-0021 actually states.
#[test]
fn the_enforced_arms_are_the_two_with_a_stated_number() {
    for budget in [Budget::Frame, Budget::ScrubPreview] {
        assert!(budget.is_enforced(), "{} should be enforced", budget.name());
    }
    for budget in [Budget::Render, Budget::FullResolutionPreview] {
        assert!(
            !budget.is_enforced(),
            "{} has no ceiling CI enforces",
            budget.name()
        );
    }
}

/// ADR-0021's argument for the reference example over an invented ceiling is
/// that it *"flags a real 2x drift that a loose invented bound would still
/// pass"*. That is the behaviour being pinned here.
#[test]
fn an_observed_run_is_compared_against_the_nearest_reference() {
    let reference = skia_reference();

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
    let reference = skia_reference();

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

fn skia_reference() -> montagent_render::budget::Reference {
    *FULL_RESOLUTION_PREVIEW_REFERENCES
        .iter()
        .find(|r| r.source.starts_with("ADR-0021"))
        .expect("ADR-0021's measured example is recorded")
}

/// #34 measured the same span at 19.04 s through `skia-safe` and 30.0 s through
/// `tiny-skia`. Both are recorded, and only the shipped arm may be a baseline:
/// scoring a `skia-safe` run against the 1.58x-slower arm would let a real
/// regression come out as drift below 1.0.
#[test]
fn only_the_shipped_rasterizer_can_be_a_baseline() {
    let both_arms_recorded = FULL_RESOLUTION_PREVIEW_REFERENCES
        .iter()
        .any(|r| r.rasterizer != SHIPPED_RASTERIZER);
    assert!(
        both_arms_recorded,
        "the exit arm's number is worth keeping (ADR-0010 names `tiny-skia` as the exit)"
    );

    let chosen = nearest_reference(Budget::FullResolutionPreview, Work::span(10_000))
        .expect("a baseline exists for 10 s");
    assert_eq!(chosen.rasterizer, SHIPPED_RASTERIZER);
    assert_eq!(chosen.elapsed_ms, skia_reference().elapsed_ms);
}

/// The doc comment tells a later ticket to append to the reference list. That
/// instruction has to be safe: selection must not depend on array position.
///
/// `render`'s list is where this actually bites — two runs of the same fixture,
/// same span, 1.14x apart — so the tie must come out at the faster of them
/// whichever order they are written in.
#[test]
fn a_reference_is_chosen_by_its_values_rather_than_its_position() {
    for (budget, span) in [
        (Budget::FullResolutionPreview, 10_000),
        (Budget::Render, RENDER_REFERENCE_OUTPUT_MS),
    ] {
        let chosen = nearest_reference(budget, Work::span(span)).expect("a baseline exists");
        let same_distance: Vec<_> = budget
            .references()
            .iter()
            .filter(|r| r.rasterizer == SHIPPED_RASTERIZER && r.output_ms == chosen.output_ms)
            .collect();
        assert!(
            same_distance
                .iter()
                .all(|r| r.elapsed_ms >= chosen.elapsed_ms),
            "{}: a tie was broken on position — {same_distance:?} does not put {chosen:?} first",
            budget.name()
        );
    }

    // Not a vacuous pass: `render` has more than one number at that span.
    assert!(
        RENDER_REFERENCES.len() > 1,
        "the tie-break is only exercised while two runs share a span"
    );
}

#[test]
fn a_wildly_negative_span_does_not_overflow_the_search_for_a_baseline() {
    // The enforced arms clamp with `.max(0)`; this one does not, and the
    // subtraction used to find the nearest reference must survive it.
    for budget in [Budget::FullResolutionPreview, Budget::Render] {
        let verdict = budget.judge(Work::span(i64::MIN), Duration::from_secs(1));
        assert!(!verdict.is_failure(), "{}", budget.name());
    }
}

/// A span budget handed a still would otherwise get a 0 ms ceiling and report
/// every measurement as a regression. It is a caller bug, and it says so.
#[test]
fn pairing_a_span_budget_with_a_still_is_a_loud_caller_bug() {
    for budget in [
        Budget::Render,
        Budget::ScrubPreview,
        Budget::FullResolutionPreview,
    ] {
        assert!(budget.is_span());
        let mismatched = std::panic::catch_unwind(|| budget.limit(Work::Still));
        assert!(
            mismatched.is_err(),
            "{} accepted a still and would have judged it against a zero budget",
            budget.name()
        );
    }

    assert!(!Budget::Frame.is_span());
    let mismatched = std::panic::catch_unwind(|| Budget::Frame.limit(Work::span(10_000)));
    assert!(
        mismatched.is_err(),
        "`frame` accepted a span; ADR-0021 keeps the two halves of the budget apart"
    );
}

#[test]
fn the_recorded_references_are_measurements_rather_than_targets() {
    for budget in [Budget::FullResolutionPreview, Budget::Render] {
        let references = budget.references();
        assert!(
            !references.is_empty(),
            "{}: an observational arm with no reference cannot catch drift",
            budget.name()
        );
        for r in references {
            assert!(r.output_ms > 0, "{}: a reference needs a span", r.source);
            assert!(r.elapsed_ms > 0, "{}: a reference needs a number", r.source);
            assert!(
                !r.conditions.is_empty() && !r.source.is_empty() && !r.rasterizer.is_empty(),
                "a number with no stated rasterizer, conditions or provenance is not evidence"
            );
        }
    }
}

/// Every recorded render reference names the resolution and frame rate it was
/// taken at (#217: *"measured and recorded at a stated resolution"*). A wall clock
/// with no frame size behind it is not comparable to anything.
#[test]
fn every_render_reference_states_the_frame_it_was_measured_at() {
    for r in RENDER_REFERENCES {
        assert!(
            r.conditions.contains('x') && r.conditions.contains("fps"),
            "{}: conditions `{}` do not state a frame size and rate",
            r.source,
            r.conditions
        );
    }
}

/// The two observational arms answer different questions at different resolutions
/// — a 720p-capped proxy versus the declared frame — so one arm's number must
/// never become the other's baseline. Mixing them is the same mistake
/// [`SHIPPED_RASTERIZER`] exists to prevent, one axis over.
#[test]
fn an_arm_is_never_baselined_against_the_other_arms_numbers() {
    let render = nearest_reference(Budget::Render, Work::span(RENDER_REFERENCE_OUTPUT_MS))
        .expect("a render baseline");
    assert!(
        RENDER_REFERENCES.contains(&render),
        "{render:?} did not come from `render`'s own list"
    );

    let preview = nearest_reference(
        Budget::FullResolutionPreview,
        Work::span(RENDER_REFERENCE_OUTPUT_MS),
    )
    .expect("a preview baseline");
    assert!(
        FULL_RESOLUTION_PREVIEW_REFERENCES.contains(&preview),
        "{preview:?} did not come from the preview's own list"
    );
    assert_ne!(render, preview);
}

/// An enforced arm has no reference list: it judges against its stated number,
/// and a reference it never consults would be a number with no reader.
#[test]
fn an_enforced_arm_carries_no_references() {
    for budget in [Budget::Frame, Budget::ScrubPreview] {
        assert!(budget.references().is_empty(), "{}", budget.name());
        assert_eq!(nearest_reference(budget, Work::span(10_000)), None);
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

/// The numbers [`RENDER_REFERENCES`]' prose states, re-derived from the entries.
///
/// `docs/agents/domain.md` wants a numeric claim to come with something that
/// "exits non-zero the moment it stops reproducing". These particular numbers
/// cannot be re-measured from a unit test — they are records of runs on stated
/// hardware — but the arithmetic the doc comment does over them can, and that is
/// the half that rots when someone appends a reading and leaves the prose alone.
#[test]
fn the_recorded_spread_is_the_one_the_prose_states() {
    let ms = |source_prefix: &str| -> Vec<u64> {
        RENDER_REFERENCES
            .iter()
            .filter(|r| r.source.starts_with(source_prefix))
            .map(|r| r.elapsed_ms)
            .collect()
    };

    // "Its two entries below are that range's ends": 19.78 s and 18.64 s.
    let mut own = ms("#217");
    own.sort_unstable();
    assert_eq!(own, vec![18_640, 19_780], "#217's recorded ends moved");
    assert_eq!(ms("#215"), vec![17_300], "#215's reading moved");

    // "the list spans 1.14x end to end".
    let slowest = RENDER_REFERENCES
        .iter()
        .map(|r| r.elapsed_ms)
        .max()
        .unwrap();
    let fastest = RENDER_REFERENCES
        .iter()
        .map(|r| r.elapsed_ms)
        .min()
        .unwrap();
    let spread = slowest as f64 / fastest as f64;
    assert!(
        (spread - 1.14).abs() < 0.005,
        "the prose says 1.14x end to end; the entries say {spread:.3}x"
    );

    // "baselines against the fastest" — the claim the paragraph makes about which
    // of these a measurement is actually scored against.
    let chosen = nearest_reference(Budget::Render, Work::span(RENDER_REFERENCE_OUTPUT_MS))
        .expect("a render baseline");
    assert_eq!(chosen.elapsed_ms, fastest);
}
