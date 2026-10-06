//! The `blur`/`shadow` layer hint keeps every byte (#652).
//!
//! The hint stops a filter layer covering the whole frame, and it is only worth having
//! because it changes no pixel. So the claim here is not about what a frame looks like but
//! that it is **the same frame**: one project of edge cases, every instant painted with the
//! hint and without it, every byte compared.
//!
//! The cases are #649's (`docs/research/filter-bound/harness/gen_edge.py` on
//! `research/649-filter-bound`): offset shadows, the frame's edges, scale 0.4–3, animated and
//! anisotropic scale, rotation, text that overflows its box and wide strokes, effect chains,
//! an image, and σ < 2, where Skia blurs with a Gaussian pass instead of three boxes. Each is
//! sampled at four instants while its `x` drifts by a fraction of a pixel, because a hint
//! that moves the layer's origin flips coverage on only some edge pixels at only some
//! offsets. To those it adds what the research left open — flips, and a σ past the
//! precondition, which must take the unbounded path and so is the same frame trivially.

use std::path::Path;

use montagent_core::report::ExitCode;
use montagent_core::verbs::frame::{Ask, frame};
use montagent_render::canvas::bound_filter_layers;
use serde_json::{Value, json};

mod common;
use common::{canonical, tempdir, write_project};

/// The trailer's directory, for the font and the photo the cases draw.
fn trailer() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/benchmark/spy-trailer")
}

fn blur(radius: f64) -> Value {
    json!({"name": "blur", "radius": radius})
}

fn shadow(dx: f64, dy: f64, radius: f64) -> Value {
    json!({"name": "shadow", "dx": dx, "dy": dy, "radius": radius, "color": "#FF9F2E", "opacity": 0.8})
}

fn glow(radius: f64, colour: &str, opacity: f64) -> Value {
    json!({"name": "shadow", "dx": 0, "dy": 0, "radius": radius, "color": colour, "opacity": opacity})
}

fn inverted(shape: &str) -> Value {
    json!({"name": "mask", "shape": shape, "invert": true})
}

fn inverted_rect(x: i64, y: i64, width: i64, height: i64, radius: i64) -> Value {
    json!({"name": "mask", "shape": "rect", "x": x, "y": y, "width": width, "height": height,
           "radius": radius, "invert": true})
}

/// A bare mask (the element's own rect) with `feather`, which may be a keyframe list.
fn feathered(shape: &str, invert: bool, feather: Value) -> Value {
    json!({"name": "mask", "shape": shape, "invert": invert, "feather": feather})
}

/// `SPY` in Cinzel Bold at 150, the trailer's title face, in a box `width`×`height`.
fn text(width: f64, height: f64) -> Value {
    json!({"type": "text", "font": "cinzel-bold", "size": 150, "color": "#E3C067",
           "align": "center", "runs": [{"text": "SPY"}], "width": width, "height": height})
}

fn shape(kind: &str, width: f64, height: f64) -> Value {
    json!({"type": kind, "width": width, "height": height, "fill": "#F2F2F2"})
}

/// A `linear` gradient paint at `angle`, opaque to transparent (ADR-0149).
fn linear(angle: f64) -> Value {
    json!({"gradient": "linear", "angle": angle, "stops": [
        {"offset": 0, "color": "#FF3366"}, {"offset": 0.5, "color": "#FFCC00"},
        {"offset": 1, "color": "#3366FF00"}]})
}

/// A `radial` gradient paint centred off the box's middle (ADR-0149).
fn radial() -> Value {
    json!({"gradient": "radial", "center": [0.3, 0.25], "radius": 0.9, "stops": [
        {"offset": 0, "color": "#FFFFFF"}, {"offset": 1, "color": "#2040A0"}]})
}

/// `base` with `fields` laid over it.
fn with(mut base: Value, fields: Value) -> Value {
    for (key, value) in fields.as_object().expect("fields are an object") {
        base[key] = value.clone();
    }
    base
}

