/**
 * Every number that appears on screen lives here, with its source attached.
 * If a figure is not in this file, it does not go in the film.
 *
 * Primary sources (fetched 2026-11-11):
 *  - Anthropic launch post, 9 Nov 2026
 *  - Claude Platform docs: Fable 5.2 overview
 *  - Claude Platform docs: What's new in Fable 5.2
 */

export type Source = { label: string; url: string };

export const SOURCES = {
  launch: {
    label: "Anthropic — Claude Fable 5.2 & Mythos 5.2, 9 Nov 2026",
    url: "https://www.anthropic.com/claude-fable-and-mythos-5-2",
  },
  overview: {
    label: "Claude Platform — Fable 5.2 overview",
    url: "https://platform.claude.com/docs/en/models/fable-5-2/overview",
  },
  whatsNew: {
    label: "Claude Platform — What's new in Fable 5.2",
    url: "https://platform.claude.com/docs/en/models/fable-5-2/whats-new-fable-5-2",
  },
} satisfies Record<string, Source>;

export const MODEL = {
  name: "Claude Fable 5.2",
  wordmark: ["CLAUDE", "FABLE"],
  version: "5.2",
  apiId: "claude-fable-5-2",
  released: "09 NOV 2026",
  contextWindow: "1M tokens",
  maxOutput: "128K tokens",
  knowledgeCutoff: "Jun 2026",
  defaultEffort: "high",
} as const;

/** Price per million tokens, USD. */
export const PRICING = {
  input: { now: 8, before: 10 },
  output: { now: 40, before: 50 },
  fastMode: { input: 16, output: 80, speedup: "2.5\u00d7" },
  outputSpeedGain: 30,
  source: SOURCES.whatsNew,
} as const;

export type Bar = {
  label: string;
  value: number;
  /** The model this film is about. Exactly one per chart. */
  hero?: boolean;
  note?: string;
};

export type Benchmark = {
  id: string;
  title: string;
  /** One-line explanation of what the benchmark actually measures. */
  measure: string;
  unit: "%";
  bars: Bar[];
  source: Source;
};

/**
 * Fable 5.2 scores only. Scores recorded on Mythos 5.2 (cyber/bio) are
 * deliberately excluded — they belong to a different, gated model and
 * mixing them would misrepresent Fable.
 */
export const BENCHMARKS: Benchmark[] = [
  {
    id: "frontiercode",
    title: "FrontierCode 2.0",
    measure: "Would the maintainers merge this change?",
    unit: "%",
    bars: [
      { label: "Fable 5.2", value: 49.1, hero: true, note: "xhigh effort" },
      { label: "Opus 5.5", value: 45.5 },
      { label: "GPT-6 Astra", value: 42.8 },
      { label: "Gemini 4 Carbon", value: 41.1 },
      { label: "Fable 5.1", value: 32.4 },
    ],
    source: SOURCES.launch,
  },
  {
    id: "swe",
    title: "FrontierSWE v2",
    measure: "Agentic software engineering",
    unit: "%",
    bars: [
      { label: "Fable 5.2", value: 69.9, hero: true },
      { label: "GPT-6 Astra", value: 65.5 },
      { label: "Opus 5.5", value: 62.3 },
      { label: "Gemini 4 Carbon", value: 61.2 },
      { label: "Fable 5.1", value: 56.3 },
    ],
    source: SOURCES.launch,
  },
  {
    id: "hle",
    title: "Humanity's Last Exam",
    measure: "Multidisciplinary reasoning, with tools",
    unit: "%",
    bars: [
      { label: "Fable 5.2", value: 74.2, hero: true },
      { label: "Opus 5.5", value: 67.7 },
      { label: "Fable 5.1", value: 65.6 },
      { label: "GPT-6 Astra", value: 57.2 },
    ],
    source: SOURCES.launch,
  },
  {
    id: "chartography",
    title: "Chartography",
    measure: "Visual chart recognition, no tools",
    unit: "%",
    bars: [
      { label: "Fable 5.2", value: 73.4, hero: true },
      { label: "Gemini 4 Carbon", value: 72.5 },
      { label: "GPT-6 Astra", value: 71.0 },
      { label: "Opus 5.5", value: 64.4 },
      { label: "Fable 5.1", value: 44.8 },
    ],
    source: SOURCES.launch,
  },
];

export const HEADLINE_BENCHMARK = BENCHMARKS[0];

/** The rotation used in the rapid stat-card montage (scene 3b). */
export const STAT_CARDS = [
  { value: 69.9, label: "FrontierSWE v2", note: "Agentic software engineering" },
  { value: 74.2, label: "Humanity's Last Exam", note: "Reasoning, with tools" },
  { value: 73.4, label: "Chartography", note: "Chart recognition, no tools" },
] as const;

/** Scene 4 — the three proof vignettes. */
export const PROOF = {
  robo: {
    value: 64.9,
    previous: 27.6,
    multiple: "2.4\u00d7",
    label: "RoboDojo Sim",
    note: "Simulated two-arm robot control",
    source: SOURCES.launch,
  },
  fermat: {
    hours: 30,
    agents: 1000,
    shorterProof: "7\u00d7",
    tokensUsed: "\u2153",
    label: "Fermat's Last Theorem",
    note: "Fully formalised in Lean by 1,000 agents",
    source: SOURCES.launch,
  },
  rust: {
    multiple: "26\u00d7",
    label: "Python \u2192 Rust migration",
    note: "Faster service startup after migration",
    source: SOURCES.launch,
  },
} as const;

export const AVAILABILITY = [
  "Claude API",
  "AWS",
  "Amazon Bedrock",
  "Google Cloud",
  "Microsoft Foundry",
] as const;

/**
 * The one published benchmark where Fable 5.2 does not lead. It is disclosed
 * on screen in the chart's footer — a film that shows only wins is not
 * credible, and the whole point of this one is that the numbers can be checked.
 */
export const DISCLOSURE = {
  benchmark: "AutomationBench",
  hero: 51.4,
  rival: 53.9,
  rivalLabel: "Gemini 4 Carbon",
  note: "Business workflow automation",
} as const;

/** Rendered as a small footnote on the end card. Non-negotiable. */
export const ATTRIBUTION =
  "Figures as reported by Anthropic, 9 Nov 2026. Scores reflect each model's highest available effort, except where noted: Gemini 4 Carbon is shown at Medium effort on FrontierCode 2.0.";
