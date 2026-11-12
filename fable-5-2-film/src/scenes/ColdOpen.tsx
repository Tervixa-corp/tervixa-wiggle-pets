import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { theme } from "../theme";
import { clamp01 } from "../components/motion";
import { MODEL } from "../data/benchmarks";

/**
 * Scene 1 — the cold open. 210 frames, local 0–210.
 *
 * The one rule of this scene: the name is SET INSIDE A MEASUREMENT, not
 * presented. The Thread draws a dimension 560→1360 above the mark (it lives in
 * Thread.tsx and renders beneath this layer), and the wordmark's ink runs
 * 560.5→1359.5 — half a pixel of air each side.
 *
 * Two corrections to the old build:
 *   1. "5.2" enters as ONE masked unit at local 130. It used to reveal per
 *      character, so at f140 the frame read "FABLE 5" — a different SKU.
 *   2. The clay halo is gone. At 0.24 opacity a blurred clay field integrates
 *      to ochre; there is no opacity at which it stops being brown. At f140 the
 *      only warm object in the frame is the 5.2.
 */

const DURATION = 210;
const MARK_LEFT = 560;
const MARK_W = 800;

const Hair: React.FC<{ delay: number; reverse?: boolean }> = ({ delay, reverse }) => {
  const frame = useCurrentFrame();
  const e = theme.ease.out(clamp01((frame - delay) / 26));
  return (
    <div
      style={{
        width: 120,
        height: 1,
        transformOrigin: reverse ? "right center" : "left center",
        transform: `scaleX(${e})`,
        background: reverse
          ? `linear-gradient(270deg, ${theme.colors.hair}, ${theme.colors.hairLit})`
          : `linear-gradient(90deg, ${theme.colors.hair}, ${theme.colors.hairLit})`,
        opacity: 0.8,
      }}
    />
  );
};

export const ColdOpen: React.FC = () => {
  const frame = useCurrentFrame();

  const dateIn = theme.ease.out(clamp01((frame - 22) / 16));
  const railIn = theme.ease.out(clamp01((frame - 40) / 20));

  // FABLE — five letters, each rising through its own mask.
  const fable = (i: number) => theme.ease.out(clamp01((frame - 118 - i * 3) / 24));
  // 5.2 — one unit, one mask, one transform. Never per character.
  const version = theme.ease.out(clamp01((frame - 130) / 22));

  const specs = [MODEL.contextWindow, MODEL.maxOutput, `effort ${MODEL.defaultEffort}`];

  return (
    <AbsoluteFill>
      {/* Date rail */}
      <div
        style={{
          position: "absolute",
          top: 236,
          left: 0,
          right: 0,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          gap: 20,
        }}
      >
        <Hair delay={12} />
        <span
          style={{
            fontFamily: theme.fonts.mono,
            fontSize: 15,
            fontWeight: 400,
            letterSpacing: "0.42em",
            color: "rgba(242,239,233,0.72)",
            opacity: dateIn,
            transform: `translateY(${(1 - dateIn) * 10}px)`,
            whiteSpace: "nowrap",
          }}
        >
          {MODEL.released}
        </span>
        <Hair delay={12} reverse />
      </div>

      {/* Wordmark — ink 560.5 → 1359.5, centred, set inside the Thread's measure */}
      <div
        style={{
          position: "absolute",
          top: 412,
          left: MARK_LEFT,
          width: MARK_W,
          display: "flex",
          justifyContent: "center",
          alignItems: "baseline",
          gap: 36,
        }}
      >
        <span
          style={{
            display: "inline-flex",
            overflow: "hidden",
            fontFamily: theme.fonts.sans,
            fontSize: theme.type.hero,
            fontWeight: 800,
            letterSpacing: "-0.052em",
            lineHeight: 0.86,
            color: theme.colors.bone,
          }}
        >
          {MODEL.wordmark[1].split("").map((ch, i) => (
            <span key={`${ch}-${i}`} style={{ display: "inline-block", overflow: "hidden" }}>
              <span style={{ display: "inline-block", transform: `translateY(${(1 - fable(i)) * 108}%)` }}>
                {ch}
              </span>
            </span>
          ))}
        </span>

        <span
          style={{
            display: "inline-block",
            overflow: "hidden",
            fontFamily: theme.fonts.sans,
            fontSize: theme.type.hero,
            fontWeight: 800,
            letterSpacing: "-0.062em",
            lineHeight: 0.86,
            color: theme.colors.clay,
          }}
        >
          <span style={{ display: "inline-block", transform: `translateY(${(1 - version) * 108}%)` }}>
            {MODEL.version}
          </span>
        </span>
      </div>

      {/* Technical footer — the model card, stated flatly */}
      <div
        style={{
          position: "absolute",
          top: 700,
          left: 0,
          right: 0,
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
          gap: 34,
        }}
      >
        {specs.map((item, i) => {
          const e = theme.ease.out(clamp01((frame - 160 - i * 6) / 20));
          return (
            <React.Fragment key={item}>
              {i > 0 && (
                <div
                  style={{
                    width: 1,
                    height: 12,
                    background: "rgba(242,239,233,0.20)",
                    opacity: railIn * e,
                  }}
                />
              )}
              <span
                style={{
                  fontFamily: theme.fonts.mono,
                  fontSize: 13,
                  fontWeight: 400,
                  letterSpacing: "0.16em",
                  textTransform: "uppercase",
                  color: "rgba(242,239,233,0.55)",
                  opacity: e,
                  transform: `translateY(${(1 - e) * 12}px)`,
                  whiteSpace: "nowrap",
                }}
              >
                {item}
              </span>
            </React.Fragment>
          );
        })}
      </div>
    </AbsoluteFill>
  );
};

export const COLD_OPEN_DURATION = DURATION;
