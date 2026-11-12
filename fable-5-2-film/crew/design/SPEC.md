# DESIGN SPEC — FABLE 5.2 / 30s
### Designer · art direction, frame geometry, motion timing
### Supersedes: every previous design note. Does not supersede `BOARD.md` Round 1.

Every number below is a build instruction. There are no ranges, no "close to", no
"depending on". If a number is uncomfortable, change the design, not the number.

---

## 0. The cut on disk is not the cut on the board

Before a single pixel is judged: **the arbitration has not been applied.** All six
stills in `out/stills` were rendered from the pre-arbitration cut. That is why the
board and the frames disagree about what scene is where.

| File | Line | Holds | Must become |
|---|---|---|---|
| `src/video/Film.tsx` | 22 | benchmarks 420f | 300f |
| `src/video/Film.tsx` | 23 | proof 840f | 480f, from 720 |
| `src/fx/Overlays.tsx` | 73 | cuts `[420,840,1200,1500]` | `[420,720,1200,1500]` |
| `src/scenes/Benchmarks.tsx` | 6 | `DURATION 420` | `300` |
| `src/scenes/Benchmarks.tsx` | 296–309 | 3 stat cards at 200/273/346 | delete |
| `src/scenes/Proof.tsx` | 7–8 | `360`, `VIGNETTE 120` | `480`, vignettes 150/180/150 |
| `src/fx/Backdrop.tsx` | 30 | phase at 840 | 720 |

`useSceneExit` (M6: deleted) is still called in all six scenes at lines
`ColdOpen:29, Positioning:12, Benchmarks:294, Proof:448, Economics:143, Lockup:97`.
Total runtime today is 1860 frames, not 1800.

**Frame map for the six stills.** Only one of them moves:

| Still | Current content | New frame for the same content |
|---|---|---|
| f0140 | ColdOpen local 140 | 140 |
| f0520 | Benchmarks local 100 | 520 |
| f0700 | stat card 2, Benchmarks local 280 | cut — no equivalent |
| f0900 | Proof local 60 (Robo) | **780** |
| f1300 | Economics local 100 | 1300 |
| f1700 | Lockup local 200 | 1700 |

Proof can no longer be 160/160/160. Fermat's held breath is 60 frames and the
brief will not give it up; the only allocation of 480 that keeps it is
**Robo 150 (720–870), Fermat 180 (870–1050), Rust 150 (1050–1200)**.
The Brief's Moment 3 frames (1092–1152) were written against the 4c table, where
Proof began at 900 and Fermat at 1080. In this cut 1092 is 42 frames into Rust.
The Brief is wrong on those frame numbers, not on the idea.

One more correction to the board: **D2 is under-diagnosed.** The cold-open halo is
not the only source of the brown. `Backdrop.tsx:27–32` sends a blurred clay bloom at
0.105 opacity behind every scene, and `Overlays.tsx:45` lays 16% clay soft-light over
the whole film. Fixing `ColdOpen.tsx:72–84` alone leaves the rust in place.

---

## 1. Diagnosis

### 1.1 `f0140` — the cold open, mid-reveal

**The most damaging thing is not the Thread. It is that the film calls the product
"FABLE 5".**

`MODEL.version` is `"5.2"` (`benchmarks.ts:31`). `ColdOpen.tsx:140–153` reveals it
with `delay={132} per={4}`, so at f140 character 0 ("5") is 31% through its mask and
"." and "2" have not started. The still shows a complete clay **5**, no point, no 2.
A viewer at 2.3 seconds concludes the product is Fable 5. That is a different SKU.
The Thread is the second defect, not the first.

The Thread at `x=960, y=96→984, width 1.4, opacity 0.9` (`Thread.tsx:15`) crosses
three text registers in one frame: the date rail at y=236 (`ColdOpen.tsx:45`, text
spans x 875–1045), the eyebrow's tracking at y=352 (`ColdOpen.tsx:90`), and the
wordmark at y=412–570 (`ColdOpen.tsx:116`, 176px at `:133`). At the wordmark it
passes through the 7px counter between B and L. One hairline through three type
registers does not read as a device; it reads as a render fault.

**Fix:** delete the vertical keyframe, reveal "5.2" as one masked unit at f130
(one transform, one mask, `dur=22`), and re-seat the Thread as a horizontal measure
at y=412 (§4).

### 1.2 `f0520` — the benchmark chart

**The inactive bars read as empty form fields.**

`Benchmarks.tsx:9–10`: `BAR_H 54`, `ROW_GAP 26`. `:139` radius 2. `:144` a 1px
`theme.colors.hair` border on **all four sides**. `:142` fill `rgba(242,239,233,0.075)`
— #1A1B1F on #080A11, lighter than the room, darker than the type. `:152` the label
inset `left: 20`, 22px, weight 500, optically centred. That is the exact anatomy of a
text input with placeholder text. A viewer concludes this is a settings panel, not a
result. The chart's own data is then set at 30% opacity (`:174`, `theme.colors.mute`)
— the numbers are the dimmest thing in the frame — while a 44px + 90px clay shadow
(`:143`) blooms around the hero and washes the "49.1%" sitting 20px off its edge.

**Fix:** label out of the bar, solid lit fill, no border, and the glow cut to one
24px / 0.35 shadow. Full geometry in §2.

### 1.3 `f0700` — the stat card being cut

**The frame is a slot machine: it prints 59.3% for a 74.2% claim.**

`Benchmarks.tsx:237` — `const live = value * p;` where `p` is
`spring(..., theme.spring.punch)` (`:235`). Card 2 begins at local 273 (`:296`) and
carries 74.2 (`benchmarks.ts:107`). At f700 (local 280) the punch spring is at 0.799,
so the frame reads 74.2 × 0.799 = **59.3**. Every number in this film is currently
multiplied by a spring; the same line exists at `Benchmarks.tsx:181`,
`Proof.tsx:162` and `Economics.tsx:134`. This is not a montage problem. It is a
house-method problem.

