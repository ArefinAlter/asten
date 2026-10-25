/**
 * The connector catalogue: what Asten reaches beyond its own window, with an
 * honest status from docs/ROADMAP.md. "Built" ships in the repository today;
 * "Planned" is a design, not a feature. Nothing here is fetched.
 */

export interface Connector {
  id: string;
  name: string;
  category:
    "Add-in" | "Import" | "Export" | "Research" | "Storage" | "Versions";
  status: "Built" | "Planned";
  auth: string;
  hosts: string[];
  summary: string;
  detail: string;
}

export const CONNECTORS: Connector[] = [
  {
    id: "word",
    name: "Microsoft Word add-in",
    category: "Add-in",
    status: "Built",
    auth: "Pairing code → guest token (revocable in Guests)",
    hosts: ["127.0.0.1 (your Asten API)"],
    summary:
      "The Scales beside a Word document; file a selection as a new scene.",
    detail:
      "apps/addin-word. A guest: it lists projects, shows marks and files a selection as a scene. It never edits the manuscript and never manages pairing (D14). AppSource listing is later.",
  },
  {
    id: "docs",
    name: "Google Docs add-on",
    category: "Add-in",
    status: "Built",
    auth: "Pairing code → guest token (revocable in Guests)",
    hosts: ["127.0.0.1 (your Asten API)"],
    summary: "The same guest, in a Docs sidebar.",
    detail:
      "apps/addin-docs. Same rules as the Word guest (D14). Marketplace listing is later.",
  },
  {
    id: "docx",
    name: "Word documents (.docx)",
    category: "Import",
    status: "Built",
    auth: "None: a file you choose",
    hosts: [],
    summary: "Import a .docx as a page; compile the manuscript to .docx.",
    detail:
      "Runs in the engine. In this browser release, .docx import and export need the desktop app.",
  },
  {
    id: "fdx",
    name: "Final Draft (.fdx)",
    category: "Import",
    status: "Built",
    auth: "None: a file you choose",
    hosts: [],
    summary: "Import and export screenplays; one screenplay classifier (D25).",
    detail: "Deterministic writer and reader on the domain crate's classifier.",
  },
  {
    id: "epub",
    name: "EPUB 3 export",
    category: "Export",
    status: "Built",
    auth: "None",
    hosts: [],
    summary: "An e-book in binder order (D25).",
    detail: "Written by the engine's compiler; EPUB is export only.",
  },
  {
    id: "text",
    name: "Markdown, HTML, Fountain, plain text",
    category: "Export",
    status: "Built",
    auth: "None",
    hosts: [],
    summary: "Compile and import open text formats.",
    detail: "Leave with your words at any time (D8).",
  },
  {
    id: "scrivener",
    name: "Scrivener project import",
    category: "Import",
    status: "Planned",
    auth: "None: a .scriv folder you choose",
    hosts: [],
    summary: "Binder, synopses and notes into an Asten project.",
    detail:
      "Not built. Today: compile from Scrivener to .docx or Markdown and import that.",
  },
  {
    id: "zotero",
    name: "Zotero",
    category: "Research",
    status: "Planned",
    auth: "Zotero's local connector (no cloud key)",
    hosts: ["127.0.0.1:23119"],
    summary:
      "Sources and citations from your Zotero library into source notes.",
    detail: "Not built. Would read the local Zotero app only.",
  },
  {
    id: "dropbox",
    name: "Dropbox folder",
    category: "Storage",
    status: "Planned",
    auth: "None: a folder on this machine",
    hosts: [],
    summary: "Keep the project file in a synced folder you already use.",
    detail:
      "Not built. Asten has no sync service (D8); this would only let you choose where the file lives.",
  },
  {
    id: "icloud",
    name: "iCloud Drive folder",
    category: "Storage",
    status: "Planned",
    auth: "None: a folder on this machine",
    hosts: [],
    summary: "As Dropbox: a place for the file, not a sync service.",
    detail: "Not built.",
  },
  {
    id: "git",
    name: "Git remotes",
    category: "Versions",
    status: "Planned",
    auth: "Your Git credentials, held by Git",
    hosts: ["A remote you name"],
    summary: "Push compiled Markdown and versions to a repository.",
    detail:
      "Not built. Versions today are snapshots in the hash-chained history (Source Control).",
  },
];
