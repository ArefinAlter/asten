/**
 * The workbench's own line icons: 16 px, 1.5 px stroke, drawn for Asten (no
 * third-party icon set, no brand marks). Each entry is SVG path data in a
 * 16 × 16 box; text-shaped icons draw a letter.
 */

const P: Record<string, string> = {
  paste: "M5 3h6v2H5zM4 4H3v10h10V4h-1M6 8h4M6 11h4",
  cut: "M4.5 12.5a1.5 1.5 0 1 0 0-.01M11.5 12.5a1.5 1.5 0 1 0 0-.01M5.5 11.5 12 3M10.5 11.5 4 3",
  copy: "M5 5h8v9H5zM3 11V2h7",
  brush: "M10 2l4 4-6 6-4-4zM4 8l-2 6 6-2",
  eraser: "M6 14h8M2.5 10.5l6-6 4 4-5 5H5z",
  outdent: "M2 3h12M7 6.5h7M7 9.5h7M2 13h12M5 6 2.5 8 5 10",
  indent: "M2 3h12M7 6.5h7M7 9.5h7M2 13h12M2.5 6 5 8l-2.5 2",
  alignLeft: "M2 3h12M2 6.5h8M2 10h12M2 13.5h8",
  alignCenter: "M2 3h12M4 6.5h8M2 10h12M4 13.5h8",
  alignRight: "M2 3h12M6 6.5h8M2 10h12M6 13.5h8",
  alignJustify: "M2 3h12M2 6.5h12M2 10h12M2 13.5h12",
  listBullet: "M6 4h8M6 8h8M6 12h8M2.5 4h.5M2.5 8h.5M2.5 12h.5",
  listNumber: "M6 4h8M6 8h8M6 12h8M2 3l1-.5V6M2 9.5c0-.8 2-.8 2 0L2 12h2",
  quote: "M3 5h4v4H5l-1 3M9 5h4v4h-2l-1 3",
  pilcrow: "M7 13V3h6M10 3v10M7 3a3 3 0 0 0 0 6",
  search: "M7 12A5 5 0 1 0 7 2a5 5 0 0 0 0 10zM10.5 10.5 14 14",
  select: "M2 2h3M11 2h3M2 14h3M11 14h3M2 2v3M14 2v3M2 11v3M14 11v3M5 8h6",
  undo: "M5 4 2 7l3 3M2 7h7a4 4 0 0 1 0 8H6",
  redo: "M11 4l3 3-3 3M14 7H7a4 4 0 0 0 0 8h3",
  sceneBreak: "M2 8h3M6.5 8h3M11 8h3M4 4h8M4 12h8",
  filePlus: "M9 2H4v12h8V5zM9 2v3h3M8 8v4M6 10h4",
  folderPlus: "M2 4h4l1.5 1.5H14V13H2zM8 7.5v4M6 9.5h4",
  note: "M3 2h10v12H3zM5.5 5h5M5.5 8h5M5.5 11h3",
  comment: "M2 3h12v8H7l-3 3v-3H2z",
  comments: "M2 2h9v6H6l-2 2V8H2zM13 6h1v6h-2v2l-2-2H7",
  footnote: "M3 3h10M3 6h10M3 9h6M12 10v4M11 11l1-1",
  image: "M2 3h12v10H2zM2 11l4-4 3 3 2-2 3 3M10.5 6.5h.01",
  table: "M2 3h12v10H2zM2 7h12M2 10h12M6 3v10M10 3v10",
  rule: "M2 8h12M2 5h1M2 11h1M13 5h1M13 11h1",
  link: "M7 9a3 3 0 0 0 4 0l2-2a3 3 0 0 0-4-4l-1 1M9 7a3 3 0 0 0-4 0L3 9a3 3 0 0 0 4 4l1-1",
  unlink: "M9 4l1-1a3 3 0 0 1 4 4l-1 1M7 12l-1 1a3 3 0 0 1-4-4l1-1M2 2l12 12",
  crossRef: "M3 3h6v4H3zM7 9h6v4H7zM6 7v2.5H7",
  person:
    "M8 8a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM2.5 14c.5-3 2.7-4.5 5.5-4.5s5 1.5 5.5 4.5",
  people:
    "M6 7a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5zM1.5 13c.4-2.5 2.2-3.8 4.5-3.8s4.1 1.3 4.5 3.8M11 7a2 2 0 1 0 0-4M12 9.5c1.5.3 2.3 1.4 2.5 3",
  pin: "M8 14s4.5-4.2 4.5-7.5a4.5 4.5 0 0 0-9 0C3.5 9.8 8 14 8 14zM8 8a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3z",
  manuscript: "M4 2h8v12H4zM6 5h4M6 7.5h4M6 10h2",
  book: "M2 3c2-1 4-1 6 1 2-2 4-2 6-1v10c-2-1-4-1-6 1-2-2-4-2-6-1zM8 4v10",
  film: "M2 3h12v10H2zM5 3v10M11 3v10M2 6h3M2 10h3M11 6h3M11 10h3",
  masterChapter: "M3 2h10v12H3zM5 5h6M5 7h3M5 10h6M5 12h6",
  masterScene: "M3 2h10v12H3zM5 4h6M5 6h6M5 8h6M5 10h6M5 12h4",
  masterFront: "M3 2h10v12H3zM5.5 6h5M6.5 8h3M7 11h2",
  swatch: "M3 3h4v10H3zM7 5l3-2 3 4-6 6M7 13h6v-3",
  ruler: "M1.5 6h13v4h-13zM4 6v2M7 6v2.5M10 6v2M13 6v2",
  scales:
    "M8 2v12M4 14h8M3 4h10M3 4 1.5 8h3zM13 4l-1.5 4h3zM1.5 8a1.5 1.5 0 0 0 3 0M11.5 8a1.5 1.5 0 0 0 3 0",
  inbox: "M2 9l2-6h8l2 6v4H2zM2 9h3.5l1 2h3l1-2H14",
  card: "M2 4h12v8H2zM4.5 7a1 1 0 1 0 0-.01M7 6.5h5M7 9h4",
  source: "M4 2h6l3 3v9H4zM10 2v3h3M6 8h5M6 10.5h5",
  clock: "M8 14A6 6 0 1 0 8 2a6 6 0 0 0 0 12zM8 4.5V8l2.5 1.5",
  check: "M3 8.5 6.5 12 13 4.5",
  up: "M4 10l4-4 4 4",
  down: "M4 6l4 4 4-4",
  problems: "M8 2l6.5 11.5h-13zM8 6.5V10M8 12h.01",
  track: "M3 13l1-3 7-7 2 2-7 7zM10 4l2 2M9 13h5",
  ins: "M3 13h10M8 3v7M5 7l3 3 3-3",
  del: "M3 8h10M5 4l6 8M11 4l-6 8",
  compare: "M2 3h5v10H2zM9 3h5v10H9zM4 6h1M4 8.5h1M11 6h1M11 8.5h1",
  blame:
    "M5 6a2 2 0 1 0 0-4 2 2 0 0 0 0 4zM1.5 11c.3-2 1.7-3 3.5-3M8 6h6M8 9h6M8 12h6",
  read: "M2 3c2-1 4-1 6 1 2-2 4-2 6-1v10c-2-1-4-1-6 1-2-2-4-2-6-1z",
  page: "M4 2h8v12H4z",
  draft: "M2 4h12M2 7h12M2 10h12M2 13h8",
  focus:
    "M2 5V2h3M11 2h3v3M14 11v3h-3M5 14H2v-3M8 9.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3z",
  sidebar: "M2 3h12v10H2zM6 3v10",
  sidebarRight: "M2 3h12v10H2zM10 3v10",
  panel: "M2 3h12v10H2zM2 9.5h12",
  agent: "M3 3h10v7H8l-3 3v-3H3zM6 6.5h.01M10 6.5h.01",
  zoomOut: "M7 12A5 5 0 1 0 7 2a5 5 0 0 0 0 10zM10.5 10.5 14 14M5 7h4",
  zoomIn: "M7 12A5 5 0 1 0 7 2a5 5 0 0 0 0 10zM10.5 10.5 14 14M5 7h4M7 5v4",
  zoomReset:
    "M7 12A5 5 0 1 0 7 2a5 5 0 0 0 0 10zM10.5 10.5 14 14M5.5 5.5v3M7.5 5.5v3",
  split: "M2 3h12v10H2zM8 3v10",
  graph:
    "M4 4.5a1.5 1.5 0 1 0 0-.01M12 4a1.5 1.5 0 1 0 0-.01M8 12a1.5 1.5 0 1 0 0-.01M5.2 5.3 7.2 10.7M10.8 5.2 8.8 10.6M5.5 4.3h5",
  theme: "M8 14A6 6 0 1 0 8 2zM8 2a6 6 0 0 0 0 12",
  keyboard:
    "M1.5 4h13v8h-13zM4 6.5h.01M6.5 6.5h.01M9 6.5h.01M11.5 6.5h.01M4.5 9.5h7",
  info: "M8 14A6 6 0 1 0 8 2a6 6 0 0 0 0 12zM8 7.5V11M8 5h.01",
  shield:
    "M8 1.5 13.5 4v4c0 3-2.4 5.4-5.5 6.5C4.9 13.4 2.5 11 2.5 8V4zM5.5 8l2 2 3-3.5",
  files: "M5 2h6l3 3v8H5zM11 2v3h3M2 5v9h8",
  knowledge:
    "M8 8a1.5 1.5 0 1 0 0-.01M3 3.5a1.5 1.5 0 1 0 0-.01M13 3.5a1.5 1.5 0 1 0 0-.01M3 12.5a1.5 1.5 0 1 0 0-.01M13 12.5a1.5 1.5 0 1 0 0-.01M4 4.5l3 2.5M12 4.5 9 7M4 11.5 7 9M12 11.5 9 9",
  scm: "M5 4.5a1.5 1.5 0 1 0 0-.01M5 13a1.5 1.5 0 1 0 0-.01M11 6a1.5 1.5 0 1 0 0-.01M5 6v5.5M11 7.5c0 2-3 2.5-5.5 4",
  checks:
    "M8 1.5 13.5 4v4c0 3-2.4 5.4-5.5 6.5C4.9 13.4 2.5 11 2.5 8V4zM5.5 8l2 2 3-3.5",
  packs: "M6 2h4v2.5a1.5 1.5 0 0 0 3 0H14v4h-1.5a1.5 1.5 0 0 0 0 3H14V14H2V2h4",
  settings:
    "M8 10a2 2 0 1 0 0-4 2 2 0 0 0 0 4zM8 1.5v2M8 12.5v2M1.5 8h2M12.5 8h2M3.4 3.4l1.4 1.4M11.2 11.2l1.4 1.4M3.4 12.6l1.4-1.4M11.2 4.8l1.4-1.4",
  x: "M4 4l8 8M12 4l-8 8",
  chevronDown: "M4 6l4 4 4-4",
  chevronRight: "M6 4l4 4-4 4",
  chevronUp: "M4 10l4-4 4 4",
  more: "M3.5 8h.01M8 8h.01M12.5 8h.01",
  plus: "M8 3v10M3 8h10",
  send: "M8 13V3M4 7l4-4 4 4",
  stop: "M4.5 4.5h7v7h-7z",
  lock: "M4 7h8v7H4zM5.5 7V5a2.5 2.5 0 0 1 5 0v2",
  terminal: "M2 3h12v10H2zM4.5 6 6.5 8l-2 2M8 10.5h3",
  output: "M3 3h10v10H3zM5.5 6h5M5.5 8.5h5M5.5 11h3",
  maximize: "M3 10l5-5 5 5",
  restore: "M3 6l5 5 5-5",
  save: "M3 2h8l2 2v10H3zM5 2v4h5V2M5 14V9h6v5",
  folder: "M2 4h4l1.5 1.5H14V13H2z",
  file: "M4 2h5l3 3v9H4zM9 2v3h3",
  sun: "M8 11a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM8 1v1.5M8 13.5V15M1 8h1.5M13.5 8H15M3 3l1 1M12 12l1 1M3 13l1-1M12 4l1-1",
  moon: "M13 9.5A5.5 5.5 0 0 1 6.5 3a5.5 5.5 0 1 0 6.5 6.5z",
  filter: "M2 3h12L9.5 8.5V13l-3 1.5v-6z",
  refresh: "M13 8a5 5 0 1 1-1.5-3.5M13 2v3h-3",
  commit: "M8 10.5a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5zM1.5 8h4M10.5 8h4",
  history: "M2.5 8a5.5 5.5 0 1 0 1.6-3.9M2 2v3h3M8 5v3l2 1.5",
  branch:
    "M5 4.5a1.5 1.5 0 1 0 0-.01M5 13a1.5 1.5 0 1 0 0-.01M11 6a1.5 1.5 0 1 0 0-.01M5 6v5.5M11 7.5c0 2-3 2.5-5.5 4",
  at: "M10.5 8a2.5 2.5 0 1 1-5 0 2.5 2.5 0 0 1 5 0zM10.5 8v1a2 2 0 0 0 4 0V8a6.5 6.5 0 1 0-2.5 5.1",
  plug: "M6 2v3M10 2v3M4 5h8v3a4 4 0 0 1-8 0zM8 12v2.5",
  key: "M5.5 10.5a3 3 0 1 0 0-.01M7.6 8.4 13 3M11 5l1.5 1.5M9.5 6.5 11 8",
  download: "M8 2v8M4.5 6.5 8 10l3.5-3.5M2.5 13.5h11",
  upload: "M8 10V2M4.5 5.5 8 2l3.5 3.5M2.5 13.5h11",
  trash: "M3 4h10M6 4V2.5h4V4M4.5 4l.5 10h6l.5-10",
  edit: "M10.5 2.5l3 3L6 13H3v-3z",
  eye: "M1.5 8S4 3.5 8 3.5 14.5 8 14.5 8 12 12.5 8 12.5 1.5 8 1.5 8zM8 10a2 2 0 1 0 0-4 2 2 0 0 0 0 4z",
  globe:
    "M8 14A6 6 0 1 0 8 2a6 6 0 0 0 0 12zM2 8h12M8 2c2 2 2 10 0 12M8 2c-2 2-2 10 0 12",
  skill: "M3 3h10v10H3zM6 6h4M6 8.5h4M6 11h2",
  tool: "M10 2.5a3 3 0 0 0-3.6 4L2.5 10.4 5.6 13.5l3.9-3.9a3 3 0 0 0 4-3.6l-2 2-2-2z",
  minimize: "M3 8h10",
};

const T: Record<string, string> = {
  bold: "B",
  italic: "I",
  underline: "U",
  strike: "S",
  code: "</>",
  sub: "x₂",
  sup: "x²",
  h1: "H1",
  h2: "H2",
  h3: "H3",
};

export type IconName = keyof typeof P | keyof typeof T;

export function Icon({
  name,
  size = 16,
  className,
}: {
  name: IconName | string;
  size?: number;
  className?: string;
}) {
  const text = T[name];
  if (text)
    return (
      <span
        className={`wb-icon-text wb-icon-${name}${className ? ` ${className}` : ""}`}
        style={{ width: size, height: size, fontSize: Math.round(size * 0.8) }}
        aria-hidden="true"
      >
        {text}
      </span>
    );
  return (
    <svg
      className={`wb-icon${className ? ` ${className}` : ""}`}
      width={size}
      height={size}
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.4"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d={P[name] ?? P.dot ?? "M8 8h.01"} />
    </svg>
  );
}
