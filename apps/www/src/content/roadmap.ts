export interface RoadmapLane {
  id: "shipped" | "next" | "later" | "never";
  title: string;
  note: string;
  items: { title: string; text: string }[];
}

export const roadmap: RoadmapLane[] = [
  {
    id: "shipped",
    title: "Shipped",
    note: "In early-access builds today.",
    items: [
      { title: "The desk", text: "Binder, rich-text page, comments, find, word counts, letter page view, track-change marks, named snapshots." },
      { title: "Canon", text: "Facts with status, modality, story-time span, and source. Entities with aliases." },
      { title: "Three canon rules", text: "CANON-E013 attribute contradiction, CANON-E101 life after a settled death, CANON-E201 reading order against story time." },
      { title: "Method packs", text: "Scribe, Lifecycle, Clocks, and the optional Diction watch. Suppressions stored with the project." },
      { title: "World", text: "Wall, atlas, and dossier. Spark, sketched, settled." },
      { title: "Compile and import", text: "Markdown, plain text, Fountain, HTML, .docx. PDF by printing HTML." },
      { title: "Guests", text: "Word task pane and Google Docs sidebar, sideloaded, talking to the local API." },
      { title: "Local-first storage", text: "SQLite on your disk, desktop shell, no account, no sync, no telemetry." },
    ],
  },
  {
    id: "next",
    title: "Next",
    note: "Being designed now. No dates until they’re real.",
    items: [
      { title: "Signed desktop installer", text: "A normal download-and-open install, instead of setup notes." },
      { title: "Flashbacks that know they’re flashbacks", text: "Mark a scene as a flashback once and CANON-E201 respects it, the way CANON-E101 already respects flashback notes." },
      { title: "Fact proposals from prose", text: "Deterministic first: names and aliases, simple attributes, dates. Always proposed, never silently canon." },
      { title: "Custom calendars", text: "Eras, months, leap rules, and fuzzy dates (“that winter”) for story time." },
    ],
  },
  {
    id: "later",
    title: "Later",
    note: "On the plan, behind the work above.",
    items: [
      { title: "Optional AI proposals", text: "Local models first; your own provider and key only with explicit consent; complete switch-off; every proposal cites its sentence." },
      { title: "More checks", text: "Travel time between places, who knows what and when, family trees, and your own world rules." },
      { title: "More formats", text: "EPUB and FDX." },
      { title: "More views", text: "Graph, maps with scale, boards, and research with citations." },
      { title: "More packs", text: "Mystery clue ledgers, screenplay sequences, and other methods, always optional." },
      { title: "Collaboration", text: "Shared projects that keep working offline, without making the cloud the only copy." },
    ],
  },
  {
    id: "never",
    title: "Never",
    note: "Promises, not plans.",
    items: [
      { title: "Writing your prose", text: "No generate, continue, rewrite, or autocomplete for your manuscript." },
      { title: "Training on your words", text: "No manuscript is used to train any model, by Verne or anyone Verne works with." },
      { title: "Locking up your files", text: "Your projects stay readable and exportable without a subscription or a server." },
      { title: "Craft doctrine by default", text: "Style advice stays in optional packs, switched off until you ask." },
    ],
  },
];
