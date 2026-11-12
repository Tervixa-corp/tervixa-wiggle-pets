# RESEARCHER — The Motion Canon

> directed at: MANAGER, DESIGNER, VERIFIER

30s / 1800f / 60fps / 120 BPM → **1 beat = 30 frames, 1 frame = 16.7ms**.
Every number below is in frames at 60fps. Where a claim is arithmetic, I show it.

Two things measured in the actual build before writing this (Remotion 4.0.542, fps=60):

| spring | peak | peak frame | settles (1%) |
|---|---|---|---|
| `theme.spring.smooth` (200/90/1) | 1.0000 | — | **42f** |
| `theme.spring.snappy` (26/170/0.8) | 1.0000 | — | 28f |
| `theme.spring.punch` (13/190/0.62) | **1.0946** | **13f** | 22f |

`punch` overshoots by **9.46%**. That number matters in A15 and in C.

---

## A. The canon — eighteen devices

### Type

**A1 · The single-clip line mask**
What: a whole line of type rises out of one mask, kerning intact.
Where: ColdOpen f58–f96 ("FABLE"), f132–f170 ("5.2"); Positioning f218–f252; Lockup f1536–f1592.
Numbers: 26 frames/line; expo-out `cubic-bezier(.16,1,.3,1)`; travel 118% of cap height; 6-frame stagger between *lines*, never between letters; **opacity stays 1 for the entire move**.
Build: one wrapper per line with `clipPath: inset(-0.24em -0.10em -0.32em -0.10em)`, inner block `translateY(p * 118%)`. Use `clipPath`, not `overflow:hidden` — overflow clips to the line box, and every headline in this film is set at `lineHeight: 0.86–1.0`, so the box is shorter than the glyph.
Fails as: per-letter masks. `MaskedText` (motion.tsx:51–74) puts each glyph in its own `inline-block`, which discards the font's kerning pairs — at 176px with `-0.052em` tracking, "FABLE" gains visible gaps at FA and LE; and the per-glyph `overflow:hidden` (motion.tsx:65) has no side padding, so italic overhang is shaved — Positioning's serif "work." at f272 will lose the right side of the k and the period. Second failure: fading while rising. The fade is the tell.

**A2 · Tracking settle**
What: the word assembles — letter-spacing closes as the mask lifts.
Where: ColdOpen f82–f112 (FABLE); Lockup f1560–f1600.
Numbers: 30 frames, three stops: `+0.012em` at f82 → `-0.030em` at f94 → `-0.052em` at f112, expo-out on the final segment only. Never overshoot past the design value.
Build: interpolate the `letterSpacing` string on the line wrapper. Cap it: ≤ 8 glyphs and ≤ 30 frames, or Chrome re-rasterises the text every frame and the stem weight visibly flickers.
Fails as: tracking that never stops (the word looks like it is breathing); or applying it to a sentence (words reflow and jump).

**A3 · The late word**
What: one word arrives 12–18 frames after the rest of its sentence, so the line has a pulse.
Where: Positioning f282–f302 ("work." lands after "technical" has settled at f278); Economics f1236–f1252 ("Smaller bill." against "Same work." at f1224).
Numbers: 12–18 frames of gap measured from the previous element's **landing**, not its start; same 20-frame mask; scale 1.0.
Build: derive the delay from the previous element's settle frame (`smooth`=42, `snappy`=28), not a hand-picked constant.
Fails as: the whole line arriving as a block (no rhythm, reads as a slide); or a gap > 24 frames (reads as a second thought, or a bug).

### Light

