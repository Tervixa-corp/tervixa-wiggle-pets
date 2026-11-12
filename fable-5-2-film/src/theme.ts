import { Easing } from "remotion";
import { fonts } from "./fonts";

/**
 * Single source of truth for the film's look and motion feel.
 * Nothing in a scene may inline a hex colour, easing curve or spring config.
 *
 * Value ladder (design spec §3.1) — every frame is built from exactly these
 * three steps, so the picture reads as a lit room rather than a black rectangle:
 *   L0 void   #05060B
 *   L1 room   #141820
 *   L2 lit    #363B45
 */
export const theme = {
  colors: {
    /** L0 — the deepest black. Behind everything. */
    void: "#05060B",
    /** L1 — the room. The default background of a lit space. */
    bg: "#141820",
    /** L2 — a surface light actually lands on. Masked at its edge, never bordered. */
    bgRaise: "#363B45",

    /** Primary text. Warm bone, never pure white. */
    bone: "#F2EFE9",
    /** Secondary text at >= 18px. 10.6:1 on L1. */
    boneDim: "rgba(242,239,233,0.72)",
    /** Small text — captions, ticks, sources, footnotes. 6.1:1 on L1. */
    boneMute: "rgba(242,239,233,0.55)",
    /** HAIRLINES ONLY. At 0.26 this is 2.0:1 and is banned for any text. */
    boneFaint: "rgba(242,239,233,0.26)",

    hair: "rgba(242,239,233,0.13)",
    hairLit: "rgba(242,239,233,0.28)",

    /** Hero accent — the single brand colour. One object per frame, max. */
    clay: "#C96442",
    /** Lighter end of the hero gradient. */
    clayLight: "#E08258",
    /** Darker end, for the bar's shaded side. */
    clayDeep: "#B85736",
    /** Mid stop for the hero bar's horizontal gradient. */
    clayWarm: "#D2714A",
    claySoft: "rgba(201,100,66,0.16)",
    clayGlow: "rgba(201,100,66,0.35)",

    /** Secondary accent — cold, used only for data and The Thread. */
    ice: "#BFE9F5",
    iceSoft: "rgba(191,233,245,0.14)",
    iceGlow: "rgba(191,233,245,0.40)",

    /** Data objects. The bar is solid, never outlined. */
    barTop: "#6B727E",
    barBottom: "#4A505B",
    /** The unfilled remainder — a groove, not an empty slot. */
    track: "#12161E",
    /** The lit plate the chart sits on. */
    wellTop: "#2C313B",
    wellBottom: "#10141C",

    /** Muted bars / inactive data. */
    mute: "rgba(242,239,233,0.45)",
    muteLine: "rgba(242,239,233,0.10)",
  },

  fonts,

  /**
   * Easing curves. Expo-out is the house curve: fast commit, long settle.
   * Never use Remotion's default linear interpolation.
   */
  ease: {
    /** Entrances. */
    out: Easing.bezier(0.16, 1, 0.3, 1),
    /** Exits — exits are always faster than entrances. */
    in: Easing.bezier(0.7, 0, 0.84, 0),
    /** Two-way moves. */
    inOut: Easing.bezier(0.65, 0, 0.35, 1),
    /** Hard, mechanical snap. Used for punches only. */
    sharp: Easing.bezier(0.2, 0, 0, 1),
  },

  /** Spring presets — the actual motion feel of the film. */
  spring: {
    /** Long, heavy settle. Titles and big blocks. */
    smooth: { damping: 200, stiffness: 90, mass: 1 },
    /** The workhorse: decisive, minimal overshoot. */
    snappy: { damping: 26, stiffness: 170, mass: 0.8 },
    /**
     * Overshoot is for TRANSFORMS ONLY — a scale or a translate that goes 4%
     * past its landing and settles. It is NEVER allowed to drive a value: at
     * damping 13 this peaked at 1.095, which printed 59.3% for a 74.2% claim.
     * Damping 18 keeps the overshoot under 1% so even an accidental use is
     * merely a wobble rather than a lie.
     */
    punch: { damping: 18, stiffness: 190, mass: 0.62 },
    /** Slow drift, for backgrounds and idle state. */
    glide: { damping: 40, stiffness: 55, mass: 1.4 },
  },

  /** Type scale in px, tuned for 1920x1080. */
  type: {
    hero: 176,
    h1: 118,
    h2: 82,
    h3: 52,
    lead: 40,
    body: 27,
    micro: 15,
    nano: 12,
  },

  /** Safe margins — nothing critical crosses these. */
  safe: {
    x: 150,
    y: 96,
  },

  /** Global cinematic overlay strengths. */
  fx: {
    grain: 0.055,
    vignette: 0.26,
    /** Clay soft-light. Higher than this turns the film ochre. */
    grade: 0.08,
    halation: 0.10,
  },
} as const;
