/**
 * A small force layout for the graph view: a centre force, a repel force
 * between every pair, and springs along links, with the four controls the
 * Forces section exposes. Deterministic (seeded start positions), bounded
 * (velocities clamped), so it is tested and never runs away.
 */

export interface Forces {
  center: number;
  repel: number;
  link: number;
  distance: number;
}

export const DEFAULT_FORCES: Forces = {
  center: 0.03,
  repel: 2400,
  link: 0.05,
  distance: 120,
};

export interface Body {
  id: string;
  x: number;
  y: number;
  vx: number;
  vy: number;
  fixed?: boolean;
}

/** Start positions on a seeded spiral so the same graph lays out the same way. */
export function seedBodies(
  ids: string[],
  previous?: Map<string, Body>,
): Body[] {
  return ids.map((id, index) => {
    const old = previous?.get(id);
    if (old) return { ...old };
    const angle = index * 2.399963;
    const radius = 30 + 14 * Math.sqrt(index);
    return {
      id,
      x: Math.cos(angle) * radius,
      y: Math.sin(angle) * radius,
      vx: 0,
      vy: 0,
    };
  });
}

const MAX_V = 40;

export function step(
  bodies: Body[],
  links: { from: string; to: string }[],
  forces: Forces,
  damping = 0.82,
): number {
  const index = new Map(bodies.map((b, i) => [b.id, i]));
  const fx = new Float64Array(bodies.length);
  const fy = new Float64Array(bodies.length);
  for (let i = 0; i < bodies.length; i += 1) {
    const a = bodies[i];
    fx[i] -= a.x * forces.center;
    fy[i] -= a.y * forces.center;
    for (let j = i + 1; j < bodies.length; j += 1) {
      const b = bodies[j];
      let dx = a.x - b.x;
      let dy = a.y - b.y;
      let d2 = dx * dx + dy * dy;
      if (d2 < 0.01) {
        dx = (i - j) * 0.1;
        dy = 0.1;
        d2 = dx * dx + dy * dy;
      }
      const force = forces.repel / d2;
      const d = Math.sqrt(d2);
      fx[i] += (dx / d) * force;
      fy[i] += (dy / d) * force;
      fx[j] -= (dx / d) * force;
      fy[j] -= (dy / d) * force;
    }
  }
  for (const link of links) {
    const i = index.get(link.from);
    const j = index.get(link.to);
    if (i === undefined || j === undefined) continue;
    const a = bodies[i];
    const b = bodies[j];
    const dx = b.x - a.x;
    const dy = b.y - a.y;
    const d = Math.sqrt(dx * dx + dy * dy) || 1;
    const force = (d - forces.distance) * forces.link;
    fx[i] += (dx / d) * force;
    fy[i] += (dy / d) * force;
    fx[j] -= (dx / d) * force;
    fy[j] -= (dy / d) * force;
  }
  let energy = 0;
  bodies.forEach((body, i) => {
    if (body.fixed) {
      body.vx = 0;
      body.vy = 0;
      return;
    }
    body.vx = Math.max(-MAX_V, Math.min(MAX_V, (body.vx + fx[i]) * damping));
    body.vy = Math.max(-MAX_V, Math.min(MAX_V, (body.vy + fy[i]) * damping));
    body.x += body.vx;
    body.y += body.vy;
    energy += body.vx * body.vx + body.vy * body.vy;
  });
  return energy;
}
