//! Prototype #786 (ADR-0167): evidence, not a build. Every test here is `#[ignore]` and is
//! run by hand in release:
//!
//! ```text
//! python3 prototype/projection-786/make_scenes.py
//! cargo test --release -p montagent-core --test projection_786 -- --ignored --nocapture --test-threads 1
//! ```
//!
//! - `byte_identity_*`: raw RGB frames at the encoder's input, compared byte for byte against
//!   a one-painter, hint-on baseline, at K painters over C-frame chunks, hint on and off.
//! - `no_op_projection`: `swivel: 0`, `tilt: 0` and any `perspective` against no projection.
//! - `corners_match_painted_pixels`: the projected quadrilateral against painted pixels.
//! - `cost_at_1080p`: one element's paint time, projected against flat.

use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use montagent_core::verbs::render::{
    Ask, Forced, Progress, force_painting, render, take_frame_bytes, tap_frame_bytes,
};
use montagent_render::canvas::{
    Canvas, Effect, Extent, Fill, Ink, MaskRect, MaskShape, Projection, Raster, Rgba, Shape,
    Transform, bound_filter_layers, projected_corners, proto786,
};
use serde_json::Value;

mod common;

fn projects() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../prototype/projection-786/projects")
}

/// One render's raw frames, with the hint `on` or off, painted as `forced`.
fn frames(path: &Path, forced: Forced, on: bool) -> Vec<Vec<u8>> {
    let path = path.to_path_buf();
    let out = std::env::temp_dir().join(format!(
        "proto786-{}-{}.mp4",
        std::process::id(),
        path.file_stem().unwrap().to_string_lossy()
    ));
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        bound_filter_layers(on);
        let _forced = force_painting(forced);
        tap_frame_bytes();
        let ask = Ask {
            output: Some(out.clone()),
            ..Ask::default()
        };
        let answer = render(&path, &ask, &mut |_: Progress| {});
        let _ = std::fs::remove_file(&out);
        let _ = tx.send((answer.report().exit_code(), answer.to_json(), take_frame_bytes()));
    });
    let (exit, answer, bytes) = rx
        .recv_timeout(Duration::from_secs(1800))
        .expect("the render finished");
    assert_eq!(exit, montagent_core::report::ExitCode::Ok, "{answer}");
    bytes
}

/// How many frames and bytes differ, and the largest byte difference.
fn diff(a: &[Vec<u8>], b: &[Vec<u8>]) -> (usize, usize, u8) {
    assert_eq!(a.len(), b.len(), "frame count");
    let (mut frames, mut bytes, mut max) = (0, 0, 0u8);
    for (x, y) in a.iter().zip(b) {
        assert_eq!(x.len(), y.len());
        let n = x.iter().zip(y).filter(|(p, q)| p != q).count();
        if n > 0 {
            frames += 1;
            bytes += n;
            max = max.max(x.iter().zip(y).map(|(p, q)| p.abs_diff(*q)).max().unwrap());
        }
    }
    (frames, bytes, max)
}

fn counters() -> [usize; 4] {
    [
        proto786::PROJECTED.load(Ordering::Relaxed),
        proto786::AWAY.load(Ordering::Relaxed),
        proto786::EYE_REFUSED.load(Ordering::Relaxed),
        proto786::HINTED.load(Ordering::Relaxed),
    ]
}

