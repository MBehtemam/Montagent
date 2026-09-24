//! Source alpha, from the probe's reading to the pixels the decode hands back.
//!
//! ADR-0089, and the two defects it closes: [#339](https://github.com/MBehtemam/Montagent/issues/339),
//! where `probe` told an agent a VP9-alpha source had no alpha, and
//! [#338](https://github.com/MBehtemam/Montagent/issues/338), where the decode returned
//! that source fully opaque and said nothing about it.
//!
//! **The fixtures are generated rather than committed.** Two encodes of the same generated
//! picture, one ProRes 4444 and one VP9-in-WebM, are a few hundred kilobytes of binary that
//! `ffmpeg` re-derives in under a second — and a committed `.webm` is a file no later
//! reader can check the *alpha* of without decoding it anyway. The generator asserts its
//! own output first, for the reason `docs/research/alpha-decode/FINDINGS.md` records: the
//! first fixture this work used carried no alpha at all, and read as "alpha lost" when the
//! pipeline was fine.
//!
//! Each test skips where the `ffmpeg` on this machine cannot build its fixture. ADR-0009
//! ships Montagent against *"an `ffmpeg` the user supplies"*, and libvpx is exactly the
//! optional piece that argument is about — a machine without it is the machine
//! [`Decoder::LibVpxVp9`]'s narrowness exists to protect, not a machine with a broken
//! checkout.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use montagent_core::media::probe::{AlphaSource, SourceAlpha};
use montagent_core::media::session::Session;
use montagent_core::media::{Source, tools};
use montagent_render::decode::{self, Decoder};

/// Red everywhere, opaque only inside a centred box. The alpha is the whole subject, so
/// the colour is deliberately constant: a test that read the *colour* could pass on a clip
/// whose alpha was gone.
const GENERATOR: &str = "color=c=red:s=320x240:r=25:d=2,format=rgba,\
                         geq=r='255':g='0':b='0':\
                         a='if(gt(X,80)*lt(X,240)*gt(Y,60)*lt(Y,180),255,0)'";

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

fn ffmpeg() -> PathBuf {
    tools::resolve().expect("an ffmpeg to probe with").ffmpeg
}

/// Encode the generator with `args`, or `None` where this `ffmpeg` cannot.
fn encode(dir: &Path, name: &str, args: &[&str]) -> Option<PathBuf> {
    let path = dir.join(name);
    let ok = Command::new(ffmpeg())
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            GENERATOR,
        ])
        .args(args)
        .arg(&path)
        .output()
        .ok()?
        .status
        .success();
    ok.then_some(path)
}

/// The alpha byte at a pixel of a decoded frame.
fn alpha_at(frame: &decode::DecodedFrame, x: u32, y: u32) -> u8 {
    frame.rgba[((y * frame.width + x) * 4 + 3) as usize]
}

/// Decode one frame a second in, the way the verbs do.
fn decoded(path: &Path, decoder: Decoder) -> decode::DecodedFrame {
    decode::frame_at(
        &ffmpeg(),
        &path.to_string_lossy(),
        decoder,
        1000,
        WIDTH,
        HEIGHT,
    )
    .expect("a frame")
}

/// What `probe` says about a source, through the session every verb uses.
fn probed_alpha(path: &Path) -> (Option<SourceAlpha>, Option<String>, Decoder) {
    let mut session = Session::open().expect("ffprobe");
    let source = Source::Local(path.to_path_buf());
    let outcome = session.probe(&source).expect("probe");
    let probe = outcome.probe().expect("a probed source").clone();
    let decoder = session.decoder_for(&source).expect("a decoder");
    (probe.alpha, probe.codec_name, decoder)
}

#[test]
fn the_generator_carries_the_alpha_the_rest_of_this_file_is_about() {
    // Asserted before any encode, because a fixture that stops carrying alpha must fail as
    // a fixture error and never as a finding about the decoder. This is the check whose
    // absence produced a false negative the first time this was measured.
    let dir = common::tempdir(line!());
    let Some(raw) = encode(
        &dir,
        "gen.mov",
        &[
            "-c:v",
            "prores_ks",
            "-profile:v",
            "4444",
            "-pix_fmt",
            "yuva444p10le",
        ],
    ) else {
        eprintln!("skipped: this ffmpeg cannot encode ProRes 4444");
        return;
    };
    let frame = decoded(&raw, Decoder::Auto);
    assert_eq!(
        alpha_at(&frame, 2, 2),
        0,
        "the corner is meant to be transparent"
    );
    assert_eq!(
        alpha_at(&frame, 160, 120),
        255,
        "the centre is meant to be opaque"
    );
}

