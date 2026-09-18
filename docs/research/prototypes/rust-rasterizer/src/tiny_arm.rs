// #34's measured code, deliberately left as it was measured. The two lints
// below want this module restyled; it is not, because what this module *draws*
// is the thing under test, and a cleanup that shifts a pixel would invalidate
// the golden frames ADR-0010 keeps it here to be guarded by.
#![allow(clippy::map_entry, clippy::field_reassign_with_default)]

use crate::backend::{path_els, src_rect, Backend};
use crate::media::load_rgba;
use crate::text::{PathEl, PlacedGlyph, TextShaper};
use std::collections::HashMap;
use tiny_skia::{
    Color, FillRule, FilterQuality, Paint, Path, PathBuilder, Pattern, Pixmap, PixmapRef, Rect,
    SpreadMode, Transform,
};

pub struct TinyArm {
    pixmap: Pixmap,
    stills: HashMap<String, Pixmap>,
    paths: HashMap<(usize, u32, u32), Path>,
}

fn solid(c: [u8; 4]) -> Paint<'static> {
    let mut p = Paint::default();
    p.anti_alias = true;
    p.set_color_rgba8(c[0], c[1], c[2], c[3]);
    p
}

/// Map a source rect in pattern space onto a destination rect.
fn fit(sx: f64, sy: f64, sw: f64, sh: f64, d: (f64, f64, f64, f64)) -> Transform {
    Transform::from_translate(d.0 as f32, d.1 as f32)
        .pre_scale((d.2 / sw) as f32, (d.3 / sh) as f32)
        .pre_translate(-sx as f32, -sy as f32)
}

impl TinyArm {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            pixmap: Pixmap::new(w, h).expect("pixmap"),
            stills: HashMap::new(),
            paths: HashMap::new(),
        }
    }

    fn still_pixmap(&mut self, src: &str) -> &Pixmap {
        if !self.stills.contains_key(src) {
            let (rgba, iw, ih) = load_rgba(src);
            let mut pm = Pixmap::new(iw, ih).expect("still pixmap");
            // tiny-skia pixmaps are premultiplied; these stills are opaque.
            let dst = pm.pixels_mut();
            for (i, px) in dst.iter_mut().enumerate() {
                let o = i * 4;
                *px = tiny_skia::ColorU8::from_rgba(rgba[o], rgba[o + 1], rgba[o + 2], rgba[o + 3])
                    .premultiply();
            }
            self.stills.insert(src.to_string(), pm);
        }
        &self.stills[src]
    }
}

impl Backend for TinyArm {
    fn name(&self) -> &'static str {
        "tiny-skia"
    }

    fn bg(&mut self, c: [u8; 4]) {
        self.pixmap.fill(Color::from_rgba8(c[0], c[1], c[2], c[3]));
    }

    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64, c: [u8; 4]) {
        if let Some(r) = Rect::from_xywh(x as f32, y as f32, w as f32, h as f32) {
            self.pixmap
                .fill_rect(r, &solid(c), Transform::identity(), None);
        }
    }

    fn still(
        &mut self,
        src: &str,
        crop: (f64, f64),
        s: (f64, f64, f64, f64),
        d: (f64, f64, f64, f64),
    ) {
        self.still_pixmap(src); // load-on-first-use, then split the borrow
        let Self { pixmap, stills, .. } = self;
        let pm = &stills[src];
        let (iw, ih) = (pm.width(), pm.height());
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
        let ts = fit(ox, oy, sw, sh, d);
        let mut paint = Paint::default();
        paint.shader = Pattern::new(
            pm.as_ref(),
            SpreadMode::Pad,
            FilterQuality::Bilinear,
            1.0,
            ts,
        );
        if let Some(r) = Rect::from_xywh(d.0 as f32, d.1 as f32, d.2 as f32, d.3 as f32) {
            pixmap.fill_rect(r, &paint, Transform::identity(), None);
        }
    }

    fn video(&mut self, rgba: &[u8], w: u32, h: u32, d: (f64, f64, f64, f64)) {
        // tiny-skia can view the decoded bytes in place — no copy — but only if
        // they are already premultiplied. Video frames are opaque, so they are.
        let view = PixmapRef::from_bytes(rgba, w, h).expect("frame view");
        let ts = fit(0.0, 0.0, w as f64, h as f64, d);
        let mut paint = Paint::default();
        paint.shader = Pattern::new(view, SpreadMode::Pad, FilterQuality::Bilinear, 1.0, ts);
        if let Some(r) = Rect::from_xywh(d.0 as f32, d.1 as f32, d.2 as f32, d.3 as f32) {
            self.pixmap
                .fill_rect(r, &paint, Transform::identity(), None);
        }
    }

    fn glyphs(&mut self, glyphs: &[PlacedGlyph], shaper: &TextShaper, c: [u8; 4]) {
        let paint = solid(c);
        for g in glyphs {
            if !self.paths.contains_key(&g.key) {
                let mut pb = PathBuilder::new();
                for el in path_els(shaper, g) {
                    match *el {
                        PathEl::Move(x, y) => pb.move_to(x, y),
                        PathEl::Line(x, y) => pb.line_to(x, y),
                        PathEl::Quad(cx, cy, x, y) => pb.quad_to(cx, cy, x, y),
                        PathEl::Cubic(a, b, cx, cy, x, y) => pb.cubic_to(a, b, cx, cy, x, y),
                        PathEl::Close => pb.close(),
                    }
                }
                let Some(p) = pb.finish() else {
                    continue;
                };
                self.paths.insert(g.key, p);
            }
            let Some(path) = self.paths.get(&g.key) else {
                continue;
            };
            self.pixmap.fill_path(
                path,
                &paint,
                FillRule::Winding,
                Transform::from_translate(g.x, g.y),
                None,
            );
        }
    }

    fn pixels(&mut self) -> &[u8] {
        // Opaque frame, so premultiplied == straight and this is zero-copy.
        self.pixmap.data()
    }
}
