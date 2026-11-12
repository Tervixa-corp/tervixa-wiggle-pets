import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { theme } from "../theme";
import { MaskedText, clamp01, progress } from "../components/motion";
import { DISCLOSURE, HEADLINE_BENCHMARK } from "../data/benchmarks";

/**
 * Scene 3 — the chart. 300 frames, local 0–300.
 *
 * Rebuilt to design spec §2. Three things changed and nothing else:
 *   1. Labels live in the gutter, never inside a bar. A label cannot be inside
 *      a bar and outside a form field.
 *   2. Bars are solid objects with a track, not 1px outlines. The eye reads
 *      length, not edges.
 *   3. No number is ever derived by multiplying a true value by a spring. The
 *      string is final from its first visible frame; only its opacity animates.
 *      At every frame a value is either invisible or true.
 */

const DURATION = 300;
const AXIS_Y = 800;
const PLOT_X = 420;
const PLOT_W = 1160;
const PX_PER_UNIT = PLOT_W / 50;
const BAR_H = 54;
const ROW_GAP = 38;
const ROW_TOP = 378;
const LABEL_W = 226;
const VALUE_W = 96;

const bars = HEADLINE_BENCHMARK.bars;
const rowTop = (i: number) => ROW_TOP + i * (BAR_H + ROW_GAP);
const rowCentre = (i: number) => rowTop(i) + BAR_H / 2;
const barWidth = (value: number) => value * PX_PER_UNIT;

/** Bar entrance starts, top row down. Hero is one beat late — that is the point. */
const IN_START = [150, 70, 76, 82, 88];

