import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { theme } from "../theme";
import { progress } from "../components/motion";
import { PRICING } from "../data/benchmarks";

/**
 * Scene 5 — economics. 300 frames, local 0–300.
 *
 * The claim gets bigger as the price gets smaller: "Same work." at 66 is set in
 * boneDim, "Smaller bill." at 104 in bone. One sans, no italic — the serif
 * italic is the most borrowed move in the category and it is out.
 *
 * The ledger's closing dimension is not a rule in this file. It is The Thread
 * (segment 1100→1520 at y 604), which is why the −20% is placed at x 1310: it
 * sits directly under the span that measures it.
 */

const DURATION = 300;
const LEDGER_W = 1000;
const LEDGER_X = 960 - LEDGER_W / 2;

const Row: React.FC<{ label: string; from: number; to: number; delay: number; top: number }> = ({
  label,
  from,
  to,
  delay,
  top,
}) => {
  const frame = useCurrentFrame();

  const enter = progress(frame, delay, 20);
  const price = progress(frame, delay + 10, 18);
  const strike = progress(frame, delay + 22, 20);

  return (
    <div
      style={{
        position: "absolute",
        left: 0,
        right: 0,
        height: 132,
        top,
        opacity: enter,
        transform: `translateY(${(1 - enter) * 24}px)`,
      }}
    >
      <div
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          width: LEDGER_W,
          height: 1,
          background: "rgba(242,239,233,0.13)",
          transformOrigin: "left center",
          transform: `scaleX(${progress(frame, delay - 6, 24)})`,
        }}
      />

      <div
        style={{
          position: "absolute",
          left: 0,
          top: 54,
          fontFamily: theme.fonts.mono,
          fontSize: 13,
          fontWeight: 400,
          letterSpacing: "0.32em",
          textTransform: "uppercase",
          color: "rgba(242,239,233,0.72)",
        }}
      >
        {label}
      </div>

      {/* Old price — legible enough to be read, not dimmed into a rumour */}
      <div
        style={{
          position: "absolute",
          right: 258,
          top: 34,
          fontFamily: theme.fonts.sans,
          fontSize: 76,
          fontWeight: 500,
          letterSpacing: "-0.045em",
          fontVariantNumeric: "tabular-nums",
          color: "rgba(242,239,233,0.55)",
          whiteSpace: "nowrap",
        }}
      >
        ${from}
        <div
          style={{
            position: "absolute",
            left: -4,
            top: "56%",
            height: 2,
            width: `calc(100% + 8px)`,
            background: "rgba(242,239,233,0.5)",
            transformOrigin: "left center",
            transform: `scaleX(${strike}) rotate(-3deg)`,
          }}
        />
      </div>

      <div
        style={{
          position: "absolute",
          right: 196,
          top: 60,
          fontFamily: theme.fonts.mono,
          fontSize: 20,
          fontWeight: 400,
          color: "rgba(242,239,233,0.5)",
          opacity: price,
        }}
      >
        →
      </div>

      {/* New price — the final string from its first visible frame */}
      <div
        style={{
          position: "absolute",
          right: 0,
          top: 20,
          display: "flex",
          alignItems: "baseline",
          fontFamily: theme.fonts.sans,
          fontSize: 104,
          fontWeight: 700,
          letterSpacing: "-0.055em",
          fontVariantNumeric: "tabular-nums",
          color: theme.colors.bone,
          opacity: price,
          transform: `translateY(${(1 - price) * 14}px)`,
          whiteSpace: "nowrap",
        }}
      >
        <span style={{ fontSize: 50, fontWeight: 500, opacity: 0.5, marginRight: 2 }}>$</span>
        {to}
      </div>
    </div>
  );
};

