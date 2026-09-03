// skia-canvas resident: cost per frame once the process is up and images decoded.
import { Canvas, loadImage } from "skia-canvas";
import { readFileSync } from "node:fs";
import { opsAt, anchors } from "../ops.js";
const scene = JSON.parse(readFileSync(new URL("../scene.json", import.meta.url)));
const t = () => performance.now();
let t0 = t();
const images = new Map();
for (const src of [...scene.spans.map(s => s.image), scene.badge.src])
  if (!images.has(src)) images.set(src, await loadImage(src));
console.log(`decode images: ${((t() - t0) / 1000).toFixed(2)}s`);
const canvas = new Canvas(scene.width, scene.height);
const ctx = canvas.getContext("2d");
function draw(frame) {
  for (const o of opsAt(scene, frame)) {
    if (o.op === "bg") { ctx.fillStyle = o.colour; ctx.fillRect(0, 0, scene.width, scene.height); }
    else if (o.op === "rect") { ctx.fillStyle = o.colour; ctx.fillRect(o.x, o.y, o.w, o.h); }
    else if (o.op === "image") {
      const im = images.get(o.src);
      if (o.whole) ctx.drawImage(im, o.dx, o.dy, o.dw, o.dh);
      else {
        const ox = (im.width - o.cropW) / 2, oy = (im.height - o.cropH) / 2;
        ctx.drawImage(im, ox + o.sx, oy + o.sy, o.sw, o.sh, o.dx, o.dy, o.dw, o.dh);
      }
    } else if (o.op === "text") {
      const { horiz, vert } = anchors(o.align);
      ctx.font = `${o.bold ? "bold " : ""}${o.size}px "${o.font}"`;
      ctx.fillStyle = o.colour; ctx.textAlign = horiz; ctx.textBaseline = "middle";
      const lh = o.size * 1.2, block = (o.lines.length - 1) * lh;
      const y0 = vert === "top" ? o.y + o.size / 2 : vert === "bottom" ? o.y - block - o.size / 2 : o.y - block / 2;
      o.lines.forEach((ln, i) => ctx.fillText(ln, o.x, y0 + i * lh));
    }
  }
}
for (let i = 0; i < 3; i++) draw(0);              // warm
for (const label of ["raw", "png"]) {
  for (const frame of [0, 1200, 1650]) {
    const N = 10; const s = t();
    for (let i = 0; i < N; i++) { draw(frame); label === "png" ? canvas.toBufferSync("png") : canvas.toBufferSync("raw"); }
    console.log(`resident frame ${frame} -> ${label}: ${((t() - s) / N).toFixed(1)}ms`);
  }
}
