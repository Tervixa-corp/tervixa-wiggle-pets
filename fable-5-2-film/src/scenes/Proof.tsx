import React from "react";
import { AbsoluteFill, Sequence, useCurrentFrame } from "remotion";
import { theme } from "../theme";
import { clamp01, progress } from "../components/motion";
import { PROOF } from "../data/benchmarks";

/**
 * Scene 4 — proof. 480 frames, local 0–480.
 *
 *   Robo    0–150    (global 720–870)
 *   Fermat  150–330  (global 870–1050)  ← Moment 3
 *   Rust    330–480  (global 1050–1200)
 *
 * House laws applied here:
 *   · Numbers resolve, they never count. No `value * p` anywhere in this file.
 *   · Per-character reveals above 12 characters are banned; sentences use a
 *     single-line mask.
 *   · No clay text-shadow. The 5.2, the hero bar, the −20% and the pill dot are
 *     the film's only warm objects; in Proof the warm object is the clock.
 */

const DURATION = 480;
const LEFT = 260;

const Kicker: React.FC<{ children: React.ReactNode; delay?: number }> = ({ children, delay = 0 }) => {
  const frame = useCurrentFrame();
  const e = progress(frame, delay, 14);
  return (
    <div
      style={{
        position: "absolute",
        left: LEFT,
        top: 296,
        display: "flex",
        alignItems: "center",
        gap: 12,
        opacity: e,
        transform: `translateY(${(1 - e) * 10}px)`,
      }}
    >
      <div style={{ width: 18, height: 1, background: theme.colors.clay }} />
      <span
        style={{
          fontFamily: theme.fonts.mono,
          fontSize: 12,
          fontWeight: 500,
          letterSpacing: "0.34em",
          textTransform: "uppercase",
          color: theme.colors.clay,
          whiteSpace: "nowrap",
        }}
      >
        {children}
      </span>
    </div>
  );
};

/** A 46px statement that rises as one line behind one clip. */
const LineMask: React.FC<{ text: string; delay: number; dur?: number; top: number; left?: number; size?: number }> = ({
  text,
  delay,
  dur = 20,
  top,
  left = LEFT,
  size = 46,
}) => {
  const frame = useCurrentFrame();
  const e = theme.ease.out(clamp01((frame - delay) / dur));
  return (
    <div
      style={{
        position: "absolute",
        left,
        top,
        overflow: "hidden",
        paddingBottom: "0.16em",
        marginBottom: "-0.16em",
      }}
    >
      <div
        style={{
          transform: `translateY(${(1 - e) * 106}%)`,
          fontFamily: theme.fonts.sans,
          fontSize: size,
          fontWeight: 600,
          letterSpacing: "-0.032em",
          lineHeight: 1.08,
          color: theme.colors.bone,
          whiteSpace: "nowrap",
        }}
      >
        {text}
      </div>
    </div>
  );
};

/** Per-word rise, for a sentence that needs to arrive in pieces. */
const Words: React.FC<{ text: string; delay: number; per?: number; top: number; left?: number; size?: number; color?: string }> = ({
  text,
  delay,
  per = 3,
  top,
  left = LEFT,
  size = 46,
  color = theme.colors.bone,
}) => {
  const frame = useCurrentFrame();
  return (
    <div
      style={{
        position: "absolute",
        left,
        top,
        display: "flex",
        flexWrap: "wrap",
        gap: "0.24em",
        width: 580,
      }}
    >
      {text.split(" ").map((w, i) => {
        const e = theme.ease.out(clamp01((frame - delay - i * per) / 18));
        return (
          <span key={`${w}-${i}`} style={{ display: "inline-block", overflow: "hidden" }}>
            <span
              style={{
                display: "inline-block",
                transform: `translateY(${(1 - e) * 106}%)`,
                fontFamily: theme.fonts.sans,
                fontSize: size,
                fontWeight: 600,
                letterSpacing: "-0.03em",
                lineHeight: 1.12,
                color,
              }}
            >
              {w}
            </span>
          </span>
        );
      })}
    </div>
  );
};

const Note: React.FC<{ text: string; delay?: number; top?: number }> = ({ text, delay = 20, top = 414 }) => {
  const frame = useCurrentFrame();
  const e = progress(frame, delay, 16);
  return (
    <div
      style={{
        position: "absolute",
        left: LEFT,
        top,
        fontFamily: theme.fonts.mono,
        fontSize: 12.5,
        fontWeight: 400,
        letterSpacing: "0.14em",
        textTransform: "uppercase",
        color: "rgba(242,239,233,0.55)",
        opacity: e,
        transform: `translateY(${(1 - e) * 10}px)`,
        whiteSpace: "nowrap",
      }}
    >
      {text}
    </div>
  );
};