fn sweep(path: &Path, painters: &[usize], chunks: &[u64]) -> bool {
    let name = path.file_name().unwrap().to_string_lossy().to_string();
    let before = counters();
    let baseline = frames(path, Forced::OnePainter, true);
    let after = counters();
    println!(
        "{name}: baseline one painter, hint on: {} frames of {} bytes; projected paints {}, \
         away/edge {}, eye-refused {}, hinted layers {}",
        baseline.len(),
        baseline[0].len(),
        after[0] - before[0],
        after[1] - before[1],
        after[2] - before[2],
        after[3] - before[3],
    );
    let mut all = true;
    let mut runs = vec![(Forced::OnePainter, false)];
    for &k in painters {
        for &c in chunks {
            for on in [true, false] {
                runs.push((
                    Forced::Chunks {
                        painters: k,
                        chunk: c,
                        window_bytes: None,
                    },
                    on,
                ));
            }
        }
    }
    for (forced, on) in runs {
        let h0 = proto786::HINTED.load(Ordering::Relaxed);
        let got = frames(path, forced, on);
        let hinted = proto786::HINTED.load(Ordering::Relaxed) - h0;
        let (f, b, m) = diff(&baseline, &got);
        let label = match forced {
            Forced::OnePainter => "K=1 (verb thread)".to_string(),
            Forced::Chunks {
                painters, chunk, ..
            } => format!("K={painters:>2} C={chunk:>2}"),
        };
        let ok = f == 0;
        all &= ok;
        println!(
            "  {name} {label} hint={:<3} hinted={hinted:>4}: {}",
            if on { "on" } else { "off" },
            if ok {
                "IDENTICAL".to_string()
            } else {
                format!("DIFFERS: {f} frames, {b} bytes, max |d| {m}")
            }
        );
    }
    all
}

#[test]
#[ignore]
fn byte_identity_blur_shadow_mask_card() {
    let path = projects().join("04-blur-shadow-mask.montagent.json");
    let ok = sweep(&path, &(1..=10).collect::<Vec<_>>(), &[1, 2, 7, 30]);
    assert!(ok, "not byte-identical");
}

#[test]
#[ignore]
fn byte_identity_blur_shadow_mask_card_1080p() {
    let src = projects().join("04-blur-shadow-mask.montagent.json");
    let mut doc: Value = serde_json::from_str(&std::fs::read_to_string(&src).unwrap()).unwrap();
    // The same scene at 1920x1080 with every element scaled 1.5 about the frame centre.
    doc["frame"] = serde_json::json!({"width": 1920, "height": 1080});
    for track in doc["tracks"].as_array_mut().unwrap() {
        for e in track["elements"].as_array_mut().unwrap() {
            for k in ["x", "y", "width", "height"] {
                if let Some(v) = e.get(k).and_then(Value::as_i64) {
                    e[k] = Value::from(v * 3 / 2);
                }
            }
            if e.get("perspective").is_some() {
                e["perspective"] = Value::from(1650);
            }
        }
    }
    let path = projects().join("tmp-04-1080.montagent.json");
    std::fs::write(&path, common::canonical(&doc.to_string())).unwrap();
    let ok = sweep(&path, &[1, 2, 3, 6, 10], &[1, 7]);
    let _ = std::fs::remove_file(&path);
    assert!(ok, "not byte-identical");
}

#[test]
#[ignore]
fn byte_identity_every_scene() {
    let mut ok = true;
    let mut names: Vec<PathBuf> = std::fs::read_dir(projects())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            let n = p.file_name().unwrap().to_string_lossy().to_string();
            n.ends_with(".montagent.json") && !n.starts_with("tmp-")
        })
        .collect();
    names.sort();
    for path in names {
        ok &= sweep(&path, &[1, 2, 3, 5, 10], &[1, 7]);
    }
    assert!(ok, "not byte-identical");
}