/// Each case: a name, the element without its timing, its `x` at the slot's start, and how
/// far `x` drifts over the slot. One case to a line, as `gen_edge.py` has them.
#[rustfmt::skip]
fn cases() -> Vec<(&'static str, Value, f64, f64)> {
    let t = || text(600.0, 225.0);
    vec![
        ("rect-blur-center", with(shape("rect", 400.0, 200.0), json!({"y": 540, "effects": [blur(40.0)]})), 960.37, 13.7),
        ("ellipse-shadow-offset", with(shape("ellipse", 300.0, 180.0), json!({"y": 500, "fill": "#3BA0FF", "effects": [shadow(24.0, 18.0, 24.0)]})), 800.5, 13.7),
        ("rect-shadow-frac-offset", with(shape("rect", 320.0, 160.0), json!({"y": 600, "radius": 24, "stroke": "#FF3B30", "stroke_width": 6, "effects": [shadow(7.5, -3.25, 20.0)]})), 700.1, 13.7),
        ("ellipse-glow-zero", with(shape("ellipse", 64.0, 64.0), json!({"y": 540, "effects": [glow(60.0, "#FFFFFF", 0.6)]})), 1000.3, 13.7),
        ("text-glow-zero", with(t(), json!({"y": 540, "effects": [shadow(0.0, 0.0, 28.0)]})), 960.4, 13.7),
        ("text-blur-left-edge", with(t(), json!({"y": 540, "effects": [blur(14.0)]})), -60.3, 13.7),
        ("rect-blur-over-right-bottom", with(shape("rect", 300.0, 200.0), json!({"y": 1040, "effects": [blur(30.0)]})), 1850.6, 13.7),
        ("rect-offframe-blur-reaches-in", with(shape("rect", 40.0, 200.0), json!({"y": 300, "fill": "#FFFFFF", "effects": [blur(60.0)]})), -45.5, 4.3),
        ("shadow-offset-into-frame", with(shape("rect", 100.0, 100.0), json!({"y": 200, "fill": "#FFFFFF", "effects": [shadow(70.0, 0.0, 30.0)]})), -80.25, 3.1),
        ("text-scale-2.5-blur", with(t(), json!({"y": 540, "scale": [2.5, 2.5], "effects": [blur(16.0)]})), 960.4, 13.7),
        ("text-scale-0.4-shadow", with(t(), json!({"y": 540, "scale": [0.4, 0.4], "effects": [shadow(0.0, 0.0, 28.0)]})), 960.4, 13.7),
        ("text-scale-anim-blur", with(t(), json!({"y": 540, "effects": [blur(14.0)]})), 400.4, 13.7),
        ("rect-anisotropic-scale-shadow", with(shape("rect", 200.0, 200.0), json!({"y": 540, "scale": [2.0, 0.5], "effects": [shadow(10.0, 10.0, 20.0)]})), 960.4, 13.7),
        ("rect-rotated-30-blur", with(shape("rect", 400.0, 120.0), json!({"y": 540, "rotation": 30, "effects": [blur(24.0)]})), 960.4, 13.7),
        ("text-overflows-box-shadow", with(text(60.0, 40.0), json!({"y": 540, "effects": [shadow(0.0, 0.0, 20.0)]})), 960.4, 13.7),
        ("text-stroke-shadow", with(t(), json!({"y": 540, "stroke": "#FF3B30", "stroke_width": 8, "effects": [shadow(0.0, 0.0, 24.0)]})), 960.4, 13.7),
        ("opacity-half-shadow", with(shape("ellipse", 200.0, 200.0), json!({"y": 540, "opacity": 0.5, "effects": [shadow(12.0, 12.0, 24.0)]})), 960.4, 13.7),
        ("blur-then-shadow", with(shape("rect", 300.0, 150.0), json!({"y": 540, "effects": [blur(10.0), shadow(20.0, 20.0, 30.0)]})), 960.4, 13.7),
        ("shadow-then-blur", with(shape("rect", 300.0, 150.0), json!({"y": 540, "effects": [shadow(20.0, 20.0, 30.0), blur(10.0)]})), 960.4, 13.7),
        ("mask-then-blur", with(shape("rect", 300.0, 300.0), json!({"y": 540, "effects": [{"name": "mask", "shape": "circle"}, blur(20.0)]})), 960.4, 13.7),
        ("image-blur-clip", json!({"type": "image", "source": "img/boat.jpg", "fit": "literal", "y": 540, "width": 640, "height": 360, "clip": [700, 400, 400, 300], "effects": [blur(20.0)]}), 960.4, 13.7),
        ("tiny-sigma-blur", with(shape("rect", 300.0, 150.0), json!({"y": 540, "effects": [blur(3.0)]})), 960.4, 13.7),
        ("text-overflow-tiny-blur", with(text(60.0, 40.0), json!({"y": 540, "effects": [blur(2.0)]})), 960.4, 13.7),
        ("text-wide-stroke-tiny-shadow", with(t(), json!({"y": 540, "stroke": "#FF3B30", "stroke_width": 24, "effects": [shadow(0.0, 0.0, 3.0)]})), 960.4, 13.7),
        ("ellipse-stroke-tiny-blur", with(shape("ellipse", 300.0, 200.0), json!({"y": 540, "stroke": "#FF3B30", "stroke_width": 20, "effects": [blur(2.0)]})), 960.4, 13.7),
        ("text-scale-3-tiny-blur-edge", with(text(300.0, 120.0), json!({"y": 1050, "scale": [3.0, 3.0], "effects": [blur(1.0)]})), 1880.4, 13.7),
        // Beyond the research's set.
        ("text-flipped-blur", with(t(), json!({"y": 540, "scale": [-1.2, 1.2], "effects": [blur(14.0)]})), 700.4, 13.7),
        ("text-flipped-both-shadow", with(t(), json!({"y": 500, "scale": [-0.8, -1.1], "effects": [shadow(24.0, 18.0, 6.0)]})), 1100.4, 13.7),
        ("rect-rotated-anisotropic-flipped-shadow", with(shape("rect", 300.0, 120.0), json!({"y": 540, "rotation": 20, "scale": [-1.5, 0.6], "effects": [shadow(10.0, -6.0, 16.0)]})), 960.4, 13.7),
        // Inverted masks (ADR-0152 §4): the eraser is the shape itself, and the hint's
        // premise that a mask only erases must hold for it, under every transform.
        ("inverted-circle-then-blur", with(shape("rect", 300.0, 300.0), json!({"y": 540, "effects": [inverted("circle"), blur(20.0)]})), 960.4, 13.7),
        ("inverted-ellipse-then-shadow", with(shape("rect", 300.0, 200.0), json!({"y": 540, "effects": [inverted("ellipse"), shadow(14.0, 10.0, 24.0)]})), 960.4, 13.7),
        ("blur-then-inverted-rect-radius", with(shape("rect", 300.0, 200.0), json!({"y": 540, "effects": [blur(16.0), inverted_rect(40, 30, 140, 90, 20)]})), 960.4, 13.7),
        ("inverted-circle-rotated-anisotropic-shadow", with(shape("rect", 300.0, 200.0), json!({"y": 540, "rotation": 25, "scale": [1.6, 0.7], "effects": [inverted("circle"), shadow(8.0, -6.0, 18.0)]})), 960.4, 13.7),
        ("inverted-rect-rotated-blur", with(shape("rect", 300.0, 200.0), json!({"y": 540, "rotation": 33, "effects": [inverted_rect(60, 40, 180, 120, 0), blur(12.0)]})), 960.4, 13.7),
        ("inverted-ellipse-flipped-blur-shadow", with(shape("rect", 300.0, 200.0), json!({"y": 540, "scale": [-1.3, 1.1], "effects": [inverted("ellipse"), blur(6.0), shadow(10.0, 10.0, 20.0)]})), 960.4, 13.7),
        ("ring-then-blur", with(shape("rect", 300.0, 300.0), json!({"y": 540, "effects": [{"name": "mask", "shape": "ellipse"}, {"name": "mask", "shape": "ellipse", "x": 90, "y": 90, "width": 120, "height": 120, "invert": true}, blur(10.0)]})), 960.4, 13.7),
        // Feathered masks (#698): a feather is a blur of the mask's own coverage, in its own
        // bounded layer, under the hinted layers or over them, plain and inverted.
        ("feathered-circle-then-blur", with(shape("rect", 300.0, 300.0), json!({"y": 540, "effects": [feathered("circle", false, json!(30)), blur(20.0)]})), 960.4, 13.7),
        ("feathered-inverted-ellipse-then-shadow", with(shape("rect", 300.0, 200.0), json!({"y": 540, "effects": [feathered("ellipse", true, json!(24)), shadow(14.0, 10.0, 24.0)]})), 960.4, 13.7),
        ("blur-then-feathered-rect-radius", with(shape("rect", 300.0, 200.0), json!({"y": 540, "effects": [blur(16.0), {"name": "mask", "shape": "rect", "x": 40, "y": 30, "width": 220, "height": 140, "radius": 20, "feather": 40}]})), 960.4, 13.7),
        ("feathered-inverted-rect-rotated-anisotropic-shadow", with(shape("rect", 300.0, 200.0), json!({"y": 540, "rotation": 25, "scale": [1.6, 0.7], "effects": [{"name": "mask", "shape": "rect", "x": 60, "y": 40, "width": 180, "height": 120, "invert": true, "feather": 20}, shadow(8.0, -6.0, 18.0)]})), 960.4, 13.7),
        ("feathered-circle-flipped-blur-shadow", with(shape("ellipse", 300.0, 200.0), json!({"y": 540, "scale": [-1.3, 1.1], "effects": [feathered("circle", false, json!(16)), blur(6.0), shadow(10.0, 10.0, 20.0)]})), 960.4, 13.7),
        ("keyed-feather-inverted-ellipse-blur", with(shape("rect", 300.0, 300.0), json!({"y": 540, "effects": [feathered("ellipse", true, json!(0)), blur(10.0)]})), 960.4, 13.7),
        ("feathered-ring-then-blur", with(shape("rect", 300.0, 300.0), json!({"y": 540, "effects": [feathered("ellipse", false, json!(20)), {"name": "mask", "shape": "ellipse", "x": 90, "y": 90, "width": 120, "height": 120, "invert": true, "feather": 30}, blur(10.0)]})), 960.4, 13.7),
        ("image-feathered-vignette-blur", json!({"type": "image", "source": "img/boat.jpg", "fit": "literal", "y": 540, "width": 640, "height": 360, "effects": [feathered("ellipse", false, json!(80)), blur(4.0)]}), 960.4, 13.7),
        ("feathered-mask-alone", with(shape("rect", 300.0, 300.0), json!({"y": 540, "rotation": 15, "effects": [feathered("circle", true, json!(30))]})), 960.4, 13.7),
        // Gradient paint (ADR-0149 §8): its coordinates live inside the layer whose origin
        // the hint moves, so it is measured here rather than inferred.
        ("rect-linear-gradient-blur", with(shape("rect", 400.0, 200.0), json!({"y": 540, "fill": linear(30.0), "effects": [blur(40.0)]})), 960.37, 13.7),
        ("ellipse-radial-gradient-shadow-offset", with(shape("ellipse", 300.0, 180.0), json!({"y": 500, "fill": radial(), "effects": [shadow(24.0, 18.0, 24.0)]})), 800.5, 13.7),
        ("rect-gradient-stroke-rotated-anisotropic-shadow", with(shape("rect", 300.0, 120.0), json!({"y": 540, "rotation": 20, "scale": [-1.5, 0.6], "fill": linear(200.0), "stroke": radial(), "stroke_width": 10, "effects": [shadow(10.0, -6.0, 16.0)]})), 960.4, 13.7),
        ("text-gradient-stroke-glow", with(t(), json!({"y": 540, "color": linear(90.0), "stroke": radial(), "stroke_width": 8, "effects": [shadow(0.0, 0.0, 24.0)]})), 960.4, 13.7),
        ("text-gradient-scale-blur", with(t(), json!({"y": 540, "scale": [2.0, 0.7], "color": radial(), "effects": [blur(14.0)]})), 700.4, 13.7),
        // Paths (ADR-0154): a centred stroke the box contains, so the hint's bound on the box
        // must hold for one whose ink runs right up to the box's edge.
        ("path-star-blur", json!({"type": "path", "width": 300, "height": 280, "closed": true, "fill": "#F2F2F2", "stroke": "#FF3B30", "stroke_width": 8, "points": [{"at": [150, 4]}, {"at": [238, 276]}, {"at": [4, 108]}, {"at": [296, 108]}, {"at": [62, 276]}], "y": 540, "effects": [blur(18.0)]}), 960.4, 13.7),
        ("path-line-rotated-shadow-edge", json!({"type": "path", "width": 400, "height": 10, "closed": false, "stroke": "#FFFFFF", "stroke_width": 10, "points": [{"at": [5, 5]}, {"at": [395, 5]}], "y": 300, "rotation": 35, "effects": [shadow(12.0, -9.0, 14.0)]}), 960.4, 13.7),
        ("path-curve-anisotropic-flipped-glow", json!({"type": "path", "width": 300, "height": 200, "closed": false, "stroke": "#3BA0FF", "stroke_width": 6, "points": [{"at": [3, 100], "out": [100, -97]}, {"at": [297, 100], "in": [-100, 97]}], "y": 540, "scale": [-2.0, 0.6], "effects": [glow(40.0, "#FFFFFF", 0.7), blur(2.0)]}), 960.4, 13.7),
        ("path-star-tiny-shadow-edge", json!({"type": "path", "width": 200, "height": 190, "closed": true, "fill": "#FFFFFF", "points": [{"at": [100, 0]}, {"at": [159, 190]}, {"at": [0, 72]}, {"at": [200, 72]}, {"at": [41, 190]}], "y": 1000, "effects": [shadow(0.0, 0.0, 3.0)]}), 1880.4, 13.7),
        // Past the σ 135 precondition, so unbounded both ways: #649's `gen_big.py` cases.
        ("rect-blur-sigma150", with(shape("rect", 400.0, 200.0), json!({"y": 540, "effects": [blur(300.0)]})), 960.4, 13.7),
        ("text-scale3-blur-sigma150", with(t(), json!({"y": 500, "scale": [3.0, 3.0], "effects": [blur(100.0)]})), 700.4, 13.7),
        ("ellipse-shadow-sigma140-offset", with(shape("ellipse", 200.0, 200.0), json!({"y": 400, "effects": [shadow(40.0, 30.0, 280.0)]})), 500.4, 13.7),
    ]
}

