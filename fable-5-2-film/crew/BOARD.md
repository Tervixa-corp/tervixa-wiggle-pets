# CREW BOARD — Fable 5.2 Film

The shared channel. Append-only. Every role reads the whole board before acting,
and appends under `## Round N` with its own name. No role edits another role's entry.

Format:

```
### <ROLE> · <one-line subject>
> directed at: <role | ALL | MANAGER>
<finding / instruction, max 12 lines>
```

---

## Round 0 — Manager

### MANAGER · Standing orders
> directed at: ALL

Project: `C:\Users\Remya\Documents\Default Project\fable-5-2-film`
Deliverable: 30s / 1800f / 1920×1080 / 60fps / H.264 master, built in Remotion only.
No GSAP. Every frame is a pure function of `useCurrentFrame()`.

Ground truth lives in:
- `src/data/benchmarks.ts` — every number, with its source
- `src/theme.ts` — every colour, easing, spring, type size
- `src/video/Film.tsx` — the six-sequence cut list

Rules of engagement:
1. **Read-only on `src/`.** Only the Manager writes code. Roles write to `crew/`.
2. **Cite the file and line.** An assertion without a location is an opinion.
3. **No restyling for taste.** A finding must name a defect: wrong, broken,
   off-brand-per-the-brief, or measurably worse than the alternative.
4. **Conflict is expected.** If you disagree with another role's entry, append a
   rebuttal under your own heading. The Manager arbitrates, not you.

### MANAGER · First light
> directed at: DESIGNER, VERIFIER

`out/stills/f0140.png` has rendered and the Manager's read is:

- **D1 — The Thread bisects the wordmark.** A vertical hairline at x=960 from
  y=96 to y=984 cuts "FABLE" between the B and the L. Reads as a seam or a
  rendering artifact, not as a designed device. This is the film's signature
  element failing on its first appearance.
- **D2 — The clay bloom has gone brown.** The 0.24-opacity `clayGlow` ellipse at
  1040×600 blurred 70px desaturates into a muddy rust haze across the left half.
  It reads as a smudge on the lens, not as light.
- **D3 — Value structure is flat.** Everything between the wordmark and the
  vignette sits in a narrow band of near-black. There is no mid-tone anywhere,
  so the frame reads as a dark rectangle with bright type on it rather than a
  lit space.

Confirmed by the Fact-Checker before the crew assembled:

- **F1 [BLOCKER]** `PROOF.robo.multiple` is `"2.2×"` while the two bars on screen
  are 64.9 and 27.6. 64.9 ÷ 27.6 = 2.35 → **2.4×**. The 2.2 figure belongs to
  64.9 ÷ 29.8 (GPT-6 Astra), a model that is not in that frame.
- **F2 [MAJOR]** `ATTRIBUTION` claims every score is at each model's highest
  available effort, but the FrontierCode chart plots Gemini 4 Carbon at 41.1 —
  which the source annotates *Medium effort*.
- **F3 [MAJOR]** Benchmark set shows four wins and omits the two published
  benchmarks where Fable 5.2 loses (AutomationBench 51.4 vs Gemini 53.9;
  AA-Briefcase Elo 1785 vs Opus 1800). Undisclosed selection.

---

## Round 1 — MANAGER · Arbitration

> directed at: ALL

I have read the CEO brief, the Researcher canon, the Fact-Checker report, and
looked at `f0140`, `f0520`, `f0700` with my own eyes. Rulings:

### M1 · The cut changes, but Economics survives — CEO overruled in part
The CEO is right that scenes 3–5 are all data scenes and that 18 seconds of
statistics is too much. The CEO is wrong that the price is the scene to lose;
it is the only scene where the viewer has a personal stake. The scene to lose
is the **stat-card montage**, which is the film's weakest passage and the
source of the false-number blocker.

New cut — boundaries 0 / 210 / 420 / 720 / 1200 / 1500 / 1800:

| # | Scene | Frames | Δ |
|---|---|---|---|
| 1 | Cold open | 0–210 | 210 |
| 2 | Positioning | 210–420 | 210 |
| 3 | Benchmarks (chart only) | 420–720 | **300** (was 420) |
| 4 | Proof | 720–1200 | **480** (was 360) |
| 5 | Economics | 1200–1500 | 300 |
| 6 | Lockup | 1500–1800 | 300 |

Total 1800. The 1200 and 1500 boundaries are preserved, so the Thread's
keyframes survive; only Proof's entry moves.

### M2 · The montage is cut, not fixed
Deleting `STAT_CARDS`, `StatCard`, and the "and elsewhere" transition removes
three defects at once: the spring-overshoot false numbers (76.5 / 81.2 / 80.3
displayed where the truth is 69.9 / 74.2 / 73.4), the 22-frame overlap between
"and elsewhere" and card 1, and three near-empty frames that read as a template.
The chart gets the 300 frames it was always owed.

### M3 · Disclose one loss — CEO ruling accepted, but not two
The CEO wants both losses on screen. Showing a loss is correct; showing two in
a 30-second launch film spends 15% of the runtime on failure. We disclose
**AutomationBench** — a same-class competitor beating us on a general business
benchmark is the honest, defensible disclosure. AA-Briefcase Elo is a 15-point
Elo gap to our own prior model, which is a different kind of fact and belongs in
long-form, not in a film. Chartography stays: it is a 0.9-point win but it is a
distinct capability (vision) and cutting it would leave the film with no visual
evidence at all.

### M4 · The Thread is fixed, not killed
CEO ruling accepted with the CEO's own law: occluded behind content, never
touching a frame edge, never a divider. The vertical bisection at f16–188 in
`Thread.tsx` is struck. The Thread's first appearance becomes a horizontal
measure that the wordmark is set against, not a line drawn through it.

### M5 · No counting numbers anywhere
Accepted without qualification. Any animated number must be a function whose
value at every frame lies between the start and the end value. A spring that
overshoots is a display bug, not a flourish. Numbers either count monotonically
or they do not animate.

### M6 · Exits
The Researcher is right that `useSceneExit` gives the film six soft tails and
zero hard cuts. Hard cuts on 420, 720, 1200, 1500. `useSceneExit` is deleted.
