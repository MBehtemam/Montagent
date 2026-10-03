//! Throwaway spike for #644: can Skia reuse a filtered element's output across frames, and do
//! K painters on K threads paint the same bytes as one painter painting sequentially?
//!
//! Every frame is 1920x1080 RGBA8888 premul on a CPU raster surface, the same `ImageInfo`
//! `montagent-render`'s `Canvas::new` makes. `element` + `through` mirror
//! `Canvas::in_element_space` + `Canvas::through` for one effect: an optional
//! `save_layer_alpha_f` for opacity *outside* the transform, then translate / rotate / scale /
//! origin, then one `save_layer` whose paint carries the effect's `ImageFilter`.
//!
//! The "live" draw (a fresh `ImageFilter` per frame, as today) is the reference. Each reuse
//! candidate is compared with it byte for byte on the composited frame.

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::time::Instant;

use skia_safe::{
    canvas::SaveLayerRec, graphics, image_filters, images, surfaces, AlphaType, Canvas, Color, Color4f,
    ColorType, FilterMode, Font, FontMgr, FontStyle, IRect, ISize, Image, ImageFilter, ImageInfo,
    Paint, PaintStyle, Picture, PictureRecorder, Point, RRect, Rect, RoundOut, SamplingOptions, Surface,
};

const W: i32 = 1920;
const H: i32 = 1080;

#[derive(Clone, Copy, Debug)]
struct T {
    x: f64,
    y: f64,
    rot: f64,
    scale: f64,
    opacity: f64,
}

#[derive(Clone, Copy, Debug)]
enum Fx {
    Blur { radius: f64 },
    Shadow { dx: f64, dy: f64, radius: f64, rgba: [u8; 4], opacity: f64 },
}

const EXT: (f64, f64) = (960.0, 240.0);
const ORIGIN: (f64, f64) = (0.5, 0.5);

fn sigma(radius: f64) -> f32 {
    (radius.max(0.0) / 2.0) as f32
}

/// `Effect::filter` from `canvas.rs`, for the two members #644 is about.
fn filter(fx: Fx) -> ImageFilter {
    match fx {
        Fx::Blur { radius } => {
            image_filters::blur((sigma(radius), sigma(radius)), None, None, None).unwrap()
        }
        Fx::Shadow { dx, dy, radius, rgba: [r, g, b, a], opacity } => {
            let alpha = f32::from(a) / 255.0 * opacity.clamp(0.0, 1.0) as f32;
            let c = Color4f::new(
                f32::from(r) / 255.0,
                f32::from(g) / 255.0,
                f32::from(b) / 255.0,
                alpha,
            );
            image_filters::drop_shadow(
                (dx as f32, dy as f32),
                (sigma(radius), sigma(radius)),
                c,
                None,
                None,
                None,
            )
            .unwrap()
        }
    }
}

fn new_surface() -> Surface {
    let info = ImageInfo::new(ISize::new(W, H), ColorType::RGBA8888, AlphaType::Premul, None);
    surfaces::raster(&info, None, None).unwrap()
}

fn pixels(s: &mut Surface) -> Vec<u8> {
    s.peek_pixels().unwrap().bytes().unwrap().to_vec()
}

fn hash(bytes: &[u8]) -> u64 {
    let mut h = DefaultHasher::new();
    h.write(bytes);
    h.finish()
}

fn make_font(size: f32) -> Font {
    let tf = FontMgr::new()
        .legacy_make_typeface(None, FontStyle::bold())
        .expect("a system typeface");
    Font::from_typeface(tf, size)
}

/// An opaque, non-uniform backdrop so that compositing differences show.
fn background(c: &Canvas) {
    c.clear(Color::from_rgb(18, 22, 30));
    let mut p = Paint::default();
    p.set_anti_alias(true);
    for i in 0..24 {
        p.set_color(Color::from_argb(255, (i * 9) as u8, (80 + i * 5) as u8, (200 - i * 6) as u8));
        c.draw_rect(Rect::from_xywh(i as f32 * 80.0 + 0.3, 0.0, 40.0, H as f32), &p);
    }
}

