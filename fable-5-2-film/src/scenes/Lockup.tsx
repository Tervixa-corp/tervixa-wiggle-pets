import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { theme } from "../theme";
import { clamp01, progress } from "../components/motion";
import { ATTRIBUTION, AVAILABILITY, MODEL } from "../data/benchmarks";

/**
 * Scene 6 — the lockup. 300 frames, local 0–300. Hard cut in at 1500, no exit.
 *
 * This is the film's last appearance of its own name, so it is the largest type
 * in the film: 196px, not 158. It is lit by a vertical gradient clipped to the
 * glyphs — no halo, no text-shadow. Exactly one warm object: the 5.2. The
 * availability dot is ice, because a second clay accent in the final frame
 * would split the eye.
 *
 * The rule at y=800 is not in this file: it is The Thread, which draws under the
 * name at global 1540 and then retracts into the counter of the 5 at 1760.
 */

const DURATION = 300;

const WORD_TOP = 606;
const WORD_SIZE = 196;

const GlyphGradient: React.FC<{
  children: string;
  from: string;
  mid?: string;
  to: string;
  delay: number;
  per: number;
  dur: number;
  size?: number;
  tracking: string;
}> = ({ children, from, mid, to, delay, per, dur, size = WORD_SIZE, tracking }) => {
  const frame = useCurrentFrame();

  return (
    <span
      style={{
        display: "inline-flex",
        fontFamily: theme.fonts.sans,
        fontSize: size,
        fontWeight: 800,
        letterSpacing: tracking,
        lineHeight: 0.88,
      }}
    >
      {children.split("").map((ch, i) => {
        const e = theme.ease.out(clamp01((frame - delay - i * per) / dur));
        return (
          <span key={`${ch}-${i}`} style={{ display: "inline-block", overflow: "hidden", paddingTop: "0.06em", marginTop: "-0.06em" }}>
            <span
              style={{
                display: "inline-block",
                transform: `translateY(${(1 - e) * 108}%)`,
                backgroundImage: mid
                  ? `linear-gradient(180deg, ${from} 0%, ${mid} 52%, ${to} 100%)`
                  : `linear-gradient(180deg, ${from} 0%, ${to} 100%)`,
                WebkitBackgroundClip: "text",
                backgroundClip: "text",
                color: "transparent",
              }}
            >
              {ch}
            </span>
          </span>
        );
      })}
    </span>
  );
};

/** The 5.2 arrives as a single masked unit, never per character. */
const VersionUnit: React.FC = () => {
  const frame = useCurrentFrame();
  const e = theme.ease.out(clamp01((frame - 64) / 24));
  return (
    <span
      style={{
        display: "inline-block",
        overflow: "hidden",
        paddingTop: "0.06em",
        marginTop: "-0.06em",
        fontFamily: theme.fonts.sans,
        fontSize: WORD_SIZE,
        fontWeight: 800,
        letterSpacing: "-0.062em",
        lineHeight: 0.88,
      }}
    >
      <span
        style={{
          display: "inline-block",
          transform: `translateY(${(1 - e) * 108}%)`,
          backgroundImage: `linear-gradient(180deg, #E08258 0%, #C96442 52%, #A94E31 100%)`,
          WebkitBackgroundClip: "text",
          backgroundClip: "text",
          color: "transparent",
        }}
      >
        {MODEL.version}
      </span>
    </span>
  );
};

/** A specular band crossing the glyphs. Felt, not seen. */
const Sweep: React.FC<{ children: string; delay: number; tracking: string; from: number; to: number }> = ({
  children,
  delay,
  tracking,
  from,
  to,
}) => {
  const frame = useCurrentFrame();
  const t = (frame - delay) / 62;
  if (t < 0 || t > 1) return null;
  const e = theme.ease.inOut(clamp01(t));
  const p = from + (to - from) * e;
  const band = 14;
  return (
    <span
      aria-hidden
      style={{
        position: "absolute",
        left: 0,
        top: 0,
        fontFamily: theme.fonts.sans,
        fontSize: WORD_SIZE,
        fontWeight: 800,
        letterSpacing: tracking,
        lineHeight: 0.88,
        whiteSpace: "nowrap",
        backgroundImage: `linear-gradient(100deg, rgba(255,255,255,0) ${p - band}%, rgba(255,246,238,0.75) ${p}%, rgba(255,255,255,0) ${p + band}%)`,
        WebkitBackgroundClip: "text",
        backgroundClip: "text",
        color: "transparent",
        mixBlendMode: "screen",
        pointerEvents: "none",
      }}
    >
      {children}
    </span>
  );
};

