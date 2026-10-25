/**
 * Keybindings: the defaults, chords written the way people read them
 * ("Ctrl+Alt+B"), matching a keyboard event, and conflict detection for the
 * Keybindings settings page. Pure and tested (`keys.test.ts`).
 */

export interface Binding {
  command: string;
  title: string;
  keys: string;
  /** When the binding applies; "notEditing" yields to the page (Ctrl+B is Bold there). */
  when?: "always" | "notEditing";
}

export const DEFAULT_BINDINGS: Binding[] = [
  { command: "palette.open", title: "Command palette", keys: "Ctrl+K" },
  {
    command: "palette.commands",
    title: "Show all commands",
    keys: "Ctrl+Shift+P",
  },
  { command: "switcher.open", title: "Quick switcher", keys: "Ctrl+O" },
  {
    command: "layout.sidebar",
    title: "Toggle side bar",
    keys: "Ctrl+B",
    when: "notEditing",
  },
  { command: "layout.panel", title: "Toggle bottom panel", keys: "Ctrl+J" },
  {
    command: "layout.secondary",
    title: "Toggle assistant panel",
    keys: "Ctrl+Alt+B",
  },
  { command: "layout.ribbon", title: "Collapse the ribbon", keys: "Ctrl+F1" },
  {
    command: "layout.maximizePanel",
    title: "Maximise the bottom panel",
    keys: "Ctrl+Shift+J",
  },
  { command: "panel.problems", title: "Show problems", keys: "Ctrl+Shift+M" },
  { command: "panel.terminal", title: "Show console", keys: "Ctrl+`" },
  {
    command: "pane.manuscript",
    title: "Show manuscript",
    keys: "Ctrl+Shift+E",
  },
  { command: "pane.knowledge", title: "Show knowledge", keys: "Ctrl+Shift+K" },
  { command: "pane.search", title: "Search the project", keys: "Ctrl+Shift+F" },
  { command: "pane.scm", title: "Show source control", keys: "Ctrl+Shift+G" },
  { command: "view.graph", title: "Open graph view", keys: "Ctrl+G" },
  { command: "app.settings", title: "Open settings", keys: "Ctrl+," },
  { command: "app.backstage", title: "File (Backstage)", keys: "Alt+F" },
  { command: "app.runChecks", title: "Run checks", keys: "F7" },
  { command: "app.nextMark", title: "Next mark", keys: "F8" },
  { command: "app.prevMark", title: "Previous mark", keys: "Shift+F8" },
  { command: "app.save", title: "Save a version (commit)", keys: "Ctrl+S" },
  { command: "view.mode.focus", title: "Focus mode", keys: "Ctrl+Shift+Enter" },
  { command: "view.zoomIn", title: "Zoom in", keys: "Ctrl+=" },
  { command: "view.zoomOut", title: "Zoom out", keys: "Ctrl+-" },
  { command: "view.zoom100", title: "Reset zoom", keys: "Ctrl+0" },
  { command: "app.find", title: "Find in the page", keys: "Ctrl+F" },
];

const MOD_ORDER = ["Ctrl", "Alt", "Shift", "Meta"] as const;

/** Normalise a written chord: modifier order, key case, aliases. */
export function normalizeChord(chord: string): string {
  const parts = chord
    .split("+")
    .map((part) => part.trim())
    .filter(Boolean);
  // "Ctrl++" splits into ["Ctrl", ""]; keep a literal plus.
  if (/\+\+$/.test(chord)) parts.push("+");
  const mods = new Set<string>();
  let key = "";
  for (const part of parts) {
    const lower = part.toLowerCase();
    if (
      lower === "ctrl" ||
      lower === "control" ||
      lower === "cmd" ||
      lower === "mod"
    )
      mods.add("Ctrl");
    else if (lower === "alt" || lower === "option") mods.add("Alt");
    else if (lower === "shift") mods.add("Shift");
    else if (lower === "meta" || lower === "win" || lower === "super")
      mods.add("Meta");
    else
      key =
        part.length === 1
          ? part.toUpperCase()
          : part[0].toUpperCase() + part.slice(1);
  }
  return [...MOD_ORDER.filter((m) => mods.has(m)), key]
    .filter(Boolean)
    .join("+");
}

export interface KeyLike {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
}

const KEY_NAMES: Record<string, string> = {
  " ": "Space",
  ArrowUp: "Up",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
  Escape: "Esc",
};

/** The chord a keyboard event is, or null for a bare modifier. */
export function chordOf(event: KeyLike): string | null {
  if (["Control", "Shift", "Alt", "Meta"].includes(event.key)) return null;
  let key = KEY_NAMES[event.key] ?? event.key;
  // Shifted punctuation reports the shifted glyph; bindings name the base key.
  if (key === "+") key = "=";
  if (key === "_") key = "-";
  if (key === "~") key = "`";
  if (key === "<") key = ",";
  const parts = [];
  if (event.ctrlKey || event.metaKey) parts.push("Ctrl");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");
  parts.push(key.length === 1 ? key.toUpperCase() : key);
  return parts.join("+");
}

/** The effective bindings: defaults with the person's overrides applied. */
export function effectiveBindings(
  overrides: Record<string, string>,
): Binding[] {
  return DEFAULT_BINDINGS.map((binding) =>
    overrides[binding.command] !== undefined
      ? { ...binding, keys: overrides[binding.command] }
      : binding,
  );
}

/** Chords bound to more than one command: chord → commands. */
export function conflicts(bindings: Binding[]): Map<string, string[]> {
  const byChord = new Map<string, string[]>();
  for (const binding of bindings) {
    if (!binding.keys) continue;
    const chord = normalizeChord(binding.keys);
    byChord.set(chord, [...(byChord.get(chord) ?? []), binding.command]);
  }
  return new Map([...byChord].filter(([, commands]) => commands.length > 1));
}

/** Chords the page's editor keeps for itself while the caret is in it. */
export const EDITOR_CHORDS = new Set([
  "Ctrl+B",
  "Ctrl+I",
  "Ctrl+U",
  "Ctrl+Z",
  "Ctrl+Y",
  "Ctrl+Shift+Z",
  "Ctrl+A",
]);

export function matchBinding(
  bindings: Binding[],
  event: KeyLike,
  editing: boolean,
): Binding | undefined {
  const chord = chordOf(event);
  if (!chord) return undefined;
  return bindings.find(
    (binding) =>
      binding.keys &&
      normalizeChord(binding.keys) === chord &&
      !(editing && (binding.when === "notEditing" || EDITOR_CHORDS.has(chord))),
  );
}
