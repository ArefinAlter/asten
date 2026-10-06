import type { CSSProperties } from "react";
import { moonPath, seeded, starPath } from "./shapes";

/**
 * A tomb-ceiling sky: rows of five-pointed stars on lapis, a few of which
 * twinkle. Absolutely positioned behind a night register.
 */
export function StarField({ className, rows = 6, cols = 14, seed = 7 }: { className?: string; rows?: number; cols?: number; seed?: number }) {
  const random = seeded(seed);
  const stars: { x: number; y: number; r: number; t: number; twinkle: boolean }[] = [];
  const width = 1400;
  const height = 600;
  const dx = width / cols;
  const dy = height / rows;
  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      const x = col * dx + dx / 2 + (row % 2 ? dx / 2 : 0) + (random() - 0.5) * 6;
      const y = row * dy + dy / 2 + (random() - 0.5) * 6;
      stars.push({ x, y, r: 7 + random() * 3, t: Math.floor(random() * 9), twinkle: random() < 0.28 });
    }
  }
  return (
    <svg
      className={`art starfield ${className ?? ""}`}
      viewBox={`0 0 ${width} ${height}`}
      preserveAspectRatio="xMidYMid slice"
      aria-hidden="true"
    >
      {stars.map((star, i) => (
        <path
          key={i}
          className={`f-star${star.twinkle ? " twinkle" : ""}`}
          d={starPath(star.x, star.y, star.r, 0.3)}
          style={star.twinkle ? ({ "--t": star.t } as CSSProperties) : undefined}
        />
      ))}
    </svg>
  );
}

/** Thoth's lunar emblem: a disk resting in a crescent. */
export function LunarEmblem({ className }: { className?: string }) {
  return (
    <svg className={`art lunar ${className ?? ""}`} viewBox="0 0 120 120" aria-hidden="true">
      <circle className="f-gold o" cx="60" cy="50" r="30" />
      <path className="f-gold o" d="M16 66 C 22 108, 98 108, 104 66 C 90 88, 30 88, 16 66 Z" />
    </svg>
  );
}

/** A moon at any phase. 0 new, 0.25 first quarter, 0.5 full, 0.75 last quarter. */
export function MoonPhase({ phase, className, r = 20 }: { phase: number; className?: string; r?: number }) {
  const size = r * 2 + 4;
  const c = size / 2;
  return (
    <svg className={`art moon-phase ${className ?? ""}`} viewBox={`0 0 ${size} ${size}`} aria-hidden="true">
      <circle className="moon-dark" cx={c} cy={c} r={r} />
      <path className="moon-lit" d={moonPath(c, c, r, phase)} />
      <circle className="moon-rim" cx={c} cy={c} r={r} fill="none" />
    </svg>
  );
}
