//! `measure`'s keyed-alpha coverage series (ADR-0088, #342), against the forcing case the
//! ADR was decided on: `docs/research/chroma-key/green-screen-trex.mp4`, 1920x1080, 24 fps,
//! 140 frames.
//!
//! **This is the regression guard the ADR's evidence asks for**, and it asserts both
//! directions throughout, exactly as `chroma_key_scan.sh` does beside the clip: the
//! tolerances that must key and the values that must not, so a change making the keyer more
//! permissive fails here as loudly as one breaking it.
//!
//! # One of the ADR's rows does not transfer, and this file says so rather than eliding it
//!
//! ADR-0088's evidence table records `ffmpeg`'s `chromakey` keying **nothing** at
//! `tolerance: 0.01`. Montagent's keyer keys the screen there, and the difference is not a
//! defect in either:
//!
//! - `ffmpeg` compares the **stream's** subsampled, limited-range 8-bit `(U, V)` against a
//!   key converted from RGB with the CCIR coefficients. The two sit in different ranges, so
//!   an exactly-uniform screen still lands a unit or two off its own key and a tolerance of
//!   0.01 cannot reach it.
//! - Montagent converts the *decoded pixel* and the *declared key* through the **same**
//!   function, so a screen that really is `#00CD00` is at distance 0 from `#00CD00` — which
//!   is the answer the document asks for, and the fixture's screen really is exactly that
//!   at all 25 points `chroma_key_scan.sh` samples.
//!
//! The cliff itself is real and is asserted where it can be: the renderer's own
//! `the_tolerance_plateau_adr_0088_measured_is_flat_here_too` puts a *noisy* screen pixel
//! below `tolerance: 0.01` and above `0.05`. What does not transfer is one clip's arithmetic
//! artefact, not the shape of the parameter.
//!
//! # Why the whole-clip pass is run once
//!
//! A 140-frame 1080p series costs ~10 s: the rasterizer paints and reads back 2 MP a frame,
//! and that is the cost of the coverage being the render's own rather than a second keyer's
//! estimate of it. So the drift claim — the one that genuinely needs every frame — is made
//! once, and the tolerance claims, which are about numbers rather than about the clip's
//! length, are made over a short sub-range of the same element.

use serde_json::Value;

mod common;
use common::{canonical, has_ffprobe, tempdir, write_project};

/// ADR-0088's forcing case, as committed.
fn clip() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/research/chroma-key/green-screen-trex.mp4")
        .canonicalize()
        .expect("the committed forcing case")
}

/// The clip's own numbers, from its container: 140 frames of 24 fps.
const FRAMES: usize = 140;
const SPAN_MS: i64 = 5833;

/// One `video` element over the fixture, keyed at `tolerance`, running `end` ms.
fn element(end: i64, tolerance: f64) -> String {
    format!(
        r##"{{"id":"trex","type":"video","start":0,"end":{end},"source":"{source}","source_start":0,"source_end":{SPAN_MS},"x":0,"y":0,"origin":"top-left","width":1920,"height":1080,"effects":[{{"name":"chroma","color":"#00CD00","tolerance":{tolerance},"softness":0.0,"spill":0.0}}]}}"##,
        source = json_escaped(&clip()),
    )
}

/// A path as it goes between the quotes of a JSON string.
///
/// On Windows the canonical path is `\\?\C:\...`, and its backslashes written raw into
/// the project are invalid escapes, so the file would not parse at all.
fn json_escaped(path: &std::path::Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}

/// The coverage series `measure` reports for that element.
#[track_caller]
fn series(end: i64, tolerance: f64) -> Vec<Value> {
    let element = element(end, tolerance);
    let body = canonical(&format!(
        r##"{{"frame":{{"width":1920,"height":1080}},"fps":24,"tracks":[{{"name":"t","layer":10,"elements":[{element}]}}]}}"##
    ));
    let dir = tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &body);

    let answer = montagent_core::verbs::measure::measure(
        &path,
        &montagent_core::verbs::measure::Ask {
            element: Some(serde_json::from_str(&element).expect("the element is JSON")),
            ..Default::default()
        },
    );
    let json = answer.to_json();
    assert_eq!(
        json["measure"]["mode"],
        "coverage",
        "a keyed element measures as coverage: {}",
        serde_json::to_string_pretty(&json).unwrap_or_default()
    );
    json["measure"]["frames"]
        .as_array()
        .expect("a series")
        .clone()
}