/* ── Vignette 01 — RoboDojo ─────────────────────────────────────────────── */

const ROBO_SCALE = 920 / 70;

const Robo: React.FC = () => {
  const frame = useCurrentFrame();

  const rows = [
    { label: "FABLE 5.2", value: PROOF.robo.value, hero: true, start: 8, top: 556 },
    { label: "FABLE 5.1", value: PROOF.robo.previous, hero: false, start: 16, top: 664 },
  ];

  const mIn = progress(frame, 44, 22);

  return (
    <AbsoluteFill>
      <Kicker delay={0}>Proof 01 — Robotics</Kicker>
      <LineMask text="More than double, one generation on." delay={6} top={340} />
      <Note text={PROOF.robo.note} delay={18} />

      {rows.map((row) => {
        const p = progress(frame, row.start, 34);
        const w = Math.max(0, row.value * ROBO_SCALE * p);
        const valueOpacity = clamp01((p - 0.72) / 0.28);
        return (
          <React.Fragment key={row.label}>
            <div
              style={{
                position: "absolute",
                left: LEFT,
                top: row.top - 30,
                fontFamily: theme.fonts.mono,
                fontSize: 12,
                fontWeight: 500,
                letterSpacing: "0.22em",
                color: row.hero ? "rgba(242,239,233,0.72)" : "rgba(242,239,233,0.55)",
                opacity: progress(frame, row.start - 6, 14),
                whiteSpace: "nowrap",
              }}
            >
              {row.label}
            </div>

            <div
              style={{
                position: "absolute",
                left: LEFT,
                top: row.top,
                width: 920,
                height: 62,
                borderRadius: 1,
                background: theme.colors.track,
                boxShadow: "inset 0 1px 0 rgba(0,0,0,0.55)",
                opacity: progress(frame, row.start - 4, 14),
              }}
            />
            <div
              style={{
                position: "absolute",
                left: LEFT,
                top: row.top,
                width: w,
                height: 62,
                borderRadius: 1,
                background: row.hero
                  ? `linear-gradient(90deg, ${theme.colors.clayDeep} 0%, ${theme.colors.clayWarm} 100%)`
                  : "linear-gradient(180deg, #6B727E 0%, #4A505B 100%)",
                boxShadow: row.hero
                  ? `inset 0 1px 0 rgba(255,225,210,0.30), 0 0 24px rgba(201,100,66,${(0.35 * p).toFixed(3)})`
                  : "inset 0 1px 0 rgba(255,255,255,0.16)",
              }}
            />
            <div
              style={{
                position: "absolute",
                left: LEFT + w + 20,
                top: row.top + 16,
                fontFamily: theme.fonts.sans,
                fontSize: 34,
                fontWeight: 600,
                letterSpacing: "-0.024em",
                lineHeight: "30px",
                fontVariantNumeric: "tabular-nums",
                color: row.hero ? theme.colors.bone : "rgba(242,239,233,0.86)",
                opacity: valueOpacity,
                whiteSpace: "nowrap",
              }}
            >
              {row.value.toFixed(1)}%
            </div>
          </React.Fragment>
        );
      })}

      {/* The measure sits above the multiple; the multiple is the warm object. */}
      <div style={{ position: "absolute", left: 1310, top: 690, opacity: mIn, transform: `translateY(${(1 - mIn) * 26}px)` }}>
        <div
          style={{
            fontFamily: theme.fonts.sans,
            fontSize: 132,
            fontWeight: 800,
            letterSpacing: "-0.055em",
            lineHeight: 0.86,
            color: theme.colors.bone,
          }}
        >
          {PROOF.robo.multiple}
        </div>
      </div>
      <div
        style={{
          position: "absolute",
          left: 1312,
          top: 836,
          fontFamily: theme.fonts.mono,
          fontSize: 11.5,
          fontWeight: 400,
          letterSpacing: "0.22em",
          textTransform: "uppercase",
          lineHeight: 1.9,
          color: "rgba(242,239,233,0.55)",
          opacity: progress(frame, 58, 18),
          whiteSpace: "nowrap",
        }}
      >
        over the previous Fable
      </div>
    </AbsoluteFill>
  );
};

/* ── Vignette 02 — Fermat (Moment 3) ────────────────────────────────────── */

const COLS = 50;
const ROWS = 20;
const CELL = 13;

const BONE = "#F2EFE9";
const CLAY = "#C96442";