/// The element's own drawing, in element space (box is (0,0,EXT)).
fn content(c: &Canvas, font: &Font) {
    let (w, h) = (EXT.0 as f32, EXT.1 as f32);
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_color(Color::from_argb(200, 40, 120, 220));
    c.draw_rrect(RRect::new_rect_xy(Rect::from_xywh(8.0, 8.0, w - 16.0, h - 16.0), 32.0, 32.0), &p);
    p.set_style(PaintStyle::Stroke);
    p.set_stroke_width(6.0);
    p.set_color(Color::from_argb(255, 255, 200, 40));
    c.draw_oval(Rect::from_xywh(40.0, 40.0, w - 80.0, h - 80.0), &p);
    let mut t = Paint::default();
    t.set_anti_alias(true);
    t.set_color(Color::WHITE);
    c.draw_str("Montagent #644", (60.0, 160.0), font, &t);
}

/// `Canvas::in_element_space`, minus clip and mask.
fn element(c: &Canvas, t: &T, body: impl FnOnce(&Canvas)) {
    if t.opacity <= 0.0 {
        return;
    }
    c.save();
    let layered = t.opacity < 1.0;
    if layered {
        c.save_layer_alpha_f(None, t.opacity as f32);
    }
    c.translate((t.x as f32, t.y as f32));
    if t.rot != 0.0 {
        c.rotate(t.rot as f32, None);
    }
    c.scale((t.scale as f32, t.scale as f32));
    c.translate(((-ORIGIN.0 * EXT.0) as f32, (-ORIGIN.1 * EXT.1) as f32));
    body(c);
    if layered {
        c.restore();
    }
    c.restore();
}

/// `Canvas::through` for one filtered effect.
fn through(c: &Canvas, f: &ImageFilter, draw: impl FnOnce(&Canvas)) {
    let mut paint = Paint::default();
    paint.set_image_filter(f.clone());
    c.save_layer(&SaveLayerRec::default().paint(&paint));
    draw(c);
    c.restore();
}

/// Side check (not a reuse candidate): the same layer with a `bounds` hint of the element box.
/// Skia sizes an unhinted filter layer to the whole device clip (`SkCanvas::internalSaveLayer`).
fn bounded(s: &mut Surface, font: &Font, fx: Fx, t: &T) -> Vec<u8> {
    let c = s.canvas();
    background(c);
    element(c, t, |c| {
        let mut paint = Paint::default();
        paint.set_image_filter(filter(fx));
        let pad: f32 = std::env::var("HINT_PAD").ok().and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let mut r = Rect::from_wh(EXT.0 as f32, EXT.1 as f32).with_outset((pad, pad));
        // HINT_ORIGIN=1: stretch the hint back to the frame's (0,0), so the layer keeps the
        // frame's pixel origin and only loses the area right of / below the element.
        if std::env::var("HINT_ORIGIN").is_ok() {
            let m = c.local_to_device_as_3x3();
            let dev = m.map_rect(r).0;
            let dev = Rect::new(0.0, 0.0, dev.right, dev.bottom);
            r = m.invert().unwrap().map_rect(dev).0;
        }
        c.save_layer(&SaveLayerRec::default().bounds(&r).paint(&paint));
        content(c, font);
        c.restore();
    });
    pixels(s)
}

/// Today's draw: a fresh filter per frame.
fn live(s: &mut Surface, font: &Font, fx: Fx, t: &T) -> Vec<u8> {
    let c = s.canvas();
    background(c);
    element(c, t, |c| through(c, &filter(fx), |c| content(c, font)));
    pixels(s)
}

/// Candidate A: the same `ImageFilter` object, built once and reused.
fn reused_filter(s: &mut Surface, font: &Font, f: &ImageFilter, t: &T) -> Vec<u8> {
    let c = s.canvas();
    background(c);
    element(c, t, |c| through(c, f, |c| content(c, font)));
    pixels(s)
}

/// Candidate B: an `SkPicture` of the element's layer + content, recorded once in element
/// space and played back under each frame's transform.
fn record(font: &Font, f: &ImageFilter) -> Picture {
    let mut rec = PictureRecorder::new();
    let c = rec.begin_recording(Rect::from_xywh(-400.0, -400.0, 1760.0, 1040.0), false);
    through(c, f, |c| content(c, font));
    rec.finish_recording_as_picture(None).unwrap()
}

