// Where does the DIY Chrome frame loop actually spend its time?
import puppeteer from "puppeteer";
import { readFileSync } from "node:fs";
import { createServer } from "node:http";
import { extname, join } from "node:path";
const scene = JSON.parse(readFileSync(new URL("./scene.json", import.meta.url)));
const dir = new URL(".", import.meta.url).pathname;
const server = createServer((req, res) => {
  const path = req.url.split("?")[0];
  const f = path.startsWith("/fx/") ? decodeURIComponent(path.slice(3)) : join(dir, path === "/" ? "scene.html" : path);
  const type = { ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".png": "image/png" }[extname(f)];
  if (!type) { res.writeHead(404); res.end(); return; }
  res.writeHead(200, { "content-type": type }); res.end(readFileSync(f));
}).listen(0);
const port = server.address().port;
const browser = await puppeteer.launch({ headless: true, args: ["--no-sandbox", "--hide-scrollbars"] });
const page = await browser.newPage();
await page.setViewport({ width: scene.width, height: scene.height, deviceScaleFactor: 1 });
await page.goto(`http://localhost:${port}/scene.html`, { waitUntil: "networkidle0" });
await page.waitForFunction("window.__ready === true");
const cdp = await page.createCDPSession();
const clip = { x: 0, y: 0, width: scene.width, height: scene.height, scale: 1 };
const N = 60;
for (const fmt of ["png", "jpeg"]) {
  let tDom = 0, tCap = 0;
  for (let f = 0; f < N; f++) {
    let a = performance.now();
    await page.evaluate((n) => window.renderFrame(n), f);
    let b = performance.now(); tDom += b - a;
    await cdp.send("Page.captureScreenshot", { format: fmt, clip, ...(fmt === "jpeg" ? { quality: 90 } : {}) });
    tCap += performance.now() - b;
  }
  console.log(`${fmt}: dom ${(tDom/N).toFixed(1)}ms/frame  capture ${(tCap/N).toFixed(1)}ms/frame  total ${((tDom+tCap)/N).toFixed(1)}ms`);
}
await browser.close(); server.close();
