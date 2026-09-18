// #34: skia-safe vs tiny-skia under a Rust host, with real video decode.
// Throwaway. Nothing here is a proposed design.
mod backend;
mod media;
mod ops;
mod repo;
mod scene;
mod skia_arm;
mod text;
mod tiny_arm;

use backend::Backend;
use media::{Decoder, Encoder};
use ops::Op;
use scene::Scene;
use std::collections::HashMap;
use std::time::Instant;
use text::TextShaper;

struct Args(HashMap<String, String>);

impl Args {
    fn parse() -> Self {
        let mut m = HashMap::new();
        for a in std::env::args().skip(1) {
            let a = a.trim_start_matches("--").to_string();
            match a.split_once('=') {
                Some((k, v)) => m.insert(k.to_string(), v.to_string()),
                None => m.insert(a, "1".into()),
            };
        }
        Self(m)
    }
    fn get(&self, k: &str) -> Option<&str> {
        self.0.get(k).map(|s| s.as_str())
    }
    fn num(&self, k: &str, d: f64) -> f64 {
        self.get(k).and_then(|v| v.parse().ok()).unwrap_or(d)
    }
    fn on(&self, k: &str) -> bool {
        self.0.contains_key(k)
    }
}

fn audio_args(sc: &Scene, from: f64, to: f64) -> Vec<String> {
    let mut ins = Vec::new();
    let mut fc = Vec::new();
    let mut labels = String::new();
    for (j, a) in sc.audio.iter().enumerate() {
        ins.push("-i".to_string());
        ins.push(a.src.clone());
        let delay = ((a.at - from) * 1000.0).max(0.0).round() as i64;
        fc.push(format!(
            "[{}:a]adelay={delay}|{delay},aformat=sample_fmts=fltp:sample_rates=24000:channel_layouts=mono[a{j}]",
            j + 1
        ));
        labels.push_str(&format!("[a{j}]"));
    }
    fc.push(format!(
        "{labels}amix=inputs={}:normalize=0:duration=longest,apad,atrim=0:{:.3}[aout]",
        sc.audio.len(),
        to - from
    ));
    let mut out = ins;
    out.push("-filter_complex".into());
    out.push(fc.join(";"));
    out.extend(["-map".into(), "0:v".into(), "-map".into(), "[aout]".into()]);
    out.extend([
        "-c:a".into(),
        "aac".into(),
        "-b:a".into(),
        "64k".into(),
        "-shortest".into(),
    ]);
    out
}

/// One live decoder per clip source, opened at the window start and pulled
/// forward. Sequential decode; a seek is a fresh process (that is the cost).
struct Decoders {
    open: HashMap<String, Decoder>,
    fps: f64,
    dw: u32,
    dh: u32,
    pub decode_ns: u128,
    pub open_ns: u128,
    pub opens: u32,
}

impl Decoders {
    fn frame(&mut self, src: &str, at: f64) -> Option<Vec<u8>> {
        if !self.open.contains_key(src) {
            let t0 = Instant::now();
            let d = Decoder::open(src, at, self.fps, self.dw, self.dh);
            self.open.insert(src.to_string(), d);
            self.open_ns += t0.elapsed().as_nanos();
            self.opens += 1;
        }
        let t0 = Instant::now();
        let d = self.open.get_mut(src).unwrap();
        let f = d.next().map(|b| b.to_vec());
        self.decode_ns += t0.elapsed().as_nanos();
        f
    }
}

/// Ablations, to split the per-frame cost. Set by --notext / --noimage.
pub struct Skip {
    pub text: bool,
    pub image: bool,
}

