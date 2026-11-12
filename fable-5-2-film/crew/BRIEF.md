# BRIEF — Claude Fable 5.2 / 30s

## 1. The one sentence

**This is a thirty-second film about the one thing a model cannot fake — staying with the work — made as a single unbroken take that refuses to cut away, and it does not say the name until the last five seconds.**

The argument underneath it: every other model is fast; this one is patient. Speed is a specification. Patience is a character. If a stranger cannot repeat that sentence after one viewing, we have made a demo reel, not a film.

## 2. What "premium" actually means here

Seven properties. Each one is pass/fail on a single frame. A frame that fails is not finished, and beauty is not a defence.

1. **One peak per frame.** Exactly one object sits at ≥90% of the frame's peak luminance; everything else is ≤60% of peak. Two competing highlights is a fail. At f0140 the wordmark and the Thread are both at peak — and they physically overlap, which is worse.
2. **Mid-tone floor.** Every frame holds ≥8% of its pixels between L\* 25 and L\* 65. A near-black field with bright type on it is a slide with the lights off. Measure f0140: my eye puts it under 3%. This is D3 and it is the single largest reason the film reads cheap.
3. **One hue, fully committed.** A frame carries bone and graphite, plus at most one accent. Never warm and cool at equal weight. And clay is either the most saturated thing in the frame or it is absent — **a blurred, low-opacity clay field is banned**, because blur plus low alpha equals brown. That is D2 in one line: the left half of f0140 looks like a thumbprint on the lens, not light.
4. **No cuts.** Every act boundary is a continuation of the same object in the same position: ≥40% of pixels carry across the join, in register. One camera, one room, one move. If you can see the join, the join is the failure. This is the entire difference between a film and a deck.
5. **Nothing flashes.** No element lives on screen for fewer than 45 frames (0.75s). Three stat cards at 73 frames each is a flip-book; at speed the audience reads exactly one and remembers none.
6. **One held breath.** A window of at least 40 frames in which no new element enters, no type changes, and frame-to-frame change stays under 2% of pixels. The score drops to room tone for its duration. The current cut has zero rests — every frame is an entrance, which is why it feels anxious rather than confident.
7. **Every number is true at every frame.** Numbers never count up, tween or tick through values that are not real. A value resolves by focus, wipe or mask: it is either absent or correct. A screenshot of any frame must survive a fact-check. f0700 currently states "Humanity's Last Exam 59.3%". That number does not exist and it is one screenshot away from a very bad week.

## 3. The three moments that must be unforgettable

Frames are in the new cut defined in 4c.

**Moment 1 — ColdOpen, f0112–f0180 — "The Turn."**
The frame is empty. One hairline of light hangs vertically, unanchored, touching neither edge. At f0112 it turns — ninety degrees, one continuous arc, no snap — and becomes a horizontal axis. Tick marks arrive beneath it as it travels. By f0180 the film is already measuring something and the audience has not been told what the product is called.
*Why it lands:* it states the film's grammar in three seconds, with no copy and no logo. Every later scene is a variation of this one gesture — a line that becomes an instrument. It also buys the ending: a film that opens by measuring earns the right to end on a name.
*Hard requirement:* no wordmark, no "CLAUDE", no version numeral anywhere before f1440. A second logo is not a brand asset, it is a watermark.

**Moment 2 — Benchmarks, f0570–f0600 — "The Room Warms."**
Four competitor bars have landed, cool and grey and at rest. At f0570 Fable's bar arrives last — half a beat late, the only lateness in the film — and it arrives long. The instant its leading edge stops, the light in the room changes: ambient slides warm over eighteen frames, the three grey bars drop twenty percent in value, and the only thing still moving is a warm edge settling two pixels.
*Why it lands:* it is the film's only special effect and it is not a graphic — it is a change of temperature. Nobody in the audience will be able to say what happened. They will only know the frame felt different afterwards. That is what money looks like on screen.
*Hard requirement:* no glow burst, no scale pop, no confetti, no numbers flying. The bar arrives and the room responds. Nothing else moves.

