/**
 * Self-hosted Google Fonts, pulled at render time by Remotion so the
 * output is byte-identical on every machine (no system-font fallback).
 */
import { loadFont as loadInter } from "@remotion/google-fonts/Inter";
import { loadFont as loadSerif } from "@remotion/google-fonts/InstrumentSerif";
import { loadFont as loadMono } from "@remotion/google-fonts/JetBrainsMono";

const interLoad = loadInter("normal", {
  weights: ["400", "500", "600", "700", "800"],
  subsets: ["latin"],
});

const serifLoad = loadSerif("italic", {
  weights: ["400"],
  subsets: ["latin"],
});

const monoLoad = loadMono("normal", {
  weights: ["400", "500"],
  subsets: ["latin"],
});

export const fonts = {
  /** Tight grotesk — display + UI + every number. */
  sans: interLoad.fontFamily,
  /** Editorial italic — used sparingly, one word at a time. */
  serif: serifLoad.fontFamily,
  /** Technical micro-labels, source footnotes, model IDs. */
  mono: monoLoad.fontFamily,
};

export const waitForFonts = () =>
  Promise.all([interLoad.waitUntilDone(), serifLoad.waitUntilDone(), monoLoad.waitUntilDone()]);