The 176px numeral with an 80px `iceGlow` text-shadow (`:259`) is also the clearest
evidence that the frame is a template: three centred lines, 70% empty, one halo.
Cutting the montage is correct. Do not port `live = value * p` anywhere else.

### 1.4 `f0900` — proof vignette 1

**The claim is cut mid-word.**

`Proof.tsx:40` puts the Label at `left: 260, top: 372`; `:64–65` sets the title at
46px/600; `:57–61` reveals it per character with `delay={6} per={1.7} dur={20}`.
"More than double, one generation on." is 36 characters. The reveal needs
6 + 35 × 1.7 + 20 = **85.5 frames** of a **120-frame** vignette. At f900 (local 60)
only 21 characters have landed; the still stops at "one gener". A viewer concludes
the render is broken. This costs more than the false number beside it, because it is
the first thing the eye hits.

Then the number: `PROOF.robo.multiple` is `"2.2×"` while the two bars are 64.9 and
27.6 (`Proof.tsx:95–96`). 64.9 ÷ 27.6 = 2.351 → **2.4×**. The 27.6 bar also prints
**27.5** at that frame (`:162`, the same `value * p` bug). And the Thread at y=300
(`Thread.tsx:22`) runs the full width above the content as a rule — a divider,
which M4 forbids.

**Fix:** sentence-level reveals are single-line masks, never per-character (§7);
2.4×; values resolve, never count (§2.3); the Thread becomes a baseline (§4).

### 1.5 `f1300` — economics

**One third into the scene, the frame is 40% empty and its hero object has not been
invented yet.**

The ledger runs top 330, two rows of 132 (`Economics.tsx:34`), a closing rule at
594 (`:221`). The −20% sits at `top: 596` with `opacity: clamp01(cut * 1.3)` (`:235`)
where `cut` is a spring starting at frame **168** (`:146`). At f1300 (local 100) it is
exactly zero. So the entire bottom half — 596 to 1080 — is blank except the footer at
872. A viewer concludes the slide is unfinished.

The change is also 400px wide and empty: old price right-aligned at x 1060 (`:73`),
arrow at x 1120 (`:102`), new price right-aligned at x 1460 (`:117`). The eye travels
340px of nothing to find the payoff. The old price is `boneFaint` 0.26 (`:80`) — a
1.9:1 contrast — and the 2px clay strike at 0.85 (`:90–94`) is brighter than the
number it cancels.

**Fix:** the delta becomes a dimension bracket spanning 1100→1460 at y=604 and the
−20% sits on it; the scene's hero enters at 1264, not 1368 (§4, §6).

### 1.6 `f1700` — the lockup

**The attribution is 9.5px at 0.16 effective alpha, 42px from the bottom edge.**

`Lockup.tsx:316` fontSize 9.5, `:320` `opacity: attrIn * 0.62` on `boneFaint` 0.26,
`:311` `bottom: 42`. Effective alpha 0.161. On a 1080p master that is a 5pt line at
1.4:1. A viewer concludes the claims are legally buried — on the one film whose whole
argument is that it tells you where it loses. This is the most damaging frame in the
film because it damages the argument, not the picture.

The composition compounds it: the availability pill at y=248 (`:138`), then 392px of
nothing, then CLAUDE at 640 (`:216`), wordmark at 690 (`:28`) and a 60px pile
(API 922, platforms 1000, footnote 1038) at the bottom. The film's last appearance of
its own name is 158px (`:10`) — smaller than the 176px it used at f0140 and smaller
than the −20% in Economics.

**Fix:** 196px wordmark, the stack re-spaced 560→969, the footnote at 11px/0.52,
the Thread retracting into the 5 (§5).

---

## 2. The chart, rebuilt

Scene: global 420–720, local 0–300. Chart only. No cards, no "and elsewhere", no
exit. Hard cut at 720.

### 2.1 The plot

| Quantity | Value |
|---|---|
| Plot x origin | 420 |
| Plot x end | 1580 |
| Plot length | 1160 |
| px per unit | 23.2 |
| Axis y | 800 |
| Bar height | 54 |
| Row gap | 38 |
| Rows | 5 |
| First row top | 378 |
| Last row bottom | 800 |
| **Plot vertical extent** | **378 → 800 (422px)** |
| Row tops | 378 / 470 / 562 / 654 / 746 |
| Row centres | 405 / 497 / 589 / 681 / 773 |

`MAX_VAL` stays 49.1. At 23.2 px/unit the hero lands at x 1559.1 and the 50 line at
1580: **21px of air between the best result and 50.** That gap is the story of the
frame. Do not normalise the axis to make the bar fill it.

### 2.2 The bar

**Inactive bar**
- Fill: vertical gradient `#6B727E 0%` → `#4A505B 100%`.
- Border: none. The only edge is a 1px top highlight `rgba(255,255,255,0.16)` inside
  the top edge.
- Radius: 1.
- Height: 54.
- Width: `value × 23.2`.
- No shadow.

The bar is a **solid object**, not a container. Its fill is 3.6× the luminance of the
track it sits in, so the bar end is the only edge the eye needs. Nothing is outlined.

**Track (the unfilled remainder)**
- x 420, width 1160, height 54, at the row top.
- Fill `#12161E`. No border. Radius 1. Inset top shadow `0 1px 0 rgba(0,0,0,0.55)`.
- The track exists so the bar has a groove, not so the frame has five empty slots.
  It is 4 L\* points above the well and 24 below the bar.

**Hero bar**
- Same geometry. Fill: `linear-gradient(90deg, #B85736 0%, #D2714A 100%)`.
- 1px top highlight `rgba(255,225,210,0.30)`.
- **Glow: exactly one shadow, `0 0 24px rgba(201,100,66,0.35)`.** 24px radius, 0.35
  opacity, no second pass, no 90px bloom, no text-shadow on the value. The glow begins
  at local 182 and reaches 0.35 at local 200.
