// Remotion composition rendering the shared scene.json through the shared ops.
// Throwaway measurement code for #6.
import { AbsoluteFill, Audio, Img, Sequence, staticFile, useCurrentFrame } from "remotion";
import { opsAt, anchors } from "./ops.js";
import scene from "./scene.json";

const FIX = "/Users/mohammedehtemam/projects/github/Montaget/fixtures/en-halloween-decorating";
const asset = (p) => staticFile("fx" + p.slice(FIX.length));

// natural sizes, so the Ken Burns maths matches the other two backends exactly
const NATURAL = { w: 1536, h: 2720 };

export const Scene = () => {
  const frame = useCurrentFrame();
  const ops = opsAt(scene, frame);
  const nodes = [];
  let bg = scene.background;

  ops.forEach((o, i) => {
    if (o.op === "bg") { bg = o.colour; }
    else if (o.op === "rect") {
      nodes.push(<div key={i} style={{ position: "absolute", left: o.x, top: o.y,
        width: o.w, height: o.h, background: o.colour }} />);
    } else if (o.op === "image") {
      if (o.whole) {
        nodes.push(<Img key={i} src={asset(o.src)} style={{ position: "absolute",
          left: o.dx, top: o.dy, width: o.dw, height: o.dh }} />);
      } else {
        const ox = (NATURAL.w - o.cropW) / 2, oy = (NATURAL.h - o.cropH) / 2;
        const k = o.dw / o.sw;
        nodes.push(
          <div key={i} style={{ position: "absolute", left: 0, top: 0,
            width: o.dw, height: o.dh, overflow: "hidden" }}>
            <Img src={asset(o.src)} style={{ position: "absolute",
              width: NATURAL.w * k, left: -(ox + o.sx) * k, top: -(oy + o.sy) * k }} />
          </div>);
      }
    } else if (o.op === "text") {
      const { horiz, vert } = anchors(o.align);
      const tx = horiz === "center" ? "-50%" : horiz === "right" ? "-100%" : "0";
      const ty = vert === "middle" ? "-50%" : vert === "bottom" ? "-100%" : "0";
      nodes.push(
        <div key={i} style={{ position: "absolute", left: o.x, top: o.y,
          transform: `translate(${tx},${ty})`, textAlign: horiz, color: o.colour,
          fontFamily: o.font, fontSize: o.size, fontWeight: o.bold ? "bold" : "normal",
          lineHeight: 1.2, whiteSpace: "pre" }}>
          {o.lines.map((l, j) => <div key={j}>{l}</div>)}
        </div>);
    }
  });

  return (
    <AbsoluteFill style={{ background: bg }}>
      {nodes}
      {scene.audio.map((a, j) => (
        <Sequence key={`a${j}`} from={Math.round(a.at * scene.fps)}>
          <Audio src={asset(a.src)} />
        </Sequence>
      ))}
    </AbsoluteFill>
  );
};
