import type { GlossaryTerm } from "./types";

/**
 * The Asten glossary. Sorted alphabetically by term. Each `short` is one
 * self-contained sentence that can be quoted on its own.
 */
export const glossary: GlossaryTerm[] = [
  {
    slug: "architect",
    term: "Architect",
    short:
      "An architect, in the comparison George R. R. Martin has made, is a writer who plans a story’s structure in detail before drafting it.",
    long: "An architect may know before chapter 1 that Captain Oren dies in chapter 9. Asten lets you confirm that as a fact early, then marks any later scene where he still acts.",
    see: "/journal/do-you-need-a-story-bible",
    related: ["gardener", "plotter-and-pantser", "story-bible"],
  },
  {
    slug: "asten",
    term: "Asten",
    short:
      "Asten is a local-first writing app for creative writers that remembers the facts of your story and marks where a draft contradicts them, without writing a word.",
    long: "Asten pairs a writing desk with a rule engine, the compiler, that checks your story against the canon you’ve confirmed. It is made by Verne, and its name is another name for Thoth, the Egyptian god of writing. Your words are black ink; Asten only ever writes in red.",
    see: "/how-it-works",
    related: ["thoth", "canon", "diagnostic", "local-first"],
  },
  {
    slug: "author-override",
    term: "Author override",
    short:
      "Author override is a fact modality that labels a value as your deliberate ruling rather than something established in the text or inferred from it.",
    long: "It records why a fact exists. Asten still compares it like any confirmed fact, so to retire the value it replaces, mark the old fact contested or deprecated.",
    see: "/features/canon",
    related: ["modality", "contested", "suppression", "canon"],
  },
  {
    slug: "binder",
    term: "Binder",
    short:
      "The binder is Asten’s tree of everything in a project: the manuscript, its chapters and scenes, plus your notes and sources.",
    long: "Chapter 3, the scene at the harbor at Veld, and your notes on its tides all sit in one binder, on your computer.",
    see: "/features/desk",
    related: ["snapshot", "compile", "track-changes"],
  },
  {
    slug: "canon",
    term: "Canon",
    short:
      "Canon is the set of facts you have settled about your story; in Asten, confirmed facts are the canon your draft is checked against.",
    long: "Settle in chapter 2 that Mira’s hair is auburn, and that becomes canon. Asten doesn’t decide what’s true in your world; it remembers what you confirmed and points out where a draft disagrees.",
    see: "/features/canon",
    related: ["fact", "confirmed-fact", "story-bible", "continuity-error"],
  },
  {
    slug: "canon-e013",
    term: "CANON-E013",
    short:
      "CANON-E013 is Asten’s attribute-contradiction rule: it marks two confirmed values for the same attribute, such as auburn and black for one character’s hair.",
    long: "If Mira’s hair was settled as auburn in chapter 2 and a later confirmed fact says black, the mark shows both sides. The rule belongs to the Scribe pack and is always on.",
    see: "/features/compiler",
    related: ["scribe-pack", "diagnostic", "confirmed-fact", "predicate"],
  },
  {
    slug: "canon-e101",
    term: "CANON-E101",
    short:
      "CANON-E101 is Asten’s life-after-death rule: it marks a character who acts after their death has been confirmed.",
    long: "Captain Oren dies in chapter 9. If a later scene has him giving orders, Asten marks it. The rule belongs to the Lifecycle pack, which is on by default.",
    see: "/features/compiler",
    related: ["lifecycle-pack", "diagnostic", "confirmed-fact", "entity"],
  },
  {
    slug: "canon-e201",
    term: "CANON-E201",
    short:
      "CANON-E201 is Asten’s clocks rule: it marks places where reading order contradicts the story time you have stated.",
    long: "It belongs to the Clocks pack, which is on by default, and it stays silent until both clocks exist. If the mark lands on a flashback you meant, suppress it; the rule is there for the jumps you didn’t mean.",
    see: "/features/time",
    related: ["story-time", "reading-order", "clocks-pack", "flashback"],
  },
  {
    slug: "cartouche",
    term: "Cartouche",
    short:
      "A cartouche is the oval loop, tied at one end, that encloses a royal name in ancient Egyptian hieroglyphic writing.",
    long: "The word is French for “cartridge,” which the shape resembles. Royal names in cartouches gave early decipherers a foothold in reading hieroglyphs. Asten’s brand borrows the shape to frame names: the wordmark, a character, a diagnostic code.",
    see: "/brand",
    related: ["rubric", "thoth", "asten"],
  },
  {
    slug: "clocks-pack",
    term: "Clocks pack",
    short:
      "The Clocks pack is the Asten method pack that compares reading order with story time; it holds CANON-E201 and is on by default.",
    long: "It stays silent until a project has both clocks, so if you never set story time, you never hear from it.",
    see: "/features/packs",
    related: ["method-pack", "canon-e201", "story-time", "reading-order"],
  },
  {
    slug: "compile",
    term: "Compile",
    short:
      "To compile, in Asten, is to turn your project into a finished file: Markdown, plain text, Fountain, HTML, or .docx.",
    long: "For a PDF, compile to HTML and print the page. Import reads the same family of formats, with a conservative .docx extract. EPUB and FDX export are planned.",
    see: "/features/desk",
    related: ["fountain", "binder", "local-first"],
  },
  {
    slug: "confirmed-fact",
    term: "Confirmed fact",
    short:
      "A confirmed fact is a fact you have settled as canon; in Asten, only confirmed facts can produce hard contradictions.",
    long: "Mira · hair color · auburn, confirmed in chapter 2, is one. Asten’s planned fact proposals would need your confirmation before they count.",
    see: "/features/canon",
    related: ["fact", "proposed-fact", "deprecated-fact", "canon"],
  },
  {
    slug: "contested",
    term: "Contested",
    short:
      "Contested is the fact modality for a claim whose truth is in dispute inside the story, such as a rumor or an unreliable narrator’s account.",
    long: "Asten doesn’t raise contradictions from a contested fact, so you can record what the gossips at Veld believe without it fighting your canon.",
    see: "/features/canon",
    related: ["modality", "author-override", "established", "inferred"],
  },
  {
    slug: "continuity-error",
    term: "Continuity error",
    short:
      "A continuity error is a detail in a story that contradicts something established earlier, like a character’s eye color changing between chapters.",
    long: "Long manuscripts make them easy to commit and hard to catch, because the original detail is hundreds of pages back. Asten’s compiler marks the kinds it can check today: attribute contradictions, a life after a settled death, and clocks that disagree.",
    see: "/journal/continuity-in-a-long-novel",
    related: ["canon", "canon-e013", "canon-e101", "canon-e201"],
  },
  {
    slug: "craft-hint",
    term: "Craft hint",
    short:
      "A craft hint is an optional style note from an Asten method pack; it is never a canon error, and it is off by default.",
    long: "Today the only craft hints come from Diction watch, which notes words like “very” and “really” (`CRAFT-H001` and `CRAFT-H002`). Asten ships no craft doctrine as a default; you opt in.",
    see: "/features/packs",
    related: ["diction-watch", "method-pack", "diagnostic"],
  },
  {
    slug: "deprecated-fact",
    term: "Deprecated fact",
    short:
      "A deprecated fact is one you have retired from canon; Asten keeps it on record but no longer checks your draft against it.",
    long: "Revision changes minds. If Mira was never a sailor after all, deprecating the fact retires it without erasing the decision.",
    see: "/features/canon",
    related: ["confirmed-fact", "proposed-fact", "fact"],
  },
  {
    slug: "diagnostic",
    term: "Diagnostic",
    short:
      "A diagnostic is a red mark Asten places on a passage when a rule notices something; it names the rule and never rewrites your words.",
    long: "Each one carries a code, such as `CANON-E013`, and points at the passage it concerns. What you do next is your call: revise the page, update canon, or suppress the mark.",
    see: "/features/compiler",
    related: ["suppression", "canon-e013", "craft-hint"],
  },
  {
    slug: "diction-watch",
    term: "Diction watch",
    short:
      "Diction watch is an optional Asten method pack that notes filler words such as “very” and “really”; it is off by default.",
    long: "Its notes are craft hints (`CRAFT-H001` and `CRAFT-H002`), not canon errors. Turn it on for a revision pass and off again when you’d rather not hear it.",
    see: "/features/packs",
    related: ["craft-hint", "method-pack"],
  },
  {
    slug: "docs-sidebar",
    term: "Docs sidebar",
    short:
      "The Docs sidebar is Asten’s add-in for Google Docs: a panel that shows your local Asten projects and diagnostics beside the document you’re editing.",
    long: "It can also file a selected passage into your local project. It talks to Asten’s local API on your own computer and is installed by sideloading; it isn’t in the Google Workspace Marketplace.",
    see: "/features/word-and-google-docs",
    related: ["word-task-pane", "sideload", "local-api"],
  },
  {
    slug: "entity",
    term: "Entity",
    short:
      "An entity, in Asten, is anything in your story that facts can be about: a character, place, event, item, rule, source, or other.",
    long: "Mira is an entity; so is the harbor at Veld. Entities can carry aliases, so “the Captain” and “Oren” point at the same person, plus notes of your own.",
    see: "/features/cast",
    related: ["fact", "predicate", "canon"],
  },
  {
    slug: "established",
    term: "Established",
    short:
      "Established is the fact modality for something your story states plainly as true, such as Mira’s auburn hair in chapter 2.",
    long: "Most of a canon is established: what the text says outright. Compare inferred, for what the reader works out, and contested, for what’s in dispute.",
    see: "/features/canon",
    related: ["modality", "inferred", "contested", "author-override"],
  },
  {
    slug: "fabula-and-syuzhet",
    term: "Fabula and syuzhet",
    short:
      "Fabula and syuzhet are Russian Formalist terms for a story’s events in chronological order (fabula) and the order and way the telling presents them (syuzhet).",
    long: "Asten’s two clocks follow the same idea: story time is roughly the fabula, and reading order is roughly the syuzhet. Keeping both lets you say when a scene happens separately from when it’s told.",
    see: "/journal/story-time-and-reading-order",
    related: ["story-time", "reading-order", "flashback"],
  },
  {
    slug: "fact",
    term: "Fact",
    short:
      "A fact, in Asten, is one recorded truth about your story: an entity, a predicate, and a value, such as Mira · hair color · auburn.",
    long: "Each fact also has a modality, a status, an optional stretch of story time when it holds (valid from, valid to), and an optional link to the passage it came from.",
    see: "/features/canon",
    related: ["entity", "predicate", "modality", "confirmed-fact"],
  },
  {
    slug: "flashback",
    term: "Flashback",
    short:
      "A flashback is a scene that shows events from earlier in story time than the scenes around it.",
    long: "Asten allows them and keeps story time and reading order as separate clocks. If `CANON-E201` marks a flashback you meant, suppress the mark and the page stays as you wrote it.",
    see: "/features/time",
    related: ["story-time", "reading-order", "canon-e201", "fabula-and-syuzhet"],
  },
  {
    slug: "fountain",
    term: "Fountain",
    short:
      "Fountain is a plain-text markup language for screenplays, readable as written and convertible into formatted scripts.",
    long: "Asten can compile a project to Fountain and import Fountain files. Export to Final Draft’s FDX format is planned.",
    see: "/for/screenwriters",
    related: ["compile", "binder"],
  },
  {
    slug: "gardener",
    term: "Gardener",
    short:
      "A gardener, in the comparison George R. R. Martin has made, is a writer who plants an idea and discovers the story by writing it.",
    long: "Asten doesn’t ask gardeners for a story bible first. The world views are optional, nothing on the wall is canon until you settle it, and a scene never waits on a wiki.",
    see: "/journal/do-you-need-a-story-bible",
    related: ["architect", "plotter-and-pantser", "spark-sketched-settled"],
  },
  {
    slug: "inferred",
    term: "Inferred",
    short:
      "Inferred is the fact modality for something the story implies rather than states, a conclusion a careful reader would draw from the text.",
    long: "If Mira never gives her age but remembers a storm that hit Veld forty years ago, her being over forty is inferred. Recording the modality keeps clear what the page said and what you worked out.",
    see: "/features/canon",
    related: ["modality", "established", "contested"],
  },
  {
    slug: "lifecycle-pack",
    term: "Lifecycle pack",
    short:
      "The Lifecycle pack is the Asten method pack that holds CANON-E101, which marks a character acting after a confirmed death; it is on by default.",
    long: "Once Captain Oren’s death in chapter 9 is confirmed, the pack notices any later scene where he still acts. If a return is intended, you can suppress the mark.",
    see: "/features/packs",
    related: ["canon-e101", "method-pack", "scribe-pack", "suppression"],
  },
  {
    slug: "local-api",
    term: "Local API",
    short:
      "The local API is the HTTP interface Asten runs on your own computer, at 127.0.0.1:4317, which the Word and Google Docs add-ins use to reach your projects.",
    long: "The address `127.0.0.1` means this machine, so the add-ins reach your projects without a cloud in between. For tinkerers: it’s an Axum server written in Rust.",
    see: "/for/ai-tinkerers",
    related: ["word-task-pane", "docs-sidebar", "local-first", "sqlite"],
  },
  {
    slug: "local-first",
    term: "Local-first",
    short:
      "Local-first software keeps the primary copy of your data on your own device, so it works offline and doesn’t depend on a company’s server.",
    long: "Asten is local-first: your project lives in SQLite on your computer, with no account, no sync, and no telemetry. Verne never receives your manuscript, and your files must stay usable without a subscription or a server.",
    see: "/features/local-first",
    related: ["sqlite", "local-api", "snapshot"],
  },
  {
    slug: "maat",
    term: "Ma’at",
    short:
      "Ma’at is the ancient Egyptian concept of truth, balance, and order, also personified as a goddess who wears an ostrich feather.",
    long: "At the Weighing of the Heart, a person’s heart was weighed against her feather. Asten borrows the idea as a principle: a mark should show its evidence.",
    see: "/name",
    related: ["weighing-of-the-heart", "thoth", "canon"],
  },
  {
    slug: "method-pack",
    term: "Method pack",
    short:
      "A method pack is a bundle of rules or hints added to Asten’s checks; packs mark your pages but never change your manuscript.",
    long: "Today there are four: Scribe (the core canon compiler, always on), Lifecycle and Clocks (on by default), and Diction watch (off by default). No craft doctrine is a default.",
    see: "/features/packs",
    related: ["scribe-pack", "lifecycle-pack", "clocks-pack", "diction-watch"],
  },
  {
    slug: "modality",
    term: "Modality",
    short:
      "A modality, in Asten, says what kind of claim a fact is: established, inferred, contested, or author override.",
    long: "Modality is separate from status. A fact can be confirmed and still contested, which is how you record a rumor that belongs in your canon as a rumor.",
    see: "/features/canon",
    related: ["established", "inferred", "contested", "author-override"],
  },
  {
    slug: "plotter-and-pantser",
    term: "Plotter and pantser",
    short:
      "Plotter and pantser are writers’ shorthand for two habits: plotters outline before drafting, while pantsers write “by the seat of their pants” and discover the story.",
    long: "Most writers mix the two. Asten works either way: you can settle facts before chapter 1 or after the draft, and nothing makes you fill in a form first.",
    see: "/journal/do-you-need-a-story-bible",
    related: ["architect", "gardener", "story-bible"],
  },
  {
    slug: "predicate",
    term: "Predicate",
    short:
      "A predicate is the attribute part of an Asten fact, the thing being said about an entity, such as hair color in Mira · hair color · auburn.",
    long: "Two confirmed facts with the same entity and predicate but different values are exactly what `CANON-E013` marks.",
    see: "/features/canon",
    related: ["fact", "entity", "canon-e013"],
  },
  {
    slug: "proposed-fact",
    term: "Proposed fact",
    short:
      "A proposed fact is a fact waiting for your decision; Asten records it, but it can’t produce a hard contradiction until you confirm it.",
    long: "Asten extracts nothing from your prose today. Planned fact extraction would only ever create proposals like this, never silent canon.",
    see: "/features/canon",
    related: ["confirmed-fact", "deprecated-fact", "fact"],
  },
  {
    slug: "reading-order",
    term: "Reading order",
    short:
      "Reading order is the sequence in which a reader meets the events of a story, page by page, which may differ from story time.",
    long: "It is one of Asten’s two clocks. If chapter 1 opens at the harbor at Veld and chapter 2 goes back ten years, reading order and story time run in different directions. That may be exactly what you meant; Asten’s job is to notice, and yours is to decide.",
    see: "/features/time",
    related: ["story-time", "flashback", "canon-e201", "fabula-and-syuzhet"],
  },
  {
    slug: "rubric",
    term: "Rubric",
    short:
      "A rubric is a heading or mark written in red ink; the word comes from Latin *rubrica*, red ochre, from *ruber*, red.",
    long: "Manuscript makers wrote headings and instructions in red, and much earlier, Egyptian scribes used red ink for headings and marks in otherwise black text. Asten keeps the rule: your words are black ink, and Asten only ever writes in red.",
    see: "/name",
    related: ["cartouche", "diagnostic", "thoth"],
  },
  {
    slug: "scribe-pack",
    term: "Scribe pack",
    short:
      "The Scribe pack is Asten’s core canon compiler; it is locked on and holds CANON-E013, the attribute-contradiction rule.",
    long: "It is the part of Asten that keeps the record. Other packs add rules alongside it, but the Scribe is always at the desk.",
    see: "/features/packs",
    related: ["method-pack", "canon-e013", "canon"],
  },
  {
    slug: "series-bible",
    term: "Series bible",
    short:
      "A series bible is a story bible that spans several books, seasons, or episodes, recording the characters, places, rules, and timeline that must stay consistent.",
    long: "A series bible usually grows out of the books themselves. In Asten, canon grows the same way: from facts you confirm as you write, not from a form filled in up front.",
    see: "/journal/do-you-need-a-story-bible",
    related: ["story-bible", "canon", "continuity-error"],
  },
  {
    slug: "sideload",
    term: "Sideload",
    short:
      "To sideload an add-in is to install it manually rather than through an official store.",
    long: "Asten’s Word task pane and Google Docs sidebar are sideload only. They aren’t in the Microsoft or Google stores.",
    see: "/features/word-and-google-docs",
    related: ["word-task-pane", "docs-sidebar", "local-api"],
  },
  {
    slug: "snapshot",
    term: "Snapshot",
    short:
      "A snapshot, in Asten, is a named record of your project at a moment in time, such as “Before cutting chapter 4.”",
    long: "Snapshots sit alongside an append-only operation history, which records each change instead of overwriting the past.",
    see: "/features/desk",
    related: ["track-changes", "binder", "local-first"],
  },
  {
    slug: "spark-sketched-settled",
    term: "Spark, sketched, settled",
    short:
      "Spark, sketched, and settled are the three statuses of an Asten world entry, from a loose idea to a decided part of your world.",
    long: "Pin “a harbor that only appears at dusk” as a spark, sketch it later, and settle it when you’re sure. Nothing on the wall is canon until you settle it, and settling an entry can mint a confirmed fact.",
    see: "/features/world",
    related: ["world-wall-atlas-and-dossier", "confirmed-fact", "gardener"],
  },
  {
    slug: "sqlite",
    term: "SQLite",
    short:
      "SQLite is a widely used, self-contained database engine that stores a whole database in a single ordinary file.",
    long: "Asten keeps your projects in SQLite on your computer. With the desktop shell, the file is `asten.db` in your system’s application-data folder, so a backup is a copy of a file.",
    see: "/features/local-first",
    related: ["local-first", "local-api", "snapshot"],
  },
  {
    slug: "story-bible",
    term: "Story bible",
    short:
      "A story bible is a reference document recording a story’s characters, places, rules, and timeline so the writing stays consistent.",
    long: "Some writers build one first; many never do. Asten doesn’t require one: the cast isn’t a mandatory character sheet, the world views are optional, and canon can grow from facts you confirm while you draft.",
    see: "/journal/do-you-need-a-story-bible",
    related: ["series-bible", "canon", "gardener", "architect"],
  },
  {
    slug: "story-time",
    term: "Story time",
    short:
      "Story time is when events happen in the world of the story, as opposed to reading order, which is when the reader meets them.",
    long: "It is one of Asten’s two clocks. The great storm at Veld can happen years before chapter 1 in story time and still be told in chapter 20.",
    see: "/journal/story-time-and-reading-order",
    related: ["reading-order", "flashback", "canon-e201", "fabula-and-syuzhet"],
  },
  {
    slug: "suppression",
    term: "Suppression",
    short:
      "A suppression silences a diagnostic you’ve decided to keep; in Asten, any rule can be suppressed, and suppressions are stored with your project.",
    long: "Use one when the page is right on purpose, such as a narrator who lies about Mira’s hair. The mark goes quiet; your sentence stays exactly as you wrote it.",
    see: "/features/compiler",
    related: ["diagnostic", "author-override", "contested"],
  },
  {
    slug: "thoth",
    term: "Thoth",
    short:
      "Thoth is the ancient Egyptian god of writing, wisdom, the moon, and reckoning, often shown with the head of an ibis, and Asten is another of his names.",
    long: "Scribe of the gods and credited in Egyptian tradition with inventing writing, he recorded the verdict at the Weighing of the Heart rather than deciding it. That is Asten’s job description: keep the record, weigh the page, leave the verdict to you.",
    see: "/name",
    related: ["asten", "weighing-of-the-heart", "maat", "rubric"],
  },
  {
    slug: "track-changes",
    term: "Track changes",
    short:
      "Track changes is a way of recording insertions and deletions as visible marks, so a writer or editor can review each edit.",
    long: "Asten’s desk records insertions and deletions as track-change marks. It isn’t a full replacement for Word’s review workflow; compile to .docx when you need that.",
    see: "/features/desk",
    related: ["snapshot", "compile", "word-task-pane"],
  },
  {
    slug: "weighing-of-the-heart",
    term: "Weighing of the Heart",
    short:
      "The Weighing of the Heart is the ancient Egyptian judgment scene in which a dead person’s heart is weighed against the feather of Ma’at.",
    long: "In versions found in the Book of the Dead, Anubis tends the scales and Thoth records the result. Asten takes Thoth’s role: it weighs each page against your canon and records what doesn’t balance, but the verdict is yours.",
    see: "/name",
    related: ["maat", "thoth", "canon"],
  },
  {
    slug: "word-task-pane",
    term: "Word task pane",
    short:
      "The Word task pane is Asten’s add-in for Microsoft Word: a side panel that shows your local Asten projects and diagnostics next to your document.",
    long: "Select a passage and it can file it into your local project. It talks to Asten’s local API on your own computer and is installed by sideloading; it isn’t in Microsoft’s add-in store.",
    see: "/features/word-and-google-docs",
    related: ["docs-sidebar", "sideload", "local-api"],
  },
  {
    slug: "world-wall-atlas-and-dossier",
    term: "World wall, atlas, and dossier",
    short:
      "The world wall, atlas, and dossier are Asten’s three optional views of your world: a board of loose cards, a rough map for places, and an organized list.",
    long: "Pin the harbor at Veld to the wall as a scrap, drop it on the atlas once you know where it sits, and find it in the dossier grouped by kind. The atlas is for orientation; it doesn’t check travel times.",
    see: "/features/world",
    related: ["spark-sketched-settled", "entity", "gardener"],
  },
];
