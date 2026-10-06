/** Smaller figures: the wedjat eye, the ibis on its standard, the lotus, the papyrus. */

export function Wedjat({ className }: { className?: string }) {
  return (
    <svg className={`art wedjat ${className ?? ""}`} viewBox="0 0 220 150" aria-hidden="true">
      {/* Brow */}
      <path className="f-lapis o" d="M30 34 C 70 14, 140 10, 196 22 L 194 34 C 140 24, 74 28, 34 46 Z" />
      {/* Eye: almond with the long kohl line */}
      <path className="f-natron o" d="M40 70 C 64 44, 120 40, 160 62 C 120 84, 66 88, 40 70 Z" />
      <g className="wedjat-iris">
        <circle className="f-ink" cx="100" cy="64" r="19" />
        <circle className="f-natron" cx="106" cy="57" r="4" />
      </g>
      <path className="f-ink" d="M40 70 C 64 44, 120 40, 160 62 L 206 62 L 206 70 L 156 70 C 116 48, 66 52, 44 74 Z" />
      <path className="f-ink" d="M40 70 C 66 88, 120 84, 160 62 L 154 70 C 116 90, 66 94, 40 74 Z" />
      {/* The falcon's cheek marking */}
      <path className="f-ink" d="M86 86 L 96 86 L 92 138 C 92 142, 84 142, 84 138 Z" />
      {/* The spiral tail */}
      <path
        className="f-none s-line"
        strokeWidth="8"
        d="M112 88 C 132 108, 150 124, 168 128 C 182 131, 188 120, 180 112 C 174 106, 164 110, 166 118"
      />
    </svg>
  );
}

/** The ibis on a standard: the hieroglyph that writes Thoth's name (G26). */
export function IbisStandard({ className }: { className?: string }) {
  return (
    <svg className={`art ibis ${className ?? ""}`} viewBox="0 0 220 260" aria-hidden="true">
      {/* Standard */}
      <path className="f-gold o" d="M60 168 H 184 V 180 H 60 Z" />
      <path className="f-ochre o" d="M116 180 H 128 V 252 H 116 Z" />
      <path className="f-red o" d="M100 252 H 144 V 258 H 100 Z" />
      <path className="f-none s-red w3" d="M128 186 C 144 196, 150 210, 148 226" />
      {/* Legs */}
      <path className="f-none s-line w3" d="M120 140 L 112 168 M134 140 L 138 168" />
      {/* Body */}
      <path
        className="f-natron o"
        d="M78 108 C 86 88, 118 82, 150 90 C 172 96, 190 108, 198 122 C 182 126, 168 136, 150 140 C 120 146, 90 138, 78 108 Z"
      />
      {/* Black tail plumes */}
      <path className="f-ink" d="M168 102 C 184 104, 200 114, 206 132 C 196 128, 186 128, 176 134 C 180 122, 176 110, 168 102 Z" />
      <path className="f-none s-line w1" d="M120 100 C 136 104, 150 112, 160 124" />
      {/* Neck and head */}
      <path className="f-ink" d="M84 104 C 72 94, 66 76, 70 58 L 82 58 C 80 74, 86 88, 96 96 Z" />
      <path className="f-ink" d="M64 58 C 62 46, 72 38, 82 40 C 90 42, 92 52, 88 60 C 84 64, 70 66, 64 58 Z" />
      <path className="f-ink" d="M66 52 C 46 58, 30 76, 20 104 C 18 108, 22 110, 24 106 C 34 82, 50 64, 68 60 Z" />
      <circle className="f-natron" cx="76" cy="49" r="3" />
    </svg>
  );
}

export function Lotus({ className }: { className?: string }) {
  return (
    <svg className={`art lotus ${className ?? ""}`} viewBox="0 0 80 90" aria-hidden="true">
      <path className="f-none s-green w3" d="M40 88 V 58" />
      <path className="f-green o" d="M40 60 C 30 58, 22 50, 20 40 C 30 42, 38 50, 40 60 Z" />
      <path className="f-green o" d="M40 60 C 50 58, 58 50, 60 40 C 50 42, 42 50, 40 60 Z" />
      <path className="f-blue o" d="M40 60 C 28 46, 24 28, 26 10 C 34 22, 40 38, 40 60 Z" />
      <path className="f-blue o" d="M40 60 C 52 46, 56 28, 54 10 C 46 22, 40 38, 40 60 Z" />
      <path className="f-turq o" d="M40 60 C 34 40, 34 20, 40 4 C 46 20, 46 40, 40 60 Z" />
    </svg>
  );
}

export function PapyrusPlant({ className }: { className?: string }) {
  return (
    <svg className={`art papyrus-plant ${className ?? ""}`} viewBox="0 0 120 200" aria-hidden="true">
      {[
        { x: 60, top: 40, lean: 0 },
        { x: 40, top: 70, lean: -14 },
        { x: 82, top: 64, lean: 14 },
      ].map((stem, i) => (
        <g key={i}>
          <path className="f-none s-green w3" d={`M60 198 C 60 160, ${stem.x - stem.lean / 2} 120, ${stem.x} ${stem.top + 20}`} />
          <path
            className="f-green o"
            d={`M${stem.x} ${stem.top + 22} C ${stem.x - 22} ${stem.top + 4}, ${stem.x - 24} ${stem.top - 10}, ${stem.x - 18} ${stem.top - 14} L ${stem.x + 18} ${stem.top - 14} C ${stem.x + 24} ${stem.top - 10}, ${stem.x + 22} ${stem.top + 4}, ${stem.x} ${stem.top + 22} Z`}
          />
          <path className="f-none s-line w1" d={`M${stem.x - 18} ${stem.top - 14} C ${stem.x - 10} ${stem.top - 20}, ${stem.x + 10} ${stem.top - 20}, ${stem.x + 18} ${stem.top - 14}`} />
        </g>
      ))}
    </svg>
  );
}
