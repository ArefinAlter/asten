export function Arrow({ className = "arrow" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" aria-hidden="true" fill="none">
      <path d="M2 8h11M9 3.5 13.5 8 9 12.5" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

export function Chevron({ className = "chev" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 10 10" aria-hidden="true" fill="none">
      <path d="M2 3.5 5 6.5 8 3.5" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

/** The lotus used as the FAQ disclosure marker. */
export function Bloom() {
  return (
    <svg className="bloom" viewBox="0 0 26 26" aria-hidden="true" fill="none">
      <g stroke="currentColor" strokeWidth="1.6" strokeLinejoin="round">
        <path className="petal petal-l" d="M13 20 C 8 17, 6 12, 7 7 C 10 10, 12 14, 13 20 Z" />
        <path className="petal petal-r" d="M13 20 C 18 17, 20 12, 19 7 C 16 10, 14 14, 13 20 Z" />
        <path d="M13 20 C 11 14, 11 9, 13 4 C 15 9, 15 14, 13 20 Z" fill="currentColor" fillOpacity="0.14" />
        <path d="M13 20 V 24" strokeLinecap="round" />
      </g>
    </svg>
  );
}

export function External({ className = "arrow" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" aria-hidden="true" fill="none">
      <path d="M6 3H3v10h10v-3M9 3h4v4M13 3 7 9" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}
