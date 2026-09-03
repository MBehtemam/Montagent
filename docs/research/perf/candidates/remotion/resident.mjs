// Remotion with the bundle built and the browser already open — the "resident
// server" case, which is what an MCP server would actually hold.
import { bundle } from "@remotion/bundler";
import { renderStill, renderMedia, selectComposition, openBrowser } from "@remotion/renderer";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const args = Object.fromEntries(process.argv.slice(2).map(a => {
  const [k, v] = a.replace(/^--/, "").split("="); return [k, v === undefined ? true : v];
}));

const t = () => performance.now();
let t0 = t();
const serveUrl = await bundle({ entryPoint: join(here, "src/index.js") });
console.log(`bundle: ${((t() - t0) / 1000).toFixed(2)}s`);

t0 = t();
const browserInstance = await openBrowser("chrome");
console.log(`browser open: ${((t() - t0) / 1000).toFixed(2)}s`);

t0 = t();
const composition = await selectComposition({ serveUrl, id: "short", browserInstance });
console.log(`select composition: ${((t() - t0) / 1000).toFixed(2)}s`);

// resident single frames — this is the agent self-verification loop
for (const frame of [0, 1200, 1650, 0, 1200, 1650]) {
  t0 = t();
  await renderStill({ composition, serveUrl, output: `/tmp/rem-still-${frame}.png`,
                      frame, browserInstance, overwrite: true });
  console.log(`still frame ${frame}: ${((t() - t0) / 1000).toFixed(2)}s`);
}

if (args.windows) {
  for (const [a, b] of [[0, 299], [1200, 1499], [1650, 1955]]) {
    t0 = t();
    await renderMedia({ composition, serveUrl, codec: "h264",
      outputLocation: `/tmp/rem-win-${a}.mp4`, frameRange: [a, b],
      browserInstance, overwrite: true });
    console.log(`window ${a}-${b}: ${((t() - t0) / 1000).toFixed(2)}s`);
  }
}
process.exit(0);