/// `swivel: 0`, `tilt: 0` and any `perspective` against the same element with none of the
/// three: byte for byte, on the painter's normal path, and with the flat layer forced at 0°.
#[test]
#[ignore]
fn no_op_projection() {
    let src = projects().join("04-blur-shadow-mask.montagent.json");
    let base: Value = serde_json::from_str(&std::fs::read_to_string(&src).unwrap()).unwrap();
    let variant = |fields: Option<(f64, f64, f64)>| {
        let mut doc = base.clone();
        doc["duration"] = Value::from(500);
        for track in doc["tracks"].as_array_mut().unwrap() {
            for e in track["elements"].as_array_mut().unwrap() {
                e["end"] = Value::from(500);
                if e["id"] == "card" {
                    let o = e.as_object_mut().unwrap();
                    o.remove("swivel");
                    o.remove("tilt");
                    o.remove("perspective");
                    // A rotation and a non-uniform scale, so the flat path's resample shows.
                    o.insert("rotation".into(), Value::from(12));
                    o.insert("scale".into(), serde_json::json!([1.1, 0.9]));
                    if let Some((s, t, p)) = fields {
                        o.insert("swivel".into(), Value::from(s));
                        o.insert("tilt".into(), Value::from(t));
                        o.insert("perspective".into(), Value::from(p));
                    }
                }
            }
        }
        doc
    };
    let write = |name: &str, doc: &Value| {
        let path = projects().join(format!("tmp-noop-{name}.montagent.json"));
        std::fs::write(&path, common::canonical(&doc.to_string())).unwrap();
        path
    };
    let plain = write("plain", &variant(None));
    let reference = frames(&plain, Forced::OnePainter, true);
    let mut ok = true;
    for p in [400.0, 1100.0, 5000.0, 1.0e9] {
        let path = write(&format!("p{p}"), &variant(Some((0.0, 0.0, p))));
        let got = frames(&path, Forced::OnePainter, true);
        let (f, b, m) = diff(&reference, &got);
        println!(
            "swivel 0, tilt 0, perspective {p}: {}",
            if f == 0 {
                "IDENTICAL to no projection".to_string()
            } else {
                format!("DIFFERS {f} frames {b} bytes max {m}")
            }
        );
        ok &= f == 0;
        proto786::FORCE_FLAT.store(true, Ordering::Relaxed);
        let forced = frames(&path, Forced::OnePainter, true);
        proto786::FORCE_FLAT.store(false, Ordering::Relaxed);
        let (f, b, m) = diff(&reference, &forced);
        let total: usize = reference.iter().map(Vec::len).sum();
        println!(
            "  same, flat layer forced at 0 deg: {f} of {} frames differ, {b} of {total} bytes \
             ({:.3}%), max |d| {m}",
            reference.len(),
            100.0 * b as f64 / total as f64
        );
        let _ = std::fs::remove_file(&path);
    }
    let _ = std::fs::remove_file(&plain);
    assert!(ok);
}

fn transform(x: f64, y: f64, origin: (f64, f64), projection: Option<Projection>) -> Transform {
    Transform {
        x,
        y,
        origin,
        scale: (1.0, 1.0),
        rotation: 0.0,
        opacity: 1.0,
        blend: Default::default(),
        projection,
    }
}