fn draw_frame(
    be: &mut dyn Backend,
    shaper: &mut TextShaper,
    decs: &mut Decoders,
    sc: &Scene,
    frame: i64,
    k: f64,
    skip: &Skip,
) {
    for o in ops::ops_at(sc, frame, k) {
        match o {
            Op::Bg { colour } => be.bg(colour),
            Op::Rect { x, y, w, h, colour } => be.rect(x, y, w, h, colour),
            Op::Image {
                src,
                whole,
                crop_w,
                crop_h,
                sx,
                sy,
                sw,
                sh,
                dx,
                dy,
                dw,
                dh,
            } => {
                if skip.image {
                    continue;
                }
                let crop = if whole { (0.0, 0.0) } else { (crop_w, crop_h) };
                be.still(&src, crop, (sx, sy, sw, sh), (dx, dy, dw, dh));
            }
            Op::Video {
                src,
                at,
                dx,
                dy,
                dw,
                dh,
            } => {
                if let Some(f) = decs.frame(&src, at) {
                    be.video(&f, decs.dw, decs.dh, (dx, dy, dw, dh));
                }
            }
            Op::Text {
                x,
                y,
                align,
                colour,
                size,
                lines,
            } => {
                if skip.text {
                    continue;
                }
                let g = shaper.place(&lines, size, x, y, align);
                be.glyphs(&g, shaper, colour);
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    let scene_path = args.get("scene").unwrap_or("scene.json").to_string();
    let scene_file = repo::in_harness(&scene_path);
    let mut sc: Scene =
        serde_json::from_str(&std::fs::read_to_string(&scene_file).expect("scene.json")).unwrap();
    // Scene asset paths are repository-relative so the harness runs from any
    // checkout, on any of ADR-0064's six targets (#189).
    sc.resolve_paths();
    let sc = sc;
    let k = args.num("scale", 1.0);
    let (w, h) = ((sc.width as f64 * k) as u32, (sc.height as f64 * k) as u32);
    let from = args.num("from", 0.0);
    let to = args.num("to", sc.duration);
    let out = args.get("out").unwrap_or("out.mp4").to_string();
    let which = args.get("backend").unwrap_or("skia").to_string();

    let mut be: Box<dyn Backend> = match which.as_str() {
        "tiny" => Box::new(tiny_arm::TinyArm::new(w, h)),
        _ => Box::new(skia_arm::SkiaArm::new(w, h)),
    };
    let font_path = args
        .get("font")
        .map(|f| std::path::PathBuf::from(repo::resolve(f)))
        .unwrap_or_else(repo::vendored_font);
    let mut shaper = TextShaper::new(&font_path);
    let mut decs = Decoders {
        open: HashMap::new(),
        fps: sc.fps,
        // clips are decoded at their dest size — the scale the compositor needs
        dw: sc.clips.first().map(|c| (c.dw * k) as u32).unwrap_or(2),
        dh: sc.clips.first().map(|c| (c.dh * k) as u32).unwrap_or(2),
        decode_ns: 0,
        open_ns: 0,
        opens: 0,
    };

    let skip = Skip {
        text: args.on("notext"),
        image: args.on("noimage"),
    };
    let f0 = (from * sc.fps).round() as i64;
    let f1 = (to * sc.fps).round() as i64;

    // --- a single frame: the agent self-verification loop
    if args.on("still") {
        let t0 = Instant::now();
        draw_frame(&mut *be, &mut shaper, &mut decs, &sc, f0, k, &skip);
        let raster = t0.elapsed();
        let t1 = Instant::now();
        let px = be.pixels().to_vec();
        let read = t1.elapsed();
        let t2 = Instant::now();
        let enc = image::RgbaImage::from_raw(w, h, px).unwrap();
        enc.save(&out).unwrap();
        let encode = t2.elapsed();
        println!(
            "{} still t={from} font={} raster={:.1}ms readback={:.1}ms encode={:.1}ms total={:.1}ms",
            be.name(),
            shaper.family(),
            raster.as_secs_f64() * 1e3,
            read.as_secs_f64() * 1e3,
            encode.as_secs_f64() * 1e3,
            t0.elapsed().as_secs_f64() * 1e3
        );
        return;
    }

    // --- resident per-frame cost, no encoder, no process churn
    if args.on("bench") {
        let n = args.num("bench", 60.0) as i64;
        draw_frame(&mut *be, &mut shaper, &mut decs, &sc, f0, k, &skip); // warm caches
        let t0 = Instant::now();
        for i in 0..n {
            draw_frame(&mut *be, &mut shaper, &mut decs, &sc, f0 + i, k, &skip);
            std::hint::black_box(be.pixels());
        }
        let el = t0.elapsed().as_secs_f64();
        println!(
            "{} resident {n} frames at {w}x{h} from t={from}: {:.2}ms/frame ({:.1} fps)",
            be.name(),
            el / n as f64 * 1e3,
            n as f64 / el
        );
        return;
    }

    // --- a render window
    let audio = if args.on("noaudio") {
        Vec::new()
    } else {
        audio_args(&sc, from, to)
    };
    let t0 = Instant::now();
    let mut encoder = Encoder::open(w, h, sc.fps, &out, audio);
    let mut raster_ns = 0u128;
    let mut pipe_ns = 0u128;
    for f in f0..f1 {
        let a = Instant::now();
        draw_frame(&mut *be, &mut shaper, &mut decs, &sc, f, k, &skip);
        raster_ns += a.elapsed().as_nanos();
        let b = Instant::now();
        let px = be.pixels();
        encoder.write(px);
        pipe_ns += b.elapsed().as_nanos();
    }
    encoder.finish();
    let el = t0.elapsed().as_secs_f64();
    println!(
        "{} {}x{} [{from}..{to}] {} frames: wall={:.2}s raster={:.2}s pipe={:.2}s decode={:.2}s (opens={}, open={:.2}s) => {:.1}x realtime",
        be.name(),
        w,
        h,
        f1 - f0,
        el,
        raster_ns as f64 / 1e9,
        pipe_ns as f64 / 1e9,
        decs.decode_ns as f64 / 1e9,
        decs.opens,
        decs.open_ns as f64 / 1e9,
        (to - from) / el
    );
}
