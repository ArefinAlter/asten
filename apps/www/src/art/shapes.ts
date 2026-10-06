/** Geometry helpers shared by the illustrations. */

export function starPath(cx: number, cy: number, r: number, inner = 0.36): string {
  const points: string[] = [];
  for (let i = 0; i < 10; i++) {
    const radius = i % 2 === 0 ? r : r * inner;
    const angle = -Math.PI / 2 + (i * Math.PI) / 5;
    points.push(`${(cx + radius * Math.cos(angle)).toFixed(2)} ${(cy + radius * Math.sin(angle)).toFixed(2)}`);
  }
  return `M${points.join(" L")} Z`;
}

/** Deterministic pseudo-random sequence, so server and client render the same art. */
export function seeded(seed: number): () => number {
  let state = seed >>> 0;
  return () => {
    state = (state * 1664525 + 1013904223) >>> 0;
    return state / 4294967296;
  };
}

/** Path for the lit part of the moon. phase: 0 new, 0.5 full, 1 new again. */
export function moonPath(cx: number, cy: number, r: number, phase: number): string {
  const p = ((phase % 1) + 1) % 1;
  const waxing = p <= 0.5;
  const k = Math.cos(p * 2 * Math.PI); // 1 at new, -1 at full
  const rx = Math.abs(k) * r;
  const top = `${cx} ${cy - r}`;
  const bottom = `${cx} ${cy + r}`;
  // Outer limb: right side when waxing, left when waning.
  const outerSweep = waxing ? 1 : 0;
  // Terminator bulges toward the lit side when gibbous.
  const innerSweep = waxing ? (k > 0 ? 0 : 1) : k > 0 ? 1 : 0;
  return `M${top} A${r} ${r} 0 0 ${outerSweep} ${bottom} A${rx.toFixed(2)} ${r} 0 0 ${innerSweep} ${top} Z`;
}
