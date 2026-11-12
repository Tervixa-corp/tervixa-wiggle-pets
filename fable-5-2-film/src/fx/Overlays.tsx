import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { theme } from "../theme";

const NOISE_URI =
  "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='220' height='220'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='220' height='220' filter='url(%23n)' opacity='0.55'/%3E%3C/svg%3E\")";

/** Procedural film grain — no asset file, deterministic per frame. */
export const Grain: React.FC = () => {
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill
      style={{
        pointerEvents: "none",
        backgroundImage: NOISE_URI,
        backgroundSize: "220px 220px",
        backgroundPosition: `${(frame * 7) % 220}px ${(frame * 13) % 220}px`,
        opacity: theme.fx.grain,
        mixBlendMode: "overlay",
      }}
    />
  );
};

/** Radial falloff — keeps the eye in the middle third. */
export const Vignette: React.FC = () => {
  const frame = useCurrentFrame();
  const pulse = theme.fx.vignette + Math.sin(frame / 48) * 0.012;
  return (
    <AbsoluteFill
      style={{
        pointerEvents: "none",
        background: `radial-gradient(ellipse 78% 74% at 50% 48%, transparent 42%, rgba(2,3,6,${pulse}) 100%)`,
      }}
    />
  );
};

/**
 * Colour grade — rendered above content, below grain.
 * Unifies every element into one film stock.
 */
export const Grade: React.FC = () => (
  <AbsoluteFill style={{ pointerEvents: "none" }}>
    <AbsoluteFill style={{ backgroundColor: theme.colors.clay, mixBlendMode: "soft-light", opacity: theme.fx.grade }} />
    <AbsoluteFill style={{ backgroundColor: "#07131F", mixBlendMode: "soft-light", opacity: 0.14 }} />
    <AbsoluteFill
      style={{
        background:
          "linear-gradient(180deg, rgba(0,0,0,0.14) 0%, transparent 24%, transparent 74%, rgba(0,0,0,0.26) 100%)",
      }}
    />
  </AbsoluteFill>
);

/** Film gate weave — a sub-pixel drift that reads as "shot", not "rendered". */
export const GateWeave: React.FC = () => {
  const frame = useCurrentFrame();
  const x = Math.sin(frame / 7.3) * 0.55 + Math.sin(frame / 2.9) * 0.25;
  const y = Math.cos(frame / 6.1) * 0.45 + Math.cos(frame / 3.4) * 0.2;
  return (
    <AbsoluteFill
      style={{
        pointerEvents: "none",
        transform: `translate(${x}px, ${y}px)`,
        boxShadow: "inset 0 0 220px rgba(0,0,0,0.55)",
      }}
    />
  );
};

/** Frames where a hard cut lands — the sweep and bass hit key off these. */
export const CUTS = [420, 720, 1200, 1500] as const;

/**
 * A soft band of light crossing the frame on a cut. Very low opacity —
 * it should be felt, not seen.
 */
export const Sweep: React.FC = () => {
  const frame = useCurrentFrame();
  for (const cut of CUTS) {
    const t = (frame - (cut - 7)) / 17;
    if (t < 0 || t > 1) continue;
    const e = theme.ease.inOut(Math.min(1, Math.max(0, t)));
    const x = -420 + e * 2760;
    return (
      <AbsoluteFill style={{ pointerEvents: "none", mixBlendMode: "screen" }}>
        <div
          style={{
            position: "absolute",
            top: -200,
            bottom: -200,
            width: 420,
            left: x,
            background:
              "linear-gradient(90deg, transparent 0%, rgba(191,233,245,0.055) 34%, rgba(242,239,233,0.13) 50%, rgba(191,233,245,0.055) 66%, transparent 100%)",
            transform: "rotate(8deg)",
            filter: "blur(2px)",
          }}
        />
      </AbsoluteFill>
    );
  }
  return null;
};
