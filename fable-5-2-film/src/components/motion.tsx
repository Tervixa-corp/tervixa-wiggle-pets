import React from "react";
import { interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { theme } from "../theme";

type SpringCfg = { damping: number; stiffness: number; mass: number };

export const clamp01 = (v: number) => Math.min(1, Math.max(0, v));

/** Frame-driven progress 0..1 across [start, start+dur], eased. */
export const progress = (frame: number, start: number, dur: number, ease = theme.ease.out) =>
  ease(clamp01((frame - start) / dur));

/**
 * The house entrance: opacity + rise + scale, together, always.
 * A lone fade is never acceptable.
 */
export const Entrance: React.FC<{
  delay?: number;
  children: React.ReactNode;
  style?: React.CSSProperties;
  y?: number;
  scaleFrom?: number;
  blurFrom?: number;
  config?: SpringCfg;
}> = ({ delay = 0, children, style, y = 44, scaleFrom = 0.965, blurFrom = 0, config = theme.spring.smooth }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const p = spring({ frame: frame - delay, fps, config });
  return (
    <div
      style={{
        opacity: p,
        filter: blurFrom ? `blur(${interpolate(p, [0, 1], [blurFrom, 0])}px)` : undefined,
        transform: `translateY(${interpolate(p, [0, 1], [y, 0])}px) scale(${interpolate(
          p,
          [0, 1],
          [scaleFrom, 1],
        )})`,
        ...style,
      }}
    >
      {children}
    </div>
  );
};

/**
 * Per-letter reveal from behind a mask. The single most reliable
 * "expensive-looking" text move there is.
 */
export const MaskedText: React.FC<{
  text: string;
  delay?: number;
  per?: number;
  dur?: number;
  style?: React.CSSProperties;
}> = ({ text, delay = 0, per = 2.2, dur = 20, style }) => {
  const frame = useCurrentFrame();
  return (
    <span style={{ display: "inline-flex", overflow: "hidden", paddingBottom: "0.14em", marginBottom: "-0.14em", ...style }}>
      {text.split("").map((ch, i) => {
        const t = clamp01((frame - delay - i * per) / dur);
        const e = theme.ease.out(t);
        return (
          <span key={`${ch}-${i}`} style={{ display: "inline-block", overflow: "hidden" }}>
            <span style={{ display: "inline-block", transform: `translateY(${(1 - e) * 106}%)` }}>
              {ch === " " ? "\u00A0" : ch}
            </span>
          </span>
        );
      })}
    </span>
  );
};

/** Word-level reveal, for body copy that shouldn't get the full letter treatment. */
export const WordReveal: React.FC<{
  text: string;
  delay?: number;
  per?: number;
  style?: React.CSSProperties;
  wordStyle?: React.CSSProperties;
}> = ({ text, delay = 0, per = 3, style, wordStyle }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  return (
    <span style={{ display: "inline-flex", flexWrap: "wrap", gap: "0.26em", ...style }}>
      {text.split(" ").map((word, i) => {
        const p = spring({ frame: frame - delay - i * per, fps, config: theme.spring.snappy });
        return (
          <span
            key={`${word}-${i}`}
            style={{
              display: "inline-block",
              opacity: p,
              transform: `translateY(${interpolate(p, [0, 1], [28, 0])}px)`,
              ...wordStyle,
            }}
          >
            {word}
          </span>
        );
      })}
    </span>
  );
};

/**
 * RGB split that decays over a few frames. Reserved for impact moments —
 * overuse is what makes a film look like a broken monitor.
 */
export const ChromaticText: React.FC<{
  children: string;
  delay: number;
  spread?: number;
  decay?: number;
  style?: React.CSSProperties;
}> = ({ children, delay, spread = 12, decay = 18, style }) => {
  const frame = useCurrentFrame();
  const off = spread * (1 - theme.ease.out(clamp01((frame - delay) / decay)));
  return (
    <span style={{ display: "inline-grid", ...style }}>
      <span
        aria-hidden
        style={{
          gridArea: "1 / 1",
          color: "#FF2D55",
          mixBlendMode: "screen",
          transform: `translateX(${-off}px)`,
          opacity: off > 0.4 ? 0.9 : 0,
        }}
      >
        {children}
      </span>
      <span
        aria-hidden
        style={{
          gridArea: "1 / 1",
          color: "#00E5FF",
          mixBlendMode: "screen",
          transform: `translateX(${off}px)`,
          opacity: off > 0.4 ? 0.9 : 0,
        }}
      >
        {children}
      </span>
      <span style={{ gridArea: "1 / 1" }}>{children}</span>
    </span>
  );
};

/** Tabular counter — no layout jitter as digits change. */
export const Counter: React.FC<{
  to: number;
  from?: number;
  delay: number;
  dur?: number;
  decimals?: number;
  prefix?: string;
  suffix?: string;
  style?: React.CSSProperties;
}> = ({ to, from = 0, delay, dur = 32, decimals = 1, prefix = "", suffix = "", style }) => {
  const frame = useCurrentFrame();
  const e = progress(frame, delay, dur);
  const v = from + (to - from) * e;
  return (
    <span style={{ fontVariantNumeric: "tabular-nums", ...style }}>
      {prefix}
      {v.toFixed(decimals)}
      {suffix}
    </span>
  );
};

/**
 * Scene exit style. Exits are faster than entrances (~14 frames vs ~22)
 * and add a touch of blur so the cut reads as motion, not disappearance.
 */
export const useSceneExit = (duration: number, dur = 14) => {
  const frame = useCurrentFrame();
  const e = theme.ease.in(clamp01((frame - (duration - dur)) / dur));
  return {
    opacity: 1 - e,
    transform: `translateY(${-e * 46}px) scale(${1 - e * 0.012})`,
    filter: e > 0.001 ? `blur(${e * 7}px)` : undefined,
  } as React.CSSProperties;
};

/** Idle micro-motion so nothing on screen for >2s ever feels frozen. */
export const useBreath = (amount = 0.012, speed = 26) => {
  const frame = useCurrentFrame();
  return 1 + Math.sin(frame / speed) * amount;
};
