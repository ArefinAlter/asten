import type { CSSProperties } from "react";
import { rng } from "./random";

/**
 * The seam between two sheets of the page: the next sheet lies over the last with a torn top edge and a shadow
 * under it, and slides up into place as it arrives. The edge is drawn once, from a seed, so every visit sees the
 * same tear. `to` names the surface of the sheet below, so the edge takes its colour.
 */
export function PageTurn({ to = "papyrus", seed = 1, curl = false }: { to?: "papyrus" | "sunk" | "raised" | "night"; seed?: number; curl?: boolean }) {
  const r = rng(seed * 977);
  const w = 1200;
  const h = 40;
  let y = 14 + r() * 8;
  let d = `M0 ${h} L0 ${y.toFixed(1)}`;
  for (let x = 0; x < w; x += 14 + r() * 16) {
    y = Math.max(5, Math.min(30, y + (r() - 0.5) * 7));
    d += ` L${(x + 8).toFixed(1)} ${y.toFixed(1)}`;
  }
  d += ` L${w} ${(14 + r() * 8).toFixed(1)} L${w} ${h} Z`;
  return (
    <div className={`page-turn page-turn--${to}`} aria-hidden="true" style={{ "--turn-seed": seed } as CSSProperties}>
      <svg viewBox={`0 0 ${w} ${h}`} preserveAspectRatio="none" focusable="false">
        <path d={d} />
      </svg>
      {curl ? <span className="page-curl" /> : null}
    </div>
  );
}
