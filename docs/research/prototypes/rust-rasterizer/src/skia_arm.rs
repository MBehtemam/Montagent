use crate::backend::{path_els, src_rect, Backend};
use crate::media::load_rgba;
use crate::text::{PathEl, PlacedGlyph, TextShaper};
use skia_safe::canvas::SrcRectConstraint;
use skia_safe::{
    images, surfaces, AlphaType, Color, ColorType, Data, FilterMode, ISize, Image, ImageInfo,
    MipmapMode, Paint, Path, PathBuilder, Rect, SamplingOptions, Surface,
};
use std::collections::HashMap;

pub struct SkiaArm {
    surface: Surface,
    info: ImageInfo,
    stills: HashMap<String, (Image, u32, u32)>,
    paths: HashMap<(usize, u32, u32), Path>,
    pixels: Vec<u8>,
    w: u32,
    h: u32,
}

fn sampling() -> SamplingOptions {
    SamplingOptions::new(FilterMode::Linear, MipmapMode::None)
}

fn paint(c: [u8; 4]) -> Paint {
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_color(Color::from_argb(c[3], c[0], c[1], c[2]));
    p
}

impl SkiaArm {
    pub fn new(w: u32, h: u32) -> Self {
        let info = ImageInfo::new(
            ISize::new(w as i32, h as i32),
            ColorType::RGBA8888,
            AlphaType::Premul,
            None,
        );
        let surface = surfaces::raster(&info, None, None).expect("raster surface");
        Self {
            surface,
            info,
            stills: HashMap::new(),
            paths: HashMap::new(),
            pixels: vec![0u8; (w as usize) * (h as usize) * 4],
            w,
            h,
        }
    }

    fn still_image(&mut self, src: &str) -> (Image, u32, u32) {
        if let Some(v) = self.stills.get(src) {
            return v.clone();
        }
        let (rgba, iw, ih) = load_rgba(src);
        let info = ImageInfo::new(
            ISize::new(iw as i32, ih as i32),
            ColorType::RGBA8888,
            AlphaType::Unpremul,
            None,
        );
        let data = Data::new_copy(&rgba);
        let img = images::raster_from_data(&info, data, iw as usize * 4).expect("raster image");
        self.stills.insert(src.to_string(), (img, iw, ih));
        self.stills[src].clone()
    }
}

impl Backend for SkiaArm {
    fn name(&self) -> &'static str {
        "skia-safe"
    }

    fn bg(&mut self, c: [u8; 4]) {
        self.surface
            .canvas()
            .clear(Color::from_argb(c[3], c[0], c[1], c[2]));
    }

    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64, c: [u8; 4]) {
        self.surface.canvas().draw_rect(
            Rect::from_xywh(x as f32, y as f32, w as f32, h as f32),
            &paint(c),
        );
    }

    fn still(&mut self, src: &str, crop: (f64, f64), s: (f64, f64, f64, f64), d: (f64, f64, f64, f64)) {
        let (img, iw, ih) = self.still_image(src);
        let whole = crop.0 <= 0.0;
        let (ox, oy) = if whole {
            (0.0, 0.0)
        } else {
            src_rect(iw, ih, crop.0, crop.1, s.0, s.1)
        };
        let (sw, sh) = if whole {
            (iw as f64, ih as f64)
        } else {
            (s.2, s.3)
        };
        let src_r = Rect::from_xywh(ox as f32, oy as f32, sw as f32, sh as f32);
        let dst_r = Rect::from_xywh(d.0 as f32, d.1 as f32, d.2 as f32, d.3 as f32);
        let p = Paint::default();
        self.surface.canvas().draw_image_rect_with_sampling_options(
            &img,
            Some((&src_r, SrcRectConstraint::Strict)),
            dst_r,
            sampling(),
            &p,
        );
    }

    fn video(&mut self, rgba: &[u8], w: u32, h: u32, d: (f64, f64, f64, f64)) {
        // skia needs the frame in an SkImage: `raster_from_data` copies it.
        let info = ImageInfo::new(
            ISize::new(w as i32, h as i32),
            ColorType::RGBA8888,
            AlphaType::Unpremul,
            None,
        );
        let img = images::raster_from_data(&info, Data::new_copy(rgba), w as usize * 4)
            .expect("video frame image");
        let dst_r = Rect::from_xywh(d.0 as f32, d.1 as f32, d.2 as f32, d.3 as f32);
        let p = Paint::default();
        self.surface.canvas().draw_image_rect_with_sampling_options(
            &img,
            None,
            dst_r,
            sampling(),
            &p,
        );
    }

    fn glyphs(&mut self, glyphs: &[PlacedGlyph], shaper: &TextShaper, c: [u8; 4]) {
        let p = paint(c);
        for g in glyphs {
            if !self.paths.contains_key(&g.key) {
                let mut pb = PathBuilder::new();
                for el in path_els(shaper, g) {
                    match *el {
                        PathEl::Move(x, y) => {
                            pb.move_to((x, y));
                        }
                        PathEl::Line(x, y) => {
                            pb.line_to((x, y));
                        }
                        PathEl::Quad(cx, cy, x, y) => {
                            pb.quad_to((cx, cy), (x, y));
                        }
                        PathEl::Cubic(a, b, cx, cy, x, y) => {
                            pb.cubic_to((a, b), (cx, cy), (x, y));
                        }
                        PathEl::Close => {
                            pb.close();
                        }
                    }
                }
                self.paths.insert(g.key, pb.detach());
            }
            let path = &self.paths[&g.key];
            let canvas = self.surface.canvas();
            canvas.save();
            canvas.translate((g.x, g.y));
            canvas.draw_path(path, &p);
            canvas.restore();
        }
    }

    fn pixels(&mut self) -> &[u8] {
        let row = self.w as usize * 4;
        let info = self.info.clone();
        self.surface
            .read_pixels(&info, &mut self.pixels, row, (0, 0));
        let _ = self.h;
        &self.pixels
    }
}