fn picture(s: &mut Surface, pic: &Picture, t: &T) -> Vec<u8> {
    let c = s.canvas();
    background(c);
    element(c, t, |c| {
        c.draw_picture(pic, None, None);
    });
    pixels(s)
}

/// Candidate C: a raster snapshot of the finished (filtered) element at `base`, opacity 1,
/// cut to its filtered device bounds.
struct Snap {
    image: Image,
    left_top: (i32, i32),
    base: T,
}

fn snapshot(font: &Font, f: &ImageFilter, base: &T) -> Snap {
    let mut s = new_surface();
    let at = T { opacity: 1.0, ..*base };
    let mut bounds = IRect::new_empty();
    {
        let c = s.canvas();
        c.clear(Color::TRANSPARENT);
        element(c, &at, |c| {
            let local = f.compute_fast_bounds(Rect::from_wh(EXT.0 as f32, EXT.1 as f32));
            let dev = c.local_to_device_as_3x3().map_rect(local).0;
            // One pixel of slack for the restore's AA on a rotated layer.
            let out: IRect = dev.round_out();
            bounds = out.with_outset((1, 1));
            through(c, f, |c| content(c, font));
        });
    }
    let bounds = IRect::intersect(&bounds, &IRect::from_wh(W, H)).unwrap();
    let image = s.image_snapshot_with_bounds(bounds).unwrap();
    Snap { image, left_top: (bounds.left, bounds.top), base: *base }
}

#[derive(Clone, Copy, Debug)]
enum Alpha {
    Layer,
    Paint,
}

fn redraw(s: &mut Surface, snap: &Snap, t: &T, alpha: Alpha, sampling: SamplingOptions) -> Vec<u8> {
    let c = s.canvas();
    background(c);
    let (dx, dy) = ((t.x - snap.base.x) as f32, (t.y - snap.base.y) as f32);
    let k = (t.scale / snap.base.scale) as f32;
    c.save();
    let mut paint = Paint::default();
    let layered = t.opacity < 1.0;
    match alpha {
        Alpha::Layer if layered => {
            c.save_layer_alpha_f(None, t.opacity as f32);
        }
        Alpha::Paint => {
            paint.set_alpha_f(t.opacity as f32);
        }
        _ => {}
    }
    if k != 1.0 {
        c.translate((t.x as f32, t.y as f32));
        c.scale((k, k));
        c.translate((-snap.base.x as f32, -snap.base.y as f32));
    } else {
        c.translate((dx, dy));
    }
    let p = Point::new(snap.left_top.0 as f32, snap.left_top.1 as f32);
    c.draw_image_with_sampling_options(&snap.image, p, sampling, Some(&paint));
    if matches!(alpha, Alpha::Layer) && layered {
        c.restore();
    }
    c.restore();
    pixels(s)
}

fn diff(a: &[u8], b: &[u8]) -> (usize, u8) {
    let mut n = 0;
    let mut m = 0u8;
    for (x, y) in a.iter().zip(b) {
        if x != y {
            n += 1;
            m = m.max(x.abs_diff(*y));
        }
    }
    (n, m)
}

fn verdict(reference: &[u8], got: &[u8]) -> String {
    let (n, m) = diff(reference, got);
    if n == 0 {
        "PASS".into()
    } else {
        format!("FAIL ({n} bytes differ, max |d| {m})")
    }
}

fn fx_cases() -> Vec<(&'static str, Fx)> {
    vec![
        ("blur r16", Fx::Blur { radius: 16.0 }),
        (
            "shadow 12,14 r24",
            Fx::Shadow { dx: 12.0, dy: 14.0, radius: 24.0, rgba: [0, 0, 0, 255], opacity: 0.6 },
        ),
        (
            "glow 0,0 r30",
            Fx::Shadow { dx: 0.0, dy: 0.0, radius: 30.0, rgba: [255, 240, 120, 255], opacity: 0.9 },
        ),
    ]
}

