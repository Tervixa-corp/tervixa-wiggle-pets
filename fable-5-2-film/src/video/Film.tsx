import React from "react";
import { AbsoluteFill, Audio, Sequence, interpolate, staticFile, useCurrentFrame } from "remotion";
import { theme } from "../theme";
import { Backdrop } from "../fx/Backdrop";
import { Room } from "../fx/Room";
import { GateWeave, Grade, Grain, Sweep, Vignette } from "../fx/Overlays";
import { Thread } from "../components/Thread";
import { ColdOpen } from "../scenes/ColdOpen";
import { Positioning } from "../scenes/Positioning";
import { Benchmarks } from "../scenes/Benchmarks";
import { Proof } from "../scenes/Proof";
import { Economics } from "../scenes/Economics";
import { Lockup } from "../scenes/Lockup";

/**
 * The six act boundaries. 120 BPM at 60fps → one beat is exactly 30 frames, so
 * every cut lands on a beat and the score is generated against the same grid.
 *
 * 0 / 210 / 420 / 720 / 1200 / 1500 — the chart lost its stat-card montage
 * (M2) and Proof gained 120 frames (M1). Every cut is hard: there is no exit
 * fade anywhere in the film (M6).
 */
export const ACTS = [
  { id: "cold-open", from: 0, duration: 210 },
  { id: "positioning", from: 210, duration: 210 },
  { id: "benchmarks", from: 420, duration: 300 },
  { id: "proof", from: 720, duration: 480 },
  { id: "economics", from: 1200, duration: 300 },
  { id: "lockup", from: 1500, duration: 300 },
] as const;

export const TOTAL_FRAMES = ACTS.reduce((n, a) => n + a.duration, 0);

/** Opening lift from black — seven frames, no smear. */
const FadeIn: React.FC = () => {
  const frame = useCurrentFrame();
  const o = interpolate(frame, [0, 7], [1, 0], {
    easing: theme.ease.out,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  if (o <= 0) return null;
  return <AbsoluteFill style={{ backgroundColor: "#000", opacity: o, pointerEvents: "none" }} />;
};

const SCENES: React.FC[] = [ColdOpen, Positioning, Benchmarks, Proof, Economics, Lockup];

export const Film: React.FC = () => (
  <AbsoluteFill style={{ backgroundColor: theme.colors.void }}>
    {/* 0 — the space the film lives in (never cuts, only morphs) */}
    <Backdrop />

    {/* 0.5 — the room: the L2 surface for the current act. It sits above the
           Backdrop and BELOW the Thread, because the Thread has to be able to
           draw on a lit wall — but a wash of background must never hide it. */}
    <Room />

    {/* 1 — the through-line. BELOW the scenes, so a glyph always occludes it.
           This order is the law: Thread above a scene layer is a build failure. */}
    <Thread />

    {/* 2 — the six acts */}
    {ACTS.map((act, i) => {
      const Scene = SCENES[i];
      return (
        <Sequence key={act.id} from={act.from} durationInFrames={act.duration} name={act.id}>
          <Scene />
        </Sequence>
      );
    })}

    {/* 3 — light and stock */}
    <Sweep />
    <Grade />
    <FadeIn />

    {/* 4 — texture */}
    <Grain />
    <Vignette />
    <GateWeave />

    <Audio src={staticFile("audio/score.wav")} volume={0.92} />
  </AbsoluteFill>
);
