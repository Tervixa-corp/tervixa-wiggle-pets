import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { theme } from "../theme";

type Phase = {
  at: number;
  ax: number;
  ay: number;
  ar: number;
  color: string;
  bx: number;
  by: number;
  br: number;
  color2: string;
  wash: number;
};

/**
 * A warm-grey bloom, never a clay bloom.
 *
 * Blur plus alpha over near-black integrates to ochre: the red channel survives,
 * the green half-survives, the blue dies. There is no opacity at which a blurred
 * clay field stops being brown. So the warm light in this film is #4A4038 — a
 * grey with a warm bias — and the only true clay in any frame is an object:
 * a bar, a numeral, a single word.
 */
const PHASES: Phase[] = [
  { at: 0, ax: 960, ay: 300, ar: 780, color: "#4A4038", bx: 620, by: 980, br: 620, color2: "#3A3330", wash: 0.055 },
  { at: 210, ax: 1180, ay: 420, ar: 900, color: "#454039", bx: 520, by: 840, br: 700, color2: "#33404A", wash: 0.058 },
  { at: 420, ax: 760, ay: 700, ar: 880, color: "#3A4450", bx: 1560, by: 240, br: 760, color2: "#46403A", wash: 0.05 },
  { at: 720, ax: 1500, ay: 280, ar: 820, color: "#46403A", bx: 380, by: 860, br: 700, color2: "#3A4450", wash: 0.055 },
  { at: 1200, ax: 960, ay: 980, ar: 900, color: "#3A4450", bx: 300, by: 200, br: 640, color2: "#46403A", wash: 0.042 },
  { at: 1500, ax: 960, ay: 620, ar: 1060, color: "#4A4038", bx: 960, by: 1080, br: 700, color2: "#3A3330", wash: 0.034 },
];

const pick = (frame: number): Phase => {
  let i = 0;
  while (i < PHASES.length - 2 && frame > PHASES[i + 1].at) i += 1;
  const a = PHASES[i];
  const b = PHASES[i + 1];
  const raw = (frame - a.at) / (b.at - a.at);
  const t = theme.ease.inOut(Math.min(1, Math.max(0, raw)));
  const l = (p: number, q: number) => interpolate(t, [0, 1], [p, q]);
  return {
    at: frame,
    ax: l(a.ax, b.ax),
    ay: l(a.ay, b.ay),
    ar: l(a.ar, b.ar),
    color: t < 0.5 ? a.color : b.color,
    bx: l(a.bx, b.bx),
    by: l(a.by, b.by),
    br: l(a.br, b.br),
    color2: t < 0.5 ? a.color2 : b.color2,
    wash: l(a.wash, b.wash),
  };
};

export const Backdrop: React.FC = () => {
  const frame = useCurrentFrame();
  const p = pick(frame);

  const dx = Math.sin(frame / 71) * 46;
  const dy = Math.cos(frame / 88) * 34;
  const dx2 = Math.cos(frame / 63) * 38;
  const dy2 = Math.sin(frame / 79) * 30;

  return (
    <AbsoluteFill style={{ backgroundColor: theme.colors.void }}>
      {/* L0 → L1. The room is the default value; the void only lives at the edges. */}
      <AbsoluteFill
        style={{
          background: `radial-gradient(ellipse 128% 104% at 50% 44%, ${theme.colors.bg} 0%, #0B0E15 58%, ${theme.colors.void} 100%)`,
        }}
      />

      {/* Primary bloom — light from above and slightly behind the content */}
      <div
        style={{
          position: "absolute",
          width: p.ar * 2,
          height: p.ar * 2,
          left: p.ax - p.ar + dx,
          top: p.ay - p.ar + dy,
          borderRadius: "50%",
          filter: "blur(90px)",
          background: `radial-gradient(circle, ${p.color} 0%, transparent 62%)`,
          opacity: p.wash * 2.1,
        }}
      />

      {/* Counter bloom */}
      <div
        style={{
          position: "absolute",
          width: p.br * 2,
          height: p.br * 2,
          left: p.bx - p.br + dx2,
          top: p.by - p.br + dy2,
          borderRadius: "50%",
          filter: "blur(110px)",
          background: `radial-gradient(circle, ${p.color2} 0%, transparent 66%)`,
          opacity: p.wash * 1.5,
        }}
      />

      {/* A full-width horizon used to live here. It was a third rule in a film
          that owns exactly one (The Thread), and it read as a table edge with
          nothing on it. Deleted per house law 10. */}
    </AbsoluteFill>
  );
};
