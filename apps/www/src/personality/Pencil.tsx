import { useEffect, useId, useMemo, useRef } from "react";
import type { Stroke } from "./strokes";
import { clamp, noise1 } from "./random";

/** Handwriting set in the margin, revealed line by line as if written. Coordinates are in the overlay's own space. */
export interface Note {
  id: string;
  x: number;
  y: number;
  lines: string[];
  delay: number;
  /** Seconds per line. */
  perLine: number;
  /** Font size in px; the line height is derived. */
  size: number;
  rotate?: number;
}

export type PencilMode = "idle" | "draw" | "erase" | "still";

interface Props {
  strokes: Stroke[];
  notes?: Note[];
  width: number;
  height: number;
  mode: PencilMode;
  /** Changes whenever the marks should be drawn afresh (new text, a second pass). */
  epoch: number;
  onErased?: () => void;
  className?: string;
}

/**
 * The red pencil. Every stroke is a filled, variable-width brush outline revealed by a mask whose centre
 * line is drawn on with stroke-dashoffset; an SVG turbulence filter roughens the edge and breaks the fill
 * into graphite grain. Pace is irregular (a noise-driven velocity under a spring-like ease-out), so no two
 * strokes travel alike. Nothing runs when nothing is moving.
 */
export function Pencil({ strokes, notes = [], width, height, mode, epoch, onErased, className }: Props) {
  const uid = useId().replace(/:/g, "");
  const centerRefs = useRef<(SVGPathElement | null)[]>([]);
  const noteRefs = useRef<(SVGRectElement | null)[][]>([]);
  const progress = useRef<number[]>([]);
  const noteProgress = useRef<number[][]>([]);
  const frame = useRef(0);

  const pace = useMemo(
    () =>
      strokes.map((_s, i) => {
        // Irregular velocity: a handful of noise-driven segments, integrated and normalised.
        const seed = 11 + i * 7 + epoch * 3;
        const k = 28;
        const cum = [0];
        for (let j = 0; j < k; j++) cum.push(cum[j] + 0.35 + 0.65 * (noise1(j * 0.55 + seed, seed) * 0.5 + 0.5));
        const total = cum[k];
        return (u: number) => {
          const x = clamp(u) * k;
          const j = Math.min(k - 1, Math.floor(x));
          return (cum[j] + (cum[j + 1] - cum[j]) * (x - j)) / total;
        };
      }),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [strokes, epoch],
  );

  useEffect(() => {
    const set = (i: number, p: number) => {
      progress.current[i] = p;
      centerRefs.current[i]?.setAttribute("stroke-dashoffset", String(1 - p));
    };
    const setNote = (n: number, l: number, p: number) => {
      (noteProgress.current[n] ??= [])[l] = p;
      const el = noteRefs.current[n]?.[l];
      const base = Number(el?.dataset.w ?? 0);
      if (el) el.setAttribute("width", String(base * p));
    };
    cancelAnimationFrame(frame.current);

    if (mode === "still" || mode === "idle") {
      const v = mode === "still" ? 1 : 0;
      strokes.forEach((_, i) => set(i, v));
      notes.forEach((n, ni) => n.lines.forEach((_, li) => setNote(ni, li, v)));
      return;
    }

    const springOut = (u: number) => (1 - Math.exp(-6.5 * u) * (1 + 6.5 * u)) / (1 - Math.exp(-6.5) * 7.5);

    if (mode === "erase") {
      const from = strokes.map((_, i) => progress.current[i] ?? 1);
      const noteFrom = notes.map((n, ni) => n.lines.map((_, li) => noteProgress.current[ni]?.[li] ?? 1));
      const t0 = performance.now();
      const dur = 340;
      const tick = (now: number) => {
        const u = clamp((now - t0) / dur);
        // rubbing out: quick, a little ragged
        strokes.forEach((_, i) => set(i, from[i] * (1 - clamp(u * 1.25 - i * 0.04) ** 1.6)));
        notes.forEach((n, ni) => n.lines.forEach((_, li) => setNote(ni, li, noteFrom[ni][li] * (1 - clamp(u * 1.3 - li * 0.05) ** 1.4))));
        if (u < 1) frame.current = requestAnimationFrame(tick);
        else onErased?.();
      };
      frame.current = requestAnimationFrame(tick);
      return () => cancelAnimationFrame(frame.current);
    }

    strokes.forEach((_, i) => set(i, 0));
    notes.forEach((n, ni) => n.lines.forEach((_, li) => setNote(ni, li, 0)));
    const t0 = performance.now();
    const tick = (now: number) => {
      const t = (now - t0) / 1000;
      let live = false;
      strokes.forEach((s, i) => {
        const u = clamp((t - s.delay) / s.duration);
        set(i, pace[i](springOut(u)));
        if (u < 1) live = true;
      });
      notes.forEach((n, ni) =>
        n.lines.forEach((_, li) => {
          const u = clamp((t - n.delay - li * n.perLine * 0.92) / n.perLine);
          setNote(ni, li, springOut(u) * 0.985 + (u >= 1 ? 0.015 : 0));
          if (u < 1) live = true;
        }),
      );
      if (live) frame.current = requestAnimationFrame(tick);
    };
    frame.current = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame.current);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [mode, epoch, strokes, notes, pace]);

  return (
    <svg className={`pencil${className ? ` ${className}` : ""}`} width={width} height={height} viewBox={`0 0 ${width} ${height}`} aria-hidden="true" focusable="false">
      <defs>
        {strokes.map((s, i) => (
          <mask key={s.id} id={`${uid}-m${i}`} maskUnits="userSpaceOnUse" x="-20" y="-20" width={width + 40} height={height + 40}>
            <path
              ref={(el) => {
                centerRefs.current[i] = el;
              }}
              d={s.center}
              pathLength={1}
              fill="none"
              stroke="#fff"
              strokeWidth={14}
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeDasharray={1}
              strokeDashoffset={mode === "still" ? 0 : 1}
            />
          </mask>
        ))}
        {notes.map((n, ni) =>
          n.lines.map((line, li) => (
            <mask key={`${n.id}-${li}`} id={`${uid}-n${ni}-${li}`} maskUnits="userSpaceOnUse" x={n.x - 8} y={n.y - n.size * 1.3 + li * n.size * 1.08} width={Math.ceil(line.length * n.size * 0.5) + 24} height={n.size * 1.7}>
              <rect
                ref={(el) => {
                  (noteRefs.current[ni] ??= [])[li] = el;
                }}
                x={n.x - 8}
                y={n.y - n.size * 1.3 + li * n.size * 1.08}
                width={mode === "still" ? Math.ceil(line.length * n.size * 0.5) + 24 : 0}
                height={n.size * 1.7}
                data-w={Math.ceil(line.length * n.size * 0.5) + 24}
                fill="#fff"
              />
            </mask>
          )),
        )}
      </defs>
      <g filter="url(#asten-pencil)" className="pencil-ink">
        {strokes.map((s, i) => (
          <path key={s.id} d={s.brush} mask={`url(#${uid}-m${i})`} opacity={s.opacity} />
        ))}
        {notes.map((n, ni) => (
          <g key={n.id} transform={n.rotate ? `rotate(${n.rotate} ${n.x} ${n.y})` : undefined}>
            {n.lines.map((line, li) => (
              <text key={li} x={n.x} y={n.y + li * n.size * 1.08} fontSize={n.size} className="pencil-note" mask={`url(#${uid}-n${ni}-${li})`}>
                {line}
              </text>
            ))}
          </g>
        ))}
      </g>
    </svg>
  );
}

