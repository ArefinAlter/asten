import { useId } from "react";
import { starPath } from "./shapes";

export type FriezeKind = "blocks" | "khekher" | "lotus" | "stars" | "water";

const units: Record<FriezeKind, { w: number; h: number }> = {
  blocks: { w: 42, h: 16 },
  khekher: { w: 24, h: 44 },
  lotus: { w: 64, h: 40 },
  stars: { w: 40, h: 28 },
  water: { w: 24, h: 14 },
};

function Unit({ kind }: { kind: FriezeKind }) {
  switch (kind) {
    case "blocks":
      return (
        <g>
          <rect className="f-blue" x="1" y="2" width="11" height="12" />
          <rect className="f-red" x="15" y="2" width="11" height="12" />
          <rect className="f-green" x="29" y="2" width="11" height="12" />
          <path className="f-none s-line w1" d="M0 1 H42 M0 15 H42" />
        </g>
      );
    case "khekher":
      return (
        <g>
          <path
            className="f-green o-thin"
            d="M4 44 C 4 34, 8 28, 9 22 C 6 20, 6 14, 12 12 C 18 14, 18 20, 15 22 C 16 28, 20 34, 20 44 Z"
          />
          <path className="f-none s-line w1" d="M8 30 H16 M7.5 34 H16.5" />
          <circle className="f-red o-thin" cx="12" cy="7" r="4.5" />
        </g>
      );
    case "lotus":
      return (
        <g>
          {/* Open flower */}
          <path className="f-none s-green w15" d="M16 40 V 26 M16 30 C 30 42, 34 42, 48 30" />
          <path className="f-blue o-thin" d="M16 26 C 8 20, 6 10, 7 3 C 12 10, 16 18, 16 26 Z" />
          <path className="f-blue o-thin" d="M16 26 C 24 20, 26 10, 25 3 C 20 10, 16 18, 16 26 Z" />
          <path className="f-turq o-thin" d="M16 26 C 13 18, 13 8, 16 1 C 19 8, 19 18, 16 26 Z" />
          {/* Bud */}
          <g className="bud">
            <path className="f-none s-green w15" d="M48 40 V 24" />
            <path className="f-blue o-thin" d="M48 26 C 41 22, 42 12, 48 6 C 54 12, 55 22, 48 26 Z" />
            <path className="f-green o-thin" d="M48 26 C 44 24, 43 20, 44 17 C 47 19, 48 22, 48 26 Z" />
            <path className="f-green o-thin" d="M48 26 C 52 24, 53 20, 52 17 C 49 19, 48 22, 48 26 Z" />
          </g>
        </g>
      );
    case "stars":
      return (
        <g>
          <rect className="f-lapis" x="0" y="0" width="40" height="28" />
          <path className="f-star" d={starPath(20, 14.6, 8, 0.3)} />
          <path className="f-none s-gold w15" d="M0 1 H40 M0 27 H40" />
        </g>
      );
    case "water":
      return <path className="f-none s-blue w2" d="M0 9 L 6 3 L 12 9 L 18 3 L 24 9" />;
  }
}

/** A full-width ornamental band, repeated from a single unit. */
export function Frieze({ kind, className }: { kind: FriezeKind; className?: string }) {
  const id = useId().replaceAll(":", "");
  const { w, h } = units[kind];
  return (
    <svg
      className={`art frieze frieze-${kind} ${className ?? ""}`}
      height={h}
      width="100%"
      aria-hidden="true"
      focusable="false"
    >
      <defs>
        <pattern id={`fz-${id}`} width={w} height={h} patternUnits="userSpaceOnUse">
          <Unit kind={kind} />
        </pattern>
      </defs>
      <rect width="100%" height={h} fill={`url(#fz-${id})`} />
    </svg>
  );
}

/** A frieze drawn as individual units, so each lotus can open on hover. */
export function LiveLotusFrieze({ count = 24, className }: { count?: number; className?: string }) {
  const { w, h } = units.lotus;
  return (
    <svg
      className={`art frieze frieze-live ${className ?? ""}`}
      viewBox={`0 0 ${count * w} ${h}`}
      preserveAspectRatio="xMidYMax slice"
      aria-hidden="true"
    >
      {Array.from({ length: count }, (_, i) => (
        <g key={i} transform={`translate(${i * w} 0)`} style={{ ["--i" as string]: i }}>
          <Unit kind="lotus" />
        </g>
      ))}
    </svg>
  );
}