const mixHex = (a: string, b: string, t: number) => {
  const pa = parseInt(a.slice(1), 16);
  const pb = parseInt(b.slice(1), 16);
  const A = [(pa >> 16) & 255, (pa >> 8) & 255, pa & 255];
  const B = [(pb >> 16) & 255, (pb >> 8) & 255, pb & 255];
  const c = A.map((v, i) => Math.round(v + (B[i] - v) * t));
  return `rgb(${c[0]},${c[1]},${c[2]})`;
};

const Fermat: React.FC = () => {
  const frame = useCurrentFrame();

  // The wave: a per-dot colour lerp that crosses left → right at local 58–80.
  const wp = (frame - 56) / 26;
  const breath = 1 + Math.sin((frame / 33) * Math.PI * 2) * 0.006;

  const clock = theme.ease.out(clamp01((frame - 98) / 6));
  const tokens = progress(frame, 140, 12);

  const dots: React.ReactNode[] = [];
  for (let r = 0; r < ROWS; r += 1) {
    for (let c = 0; c < COLS; c += 1) {
      const appear = clamp01((frame - 4 - (c / COLS) * 24 - (r / ROWS) * 12) / 9);
      const d = c / (COLS - 1);
      const band = clamp01(1 - Math.abs(d - wp) / 0.16) * 0.55;
      dots.push(
        <div
          key={`${r}-${c}`}
          style={{
            position: "absolute",
            left: c * CELL,
            top: r * CELL,
            width: 4,
            height: 4,
            borderRadius: "50%",
            backgroundColor: mixHex(BONE, CLAY, band),
            opacity: appear * 0.34 * breath,
          }}
        />,
      );
    }
  }

  const statLabel: React.CSSProperties = {
    fontFamily: theme.fonts.mono,
    fontSize: 11.5,
    fontWeight: 400,
    letterSpacing: "0.2em",
    textTransform: "uppercase",
    color: "rgba(242,239,233,0.72)",
  };

  return (
    <AbsoluteFill>
      <Kicker delay={0}>Proof 02 — Mathematics</Kicker>

      {/* The light table */}
      <div
        style={{
          position: "absolute",
          left: 200,
          top: 470,
          width: 1100,
          height: 360,
          background:
            "radial-gradient(ellipse 100% 100% at 50% 50%, #363B45 0%, #363B45 52%, rgba(54,59,69,0) 100%)",
          opacity: 0.85 * progress(frame, 0, 24),
        }}
      />

      <div style={{ position: "absolute", left: LEFT, top: 536, width: COLS * CELL, height: ROWS * CELL, opacity: breath }}>
        {dots}
      </div>

      {/* Payoff — after the silence, never before it */}
      <Words text="A century-old theorem, closed overnight." delay={98} per={3} top={330} left={1130} size={46} />

      <div
        style={{
          position: "absolute",
          left: 1130,
          top: 500,
          fontFamily: theme.fonts.sans,
          fontSize: 96,
          fontWeight: 700,
          letterSpacing: "-0.05em",
          lineHeight: 0.9,
          fontVariantNumeric: "tabular-nums",
          color: theme.colors.clay,
          opacity: clock,
          filter: `blur(${(1 - clock) * 8}px)`,
          transform: `translateY(${(1 - clock) * 8}px)`,
        }}
      >
        {PROOF.fermat.hours}h
      </div>
      <div
        style={{
          ...statLabel,
          position: "absolute",
          left: 1130,
          top: 600,
          opacity: progress(frame, 104, 12),
        }}
      >
        wall clock
      </div>

      <div
        style={{
          position: "absolute",
          left: 1130,
          top: 636,
          fontFamily: theme.fonts.sans,
          fontSize: 56,
          fontWeight: 700,
          letterSpacing: "-0.04em",
          lineHeight: 1,
          fontVariantNumeric: "tabular-nums",
          color: theme.colors.bone,
          opacity: progress(frame, 112, 12),
        }}
      >
        1,000
      </div>
      <div style={{ ...statLabel, position: "absolute", left: 1300, top: 658, opacity: progress(frame, 118, 12) }}>
        agents in parallel
      </div>

      <div
        style={{
          position: "absolute",
          left: 1130,
          top: 704,
          fontFamily: theme.fonts.sans,
          fontSize: 56,
          fontWeight: 700,
          letterSpacing: "-0.04em",
          lineHeight: 1,
          fontVariantNumeric: "tabular-nums",
          color: theme.colors.bone,
          opacity: progress(frame, 124, 12),
        }}
      >
        {PROOF.fermat.shorterProof}
      </div>
      <div style={{ ...statLabel, position: "absolute", left: 1236, top: 726, opacity: progress(frame, 130, 12) }}>
        shorter proof
      </div>

      <div
        style={{
          position: "absolute",
          left: 1130,
          top: 762,
          fontFamily: theme.fonts.sans,
          fontSize: 20,
          fontWeight: 500,
          letterSpacing: "-0.01em",
          color: "rgba(242,239,233,0.72)",
          opacity: tokens,
          whiteSpace: "nowrap",
        }}
      >
        <span style={{ color: theme.colors.bone, fontWeight: 600 }}>{PROOF.fermat.tokensUsed}</span> the action tokens.
      </div>
    </AbsoluteFill>
  );
};

