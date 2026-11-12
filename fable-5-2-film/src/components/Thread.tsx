import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { theme } from "../theme";
import { clamp01, progress } from "./motion";

type Dir = "up" | "down" | "left" | "right";
type Tick = { dir: Dir; len: number };

type Seg = {
  /** Global frame the segment starts drawing. */
  from: number;
  /** Frames to draw the full span. */
  grow: number;
  /** Frames to fade the segment up from zero. 0 = hard cut in. */
  fade?: number;
  /**
   * Global frame at which the segment stops existing. Segments end on hard
   * cuts, so there is no fade-out: a hard cut removes the instrument with the
   * picture, which is the point of M6.
   */
  until?: number;
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  o: number;
  tA?: Tick;
  tB?: Tick;
  /** Minor tick positions along the span, as a fraction 0..1. */
  minor?: number[];
};

/**
 * The Thread — a dimension line, not a rule.
 *
 * It is the film's one continuous instrument: it measures the wordmark, becomes
 * the chart's axis, marks each proof, brackets the price cut, and finally folds
 * itself into the counter of the "5".
 *
 * Three laws, enforced by geometry rather than by masking:
 *   1. It renders BELOW the scenes, so a glyph always occludes it.
 *   2. It never touches a frame edge (minimum 96px clearance, always).
 *   3. It never divides content: a segment has content on one side and air on
 *      the other. A span with content above and below is a rule, and forbidden.
 */
const SEGMENTS: Seg[] = [
  // ── Cold open: the leader drops, then the measure is drawn above the name ──
  { from: 96, until: 210, grow: 16, fade: 10, x1: 560, y1: 316, x2: 560, y2: 412, o: 0.5 },
  {
    from: 112,
    until: 210,
    grow: 44,
    fade: 8,
    x1: 560,
    y1: 412,
    x2: 1360,
    y2: 412,
    o: 0.5,
    tA: { dir: "up", len: 16 },
    tB: { dir: "down", len: 6 },
    minor: [0.25, 0.5, 0.75],
  },

  // ── Positioning: a margin mark down the left of the text block ────────────
  {
    from: 210,
    until: 420,
    grow: 24,
    fade: 6,
    x1: 218,
    y1: 312,
    x2: 218,
    y2: 806,
    o: 0.32,
    tA: { dir: "right", len: 8 },
    tB: { dir: "right", len: 8 },
  },

  // ── Benchmarks: no segment. The chart's axis is drawn by the chart, because
  //    the lit plate occludes anything underneath it. One axis, one author.

  // ── Proof: one mark per vignette, hard-cut with its scene ─────────────────
  { from: 740, until: 870, grow: 22, x1: 1310, y1: 664, x2: 1590, y2: 664, o: 0.45, tA: { dir: "down", len: 6 } },
  { from: 890, until: 1050, grow: 22, x1: 260, y1: 812, x2: 1180, y2: 812, o: 0.4, tA: { dir: "down", len: 6 } },
  { from: 1070, until: 1200, grow: 22, x1: 260, y1: 724, x2: 1200, y2: 724, o: 0.4, tA: { dir: "down", len: 6 } },

  // ── Economics: the dimension bracket over the price delta ─────────────────
  {
    from: 1252,
    until: 1500,
    grow: 48,
    fade: 10,
    x1: 1100,
    y1: 604,
    x2: 1520,
    y2: 604,
    o: 0.62,
    tA: { dir: "down", len: 8 },
    tB: { dir: "down", len: 8 },
  },

  // ── Lockup: the rule under the name, which then folds into the numeral ────
  {
    from: 1540,
    until: 1800,
    grow: 28,
    fade: 8,
    x1: 519,
    y1: 800,
    x2: 1077,
    y2: 800,
    o: 0.55,
    tA: { dir: "up", len: 7 },
    tB: { dir: "up", len: 7 },
  },
];

/** Where the rule dies: the counter of the 5. */
const VANISH = { x: 1156, y: 720, at: 1760, dur: 20 };

const tickGeometry = (x: number, y: number, t: Tick) => {
  switch (t.dir) {
    case "up":
      return { x1: x, y1: y, x2: x, y2: y - t.len };
    case "down":
      return { x1: x, y1: y, x2: x, y2: y + t.len };
    case "left":
      return { x1: x, y1: y, x2: x - t.len, y2: y };
    case "right":
      return { x1: x, y1: y, x2: x + t.len, y2: y };
  }
};