/// Every frame's transparent fraction — what the key took out.
fn transparent(frames: &[Value]) -> Vec<f64> {
    frames
        .iter()
        .map(|frame| frame["transparent"].as_f64().expect("a fraction"))
        .collect()
}

#[test]
fn the_series_is_one_sample_per_frame_and_holds_across_every_one_of_them() {
    if !has_ffprobe() {
        return;
    }
    // ADR-0088's fifth measured claim: one static tolerance holds for the whole clip —
    // "zero border-band contamination across all 140 frames at tolerance 0.10". The
    // coverage series is the instrument that *finds* a drift, so the fixture's own
    // non-drift is what it has to report: a flat series, over every frame the clip has.
    let frames = series(SPAN_MS, 0.10);
    assert_eq!(frames.len(), FRAMES, "one sample per frame of the clip");
    assert_eq!(frames[0]["frame"], 0);
    assert_eq!(frames[0]["at"], 0);
    assert_eq!(
        frames[FRAMES - 1]["at"],
        (FRAMES as i64 - 1) * 1000 / 24,
        "the last sample sits on the project's own grid"
    );

    let keyed = transparent(&frames);
    let low = keyed.iter().cloned().fold(f64::MAX, f64::min);
    let high = keyed.iter().cloned().fold(0.0, f64::max);
    assert!(
        (0.85..0.95).contains(&low) && (0.85..0.95).contains(&high),
        "the screen is 85-95% of the frame throughout: {low}..{high}"
    );
    assert!(
        high - low < 0.03,
        "the series is flat — a step is the drift signal ADR-0088 defines, and this clip is \
         CGI and uniform by construction: {low}..{high}"
    );

    // The other direction, and the one a keyer that simply erased everything would fail:
    // the T-rex is still there on every frame.
    for frame in &frames {
        let opaque = frame["opaque"].as_f64().expect("a fraction");
        assert!(opaque > 0.05, "the subject survived: {opaque}");
    }
}

#[test]
fn the_tolerance_plateau_is_flat_and_the_identity_keys_nothing() {
    if !has_ffprobe() {
        return;
    }
    // ADR-0088's measured plateau: 0.05 through 0.30 "all key identically", six times wider
    // than the precision anyone would tune to. That width is the whole reason one static
    // tolerance is usable on footage the author cannot re-light.
    let mut plateau = Vec::new();
    for tolerance in [0.05, 0.10, 0.20, 0.30] {
        let keyed = transparent(&series(500, tolerance));
        assert!(!keyed.is_empty(), "the series has frames");
        plateau.push((tolerance, keyed[0]));
    }
    let low = plateau.iter().map(|(_, at)| *at).fold(f64::MAX, f64::min);
    let high = plateau.iter().map(|(_, at)| *at).fold(0.0, f64::max);
    assert!(
        high - low < 0.01,
        "0.05-0.30 key within a point of each other: {plateau:?}"
    );

    // The identity ADR-0088 specifies, which is the direction that matters most: at
    // `tolerance: 0` the member is a no-op and the element is exactly what it was.
    let frames = series(500, 0.0);
    for frame in &frames {
        assert_eq!(
            frame["transparent"].as_f64(),
            Some(0.0),
            "`tolerance: 0` keys nothing: {frame}"
        );
        assert_eq!(frame["opaque"].as_f64(), Some(1.0), "{frame}");
    }
}