/* ── Vignette 03 — Python → Rust ────────────────────────────────────────── */

const Rust: React.FC = () => {
  const frame = useCurrentFrame();

  const rust = progress(frame, 16, 18);
  const python = progress(frame, 30, 78) * 0.045;

  const tracks = [
    { label: "RUST", note: "READY", p: rust, hero: true, top: 560, delay: 14 },
    { label: "PYTHON", note: "STILL STARTING", p: python, hero: false, top: 640, delay: 22 },
  ];

  return (
    <AbsoluteFill>
      <Kicker delay={0}>Proof 03 — Migration</Kicker>
      <LineMask text="The rewrite nobody wants to start." delay={6} top={340} />
      <Note text={PROOF.rust.note} delay={18} />

      {tracks.map((t) => (
        <React.Fragment key={t.label}>
          <div
            style={{
              position: "absolute",
              left: LEFT,
              top: t.top - 28,
              display: "flex",
              gap: 16,
              opacity: progress(frame, t.delay - 6, 12),
              whiteSpace: "nowrap",
            }}
          >
            <span
              style={{
                fontFamily: theme.fonts.mono,
                fontSize: 12,
                fontWeight: 500,
                letterSpacing: "0.3em",
                color: t.hero ? theme.colors.clay : "rgba(242,239,233,0.55)",
              }}
            >
              {t.label}
            </span>
            <span
              style={{
                fontFamily: theme.fonts.mono,
                fontSize: 11,
                fontWeight: 400,
                letterSpacing: "0.18em",
                color: "rgba(242,239,233,0.45)",
              }}
            >
              {t.note}
            </span>
          </div>

          <div
            style={{
              position: "absolute",
              left: LEFT,
              top: t.top,
              width: 940,
              height: 14,
              borderRadius: 1,
              background: theme.colors.track,
              boxShadow: "inset 0 1px 0 rgba(0,0,0,0.55)",
            }}
          />
          <div
            style={{
              position: "absolute",
              left: LEFT,
              top: t.top,
              width: 940 * t.p,
              height: 14,
              borderRadius: 1,
              background: t.hero
                ? `linear-gradient(90deg, ${theme.colors.clayDeep} 0%, ${theme.colors.clayWarm} 100%)`
                : "linear-gradient(180deg, #6B727E 0%, #4A505B 100%)",
              boxShadow: t.hero
                ? "inset 0 1px 0 rgba(255,225,210,0.30), 0 0 24px rgba(201,100,66,0.35)"
                : "inset 0 1px 0 rgba(255,255,255,0.16)",
            }}
          />
        </React.Fragment>
      ))}

      <div style={{ position: "absolute", left: 1310, top: 540, opacity: progress(frame, 36, 22), transform: `translateY(${(1 - progress(frame, 36, 22)) * 24}px)` }}>
        <div
          style={{
            fontFamily: theme.fonts.sans,
            fontSize: 150,
            fontWeight: 800,
            letterSpacing: "-0.06em",
            lineHeight: 0.84,
            color: theme.colors.bone,
          }}
        >
          {PROOF.rust.multiple}
        </div>
      </div>
      <div
        style={{
          position: "absolute",
          left: 1312,
          top: 700,
          fontFamily: theme.fonts.mono,
          fontSize: 11.5,
          fontWeight: 400,
          letterSpacing: "0.22em",
          textTransform: "uppercase",
          lineHeight: 1.9,
          color: "rgba(242,239,233,0.55)",
          opacity: progress(frame, 50, 18),
          whiteSpace: "nowrap",
        }}
      >
        faster service
        <br />
        startup
      </div>
    </AbsoluteFill>
  );
};

/* ── Scene ─────────────────────────────────────────────────────────────── */

export const Proof: React.FC = () => (
  <AbsoluteFill>
    <Sequence from={0} durationInFrames={150}>
      <Robo />
    </Sequence>
    <Sequence from={150} durationInFrames={180}>
      <Fermat />
    </Sequence>
    <Sequence from={330} durationInFrames={150}>
      <Rust />
    </Sequence>
  </AbsoluteFill>
);

export const PROOF_DURATION = DURATION;
