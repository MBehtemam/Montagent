// DIY headless Chrome + FFmpeg: own the frame loop, capture frames over CDP,
// assemble with ffmpeg. Throwaway measurement code for #6.
//
// --workers=N   N concurrent tabs, each taking a contiguous block of frames
// --format=     png (lossless) | jpeg (what Remotion defaults to for video)
import puppeteer from "puppeteer";
import { spawn } from "node:child_process";
import { readFileSync, writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { createServer } from "node:http";
import { extname, join } from "node:path";
import { tmpdir } from "node:os";

const scene = JSON.parse(readFileSync(new URL("./scene.json", import.meta.url)));
const args = Object.fromEntries(process.argv.slice(2).map(a => {
  const [k, v] = a.replace(/^--/, "").split("=");
  return [k, v === undefined ? true : v];
}));
const from = Number(args.from ?? 0), to = Number(args.to ?? scene.duration);
const still = !!args.still, out = args.out ?? "out.mp4";
const format = args.format ?? "png";
const workers = Number(args.workers ?? 1);
const t0 = Date.now();
const mark = (l) => { if (args.timings) console.error(`  ${l}: ${((Date.now() - t0) / 1000).toFixed(2)}s`); };

const dir = new URL(".", import.meta.url).pathname;
const server = createServer((req, res) => {
  const path = req.url.split("?")[0];
  const f = path.startsWith("/fx/") ? decodeURIComponent(path.slice(3))
          : join(dir, path === "/" ? "scene.html" : path);
  const type = { ".html": "text/html", ".js": "text/javascript",
                 ".json": "application/json", ".png": "image/png" }[extname(f)];
  if (!type) { res.writeHead(404); res.end(); return; }
  res.writeHead(200, { "content-type": type });
  res.end(readFileSync(f));
}).listen(0);
const port = server.address().port;

const LAUNCH = {
  headless: true,
  args: ["--no-sandbox", "--font-render-hinting=none", "--disable-lcd-text",
         "--hide-scrollbars", "--force-device-scale-factor=1",
         // background tabs are throttled, which is what makes multi-tab capture
         // slower than a single tab; these are the flags that turn that off
         "--disable-background-timer-throttling",
         "--disable-backgrounding-occluded-windows",
         "--disable-renderer-backgrounding"],
};
const browsers = [];
async function getBrowser() {
  // --isolate gives every worker its own browser process rather than a tab
  const b = await puppeteer.launch(LAUNCH);
  browsers.push(b);
  return b;
}
const browser = await getBrowser();
mark("browser launched");

const clip = { x: 0, y: 0, width: scene.width, height: scene.height, scale: 1 };

async function newWorker(i = 0) {
  const b = (args.isolate && i > 0) ? await getBrowser() : browser;
  const page = await b.newPage();
  await page.setViewport({ width: scene.width, height: scene.height, deviceScaleFactor: 1 });
  await page.goto(`http://localhost:${port}/scene.html`, { waitUntil: "networkidle0" });
  await page.waitForFunction("window.__ready === true");
  await page.evaluateHandle("document.fonts.ready");
  const cdp = await page.createCDPSession();
  return async (frame) => {
    await page.evaluate((n) => window.renderFrame(n), frame);
    const { data } = await cdp.send("Page.captureScreenshot", {
      format, clip, ...(format === "jpeg" ? { quality: 90 } : {}),
      captureBeyondViewport: false, fromSurface: true,
    });
    return Buffer.from(data, "base64");
  };
}

if (still) {
  const capture = await newWorker();
  mark("page ready");
  writeFileSync(out, await capture(Math.round(from * scene.fps)));
  mark("frame captured");
  await Promise.all(browsers.map(b => b.close())); server.close();
  process.exit(0);
}

const f0 = Math.round(from * scene.fps), f1 = Math.round(to * scene.fps);
const shots = await Promise.all(Array.from({ length: workers }, (_, i) => newWorker(i)));
mark("pages ready");

// each worker owns a contiguous block; frames land as numbered files, then ffmpeg
// assembles them — the same architecture Remotion uses.
const tmp = mkdtempSync(join(tmpdir(), "diy-frames-"));
const total = f1 - f0;
const per = Math.ceil(total / workers);
mark("frame loop start");
await Promise.all(shots.map(async (capture, w) => {
  const a = f0 + w * per, b = Math.min(f1, a + per);
  for (let f = a; f < b; f++) {
    writeFileSync(join(tmp, `${String(f - f0).padStart(6, "0")}.${format}`), await capture(f));
  }
}));
mark("frames done");

const ins = [], fc = [], labels = [];
scene.audio.forEach((a, j) => {
  ins.push("-i", a.src);
  const d = Math.max(0, Math.round((a.at - from) * 1000));
  fc.push(`[${j + 1}:a]adelay=${d}|${d},aformat=sample_fmts=fltp:sample_rates=24000:channel_layouts=mono[a${j}]`);
  labels.push(`[a${j}]`);
});
fc.push(`${labels.join("")}amix=inputs=${scene.audio.length}:normalize=0:duration=longest,apad,atrim=0:${(to - from).toFixed(3)}[aout]`);

await new Promise(r => spawn("ffmpeg", [
  "-y", "-hide_banner", "-loglevel", "error",
  "-framerate", String(scene.fps), "-i", join(tmp, `%06d.${format}`),
  ...ins, "-filter_complex", fc.join(";"), "-map", "0:v", "-map", "[aout]",
  "-c:v", "libx264", "-preset", "medium", "-crf", "23", "-pix_fmt", "yuv420p",
  "-c:a", "aac", "-b:a", "64k", "-shortest", out,
], { stdio: ["ignore", "inherit", "inherit"] }).on("close", r));
mark("encoded");
rmSync(tmp, { recursive: true, force: true });
await Promise.all(browsers.map(b => b.close())); server.close();
