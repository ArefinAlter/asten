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
    note: "Built and working in the current code.",
    items: [
      { title: "The desk", text: "Binder, rich-text page, comments, find, word counts, letter page view, track-change marks, named snapshots, an append-only history you can verify, blame, and drafts you can merge." },
      { title: "Canon", text: "Facts with status, context, story-time span, confidence, and a source anchored to the sentence that settled them. Entities with aliases and an introduction point." },
      { title: "Reading the page", text: "Deterministic perception, no model: names and aliases across scripts, hair and eyes, height and age, where someone is, alive or dead, who is married to whom. Dialogue, belief, memory, dream, and frame stories are separate contexts, so a lie is not a contradiction." },
      { title: "Seventeen canon rules", text: "Attributes (CANON-E013–E015), lifecycle (E101–E103), two clocks and a temporal network (E201–E203), travel and place (E301–E302), knowledge (E401), relations and genealogy (E501–E503), author invariants (E601–E602), plus name hygiene (NAME-W001–W004) and setup ledgers (PLOT-I001–I002)." },
      { title: "The Scales", text: "Every mark explains itself in a numbered chain, names the most likely slip, and offers repairs you own: add an event, mark the scene a memory, retire a fact, keep your sentence. Suppressions know when their evidence changed. Bisect finds the edit that raised a mark." },
      { title: "Two clocks and calendars", text: "Story time as day counts, Gregorian dates (“winter of 1890”), or a fictional calendar with your own months, plus relative placements (“three days after the siege”). Flashbacks are a choice: Asten argues only inside a run you declared continuous." },
      { title: "Method packs", text: "Scribe, Lifecycle, Clocks, Relations, Knowledge, Atlas, Invariants, and the optional Diction watch. Suppressions and baselines stored with the project." },
      { title: "World", text: "Wall, atlas, and dossier. Spark, sketched, settled. Routes with travel times feed the atlas checks." },
      { title: "Compile and import", text: "Markdown, plain text, Fountain, HTML, .docx, EPUB 3, and Final Draft (.fdx); import of all but EPUB. PDF by printing HTML." },
      { title: "Guests", text: "Word task pane and Google Docs sidebar, sideloaded. They pair once with a six-digit code and hold a token you can revoke." },
      { title: "Local-first storage", text: "SQLite on your disk, a desktop shell that embeds the local API, no account, no sync, no telemetry. The API answers only to paired clients." },
      { title: "API contracts", text: "TypeScript contracts generated from the Rust types and checked for drift, and an OpenAPI 3.1 document served by the local API." },
      { title: "Pre-alpha 0.0.1a", text: "The first public release: unsigned desktop builds for Windows x64 and Linux x64 that run the engine on your machine, and a hosted copy in the browser that gives each visitor a private sandbox for 24 hours." },
    ],
  },
  {
    id: "next",
    title: "Next",
    note: "Being worked on now. No dates.",
    items: [
      { title: "Installers for the other desktop platforms", text: "Release 0.0.1a ships Windows x64 and Linux x64. Windows on ARM, macOS on Apple silicon and Intel in one universal app, and Linux on aarch64 are configured; their installers still need building and testing on hardware. macOS comes later." },
      { title: "Signed installers", text: "Code-signed installers for Windows, macOS and Linux, so the system no longer warns before the first run. The 0.0.1a builds are unsigned." },
      { title: "Phones and tablets", text: "The one-column layout, with the binder as a drawer and the Scales in a bottom sheet, is built. Installing it from a phone’s browser requires trusted HTTPS to your own computer. The Android and iOS apps from the same shell still need signing and device testing." },
    ],
  },
  {
    id: "later",
    title: "Later",
    note: "On the plan, behind the work above.",
    items: [
      { title: "Trained perception, still local", text: "Small classifiers for mentions, attributes, time phrases, and who is speaking, for languages and sentences the rules miss. They only propose, nothing becomes canon without you, and none of them generates text. The compiler that judges contradictions stays rule-based." },
      { title: "More views", text: "Graph, maps with scale, and boards." },
      { title: "More packs", text: "Mystery clue ledgers, screenplay sequences, and other methods, always optional, with a runtime for third-party packs." },
      { title: "Collaboration", text: "A second author on the same project without giving up offline work. This is still a design question, not a feature." },
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
