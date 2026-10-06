/**
 * Icons drawn from hieroglyphs that already mean the thing. 32 × 32 grid,
 * line in currentColor, one optional flat fill. See docs/BRAND.md §10.
 */

export type GlyphKey =
  | "scroll"
  | "seated"
  | "moon"
  | "hills"
  | "feather"
  | "kit"
  | "papyrus"
  | "door"
  | "house"
  | "eye"
  | "scales"
  | "mouth"
  | "heart"
  | "star"
  | "ibis"
  | "lotus"
  | "water"
  | "sun";

const paths: Record<GlyphKey, React.ReactNode> = {
  scroll: (
    <>
      <rect className="g-fill" x="4" y="10" width="24" height="12" rx="6" />
      <rect className="g-line" x="4" y="10" width="24" height="12" rx="6" />
      <path className="g-line" d="M22 10.5 C 19.5 13, 19.5 19, 22 21.5" />
      <path className="g-line" d="M13 9 V 23 M15.5 9 V 23" />
    </>
  ),
  seated: (
    <>
      <circle className="g-fill" cx="12" cy="7" r="3.4" />
      <circle className="g-line" cx="12" cy="7" r="3.4" />
      <path className="g-line" d="M11 11 C 9 15, 9 19, 10.5 22 H 20 L 22 28 M10.5 22 L 9 28 M11.5 14 L 19 17.5" />
      <path className="g-line" d="M5 28 H 27" />
    </>
  ),
  moon: (
    <>
      <circle className="g-fill" cx="16" cy="19" r="7" />
      <circle className="g-line" cx="16" cy="19" r="7" />
      <path className="g-line" d="M5 10 C 7 17, 25 17, 27 10" />
    </>
  ),
  hills: (
    <>
      <path className="g-fill" d="M4 25 C 6 15, 10 15, 12 25 C 14 11, 18 11, 20 25 C 22 15, 26 15, 28 25 Z" />
      <path className="g-line" d="M4 25 C 6 15, 10 15, 12 25 C 14 11, 18 11, 20 25 C 22 15, 26 15, 28 25" />
      <path className="g-line" d="M3 27 H 29" />
    </>
  ),
  feather: (
    <>
      <path className="g-fill" d="M15 28 C 12 20, 12 11, 16 6 C 19 3, 24 3, 25 6 C 21 6, 19 9, 19.5 14 C 20 20, 18 25, 17 28 Z" />
      <path className="g-line" d="M15 28 C 12 20, 12 11, 16 6 C 19 3, 24 3, 25 6 C 21 6, 19 9, 19.5 14 C 20 20, 18 25, 17 28 Z" />
      <path className="g-line" d="M16 27 C 15 20, 15.5 13, 18 8" />
    </>
  ),
  kit: (
    <>
      <rect className="g-fill" x="5" y="4" width="8" height="24" rx="2" />
      <rect className="g-line" x="5" y="4" width="8" height="24" rx="2" />
      <circle className="g-dot" cx="9" cy="8.5" r="1.6" />
      <circle className="g-line" cx="9" cy="13" r="1.6" />
      <path className="g-line" d="M13 9 C 17 8, 19 11, 20 14 M20 12 H 25 V 22 C 25 25, 20 25, 20 22 Z M24 4 L 22 12" />
    </>
  ),
  papyrus: (
    <>
      <path className="g-line" d="M16 28 V 12 M16 28 C 15 22, 11 18, 8 15 M16 28 C 17 22, 21 18, 24 15" />
      <path className="g-fill" d="M16 13 C 11 9, 11 5, 12 4 H 20 C 21 5, 21 9, 16 13 Z" />
      <path className="g-line" d="M16 13 C 11 9, 11 5, 12 4 H 20 C 21 5, 21 9, 16 13 Z" />
      <path className="g-line" d="M8 16 C 4 14, 4 11, 5 10 L 10 11 C 11 13, 10 15, 8 16 Z M24 16 C 28 14, 28 11, 27 10 L 22 11 C 21 13, 22 15, 24 16 Z" />
    </>
  ),
  door: (
    <>
      <rect className="g-fill" x="10" y="9" width="12" height="18" />
      <path className="g-line" d="M5 6 H 27 M7 6 V 27 M25 6 V 27 M10 9 H 22 V 27 H 10 Z M18 18 V 19" />
      <path className="g-line" d="M4 27 H 28" />
    </>
  ),
  house: (
    <>
      <path className="g-fill" d="M6 7 H 26 V 25 H 6 Z" />
      <path className="g-line" d="M14 25 H 6 V 7 H 26 V 25 H 19" />
      <circle className="g-dot" cx="16" cy="16" r="1.6" />
    </>
  ),
  eye: (
    <>
      <path className="g-fill" d="M4 15 C 9 9, 19 9, 25 14 C 19 19, 9 19, 4 15 Z" />
      <path className="g-line" d="M3 9 C 10 5, 20 5, 28 7 M4 15 C 9 9, 19 9, 25 14 C 19 19, 9 19, 4 15 Z M25 14 H 29 M12 19 L 11.5 27 M15 19 C 18 23, 21 25, 23 24 C 25 23, 24 21, 22.5 21.5" />
      <circle className="g-dot" cx="14" cy="14" r="2.6" />
    </>
  ),
  scales: (
    <>
      <path className="g-line" d="M16 6 V 26 M11 27 H 21 M5 9 H 27 M5 9 L 2.5 17 M5 9 L 7.5 17 M27 9 L 24.5 17 M27 9 L 29.5 17" />
      <path className="g-fill" d="M2 17 H 8 C 7.5 20, 2.5 20, 2 17 Z M24 17 H 30 C 29.5 20, 24.5 20, 24 17 Z" />
      <path className="g-line" d="M2 17 H 8 C 7.5 20, 2.5 20, 2 17 Z M24 17 H 30 C 29.5 20, 24.5 20, 24 17 Z" />
      <circle className="g-dot" cx="16" cy="9" r="1.8" />
    </>
  ),
  mouth: (
    <>
      <path className="g-fill" d="M4 16 C 10 8, 22 8, 28 16 C 22 24, 10 24, 4 16 Z" />
      <path className="g-line" d="M4 16 C 10 8, 22 8, 28 16 C 22 24, 10 24, 4 16 Z M7 16 H 25" />
    </>
  ),
  heart: (
    <>
      <path className="g-fill" d="M11 6 C 10 4, 22 4, 21 6 L 20 9 C 26 12, 26 24, 16 27 C 6 24, 6 12, 12 9 Z" />
      <path className="g-line" d="M11 6 C 10 4, 22 4, 21 6 L 20 9 C 26 12, 26 24, 16 27 C 6 24, 6 12, 12 9 Z M12 9 H 20 M9 13 L 6 11 M23 13 L 26 11" />
    </>
  ),
  star: (
    <>
      <path className="g-fill" d="M16 4 L 18.4 13 L 28 13 L 20.3 18.6 L 23.2 28 L 16 22.2 L 8.8 28 L 11.7 18.6 L 4 13 L 13.6 13 Z" />
      <path className="g-line" d="M16 4 L 18.4 13 L 28 13 L 20.3 18.6 L 23.2 28 L 16 22.2 L 8.8 28 L 11.7 18.6 L 4 13 L 13.6 13 Z" />
    </>
  ),
  ibis: (
    <>
      <path className="g-fill" d="M12 15 C 14 11, 20 11, 25 13 L 28 17 C 24 18, 20 20, 15 19 C 13 18.5, 12 17, 12 15 Z" />
      <path className="g-line" d="M12 15 C 14 11, 20 11, 25 13 L 28 17 C 24 18, 20 20, 15 19 C 13 18.5, 12 17, 12 15 Z M13 15 C 10 12, 9 9, 10 6 M10 6 C 8 5, 7 5, 6 6 C 4 9, 3 12, 2.5 15 M17 19.5 L 16 27 M21 19 L 22 27 M11 27 H 27" />
      <circle className="g-dot" cx="9" cy="6.5" r="1.1" />
    </>
  ),
  lotus: (
    <>
      <path className="g-fill" d="M16 22 C 12 16, 11 9, 12 4 C 14 9, 16 14, 16 22 Z M16 22 C 20 16, 21 9, 20 4 C 18 9, 16 14, 16 22 Z" />
      <path className="g-line" d="M16 22 C 12 16, 11 9, 12 4 C 14 9, 16 14, 16 22 Z M16 22 C 20 16, 21 9, 20 4 C 18 9, 16 14, 16 22 Z M16 22 C 9 21, 6 16, 5 11 C 10 13, 14 17, 16 22 Z M16 22 C 23 21, 26 16, 27 11 C 22 13, 18 17, 16 22 Z M16 22 V 28" />
    </>
  ),
  water: (
    <path className="g-line" d="M3 11 L 7 7 L 11 11 L 15 7 L 19 11 L 23 7 L 27 11 M3 18 L 7 14 L 11 18 L 15 14 L 19 18 L 23 14 L 27 18 M3 25 L 7 21 L 11 25 L 15 21 L 19 25 L 23 21 L 27 25" />
  ),
  sun: (
    <>
      <circle className="g-fill" cx="16" cy="16" r="9" />
      <circle className="g-line" cx="16" cy="16" r="9" />
      <circle className="g-dot" cx="16" cy="16" r="2.2" />
    </>
  ),
};

export function Glyph({ name, className, label }: { name: GlyphKey; className?: string; label?: string }) {
  return (
    <svg
      className={`glyph glyph-${name} ${className ?? ""}`}
      viewBox="0 0 32 32"
      role={label ? "img" : undefined}
      aria-hidden={label ? undefined : true}
      aria-label={label}
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      {paths[name]}
    </svg>
  );
}
