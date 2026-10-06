interface Props {
  className?: string;
  title?: string;
}

/**
 * The Palette mark: a scribe's palette with two ink wells and a reed pen,
 * after the hieroglyph for "scribe" (Gardiner Y3). The left well is black for
 * the writer; the right well is red for the editor.
 */
export function PaletteMark({ className, title }: Props) {
  return (
    <svg
      className={`art palette-mark ${className ?? ""}`}
      viewBox="0 0 32 44"
      role={title ? "img" : undefined}
      aria-hidden={title ? undefined : true}
      aria-label={title}
    >
      <rect className="slate" x="3.5" y="2.5" width="25" height="39" rx="5.5" />
      <circle className="well well-black" cx="11.2" cy="11" r="3.9" />
      <circle className="well well-red" cx="20.8" cy="11" r="3.9" />
      <rect className="slot" x="13.6" y="18.5" width="4.8" height="19.5" rx="2.4" />
      <path className="reed" d="M16 16.5 V32.4" />
      <path className="nib" d="M14.9 32 L17.1 32 L16.4 36.2 L15.6 36.2 Z" />
    </svg>
  );
}