fn bases() -> Vec<(&'static str, T)> {
    vec![
        ("integer base", T { x: 960.0, y: 540.0, rot: 0.0, scale: 1.0, opacity: 1.0 }),
        ("fractional base", T { x: 960.37, y: 540.81, rot: 0.0, scale: 1.0, opacity: 1.0 }),
        ("rotated 7deg base", T { x: 960.37, y: 540.81, rot: 7.0, scale: 1.0, opacity: 1.0 }),
    ]
}

fn changes(b: &T) -> Vec<(&'static str, T)> {
    vec![
        ("same transform", *b),
        ("translate +37,-23 (integer)", T { x: b.x + 37.0, y: b.y - 23.0, ..*b }),
        ("translate +0.5,+0.25 (subpixel)", T { x: b.x + 0.5, y: b.y + 0.25, ..*b }),
        ("opacity 0.6", T { opacity: 0.6, ..*b }),
        ("translate +37,-23 and opacity 0.6", T { x: b.x + 37.0, y: b.y - 23.0, opacity: 0.6, ..*b }),
        ("scale x1.1 (control)", T { scale: b.scale * 1.1, ..*b }),
    ]
}

fn bit_identity() {
    println!("## Bit identity vs the live save_layer draw (1920x1080 RGBA8888 premul)\n");
    let font = make_font(110.0);
    let mut s = new_surface();
    let nearest = SamplingOptions::default();
    let linear = SamplingOptions::from(FilterMode::Linear);
    for (fx_name, fx) in fx_cases() {
        let f = filter(fx);
        let pic = record(&font, &f);
        for (base_name, base) in bases() {
            let snap = snapshot(&font, &f, &base);
            for (change_name, t) in changes(&base) {
                let reference = live(&mut s, &font, fx, &t);
                // Live twice: the reference itself must be reproducible.
                let again = live(&mut s, &font, fx, &t);
                let a = reused_filter(&mut s, &font, &f, &t);
                let e = bounded(&mut s, &font, fx, &t);
                let b = picture(&mut s, &pic, &t);
                let c_layer_n = redraw(&mut s, &snap, &t, Alpha::Layer, nearest);
                let c_layer_l = redraw(&mut s, &snap, &t, Alpha::Layer, linear);
                let c_paint_n = redraw(&mut s, &snap, &t, Alpha::Paint, nearest);
                println!("{fx_name} | {base_name} | {change_name}");
                println!("  live repeated             : {}", verdict(&reference, &again));
                println!("  A reused ImageFilter      : {}", verdict(&reference, &a));
                println!("  (side) save_layer bounds hint : {}", verdict(&reference, &e));
                println!("  B SkPicture playback      : {}", verdict(&reference, &b));
                println!("  C snapshot, alpha layer, nearest : {}", verdict(&reference, &c_layer_n));
                println!("  C snapshot, alpha layer, linear  : {}", verdict(&reference, &c_layer_l));
                println!("  C snapshot, paint alpha, nearest : {}", verdict(&reference, &c_paint_n));
            }
        }
        // Memory, per cached element, for this effect at the integer base.
        let snap = snapshot(&font, &f, &bases()[0].1);
        let info = snap.image.image_info();
        println!(
            "MEM {fx_name}: snapshot {}x{} = {} bytes; full-frame snapshot would be {} bytes; \
             SkPicture approximate_bytes_used = {} bytes ({} ops)\n",
            info.width(),
            info.height(),
            info.compute_min_byte_size(),
            (W * H * 4),
            pic.approximate_bytes_used(),
            pic.approximate_op_count(),
        );
    }
}

