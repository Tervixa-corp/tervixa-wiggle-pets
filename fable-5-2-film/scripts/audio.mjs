/**
 * Synthesises the film's 30-second score as a stereo 16-bit WAV.
 *
 * No downloads, no sample library, no licensing questions — every sound here is
 * generated from a handful of oscillators and noise, then shaped to the same
 * 120 BPM grid the picture is cut on (1 beat = 30 frames = 0.5s).
 *
 *   node scripts/audio.mjs
 */
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SR = 48000;
const DUR = 30;
const N = SR * DUR;

const L = new Float32Array(N);
const R = new Float32Array(N);

/* ── helpers ─────────────────────────────────────────────────────────────── */

let rngState = 0x2f6e2b1;
const noise = () => {
  rngState = (rngState * 1664525 + 1013904223) >>> 0;
  return (rngState / 0xffffffff) * 2 - 1;
};

const add = (i, v, gl, gr) => {
  if (i < 0 || i >= N) return;
  L[i] += v * gl;
  R[i] += v * gr;
};

/** Equal-power pan, pan in -1..1. */
const pan = (p) => {
  const a = ((p + 1) * 0.25) * Math.PI;
  return [Math.cos(a), Math.sin(a)];
};

const midiHz = (m) => 440 * Math.pow(2, (m - 69) / 12);

/* ── 1. drone — the room tone ────────────────────────────────────────────── */

const DRONE = [
  { f: 55, a: 0.9, d: 0.0 },
  { f: 82.5, a: 0.46, d: 0.9 },
  { f: 110, a: 0.3, d: 1.8 },
  { f: 164.5, a: 0.14, d: 2.6 },
];

for (let i = 0; i < N; i += 1) {
  const t = i / SR;
  let v = 0;
  for (const o of DRONE) {
    const drift = 1 + Math.sin(t * 0.061 + o.d) * 0.0011 + Math.sin(t * 0.017 + o.d) * 0.0007;
    v += Math.sin(2 * Math.PI * o.f * drift * t + o.d) * o.a;
  }
  const swell = 0.5 + 0.5 * Math.sin(t * 0.19 - 1.1);
  const fadeIn = Math.min(1, t / 4.5);
  const fadeOut = t > 28.2 ? Math.max(0, (30 - t) / 1.8) : 1;
  const g = 0.052 * (0.32 + 0.68 * swell) * fadeIn * fadeOut;
  add(i, v * g, 0.72, 0.69);
}

/* ── 2. air bed — filtered noise, breathing ──────────────────────────────── */

{
  let lpL = 0;
  let lpR = 0;
  const a = 1 - Math.exp((-2 * Math.PI * 460) / SR);
  for (let i = 0; i < N; i += 1) {
    const t = i / SR;
    lpL += a * (noise() - lpL);
    lpR += a * (noise() - lpR);
    const env = 0.5 + 0.5 * Math.sin(t * 0.31 + 0.4);
    const fadeIn = Math.min(1, t / 5.5);
    const fadeOut = t > 28 ? Math.max(0, (30 - t) / 2) : 1;
    const g = 0.042 * (0.28 + 0.72 * env) * fadeIn * fadeOut;
    L[i] += lpL * g;
    R[i] += lpR * g;
  }
}

/* ── 3. sub impacts on every act boundary ────────────────────────────────── */

const impact = (t0, gain, f0) => {
  const start = Math.round(t0 * SR);
  const len = Math.round(2.2 * SR);
  let ph = 0;
  for (let n = 0; n < len; n += 1) {
    const t = n / SR;
    const f = f0 * (1 + 1.2 * Math.exp(-t * 10)) * (1 - 0.3 * Math.min(1, t / 1.4));
    ph += (2 * Math.PI * f) / SR;
    const env = Math.exp(-t * 2.9) * (1 - Math.exp(-t * 260));
    add(start + n, Math.sin(ph) * env * gain, 0.7, 0.7);
  }

  // Bright transient so the hit reads on laptop speakers too.
  const tlen = Math.round(0.1 * SR);
  let hpPrev = 0;
  let hpY = 0;
  const ah = 1 - Math.exp((-2 * Math.PI * 2600) / SR);
  for (let n = 0; n < tlen; n += 1) {
    const t = n / SR;
    const x = noise();
    hpY = ah * (hpY + x - hpPrev);
    hpPrev = x;
    const env = Math.exp(-t * 58);
    add(start + n, hpY * env * gain * 0.42, 0.6, 0.8);
  }
};