- The hero's light end `#D2714A` is L\* 58.0, 25.3% of bone's luminance. It never
  competes with the title.

**Warm phase (Moment 2, local 182–200)**
- Inactive bar top `#6B727E → #606772`; bottom `#4A505B → #424852`. That is exactly a
  20% luminance drop, not a hue change.
- Well top `#2C313B → #342C27`; well bottom `#10141C → #17120F`.
- The hero bar does not change. It is the source of the warmth.
- Nothing is blurred. The temperature change is a fill interpolation on a flat
  surface, which is why it cannot go brown.

### 2.3 Label and value

**Label — out of the bar, in the gutter.**
- Right-aligned at x=376, left edge x=150 (safe margin), width 226.
- Top = row centre − 11. Height 22.
- Hero: 21px / 600 / `#F2EFE9`. Others: 21px / 500 / `rgba(242,239,233,0.62)`.
- Letter-spacing −0.012em. No uppercase. The model name is a name.

Placing the label in the gutter is the single change that kills the form-field read.
A label cannot be inside a bar and outside a form.

**Value — rides the edge, never counts.**
- Left edge = `420 + barWidth + 24`. Top = row top + 12. Box 96 × 30.
- Hero: 26px / 600 / `#F2EFE9`. Others: 26px / 600 / `rgba(242,239,233,0.86)`.
- Tabular numerals, −0.02em.
- The string is **the final value from its first visible frame**. There is no
  `value × p`, no `toFixed` on a live variable, no counter.
- Opacity = `clamp01((p − 0.72) / 0.28)` where `p` is the same monotone progress that
  drives the width (ease-out, 32 frames). The number is absent until the bar is 72%
  grown and fully on at 100%. At every frame it is either invisible or true.
- Because the value's x is a function of the width, it travels with the growing edge,
  then stops with it. At rest the five values trace the same ragged contour as the
  five bar ends. That is the point: two readings of one shape.

Rest positions: hero x 1583, Opus 1500, GPT 1437, Gemini 1398, Fable 5.1 1196.

**Scale factor.** No spring ever drives width. `theme.spring.snappy` on width
overshoots 1–2%, which makes a 49.1 bar 50.0 long for a few frames. Width and value
opacity both read from `progress(frame, start, 32, theme.ease.out)`.

### 2.4 Axis, grid, source, disclosure

The current frame is bottom-empty, not top-heavy: plot ends at 790, source at 862,
and 218px of nothing below. The rebuilt frame gives the bottom a job — a footer rail
carrying the source and the loss — so the composition reads top (claim), middle
(evidence), bottom (provenance).

| Element | Geometry | Treatment |
|---|---|---|
| Well (lit plate) | x 384, y 362, w 1232, h 476 | `linear-gradient(180deg,#2C313B 0%,#10141C 100%)`, radius 2, 1px top edge `rgba(242,239,233,0.10)` |
| Axis | x 420→1580, y 800, h 1 | `rgba(242,239,233,0.28)` |
| Gridlines | x 652/884/1116/1348/1580, y 368, h 432, w 1 | `rgba(242,239,233,0.075)` |
| Ticks | x = gridline − 0.5, y 800, w 1, h 6 | `rgba(242,239,233,0.28)` |
| Tick labels | centred on gridline, y 818, h 14, w 32 | mono 11 / 400 / `rgba(242,239,233,0.52)`, tracking 0.08em |
| Footer rule | x 150→1770, y 852, h 1 | `rgba(242,239,233,0.10)` |
| Source | x 150, y 866, h 14 | mono 11.5 / 400 / `rgba(242,239,233,0.55)`, tracking 0.10em |
| Loss line | x 150, y 892, h 26 | sans 19 / 500; numbers `#F2EFE9`, label `rgba(242,239,233,0.72)` |
| Loss tick | x 150→168, y 904, h 1 | `rgba(242,239,233,0.30)` |

Grid values are 0/10/20/30/40/50, not 25/50/75. The reader needs to place 49.1
against 50; quarter-grids make that impossible and were invisible anyway
(`Benchmarks.tsx:110` sets 0.45 opacity on a 0.13 hairline = 0.058 effective).

Source text: `Source: Anthropic, 9 Nov 2026`.
Loss text: `AutomationBench — Fable 5.2 51.4 · Gemini 4 Carbon 53.9`.
The loss is 19px, bone, same left margin as the labels, no colour, no grey, no
"yet". It lands at local 230, after the dead still. This is the only place in the
film where a number is allowed to be bad news; give it the best typography in the
footer.

Delete the "highest of any model tested" caption from beside the title. It is 13px
in `boneFaint` (`Benchmarks.tsx:68–78`) sitting on the title's baseline and reads as
a stray caption. Replace with a right-aligned note above the plot:

| Element | x | y | w | h | Colour | Font | Size | Weight |
|---|---|---|---|---|---|---|---|---|
| Hero note | 1330 | 252 | 250 | 15 | `rgba(242,239,233,0.55)` | mono | 11.5 | 400 |
| Hero note tick | 1316 | 259 | 10 | 1 | `#C96442` | — | — | — |

Text: `HIGHEST OF ANY MODEL TESTED`, right-aligned, tracking 0.16em.

### 2.5 The entrance (local frames)

