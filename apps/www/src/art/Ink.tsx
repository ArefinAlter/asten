interface InkProps {
  className?: string;
}

export function BrushWash({ className }: InkProps) {
  return (
    <svg
      className={className}
      viewBox="0 0 720 220"
      fill="none"
      aria-hidden="true"
    >
      <path
        className="ink-fill"
        d="M38 128c42-46 96-78 162-84 58-6 94 18 128 44 28 22 71 18 98-6 32-28 86-38 128-18 48 22 86 18 118-16 8 22-6 54-38 68-46 20-108 8-146-18-24-16-62-12-86 10-36 34-92 48-146 36-62-14-98 8-148 28-42 16-88 8-70-44z"
        fill="currentColor"
      />
      <path
        className="ink-draw"
        pathLength="1"
        d="M24 142c70-62 150-96 246-88 72 6 118 44 176 28 54-14 108-8 164 24"
        stroke="currentColor"
        strokeWidth="3.2"
        strokeLinecap="round"
      />
    </svg>
  );
}

export function BrushUnderline({ className }: InkProps) {
  return (
    <svg
      className={className}
      viewBox="0 0 180 18"
      fill="none"
      aria-hidden="true"
    >
      <path
        className="ink-draw"
        pathLength="1"
        d="M3 11c18-6 36 4 54 1 22-4 28 6 48 2 16-3 38 4 70-3"
        stroke="currentColor"
        strokeWidth="2.4"
        strokeLinecap="round"
      />
    </svg>
  );
}

export function MarginRuleArt({ className }: InkProps) {
  return (
    <svg
      className={className}
      viewBox="0 0 40 280"
      fill="none"
      aria-hidden="true"
    >
      <path
        className="ink-draw"
        pathLength="1"
        d="M18 8c-2 40 4 80-1 122 3 42-4 78 2 138"
        stroke="currentColor"
        strokeWidth="1.8"
        strokeLinecap="round"
      />
      <path
        className="ink-draw delay"
        pathLength="1"
        d="M26 22c1 36-5 70 0 108"
        stroke="currentColor"
        strokeWidth="0.8"
        opacity="0.45"
      />
    </svg>
  );
}

export function HarborStroke({ className }: InkProps) {
  return (
    <svg
      className={className}
      viewBox="0 0 480 200"
      fill="none"
      aria-hidden="true"
    >
      <path
        className="ink-draw"
        pathLength="1"
        d="M12 118c48-28 92-8 138-22 46-14 70 18 118 8 52-10 88-36 140-18 36 12 52 8 64-10"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
      />
      <path
        className="ink-draw delay"
        pathLength="1"
        d="M40 150c70 10 140 16 210 4 64-10 120 8 190 2"
        stroke="currentColor"
        strokeWidth="1.2"
        strokeLinecap="round"
      />
      <polygon className="ink-fill" points="292,92 308,54 326,92" />
      <polygon className="ink-fill" points="318,98 336,62 354,98" />
      <polygon className="ink-fill" points="348,94 368,58 388,94" />
    </svg>
  );
}

export function IcebergMark({ className }: InkProps) {
  return (
    <svg
      className={className}
      viewBox="0 0 120 160"
      fill="none"
      aria-hidden="true"
    >
      <path
        className="ink-draw"
        pathLength="1"
        d="M8 118h104"
        stroke="currentColor"
        strokeWidth="1"
      />
      <path
        className="iceberg-rise"
        d="M60 18 L108 118 H12 Z"
        fill="currentColor"
      />
    </svg>
  );
}

export function WaxSeal({ className, label }: InkProps & { label: string }) {
  return (
    <span className={`wax ${className ?? ""}`}>
      <svg viewBox="0 0 72 72" aria-hidden="true">
        <circle cx="36" cy="36" r="30" fill="currentColor" opacity="0.12" />
        <circle
          cx="36"
          cy="36"
          r="26"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.2"
          strokeDasharray="3 4"
        />
        <path
          className="ink-draw"
          pathLength="1"
          d="M22 38c6-10 14-12 22-6 6 5 12 4 16-2"
          stroke="currentColor"
          strokeWidth="1.4"
          fill="none"
        />
      </svg>
      {label}
    </span>
  );
}

export function Wordmark({ large = false }: { large?: boolean }) {
  return (
    <span className={`wordmark${large ? " large" : ""}`}>
      <span className="spine" aria-hidden="true" />
      Asten
    </span>
  );
}

export function ReedPen({ className }: InkProps) {
  return (
    <svg
      className={className}
      viewBox="0 0 280 48"
      fill="none"
      aria-hidden="true"
    >
      <path
        className="ink-draw"
        pathLength="1"
        d="M8 30c36-18 72-22 118-8 28 8 54 6 86-8 18-8 40-10 60-2"
        stroke="currentColor"
        strokeWidth="2.4"
        strokeLinecap="round"
      />
      <path
        d="M248 12l18 14-28 8 10-22z"
        fill="currentColor"
        opacity="0.7"
      />
    </svg>
  );
}

export function IbisMark({ className }: InkProps) {
  return (
    <svg
      className={className}
      viewBox="0 0 120 90"
      fill="none"
      aria-hidden="true"
    >
      <path
        className="ink-draw"
        pathLength="1"
        d="M22 58c8-22 28-38 48-34 14 2 22 14 18 26-6 18-28 22-40 12-4 10 2 22 16 26"
        stroke="currentColor"
        strokeWidth="2.2"
        strokeLinecap="round"
      />
      <path
        className="ink-draw delay"
        pathLength="1"
        d="M68 28c18-10 34-8 46 6"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
      />
    </svg>
  );
}