/// Candidate D, Skia's own caches. Bytes are checked with the caches warm vs purged before
/// every frame; timing shows whether a reused filter ever hits the image-filter cache across
/// `save_layer` frames, against `Image::with_filter` on a *stable* source image, whose cache
/// key (filter id, matrix, clip, source gen id) can repeat.
fn caches() {
    println!("## Skia's own caches\n");
    let font = make_font(110.0);
    let mut s = new_surface();
    let fx = fx_cases()[0].1;
    let t = bases()[1].1;
    let warm = live(&mut s, &font, fx, &t);
    graphics::purge_all_caches();
    let cold = live(&mut s, &font, fx, &t);
    println!("purge_all_caches before the frame vs warm: {}", verdict(&warm, &cold));

    let f = filter(fx);
    let n = 40;
    let time = |label: &str, mut run: Box<dyn FnMut()>| {
        let mut v = Vec::with_capacity(n);
        for _ in 0..n {
            let t0 = Instant::now();
            run();
            v.push(t0.elapsed().as_secs_f64() * 1000.0);
        }
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!("  {label}: median {:.3} ms (min {:.3}, max {:.3})", v[n / 2], v[0], v[n - 1]);
    };
    {
        let mut s1 = new_surface();
        let font = make_font(110.0);
        time("live, fresh filter per frame       ", Box::new(move || {
            live(&mut s1, &font, fx, &t);
        }));
    }
    {
        let mut s1 = new_surface();
        let font = make_font(110.0);
        time("live, save_layer bounds hint       ", Box::new(move || {
            bounded(&mut s1, &font, fx, &t);
        }));
    }
    {
        let mut s1 = new_surface();
        let font = make_font(110.0);
        let f = f.clone();
        time("live, one reused filter            ", Box::new(move || {
            reused_filter(&mut s1, &font, &f, &t);
        }));
    }
    {
        let mut s1 = new_surface();
        let font = make_font(110.0);
        let pic = record(&font, &f);
        time("SkPicture playback                 ", Box::new(move || {
            picture(&mut s1, &pic, &t);
        }));
    }
    {
        let mut s1 = new_surface();
        let font = make_font(110.0);
        let snap = snapshot(&font, &f, &t);
        time("snapshot redraw                    ", Box::new(move || {
            redraw(&mut s1, &snap, &t, Alpha::Layer, SamplingOptions::default());
        }));
    }
    {
        let mut s1 = new_surface();
        time("background only (floor)            ", Box::new(move || {
            background(s1.canvas());
            pixels(&mut s1);
        }));
    }
    // A stable source image through make_with_filter: same filter id, same source gen id.
    let mut src = new_surface();
    src.canvas().clear(Color::TRANSPARENT);
    content(src.canvas(), &make_font(110.0));
    let src_img = src.image_snapshot();
    let clip = IRect::from_wh(W, H);
    let subset = IRect::from_wh(W, H);
    {
        let f = f.clone();
        let src_img = src_img.clone();
        time("Image::with_filter, same filter+src", Box::new(move || {
            let _ = images::make_with_filter(src_img.clone(), &f, subset, clip);
        }));
    }
    {
        let src_img = src_img.clone();
        time("Image::with_filter, fresh filter   ", Box::new(move || {
            let _ = images::make_with_filter(src_img.clone(), &filter(fx), subset, clip);
        }));
    }
    println!();
}

/// An animated multi-element scene, filters rebuilt every frame as today.
fn scene(s: &mut Surface, font: &Font, small: &Font, i: usize) -> u64 {
    let c = s.canvas();
    background(c);
    let fi = i as f64;
    for k in 0..8 {
        let kf = k as f64;
        let fx = match k % 3 {
            0 => Fx::Blur { radius: 4.0 + (fi * 0.37 + kf).rem_euclid(20.0) },
            1 => Fx::Shadow { dx: 6.0, dy: 8.0, radius: 18.0, rgba: [0, 0, 0, 255], opacity: 0.7 },
            _ => Fx::Shadow { dx: 0.0, dy: 0.0, radius: 26.0, rgba: [120, 220, 255, 255], opacity: 0.9 },
        };
        let t = T {
            x: 200.0 + kf * 190.0 + fi * 3.17,
            y: 140.0 + kf * 105.0 + (fi * 0.21).sin() * 40.0,
            rot: if k % 2 == 0 { fi * 0.7 } else { 0.0 },
            scale: 0.4 + 0.05 * kf + 0.002 * fi,
            opacity: 0.55 + 0.45 * ((fi * 0.13 + kf).cos()).abs(),
        };
        let f = filter(fx);
        element(c, &t, |c| through(c, &f, |c| content(c, if k % 2 == 0 { font } else { small })));
    }
    hash(&pixels(s))
}