/// The four corners [`projected_corners`] reports, checked against painted pixels: a point
/// 3 px inside each corner (toward the quadrilateral's centroid) is the fill, a point 3 px
/// outside is the background.
#[test]
#[ignore]
fn corners_match_painted_pixels() {
    let (w, h) = (1280i64, 720i64);
    let fill = Fill {
        fill: Some(Ink::Flat(Rgba([255, 255, 255, 255]))),
        stroke: None,
        stroke_width: 0.0,
    };
    let p = |swivel, tilt, perspective| {
        Some(Projection {
            swivel,
            tilt,
            perspective,
        })
    };
    let mut cases = vec![
        ("centre, swivel 35 tilt 20", transform(640.0, 360.0, (0.5, 0.5), p(35.0, 20.0, 900.0))),
        ("center-left door, swivel -60", transform(400.0, 360.0, (0.0, 0.5), p(-60.0, 0.0, 700.0))),
        ("top-left, tilt 50", transform(300.0, 150.0, (0.0, 0.0), p(0.0, 50.0, 800.0))),
        ("eye bound + 1 (r = 235.8), swivel 40", transform(640.0, 360.0, (0.5, 0.5), p(40.0, 0.0, 236.0))),
    ];
    let mut turned = transform(640.0, 360.0, (0.5, 0.5), p(40.0, -25.0, 1000.0));
    turned.rotation = 30.0;
    turned.scale = (1.3, 0.8);
    cases.push(("swivel 40 tilt -25, then rotation 30, scale [1.3, 0.8]", turned));
    let extent = Extent {
        width: 400.0,
        height: 250.0,
    };
    let mut ok = true;
    for (name, t) in cases {
        let mut canvas = Canvas::new(w, h).unwrap();
        canvas.background(Rgba([0, 0, 0, 255]));
        canvas.shape(Shape::Rect { radius: 0.0 }, extent, &t, &fill, None, None, None, &[]);
        let rgba = canvas.rgba().unwrap();
        let at = |x: f64, y: f64| {
            let (xi, yi) = (x.round() as i64, y.round() as i64);
            if xi < 0 || yi < 0 || xi >= w || yi >= h {
                return None;
            }
            Some(rgba[((yi * w + xi) * 4) as usize])
        };
        let corners = projected_corners(extent, &t, &[]);
        let (cx, cy) = corners
            .iter()
            .fold((0.0, 0.0), |(a, b), (x, y)| (a + x / 4.0, b + y / 4.0));
        println!("{name}:");
        for (i, (x, y)) in corners.iter().enumerate() {
            let (dx, dy) = (cx - x, cy - y);
            let len = (dx * dx + dy * dy).sqrt();
            let (ux, uy) = (dx / len, dy / len);
            let inside = at(x + 3.0 * ux, y + 3.0 * uy);
            let outside = at(x - 3.0 * ux, y - 3.0 * uy);
            let good = inside == Some(255) && matches!(outside, Some(0) | None);
            ok &= good;
            println!(
                "  {} ({x:8.2}, {y:8.2})  3px in: {:?}  3px out: {:?}  {}",
                ["TL", "TR", "BR", "BL"][i],
                inside,
                outside,
                if good { "ok" } else { "MISMATCH" }
            );
        }
    }
    assert!(ok);
}

/// One element's paint at 1080p, projected against flat, with and without blur and shadow:
/// the median of `N` paints, the background clear subtracted.
#[test]
#[ignore]
fn cost_at_1080p() {
    const N: usize = 60;
    let bytes = std::fs::read(projects().join("media/cards.jpg")).unwrap();
    let source = Raster::decode(&bytes).unwrap();
    let extent = Extent {
        width: 960.0,
        height: 600.0,
    };
    let mask = Effect::Mask {
        shape: MaskShape::Rect,
        rect: None::<MaskRect>,
        radius: 36.0,
        invert: false,
        feather: 0.0,
    };
    let shadow = Effect::Shadow {
        dx: 18.0,
        dy: 26.0,
        radius: 24.0,
        colour: Rgba([0, 0, 0, 255]),
        opacity: 0.75,
    };
    let blur = Effect::Blur { radius: 3.0 };
    let median = |f: &mut dyn FnMut()| {
        for _ in 0..5 {
            f();
        }
        let mut times: Vec<f64> = (0..N)
            .map(|_| {
                let s = Instant::now();
                f();
                s.elapsed().as_secs_f64() * 1e3
            })
            .collect();
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        times[N / 2]
    };
    let mut canvas = Canvas::new(1920, 1080).unwrap();
    let clear = median(&mut || canvas.background(Rgba([20, 22, 28, 255])));
    println!("background clear alone: {clear:.2} ms (subtracted below)");
    let projection = Some(Projection {
        swivel: 30.0,
        tilt: 15.0,
        perspective: 1800.0,
    });
    for (label, effects) in [
        ("no effects", vec![]),
        ("mask + shadow + blur", vec![mask.clone(), shadow.clone(), blur.clone()]),
    ] {
        for (kind, p) in [("flat", None), ("projected", projection)] {
            let t = transform(960.0, 540.0, (0.5, 0.5), p);
            let ms = median(&mut || {
                canvas.background(Rgba([20, 22, 28, 255]));
                canvas.raster(&source, extent, &t, None, &effects);
            }) - clear;
            println!("  960x600 image, {label:<22} {kind:<10} {ms:7.2} ms/frame");
        }
    }
}
