/**
 * Workbench preferences: appearance, page display, keybindings, the Quick
 * Access Toolbar, the assistant and its providers. Kept by `deskStore`: this
 * webview's storage on the desktop and local builds, the hosting service on
 * the hosted pre-alpha (never the browser's storage there). Provider keys are
 * never part of these preferences: they live in `secrets.ts`, in memory.
 */
import { useSyncExternalStore } from "react";
import type { Access } from "./agent/tools";
import { deskStore, storageAllowed } from "../lib/storage";

export type Theme = "system" | "light" | "dark" | "sepia";

export interface ProviderEntry {
  id: string;
  name: string;
  kind: "local" | "cloud";
  model: string;
  endpoint: string;
  enabled: boolean;
  /** Consent given for this provider: what is sent was shown and accepted. */
  consent: boolean;
}

export interface SkillState {
  enabled: boolean;
  /** The person's edited instructions, if they changed the default. */
  body?: string;
}

export interface Prefs {
  theme: Theme;
  accent: string;
  density: "comfortable" | "compact";
  uiFontSize: number;
  reduceMotion: boolean;
  highContrast: boolean;
  pageFont: string;
  pageFontSize: string;
  pageSize: string;
  pageMargins: string;
  pageColumns: string;
  lineSpacing: string;
  scheme: "papyrus" | "lapis" | "malachite" | "ink";
  master: "chapter" | "scene" | "front";
  language: string;
  spellcheck: boolean;
  showMarksInline: boolean;
  showMargin: boolean;
  confirmDelete: boolean;
  openOn: "last" | "overview";
  snapshotOnCompile: boolean;
  autoSnapshotMinutes: number;
  keybindings: Record<string, string>;
  qat: string[];
  hiddenTabs: string[];
  /** The assistant panel. */
  agentEnabled: boolean;
  agentAccess: Access;
  /** `rules` (Asten's own tools), `auto` (the first ready provider) or a provider id. */
  agentModel: string;
  disabledTools: string[];
  skills: Record<string, SkillState>;
  /** AI providers: the master switch is off by default; local first. */
  aiEnabled: boolean;
  aiLocalFirst: boolean;
  providers: ProviderEntry[];
  addins: Record<string, boolean>;
}

export const DEFAULT_PROVIDERS: ProviderEntry[] = [
  {
    id: "ollama",
    name: "Ollama (this machine)",
    kind: "local",
    model: "llama3.2:3b",
    endpoint: "http://127.0.0.1:11434",
    enabled: false,
    consent: false,
  },
  {
    id: "openai",
    name: "OpenAI",
    kind: "cloud",
    model: "gpt-4.1-mini",
    endpoint: "https://api.openai.com/v1",
    enabled: false,
    consent: false,
  },
  {
    id: "anthropic",
    name: "Anthropic",
    kind: "cloud",
    model: "claude-haiku",
    endpoint: "https://api.anthropic.com",
    enabled: false,
    consent: false,
  },
  {
    id: "google",
    name: "Google Gemini",
    kind: "cloud",
    model: "gemini-flash",
    endpoint: "https://generativelanguage.googleapis.com",
    enabled: false,
    consent: false,
  },
  {
    id: "mistral",
    name: "Mistral",
    kind: "cloud",
    model: "mistral-small",
    endpoint: "https://api.mistral.ai",
    enabled: false,
    consent: false,
  },
  {
    id: "deepseek",
    name: "DeepSeek",
    kind: "cloud",
    model: "deepseek-chat",
    endpoint: "https://api.deepseek.com",
    enabled: false,
    consent: false,
  },
  {
    id: "qwen",
    name: "Qwen (Alibaba Cloud)",
    kind: "cloud",
    model: "qwen-plus",
    endpoint: "https://dashscope-intl.aliyuncs.com",
    enabled: false,
    consent: false,
  },
  {
    id: "kimi",
    name: "Kimi (Moonshot)",
    kind: "cloud",
    model: "moonshot-v1-8k",
    endpoint: "https://api.moonshot.ai",
    enabled: false,
    consent: false,
  },
  {
    id: "glm",
    name: "GLM (Zhipu)",
    kind: "cloud",
    model: "glm-4-flash",
    endpoint: "https://open.bigmodel.cn",
    enabled: false,
    consent: false,
  },
];