**A4 · Clipped-core bloom**
What: light with a visible source, a hard core, and a falloff that is a gradient — not a blur.
Where: ColdOpen f90–f206 (replaces the 1040×600 `clayGlow` ellipse at ColdOpen.tsx:72–84); Lockup f1540–f1800.
Numbers: core disc 220px, stops `rgba(201,100,66,0.55) 0% → 0.18 42% → transparent 68%`; second pass 640px at half alpha; both `mixBlendMode: screen`; **no `filter: blur` at all**; added luminance across the frame ≤ 12%.
Build: two absolutely-positioned divs. The radial-gradient's stops *are* the falloff. `blur()` on top of a gradient is what makes mud: it averages the accent with the dark field instead of adding light to it.
Fails as: D2 exactly. Warm accent at 0.24 alpha over `#04050A`, blurred 70px across 1040px, produces a ~500px ramp averaging to `#3A2420` — a smudge on the lens. Diagnostics: if you can see a soft *rectangle*, it's wrong; if the brightest point is more than 8px from the object it is supposed to be lighting, it's wrong.

**A5 · Halation fringe**
What: a warm bleed that exists only where the image clipped.
Where: Benchmarks f484–f520 (hero bar's leading edge); Economics f1368–f1400 (`−20%`); Lockup f1596–f1658 (specular pass).
Numbers: `blur(14px)`, `scale(1.004)`, `#FF3B10`, `screen`, alpha 0.10, present only while the source is in the top 5% of frame luminance; decays over 10 frames after the source settles.
Build: for text use `textShadow: 0 0 14px rgba(255,59,16,.35)` — cheaper than a duplicate and it respects glyph shape. For shapes, duplicate underneath the source so only the fringe shows.
Fails as: red fringing on *every* edge — the signature of a misregistered CRT, not a lens.

**A6 · Three-value plan**
What: the frame carries shadow / mid / highlight populations, so it reads as a lit space.
Where: measured at Benchmarks f510–f540 and Positioning f330–f360; implemented globally.
Numbers: shadow 2–5%, **mid 9–14%**, highlight 70%+. This film currently has no mid-tone (D3).
Build: one `AbsoluteFill` between Backdrop and content — `radial-gradient(ellipse 70% 55% at 38% 62%, rgba(38,48,66,0.16), transparent 70%)` plus a narrow bounce on the opposite edge at 0.05. No blur.
Fails as: bright type on a black rectangle. Test: downsample the render to 64×36; if fewer than 8% of pixels fall between 20 and 90 of 255, the frame is flat.

**A7 · Rim light**
What: a 1px brighter edge where the front object overlaps the back one.
Where: Benchmarks f484–f540 (hero bar crossing the four grey bars); Economics f1240–f1400 (ledger rules under the figures).
Numbers: 1px, `rgba(242,239,233,0.22)`, on 2–3 elements per frame maximum, fades in over 8 frames, never animates after.
Build: `boxShadow: inset 1px 0 0 rgba(...)` on an `overflow:hidden` element, or a 1px gradient div.
Fails as: an outline on everything (wireframe); or a rim on something that overlaps nothing (no occlusion, so no depth — it just looks drawn on).

### Depth

**A8 · Atmospheric falloff, not blur**
What: depth built from contrast and saturation, not from Gaussian blur.
Where: Positioning f210–f420 (pills at three depths); Benchmarks f420–f540 (grid behind bars).
Numbers: back layer contrast ×0.55, saturation ×0.70; mid ×0.78 / ×0.88; front 1.0 / 1.0. Blur only if the layer moves > 30px/frame.
Build: `filter: saturate(.7) contrast(.55)` on the back wrapper, or pre-bake the colours. Blur is the last resort because Chrome pays for it per frame at 1920×1080.
Fails as: portrait mode — everything behind the subject is Gaussian-blurred, which reads as a phone photo, not a lens.

### Transitions

**A9 · The hard cut**
What: frame 209 full, frame 210 full. Zero overlap.
Where: f210, f420, f840, f1200, f1500 — all five.
Numbers: 0 frames of overlap; 0 blur; 0 fade.
Build: delete `useSceneExit` from all six scenes. Each `Sequence` already ends cleanly; the dissolve is the only thing preventing a cut.
Fails as: the current state. `useSceneExit` (motion.tsx:179–187) puts 14 frames of fade + 46px rise + **7px blur** on the tail of every act — f196–209, 406–419, 826–839, 1186–1199, 1486–1499. So the picture is soft and dim for 233ms before every hit, then dim again for ~40 frames of entrance on the other side. That is a ~54-frame soft shoulder around every cut. The 120 BPM grid is currently a suggestion because the picture never arrives at full contrast on a beat.

**A10 · Cut on motion**
What: cut 4–8 frames before the outgoing move finishes, so velocity carries across the edit.
Where: f840 (the third stat card is still 6 frames from settling); f1500 (the Economics footer is still rising).
Numbers: cut at 4–8 frames before settle; incoming move must continue the same direction within 2 frames.
Build: shorten the scene by 6 frames instead of adding an exit. The card sequence ends at 834; the cut is at 840; the card never reaches rest.
Fails as: cutting on a settled frame (dead cut); or cutting mid-motion in the opposite direction (reads as a mistake).

**A11 · The two-frame black**
What: a true black gap that turns a cut into a hit.
Where: f839–f840 and f1199–f1200 only. Not at 210, 420, 1500.
Numbers: exactly 2 frames (33ms) at ≤3% luminance, hard in, hard out. 1 frame is invisible at 60fps; 3 frames starts to read as a dropout.
Build: a black `AbsoluteFill` at `opacity: frame === 839 ? 1 : 0`.
Fails as: fading to black (reads as the end of a scene, not a hit); holding 4+ frames (reads as a render error).

**A12 · The Thread-as-wipe**
What: the signature hairline *is* the transition edge — content is revealed by a clip that tracks it.
Where: f1190–f1258 (Proof rule at y=300 descends to Economics underline at y=814); also f186–f230.
Numbers: 68 frames of travel, 514px, `inOut`; reveal edge within 0px of the Thread; content underneath already at final position for the first 12 frames (no entrance animation competing with the wipe).
Build: export the `KEYS` sampler from `Thread.tsx` as a pure `sampleThread(frame)` and call it from a `clipPath: inset(0 0 calc(100% - ${y}px) 0)` on the incoming scene. It is a pure function of frame, so it is safe in Remotion.
Fails as: `Sweep` (Overlays.tsx:79–105) — a light band crossing over a transition that has already happened. Decoration on top of an event. If the wipe edge and the light are not the same object, don't do it. Note also that `CUTS` omits 210, so the device fires at four of the five boundaries — an inconsistency the eye registers without knowing why.

### Time

**A13 · The eighteen-frame hold**
What: the subject stops. Nothing enters, nothing leaves.
Where: Benchmarks f510–f540 (30f, at the chart's peak); Proof f906–f934; Economics f1440–f1470; Lockup f1710–f1770 (60f).
Numbers: 18–60 frames. **Subject motion zero; grain, gate weave and the Thread keep moving.** A fully frozen frame reads as a stalled render.
Build: clamp every interpolate/spring input with `Math.min(frame, holdStart)` for the held elements; leave the overlay stack live.
Fails as: freezing the grain too (looks like a JPEG); holding *before* the payload (kills the payload); holding 90+ frames in a 30s film (looks like a bug).

**A14 · The late payload**
What: the biggest number arrives after everything around it has stopped.
Where: Proof f888–f906 (`2.4×` lands 12 frames after the last bar settles at 892); Economics f1368–f1390.
Numbers: 10–14 frames of nothing between setup-complete and payload; payload gets 8 frames of move then 40+ of hold.
Build: compute the payload delay from the measured settle of the preceding animation (42f `smooth`, 28f `snappy`), not a guessed constant.
Fails as: the payload landing while the setup is still moving (no contrast, no read); or a 30-frame gap (the audience starts reading the source line instead).

### Data

**A15 · The non-overshooting number**
What: a figure that only ever moves toward the truth.
Where: every number in the film — Benchmarks f484 (49.1), f540/f630/f720 (cards), Proof f892, Economics f1274–f1360.
Numbers: **0% overshoot, always.** 30-frame count, expo-out; the final 10 frames pinned to the exact target string. Optional odometer: last digit only, 14 frames, `translateY(-d × 1em)` in a masked 10-cell column.
Build: `const v = target * Math.min(1, p)` — clamp. Or use `interpolate` with `extrapolateRight: "clamp"` for anything numeric.
Fails as: **verified defect.** `StatCard` (Benchmarks.tsx:235, 237, 262) springs with `theme.spring.punch`, measured peak **1.0946 at frame 13**. So `live = value * p` renders **76.5%** at f620–630 where the truth is 69.9, **81.2%** at f693–703 where it is 74.2, **80.3%** at f766–776 where it is 73.4. That is ~30 frames of false figures in a film whose entire job is to be believed. It is not a style note; it is the same class of error as F1.

**A16 · The weighted bar**
What: a bar that arrives with mass and stops exactly on its value.
Where: Benchmarks f452–f506; Proof f856–f892.
Numbers: 26 frames, expo-out, 7-frame stagger, no overshoot; row label at 22px fades in at 55% of growth; value rides the leading edge with a 2-frame lag at `x = width + 20`.
Build: width in px from `progress()`. Never `scaleX` on a bordered element — Benchmarks.tsx:144 borders the non-hero bars, and `scaleX(0.5)` would render a 2px border where 1px was designed.
Fails as: bars that spring past the value and settle back (a lie, and it reads as a toy); all bars starting on the same frame; value labels that jump rather than ride.

### Texture

**A17 · Grain that resolves**
What: noise at the right scale, alive in the shadows.
Where: whole film; judged at Benchmarks f510–f540 and Lockup f1710–f1770.
Numbers: feature size 1.6–2.2px at 1920 (`baseFrequency ≈ 0.55–0.70`, `numOctaves: 1` — the current 0.9/3 gives sub-pixel noise that aliases under H.264); opacity 0.045–0.065; blend **`soft-light`, not `overlay`** (overlay preserves black, so grain disappears precisely where this film is darkest); 8-frame cycle at prime offsets (0, 37, 71, 113, 149, 191, 233, 277 px) so no repeat is legible.
Build: pre-render eight 512×512 transparent PNG tiles and cycle `backgroundImage` on `frame % 8`.
Fails as: a visible 220px tile across 8.7 repeats; the current `frame*7 / frame*13` translate makes the grain *scroll*, and the eye locks onto that diagonal within two seconds; grain visible only in the highlights, which is the tell of an overlay-blended layer over near-black.

**A18 · Sub-frame smear**
What: motion blur that is honest to the shutter, applied only where displacement is real.
Where: Economics f1368–f1378 (`−20%`); Benchmarks f484; Thread f1190–f1200.
Numbers: only when displacement > 18px/frame. Three samples at shutter offsets 0 / 0.42 / 0.85, opacities 0.30 / 0.45 / 1.0, outer two `blur(1.2px)`.
Build: `@remotion/motion-blur`'s `<Trail>` is already in package.json and unused — or three stacked copies with per-copy `translateX(-v * offset)`. Costs 3× the DOM; disable below 18px/frame.
Fails as: uniform blur (out of focus); a long ghost trail (double exposure); blur on type that isn't moving fast.

---

### Render economics — what headless Chrome will charge you for

The film is 1800 frames. Three things are currently re-computed per frame at 1920×1080 and will dominate render time:

1. `Backdrop` runs `filter: blur(90px)` and `blur(110px)` on divs up to 2120px across, every frame. The source is already a radial-gradient — the blur buys almost nothing and costs a full-frame filter. **Replace with a 4th gradient stop.** Same look, no filter.
2. `Thread` runs an SVG `feGaussianBlur stdDeviation=14` across a full-frame `<svg>` every frame. **Bake the glow once as a 1920×1080 PNG sprite** and animate `opacity` + `scaleY`, or draw the glow as three stacked lines at 6px/14px/28px with declining alpha. A gaussian on a known shape is cacheable; a filter primitive is not.
3. `Grain` regenerates `feTurbulence` surface area every frame. See A17 — eight PNGs.

Real-time WebGL is out (headless Chrome falls back to SwiftShader on CPU at 1–2fps; 1800 frames is not viable). If any device needs a shader — a displacement wipe, a fluid, a real bloom pass — **pre-render it to a PNG sequence at 30fps and step two frames per output frame**, or bake a single texture and animate it with `transform` only. Nothing in this film needs a shader; A4 and A5 get you bloom and halation for free in CSS.

---

## B. Three devices this film is missing

The film is not too busy in *content*. It is too busy in *ambient motion*: six independent sinusoids (26, 34, 44, 48, 63/71/79/88, 6.1/7.3) plus scrolling grain plus gate weave means there is not one frame in 1800 where the image is at rest. That is where the "generated" feel comes from — not from density. So all three additions below **subtract** something.

**B1 · The empty beat — RANK 1**
Where: f826–f840 (before the Proof cut) and f1178–f1200 (before Economics).
Replaces: the tail of the third stat card, and the `Sweep` run-in.
Numbers: 14 frames where the frame holds ≤8% luminance and the **only** element is the Thread. No type. No card. No sweep.
Why: contrast is manufactured by what precedes it. The film's two biggest claims — `2.4×` at f888 and `−20%` at f1368 — both currently land after 100+ frames of dense content, so they have nothing to be bigger than. Fourteen frames of near-black buys both of them their weight, costs 0.23s, and is the cheapest change on this page. In the cut, this is the beat where the audience inhales.

**B2 · The cropped hero — RANK 2**
Where: ColdOpen f82–f210. "FABLE" reset at 340px, baseline y=760, cropped top and bottom by the frame edge.
Replaces: the 176px centred wordmark at ColdOpen.tsx:126–139.
Numbers: cap height 247px at 340px Inter 800; the B's counter sits on the optical centre (y=540); the right edge of the E breaks the safe margin by 60px. One grid break, once, in the whole film.
Why: it fixes D1 by changing the argument. A vertical hairline through a 176px caption is a seam; a vertical hairline through a 340px cropped poster is a **measurement mark** — the Thread reads as instrumentation because the type is no longer the same size as the device. It also gives the cold open the only thing it currently lacks: scale.

**B3 · The captured substrate — RANK 3**
Where: Economics f1200–f1500.
Replaces: the flat `#080A11` field behind the ledger rows.
Numbers: one 1920×1080 scan of 90gsm paper, `mixBlendMode: multiply`, opacity 0.05, `contrast(1.08)`; ledger rules drawn with a per-rule vertical jitter of ±0.4px and per-rule alpha between 0.09 and 0.14, so no two rules are identical.
Why: there is not a single frame in this film that looks *recorded*. Every pixel is vector. The ledger is the one scene where a physical document is diegetic, and the price cut is the one claim a viewer is most likely to screenshot. Five percent of paper fibre under it converts "a slide of prices" into "a document someone is showing you" — and the jittered rules do the same for the hairlines, which currently read as CSS because they are perfect.

---

## C. Two devices this film should stop using

**C1 · `useSceneExit` — the dissolve that eats every cut** (motion.tsx:179–187)
Fourteen frames of fade + 46px rise + 7px blur, applied to the whole scene, at the end of all six acts: f196–209, 406–419, 826–839, 1186–1199, 1486–1499, 1782–1800.
The damage: at 60fps that is **233ms of the picture going soft before every boundary**, on a film whose score is cut on a 120 BPM grid. Blurring 176px type by 7px is the single most template-looking move available, and doing it five times removes every hard edge the edit has. Combined with ~40 frames of entrance on the far side, each cut has a ~54-frame soft shoulder. The grid exists for hits; the picture is never there to meet them. **Delete it. Replace with A9 + A10 + A11.**

**C2 · `Entrance` as the universal default** (motion.tsx:17–45)
One choreography — 44px rise, 0.965 scale, fade — on `spring.smooth`, which measured **42 frames to settle (0.7s)**. It is applied to the 176px hero, the 82px headline, the 13px pills, the 12px mono labels and the 11px source line. Consequences: (a) hierarchy is carried entirely by type size, because every element moves identically; (b) a 13px label travelling 44px moves **3.4× its own height**, which is the visual signature of a component library, not a decision; (c) 42 frames is **1.4 beats**, so anything that starts on a beat lands 12 frames into the *next* one — nothing in this film can land on a beat by construction.
**Restrict it to one element per scene.** Type gets A1; micro-labels get a hard cut or an 8-frame opacity only; numbers get A15.

> **Not a device — a bug, and a blocker.** The same spring instinct is applied to a *quantity* in `StatCard`. `theme.spring.punch` peaks at 1.0946 on frame 13, and `live = value * p` multiplies the score by it: 69.9 renders as 76.5, 74.2 as 81.2, 73.4 as 80.3, for ~10 frames each, ~30 frames total. Fix before anything else in this document is worth doing.

---

## D. The timing table — 60 beats

**Legend.** `CUT` hard cut on the beat · `LAND` text/number reaches final position · `STILL` nothing moves but grain, weave and the Thread · `HOLD` one element carries, no new event · `SILENT` no music, no room tone, no SFX · `EMPTY` ≤8% luminance, Thread only.

| # | Frames | Picture | Sound | Flag |
|---|---|---|---|---|
| 01 | 0–29 | True black to f13. Date rail draws L/R f14–f29 | **nothing** | **SILENT · EMPTY** |
| 02 | 30–59 | Date lands f34. "CLAUDE" masks f38–f60 | sub swell from f46 (41Hz, −30dB) | LAND |
| 03 | 60–89 | FABLE line-mask f58–f82 | sub only, no transient | LAND f82 |
| 04 | 90–119 | Bloom ignites f90–f120 (A4) | first hit f90, low-passed | SFX |
| 05 | 120–149 | "5.2" masks f132–f158 | pulse 120, 150 | — |
| 06 | 150–179 | Technical footer f160–f190 | pulse 150, 180 + filtered noise rise | LAND |
| 07 | 180–209 | Wordmark complete. **Nothing moves** f186–209 | pulse 180 only | **STILL** |
| 08 | 210–239 | **CUT.** Eyebrow + line 1 f218 (A1) | full groove enters, kick 210 | **CUT** |
| 09 | 240–269 | "most ambitious" lands f246 | kick 240 | LAND |
| 10 | 270–299 | "technical" masks f254–278 | kick 270, hats 16ths | LAND f278 |
| 11 | 300–329 | "work." masks f282–302 (A3) | kick 300 + single bell | LAND f302 |
| 12 | 330–359 | Four pills f330–f342, 4f apart | four 16th clicks | — |
| 13 | 360–389 | Supporting line word-reveal f328–390 | bass drops out | — |
| 14 | 390–419 | Hold. Pre-cut | 4-beat riser 390–420 | **STILL** |
| 15 | 420–449 | **CUT.** Chart header masks f424–446 | hit 420, half-time | **CUT** |
| 16 | 450–479 | Bars 1–5 start f452/459/466/473/480 (A16) | one tick per bar start | — |
| 17 | 480–509 | Bars settle by f506. Hero value lands f484 (A15) | tick 484 | LAND |
| 18 | 510–539 | **Chart held. Nothing moves.** Source line f512 | pulse only | **STILL** |
| 19 | 540–569 | **CUT** to card 1: 69.9 | hit 540 | **CUT** |
| 20 | 570–599 | Card 1 static | pulse | HOLD |
| 21 | 600–629 | Card 1 static, last 8f dead | reverse swell | **STILL** |
| 22 | 630–659 | **CUT** to card 2: 74.2 | hit 630 | **CUT** |
| 23 | 660–689 | Card 2 static | pulse | HOLD |
| 24 | 690–719 | Card 2 static | pulse | — |
| 25 | 720–749 | **CUT** to card 3: 73.4 | hit 720 | **CUT** |
| 26 | 750–779 | Card 3 static | pulse | HOLD |
| 27 | 780–809 | Card 3 static | pulse thins | — |
| 28 | 810–839 | Card gone. Thread only (B1) | **near-silence**, one low tone | **EMPTY · SILENT** |
| 29 | 840–869 | **2f black 839–840. CUT.** Proof header | hit 840 | **CUT** |
| 30 | 870–899 | Robo label + two bars f856/f866 (A16) | two ticks | — |
| 31 | 900–929 | **`2.4×` lands f902** (A14) | hit 902 | LAND |
| 32 | 930–959 | Hold | pulse settles | **STILL** |
| 33 | 960–989 | **CUT.** Fermat. Dot grid assembles f974–f998 | hit 960 | **CUT** |
| 34 | 990–1019 | Grid consolidates, wave passes | granular texture | — |
| 35 | 1020–1049 | Stats land f1024 / 1033 / 1042 (A3) | three ticks | LAND |
| 36 | 1050–1079 | Hold. Tokens line f1056 | pulse | HOLD |
| 37 | 1080–1109 | **CUT.** Rust. Two tracks f1080 | hit 1080 | **CUT** |
| 38 | 1110–1139 | Rust fills f1110–f1125; Python crawls | two ticks | — |
| 39 | 1140–1169 | **`26×` lands f1146** | hit 1146 | LAND |
| 40 | 1170–1199 | Hold. Thread starts descent f1190 (A12) | tail | **STILL** |
| 41 | 1200–1229 | **2f black 1199–1200. CUT.** Economics title f1208 | hit 1200 | **CUT** |
| 42 | 1230–1259 | Ledger rule 1 + label f1240 | paper tick | — |
| 43 | 1260–1289 | Old price struck f1266 | scratch | — |
| 44 | 1290–1319 | Row 1 counts f1274–f1318 (A15) | ticking | LAND f1318 |
| 45 | 1320–1349 | Rule 2 + row 2 f1322 | pulse | — |
| 46 | 1350–1379 | Row 2 counts 1316–1360. **`−20%` lands f1368** | hit 1368 | LAND |
| 47 | 1380–1409 | `−20%` held | sustained | HOLD |
| 48 | 1410–1439 | Secondary economics footer f1410 | pulse | — |
| 49 | 1440–1469 | Hold | pulse thins | — |
| 50 | 1470–1499 | Pre-cut. Nothing moves | riser 1470–1500 | **STILL** |
| 51 | 1500–1529 | **CUT.** Rail draws f1506 | **nothing** | **CUT · SILENT** |
| 52 | 1530–1559 | Eyebrow f1522. FABLE masks f1536–f1576 (A1) | single low tone f1536 | — |
| 53 | 1560–1589 | FABLE complete. "5.2" f1578 | — | LAND f1576 |
| 54 | 1590–1619 | "5.2" completes f1618. Thread underline f1562 | chord opens f1600 | LAND |
| 55 | 1620–1649 | Model ID + platforms f1618–f1660 | — | — |
| 56 | 1650–1679 | Specular pass 1596–1658 (A5) completes | shimmer | — |
| 57 | 1680–1709 | Attribution f1686 | tail | — |
| 58 | 1710–1739 | **Nothing moves** | pad only | **STILL** |
| 59 | 1740–1769 | **Nothing moves.** Bloom at final value | pad decaying | **STILL** |
| 60 | 1770–1799 | Hold. Fade to black 1788–1800 | silence by f1790 | **STILL · SILENT** |

**Budget.** SILENT: beats 1, 28, 51, plus the tail of 60 — **~100 frames, 1.7s, 5.5% of the film with no sound at all.** STILL: beats 7, 18, 21, 32, 40, 50, 58, 59 — **8 beats, 240 frames, 4s, 13% of the film with no subject motion.** Both are deliberate. Every one of them will be proposed for deletion in the first review pass; that is the point.

**Three fixes the table encodes, against the current build:**
1. The chart currently punches its hero value at **f594** — 88 frames (1.47s) after the last bar settled at f506. That is dead air in the middle of the film's most important chart. Move the punch to f484 (table beat 17) and spend the recovered 110 frames on beats 18–21.
2. `ChromaticText` "and elsewhere" runs f616–f641 and **collides with stat card 1 (f620–f692) for 22 frames**, both centred. Delete the device — the hard cut at f540 does its job.
3. There is no `Sweep` at f210 and f0. If the device is worth keeping it fires at every boundary; since A12 replaces it with a motivated wipe, it fires at none.