| Local | Duration | What moves | From → to |
|---|---|---|---|
| 0 | 12 | Eyebrow tick + label | opacity 0→1, y +10→0 |
| 4 | 36 | Title, per-character | 16 chars, per 1.1, dur 18 |
| 14 | 16 | Measure line | opacity 0→1, y +8→0 |
| 22 | 18 | Axis | length 0→1160, draws left→right |
| 26 | 18 | Five gridlines | opacity 0→1, stagger 3 |
| 36 | 16 | Well | opacity 0→1, scaleY 1.02→1 |
| 42 | 14 | Tracks | opacity 0→1 as one plane |
| 44 / 48 / 52 / 56 / 60 | 14 | Labels, top row down | opacity 0→1, y +8→0 |
| 70 | 32 | Opus bar | width 0→1055.6 |
| 76 | 32 | GPT bar | width 0→993.0 |
| 82 | 32 | Gemini bar | width 0→953.5 |
| 88 | 32 | Fable 5.1 bar | width 0→751.7 |
| start+24 | 8 | Their four values | opacity 0→1, final string |
| **150** | **32** | **Hero bar** | **width 0→1139.1** |
| 174 | 8 | Hero value | opacity 0→1, "49.1%" |
| 182 | 18 | Warm phase + glow | surface lerp, glow 0→0.35 |
| 182 | 36 | Hero top edge | x 1557→1559 (2px) |
| **200** | **30** | **Dead still** | **nothing moves** |
| 230 | 18 | Loss line | opacity 0→1, y +6→0 |
| 248 | 52 | Camera only | scale 1.000→1.006, origin 50% 62% |

The hero is 30 frames late on purpose — one beat at 120 BPM. One empty track at the
top for 44 frames before it starts is the held absence that makes its arrival read
as arrival. Do not pre-fill it: at f0520 the rebuilt frame is **four grey bars and one
empty track**, and that is the correct still.

### 2.6 The 30-frame dead still (local 200–230)

No element moves. No opacity changes. No scale. No breath. The Thread's axis holds at
0.50. The only changing pixels are global grain. The warm has just settled; the hero
edge has just stopped its 2px travel. This is the frame the audience screenshots and
the frame the Verifier measures. It exists so that the loss line at 230 lands in
silence rather than on top of motion.

After it, the loss enters and the camera pushes 1.000→1.006 over 52 frames. Nothing
else moves for the last 52.

### 2.7 Full element table

| Element | x | y | w | h | Colour | Font | Size | Weight |
|---|---|---|---|---|---|---|---|---|
| Eyebrow tick | 150 | 120 | 22 | 1 | `#C96442` | — | — | — |
| Eyebrow | 186 | 112 | 420 | 16 | `#C96442` | mono | 12.5 | 500 |
| Title | 150 | 146 | 640 | 82 | `#F2EFE9` | sans | 82 | 700 |
| Measure | 150 | 244 | 560 | 25 | `rgba(242,239,233,0.62)` | sans | 24 | 400 |
| Hero note | 1330 | 252 | 250 | 15 | `rgba(242,239,233,0.55)` | mono | 11.5 | 400 |
| Hero note tick | 1316 | 259 | 10 | 1 | `#C96442` | — | — | — |
| Well | 384 | 362 | 1232 | 476 | `#2C313B`→`#10141C` | — | — | — |
| Axis | 420 | 800 | 1160 | 1 | `rgba(242,239,233,0.28)` | — | — | — |
| Grid ×5 | 652/884/1116/1348/1580 | 368 | 1 | 432 | `rgba(242,239,233,0.075)` | — | — | — |
| Tick ×6 | 419.5…1579.5 | 800 | 1 | 6 | `rgba(242,239,233,0.28)` | — | — | — |
| Tick label ×6 | 404/636/868/1100/1332/1564 | 818 | 32 | 14 | `rgba(242,239,233,0.52)` | mono | 11 | 400 |
| Track ×5 | 420 | 378/470/562/654/746 | 1160 | 54 | `#12161E` | — | — | — |
| Bar 1 (hero) | 420 | 378 | 1139 | 54 | `#B85736`→`#D2714A` | — | — | — |
| Bar 2 | 420 | 470 | 1056 | 54 | `#6B727E`→`#4A505B` | — | — | — |
| Bar 3 | 420 | 562 | 993 | 54 | `#6B727E`→`#4A505B` | — | — | — |
| Bar 4 | 420 | 654 | 954 | 54 | `#6B727E`→`#4A505B` | — | — | — |
| Bar 5 | 420 | 746 | 752 | 54 | `#6B727E`→`#4A505B` | — | — | — |
| Label 1 | 150 | 394 | 226 | 22 | `#F2EFE9` | sans | 21 | 600 |
| Label 2–5 | 150 | 486/578/670/762 | 226 | 22 | `rgba(242,239,233,0.62)` | sans | 21 | 500 |
| Value 1 | 1583 | 390 | 96 | 30 | `#F2EFE9` | sans | 26 | 600 |
| Value 2 | 1500 | 482 | 96 | 30 | `rgba(242,239,233,0.86)` | sans | 26 | 600 |
| Value 3 | 1437 | 574 | 96 | 30 | `rgba(242,239,233,0.86)` | sans | 26 | 600 |
| Value 4 | 1398 | 666 | 96 | 30 | `rgba(242,239,233,0.86)` | sans | 26 | 600 |
| Value 5 | 1196 | 758 | 96 | 30 | `rgba(242,239,233,0.86)` | sans | 26 | 600 |
| Footer rule | 150 | 852 | 1620 | 1 | `rgba(242,239,233,0.10)` | — | — | — |
| Source | 150 | 866 | 620 | 14 | `rgba(242,239,233,0.55)` | mono | 11.5 | 400 |
| Loss tick | 150 | 904 | 18 | 1 | `rgba(242,239,233,0.30)` | — | — | — |
| Loss line | 150 | 892 | 1200 | 26 | `#F2EFE9` / 0.72 | sans | 19 | 500 |

---

## 3. Value structure

The frames are a black rectangle with bright type because every surface in the film
sits between L\* −2 and L\* 11. There is no plane for light to land on. The theme even
defines one — `bgRaise: #0D111A` (`theme.ts:15`) — and no scene uses it. `grep`
confirms zero references.

### 3.1 The ladder

Three fills. Built as a base plus two overlays so the coder can generate every
intermediate honestly rather than guessing hexes.

