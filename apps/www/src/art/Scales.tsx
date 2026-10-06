import type { CSSProperties } from "react";

interface Props {
  /** Degrees. Negative tips the left pan (the page) down. Omit to let CSS drive --tilt. */
  tilt?: number;
  className?: string;
  /** What sits in the left pan. */
  left?: "scroll" | "heart";
  marked?: boolean;
}

/**
 * The balance from the Weighing of the Heart: a page in one pan, the feather
 * of Ma'at (your settled canon) in the other. Pans stay plumb as the beam tips.
 */
export function Scales({ tilt, className, left = "scroll", marked = false }: Props) {
  const style = tilt === undefined ? undefined : ({ "--tilt": `${tilt}deg` } as CSSProperties);
  return (
    <svg
      className={`art scales ${marked ? "is-marked" : ""} ${className ?? ""}`}
      viewBox="0 0 320 300"
      aria-hidden="true"
    >
      {/* Plinth and post */}
      <path className="f-sand o" d="M96 268 H 224 L 216 284 H 104 Z" />
      <path className="f-ochre o" d="M112 256 H 208 V 268 H 112 Z" />
      <path className="f-ochre o" d="M154 104 H 166 L 168 256 H 152 Z" />
      {[132, 172, 212].map((y) => (
        <path key={y} className="f-none s-line w1" d={`M154.6 ${y} H 165.6`} />
      ))}
      {/* Papyrus-umbel capital */}
      <path className="f-green o" d="M146 108 C 150 94, 170 94, 174 108 Z" />
      <path className="f-gold o" d="M150 108 H 170 V 114 H 150 Z" />

      {/* Plummet, hanging plumb from the pivot */}
      <g>
        <path className="f-none s-line w1" d="M160 92 V 150" />
        <path className="f-red o-thin" d="M160 148 C 166 156, 166 164, 160 168 C 154 164, 154 156, 160 148 Z" />
      </g>

      {/* Beam, strings, and pans */}
      <g className="beam" style={style}>
        <path className="f-ochre o" d="M36 84 C 90 80, 230 80, 284 84 L 284 92 C 230 88, 90 88, 36 92 Z" />
        <circle className="f-gold o" cx="160" cy="88" r="7" />
        <path className="f-gold o-thin" d="M156 76 L 160 62 L 164 76 Z" />

        {/* Left pan: the page */}
        <g className="pan pan-left" style={{ transformOrigin: "40px 88px", transform: "rotate(calc(var(--tilt) * -1))" }}>
          <path className="f-none s-line w1" d="M40 88 L 12 166 M40 88 L 40 164 M40 88 L 68 166" />
          <path className="f-gold o" d="M4 164 H 76 C 70 182, 10 182, 4 164 Z" />
          {left === "scroll" ? (
            <g>
              <rect className="f-papyrus o" x="14" y="140" width="52" height="22" rx="11" />
              <ellipse className="f-sand o-thin" cx="58" cy="151" rx="6" ry="11" />
              <path className="f-none s-red w2" d="M34 141 V 161" />
              <path className="f-none s-line w1" d="M20 147 H 30 M20 152 H 28 M20 157 H 31" />
            </g>
          ) : (
            <g>
              <path className="f-red o" d="M28 136 C 26 128, 54 128, 52 136 L 50 140 C 60 146, 58 162, 40 162 C 22 162, 20 146, 30 140 Z" />
              <path className="f-none s-gold w2" d="M30 140 H 50" />
            </g>
          )}
        </g>

        {/* Right pan: the feather of Ma'at, your settled canon */}
        <g className="pan pan-right" style={{ transformOrigin: "280px 88px", transform: "rotate(calc(var(--tilt) * -1))" }}>
          <path className="f-none s-line w1" d="M280 88 L 252 166 M280 88 L 280 164 M280 88 L 308 166" />
          <path className="f-gold o" d="M244 164 H 316 C 310 182, 250 182, 244 164 Z" />
          <g className="feather">
            <path
              className="f-natron o"
              d="M278 162 C 272 140, 270 116, 278 98 C 284 86, 296 84, 300 92 C 292 92, 288 102, 290 116 C 292 134, 288 150, 282 162 Z"
            />
            <path className="f-none s-line w1" d="M280 160 C 278 140, 278 118, 286 100" />
            <path className="f-none s-blue w2" d="M276 124 C 280 122, 286 122, 290 125" />
            <path className="f-none s-green w2" d="M275 134 C 280 132, 286 132, 290 135" />
          </g>
        </g>
      </g>
    </svg>
  );
}