/// The project, one case per second on its own track, and the instants to compare.
fn edge_project() -> (Value, Vec<(&'static str, i64)>) {
    let mut tracks = Vec::new();
    let mut instants = Vec::new();
    for (i, (name, element, x, drift)) in cases().into_iter().enumerate() {
        let start = i as i64 * 1000;
        let mut element = with(
            element,
            json!({
                "id": name, "start": start, "end": start + 1000, "origin": "center",
                "x": [{"t": start, "v": x.round()},
                      {"t": start + 1000, "v": (x + drift).round(), "ease": "linear"}],
            }),
        );
        if name == "text-scale-anim-blur" {
            element["scale"] = json!([{"t": start, "v": [1.4, 1.4]},
                                      {"t": start + 1000, "v": [0.93, 0.93], "ease": "linear"}]);
        }
        if name == "keyed-feather-inverted-ellipse-blur" {
            // 0 → 45 → 7: a hard edge at the first instant, fractional feathers after.
            element["effects"][0]["feather"] = json!([{"t": start, "v": 0},
                                                      {"t": start + 500, "v": 45, "ease": "ease-in-out"},
                                                      {"t": start + 1000, "v": 7, "ease": "linear"}]);
        }
        tracks.push(json!({"name": format!("t{i}"), "layer": i, "elements": [element]}));
        instants.extend([0, 333, 667, 967].map(|k| (name, start + k)));
    }
    let project = json!({
        "frame": {"width": 1920, "height": 1080}, "fps": 30, "background": "#101418",
        "duration": tracks.len() * 1000,
        "fonts": {"cinzel-bold": [{"file": "fonts/Cinzel-Bold.ttf"}]},
        "fontVendor": {"fonts/Cinzel-Bold.ttf": {
            "licence": "OFL-1.1",
            "source": "google/fonts ofl/cinzel, instanced wght=700",
            "sha256": "c9ac320a5f48ecb57db76bc0b942ccc39eb89994e37fb182a454ec273ee43e02"}},
        "tracks": tracks,
    });
    (project, instants)
}

/// One instant at true scale, as lossless PNG bytes.
///
/// Compared as bytes: the encoder is deterministic and lossless, so the same pixels make
/// the same file and different pixels a different one. Decoding is left to a failure, where
/// it says how many pixels moved.
#[track_caller]
fn painted(project: &Path, at: i64) -> Vec<u8> {
    let answer = frame(
        project,
        &Ask {
            at: Some(at),
            full: true,
            png: true,
            ..Ask::default()
        },
    );
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "frame did not answer at {at}: {}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );
    answer.image().expect("a picture").bytes.clone()
}

/// How many pixels of two PNGs differ.
fn differing(a: &[u8], b: &[u8]) -> usize {
    let decode = |png| {
        image::load_from_memory(png)
            .expect("the bytes decode")
            .to_rgba8()
    };
    let (a, b) = (decode(a), decode(b));
    a.pixels().zip(b.pixels()).filter(|(a, b)| a != b).count()
}

/// Every `SHARDS`th instant from `shard`, painted both ways; the ones that differ.
///
/// Sharded so the instants paint on several test threads at once. The switch is per thread,
/// so one shard turning the hint off cannot reach another's painter.
fn differing_instants(shard: usize) -> Vec<String> {
    let dir = tempdir(line!() * 10 + shard as u32);
    for asset in ["fonts/Cinzel-Bold.ttf", "img/boat.jpg"] {
        let to = dir.join(asset);
        std::fs::create_dir_all(to.parent().expect("a parent")).expect("asset dir");
        std::fs::copy(trailer().join(asset), &to).expect("copy the trailer's asset");
    }
    let (project, instants) = edge_project();
    let project = write_project(
        &dir,
        "edge.montagent.json",
        &canonical(&project.to_string()),
    );

    let mut differ = Vec::new();
    for (name, at) in instants.into_iter().skip(shard).step_by(SHARDS) {
        // `frame` paints on this thread, which is the one the switch reaches.
        bound_filter_layers(false);
        let unbounded = painted(&project, at);
        bound_filter_layers(true);
        let bounded = painted(&project, at);
        if bounded != unbounded {
            let pixels = differing(&bounded, &unbounded);
            differ.push(format!("{name} at {at}: {pixels} pixels"));
        }
    }
    differ
}

const SHARDS: usize = 4;

#[track_caller]
fn same_bytes(shard: usize) {
    let differ = differing_instants(shard);
    assert!(
        differ.is_empty(),
        "the hint changed pixels:\n{}",
        differ.join("\n")
    );
}

#[test]
fn the_edge_cases_paint_the_same_bytes_with_the_hint_as_without_it_0() {
    same_bytes(0);
}

#[test]
fn the_edge_cases_paint_the_same_bytes_with_the_hint_as_without_it_1() {
    same_bytes(1);
}

#[test]
fn the_edge_cases_paint_the_same_bytes_with_the_hint_as_without_it_2() {
    same_bytes(2);
}

#[test]
fn the_edge_cases_paint_the_same_bytes_with_the_hint_as_without_it_3() {
    same_bytes(3);
}