const activeAt = (frame: number) => {
  return SEGMENTS.filter((s) => frame >= s.from && (s.until === undefined || frame < s.until));
};

export const Thread: React.FC = () => {
  const frame = useCurrentFrame();
  const segs = activeAt(frame);
  if (segs.length === 0) return null;

  return (
    <AbsoluteFill style={{ pointerEvents: "none", mixBlendMode: "screen" }}>
      {segs.map((seg, i) => (
        <SegmentLine key={i} frame={frame} seg={seg} />
      ))}
    </AbsoluteFill>
  );
};

const SegmentLine: React.FC<{ frame: number; seg: Seg }> = ({ frame, seg }) => {
  const draw = progress(frame, seg.from, seg.grow);
  const fade = seg.fade ? clamp01((frame - seg.from) / seg.fade) : 1;

  let { x1, y1, x2, y2 } = seg;
  let opacity = seg.o * fade;

  // Only the span drawn so far exists on screen.
  const dx = (x2 - x1) * draw;
  const dy = (y2 - y1) * draw;
  const ex = x1 + dx;
  const ey = y1 + dy;

  // The ending: the two ends leave their ticks and converge on the counter of
  // the 5, arriving at f1780 and dying there.
  const fold = frame >= VANISH.at ? theme.ease.in(clamp01((frame - VANISH.at) / VANISH.dur)) : 0;
  if (fold > 0) {
    x1 = x1 + (VANISH.x - x1) * fold;
    y1 = y1 + (VANISH.y - y1) * fold;
    x2 = x2 + (VANISH.x - x2) * fold;
    y2 = y2 + (VANISH.y - y2) * fold;
    opacity *= 1 - clamp01((frame - (VANISH.at + VANISH.dur - 6)) / 6);
  }

  const ticksDone = draw >= 1;
  const tickA = seg.tA && ticksDone ? tickGeometry(x1, y1, seg.tA) : null;
  const tickB = seg.tB && ticksDone ? tickGeometry(x2, y2, seg.tB) : null;

  return (
    <svg width={1920} height={1080} style={{ position: "absolute", inset: 0 }}>
      <defs>
        <filter id={`thread-glow-${seg.from}`} x="-30%" y="-30%" width="160%" height="160%">
          <feGaussianBlur stdDeviation="3" result="b" />
          <feMerge>
            <feMergeNode in="b" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
        <linearGradient
          id={`thread-grad-${seg.from}`}
          gradientUnits="userSpaceOnUse"
          x1={x1}
          y1={y1}
          x2={x2}
          y2={y2}
        >
          <stop offset="0%" stopColor={theme.colors.ice} stopOpacity="0" />
          <stop offset="12%" stopColor={theme.colors.ice} stopOpacity="0.85" />
          <stop offset="88%" stopColor={theme.colors.bone} stopOpacity="0.9" />
          <stop offset="100%" stopColor={theme.colors.ice} stopOpacity="0" />
        </linearGradient>
      </defs>

      {/* Bloom — one pass only. At two passes the line becomes a bar. */}
      <line
        x1={x1}
        y1={y1}
        x2={ex}
        y2={ey}
        stroke={theme.colors.ice}
        strokeWidth={2.6}
        strokeLinecap="round"
        opacity={opacity * 0.22}
        filter={`url(#thread-glow-${seg.from})`}
      />

      {/* Core hairline */}
      <line
        x1={x1}
        y1={y1}
        x2={ex}
        y2={ey}
        stroke={`url(#thread-grad-${seg.from})`}
        strokeWidth={1}
        strokeLinecap="round"
        opacity={opacity}
      />

      {tickA && <line {...tickA} stroke={theme.colors.bone} strokeWidth={1} opacity={opacity * 0.9} />}
      {tickB && <line {...tickB} stroke={theme.colors.bone} strokeWidth={1} opacity={opacity * 0.9} />}

      {seg.minor &&
        ticksDone &&
        seg.minor.map((f) => {
          const mx = x1 + (x2 - x1) * f;
          const my = y1 + (y2 - y1) * f;
          return (
            <line
              key={f}
              x1={mx}
              y1={my}
              x2={mx}
              y2={my + 4}
              stroke={theme.colors.bone}
              strokeWidth={1}
              opacity={opacity * 0.7}
            />
          );
        })}
    </svg>
  );
};