export const Economics: React.FC = () => {
  const frame = useCurrentFrame();

  const eyebrow = progress(frame, 0, 14);
  const title = progress(frame, 8, 26);
  const hero = progress(frame, 64, 24);
  const caption = progress(frame, 84, 20);
  const foot = progress(frame, 210, 22);

  const secondary = [
    `${PRICING.outputSpeedGain}% faster output`,
    `Fast mode ${PRICING.fastMode.speedup}`,
    `$${PRICING.fastMode.input} / $${PRICING.fastMode.output} per MTok`,
  ];

  return (
    <AbsoluteFill>
      {/* Eyebrow */}
      <div
        style={{
          position: "absolute",
          left: 150,
          top: 120,
          width: 22,
          height: 1,
          background: theme.colors.clay,
          opacity: eyebrow,
        }}
      />
      <div
        style={{
          position: "absolute",
          left: 186,
          top: 112,
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
        Economics
      </div>

      {/* The claim */}
      <div
        style={{
          position: "absolute",
          left: 150,
          top: 168,
          display: "flex",
          alignItems: "baseline",
          gap: 20,
          opacity: title,
          transform: `translateY(${(1 - title) * 16}px)`,
          whiteSpace: "nowrap",
        }}
      >
        <span
          style={{
            fontFamily: theme.fonts.sans,
            fontSize: 66,
            fontWeight: 500,
            letterSpacing: "-0.038em",
            color: "rgba(242,239,233,0.72)",
          }}
        >
          Same work.
        </span>
        <span
          style={{
            fontFamily: theme.fonts.sans,
            fontSize: 104,
            fontWeight: 700,
            letterSpacing: "-0.045em",
            color: theme.colors.bone,
          }}
        >
          Smaller bill.
        </span>
      </div>

      {/* Ledger */}
      <div style={{ position: "absolute", left: LEDGER_X, top: 330, width: LEDGER_W, height: 264 }}>
        <Row label="Input" from={PRICING.input.before} to={PRICING.input.now} delay={40} top={0} />
        <Row label="Output" from={PRICING.output.before} to={PRICING.output.now} delay={82} top={132} />
        <div
          style={{
            position: "absolute",
            left: 0,
            bottom: 0,
            width: LEDGER_W,
            height: 1,
            background: "rgba(242,239,233,0.13)",
            transformOrigin: "left center",
            transform: `scaleX(${progress(frame, 50, 30)})`,
          }}
        />
      </div>

      {/* Bottom left — the three facts that make the cut a capability, not a cut */}
      <div style={{ position: "absolute", left: 150, top: 700 }}>
        {secondary.map((t, i) => {
          const e = progress(frame, 210 + i * 6, 18);
          return (
            <div
              key={t}
              style={{
                fontFamily: theme.fonts.mono,
                fontSize: 12.5,
                fontWeight: 400,
                letterSpacing: "0.18em",
                textTransform: "uppercase",
                color: "rgba(242,239,233,0.72)",
                opacity: e,
                transform: `translateY(${(1 - e) * 10}px)`,
                marginBottom: 16,
                whiteSpace: "nowrap",
              }}
            >
              {t}
            </div>
          );
        })}
      </div>

      {/* The hero: one clay object, sitting under the Thread's span */}
      <div
        style={{
          position: "absolute",
          left: 1310,
          top: 630,
          transform: `translateX(-50%) translateY(${(1 - hero) * 30}px)`,
          opacity: hero,
        }}
      >
        <div
          style={{
            fontFamily: theme.fonts.sans,
            fontSize: 176,
            fontWeight: 800,
            letterSpacing: "-0.06em",
            lineHeight: 0.86,
            fontVariantNumeric: "tabular-nums",
            color: theme.colors.clay,
            whiteSpace: "nowrap",
          }}
        >
          −20%
        </div>
      </div>
      <div
        style={{
          position: "absolute",
          left: 1310,
          top: 800,
          transform: "translateX(-50%)",
          textAlign: "center",
          fontFamily: theme.fonts.mono,
          fontSize: 12,
          fontWeight: 400,
          letterSpacing: "0.26em",
          textTransform: "uppercase",
          color: "rgba(242,239,233,0.72)",
          opacity: caption,
          whiteSpace: "nowrap",
        }}
      >
        price per million tokens
      </div>

      {/* Foot — the provenance of the numbers */}
      <div
        style={{
          position: "absolute",
          left: 150,
          top: 952,
          fontFamily: theme.fonts.mono,
          fontSize: 11,
          fontWeight: 400,
          letterSpacing: "0.10em",
          color: "rgba(242,239,233,0.52)",
          opacity: foot,
          whiteSpace: "nowrap",
        }}
      >
        Source: Anthropic, 9 Nov 2026 · USD per million tokens
      </div>
    </AbsoluteFill>
  );
};

export const ECONOMICS_DURATION = DURATION;