#[test]
fn softness_opens_a_partial_band_that_a_hard_matte_does_not_have() {
    if !has_ffprobe() {
        return;
    }
    // ADR-0088 lists `softness` as untested — "specified but untested; the forcing case
    // produced a binary matte". It is testable on the forcing case in one direction, which
    // is the one the identity value claims: a hard matte has no partial pixels beyond the
    // resampler's own edges, and opening the band produces some.
    let hard = series(500, 0.10);
    let partial = |frames: &[Value]| frames[0]["partial"].as_f64().expect("a fraction");

    let element = element(500, 0.10).replace(r#""softness":0.0"#, r#""softness":0.3"#);
    let body = canonical(&format!(
        r##"{{"frame":{{"width":1920,"height":1080}},"fps":24,"tracks":[{{"name":"t","layer":10,"elements":[{element}]}}]}}"##
    ));
    let dir = tempdir(line!());
    let path = write_project(&dir, "p.montagent.json", &body);
    let answer = montagent_core::verbs::measure::measure(
        &path,
        &montagent_core::verbs::measure::Ask {
            element: Some(serde_json::from_str(&element).expect("the element is JSON")),
            ..Default::default()
        },
    );
    let soft = answer.to_json()["measure"]["frames"]
        .as_array()
        .expect("a series")
        .clone();

    assert!(
        partial(&soft) > partial(&hard),
        "the blend band is wider than the hard matte's edge: {} vs {}",
        partial(&soft),
        partial(&hard)
    );
}

#[test]
fn the_three_fractions_are_one_whole() {
    if !has_ffprobe() {
        return;
    }
    // Every pixel is in exactly one of the three buckets, which is what makes the partial
    // band readable as "how much of this matte is neither in nor out" rather than as a
    // residue. Asserted rather than assumed, because the third is derived from the other
    // two and a bug there would look like a plausible number.
    for frame in series(500, 0.10) {
        let sum = ["opaque", "partial", "transparent"]
            .iter()
            .map(|key| frame[*key].as_f64().expect("a fraction"))
            .sum::<f64>();
        assert!((sum - 1.0).abs() < 1e-9, "{frame} sums to {sum}");
    }
}

// ---------------------------------------------------------------------------
// R-CHROMA-ON-ALPHA-SOURCE — the one finding that needs the disk
// ---------------------------------------------------------------------------

/// A two-second ProRes 4444 clip whose pixels carry a real alpha channel.
///
/// Generated rather than committed, and the recipe is `alpha_decode_scan.sh`'s own
/// (`docs/research/alpha-decode/`): that script measured ProRes 4444 as the container in
/// which alpha survives the decode path, which is exactly the property this check reads.
/// A second committed binary for a two-line `lavfi` expression would be a fixture to keep
/// in step with a script that already produces it.
fn already_keyed(into: &std::path::Path) -> std::path::PathBuf {
    let tools = montagent_core::media::tools::resolve().expect("ffmpeg");
    let path = into.join("already-keyed.mov");
    let generated = "color=c=red:s=320x240:r=25:d=2,format=rgba,\
                     geq=r='255':g='0':b='0':a='if(gt(X,80)*lt(X,240),255,0)'";
    let out = std::process::Command::new(tools.ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
        ])
        .arg(generated)
        .args([
            "-c:v",
            "prores_ks",
            "-profile:v",
            "4444",
            "-pix_fmt",
            "yuva444p10le",
        ])
        .arg(&path)
        .output()
        .expect("run ffmpeg");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    path
}

#[test]
fn keying_a_source_that_already_carries_alpha_is_reported_and_keying_one_that_does_not_is_not() {
    if !has_ffprobe() {
        return;
    }
    // "Keying an already-keyed asset is almost always a mistake, and it costs one `ffprobe`
    // the pipeline runs anyway" (ADR-0088). Both directions in one test, because the whole
    // value of the check is the distinction: the fixture's own h264 clip carries no alpha
    // and must stay silent.
    let dir = tempdir(line!());
    let keyed = already_keyed(&dir);

    for (source, expected) in [(json_escaped(&keyed), 1), (json_escaped(&clip()), 0)] {
        let element = format!(
            r##"{{"id":"a","type":"video","start":0,"end":1000,"source":"{source}","source_start":0,"source_end":1000,"x":0,"y":0,"origin":"top-left","width":320,"height":240,"fit":"literal","effects":[{{"name":"chroma","color":"#00CD00","tolerance":0.1,"softness":0.0,"spill":0.0}}]}}"##
        );
        let body = canonical(&format!(
            r##"{{"frame":{{"width":320,"height":240}},"fps":25,"tracks":[{{"name":"t","layer":10,"elements":[{element}]}}]}}"##
        ));
        let path = write_project(&dir, "alpha.montagent.json", &body);

        let report = montagent_core::verbs::validate::validate(&path);
        let found = report
            .findings
            .iter()
            .filter(|finding| finding.code == "R-CHROMA-ON-ALPHA-SOURCE")
            .count();
        assert_eq!(found, expected, "for {source}: {:?}", report.findings);
    }
}