**Moment 3 — Proof / Fermat, f1092–f1152 — "The Hold."**
One thousand points of light are working. Then the film stops. For sixty frames nothing enters: the swarm breathes, one warm wave crosses it, the score falls to room tone. At f1152 — only after the silence — the clock reads 30h and a single line arrives: *A century-old theorem, closed overnight.*
*Why it lands:* it is the only silence in thirty seconds, and it is spent on the largest idea in the dataset. Three hundred and fifty-eight years of human failure, closed in a night by a thousand agents who never sleep. The audience supplies the warmth: everyone has left something running overnight. Do not put a person on screen. The night is the person.
*Hard requirement:* no stat cards inside the hold, no cut, no new colour. If anything enters before f1152, the moment is dead and so is the film's best thirty seconds.

## 4. Rulings on the open questions

**a. Disclose the two losses — yes, on screen, in the same layout as the wins. And drop Chartography to pay for it.**
The losses get the same type size, the same left margin, the same duration and no colour. Not fine print, not grey, not the word "yet". Card runs f0795–f0900, the longest card in the act, deliberately: `AutomationBench 51.4 — Gemini 4 Carbon 53.9` / `AA-Briefcase Elo 1,785 — Opus 5.5 1,800`. Chartography comes out because 73.4 against 72.5 is a nine-tenths-of-a-point win: the one number that makes the other three look cherry-picked, and worth less than the credibility of saying aloud where you lose. I would do this even if the client objects. Only a company that is genuinely ahead can afford to say where it isn't; the omission is the tell, not the loss.

**b. Fix the Thread. Not kill, not demote — but the fix is a law, not a nudge.** Three rules, all three or the device goes:
1. **It is occluded.** The Thread lives inside the room, not on top of it. It currently renders as layer 4, above every scene (Film.tsx:60), so it paints over the wordmark. Light does not paint over the object in front of it. Any pixel of Thread landing on a glyph is a fail.
2. **It never touches the frame edge.** A line running edge to edge at y=96→984 (Thread.tsx:15) reads as a seam, a gutter or a crop mark. It terminates in air at both ends, or it terminates in an object.
3. **It is never a divider.** Three legal uses only: an axis data is drawn against; an edge something emerges from; a path motion travels along. It may not close a shape, may not cross type, and may not sit at x=960 while a centred composition is centred on it. That is the f0140 failure in one sentence.
And it gets a different ending. Not an underline — underline is the least interesting thing a line can do and the most common. From f1760 the Thread retracts into the wordmark, enters the counter of the "5", and is gone by f1780. The instrument that measured the film ends up inside the name. At f1799 there is no line on screen.

**c. Thirty seconds is right. The film is carrying one scene too many: cut Economics (f1200–1500) and replace it with nothing.** Price is a procurement fact, not a film beat, and "−20%" is the weakest emotional beat in the film sitting in the position where the strongest should be. Meanwhile Fermat — the only story here a stranger will retell at dinner — gets 120 frames, fewer than the pricing slide. Reinvest the 300: Proof to 540 (three vignettes at 180 each), Lockup to 360. All boundaries stay on the 120 BPM grid, 30 frames to the beat.

| Act | Frames | Beats |
|---|---|---|
| ColdOpen | 000–180 | 6 |
| Positioning | 180–420 | 8 |
| Benchmarks | 420–900 | 16 |
| Proof | 900–1440 | 18 |
| Lockup | 1440–1800 | 12 |

Also in Positioning: delete the four pills. "Long-horizon / Agentic / 1M token context / Adaptive thinking" is a feature list with rounded corners, and a film does not have a feature list. The line already underneath it — *best for open-ended work that runs for hours, not seconds* — says all four of them better.

**d. One warm moment, yes. It is a night passing, not a person — Proof / Fermat, f1092–f1152.** Fermat is the only scene in the film with a time of day: the room's colour temperature climbs through the vignette, cool at the first point of light, warm by the last, one full dawn in three seconds. That is the whole gesture. No hands, no faces, no stock footage, no engineer smiling at a monitor. Warmth in this film is a thermometer, not an expression. If anyone proposes a human cutaway the answer is no, and the reason is that the audience is the human.

## 5. Directives