| Step | Name | Hex | L\* | Recipe |
|---|---|---|---|---|
| L0 | void | `#05060B` | −1.7 | base fill, no overlay |
| L1 | room | `#141820` | 8.2 | L0 + `rgba(168,196,232,0.095)` |
| L2 | lit surface | `#363B45` | 24.8 | L1 + `rgba(214,222,239,0.177)` |

L2 is the mid-tone. L2 is the only value in the film allowed to be flat over a large
area, and every L2 plane must be masked at its edge — no borders, no radii above 2,
no drop shadows. A plane with a hard edge is a card; a plane with a 40% feather is a
wall.

**Specular (not a fourth fill):** a data object on L2 is coated with light, not raised
to a new fill. The top face of a bar reaches `#6B727E` (L\* 47.8) from L2 +
`rgba(201,212,228,0.359)`. It is the brightest non-type value in the film and still
only 19.3% of bone's luminance.

### 3.2 Where the mid-tone is, per act

Each plane is `background: #363B45` with
`mask-image: radial-gradient(ellipse 100% 100% at 50% 50%, #000 0%, #000 {stop}%, transparent 100%)`.
The percentage is of 2,073,600 pixels.

| Act | Plane x, y, w, h | Mask stop | Mid-tone pixels | % of frame |
|---|---|---|---|---|
| ColdOpen | 300, 300, 1320, 380 | 60% | 301,000 | **14.5%** |
| Positioning | 170, 250, 860, 600 | 55% | 284,000 | **13.7%** |
| Benchmarks | none — the four grey bars | — | 202,700 | **9.8%** |
| Proof | 200, 470, 1100, 360 | 60% | 238,000 | **11.5%** |
| Economics | 430, 300, 1100, 400 | 60% | 264,000 | **12.7%** |
| Lockup | 210, 520, 1500, 320 | 58% | 278,000 | **13.4%** |

All six clear the 8% floor. The Benchmarks plane is the data itself, which is the
correct answer for a chart: the evidence is the light source.

**Key / fill / background per act** (luminance as % of bone):

| Act | Key | Fill | Background | Mid-tone source |
|---|---|---|---|---|
| ColdOpen | overhead, 50% 34%, L2 | ice 3% from lower left | L1 | the far wall at y 300–680 |
| Positioning | left, 38% 48%, L2 | none | L1 | the wall behind the text block |
| Benchmarks | the bars, self-lit | well top edge 1% | L1 | the four data bars |
| Proof | overhead, 42% 57%, L2 | ice 2% at the swarm | L1 | the light table under the data |
| Economics | top, 50% 42%, L2 | none | L1 | the ledger slab |
| Lockup | overhead, 50% 64%, L2 | none | L1 | the floor under the name |

There is no key light in the top-left corner. The light comes from above and slightly
behind the content in every act, so the film reads as one room.

### 3.3 Text on surfaces — contrast floor

Measured against the surface the text actually sits on, not against black.

| Text size | On L1 `#141820` | On L2 `#363B45` |
|---|---|---|
| ≥ 18px | bone ≥ 0.52 | bone ≥ 0.66 |
| < 18px | bone ≥ 0.55 | bone ≥ 0.70 |

`boneFaint` is 0.26 (`theme.ts:20`). It gives 2.0:1 on L1. **It is banned for all
text.** Keep it for hairlines only. Every caption, tick, note, source and footnote in
the film currently uses it — that is why the frame reads as if the small type is
optional. It is not optional; it is the provenance.

### 3.4 The brown, deleted — not reduced

A blurred low-opacity clay field is banned (Brief §2.3). It is banned because blur
plus alpha over near-black integrates to ochre: the red channel survives, the green
half-survives, the blue dies. There is no opacity at which it stops being brown.

Delete, in this order:

1. `ColdOpen.tsx:72–84` — the 1040×600 `clayGlow` ellipse, blur 70, opacity 0.24.
2. `Lockup.tsx:120–132` — the 1240×600 clay→ice ellipse, blur 80, opacity 0.16–0.42.
3. Every clay text-shadow: `Benchmarks.tsx:177`, `Economics.tsx:130` and `:247`,
   `Proof.tsx:180` and `:416`, `Lockup.tsx` none. Text is lit by its own gradient, not
   by a halo.
4. `Backdrop.tsx:27–32` — replace `theme.colors.clay` with `#4A4038` and
   `theme.colors.clay` as `color2` with `#3A3330`. Keep the wash at 0.05–0.06. A
   warm-grey bloom at 10.5% is a light wash; a clay bloom at 10.5% is a stain.
5. `Overlays.tsx:45` — clay soft-light 0.16 → **0.08**; `#0A1A2A` at 0.10 → `#07131F`
   at 0.14. Cool shadows, warm highlight. The grade now separates the accents instead
   of marinating them.

Warmth is then produced only two ways: a flat surface colour interpolation
(§2.2) and the clay object itself. At f0140 the only warm thing in the frame is the
5.2. That is what makes it read as light rather than a smudge.

---

## 4. The Thread, as a device

### 4.1 First appearance and first geometry

It is not a line that crosses the frame. It is a **dimension line**: a measured span
with two end ticks, drawn from a leader, at rest above the object it measures.

- **f96**: a 1px vertical segment begins at (560, 316) and grows down to (560, 412)
  by f112. Opacity 0 → 0.50 over 16 frames.
- **f112–134**: it turns 90° at (560, 412) with a 24px corner radius. One continuous
  arc, no snap.
- **f134–156**: the horizontal leg extends to (1360, 412).
- **f156–176**: the vertical leader fades from 0.50 to 0.15 and shortens to a 16px end
  tick (560, 396→412). The horizontal remains.
- At rest: **x 560 → 1360, y 412, 1px, opacity 0.50**, with a 16px tick up at x=560
  and a 6px tick down at x=1360. Three 4px ticks descend at x 760, 960, 1160,
  opacity 0.35.

