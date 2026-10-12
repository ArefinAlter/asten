import type { ReactNode } from "react";

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
  | "star"
  | "water"
  | "sun"
  | "lotus"
  | "ibis"
  | "pyramid";

const paths: Record<GlyphKey, ReactNode> = {
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
      <path
        className="g-line"
        d="M11 11 C 9 15, 9 19, 10.5 22 H 20 L 22 28 M10.5 22 L 9 28 M11.5 14 L 19 17.5"
      />
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
      <path
        className="g-fill"
        d="M4 25 C 6 15, 10 15, 12 25 C 14 11, 18 11, 20 25 C 22 15, 26 15, 28 25 Z"
      />
      <path
        className="g-line"
        d="M4 25 C 6 15, 10 15, 12 25 C 14 11, 18 11, 20 25 C 22 15, 26 15, 28 25"
      />
      <path className="g-line" d="M3 27 H 29" />
    </>
  ),
  feather: (
    <>
      <path
        className="g-fill"
        d="M15 28 C 12 20, 12 11, 16 6 C 19 3, 24 3, 25 6 C 21 6, 19 9, 19.5 14 C 20 20, 18 25, 17 28 Z"
      />
      <path
        className="g-line"
        d="M15 28 C 12 20, 12 11, 16 6 C 19 3, 24 3, 25 6 C 21 6, 19 9, 19.5 14 C 20 20, 18 25, 17 28 Z"
      />
      <path className="g-line" d="M16 27 C 15 20, 15.5 13, 18 8" />
    </>
  ),
  kit: (
    <>
      <rect className="g-fill" x="5" y="4" width="8" height="24" rx="2" />
      <rect className="g-line" x="5" y="4" width="8" height="24" rx="2" />
      <circle className="g-dot" cx="9" cy="8.5" r="1.6" />
      <circle className="g-line" cx="9" cy="13" r="1.6" />
      <path
        className="g-line"
        d="M13 9 C 17 8, 19 11, 20 14 M20 12 H 25 V 22 C 25 25, 20 25, 20 22 Z M24 4 L 22 12"
      />
    </>
  ),
  papyrus: (
    <>
      <path
        className="g-line"
        d="M16 28 V 12 M16 28 C 15 22, 11 18, 8 15 M16 28 C 17 22, 21 18, 24 15"
      />
      <path
        className="g-fill"
        d="M16 13 C 11 9, 11 5, 12 4 H 20 C 21 5, 21 9, 16 13 Z"
      />
      <path
        className="g-line"
        d="M16 13 C 11 9, 11 5, 12 4 H 20 C 21 5, 21 9, 16 13 Z"
      />
      <path
        className="g-line"
        d="M8 16 C 4 14, 4 11, 5 10 L 10 11 C 11 13, 10 15, 8 16 Z M24 16 C 28 14, 28 11, 27 10 L 22 11 C 21 13, 22 15, 24 16 Z"
      />
    </>
  ),
  door: (
    <>
      <path className="g-fill" d="M8 28 V 6 H 24 V 28 Z" />
      <path
        className="g-line"
        d="M8 28 V 6 H 24 V 28 M5 28 H 27 M12 10 H 20 M12 10 V 28"
      />
    </>
  ),
  house: (
    <>
      <path className="g-fill" d="M6 8 H 26 V 26 H 6 Z" />
      <path
        className="g-line"
        d="M6 8 H 26 V 26 H 6 Z M6 17 H 14 M14 26 V 17"
      />
    </>
  ),
  eye: (
    <>
      <path
        className="g-fill"
        d="M4 15 C 10 7, 22 7, 28 15 C 22 23, 10 23, 4 15 Z"
      />
      <path
        className="g-line"
        d="M4 15 C 10 7, 22 7, 28 15 C 22 23, 10 23, 4 15 Z"
      />
      <circle className="g-dot" cx="16" cy="15" r="3.5" />
      <path
        className="g-line"
        d="M20 22 C 21 25, 24 27, 27 26 M12 22 C 12 25, 10 27, 8 28 M8 10 C 6 8, 4 8, 3 9"
      />
    </>
  ),
  scales: (
    <>
      <path
        className="g-line"
        d="M16 6 V 26 M8 26 H 24 M5 11 H 27 M9 11 L 5 19 H 13 Z M23 11 L 19 19 H 27 Z"
      />
      <circle className="g-dot" cx="16" cy="11" r="1.6" />
    </>
  ),
  star: (
    <>
      <path
        className="g-fill"
        d="M16 4 L 18.8 12.2 L 27.4 12.4 L 20.6 17.6 L 23 26 L 16 21 L 9 26 L 11.4 17.6 L 4.6 12.4 L 13.2 12.2 Z"
      />
      <path
        className="g-line"
        d="M16 4 L 18.8 12.2 L 27.4 12.4 L 20.6 17.6 L 23 26 L 16 21 L 9 26 L 11.4 17.6 L 4.6 12.4 L 13.2 12.2 Z"
      />
    </>
  ),
  water: (
    <>
      <path
        className="g-line"
        d="M4 11 L 8 7 L 12 11 L 16 7 L 20 11 L 24 7 L 28 11 M4 18 L 8 14 L 12 18 L 16 14 L 20 18 L 24 14 L 28 18 M4 25 L 8 21 L 12 25 L 16 21 L 20 25 L 24 21 L 28 25"
      />
    </>
  ),
  sun: (
    <>
      <circle className="g-fill" cx="16" cy="16" r="9" />
      <circle className="g-line" cx="16" cy="16" r="9" />
      <circle className="g-dot" cx="16" cy="16" r="2" />
    </>
  ),
  lotus: (
    <>
      <path
        className="g-fill"
        d="M16 26 C 9 22, 7 15, 8 8 C 12 11, 15 17, 16 26 Z M16 26 C 23 22, 25 15, 24 8 C 20 11, 17 17, 16 26 Z"
      />
      <path
        className="g-line"
        d="M16 26 C 9 22, 7 15, 8 8 C 12 11, 15 17, 16 26 Z M16 26 C 23 22, 25 15, 24 8 C 20 11, 17 17, 16 26 Z M16 26 C 13 19, 13 12, 16 5 C 19 12, 19 19, 16 26 Z M16 26 V 30"
      />
    </>
  ),
  ibis: (
    <>
      <path
        className="g-fill"
        d="M9 22 C 7 16, 12 11, 18 12 C 22 13, 24 16, 23 20 C 22 24, 15 26, 9 22 Z"
      />
      <path
        className="g-line"
        d="M9 22 C 7 16, 12 11, 18 12 C 22 13, 24 16, 23 20 C 22 24, 15 26, 9 22 Z M18 12 C 20 9, 23 7, 26 6 C 25 9, 23 10, 22 13 M26 6 C 28 7, 29 9, 28 11 M12 24 L 11 29 M17 25 L 17 29 M5 29 H 27"
      />
    </>
  ),
  pyramid: (
    <>
      <path className="g-fill" d="M16 5 L 29 27 H 3 Z" />
      <path
        className="g-line"
        d="M16 5 L 29 27 H 3 Z M16 5 L 20 27 M9 19 H 23"
      />
    </>
  ),
};

export function Glyph({
  name,
  className,
  title,
}: {
  name: GlyphKey;
  className?: string;
  title?: string;
}) {
  return (
    <svg
      className={`glyph ${className ?? ""}`}
      viewBox="0 0 32 32"
      role={title ? "img" : undefined}
      aria-hidden={title ? undefined : true}
      aria-label={title}
    >
      {paths[name]}
    </svg>
  );
}