impact(0.0, 0.3, 62);
impact(3.5, 0.42, 58);
impact(7.0, 0.62, 52);
impact(12.0, 0.64, 50);
impact(20.0, 0.6, 52);
impact(25.0, 0.72, 44);

/* ── 4. risers — the eight frames before each cut ────────────────────────── */

const riser = (t0, t1, gain) => {
  const start = Math.round(t0 * SR);
  const end = Math.round(t1 * SR);
  let lp = 0;
  let lpPrev = 0;
  let hpY = 0;
  for (let i = start; i < end; i += 1) {
    const u = (i - start) / (end - start);
    const fc = 320 * Math.pow(13, u);
    const a = 1 - Math.exp((-2 * Math.PI * fc) / SR);
    const x = noise();
    lp += a * (x - lp);
    const ah = 1 - Math.exp((-2 * Math.PI * (fc / 5)) / SR);
    hpY = ah * (hpY + lp - lpPrev);
    lpPrev = lp;
    const env = Math.pow(u, 2.8);
    const [gl, gr] = pan(-0.55 + u * 1.1);
    add(i, hpY * env * gain, gl, gr);
  }
};

riser(2.15, 3.5, 0.2);
riser(5.55, 7.0, 0.24);
riser(10.3, 12.0, 0.28);
riser(18.1, 20.0, 0.28);
riser(23.2, 25.0, 0.32);

/* ── 5. plucks — the machine ticking over ────────────────────────────────── */

const pluck = (t0, midi, gain, p) => {
  const [gl, gr] = pan(p);
  const f = midiHz(midi);
  const start = Math.round(t0 * SR);
  const len = Math.round(0.55 * SR);
  for (let n = 0; n < len; n += 1) {
    const t = n / SR;
    const env = Math.exp(-t * 7.2) * (1 - Math.exp(-t * 900));
    const v =
      Math.sin(2 * Math.PI * f * t) * 0.74 +
      Math.sin(4 * Math.PI * f * t) * 0.17 +
      Math.sin(6 * Math.PI * f * t) * 0.06;
    add(start + n, v * env * gain, gl, gr);
  }
};

/** Intensity per act, used to gate the pluck grid. */
const intensity = (t) => {
  if (t < 3.5) return 0.16;
  if (t < 7) return 0.32;
  if (t < 12) return 0.6;
  if (t < 20) return 0.52;
  if (t < 25) return 0.62;
  if (t < 28.4) return 0.22;
  return 0;
};

// A minor pentatonic, walking upward as the film gains confidence.
const SCALE = [45, 48, 50, 52, 55, 57, 60, 62, 64, 67, 69, 72, 74, 76, 79, 81];

{
  let step = 0;
  for (let t = 1.0; t < 28.4; t += 0.25) {
    const gate = noise() * 0.5 + 0.5;
    const local = intensity(t);
    if (gate > local + 0.34) {
      step += 1;
      continue;
    }
    const idx = (step * 3 + Math.floor(t * 2)) % SCALE.length;
    const note = SCALE[idx];
    const p = Math.sin(step * 1.7) * 0.75;
    pluck(t, note, 0.032 + local * 0.026, p);
    step += 1;
  }
}

/* ── 6. the resolution — a warm sustained chord under the lockup ─────────── */

