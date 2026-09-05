// Port of #6's ops.js: (scene, frame) -> a flat list of draw ops.
// Both backends execute *this* list, so a difference in the numbers is the
// rasterizer and not the scene. `scale` is the 4K pass (ADR-0003 generalised
// the scope; the budget was only ever written for 1080x1920).
use crate::scene::{colour, Event, Scene};

pub enum Op {
    Bg {
        colour: [u8; 4],
    },
    /// A still, sampled from a source rect inside the centred cropW x cropH window.
    Image {
        src: String,
        whole: bool,
        crop_w: f64,
        crop_h: f64,
        sx: f64,
        sy: f64,
        sw: f64,
        sh: f64,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
    },
    /// One decoded video frame, blitted whole into a dest rect.
    Video {
        src: String,
        /// seconds into the source file
        at: f64,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
    },
    Rect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        colour: [u8; 4],
    },
    Text {
        x: f64,
        y: f64,
        align: u8,
        colour: [u8; 4],
        size: f64,
        font: String,
        lines: Vec<String>,
    },
}

/// ASS \an numpad alignment -> (horizontal, vertical) anchors, as in ops.js.
pub fn anchors(an: u8) -> (u8, u8) {
    let horiz = match an % 3 {
        1 => 0, // left
        2 => 1, // center
        _ => 2, // right
    };
    let vert = if an >= 7 {
        0 // top
    } else if an >= 4 {
        1 // middle
    } else {
        2 // bottom
    };
    (horiz, vert)
}

pub fn ops_at(scene: &Scene, frame: i64, k: f64) -> Vec<Op> {
    let t = frame as f64 / scene.fps;
    let mut ops = Vec::new();

    ops.push(Op::Bg {
        colour: colour(&scene.background),
    });

    // --- Ken Burns still for the span covering t
    for s in &scene.spans {
        if t < s.start || t >= s.end {
            continue;
        }
        let n = ((t - s.start) * scene.fps).round();
        let dir = if s.zoom_to > s.zoom_from { 1.0 } else { -1.0 };
        let mut z = s.zoom_from + dir * s.zoom_step * n;
        z = if dir > 0.0 {
            z.min(s.zoom_to)
        } else {
            z.max(s.zoom_to)
        };
        let (sw, sh) = (s.crop_w / z, s.crop_h / z);
        ops.push(Op::Image {
            src: s.image.clone(),
            whole: false,
            crop_w: s.crop_w,
            crop_h: s.crop_h,
            sx: (s.crop_w - sw) / 2.0,
            sy: (s.crop_h - sh) / 2.0,
            sw,
            sh,
            dx: 0.0,
            dy: 0.0,
            dw: scene.width as f64 * k,
            dh: scene.card_h * k,
        });
        break;
    }

    // --- video clips, over the still, in the same card
    for c in &scene.clips {
        if t < c.start || t >= c.end {
            continue;
        }
        ops.push(Op::Video {
            src: c.src.clone(),
            at: c.src_start + (t - c.start),
            dx: c.dx * k,
            dy: c.dy * k,
            dw: c.dw * k,
            dh: c.dh * k,
        });
    }

    ops.push(Op::Image {
        src: scene.badge.src.clone(),
        whole: true,
        crop_w: 0.0,
        crop_h: 0.0,
        sx: 0.0,
        sy: 0.0,
        sw: 0.0,
        sh: 0.0,
        dx: scene.badge.x * k,
        dy: scene.badge.y * k,
        dw: scene.badge.w * k,
        dh: scene.badge.h * k,
    });

    // --- text and shapes, in ASS layer order
    let mut live: Vec<&Event> = scene
        .events
        .iter()
        .filter(|e| {
            let (s, en) = e.window();
            t >= s && t < en
        })
        .collect();
    live.sort_by_key(|e| e.layer());

    for e in live {
        match e {
            Event::Rect {
                x, y, w, h, colour: c, ..
            } => ops.push(Op::Rect {
                x: x * k,
                y: y * k,
                w: w * k,
                h: h * k,
                colour: colour(c),
            }),
            Event::Text {
                x,
                y,
                align,
                colour: c,
                size,
                font,
                lines,
                ..
            } => ops.push(Op::Text {
                x: x * k,
                y: y * k,
                align: *align,
                colour: colour(c),
                size: size * k,
                font: font.clone(),
                lines: lines.clone(),
            }),
        }
    }
    ops
}