The wordmark is 799px wide at 176px (`FABLE` 501 + gap 36 + `5.2` 262), centred at
960, so its ink runs 560.5 → 1359.5. The measure is 800px from 560 to 1360. The mark
is set inside the measure with 0.5px of air each side. That is the whole idea: the
first thing the film does is measure the name it will not explain for thirty seconds.

At f140 the line is above the wordmark, mid-extension, and the wordmark is mid-reveal
below it. Zero pixels of Thread land on a glyph.

### 4.2 The law

1. **Paint order is the occlusion.** `Thread` renders at z=1, between `Backdrop` (z=0)
   and the scene sequences (z=2). A glyph is opaque; it covers the line. The Thread is
   never above a scene layer. If it is, that is a build failure, not a tuning issue.
2. **24px clearance.** No Thread segment may pass within 24px of the bounding box of
   any text. This is enforced by geometry, not by a mask. If a future layout moves
   type, the Thread moves.
3. **No edge contact.** Every segment terminates at least 96px from all four frame
   edges. No exceptions, including the axis: 420 and 1580 are 420 and 340 from the
   edges.
4. **Never a divider.** A Thread segment must have data or a measured object on one
   side and nothing on the other. A segment with content above and below it is a rule
   and is forbidden. This kills `y=300` in Proof and `y=894` as an underline.
5. **No breath.** `Thread.tsx:54` multiplies opacity by `1 + sin(frame/34) × 0.07`.
   A 1px line pulsing 7% is visible as a flicker, and it violates every dead-still
   beat in the film. Opacity is a constant per segment.

### 4.3 Geometry per scene

| Scene | Global | Segment (x1,y1 → x2,y2) | Opacity | End ticks |
|---|---|---|---|---|
| ColdOpen | 96–112 | 560,316 → 560,412 | 0.50 | — |
| ColdOpen | 112–156 | 560,412 → 1360,412 (growing) | 0.50 | up at 560 |
| ColdOpen | 156–210 | 560,412 → 1360,412 | 0.50 | up 16 at 560, down 6 at 1360 |
| Positioning | 210–234 | 218,312 → 218,806 (growing down) | 0.32 | right 8 at both ends |
| Positioning | 234–420 | 218,312 → 218,806 | 0.32 | right 8 at both ends |
| Benchmarks | 442–460 | 420,800 → 1580,800 (drawing) | 0.50 | down 6 at 420 and 1580 |
| Benchmarks | 460–720 | 420,800 → 1580,800 | 0.50 | down 6 at 420 and 1580 |
| Proof / Robo | 740–762 | 1310,664 → 1590,664 (drawing) | 0.45 | down 6 at 1310 |
| Proof / Robo | 762–870 | 1310,664 → 1590,664 | 0.45 | down 6 at 1310 |
| Proof / Fermat | 890–912 | 260,812 → 1180,812 (drawing) | 0.40 | down 6 at 260 |
| Proof / Fermat | 912–1050 | 260,812 → 1180,812 | 0.40 | down 6 at 260 |
| Proof / Rust | 1070–1092 | 260,724 → 1200,724 (drawing) | 0.40 | down 6 at 260 |
| Proof / Rust | 1092–1200 | 260,724 → 1200,724 | 0.40 | down 6 at 260 |
| Economics | 1252–1300 | 1100,604 → 1520,604 (drawing) | 0.62 | down 8 at both ends |
| Economics | 1300–1500 | 1100,604 → 1520,604 | 0.62 | down 8 at both ends |
| Lockup | 1540–1568 | 519,800 → 1077,800 (drawing) | 0.55 | up 7 at both ends |
| Lockup | 1568–1760 | 519,800 → 1077,800 | 0.55 | up 7 at both ends |
| Lockup | 1760–1780 | ends converge on 1156,720 | 0.55 → 0 | — |

The Benchmarks axis is the Thread: one object, not a line plus an axis. Its ticks are
the tick marks at 800–806. It is legal under the law because data is drawn against it
and nothing sits below it but the footer rail.

The Proof segments hard-cut with their vignettes. That is allowed: the cut is a cut,
and a hard cut is not a teleport inside a shot. Do not tween between them.

**Which layer?** In `Film.tsx`, order must be: `Backdrop` → `Thread` → scenes →
`Sweep` → `Grade` → `FadeIn` → `Grain` → `Vignette` → `GateWeave`. The Thread's
`mixBlendMode: screen` stays; it now screens against the room, not the type.

**Opacity cap:** 0.50 in any frame containing bone type. `ice #BFE9F5` at 0.50 over
`#141820` composites to `#6E8690`, 22% of bone's luminance. The Thread is never the
frame's peak. At 0.90 it was.

### 4.4 The ending

At f1760 the Lockup rule's two ends leave their ticks and converge on **(1156, 720)**,
the counter of the 5, reaching it at f1780 and dying there. Duration 20 frames,
`theme.ease.in` — it accelerates into the glyph. From f1780 to f1800 there is no line
on screen.

The instrument that measured the film ends up inside the name. Nothing else in the
film can end this way, which is why the device is worth keeping.

---

## 5. The lockup

Global 1500–1800, local 0–300. Hard cut in at 1500, no exit.

### 5.1 Layout