{
  const t0 = 24.86;
  const t1 = 30.0;
  const start = Math.round(t0 * SR);
  const end = Math.round(t1 * SR);
  const NOTES = [45, 57, 61, 64, 69, 76]; // A2 A3 C#4 E4 A4 E5
  NOTES.forEach((m, k) => {
    const f = midiHz(m);
    let ph = k * 0.37;
    const sway = 0.28;
    for (let i = start; i < end; i += 1) {
      const t = (i - start) / SR;
      const attack = 1 - Math.exp(-t * 2.4);
      const fade = t > 3.2 ? Math.max(0, 1 - (t - 3.2) / 1.9) : 1;
      const env = attack * fade;
      ph += (2 * Math.PI * f * (1 + Math.sin(t * 0.9 + k) * 0.0008)) / SR;
      const v = (Math.sin(ph) + Math.sin(ph * 2) * 0.1 + Math.sin(ph * 3) * 0.035) * env * sway * 0.055;
      add(i, v, 0.7, 0.7);
    }
  });

  // High shimmer, barely there.
  [1760, 2637].forEach((f, k) => {
    let ph = k * 1.1;
    for (let i = start; i < end; i += 1) {
      const t = (i - start) / SR;
      const env = (1 - Math.exp(-t * 1.6)) * (t > 3.4 ? Math.max(0, 1 - (t - 3.4) / 1.7) : 1);
      ph += (2 * Math.PI * f * (1 + Math.sin(t * 5.1 + k) * 0.004)) / SR;
      add(i, Math.sin(ph) * env * 0.0055, 0.45, 0.55);
    }
  });
}

/* ── 7. master — gentle bus compression, then normalise ──────────────────── */

let peak = 0;
for (let i = 0; i < N; i += 1) {
  // Soft knee: everything above 0.55 gets curved down instead of clipped.
  L[i] = Math.tanh(L[i] * 1.7) / 1.7;
  R[i] = Math.tanh(R[i] * 1.7) / 1.7;
  peak = Math.max(peak, Math.abs(L[i]), Math.abs(R[i]));
}

const target = 0.9;
const norm = peak > 0 ? target / peak : 1;
for (let i = 0; i < N; i += 1) {
  L[i] *= norm;
  R[i] *= norm;
}

/* ── 8. write 16-bit PCM WAV ─────────────────────────────────────────────── */

const bytesPerSample = 2;
const dataSize = N * 2 * bytesPerSample;
const buffer = Buffer.alloc(44 + dataSize);

buffer.write("RIFF", 0, "ascii");
buffer.writeUInt32LE(36 + dataSize, 4);
buffer.write("WAVE", 8, "ascii");
buffer.write("fmt ", 12, "ascii");
buffer.writeUInt32LE(16, 16); // PCM chunk size
buffer.writeUInt16LE(1, 20); // format = PCM
buffer.writeUInt16LE(2, 22); // channels
buffer.writeUInt32LE(SR, 24);
buffer.writeUInt32LE(SR * 2 * bytesPerSample, 28);
buffer.writeUInt16LE(2 * bytesPerSample, 32);
buffer.writeUInt16LE(16, 34);
buffer.write("data", 36, "ascii");
buffer.writeUInt32LE(dataSize, 40);

let o = 44;
for (let i = 0; i < N; i += 1) {
  const l = Math.max(-1, Math.min(1, L[i]));
  const r = Math.max(-1, Math.min(1, R[i]));
  buffer.writeInt16LE(Math.round(l * 32767), o);
  buffer.writeInt16LE(Math.round(r * 32767), o + 2);
  o += 4;
}

const here = dirname(fileURLToPath(import.meta.url));
const out = resolve(here, "..", "public", "audio", "score.wav");
mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, buffer);

console.log(`wrote ${out}`);
console.log(`  ${DUR}s · ${SR}Hz · stereo · 16-bit · peak ${(peak * norm).toFixed(3)} · ${(dataSize / 1048576).toFixed(2)} MB`);