export const Lockup: React.FC = () => {
  const frame = useCurrentFrame();

  const pill = progress(frame, 0, 10);
  const rail = progress(frame, 0, 14);
  const api = progress(frame, 118, 24);
  const foot = progress(frame, 186, 26);

  return (
    <AbsoluteFill>
      {/* ── Availability ─────────────────────────────────────────────────── */}
      <div
        style={{
          position: "absolute",
          left: 796,
          top: 196,
          width: 328,
          height: 44,
          borderRadius: 999,
          background: "rgba(242,239,233,0.03)",
          border: "1px solid rgba(242,239,233,0.13)",
          opacity: pill,
          transform: `translateY(${(1 - pill) * 10}px)`,
        }}
      />
      <div
        style={{
          position: "absolute",
          left: 818,
          top: 215,
          width: 6,
          height: 6,
          borderRadius: "50%",
          background: theme.colors.ice,
          opacity: pill,
          transform: `translateY(${(1 - pill) * 10}px)`,
        }}
      />
      <div
        style={{
          position: "absolute",
          left: 836,
          top: 211,
          fontFamily: theme.fonts.mono,
          fontSize: 12,
          fontWeight: 500,
          letterSpacing: "0.18em",
          color: theme.colors.bone,
          opacity: pill,
          transform: `translateY(${(1 - pill) * 10}px)`,
          whiteSpace: "nowrap",
        }}
      >
        AVAILABLE NOW
      </div>
      <div
        style={{
          position: "absolute",
          left: 1006,
          top: 210,
          width: 1,
          height: 16,
          background: "rgba(242,239,233,0.13)",
          opacity: pill,
        }}
      />
      <div
        style={{
          position: "absolute",
          left: 1022,
          top: 211,
          fontFamily: theme.fonts.mono,
          fontSize: 12,
          fontWeight: 400,
          letterSpacing: "0.10em",
          color: "rgba(242,239,233,0.72)",
          opacity: pill,
          transform: `translateY(${(1 - pill) * 10}px)`,
          whiteSpace: "nowrap",
        }}
      >
        {MODEL.released}
      </div>

      <div
        style={{
          position: "absolute",
          left: 672,
          top: 218,
          width: 108,
          height: 1,
          background: "linear-gradient(90deg, rgba(242,239,233,0), rgba(242,239,233,0.13))",
          opacity: rail,
          transformOrigin: "right center",
          transform: `scaleX(${rail})`,
        }}
      />
      <div
        style={{
          position: "absolute",
          left: 1140,
          top: 218,
          width: 108,
          height: 1,
          background: "linear-gradient(270deg, rgba(242,239,233,0), rgba(242,239,233,0.13))",
          opacity: rail,
          transformOrigin: "left center",
          transform: `scaleX(${rail})`,
        }}
      />

      {/* ── Name ─────────────────────────────────────────────────────────── */}
      <div
        style={{
          position: "absolute",
          left: 890,
          top: 560,
          width: 140,
          fontFamily: theme.fonts.mono,
          fontSize: 16,
          fontWeight: 500,
          letterSpacing: "0.62em",
          color: "rgba(242,239,233,0.62)",
          opacity: progress(frame, 20, 20),
          transform: `translateY(${(1 - progress(frame, 20, 20)) * 8}px)`,
          whiteSpace: "nowrap",
        }}
      >
        CLAUDE
      </div>

      <div
        style={{
          position: "absolute",
          left: 957,
          top: WORD_TOP,
          transform: "translateX(-50%)",
          display: "flex",
          alignItems: "baseline",
          gap: 26,
        }}
      >
        <span style={{ position: "relative", display: "inline-block" }}>
          <GlyphGradient from="#FBF8F2" to="#E6E2DA" delay={36} per={3.2} dur={24} tracking="-0.052em">
            FABLE
          </GlyphGradient>
          <Sweep delay={96} tracking="-0.052em" from={-20} to={120}>
            FABLE
          </Sweep>
        </span>

        <span style={{ position: "relative", display: "inline-block" }}>
          <VersionUnit />
          <Sweep delay={96} tracking="-0.062em" from={-20} to={120}>
            {MODEL.version}
          </Sweep>
        </span>
      </div>

      {/* ── API id ───────────────────────────────────────────────────────── */}
      <div
        style={{
          position: "absolute",
          left: 862,
          top: 836,
          width: 196,
          height: 36,
          borderRadius: 3,
          background: "rgba(191,233,245,0.06)",
          border: "1px solid rgba(191,233,245,0.16)",
          opacity: api,
          transform: `translateY(${(1 - api) * 12}px)`,
        }}
      />
      <div
        style={{
          position: "absolute",
          left: 876,
          top: 847,
          fontFamily: theme.fonts.mono,
          fontSize: 15,
          fontWeight: 400,
          letterSpacing: "0.02em",
          color: theme.colors.ice,
          opacity: api,
          transform: `translateY(${(1 - api) * 12}px)`,
          whiteSpace: "nowrap",
        }}
      >
        {MODEL.apiId}
      </div>

      {/* ── Platforms ────────────────────────────────────────────────────── */}
      <div
        style={{
          position: "absolute",
          left: 624,
          top: 916,
          width: 672,
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
          gap: 18,
        }}
      >
        {AVAILABILITY.map((p, i) => {
          const e = progress(frame, 132 + i * 5, 16);
          return (
            <React.Fragment key={p}>
              {i > 0 && (
                <div
                  style={{
                    width: 3,
                    height: 3,
                    borderRadius: "50%",
                    background: "rgba(242,239,233,0.30)",
                    opacity: e,
                  }}
                />
              )}
              <span
                style={{
                  fontFamily: theme.fonts.mono,
                  fontSize: 11.5,
                  fontWeight: 400,
                  letterSpacing: "0.16em",
                  color: "rgba(242,239,233,0.66)",
                  opacity: e,
                  transform: `translateY(${(1 - e) * 8}px)`,
                  whiteSpace: "nowrap",
                }}
              >
                {p}
              </span>
            </React.Fragment>
          );
        })}
      </div>

      {/* ── Attribution ──────────────────────────────────────────────────── */}
      <div
        style={{
          position: "absolute",
          left: 260,
          top: 952,
          width: 1400,
          textAlign: "center",
          fontFamily: theme.fonts.mono,
          fontSize: 11,
          fontWeight: 400,
          letterSpacing: "0.02em",
          lineHeight: 1.5,
          color: "rgba(242,239,233,0.52)",
          opacity: foot,
          transform: `translateY(${(1 - foot) * 6}px)`,
        }}
      >
        {ATTRIBUTION}
      </div>
    </AbsoluteFill>
  );
};

export const LOCKUP_DURATION = DURATION;
