/**
 * Thoth the scribe, in the Egyptian flat canon: ibis head in profile, frontal
 * shoulders, striding legs, lunar disk and crescent. Drawn on the 18-square
 * grid (one square = 20 units; ground at y = 500, hairline at y = 140).
 * Faces right. Mirror with `flip`.
 */
export function Thoth({ className, flip = false }: { className?: string; flip?: boolean }) {
  const beads = Array.from({ length: 9 }, (_, i) => {
    const t = i / 8;
    return { x: 106 + t * 92, y: 202 + Math.sin(t * Math.PI) * 32 };
  });
  return (
    <svg className={`art thoth ${className ?? ""}`} viewBox="0 0 330 520" aria-hidden="true">
      <g transform={flip ? "translate(330 0) scale(-1 1)" : undefined}>
        {/* Lunar crown: the full disk resting in a crescent */}
        <circle className="f-gold moon-glow" cx="146" cy="74" r="40" />
        <circle className="f-gold o" cx="146" cy="74" r="24" />
        <path className="f-gold o" d="M112 88 C 116 116, 176 116, 180 88 C 170 102, 122 102, 112 88 Z" />

        {/* Wig: back mass, falling behind the shoulder */}
        <path
          className="f-lapis o"
          d="M118 112 C 100 126, 96 160, 100 198 L 134 202 L 150 116 Z"
        />
        {[0, 1, 2, 3, 4].map((i) => (
          <path key={i} className="f-none s-line w1" d={`M${103 + i} ${136 + i * 13} H ${134 - i * 2}`} />
        ))}

        {/* Torso: frontal shoulders narrowing to a profile waist */}
        <path
          className="f-flesh o"
          d="M94 192 C 104 182, 128 179, 150 179 C 172 179, 196 181, 210 189
             C 214 201, 207 212, 197 220 C 190 238, 185 258, 181 282
             L 127 282 C 124 256, 114 226, 94 192 Z"
        />

        {/* Broad collar (wesekh) */}
        <path
          className="f-gold o"
          d="M99 193 C 117 240, 187 240, 207 193 C 197 187, 183 183, 171 182
             C 165 197, 137 197, 131 182 C 119 183, 107 187, 99 193 Z"
        />
        <path className="f-none s-blue w4" d="M109 197 C 125 227, 181 227, 199 197" />
        <path className="f-none s-red w3" d="M116 198 C 131 217, 175 217, 192 198" />
        <path className="f-none s-green w3" d="M123 194 C 135 207, 167 207, 183 194" />
        {beads.map((bead, i) => (
          <ellipse key={i} className="f-blue o-thin" cx={bead.x} cy={bead.y} rx="3.4" ry="5" />
        ))}

        {/* Kilt (shendyt): pleated apron and belt */}
        <path
          className="f-natron o"
          d="M125 282 L 183 282 C 191 310, 201 340, 211 370 L 117 370 C 119 340, 121 310, 125 282 Z"
        />
        <path className="f-natron o" d="M161 290 L 211 370 L 169 370 Z" />
        {[0, 1, 2, 3, 4].map((i) => (
          <path key={i} className="f-none o-hair" d={`M${165 + i * 2} ${296 + i * 4} L ${175 + i * 8} 368`} />
        ))}
        <path className="f-gold o" d="M124 276 L 184 276 L 185 290 L 123 290 Z" />
        <path className="f-none s-red w2" d="M127 283 H 181" />

        {/* Legs, striding: the far leg, then the near leg forward */}
        <path
          className="f-flesh o"
          d="M125 370 C 125 400, 131 420, 129 450 C 128 466, 129 476, 131 484
             L 123 498 L 177 500 C 171 492, 157 488, 147 482
             C 147 460, 151 420, 151 370 Z"
        />
        <path
          className="f-flesh o"
          d="M171 370 C 177 400, 193 424, 199 450 C 203 464, 207 474, 209 482
             L 201 498 L 263 500 C 255 490, 239 486, 227 480
             C 223 452, 211 410, 205 370 Z"
        />

        {/* Far forearm, emerging forward from the chest, holding the palette */}
        <path
          className="f-flesh o"
          d="M180 262 C 198 258, 216 254, 234 250 L 236 264 C 218 268, 200 273, 182 278 Z"
        />
        <path className="f-gold o-thin" d="M200 257 L 210 255 L 212 268 L 202 270 Z" />
        <g transform="rotate(-9 240 256)">
          <rect className="f-natron o" x="226" y="248" width="94" height="17" rx="4" />
          <circle className="f-ink" cx="238" cy="256.5" r="4" />
          <circle className="f-red" cx="250" cy="256.5" r="4" />
          <rect className="f-sunk o-thin" x="261" y="253" width="50" height="7" rx="3.5" />
        </g>
        <circle className="f-flesh o" cx="236" cy="257" r="8.5" />

        {/* Near arm: shoulder to elbow, forearm raised, reed pen to the palette */}
        <path
          className="f-flesh o"
          d="M196 194 C 212 206, 218 232, 214 262 L 198 262 C 200 238, 194 218, 186 206 Z"
        />
        <g className="pen-hand">
          <path
            className="f-flesh o"
            d="M199 254 C 214 244, 236 230, 254 220 L 262 232 C 244 242, 226 256, 211 268 Z"
          />
          <path className="f-gold o-thin" d="M206 252 L 215 247 L 222 259 L 213 264 Z" />
          {/* Reed pen, nib down on the palette */}
          <path className="f-none s-line" strokeWidth="5" d="M246 210 L 288 246" />
          <path className="f-none s-ochre" strokeWidth="2.6" d="M246 210 L 288 246" />
          <path className="f-ink" d="M285 242 L 292 250 L 283 247 Z" />
          <circle className="f-flesh o" cx="260" cy="224" r="9" />
        </g>

        {/* Wig: front lappet, falling in front of the near shoulder */}
        <path
          className="f-lapis o"
          d="M156 140 C 164 164, 170 194, 168 232 L 186 230 C 190 198, 184 162, 174 134 Z"
        />
        {[0, 1, 2, 3, 4, 5, 6].map((i) => (
          <path key={i} className="f-none s-line w1" d={`M${160 + i * 1.2} ${154 + i * 11} H ${181 - i * 0.2}`} />
        ))}
        <path className="f-gold o-thin" d="M166 222 L 186 220 L 186 230 L 168 232 Z" />

        {/* Ibis head: small skull, long curved beak */}
        <path
          className="f-ink"
          d="M130 120 C 131 106, 144 98, 158 100 C 172 102, 182 112, 182 124
             C 182 136, 174 144, 164 148 C 160 156, 158 166, 158 180 L 144 182
             C 146 166, 144 156, 138 148 C 132 142, 129 132, 130 120 Z"
        />
        <path
          className="f-ink"
          d="M172 112 C 206 116, 240 140, 266 186 C 269 192, 266 194, 262 189
             C 238 154, 208 136, 176 132 Z"
        />
        <g className="eye">
          <circle className="f-natron" cx="161" cy="117" r="5" />
          <circle className="f-ink" cx="162.3" cy="117" r="2.3" />
        </g>
        <path className="f-none s-natron w15" d="M156 119 L 145 123" />
      </g>
    </svg>
  );
}