export const DEFAULT_PREFS: Prefs = {
  theme: "system",
  accent: "#b0351f",
  density: "comfortable",
  uiFontSize: 13,
  reduceMotion: false,
  highContrast: false,
  pageFont: "literata",
  pageFontSize: "18",
  pageSize: "letter",
  pageMargins: "normal",
  pageColumns: "1",
  lineSpacing: "1.7",
  scheme: "papyrus",
  master: "scene",
  language: "en-GB",
  spellcheck: true,
  showMarksInline: true,
  showMargin: true,
  confirmDelete: true,
  openOn: "last",
  snapshotOnCompile: false,
  autoSnapshotMinutes: 0,
  keybindings: {},
  qat: [
    "app.save",
    "editor.undo",
    "editor.redo",
    "app.runChecks",
    "app.compileQuick",
  ],
  hiddenTabs: [],
  agentEnabled: true,
  agentAccess: "read",
  agentModel: "rules",
  disabledTools: [],
  skills: {},
  aiEnabled: false,
  aiLocalFirst: true,
  providers: DEFAULT_PROVIDERS,
  addins: { word: true, docs: true },
};

const KEY = "asten-workbench:prefs:v1";

export { storageAllowed };

function load(): Prefs {
  const parsed = deskStore.getJson<Partial<Prefs>>(KEY);
  return parsed ? { ...DEFAULT_PREFS, ...parsed } : DEFAULT_PREFS;
}

let state: Prefs = load();

/** Read the preferences again, after the hosted state has arrived. */
export function reloadPrefs() {
  state = load();
  applyPrefs(state);
  listeners.forEach((listener) => listener());
}
const listeners = new Set<() => void>();

export function getPrefs(): Prefs {
  return state;
}

export function setPrefs(
  patch: Partial<Prefs> | ((prefs: Prefs) => Partial<Prefs>),
) {
  const next = typeof patch === "function" ? patch(state) : patch;
  state = { ...state, ...next };
  deskStore.setJson(KEY, state);
  applyPrefs(state);
  listeners.forEach((listener) => listener());
}

export function resetPrefs() {
  setPrefs(DEFAULT_PREFS);
}

export function usePrefs(): Prefs {
  return useSyncExternalStore(
    (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    () => state,
    () => state,
  );
}

export function resolvedTheme(theme: Theme): "light" | "dark" | "sepia" {
  if (theme !== "system") return theme;
  return typeof window !== "undefined" &&
    window.matchMedia?.("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

const PAGE_FONTS: Record<string, string> = {
  literata: "var(--font-text)",
  fraunces: "var(--font-display)",
  mono: "var(--font-mono)",
  system: "var(--wb-font-sans)",
};

/** Write the preferences onto <html> as attributes and custom properties. */
export function applyPrefs(prefs: Prefs) {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  const theme = resolvedTheme(prefs.theme);
  root.dataset.wbTheme = theme;
  // The brand's day and night tokens follow, so the document surface matches.
  root.setAttribute("data-theme", theme === "dark" ? "night" : "day");
  root.dataset.density = prefs.density;
  root.dataset.scheme = prefs.scheme;
  root.dataset.master = prefs.master;
  root.dataset.contrast = prefs.highContrast ? "more" : "normal";
  root.dataset.motion = prefs.reduceMotion ? "reduce" : "normal";
  root.style.setProperty("--wb-accent", prefs.accent);
  root.style.setProperty("--wb-ui-size", `${prefs.uiFontSize}px`);
  root.style.setProperty(
    "--page-font",
    PAGE_FONTS[prefs.pageFont] ?? PAGE_FONTS.literata,
  );
  root.style.setProperty("--page-font-size", `${prefs.pageFontSize}px`);
  root.style.setProperty("--page-line", prefs.lineSpacing);
  root.style.setProperty(
    "--page-width",
    { letter: "8.5in", a4: "210mm", a5: "148mm", trade: "6in" }[
      prefs.pageSize
    ] ?? "8.5in",
  );
  root.style.setProperty(
    "--page-pad",
    { normal: "1in", narrow: "0.5in", wide: "1.5in" }[prefs.pageMargins] ??
      "1in",
  );
  root.style.setProperty("--page-columns", prefs.pageColumns);
}

if (typeof window !== "undefined") {
  window
    .matchMedia?.("(prefers-color-scheme: dark)")
    .addEventListener?.("change", () => applyPrefs(state));
}
