export interface Chamber {
  id: string;
  label: string;
  x: number;
  y: number;
  w: number;
  h: number;
}

/** Chambers below the sand: the parts of a world a reader never sees directly. */
export const chambers: Chamber[] = [
  { id: "maps", label: "Maps", x: 116, y: 236, w: 70, h: 34 },
  { id: "history", label: "History", x: 206, y: 222, w: 74, h: 34 },
  { id: "factions", label: "Factions", x: 300, y: 238, w: 76, h: 34 },
  { id: "rules", label: "Magic rules", x: 150, y: 296, w: 88, h: 34 },
  { id: "families", label: "Family trees", x: 262, y: 300, w: 96, h: 34 },
];

interface Props {
  className?: string;
  active?: string | null;
  onEnter?: (id: string | null) => void;
}

/**
 * A pyramid in section. Above the sand: the novel, which readers see. Below:
 * the world, which mostly stays buried.
 */
export function PyramidSection({ className, active = null, onEnter }: Props) {
  return (
    <svg className={`art pyramid ${className ?? ""}`} viewBox="0 0 480 360" aria-hidden="true">
      {/* Sun disk */}
      <circle className="f-gold" cx="408" cy="54" r="22" opacity="0.9" />
      {/* Sand below the surface */}
      <path className="f-sand" d="M0 200 H 480 V 360 H 0 Z" />
      {Array.from({ length: 22 }, (_, i) => (
        <circle
          key={i}
          className="f-shadow"
          cx={(i * 53) % 470 + 6}
          cy={214 + ((i * 37) % 136)}
          r={1.4}
        />
      ))}
      {/* The pyramid, in courses */}
      <path className="f-ochre o" d="M90 200 L 240 44 L 390 200 Z" />
      <path className="f-gold o" d="M240 44 L 226 58.6 L 254 58.6 Z" />
      {[0, 1, 2, 3, 4, 5, 6, 7].map((i) => {
        const y = 78 + i * 16;
        const half = ((y - 44) / 156) * 150;
        return <path key={i} className="f-none s-line w1" opacity="0.45" d={`M${240 - half} ${y} H ${240 + half}`} />;
      })}
      <path className="f-none s-line w15" d="M240 44 L 240 200" opacity="0.18" />
      {/* Descending passage and corridors between the chambers */}
      <path className="f-ink" d="M234 200 h 12 v -10 h -12 Z" opacity="0.85" />
      <path
        className="f-none s-line w15"
        strokeDasharray="3 4"
        d="M240 200 V 222 M186 253 H 206 M280 239 H 300 M194 270 V 296 M243 256 V 286 L 300 300 M243 286 L 214 296"
      />
      {/* Ground line */}
      <path className="f-none s-line w3" d="M0 200 H 480" />
      <path className="f-none s-line w1" d="M0 205 H 480" />

      {chambers.map((chamber) => (
        <g
          key={chamber.id}
          className={`chamber${active === chamber.id ? " is-active" : ""}`}
          onMouseEnter={() => onEnter?.(chamber.id)}
          onMouseLeave={() => onEnter?.(null)}
        >
          <rect
            className="chamber-box"
            x={chamber.x}
            y={chamber.y}
            width={chamber.w}
            height={chamber.h}
            rx="3"
          />
          <text className="chamber-label" x={chamber.x + chamber.w / 2} y={chamber.y + chamber.h / 2 + 4}>
            {chamber.label}
          </text>
        </g>
      ))}

      <text className="pyramid-caption" x="14" y="350">
        below the sand
      </text>
      <text className="pyramid-caption" x="14" y="28">
        what readers see
      </text>
    </svg>
  );
}
