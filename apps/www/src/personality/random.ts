/** Small seeded helpers shared by the hand-made motion: the same seed always draws the same wobble. */

export function rng(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

export const clamp = (v: number, lo = 0, hi = 1) => Math.min(hi, Math.max(lo, v));
export const lerp = (a: number, b: number, t: number) => a + (b - a) * t;
const smooth = (t: number) => t * t * (3 - 2 * t);

function hash(i: number, seed: number): number {
  let h = (i * 374761393 + seed * 668265263) | 0;
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

/** Smooth one-dimensional value noise in [-1, 1]. */
export function noise1(x: number, seed = 0): number {
  const i = Math.floor(x);
  const f = x - i;
  return lerp(hash(i, seed), hash(i + 1, seed), smooth(f)) * 2 - 1;
}

/** Layered noise, a little rougher: the way a hand wanders. */
export function fbm1(x: number, seed = 0, octaves = 3): number {
  let sum = 0;
  let amp = 1;
  let total = 0;
  let f = 1;
  for (let o = 0; o < octaves; o++) {
    sum += amp * noise1(x * f, seed + o * 31);
    total += amp;
    amp *= 0.5;
    f *= 2.07;
  }
  return sum / total;
}

/** Whether the visitor asked for stillness. */
export function reducedMotion(): boolean {
  return typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

/**
 * A damped spring stepped by hand. Returns the next [position, velocity]. Stiffness and damping are tuned
 * so a beam settles with about one small overshoot, the way a real balance does.
 */
export function springStep(x: number, v: number, target: number, dt: number, k = 90, c = 6.5): [number, number] {
  const sub = Math.max(1, Math.ceil(dt / 0.008));
  const h = dt / sub;
  for (let i = 0; i < sub; i++) {
    const a = -k * (x - target) - c * v;
    v += a * h;
    x += v * h;
  }
  return [x, v];
}
