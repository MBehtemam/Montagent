import { Composition } from "remotion";
import { Scene } from "./Scene.jsx";
import scene from "./scene.json";

export const RemotionRoot = () => (
  <Composition
    id="short"
    component={Scene}
    durationInFrames={Math.round(scene.duration * scene.fps)}
    fps={scene.fps}
    width={scene.width}
    height={scene.height}
    defaultProps={{}}
  />
);