/** The one filter every pencil mark shares: a wobbling edge and graphite grain. Rendered once, in the layout. */
export function PencilFilters() {
  return (
    <svg className="pencil-defs" width="0" height="0" aria-hidden="true" focusable="false">
      <filter id="asten-pencil" x="-8%" y="-12%" width="116%" height="124%" colorInterpolationFilters="sRGB">
        <feTurbulence type="fractalNoise" baseFrequency="0.035" numOctaves="2" seed="4" result="warp" />
        <feDisplacementMap in="SourceGraphic" in2="warp" scale="3.2" xChannelSelector="R" yChannelSelector="G" result="wobble" />
        <feTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="2" seed="9" result="grain" />
        <feColorMatrix in="grain" type="matrix" values="0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  2.6 0 0 0 -0.55" result="speck" />
        <feComposite in="wobble" in2="speck" operator="in" />
      </filter>
      <filter id="asten-ink" x="-5%" y="-10%" width="110%" height="120%" colorInterpolationFilters="sRGB">
        <feTurbulence type="fractalNoise" baseFrequency="0.06" numOctaves="2" seed="2" result="warp" />
        <feDisplacementMap in="SourceGraphic" in2="warp" scale="1.6" xChannelSelector="R" yChannelSelector="G" result="bleed" />
        <feGaussianBlur in="bleed" stdDeviation="0.35" />
      </filter>
    </svg>
  );
}