**DESIGNER.** You are not styling a deck; stop laying out slides. Come back with two things and nothing else. First, a lighting plan: for each of the five acts, key, fill and background expressed as luminance percentages, and a named source for the 8% mid-tone band. It cannot be the type — it has to be a surface, a haze, a floor, a structured gradient. Every frame right now is content floating in void; give the film a floor and give the void a far wall. Second, six frames redrawn as stills — f0140, f0300, f0570, f0700, f1100, f1700 — each with one maximum-contrast object and a legible subordinate plane behind it. Separately: the italic serif in clay ("technical *work.*") is the most borrowed move in the category and it is why f0300 could belong to any company on earth. Replace it with a typographic idea that belongs to this film, or cut the line.

**RESEARCHER.** The film's argument is now "it stays with the work" and the data does not yet carry it as hard as it must. Come back with five facts, each with URL and the exact quoted sentence from the source: the previous best time to formalise Fermat's Last Theorem in Lean; what "30h wall clock" actually measures and whether it is wall clock or aggregate agent time; the prior baseline for Python→Rust migration startup gains; what AutomationBench measures and why 2.5 points there is unremarkable; and what the 27.6 robotics baseline actually was. Then resolve F1: Anthropic's exact wording for the robotics multiple, and whether the published figure is 2.2× or 2.4×. Do not editorialise, do not round, do not interpret. If a fact cannot be sourced, write "unsourced" and the film loses it.

**VERIFIER.** You are the only person who can stop this film and I expect you to use it. Turn the seven properties in §2 into frame-level checks with numbers, run them against the eight existing stills, and publish a pass/fail table with the actual readings: peak-luminance region count, mid-tone percentage, hue spread, element lifetime in frames, pixel continuity across every act boundary. Then run the same suite on the three moments. Specifically and non-negotiably: at f0140, count the pixels of Thread that land on a glyph; at f0700, read the displayed value and check it against `benchmarks.ts`; at each boundary, report the continuity percentage. Fail anything under threshold even if it is beautiful. A beautiful frame that fails is a frame we cut.

**FACT-CHECKER.** Re-audit from zero against the recut, and add the two losses to `benchmarks.ts` with their sources — you supply the numbers and the exact bar labels, the Manager types them. Rules I am setting now, effective immediately. One: the attribution "each model's highest available effort" (benchmarks.ts:176) is false for as long as Gemini 4 Carbon at 41.1 Medium is plotted; either that bar is re-plotted at its highest published effort or it comes out — I would take it out, it is fourth place and it buys nothing but a lie. Two: every number on screen must trace to a line in `benchmarks.ts` and a sentence in a source. Three: the disclosure card gets the same standard as the wins — if 1,800 is an Opus 5.5 figure, the card says Opus 5.5, not Opus. Four: a screenshot of any single frame must be defensible to a journalist. Come back with a line-by-line audit table: frame, on-screen text, source, verdict.

**MANAGER.** Rebuild the cut to 4c before you touch a single pixel of polish: re-time to the table, move the chart's hero punch to f0570, extend Proof to 180-frame vignettes, delete the pill row, delete the wordmark from the cold open. Then three structural changes. The Thread moves beneath all scene content and gains masks — occluded, edge-free, and retracting into the "5" at f1760–1780. Every counting number is replaced by a resolve — focus, wipe, mask — so that no frame ever displays a value that isn't real; f0700 is currently a false claim and it is the first thing a critic will screenshot. And the version numeral never appears as a bare integer: "5.2" enters and leaves as one unit, at one size, in one frame, or it does not appear at all — at f0140 the film currently reads "FABLE 5", which is a different product. Come back with the new ACTS table, a rebuilt f0140 with no wordmark and a mid-tone that isn't type, and one rebuilt card that nobody could mistake for a slide. Do not colour-grade a slide; grade is not a substitute for light.

## 6. The bar

I reject this film if a stranger can pause it anywhere, glance at the frame, and know exactly what the next five seconds will look like — because that means no frame is doing work the others aren't. I reject it if it plays at 480 pixels wide with the sound off and reads as five slides. I reject it if "premium" can be swapped for "expensive" without changing a single decision. I reject it at f0140 if one pixel of that line still lands on a letter, because that pixel is the difference between a studio and a template and I have already said so once. And I reject it if nobody on this crew can name the best frame in the film without consulting the shot list — because then we have delivered an asset, not a film.