| Element | x | y | w | h | Colour | Font | Size | Weight |
|---|---|---|---|---|---|---|---|---|
| Availability pill | 796 | 196 | 328 | 44 | fill `rgba(242,239,233,0.03)`, border `rgba(242,239,233,0.13)`, radius 999 | — | — | — |
| Pill dot | 818 | 215 | 6 | 6 | `#BFE9F5` | — | — | — |
| "AVAILABLE NOW" | 836 | 211 | 160 | 14 | `#F2EFE9` | mono | 12 | 500 |
| Pill divider | 1006 | 210 | 1 | 16 | `rgba(242,239,233,0.13)` | — | — | — |
| Date | 1022 | 211 | 100 | 14 | `rgba(242,239,233,0.72)` | mono | 12 | 400 |
| Left rail | 672 | 218 | 108 | 1 | `transparent`→`rgba(242,239,233,0.13)` | — | — | — |
| Right rail | 1140 | 218 | 108 | 1 | `rgba(242,239,233,0.13)`→`transparent` | — | — | — |
| Eyebrow CLAUDE | 890 | 560 | 140 | 22 | `rgba(242,239,233,0.62)` | mono | 16 | 500 |
| Wordmark FABLE | 519 | 606 | 558 | 172 | `#FBF8F2`→`#E6E2DA` | sans | 196 | 800 |
| Wordmark 5.2 | 1103 | 606 | 292 | 172 | `#E08258`→`#C96442`→`#A94E31` | sans | 196 | 800 |
| Thread rule | 519 | 800 | 558 | 1 | `rgba(242,239,233,0.55)` | — | — | — |
| Rule tick L | 518.5 | 793 | 1 | 7 | `rgba(242,239,233,0.55)` | — | — | — |
| Rule tick R | 1076.5 | 793 | 1 | 7 | `rgba(242,239,233,0.55)` | — | — | — |
| API pill | 862 | 836 | 196 | 36 | fill `rgba(191,233,245,0.06)`, border `rgba(191,233,245,0.16)`, radius 3 | — | — | — |
| API id | 876 | 847 | 168 | 15 | `#BFE9F5` | mono | 15 | 400 |
| Platform row | 624 | 916 | 672 | 14 | `rgba(242,239,233,0.66)` | mono | 11.5 | 400 |
| Footnote | 260 | 952 | 1400 | 17 | `rgba(242,239,233,0.52)` | mono | 11 | 400 |

Wordmark geometry: gap 26, letter-spacing −0.052em on FABLE and −0.062em on 5.2,
line-height 0.88, `top: 606`. `FABLE` ink 519→1077; `5.2` ink 1103→1395. Full mark
882px, optical centre 957, 3px left of frame centre because the clay numeral is
optically heavier. Nudge the whole flex group `left: 963` so the bone mass centres.

The 196px size is deliberate: the last appearance of the name is the largest type in
the film. At 158px the film ended on a diminuendo.

**No glow.** The 1240×600 clay ellipse is deleted. The name sits on the Lockup plane
(§3.2) and is lit by its own vertical gradient: bone from `#FBF8F2` to `#E6E2DA`,
clay from `#E08258` through `#C96442` to `#A94E31`. Type as a lit surface, not type
with a halo. Both gradients are clipped to the glyphs. No text-shadow.

Only one clay object: the 5.2. The availability dot is **ice**, not clay (it is
currently clay at `Lockup.tsx:174`, which makes two hero accents in the final frame).

### 5.2 Timeline (local)

| Local | Duration | What |
|---|---|---|
| 0 | 10 | Pill + both rails in, 10px rise |
| 20 | 22 | Eyebrow, per-character, 6 chars × 3 |
| 36 | 37 | FABLE, per-character, 5 chars × 3.2, dur 24 |
| 64 | 24 | 5.2 as one masked unit |
| 80 | 28 | Rule draws 519 → 1077 |
| 96 | 62 | Specular sweep across both words (existing, keep) |
| 118 | 24 | API pill in, 12px rise |
| 132 | 36 | Platform items, stagger 5 |
| 186 | 26 | Footnote in, 6px rise |
| 260 | 20 | Thread retracts into the 5 |
| 280 | 20 | Hold, no line |

The footnote at 186 is late on purpose: it is the last thing to arrive, and it is
readable. 114 frames of hold let a viewer actually read it.

### 5.3 The authored detail

The rule is **558px long** — exactly the ink width of `FABLE` — while `5.2` overhangs
its right end by **318px**. The rule measures the family name; the version escapes the
measurement. At f1760 the rule's ends converge on the counter of the 5 and it dies
there. So the last gesture in the film is the measuring instrument retracting into the
thing it could not measure.

That is the detail that makes the frame authored rather than assembled. It is not a
decoration; it is the film's argument in one move.

### 5.4 The footnote

11px, 0.52 alpha, line-height 1.55, max-width 1400, centred, at y 952. One line:
*Figures as reported by Anthropic, 9 Nov 2026. Benchmark scores reflect each model's
highest available effort.*

The attribution stays word-for-word (`benchmarks.ts:175–176`). It is now 5.3:1 and
inside the 96px safe margin (bottom edge 969). A footnote nobody can read is a
disclaimer, not a credit.

---

## 6. The three moments

The Brief's three moments are kept. Two of the three move because the cut moved.

### 6.1 Moment 1 — "The Turn" — ColdOpen 112–180

At f112 the frame contains the date rail (y 252) and nothing else but the vertical
leader. The wordmark has not entered. The leader turns at (560,412) between 112 and
134, and the horizontal leg runs to 1360 by 156. The three 4px ticks arrive at 146,
150 and 154 as the leg passes them. At 118 — six frames after the turn begins — FABLE
starts to rise through its mask; at 130 the version enters as one unit.

By f180 the film is measuring a name it has not finished saying. No logo has been
"presented"; the mark has been *set inside a measurement*.

Hard requirements: zero pixels of Thread on a glyph at any frame in this window; the
version never appears as a bare "5"; the leader never touches x=560 below y=412 after
f156.

### 6.2 Moment 2 — "The Room Warms" — Benchmarks 570–650

- **570**: the hero bar starts. All four competitors have been at rest since 540. The
  top track has been empty for 30 frames.
- **602**: the leading edge stops at x 1559.1. No scale pop, no glow burst, no number
  flying.
- **602–620**: 18 frames. The well's fill warms (`#2C313B→#342C27` top,
  `#10141C→#17120F` bottom). The four grey bars drop exactly 20% in luminance
  (`#6B727E→#606772`, `#4A505B→#424852`). The hero does not change. No blur, no
  overlay, no colour wash: the room's surfaces change temperature.
