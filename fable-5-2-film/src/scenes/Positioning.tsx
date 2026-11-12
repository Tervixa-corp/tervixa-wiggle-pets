import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { theme } from "../theme";
import { clamp01, progress } from "../components/motion";

/**
 * Scene 2 — positioning. 210 frames, local 0–210.
 *
 * One claim, set at 96, with the single word that carries it in clay. No pills,
 * no supporting sentence, no serif italic: the italic was the most borrowed move
 * in the category and it is out.
 *
 * The Thread's vertical at x=218 runs the height of this block — a margin mark,
 * not a divider, because there is nothing to its left.
 */

const DURATION = 210;
const LEFT = 260;

const Line: React.FC<{ children: React.ReactNode; delay: number; top: number }> = ({ children, delay, top }) => {
  const frame = useCurrentFrame();
  const e = theme.ease.out(clamp01((frame - delay) / 24));
  return (
    <div style={{ position: "absolute", left: LEFT, top, overflow: "hidden", paddingBottom: "0.14em", marginBottom: "-0.14em" }}>
      <div
        style={{
          transform: `translateY(${(1 - e) * 106}%)`,
          fontFamily: theme.fonts.sans,
          fontSize: 96,
          fontWeight: 600,
          letterSpacing: "-0.042em",
          lineHeight: 1.06,
          color: theme.colors.bone,
          whiteSpace: "nowrap",
        }}
      >
        {children}
      </div>
    </div>
  );
};

export const Positioning: React.FC = () => {
  const frame = useCurrentFrame();

  const eyebrow = progress(frame, 0, 16);
  const push = progress(frame, 0, 210, theme.ease.inOut);

  return (
    <AbsoluteFill>
      <AbsoluteFill style={{ transform: `scale(${1 + push * 0.012})`, transformOrigin: "38% 48%" }}>
        <div style={{ position: "absolute", left: LEFT, top: 430, width: 18, height: 1, background: theme.colors.clay, opacity: eyebrow }} />
        <div
          style={{
            position: "absolute",
            left: LEFT + 30,
            top: 422,
            fontFamily: theme.fonts.mono,
            fontSize: 12.5,
            fontWeight: 500,
            letterSpacing: "0.32em",
            textTransform: "uppercase",
            color: theme.colors.clay,
            opacity: eyebrow,
            transform: `translateY(${(1 - eyebrow) * 10}px)`,
            whiteSpace: "nowrap",
          }}
        >
          Positioning
        </div>

        <Line delay={18} top={482}>
          For the most <span style={{ color: theme.colors.clay }}>ambitious</span>
        </Line>
        <Line delay={30} top={588}>
          technical work.
        </Line>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

export const POSITIONING_DURATION = DURATION;
