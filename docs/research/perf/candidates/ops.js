// Shared layout math: scene + frame -> a flat list of draw ops.
// The three backends differ only in how they *execute* these ops, so any
// difference in the numbers is the renderer, not the scene.
// Throwaway.

export function opsAt(scene, frame) {
  const t = frame / scene.fps;
  const ops = [];

  ops.push({ op: "bg", colour: scene.background });

  // --- Ken Burns image for the span covering t
  for (const s of scene.spans) {
    if (t < s.start || t >= s.end) continue;
    const n = Math.round((t - s.start) * scene.fps);          // frame within the span
    const dir = s.zoomTo > s.zoomFrom ? 1 : -1;
    let z = s.zoomFrom + dir * s.zoomStep * n;
    z = dir > 0 ? Math.min(z, s.zoomTo) : Math.max(z, s.zoomTo);
    // centre crop of the already-cropped window, matching zoompan's x/y expressions
    const sw = s.cropW / z, sh = s.cropH / z;
    ops.push({
      op: "image", src: s.image,
      // source rect inside the full still: centre the cropW x cropH window, then the zoom
      cropW: s.cropW, cropH: s.cropH,
      sx: (s.cropW - sw) / 2, sy: (s.cropH - sh) / 2, sw, sh,
      dx: 0, dy: 0, dw: scene.width, dh: scene.cardH,
    });
    break;
  }

  ops.push({ op: "image", src: scene.badge.src, whole: true,
             dx: scene.badge.x, dy: scene.badge.y, dw: scene.badge.w, dh: scene.badge.h });

  // --- text and shapes, in ASS layer order
  const live = scene.events.filter(e => t >= e.start && t < e.end)
                           .sort((a, b) => a.layer - b.layer);
  for (const e of live) {
    if (e.kind === "rect") {
      ops.push({ op: "rect", x: e.x, y: e.y, w: e.w, h: e.h, colour: e.colour });
    } else {
      ops.push({ op: "text", x: e.x, y: e.y, align: e.align, colour: e.colour,
                 size: e.size, font: e.font, bold: e.bold, lines: e.lines });
    }
  }
  return ops;
}

// ASS \\an numpad alignment -> horizontal/vertical anchors
export function anchors(an) {
  const horiz = an % 3 === 1 ? "left" : an % 3 === 2 ? "center" : "right";
  const vert  = an >= 7 ? "top" : an >= 4 ? "middle" : "bottom";
  return { horiz, vert };
}
