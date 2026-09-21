//! `render`'s wall clock: **measured and recorded, judged against nothing** (#217).
//!
//! ## Why there is no assertion about speed here
//!
//! The budget this verb inherited is *"a 60 s video renders in under two minutes"*.
//! ADR-0021 records that pair as *"written for 1080×1920/30 and never re-derived"* after
//! ADR-0003 generalised the scope to a CapCut/Premiere-class editor where 4K is ordinary,
//! and it amended the preview half of the map's informal budget while leaving this half's
//! replacement **deferred**. An earlier reading of that gap divided the pair into a rate
//! and enforced it; #217 retires the reading along with the figure. A test that fails
//! against a number nothing measured is not a regression gate — it is the retired claim
//! wearing an assertion.
//!
//! **The gap, named:** `render` has no performance target, and this test does not invent
//! one. What it does is take the measurement an ADR would need in order to write one, and
//! record it in [`montaget_render::budget::RENDER_REFERENCES`] where a later reader can
//! find every number this project has taken. A target also needs numbers at the frame
//! sizes ADR-0003 put in scope, and the fixture is one frame size; that is the rest of the
//! gap and it is a measurement ticket, not an assertion.
//!
//! What *is* asserted: that the render succeeded, that it produced exactly the span the
//! recorded references were taken over, and that the harness consulted **no ceiling** —
//! [`Verdict::Observed`] rather than a pass against a limit. That last one is the guard
//! that keeps the retired figure from coming back as a constant.
//!
//! ## How the number is taken
//!
//! In the binary's crate and against the built executable, like `frame_budget.rs`, so it
//! includes process launch, the `ffmpeg` spawns for the encoder and the probes, and Skia's
//! first touch. One cold run per invocation: there is no threshold for a second run to
//! disambiguate, and the fixture is 65 s of video. Repetition is not thrown away, though —
//! each reading worth keeping is **appended** to the reference list rather than averaged
//! into the last one, because the run-to-run spread is itself something an ADR deriving a
//! target would need to see. Three #217 readings minutes apart spanned 18.64–19.78 s, and
//! the two ends of that are what the list carries. A fourth, taken while the machine was
//! compiling, came back at 22.10 s and is left out of the list and kept in the prose —
//! `RENDER_REFERENCES` says why.
//!
//! Nothing here enforces "cold" beyond the sidecar: an idle machine is the reader's job to
//! supply, and a number taken on a busy one is a number about the machine.
//!
//! **What "cold" includes.** An empty probe sidecar (ADR-0069), so the run pays for its
//! `ffprobe` spawns as `validate` would on a first turn.
//!
//! Release only. A debug number is not comparable to the recorded ones and is not worth
//! recording, so a debug run reports and returns.

use std::path::PathBuf;
use std::process::Command;

use montaget_render::budget::{Budget, RENDER_REFERENCE_OUTPUT_MS, Verdict, Work};

#[test]
fn the_whole_fixture_renders_and_its_wall_clock_is_recorded() {
    // Every recorded reference is from an optimised build, and CI runs the suite with
    // `--release`. A debug run would add a number that is comparable to none of them.
    if cfg!(debug_assertions) {
        eprintln!(
            "skipping: this is a debug build, and every recorded render measurement is \
             from the binary that ships. CI runs the suite with --release."
        );
        return;
    }
    if montaget_core::media::tools::resolve().is_err() {
        eprintln!("skipping: no ffmpeg/ffprobe on PATH");
        return;
    }

    let dir = scratch("render-budget");
    let output = dir.join("fixture.mp4");
    let project = fixture();
    // Through `Budget::measure`, which is the harness's own timing entry point (#189):
    // it takes the clock, judges, and hands back a `Measured` that prints itself. The
    // unit of work has to be named before the run rather than read off the answer, and
    // it is the span every recorded reference was taken over — which the assertion below
    // then confirms the render actually produced.
    let work = Work::span(RENDER_REFERENCE_OUTPUT_MS);
    let (out, measured) = Budget::Render.measure(work, || {
        Command::new(binary())
            .args([
                "render",
                project.to_str().expect("a fixture path"),
                "--output",
                output.to_str().expect("a scratch path"),
                "--json",
            ])
            .env(
                montaget_core::media::sidecar::CACHE_DIR_VAR,
                dir.join("cache"),
            )
            .output()
            .expect("run montaget render")
    });
    assert!(
        out.status.success(),
        "render did not answer:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("the machine-readable result on stdout");
    let duration_ms = json["render"]["duration_ms"].as_i64().expect("a span");
    assert_eq!(
        duration_ms, RENDER_REFERENCE_OUTPUT_MS,
        "the recorded references were taken over the fixture's declared duration; a number \
         over a different span is not comparable to them"
    );

    let Verdict::Observed {
        reference, drift, ..
    } = measured.verdict
    else {
        panic!(
            "`render` was judged against a ceiling: {measured}. #217 retired the only \
             number there was, and no measurement has replaced it."
        );
    };
    let reference = reference.expect("a recorded render reference to compare against");

    eprintln!(
        "render: {:.2?} cold for {duration_ms} ms of output ({:.2}× realtime), at \
         {conditions}",
        measured.elapsed,
        json["render"]["realtime"].as_f64().unwrap_or_default(),
        conditions = reference.conditions
    );
    match drift {
        Some(drift) => eprintln!(
            "  {drift:.2}× the recorded {} ms [{}] — recorded, not judged: `render` has no \
             target, and a target needs its own ADR with a measurement behind it",
            reference.elapsed_ms, reference.source
        ),
        None => eprintln!("  no comparable recorded measurement"),
    }
    if measured.verdict.is_notable() {
        eprintln!(
            "  ^ past the {}× drift ADR-0021 names as worth a human's attention. Still not \
             a failure: nothing here is entitled to say what \"too slow\" is.",
            montaget_render::budget::OBSERVATIONAL_DRIFT_FACTOR
        );
    }
}

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_montaget"))
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json")
        .canonicalize()
        .expect("the committed fixture")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("montaget-render-budget/{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}
