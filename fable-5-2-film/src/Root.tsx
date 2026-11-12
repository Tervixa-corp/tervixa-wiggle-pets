import React from "react";
import { Composition } from "remotion";
import { Film, TOTAL_FRAMES } from "./video/Film";

export const Root: React.FC = () => (
  <Composition
    id="Fable52"
    component={Film}
    durationInFrames={TOTAL_FRAMES}
    fps={60}
    width={1920}
    height={1080}
  />
);
