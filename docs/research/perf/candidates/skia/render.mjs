// skia-canvas backend: draw scene.json ops with the Canvas 2D API, stream raw
// RGBA into ffmpeg. Throwaway measurement code for #6.
import { Canvas, loadImage } from "skia-canvas";
import { spawn } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { opsAt, anchors } from "../ops.js";

const scene = JSON.parse(readFileSync(new URL("../scene.json", import.meta.url)));
const args = Object.fromEntries(process.argv.slice(2).map(a => {
  const [k, v] = a.replace(/^--/, "").split("=");
  return [k, v === undefined ? true : v];
}));

const from = Number(args.from ?? 0);
const to = Number(args.to ?? scene.duration);
const still = !!args.still;
const out = args.out ?? "out.mp4";

const images = new Map();
async function img(src) {
  if (!images.has(src)) images.set(src, await loadImage(src));
  return images.get(src);
}

const canvas = new Canvas(scene.width, scene.height);
const ctx = canvas.getContext("2d");

async function drawFrame(frame) {
  for (const o of opsAt(scene, frame)) {
    if (o.op === "bg") { ctx.fillStyle = o.colour; ctx.fillRect(0, 0, scene.width, scene.height); }
    else if (o.op === "rect") { ctx.fillStyle = o.colour; ctx.fillRect(o.x, o.y, o.w, o.h); }
    else if (o.op === "image") {
      const im = await img(o.src);
      if (o.whole) ctx.drawImage(im, o.dx, o.dy, o.dw, o.dh);
      else {
        // the ops give the source rect inside the centred cropW x cropH window
        const ox = (im.width - o.cropW) / 2, oy = (im.height - o.cropH) / 2;
        ctx.drawImage(im, ox + o.sx, oy + o.sy, o.sw, o.sh, o.dx, o.dy, o.dw, o.dh);
      }
    } else if (o.op === "text") {
      const { horiz, vert } = anchors(o.align);
      ctx.font = `${o.bold ? "bold " : ""}${o.size}px "${o.font}"`;
      ctx.fillStyle = o.colour;
      ctx.textAlign = horiz;
      ctx.textBaseline = "middle";
      const lh = o.size * 1.2;
      const block = (o.lines.length - 1) * lh;
      let y0 = vert === "top" ? o.y + o.size / 2 : vert === "bottom" ? o.y - block - o.size / 2 : o.y - block / 2;
      o.lines.forEach((ln, i) => ctx.fillText(ln, o.x, y0 + i * lh));
    }
  }
}

// warm the image cache and the font before timing anything downstream
await Promise.all([...scene.spans.map(s => img(s.image)), img(scene.badge.src)]);

if (still) {
  await drawFrame(Math.round(from * scene.fps));
  writeFileSync(out, await canvas.png);
  process.exit(0);
}

const f0 = Math.round(from * scene.fps), f1 = Math.round(to * scene.fps);
const ff = spawn("ffmpeg", [
  "-y", "-hide_banner", "-loglevel", "error",
  "-f", "rawvideo", "-pix_fmt", "rgba", "-s", `${scene.width}x${scene.height}`,
  "-r", String(scene.fps), "-i", "pipe:0",
  ...(args.noaudio ? [] : audioArgs()),
  "-c:v", "libx264", "-preset", "medium", "-crf", "23", "-pix_fmt", "yuv420p",
  ...(args.noaudio ? [] : ["-c:a", "aac", "-b:a", "64k", "-shortest"]),
  out,
], { stdio: ["pipe", "inherit", "inherit"] });

function audioArgs() {
  // same 12 placements the baseline mixed, trimmed to the window
  const ins = [], fc = [], labels = [];
  scene.audio.forEach((a, j) => {
    ins.push("-i", a.src);
    const delay = Math.max(0, Math.round((a.at - from) * 1000));
    fc.push(`[${j + 1}:a]adelay=${delay}|${delay},aformat=sample_fmts=fltp:sample_rates=24000:channel_layouts=mono[a${j}]`);
    labels.push(`[a${j}]`);
  });
  fc.push(`${labels.join("")}amix=inputs=${scene.audio.length}:normalize=0:duration=longest,apad,atrim=0:${(to - from).toFixed(3)}[aout]`);
  return [...ins, "-filter_complex", fc.join(";"), "-map", "0:v", "-map", "[aout]"];
}

for (let f = f0; f < f1; f++) {
  await drawFrame(f);
  const buf = canvas.toBufferSync("raw");
  if (!ff.stdin.write(buf)) await new Promise(r => ff.stdin.once("drain", r));
}
ff.stdin.end();
await new Promise(r => ff.on("close", r));
