import { fbm1, rng } from "./random";

export interface Pt {
  x: number;
  y: number;
}

export interface Box {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** One pencil stroke: a centre line (drawn on by a mask) and a variable-width brush outline (what you see). */
export interface Stroke {
  id: string;
  /** Centre line, used as the mask that reveals the brush as the pencil travels. */
  center: string;
  /** Closed outline of the stroke, thicker in the middle, thinner at the ends. */
  brush: string;
  length: number;
  /** Seconds after the sequence starts. */
  delay: number;
  /** Seconds the stroke takes to travel. */
  duration: number;
  opacity: number;
}

const fmt = (n: number) => Math.round(n * 10) / 10;

function pathOf(points: Pt[], close = false): string {
  return points.map((p, i) => `${i ? "L" : "M"}${fmt(p.x)} ${fmt(p.y)}`).join("") + (close ? "Z" : "");
}

function lengthOf(points: Pt[]): number {
  let total = 0;
  for (let i = 1; i < points.length; i++) total += Math.hypot(points[i].x - points[i - 1].x, points[i].y - points[i - 1].y);
  return total;
}

/** Thicken a centre line into an outline whose width follows `width(t)`, t running 0 to 1 along the stroke. */
function brushOutline(points: Pt[], width: (t: number) => number): string {
  const left: Pt[] = [];
  const right: Pt[] = [];
  const n = points.length;
  for (let i = 0; i < n; i++) {
    const a = points[Math.max(0, i - 1)];
    const b = points[Math.min(n - 1, i + 1)];
    const dx = b.x - a.x;
    const dy = b.y - a.y;
    const len = Math.hypot(dx, dy) || 1;
    const nx = -dy / len;
    const ny = dx / len;
    const w = width(i / (n - 1)) / 2;
    left.push({ x: points[i].x + nx * w, y: points[i].y + ny * w });
    right.push({ x: points[i].x - nx * w, y: points[i].y - ny * w });
  }
  return pathOf([...left, ...right.reverse()], true);
}

function densify(points: Pt[], step: number): Pt[] {
  // Catmull-Rom through the points, sampled every `step` px.
  const out: Pt[] = [];
  const P = (i: number) => points[Math.max(0, Math.min(points.length - 1, i))];
  for (let i = 0; i < points.length - 1; i++) {
    const p0 = P(i - 1);
    const p1 = P(i);
    const p2 = P(i + 1);
    const p3 = P(i + 2);
    const segLen = Math.hypot(p2.x - p1.x, p2.y - p1.y);
    const count = Math.max(2, Math.ceil(segLen / step));
    for (let s = 0; s < count; s++) {
      const t = s / count;
      const t2 = t * t;
      const t3 = t2 * t;
      out.push({
        x: 0.5 * (2 * p1.x + (-p0.x + p2.x) * t + (2 * p0.x - 5 * p1.x + 4 * p2.x - p3.x) * t2 + (-p0.x + 3 * p1.x - 3 * p2.x + p3.x) * t3),
        y: 0.5 * (2 * p1.y + (-p0.y + p2.y) * t + (2 * p0.y - 5 * p1.y + 4 * p2.y - p3.y) * t2 + (-p0.y + 3 * p1.y - 3 * p2.y + p3.y) * t3),
      });
    }
  }
  out.push(points[points.length - 1]);
  return out;
}

function makeStroke(id: string, points: Pt[], opts: { peak: number; delay: number; speed: number; opacity: number; seed: number }): Stroke {
  const dense = densify(points, 3);
  const length = lengthOf(dense);
  const wob = (t: number) => 1 + 0.18 * fbm1(t * 9, opts.seed);
  // Pressure: light at the start, heaviest a third of the way, lifting at the end.
  const width = (t: number) => Math.max(0.7, opts.peak * (0.42 + 0.58 * Math.sin(Math.PI * Math.pow(t, 0.8))) * wob(t));
  return {
    id,
    center: pathOf(dense),
    brush: brushOutline(dense, width),
    length,
    delay: opts.delay,
    duration: Math.max(0.28, length / opts.speed),
    opacity: opts.opacity,
  };
}

/** A loose loop round a word: starts at the upper left, runs a little past where it began, drifts outward. */
export function loopAround(id: string, box: Box, seed: number, delay: number, pass: 1 | 2): Stroke {
  const r = rng(seed);
  const cx = box.x + box.w / 2 + (r() - 0.5) * 4;
  const cy = box.y + box.h / 2 + (r() - 0.5) * 3;
  const rx0 = box.w / 2 + 11 + r() * 5 - (pass === 2 ? 3 : 0);
  const ry0 = box.h / 2 + 7 + r() * 3 - (pass === 2 ? 2 : 0);
  const tilt = (-5 + (r() - 0.5) * 5) * (Math.PI / 180);
  const start = -Math.PI * (0.78 + r() * 0.14);
  const sweep = Math.PI * 2 + 0.55 + r() * 0.5; // overshoot past the start
  const steps = 26;
  const pts: Pt[] = [];
  for (let i = 0; i <= steps; i++) {
    const t = i / steps;
    const a = start - t * sweep;
    const grow = 1 + 0.11 * t; // the pencil drifts outward as the hand turns
    const wobble = 1 + 0.05 * fbm1(t * 6 + seed, seed) + 0.025 * fbm1(t * 15, seed + 5);
    const x = Math.cos(a) * rx0 * grow * wobble;
    const y = Math.sin(a) * ry0 * grow * wobble;
    pts.push({ x: cx + x * Math.cos(tilt) - y * Math.sin(tilt), y: cy + x * Math.sin(tilt) + y * Math.cos(tilt) });
  }
  return makeStroke(id, pts, { peak: pass === 1 ? 3.4 : 2.1, delay, speed: pass === 1 ? 430 : 520, opacity: pass === 1 ? 0.92 : 0.6, seed });
}

/** A hand-drawn underline: a slight bow and a lazy wave. */
export function underline(id: string, x0: number, x1: number, y: number, seed: number, delay: number): Stroke {
  const r = rng(seed);
  const n = 12;
  const pts: Pt[] = [];
  const bow = 2 + r() * 3;
  for (let i = 0; i <= n; i++) {
    const t = i / n;
    pts.push({ x: x0 - 3 + (x1 - x0 + 8) * t, y: y + Math.sin(t * Math.PI) * bow + 1.6 * fbm1(t * 7 + seed, seed) + t * (r() * 0.4) });
  }
  return makeStroke(id, pts, { peak: 2.6, delay, speed: 360, opacity: 0.85, seed });
}

/** A curved leader with an open arrowhead, from `a` towards `b`. Returns the shaft and the head as two strokes. */
export function leader(id: string, a: Pt, b: Pt, seed: number, delay: number): Stroke[] {
  const r = rng(seed);
  const mx = (a.x + b.x) / 2;
  const my = (a.y + b.y) / 2;
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const len = Math.hypot(dx, dy) || 1;
  const bend = (10 + r() * 10) * (r() > 0.5 ? 1 : -1);
  const c = { x: mx + (-dy / len) * bend, y: my + (dx / len) * bend };
  const pts: Pt[] = [];
  for (let i = 0; i <= 10; i++) {
    const t = i / 10;
    pts.push({
      x: (1 - t) * (1 - t) * a.x + 2 * (1 - t) * t * c.x + t * t * b.x + 0.8 * fbm1(t * 8, seed),
      y: (1 - t) * (1 - t) * a.y + 2 * (1 - t) * t * c.y + t * t * b.y + 0.8 * fbm1(t * 8 + 3, seed),
    });
  }
  const shaft = makeStroke(`${id}-shaft`, pts, { peak: 2.2, delay, speed: 320, opacity: 0.85, seed });
  // Arrowhead: two short ticks back from the tip, along the end tangent.
  const tip = pts[pts.length - 1];
  const prev = pts[pts.length - 3];
  const ang = Math.atan2(tip.y - prev.y, tip.x - prev.x);
  const head = (sign: number): Pt[] => [
    { x: tip.x + Math.cos(ang + Math.PI + sign * 0.5) * 9, y: tip.y + Math.sin(ang + Math.PI + sign * 0.5) * 9 },
    { x: tip.x + (r() - 0.5), y: tip.y + (r() - 0.5) },
  ];
  const h1 = makeStroke(`${id}-head1`, head(1), { peak: 2, delay: delay + shaft.duration * 0.95, speed: 150, opacity: 0.85, seed: seed + 1 });
  const h2 = makeStroke(`${id}-head2`, head(-1), { peak: 2, delay: delay + shaft.duration * 0.95 + 0.07, speed: 150, opacity: 0.85, seed: seed + 2 });
  return [shaft, h1, h2];
}