#[test]
fn prores_4444_alpha_is_read_off_its_pixel_format_and_survives_the_default_decoder() {
    // The capability that already worked. It is asserted here so that a fix aimed at VP9
    // cannot quietly break it — which is the failure this whole file is shaped to catch.
    let dir = common::tempdir(line!());
    let Some(mov) = encode(
        &dir,
        "alpha.mov",
        &[
            "-c:v",
            "prores_ks",
            "-profile:v",
            "4444",
            "-pix_fmt",
            "yuva444p10le",
        ],
    ) else {
        eprintln!("skipped: this ffmpeg cannot encode ProRes 4444");
        return;
    };

    let (alpha, codec, decoder) = probed_alpha(&mov);
    assert_eq!(
        alpha,
        Some(SourceAlpha {
            carries: true,
            source: AlphaSource::PixelFormat
        })
    );
    assert_eq!(codec.as_deref(), Some("prores"));
    assert_eq!(
        decoder,
        Decoder::Auto,
        "nothing to recover: the alpha is in the pixels"
    );

    let frame = decoded(&mov, decoder);
    assert_eq!(alpha_at(&frame, 2, 2), 0);
    assert_eq!(alpha_at(&frame, 160, 120), 255);
}

#[test]
fn vp9_alpha_is_found_by_the_probe_and_recovered_by_the_decode() {
    // #339 and #338 end to end: the probe reports the alpha the pixel format cannot state,
    // and the decoder that reading selects hands the pixels back intact.
    let dir = common::tempdir(line!());
    let Some(webm) = encode(
        &dir,
        "alpha.webm",
        &[
            "-c:v",
            "libvpx-vp9",
            "-pix_fmt",
            "yuva420p",
            "-auto-alt-ref",
            "0",
        ],
    ) else {
        eprintln!("skipped: this ffmpeg has no libvpx-vp9 encoder");
        return;
    };

    let (alpha, codec, decoder) = probed_alpha(&webm);
    assert_eq!(
        alpha,
        Some(SourceAlpha {
            carries: true,
            source: AlphaSource::ContainerDeclaration
        }),
        "#339: ffprobe says pix_fmt=yuv420p, and the container says alpha_mode=1"
    );
    assert_eq!(codec.as_deref(), Some("vp9"));
    assert_eq!(decoder, Decoder::LibVpxVp9);

    let frame = decoded(&webm, decoder);
    assert_eq!(
        alpha_at(&frame, 2, 2),
        0,
        "#338: the corner is transparent in the source"
    );
    assert_eq!(alpha_at(&frame, 160, 120), 255);
}

#[test]
fn the_default_decoder_is_what_drops_vp9_alpha_and_that_is_why_the_probe_must_choose() {
    // The defect itself, pinned. If a future ffmpeg makes the native `vp9` decoder surface
    // the side stream, this fails — and that is the signal to revisit ADR-0089's rule
    // rather than a break. The assertion names that, so the next reader does not "fix" it
    // by deleting the conditional.
    let dir = common::tempdir(line!());
    let Some(webm) = encode(
        &dir,
        "alpha.webm",
        &[
            "-c:v",
            "libvpx-vp9",
            "-pix_fmt",
            "yuva420p",
            "-auto-alt-ref",
            "0",
        ],
    ) else {
        eprintln!("skipped: this ffmpeg has no libvpx-vp9 encoder");
        return;
    };

    let frame = decoded(&webm, Decoder::Auto);
    assert_eq!(
        alpha_at(&frame, 2, 2),
        255,
        "ffmpeg's native vp9 decoder still drops the alpha side stream. If this now reads \
         0, the native decoder has gained the capability and ADR-0089's conditional wants \
         revisiting — do not simply widen the assertion."
    );
}

#[test]
fn a_vp9_with_no_alpha_keeps_the_default_decoder() {
    // The narrowness ADR-0089 rests on: an ffmpeg without libvpx must keep decoding these.
    let dir = common::tempdir(line!());
    let Some(webm) = encode(
        &dir,
        "flat.webm",
        &["-c:v", "libvpx-vp9", "-pix_fmt", "yuv420p"],
    ) else {
        eprintln!("skipped: this ffmpeg has no libvpx-vp9 encoder");
        return;
    };

    let (alpha, codec, decoder) = probed_alpha(&webm);
    assert_eq!(alpha, Some(SourceAlpha::ABSENT));
    assert_eq!(codec.as_deref(), Some("vp9"));
    assert_eq!(decoder, Decoder::Auto);
}
