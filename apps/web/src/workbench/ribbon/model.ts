/**
 * The ribbon's tabs, groups and controls, as data. Labels follow Word and
 * Publisher where the function matches; each control names a command id that
 * `commands.ts` resolves. The File tab opens Backstage instead of a strip.
 */
import type { IconName } from "../icons";

export interface RibbonControl {
  id: string;
  label: string;
  icon?: IconName;
  /** "big" is a tall button with the label under it; the rest are rows. */
  size?: "big" | "small";
  /** A toggle shows a pressed state (from ACTIVE_STATE or app state). */
  toggle?: boolean;
  /** A select: the pref key it edits and its options. */
  select?: { pref: string; options: { value: string; label: string }[] };
  /** Shown in the tooltip after the label. */
  hint?: string;
  keys?: string;
}

export interface RibbonGroup {
  label: string;
  controls: RibbonControl[];
}

export interface RibbonTab {
  id: string;
  label: string;
  groups: RibbonGroup[];
}

export const RIBBON_TABS: RibbonTab[] = [
  {
    id: "home",
    label: "Home",
    groups: [
      {
        label: "Clipboard",
        controls: [
          {
            id: "app.paste",
            label: "Paste",
            icon: "paste",
            size: "big",
            keys: "Ctrl+V",
          },
          { id: "app.cut", label: "Cut", icon: "cut", keys: "Ctrl+X" },
          { id: "app.copy", label: "Copy", icon: "copy", keys: "Ctrl+C" },
          {
            id: "editor.formatPainter",
            label: "Format painter",
            icon: "brush",
          },
        ],
      },
      {
        label: "Font",
        controls: [
          {
            id: "app.pref",
            label: "Display font",
            select: {
              pref: "pageFont",
              options: [
                { value: "literata", label: "Literata" },
                { value: "fraunces", label: "Fraunces" },
                { value: "mono", label: "IBM Plex Mono" },
                { value: "system", label: "System" },
              ],
            },
            hint: "How the page is shown. The manuscript stores structure, not fonts.",
          },
          {
            id: "app.pref",
            label: "Size",
            select: {
              pref: "pageFontSize",
              options: ["15", "16", "17", "18", "19", "20", "22"].map((v) => ({
                value: v,
                label: v,
              })),
            },
          },
          {
            id: "editor.bold",
            label: "Bold",
            icon: "bold",
            toggle: true,
            keys: "Ctrl+B",
          },
          {
            id: "editor.italic",
            label: "Italic",
            icon: "italic",
            toggle: true,
            keys: "Ctrl+I",
          },
          {
            id: "editor.underline",
            label: "Underline",
            icon: "underline",
            toggle: true,
            keys: "Ctrl+U",
          },
          {
            id: "editor.strike",
            label: "Strikethrough",
            icon: "strike",
            toggle: true,
          },
          {
            id: "editor.code",
            label: "Inline code",
            icon: "code",
            toggle: true,
          },
          { id: "editor.subscript", label: "Subscript", icon: "sub" },
          { id: "editor.superscript", label: "Superscript", icon: "sup" },
          {
            id: "editor.clearFormatting",
            label: "Clear formatting",
            icon: "eraser",
          },
        ],
      },
      {
        label: "Paragraph",
        controls: [
          {
            id: "editor.bulletList",
            label: "Bullets",
            icon: "listBullet",
            toggle: true,
          },
          {
            id: "editor.orderedList",
            label: "Numbering",
            icon: "listNumber",
            toggle: true,
          },
          { id: "editor.outdent", label: "Decrease indent", icon: "outdent" },
          { id: "editor.indent", label: "Increase indent", icon: "indent" },
          {
            id: "editor.alignLeft",
            label: "Align left",
            icon: "alignLeft",
            toggle: true,
          },
          {
            id: "editor.alignCenter",
            label: "Center",
            icon: "alignCenter",
            toggle: true,
          },
          {
            id: "editor.alignRight",
            label: "Align right",
            icon: "alignRight",
            toggle: true,
          },
          {
            id: "editor.alignJustify",
            label: "Justify",
            icon: "alignJustify",
            toggle: true,
          },
          {
            id: "editor.blockquote",
            label: "Quote block",
            icon: "quote",
            toggle: true,
          },
        ],
      },
      {
        label: "Styles",
        controls: [
          {
            id: "editor.styleNormal",
            label: "Normal",
            icon: "pilcrow",
            size: "big",
            toggle: true,
          },
          {
            id: "editor.styleHeading1",
            label: "Heading 1",
            icon: "h1",
            size: "big",
            toggle: true,
          },
          {
            id: "editor.styleHeading2",
            label: "Heading 2",
            icon: "h2",
            size: "big",
            toggle: true,
          },
          {
            id: "editor.styleHeading3",
            label: "Heading 3",
            icon: "h3",
            size: "big",
            toggle: true,
          },
          {
            id: "editor.styleQuote",
            label: "Quote",
            icon: "quote",
            size: "big",
          },
        ],
      },
      {
        label: "Editing",
        controls: [
          { id: "app.find", label: "Find", icon: "search", keys: "Ctrl+F" },
          {
            id: "editor.selectAll",
            label: "Select all",
            icon: "select",
            keys: "Ctrl+A",
          },
          { id: "editor.undo", label: "Undo", icon: "undo", keys: "Ctrl+Z" },
          { id: "editor.redo", label: "Redo", icon: "redo", keys: "Ctrl+Y" },
        ],
      },
    ],
  },
  {
    id: "insert",
    label: "Insert",
    groups: [
      {
        label: "Pages",
        controls: [
          {
            id: "editor.sceneBreak",
            label: "Scene break",
            icon: "sceneBreak",
            size: "big",
          },
          {
            id: "app.newScene",
            label: "New scene",
            icon: "filePlus",
            size: "big",
          },
          {
            id: "app.newChapter",
            label: "New chapter",
            icon: "folderPlus",
            size: "big",
          },
        ],
      },
      {
        label: "Notes",
        controls: [
          { id: "app.newNote", label: "Note", icon: "note", size: "big" },
          { id: "app.comment", label: "Comment", icon: "comment", size: "big" },
          {
            id: "editor.footnote",
            label: "Footnote",
            icon: "footnote",
            size: "big",
          },
        ],
      },
      {
        label: "Media",
        controls: [
          { id: "editor.image", label: "Image", icon: "image", size: "big" },
          { id: "editor.table", label: "Table", icon: "table", size: "big" },
          {
            id: "editor.horizontalRule",
            label: "Rule",
            icon: "rule",
            size: "big",
          },
        ],
      },
      {
        label: "Links",
        controls: [
          { id: "app.link", label: "Link", icon: "link", size: "big" },
          {
            id: "app.crossReference",
            label: "Cross-reference",
            icon: "crossRef",
            size: "big",
          },
          { id: "editor.unlink", label: "Remove link", icon: "unlink" },
        ],
      },
      {
        label: "Canon",
        controls: [
          {
            id: "app.insertCharacter",
            label: "Character",
            icon: "person",
            size: "big",
            hint: "Puts a name you pick at the caret. Asten never adds words you did not choose.",
          },
          { id: "app.insertPlace", label: "Place", icon: "pin", size: "big" },
        ],
      },
    ],
  },
  {
    id: "layout",
    label: "Layout",
    groups: [
      {
        label: "Page setup",
        controls: [
          {
            id: "app.pref",
            label: "Size",
            select: {
              pref: "pageSize",
              options: [
                { value: "letter", label: "Letter" },
                { value: "a4", label: "A4" },
                { value: "a5", label: "A5" },
                { value: "trade", label: "Trade 6 × 9 in" },
              ],
            },
          },
          {
            id: "app.pref",
            label: "Margins",
            select: {
              pref: "pageMargins",
              options: [
                { value: "normal", label: "Normal" },
                { value: "narrow", label: "Narrow" },
                { value: "wide", label: "Wide" },
              ],
            },
          },
          {
            id: "app.pref",
            label: "Columns",
            select: {
              pref: "pageColumns",
              options: [
                { value: "1", label: "One" },
                { value: "2", label: "Two" },
              ],
            },
          },
          {
            id: "app.pref",
            label: "Line spacing",
            select: {
              pref: "lineSpacing",
              options: [
                { value: "1.5", label: "1.5" },
                { value: "1.7", label: "1.7" },
                { value: "2", label: "Double" },
              ],
            },
          },
        ],
      },
      {
        label: "Manuscript format",
        controls: [
          {
            id: "app.preset.standard",
            label: "Standard manuscript",
            icon: "manuscript",
            size: "big",
            hint: "Mono 12, double spaced, wide margins",
          },
          {
            id: "app.preset.book",
            label: "Book",
            icon: "book",
            size: "big",
            hint: "Literata, trade size, justified display",
          },
          {
            id: "app.preset.screenplay",
            label: "Screenplay",
            icon: "film",
            size: "big",
            hint: "Mono, Letter, for Fountain and FDX compile",
          },
        ],
      },
    ],
  },
  {
    id: "design",
    label: "Page Design",
    groups: [
      {
        label: "Master pages",
        controls: [
          {
            id: "app.master.chapter",
            label: "Chapter opener",
            icon: "masterChapter",
            size: "big",
            toggle: true,
          },
          {
            id: "app.master.scene",
            label: "Scene page",
            icon: "masterScene",
            size: "big",
            toggle: true,
          },
          {
            id: "app.master.front",
            label: "Front matter",
            icon: "masterFront",
            size: "big",
            toggle: true,
          },
        ],
      },
      {
        label: "Schemes",
        controls: [
          {
            id: "app.scheme.papyrus",
            label: "Papyrus",
            icon: "swatch",
            size: "big",
            toggle: true,
          },
          {
            id: "app.scheme.lapis",
            label: "Lapis",
            icon: "swatch",
            size: "big",
            toggle: true,
          },
          {
            id: "app.scheme.malachite",
            label: "Malachite",
            icon: "swatch",
            size: "big",
            toggle: true,
          },
          {
            id: "app.scheme.ink",
            label: "Plain ink",
            icon: "swatch",
            size: "big",
            toggle: true,
          },
        ],
      },
      {
        label: "Page background",
        controls: [
          { id: "view.ruler", label: "Ruler", icon: "ruler", toggle: true },
        ],
      },
    ],
  },
  {
    id: "references",
    label: "References",
    groups: [
      {
        label: "Canon",
        controls: [
          {
            id: "app.openCanon",
            label: "Canon facts",
            icon: "scales",
            size: "big",
          },
          {
            id: "app.proposals",
            label: "Proposals",
            icon: "inbox",
            size: "big",
          },
          {
            id: "app.factCard",
            label: "Fact card",
            icon: "card",
            size: "big",
            hint: "What is settled about the name at the caret",
          },
        ],
      },
      {
        label: "Citations and sources",
        controls: [
          { id: "app.sources", label: "Sources", icon: "source", size: "big" },
          {
            id: "app.citation",
            label: "Insert citation",
            icon: "quote",
            size: "big",
            hint: "Links the selection to a source note",
          },
        ],
      },
      {
        label: "Cross-references",
        controls: [
          {
            id: "app.crossReference",
            label: "Cross-reference",
            icon: "crossRef",
            size: "big",
          },
        ],
      },
      {
        label: "Index",
        controls: [
          {
            id: "app.openCast",
            label: "Names index",
            icon: "people",
            size: "big",
          },
          {
            id: "app.openTimeline",
            label: "Timeline",
            icon: "clock",
            size: "big",
          },
        ],
      },
    ],
  },
  {
    id: "review",
    label: "Review",
    groups: [
      {
        label: "Continuity",
        controls: [
          {
            id: "app.runChecks",
            label: "Run checks",
            icon: "check",
            size: "big",
            keys: "F7",
          },
          {
            id: "app.prevMark",
            label: "Previous mark",
            icon: "up",
            keys: "Shift+F8",
          },
          { id: "app.nextMark", label: "Next mark", icon: "down", keys: "F8" },
          { id: "panel.problems", label: "Problems", icon: "problems" },
        ],
      },
      {
        label: "Comments",
        controls: [
          {
            id: "app.comment",
            label: "New comment",
            icon: "comment",
            size: "big",
          },
          {
            id: "app.showComments",
            label: "Show comments",
            icon: "comments",
            size: "big",
          },
        ],
      },
      {
        label: "Tracking",
        controls: [
          {
            id: "app.trackChanges",
            label: "Track changes",
            icon: "track",
            size: "big",
            toggle: true,
          },
          { id: "editor.trackInsert", label: "Mark insertion", icon: "ins" },
          { id: "editor.trackDelete", label: "Mark deletion", icon: "del" },
          { id: "editor.acceptMarked", label: "Accept marked", icon: "check" },
        ],
      },
      {
        label: "Compare",
        controls: [
          {
            id: "app.compare",
            label: "Compare versions",
            icon: "compare",
            size: "big",
          },
          {
            id: "app.blame",
            label: "Who wrote it",
            icon: "blame",
            size: "big",
            toggle: true,
          },
        ],
      },
    ],
  },
  {
    id: "view",
    label: "View",
    groups: [
      {
        label: "Views",
        controls: [
          {
            id: "view.mode.read",
            label: "Read mode",
            icon: "read",
            size: "big",
            toggle: true,
          },
          {
            id: "view.mode.print",
            label: "Print layout",
            icon: "page",
            size: "big",
            toggle: true,
          },
          {
            id: "view.mode.draft",
            label: "Draft",
            icon: "draft",
            size: "big",
            toggle: true,
          },
          {
            id: "view.mode.focus",
            label: "Focus",
            icon: "focus",
            size: "big",
            toggle: true,
          },
        ],
      },
      {
        label: "Show",
        controls: [
          { id: "view.ruler", label: "Ruler", icon: "ruler", toggle: true },
          {
            id: "view.navigationPane",
            label: "Navigation pane",
            icon: "sidebar",
            toggle: true,
          },
          {
            id: "panel.toggle",
            label: "Bottom panel",
            icon: "panel",
            toggle: true,
            keys: "Ctrl+J",
          },
          {
            id: "view.secondary",
            label: "Assistant panel",
            icon: "agent",
            toggle: true,
            keys: "Ctrl+Alt+B",
          },
        ],
      },
      {
        label: "Zoom",
        controls: [
          { id: "view.zoomOut", label: "Zoom out", icon: "zoomOut" },
          { id: "view.zoom100", label: "100%", icon: "zoomReset" },
          { id: "view.zoomIn", label: "Zoom in", icon: "zoomIn" },
        ],
      },
      {
        label: "Window",
        controls: [
          {
            id: "view.split",
            label: "Split",
            icon: "split",
            size: "big",
            toggle: true,
          },
          { id: "view.graph", label: "Graph", icon: "graph", size: "big" },
          { id: "app.theme", label: "Theme", icon: "theme", size: "big" },
        ],
      },
    ],
  },
  {
    id: "help",
    label: "Help",
    groups: [
      {
        label: "Help",
        controls: [
          {
            id: "app.shortcuts",
            label: "Keyboard shortcuts",
            icon: "keyboard",
            size: "big",
          },
          {
            id: "app.about",
            label: "About this release",
            icon: "info",
            size: "big",
          },
          {
            id: "app.rules",
            label: "What Asten never does",
            icon: "shield",
            size: "big",
          },
        ],
      },
    ],
  },
];

/** Backstage pages, in Word's order. */
export const BACKSTAGE_PAGES = [
  { id: "home", label: "Home" },
  { id: "new", label: "New" },
  { id: "open", label: "Open" },
  { id: "info", label: "Info" },
  { id: "saveAs", label: "Save as" },
  { id: "export", label: "Export / Compile" },
  { id: "print", label: "Print" },
  { id: "share", label: "Share" },
  { id: "account", label: "Account" },
  { id: "options", label: "Options" },
] as const;

export type BackstagePage = (typeof BACKSTAGE_PAGES)[number]["id"];

/** Every command id the ribbon uses, for tests and the Customize Ribbon page. */
export function ribbonCommandIds(): string[] {
  return RIBBON_TABS.flatMap((tab) =>
    tab.groups.flatMap((group) => group.controls.map((control) => control.id)),
  );
}