const hex = (h: string) => {
  const n = parseInt(h.slice(1), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
};
const mix = (a: string, b: string, t: number) => {
  const A = hex(a);
  const B = hex(b);
  const c = A.map((v, i) => Math.round(v + (B[i] - v) * t));
  return `rgb(${c[0]},${c[1]},${c[2]})`;
};

export const Benchmarks: React.FC = () => {
  const frame = useCurrentFrame();

  // ── the warm phase: a flat surface lerp, never a blur, so it cannot go brown
  const warm = progress(frame, 182, 18, theme.ease.inOut);

  // ── the last 52 frames are a camera move and nothing else
  const push = progress(frame, 248, 52, theme.ease.inOut);
  const scale = 1 + push * 0.006;

  const eyebrow = progress(frame, 0, 12);
  const titleStart = 4;
  const measure = progress(frame, 14, 16);
  const axis = progress(frame, 22, 18);
  const well = progress(frame, 36, 16);
  const tracks = progress(frame, 42, 14);

  const glow = 0.35 * progress(frame, 182, 18, theme.ease.inOut);
  const loss = progress(frame, 230, 18);

  const gridValues = [10, 20, 30, 40, 50];

  return (
    <AbsoluteFill style={{ transform: `scale(${scale})`, transformOrigin: "50% 62%" }}>
        {/* ── Eyebrow ─────────────────────────────────────────────────────── */}
        <div
          style={{
            position: "absolute",
            left: 150,
            top: 120,
            width: 22,
            height: 1,
            background: theme.colors.clay,
            opacity: eyebrow,
            transform: `translateY(${(1 - eyebrow) * 10}px)`,
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
          Benchmark 01 — Agentic coding
        </div>

        {/* ── Title ───────────────────────────────────────────────────────── */}
        <div style={{ position: "absolute", left: 150, top: 146 }}>
          <MaskedText
            text={HEADLINE_BENCHMARK.title}
            delay={titleStart}
            per={1.1}
            dur={18}
            style={{
              fontFamily: theme.fonts.sans,
              fontSize: 82,
              fontWeight: 700,
              letterSpacing: "-0.042em",
              color: theme.colors.bone,
              lineHeight: 1,
            }}
          />
        </div>

        {/* ── Measure ─────────────────────────────────────────────────────── */}
        <div
          style={{
            position: "absolute",
            left: 150,
            top: 244,
            width: 560,
            fontFamily: theme.fonts.sans,
            fontSize: 24,
            fontWeight: 400,
            letterSpacing: "-0.01em",
            color: "rgba(242,239,233,0.62)",
            opacity: measure,
            transform: `translateY(${(1 - measure) * 8}px)`,
            whiteSpace: "nowrap",
          }}
        >
          {HEADLINE_BENCHMARK.measure}
        </div>

        {/* ── Hero note, above the plot, right-aligned to the last gridline ─ */}
        <div
          style={{
            position: "absolute",
            left: 1316,
            top: 259,
            width: 10,
            height: 1,
            background: theme.colors.clay,
            opacity: progress(frame, 30, 14),
          }}
        />
        <div
          style={{
            position: "absolute",
            left: 1330,
            top: 252,
            width: 250,
            textAlign: "right",
            fontFamily: theme.fonts.mono,
            fontSize: 11.5,
            fontWeight: 400,
            letterSpacing: "0.16em",
            textTransform: "uppercase",
            color: "rgba(242,239,233,0.55)",
            opacity: progress(frame, 30, 14),
            whiteSpace: "nowrap",
          }}
        >
          Highest of any model tested
        </div>

        {/* ── The well: the lit plate the evidence sits on ─────────────────── */}
        <div
          style={{
            position: "absolute",
            left: 384,
            top: 362,
            width: 1232,
            height: 476,
            borderRadius: 2,
            background: `linear-gradient(180deg, ${mix("#2C313B", "#342C27", warm)} 0%, ${mix(
              "#10141C",
              "#17120F",
              warm,
            )} 100%)`,
            boxShadow: "inset 0 1px 0 rgba(242,239,233,0.10)",
            opacity: well,
          }}
        />

        {/* ── Gridlines ───────────────────────────────────────────────────── */}
        {gridValues.map((v, i) => {
          const g = progress(frame, 26 + i * 3, 18);
          return (
            <div
              key={v}
              style={{
                position: "absolute",
                left: PLOT_X + v * PX_PER_UNIT,
                top: 368,
                width: 1,
                height: 432,
                background: "rgba(242,239,233,0.075)",
                opacity: g,
              }}
            />
          );
        })}

        {/* ── Tracks: one plane, faded in as a whole ──────────────────────── */}
        {bars.map((bar, i) => (
          <div
            key={`track-${bar.label}`}
            style={{
              position: "absolute",
              left: PLOT_X,
              top: rowTop(i),
              width: PLOT_W,
              height: BAR_H,
              borderRadius: 1,
              background: theme.colors.track,
              boxShadow: "inset 0 1px 0 rgba(0,0,0,0.55)",
              opacity: tracks,
            }}
          />
        ))}

        {/* ── Bars ────────────────────────────────────────────────────────── */}
        {bars.map((bar, i) => {
          const isHero = !!bar.hero;
          const p = progress(frame, IN_START[i], 32, theme.ease.out);
          // The hero settles its last 2px after the plate warms, so the eye
          // catches the edge arriving rather than the bar simply stopping.
          const w = isHero
            ? Math.max(0, (barWidth(bar.value) - 2.1) * p + 2.1 * progress(frame, 182, 36))
            : Math.max(0, barWidth(bar.value) * p);

          const top = isHero && frame >= 182 ? "rgba(255,225,210,0.30)" : "rgba(255,255,255,0.16)";
          const valueOpacity = clamp01((p - 0.72) / 0.28);

          return (
            <React.Fragment key={bar.label}>
              <div
                style={{
                  position: "absolute",
                  left: PLOT_X,
                  top: rowTop(i),
                  width: w,
                  height: BAR_H,
                  borderRadius: 1,
                  background: isHero
                    ? `linear-gradient(90deg, ${theme.colors.clayDeep} 0%, ${theme.colors.clayWarm} 100%)`
                    : `linear-gradient(180deg, ${mix("#6B727E", "#606772", warm)} 0%, ${mix(
                        "#4A505B",
                        "#424852",
                        warm,
                      )} 100%)`,
                  boxShadow: isHero
                    ? `inset 0 1px 0 ${top}, 0 0 24px rgba(201,100,66,${glow.toFixed(3)})`
                    : `inset 0 1px 0 ${top}`,
                }}
              />

              {/* Value — rides the edge, is either invisible or true */}
              <div
                style={{
                  position: "absolute",
                  left: PLOT_X + w + 24,
                  top: rowTop(i) + 12,
                  width: VALUE_W,
                  fontFamily: theme.fonts.sans,
                  fontSize: 26,
                  fontWeight: 600,
                  letterSpacing: "-0.02em",
                  fontVariantNumeric: "tabular-nums",
                  lineHeight: "30px",
                  color: isHero ? theme.colors.bone : "rgba(242,239,233,0.86)",
                  opacity: valueOpacity,
                  whiteSpace: "nowrap",
                }}
              >
                {bar.value.toFixed(1)}%
              </div>
            </React.Fragment>
          );
        })}

        {/* ── Labels, in the gutter ───────────────────────────────────────── */}
        {bars.map((bar, i) => {
          const l = progress(frame, 44 + i * 4, 14);
          return (
            <div
              key={`label-${bar.label}`}
              style={{
                position: "absolute",
                left: 150,
                top: rowCentre(i) - 11,
                width: LABEL_W,
                textAlign: "right",
                fontFamily: theme.fonts.sans,
                fontSize: 21,
                fontWeight: bar.hero ? 600 : 500,
                letterSpacing: "-0.012em",
                lineHeight: "22px",
                color: bar.hero ? theme.colors.bone : "rgba(242,239,233,0.62)",
                opacity: l,
                transform: `translateY(${(1 - l) * 8}px)`,
                whiteSpace: "nowrap",
              }}
            >
              {bar.label}
            </div>
          );
        })}

        {/* ── Axis: the Thread's own treatment, drawn above the plate ─────── */}
        <div
          style={{
            position: "absolute",
            left: PLOT_X,
            top: AXIS_Y,
            width: PLOT_W * axis,
            height: 1,
            background: `linear-gradient(90deg, rgba(191,233,245,0.55) 0%, rgba(242,239,233,0.34) 22%, rgba(242,239,233,0.28) 100%)`,
          }}
        />
        {[0, 50].map((v) => (
          <div
            key={`endtick-${v}`}
            style={{
              position: "absolute",
              left: PLOT_X + v * PX_PER_UNIT - 0.5,
              top: AXIS_Y,
              width: 1,
              height: 6,
              background: "rgba(242,239,233,0.34)",
              opacity: axis,
            }}
          />
        ))}

        {/* ── Ticks + labels ──────────────────────────────────────────────── */}
        {[0, 10, 20, 30, 40, 50].map((v, i) => {
          const t = progress(frame, 26 + i * 3, 18);
          const x = PLOT_X + v * PX_PER_UNIT;
          return (
            <React.Fragment key={`tick-${v}`}>
              {v !== 0 && v !== 50 && (
                <div
                  style={{
                    position: "absolute",
                    left: x - 0.5,
                    top: AXIS_Y,
                    width: 1,
                    height: 6,
                    background: "rgba(242,239,233,0.28)",
                    opacity: t,
                  }}
                />
              )}
              <div
                style={{
                  position: "absolute",
                  left: x - 16,
                  top: 818,
                  width: 32,
                  textAlign: "center",
                  fontFamily: theme.fonts.mono,
                  fontSize: 11,
                  fontWeight: 400,
                  letterSpacing: "0.08em",
                  color: "rgba(242,239,233,0.52)",
                  opacity: t,
                  whiteSpace: "nowrap",
                }}
              >
                {v}
              </div>
            </React.Fragment>
          );
        })}

        {/* ── Footer rail: the bottom of the frame now has a job ──────────── */}
        <div
          style={{
            position: "absolute",
            left: 150,
            top: 852,
            width: 1620,
            height: 1,
            background: "rgba(242,239,233,0.10)",
            opacity: progress(frame, 60, 20),
          }}
        />
        <div
          style={{
            position: "absolute",
            left: 150,
            top: 866,
            fontFamily: theme.fonts.mono,
            fontSize: 11.5,
            fontWeight: 400,
            letterSpacing: "0.10em",
            color: "rgba(242,239,233,0.55)",
            opacity: progress(frame, 64, 20),
            whiteSpace: "nowrap",
          }}
        >
          Source: Anthropic, 9 Nov 2026
        </div>

        {/* The one place in the film where a number is bad news. */}
        <div
          style={{
            position: "absolute",
            left: 150,
            top: 904,
            width: 18,
            height: 1,
            background: "rgba(242,239,233,0.30)",
            opacity: loss,
          }}
        />
        <div
          style={{
            position: "absolute",
            left: 150,
            top: 892,
            fontFamily: theme.fonts.sans,
            fontSize: 19,
            fontWeight: 500,
            letterSpacing: "-0.008em",
            lineHeight: "26px",
            color: "rgba(242,239,233,0.72)",
            opacity: loss,
            transform: `translateY(${(1 - loss) * 6}px)`,
            whiteSpace: "nowrap",
          }}
        >
          {DISCLOSURE.benchmark} — Fable 5.2{" "}
          <span style={{ color: theme.colors.bone }}>{DISCLOSURE.hero.toFixed(1)}</span> ·{" "}
          {DISCLOSURE.rivalLabel}{" "}
          <span style={{ color: theme.colors.bone }}>{DISCLOSURE.rival.toFixed(1)}</span>
        </div>
      </AbsoluteFill>
  );
};

export const BENCHMARKS_DURATION = DURATION;