- **602–638**: the hero's 1px top specular travels from x 1557 to 1559. Two pixels.
  It is the only thing moving.
- **620–650**: dead still, 30 frames.

At f620 the frame is: a bone title, four grey bars at 80%, one clay bar at 100%, and a
value. A viewer cannot say what changed. They know the room is warmer.

Hard requirements: the glow reaches 0.35 at 620 and never exceeds it; no other element
moves between 602 and 650; the warm is a fill interpolation on the well and the bars,
never a blurred ellipse.

### 6.3 Moment 3 — "The Hold" — Proof/Fermat 908–968, payoff 968–1028

Fermat runs 870–1050. The hold is local 38–98, global **908–968**.

- **870–878**: only the kicker `PROOF 02 — MATHEMATICS` enters at x 260, y 296.
  No title. No stats. No clock.
- **874–908**: 1,000 dots assemble on a 50×20 grid, 13px cells, 4px dots, at x 260,
  y 536. The plate behind them is the Proof plane (§3.2), 1100×360 at 200,470.
- **908–968**: 60 frames. Nothing enters. The swarm breathes at ±0.6% opacity,
  0.55 Hz. One warm wave crosses left→right at local 58–80 (global 928–950): each dot
  lerps its colour from bone to clay as the wave passes, peak 0.55 mix, and returns.
  It is a colour change per dot, not a blurred band. No new element, no new colour on
  screen, no type.
- **968**: the clock resolves at x 1130, y 520: `30h`, 96px, 700, clay, via a 6-frame
  focus resolve (blur 8→0, y +8→0, opacity 0→1). It does not count.
- **968–986**: the line resolves by word at x 1130, y 340, 46px/600 bone:
  *A century-old theorem, closed overnight.* Six words, per 3, 18 frames.
- **974**: `wall clock` label. **982**: `1,000`. **988**: `agents in parallel`.
  **994**: `7×`. **1000**: `shorter proof`. **1010**: `⅓ the action tokens.`
- **1010–1050**: hold.

The title is moved from the top of the vignette to the payoff. In the current frame it
is present from local 6, so the hold has nothing to reveal and the claim is spent
before the silence. The claim must arrive *after* the silence or the moment is a
caption.

Hard requirements: no element enters between 908 and 968; the wave is a per-dot colour
lerp; the clock is a constant, never a counter; no cut inside the window.

---

## 7. House laws for the coder

1. **Numbers resolve, never count.** No `value * p`, no `toFixed` on a live variable,
   no `Counter` (`motion.tsx:153`). A number is either absent or exactly true. This
   applies to bar values, prices, multiples, 30h, 1,000 and 7×.
2. **No spring drives a data dimension.** Widths, positions of values, and any axis
   tick use `progress(...)` with `theme.ease.out`. Springs are for type blocks and
   nothing that encodes a quantity. `theme.spring.punch` (`theme.ts:62`) is deleted.
3. **Per-character reveals are banned above 12 characters.** A 36-character sentence
   is a single-line mask: the whole line rises 105%→0 behind one clip, 20 frames, one
   transform. This is what makes f0900 legible.
4. **One clay object per frame.** The 5.2, or the hero bar, or the −20%, or the pill
   dot — never two. The dot becomes ice.
5. **No clay text-shadow, anywhere.** Type is lit by a gradient clipped to the glyphs.
6. **No blurred clay field, anywhere.** Warmth is a flat fill interpolation.
7. **Exits are deleted.** M6. Hard cuts at 420, 720, 1200, 1500. No `useSceneExit`, no
   `headOut`, no 46px rise, no 7px blur on the way out.
8. **`boneFaint` is not a text colour.** Hairlines only.
9. **Serif italic is out.** The Brief §5 is right: it is the most borrowed move in the
   category. Economics becomes `Same work.` at 66/500 `boneDim` + `Smaller bill.` at
   104/700 bone, same sans, no italic — the claim gets bigger as the price gets
   smaller. Positioning becomes one line at 96/600, `ambitious` in clay. The serif
   font stays loaded but unused.
10. **The Thread is one object.** Do not add a second axis, a second rule or a
    "helper" hairline in a scene. The Backdrop's horizon at y=905
    (`Backdrop.tsx:107–117`) is a third full-width line and must be deleted. It is
    visible in f1300 as a table rule with nothing on it.

---

## 8. Acceptance checks

The Verifier can run these without interpretation.

| Check | Frame | Pass condition |
|---|---|---|
| Thread on glyph | 140 | 0 pixels of Thread inside any text bbox |
| Thread edge contact | all | ≥96px from every frame edge |
| Value truth | 520, 570–720 | every visible value equals its `benchmarks.ts` value |
| No overshoot | 420–720 | bar width and value opacity monotone; no width > value × 23.2 |
| Mid-tone floor | 140, 520, 900, 1300, 1700 | ≥8% of pixels in L\* 25–65 |
| Chart bar fill | 520 | inactive fill L\* ≥ 25; no 1px border on 3 sides |
| Glow cap | 620 | hero shadow radius 24px, alpha ≤ 0.35, one shadow |
| Footnote | 1700 | ≥11px, contrast ≥4.5:1, bottom edge ≤984 |
| Dead still | 200–230 | frame-to-frame Δ < 2% of pixels |
| Hold | 908–968 | zero elements enter |
| Lockup line | 1780–1800 | zero pixels of Thread |
| Cut integrity | 420/720/1200/1500 | `TOTAL_FRAMES === 1800` |

---

## 9. What I would spend the remaining budget on

The Fermat hold, 908–968, and the 60-frame dawn wave inside it. It is the only beat in
the film a stranger will retell at dinner, and at 480 frames of Proof it is the one
place where a single frame of extra craft is worth more than a week of polish on the
chart. Render that window at 2× supersampling so 1,000 4px dots do not alias, and give
the wave a per-dot colour curve rather than a moving gradient — the difference between
a screensaver and a night passing.