fn determinism() {
    println!("## Determinism: K threads, own surface each, vs one sequential surface\n");
    let frames = 96;
    let font = make_font(110.0);
    let small = make_font(64.0);
    let mut s = new_surface();
    let t0 = Instant::now();
    let seq: Vec<u64> = (0..frames).map(|i| scene(&mut s, &font, &small, i)).collect();
    println!("sequential: {frames} frames in {:.2} s", t0.elapsed().as_secs_f64());
    // A fresh surface per frame, same thread.
    let fresh: Vec<u64> = (0..frames)
        .map(|i| scene(&mut new_surface(), &font, &small, i))
        .collect();
    println!("fresh surface per frame, same thread: {}", if fresh == seq { "IDENTICAL" } else { "DIFFERENT" });

    for &k in &[2usize, 4, 8] {
        for &layout in &["interleaved", "chunked"] {
            graphics::purge_all_caches();
            let t0 = Instant::now();
            let handles: Vec<_> = (0..k)
                .map(|w| {
                    std::thread::spawn(move || {
                        let font = make_font(110.0);
                        let small = make_font(64.0);
                        let mut s = new_surface();
                        let mine: Vec<usize> = if layout == "interleaved" {
                            (0..frames).filter(|i| i % k == w).collect()
                        } else {
                            let per = frames.div_ceil(k);
                            (w * per..((w + 1) * per).min(frames)).collect()
                        };
                        mine.into_iter().map(|i| (i, scene(&mut s, &font, &small, i))).collect::<Vec<_>>()
                    })
                })
                .collect();
            let mut par = vec![0u64; frames];
            for h in handles {
                for (i, v) in h.join().unwrap() {
                    par[i] = v;
                }
            }
            let mismatches = par.iter().zip(&seq).filter(|(a, b)| a != b).count();
            println!(
                "K={k} {layout}: {} ({mismatches}/{frames} frames differ) in {:.2} s",
                if mismatches == 0 { "IDENTICAL" } else { "DIFFERENT" },
                t0.elapsed().as_secs_f64()
            );
        }
    }
    println!();
}

/// Where do the integer-translation misses land? Prints the bounding box of the differing
/// pixels for a range of integer deltas, with and without the filter.
fn probe() {
    println!("## Probe: integer translation of a snapshot, where the bytes differ\n");
    let font = make_font(110.0);
    let mut s = new_surface();
    let none = image_filters::offset((0.0, 0.0), None, None).unwrap();
    let mut cases: Vec<(&str, ImageFilter, Option<Fx>)> = vec![("no-op offset filter", none, None)];
    for (n, fx) in fx_cases() {
        cases.push((n, filter(fx), Some(fx)));
    }
    for (name, f, fx) in cases {
        for (base_name, base) in bases().into_iter().take(2) {
            let snap = snapshot(&font, &f, &base);
            for d in [(1.0, 0.0), (0.0, 1.0), (2.0, 0.0), (4.0, 4.0), (16.0, 0.0), (37.0, -23.0), (-200.0, 100.0)] {
                let t = T { x: base.x + d.0, y: base.y + d.1, ..base };
                let reference = match fx {
                    Some(fx) => live(&mut s, &font, fx, &t),
                    None => reused_filter(&mut s, &font, &f, &t),
                };
                let got = redraw(&mut s, &snap, &t, Alpha::Layer, SamplingOptions::default());
                let mut bb: Option<(usize, usize, usize, usize)> = None;
                let mut px = 0;
                for (i, (a, b)) in reference.chunks(4).zip(got.chunks(4)).enumerate() {
                    if a != b {
                        px += 1;
                        let (x, y) = (i % W as usize, i / W as usize);
                        bb = Some(match bb {
                            None => (x, y, x, y),
                            Some((l, t, r, b)) => (l.min(x), t.min(y), r.max(x), b.max(y)),
                        });
                    }
                }
                println!("{name} | {base_name} | d={d:?}: {px} px differ, bbox {bb:?} (snapshot at {:?})", snap.left_top);
            }
        }
    }
}

fn main() {
    let which = std::env::args().nth(1).unwrap_or_else(|| "all".into());
    if which == "probe" {
        probe();
    }
    if which == "all" || which == "bits" {
        bit_identity();
    }
    if which == "all" || which == "caches" {
        caches();
    }
    if which == "all" || which == "threads" {
        determinism();
    }
}
