import type { GlyphKey } from "../art/Glyphs";
import type { ArtKey, QA, Section } from "./types";

export type DemoKey = "weigh" | "clocks" | "pyramid" | "cartouche" | "packs" | "compile" | "local" | "desk";

export interface Feature {
  slug: string;
  path: string;
  name: string;
  glyph: GlyphKey;
  /** One line for cards and menus. */
  short: string;
  title: string;
  description: string;
  h1: string;
  lede: string;
  /** A direct, quotable answer to "what is this?" */
  answer: string;
  points: { title: string; text: string }[];
  sections: Section[];
  faqs: QA[];
  demo?: DemoKey;
  art: ArtKey;
  related: string[];
}

export const features: Feature[] = [
  {
    slug: "desk",
    path: "/features/desk",
    name: "The desk",
    glyph: "scroll",
    short: "A calm manuscript editor with a binder, comments, find, page view, and track changes.",
    title: "The desk: a manuscript editor built for long stories",
    description:
      "Asten’s desk is a calm manuscript editor: a binder for chapters and scenes, comments, find, word counts, a letter-sized page view, track changes, and snapshots.",
    h1: "A quiet desk for a long manuscript.",
    lede: "Chapters and scenes in a binder, a clean page in the middle, and Asten’s red marks in the margin. The page itself stays yours, with no ornament, no pop-ups, and no ghostwriter.",
    answer:
      "The Asten desk is a manuscript editor for long-form writing. It has a binder of chapters, scenes, notes, and sources; a rich-text page; comments; find; word counts; a letter-sized page view; track-change marks; and named snapshots. Diagnostics sit in the margin and never rewrite your text.",
    points: [
      { title: "A binder, not a folder maze", text: "Manuscript, chapters, scenes, notes, and sources in one tree. Exclude a scene from compile without deleting it." },
      { title: "A page that stays out of the way", text: "Headings, emphasis, and the formatting a manuscript needs. Literata on warm paper. Nothing animated near your words." },
      { title: "Comments and track changes", text: "Leave notes to yourself or an editor. Insertions and deletions show as marks you can read back." },
      { title: "Page view for the feel of length", text: "A letter-sized sheet with a running header, so you can feel how long the chapter is. It is not a typesetter." },
      { title: "Find, counts, snapshots", text: "Find next and previous, live word counts, and named snapshots you can restore without any cloud." },
      { title: "Red in the margin", text: "When a page disagrees with your canon, the mark appears beside the sentence, with the evidence one click away." },
    ],
    sections: [
      {
        id: "what-the-desk-is-for",
        heading: "What is the desk for?",
        blocks: [
          { kind: "p", text: "The desk is where the book gets written. Everything else in Asten (canon, cast, time, world) exists to serve the page, and none of it is required before you start typing." },
          { kind: "p", text: "Think of it as a writer’s subset of a word processor: the parts you use every day on a manuscript, without the layout features you only need for a printed brochure. When the book needs to leave, you [compile](/features/word-and-google-docs) it to Markdown, Fountain, HTML, or Word." },
        ],
      },
      {
        id: "the-binder",
        heading: "How does the binder work?",
        blocks: [
          { kind: "p", text: "The binder is a tree. A manuscript holds chapters; chapters hold scenes. Notes and sources live alongside the prose, so research is one click from the scene that needs it." },
          { kind: "list", items: ["Reorder chapters and scenes by moving them in the tree.", "Mark a scene as excluded from compile while you decide its fate.", "Keep wiki pages, notes, and sources in the same project, never in a separate app."] },
        ],
      },
      {
        id: "what-the-desk-is-not",
        heading: "What the desk is not",
        blocks: [
          { kind: "p", text: "Asten is not trying to be Word. Track changes are insertion and deletion marks, not a full review workflow. Find moves one match at a time. Page view is a letter sheet for the feel of length, not a print-production engine. These are deliberate limits, written down so nobody is surprised." },
          { kind: "callout", tone: "note", title: "Asten never writes on the page", text: "There is no “continue writing” button, no rewrite suggestion, and no autocomplete for prose. The only thing Asten adds is a red mark in the margin." },
        ],
      },
    ],
    faqs: [
      { q: "Is the Asten desk a replacement for Microsoft Word?", a: "Not entirely. The Asten desk covers what a manuscript needs day to day, including a binder, comments, find, counts, page view, and track-change marks, and it compiles to .docx when a publisher or editor wants Word." },
      { q: "Can I import an existing manuscript into Asten?", a: "Yes. Asten imports Markdown, plain text, Fountain, HTML, and .docx. The .docx import is deliberately conservative: it keeps your text and basic structure, not complex layout." },
      { q: "How do I see how long my book is in Asten?", a: "The Asten desk shows live word counts, and its page view lays a chapter out on letter-sized sheets with a running header, so you can feel the length in pages." },
    ],
    demo: "desk",
    art: "scroll",
    related: ["/features/canon", "/features/word-and-google-docs", "/features/local-first", "/for/novelists"],
  },
  {
    slug: "canon",
    path: "/features/canon",
    name: "Canon checks",
    glyph: "feather",
    short: "Settle a fact once. Asten marks any page that disagrees, and shows both sides.",
    title: "Canon checks: continuity without the spreadsheet",
    description:
      "Settle a fact once, like Mira’s auburn hair, and Asten marks any later page that disagrees, with the code, the evidence, and your options. You always decide.",
    h1: "Settle it once. Asten remembers.",
    lede: "Canon is what your story says is true. Settle a fact (*Mira · hair · auburn*) and Asten weighs every later page against it. If chapter 19 says black, you get a red mark with both sides, never a rewrite.",
    answer:
      "Canon checks in Asten compare your manuscript against facts you have settled. A fact is an entity, a predicate, and a value, with a status, a modality, an optional span of story time, and a source. When two confirmed facts disagree, or prose names a different hair or eye color near a character, Asten marks it in red with code CANON-E013.",
    points: [
      { title: "Facts with a source", text: "Every fact records where it came from, so a mark can show you the sentence that settled it." },
      { title: "Statuses you control", text: "Proposed, confirmed, deprecated. Only confirmed facts raise hard errors; softer disagreements arrive as warnings." },
      { title: "Room for lies and legends", text: "Modalities include established, inferred, contested, and author override. Contested facts never raise contradictions." },
      { title: "Change over time", text: "Give a fact a valid-from and valid-to in story time, and a dyed head of hair stops being a contradiction." },
      { title: "Stable codes", text: "Every mark carries a code, like CANON-E013, that means the same thing every time." },
      { title: "Silence with a reason", text: "Suppress a mark or a whole rule. Suppressions are stored with the project, so the decision is remembered." },
    ],
    sections: [
      {
        id: "what-is-canon",
        heading: "What does “canon” mean in Asten?",
        blocks: [
          { kind: "p", text: "In fandom, *canon* means what the source material says is true. In Asten it means the same thing for your own book: the details you have decided are true, recorded as [facts](/glossary#fact)." },
          { kind: "terms", items: [
            { term: "Entity", text: "Who or what the fact is about: Mira, the harbor at Veld, Captain Oren’s sword." },
            { term: "Predicate", text: "The attribute: hair color, status, birthplace." },
            { term: "Value", text: "What is true: auburn, dead, Veld." },
            { term: "Status", text: "Proposed, confirmed, or deprecated. Only confirmed facts produce errors." },
            { term: "Modality", text: "Established, inferred, contested, or author override. Contested facts stay quiet." },
            { term: "Story time", text: "An optional valid-from and valid-to, for facts that change during the book." },
          ] },
        ],
      },
      {
        id: "what-a-mark-looks-like",
        heading: "What does a mark look like?",
        blocks: [
          { kind: "p", text: "A mark is short and specific. It names the rule, quotes both sides, and lists what you can do. It does not tell you which side is right, because only you know that." },
          { kind: "diagnostic", code: "CANON-E013", title: "Mira’s hair was settled as auburn.", lines: ["Settled in chapter 2: Mira · hair_color · auburn", "Chapter 19 says: “Mira pushed her black hair out of her eyes.”", "Keep my sentence · Update canon · Add a change in story time · Silence this rule"] },
          { kind: "p", text: "Two confirmed facts that disagree over overlapping story time are an **error**. A disagreement involving a proposed fact, or a color word that appears near a character’s name in the prose, is a **warning**: worth a look, not a verdict." },
        ],
      },
      {
        id: "how-facts-get-in",
        heading: "How do facts get into canon?",
        blocks: [
          { kind: "list", items: [
            "Add them directly in the cast or canon views.",
            "Settle a world entry. Moving it from *sketched* to *settled* can mint a confirmed fact.",
            "Give a wiki page an entity and attributes; each attribute becomes a fact.",
            "Write an inline assertion in a note, such as `[[mira.hair_color=auburn]]`, if you like typing more than clicking.",
          ] },
          { kind: "callout", tone: "rubric", title: "Asten does not read your mind", text: "Today, Asten does not extract facts from your prose. You settle what matters. Planned extraction will only ever *propose* facts for you to confirm. See the [roadmap](/roadmap)." },
        ],
      },
      {
        id: "intentional-contradictions",
        heading: "What if the contradiction is on purpose?",
        blocks: [
          { kind: "p", text: "Unreliable narrators, lies, disguises, and legends are part of fiction. Mark a fact as *contested* and it stays on file without raising errors. *Deprecate* a fact you’ve abandoned. Or silence the mark for that passage, and the decision stays with the project." },
        ],
      },
    ],
    faqs: [
      { q: "How does Asten know a sentence contradicts my canon?", a: "Asten compares facts you have settled, and for hair and eye colors it also looks for a different color word within a short window of the character’s name in your prose. It uses deterministic rules, not an AI model." },
      { q: "Can Asten handle a character who changes over the story?", a: "Yes. Give a fact a valid-from and valid-to in story time, and Asten only compares facts whose spans overlap, so auburn hair before the dye and black hair after it are not a contradiction." },
      { q: "Can I turn off a canon check in Asten?", a: "Yes. You can suppress a single mark or a whole rule, and the suppression is stored with the project. The attribute check belongs to the Scribe pack, which is always available, but any individual mark can be silenced." },
    ],
    demo: "weigh",
    art: "scales",
    related: ["/how-it-works", "/features/cast", "/features/compiler", "/journal/continuity-in-a-long-novel"],
  },
  {
    slug: "cast",
    path: "/features/cast",
    name: "Cast",
    glyph: "seated",
    short: "Characters and other entities with aliases, notes, and the facts you’ve confirmed.",
    title: "Cast: characters without the character-sheet homework",
    description:
      "The Asten cast keeps characters, places, items, and events as entities with aliases and confirmed facts. Name someone when the draft needs them, not before.",
    h1: "Name someone when the draft needs them.",
    lede: "The cast is a view of your book, not a form to fill in before chapter one. Characters, places, items, and events appear when you give them a fact worth keeping.",
    answer:
      "The Asten cast lists the entities in your story (characters, places, events, items, rules, sources, and others) with their aliases, notes, and confirmed facts. It is optional and grows as you write; nothing in it is required before you draft a scene.",
    points: [
      { title: "More than characters", text: "Entities can be characters, places, events, items, rules, sources, or anything else." },
      { title: "Aliases", text: "Mira, Captain Mira, the harbormaster’s daughter. One entity, many names." },
      { title: "Facts at a glance", text: "Each entity shows the facts you’ve settled about it, with their status and source." },
      { title: "No mandatory fields", text: "A cast member can be a name and nothing else until the story needs more." },
    ],
    sections: [
      {
        id: "why-not-character-sheets",
        heading: "Why not character sheets?",
        blocks: [
          { kind: "p", text: "Many writing tools greet you with a 40-field character questionnaire. Some writers love that. Many [discovery writers](/glossary#gardener) find it kills the draft. Asten takes the other road: the cast fills in as you settle facts, so the record follows the story rather than leading it." },
        ],
      },
      {
        id: "names-in-cartouches",
        heading: "Why are names drawn in cartouches?",
        blocks: [
          { kind: "p", text: "Egyptian scribes enclosed royal names in a *cartouche*, an oval ring that marks a name as important. Asten borrows the idea: an entity’s name appears in a cartouche wherever it is referenced. Try writing a character’s name in one below. It’s a toy, but it is how the cast looks." },
        ],
      },
    ],
    faqs: [
      { q: "Do I have to create characters before writing in Asten?", a: "No. Asten lets you write first and add a character to the cast only when there is a fact about them worth keeping." },
      { q: "Can one Asten character have several names?", a: "Yes. Each entity in the Asten cast can carry aliases, so nicknames, titles, and false names point to the same person." },
    ],
    demo: "cartouche",
    art: "ibis",
    related: ["/features/canon", "/features/world", "/for/fan-fiction", "/glossary"],
  },
  {
    slug: "time",
    path: "/features/time",
    name: "Two clocks",
    glyph: "moon",
    short: "Story time and reading order, kept separately, so flashbacks are a choice and not a mistake.",
    title: "Two clocks: story time and reading order",
    description:
      "Asten keeps two clocks: when events happen (story time) and when readers meet them (reading order). It marks scenes whose story time runs backward, so you can decide.",
    h1: "Two clocks, kept honest.",
    lede: "Every story has two timelines: the order things happen, and the order readers find out. Asten keeps both, and marks the moments they cross so you can tell a flashback from a slip.",
    answer:
      "Asten tracks story time (when an event happens in the world) and reading order (when the reader meets it) as two separate clocks. When a scene’s story time runs backward from the scene before it, Asten raises CANON-E201. If the jump is a deliberate flashback, you silence the mark.",
    points: [
      { title: "Story time", text: "When it happens in the world: day 14, the spring of 412, three days after the siege." },
      { title: "Reading order", text: "When the reader meets it: chapter 3, scene 2." },
      { title: "CANON-E201", text: "A scene that steps backward in story time from the one before it in reading order gets a mark." },
      { title: "CANON-E101", text: "Someone acting after a settled death gets a mark, unless the event’s notes say it’s a flashback, memory, or earlier beat." },
      { title: "Silent until dated", text: "The clocks only compare scenes you’ve placed on both. Undated scenes are never judged." },
      { title: "No timeline required", text: "Draft first. Date scenes when the order starts to matter." },
    ],
    sections: [
      {
        id: "why-two-clocks",
        heading: "Why does a story need two clocks?",
        blocks: [
          { kind: "p", text: "Narrative theorists call these the *fabula* (what happened, in order) and the *syuzhet* (how the telling arranges it). You don’t need the jargon to feel the problem: a flashback is the syuzhet stepping back in the fabula. Done on purpose, it’s craft. Done by accident, it’s a reader frowning at page 212." },
          { kind: "p", text: "Read more in [Story time vs. reading order](/journal/story-time-and-reading-order)." },
        ],
      },
      {
        id: "what-asten-checks",
        heading: "What does Asten check, exactly?",
        blocks: [
          { kind: "diagnostic", code: "CANON-E201", title: "Reading order places “The harbor burns” before “Oren sails”, but story time runs the other way.", lines: ["The harbor burns · story time: day 15 · chapter 9", "Oren sails · story time: day 14 · chapter 10", "Reorder the binder · Retcon a date · Silence for this flashback"] },
          { kind: "p", text: "Asten compares neighboring dated scenes in reading order. If story time runs backward, it raises CANON-E201. It does not yet recognize flashbacks automatically. When you meant it, silence the mark and the decision is stored with the project." },
          { kind: "diagnostic", code: "CANON-E101", title: "Oren acts after a settled death (Oren pours the tea).", lines: ["Settled: Captain Oren · status · dead (chapter 9)", "Later beat: “Oren pours the tea” · chapter 14", "Mark the beat a flashback · Move the death · Silence this rule"] },
        ],
      },
      {
        id: "thoth-and-the-moon",
        heading: "Why the moon?",
        blocks: [
          { kind: "p", text: "Thoth, the god Asten is named for, was a moon god and the reckoner of time. A story retold by the Greek writer Plutarch tells how Thoth won five extra days from the moon in a game, so that a birth forbidden on every day of the year could happen on days that belonged to no month. A keeper of time who makes room for a story is a fair patron for a timeline." },
        ],
      },
    ],
    faqs: [
      { q: "Does Asten treat every flashback as an error?", a: "Asten marks a scene whose story time runs backward from the previous scene in reading order with CANON-E201, because it cannot yet tell a deliberate flashback from a slip. You silence the mark for scenes you meant, and Asten remembers." },
      { q: "Do I have to date every scene in Asten?", a: "No. Asten only compares scenes that have both a story time and a reading order. Undated scenes are never marked." },
      { q: "Can Asten use a fantasy calendar?", a: "Story time in Asten is a label you write, such as “day 14”, so any calendar can be used today. Proper custom calendars, with eras and leap rules, are planned." },
    ],
    demo: "clocks",
    art: "moon",
    related: ["/journal/story-time-and-reading-order", "/features/canon", "/features/packs", "/for/screenwriters"],
  },
  {
    slug: "world",
    path: "/features/world",
    name: "World",
    glyph: "hills",
    short: "An optional wall, atlas, and dossier. Spark, sketch, settle. Never a gate.",
    title: "World: optional worldbuilding that never blocks the draft",
    description:
      "Asten’s world tools are optional: a wall for scraps, an atlas for places, and a dossier by kind. Ideas move from spark to sketched to settled, and settled ones can become canon.",
    h1: "Most of the world stays below the sand.",
    lede: "A novel shows readers the pyramid. The maps, factions, histories, and rules sit in the chambers underneath. Asten gives them a place to live without making you dig before you write.",
    answer:
      "Asten’s world tools are an optional wall (loose cards for scraps), an atlas (a rough map for pinning places), and a dossier (entries grouped by kind). Entries can be places, cultures, systems, creatures, items, questions, or fragments, and move from spark to sketched to settled. Settling an entry can mint a confirmed fact.",
    points: [
      { title: "The wall", text: "Pin a scrap, a question, a fragment of dialogue. Loose cards, no filing required." },
      { title: "The atlas", text: "A rough map for pinning places. Good enough to remember that Veld is east of the reef." },
      { title: "The dossier", text: "Everything grouped by kind: places, cultures, systems, creatures, items, questions, fragments." },
      { title: "Spark → sketched → settled", text: "Ideas mature at your pace. Only settled entries become canon." },
    ],
    sections: [
      {
        id: "gardeners-and-architects",
        heading: "Do I have to build the world first?",
        blocks: [
          { kind: "p", text: "No. George R. R. Martin has described two kinds of writers: architects, who plan the house before building, and gardeners, who plant and see what grows. Asten doesn’t rank them. An architect can fill the dossier before chapter one. A gardener can write twenty chapters and settle the harbor at Veld afterwards." },
          { kind: "p", text: "There is a real risk on the architect side: the world can become a beautiful way to avoid writing the book. That’s why every world tool in Asten is optional and lives one click away from the page, not in front of it. More in [Do you need a story bible?](/journal/do-you-need-a-story-bible)" },
        ],
      },
      {
        id: "settling-mints-facts",
        heading: "How does the world connect to canon?",
        blocks: [
          { kind: "p", text: "A world entry starts as a *spark*. You sketch it as it grows. When you *settle* it, Asten can mint a confirmed fact, and from then on the canon checks treat it as true. Sparks and sketches never raise errors; they’re allowed to be wrong." },
        ],
      },
    ],
    faqs: [
      { q: "Is Asten a worldbuilding app?", a: "Asten is a writing app with optional worldbuilding. Its wall, atlas, and dossier hold places, cultures, systems, creatures, items, questions, and fragments, but none of them are required to write." },
      { q: "Does Asten’s atlas check travel times?", a: "Not yet. Asten’s atlas is a rough map for pinning places. Travel and distance checks are on the roadmap." },
    ],
    demo: "pyramid",
    art: "pyramid",
    related: ["/for/worldbuilders", "/journal/do-you-need-a-story-bible", "/features/canon", "/features/cast"],
  },
  {
    slug: "compiler",
    path: "/features/compiler",
    name: "The compiler",
    glyph: "kit",
    short: "Deterministic rules with stable codes, suppressions, snapshots, and an append-only history.",
    title: "The compiler: deterministic checks you can read",
    description:
      "Under Asten’s red marks is a small, deterministic rule engine with stable codes, three severities, suppressions, snapshots, and an append-only history of your edits.",
    h1: "A compiler for your canon.",
    lede: "Programmers have compilers that read code and point at the line that breaks. Asten does the same for a story’s facts, with deterministic rules, stable codes, and evidence for every mark.",
    answer:
      "Asten’s compiler is a deterministic rule engine written in Rust. It reads your settled facts, timeline, and manuscript, and emits diagnostics with stable codes (CANON-E013, CANON-E101, CANON-E201, and optional CRAFT hints) at three severities: error, warning, and hint. It runs entirely on your machine and never changes your text.",
    points: [
      { title: "Deterministic", text: "Same project, same marks. No model, no randomness, no “it depends on the prompt”." },
      { title: "Stable codes", text: "CANON-E013 always means an attribute contradiction. Search it, discuss it, suppress it." },
      { title: "Three severities", text: "Errors for confirmed contradictions, warnings for softer ones, hints for optional craft notes." },
      { title: "Evidence attached", text: "Every mark points at a span in a document and explains itself." },
      { title: "Append-only history", text: "Document operations are recorded, and named snapshots restore a revision on your own disk." },
      { title: "Stable block ids", text: "Paragraphs carry identities, so a mark can follow its sentence when you edit around it." },
    ],
    sections: [
      {
        id: "the-rules-today",
        heading: "Which rules ship today?",
        blocks: [
          { kind: "table", head: ["Code", "Pack", "What it notices", "Default"], rows: [
            ["CANON-E013", "Scribe", "Two values for the same attribute over overlapping story time; a different hair or eye color near a character’s name", "Always on"],
            ["CANON-E101", "Lifecycle", "A character acting after a settled death (flashback, memory, and earlier beats excepted)", "On"],
            ["CANON-E201", "Clocks", "Neighboring scenes whose story time runs backward against reading order", "On"],
            ["CRAFT-H001/H002", "Diction watch", "Filter words such as “very” and “really”, as optional notes", "Off"],
          ] },
        ],
      },
      {
        id: "why-not-ai",
        heading: "Why rules and not an AI model?",
        blocks: [
          { kind: "p", text: "Because a mark you can’t trust trains you to ignore all of them. Deterministic rules can be explained, tested, and argued with. They also run instantly on a laptop with no network. AI may later help *propose* facts, but the compiler that judges contradictions stays rule-based and inspectable. See [our AI stance](/ai)." },
        ],
      },
      {
        id: "under-the-hood",
        heading: "What’s under the hood?",
        blocks: [
          { kind: "p", text: "For the curious: the domain, application, storage, and diagnostics layers are Rust crates. Storage is SQLite. A small Axum server exposes a local API on `127.0.0.1:4317`. The interface is React and TypeScript, wrapped in a Tauri desktop shell. One frontend serves both the desktop and the browser. More in [For AI tinkerers](/for/ai-tinkerers)." },
        ],
      },
    ],
    faqs: [
      { q: "Is the Asten compiler an AI?", a: "No. The Asten compiler is a deterministic rule engine. It produces the same marks for the same project every time, and it runs entirely on your computer." },
      { q: "What do Asten’s diagnostic codes mean?", a: "CANON-E013 is an attribute contradiction, CANON-E101 is a life after a settled death, CANON-E201 is story time running backward against reading order, and CRAFT-H codes are optional diction hints. The [glossary](/glossary) defines each one." },
    ],
    art: "palette",
    related: ["/features/packs", "/features/canon", "/for/ai-tinkerers", "/ai"],
  },
  {
    slug: "packs",
    path: "/features/packs",
    name: "Method packs",
    glyph: "papyrus",
    short: "Optional sets of rules and hints. Turn on what helps; nothing becomes doctrine.",
    title: "Method packs: rules you can decline",
    description:
      "Asten’s checks come in packs: Scribe (always on), Lifecycle, Clocks, and the optional Diction watch. Enable what helps you. No craft doctrine is ever a default.",
    h1: "A pack is a set of rules you can decline.",
    lede: "Asten doesn’t believe in one right way to write. Its checks come in packs you turn on and off, and craft advice stays switched off until you ask for it.",
    answer:
      "Method packs are optional groups of rules in Asten. Four ship today: Scribe (the core canon check, always on), Lifecycle (life after a settled death), Clocks (story time against reading order), and Diction watch (optional filter-word hints, off by default). Packs add marks; they never change your manuscript.",
    points: [
      { title: "Scribe", text: "The core canon compiler. Attribute contradictions, CANON-E013. Always on." },
      { title: "Lifecycle", text: "Notices a settled death followed by a later act. CANON-E101. On by default." },
      { title: "Clocks", text: "Compares reading order with stated story time. CANON-E201. Silent until both clocks exist." },
      { title: "Diction watch", text: "Optional craft hints for filter words. Never a canon error. Off until you enable it." },
    ],
    sections: [
      {
        id: "no-doctrine",
        heading: "Why packs instead of built-in advice?",
        blocks: [
          { kind: "p", text: "Craft advice is contested. One editor’s “cut every adverb” is another writer’s voice. So Asten separates *truth* (your canon contradicts itself) from *taste* (you used “very” twice), and puts taste in a pack that starts switched off." },
          { kind: "callout", tone: "note", title: "Hints are notes, not errors", text: "Diction hints appear as gentle notes: “Filter word. Keep it if the voice wants it.” They never count as a canon error." },
        ],
      },
      {
        id: "future-packs",
        heading: "What packs might come next?",
        blocks: [
          { kind: "p", text: "The plan is to let method packs carry templates, scene fields, reports, and advisory rules for approaches such as mystery clue ledgers or screenplay sequences, always optional and never the default. These are planned, not shipped. See the [roadmap](/roadmap)." },
        ],
      },
    ],
    faqs: [
      { q: "Can I turn off Asten’s checks?", a: "Yes. Lifecycle, Clocks, and Diction watch can be switched off. Scribe, the core attribute check, stays on, but any individual mark can be silenced." },
      { q: "Does Asten give writing-style advice?", a: "Only if you ask. Asten’s Diction watch pack offers optional filter-word hints and is off by default. Asten never rewrites your sentences." },
    ],
    demo: "packs",
    art: "lotus",
    related: ["/features/compiler", "/features/canon", "/ai", "/glossary"],
  },
  {
    slug: "word-and-google-docs",
    path: "/features/word-and-google-docs",
    name: "Word & Google Docs",
    glyph: "door",
    short: "Compile to Markdown, Fountain, HTML, and .docx. Optional Word and Docs add-ins talk to your own machine.",
    title: "Word and Google Docs: welcome guests",
    description:
      "Asten compiles to Markdown, plain text, Fountain, HTML, and .docx, and imports them back. Optional Word and Google Docs add-ins show marks from your local Asten.",
    h1: "Word and Google Docs are welcome guests.",
    lede: "Your editor wants a .docx. Your co-writer lives in Google Docs. Asten sends the manuscript out and takes it back, and two small add-ins let those apps show Asten’s marks without your canon leaving your machine.",
    answer:
      "Asten compiles a manuscript to Markdown, plain text, Fountain, HTML, or .docx, and imports those formats back. PDF comes from printing the HTML. Optional Word and Google Docs add-ins list your local projects, show diagnostics, and can file a selected passage into Asten; they talk only to the Asten API on 127.0.0.1:4317.",
    points: [
      { title: "Compile", text: "Markdown, plain text, Fountain, HTML, and .docx. Excluded scenes stay out." },
      { title: "Import", text: "Bring drafts home from the same formats. The .docx import is conservative on purpose." },
      { title: "Word task pane", text: "See Asten’s marks next to your Word document and file a selection into a local project." },
      { title: "Docs sidebar", text: "The same, inside Google Docs, through Apps Script." },
    ],
    sections: [
      {
        id: "formats",
        heading: "Which formats does Asten support?",
        blocks: [
          { kind: "table", head: ["Format", "Compile", "Import", "Good for"], rows: [
            ["Markdown", "Yes", "Yes", "Plain-text backups, static sites, version control"],
            ["Plain text", "Yes", "Yes", "Anything, forever"],
            ["Fountain", "Yes", "Yes", "Screenplays"],
            ["HTML", "Yes", "Yes", "Web, and PDF by printing"],
            [".docx", "Yes", "Yes (conservative)", "Editors, agents, publishers"],
          ] },
        ],
      },
      {
        id: "how-the-add-ins-work",
        heading: "How do the add-ins work?",
        blocks: [
          { kind: "p", text: "The add-ins are thin windows onto your own computer. They call the Asten API at `127.0.0.1:4317` (that address only exists on your machine) to list projects, show diagnostics, and file a selected passage into a project. Microsoft and Google already have the document you opened; Asten doesn’t send your canon to them or to Verne." },
          { kind: "callout", tone: "note", title: "Sideload only, for now", text: "The add-ins are installed by hand (“sideloaded”) and are not in the Microsoft or Google stores. Early-access writers get instructions." },
        ],
      },
    ],
    faqs: [
      { q: "Can Asten export to Word?", a: "Yes. Asten compiles a manuscript to .docx, as well as Markdown, plain text, Fountain, and HTML." },
      { q: "Does the Asten Google Docs add-on send my book to Verne?", a: "No. The Asten Docs sidebar talks only to the Asten API running on your own computer at 127.0.0.1:4317. Verne never receives your text." },
      { q: "Can Asten export EPUB or PDF?", a: "PDF comes from printing Asten’s HTML output. EPUB is planned but not available yet." },
    ],
    demo: "compile",
    art: "scroll",
    related: ["/features/desk", "/features/local-first", "/for/screenwriters", "/privacy"],
  },
  {
    slug: "local-first",
    path: "/features/local-first",
    name: "Local-first",
    glyph: "house",
    short: "One file on your computer. No account, no sync, no telemetry, no training.",
    title: "Local-first: your manuscript never leaves home",
    description:
      "Asten keeps your whole project in one SQLite file on your computer. No account, no sync, no telemetry, and no training on your words. Export anytime.",
    h1: "Your manuscript never leaves home.",
    lede: "No account. No cloud. No telemetry. No training. Your project is a single file on your own disk, and if Asten disappeared tomorrow your words would still be yours, in formats anything can open.",
    answer:
      "Asten is local-first: every project lives in a SQLite database file on your computer (asten.db, in the operating system’s app-data folder). There is no account, no sync service, and no telemetry, and Verne never receives your manuscript. You can export to Markdown, plain text, Fountain, HTML, or .docx at any time.",
    points: [
      { title: "One file", text: "Projects, scenes, facts, comments, snapshots, and marks live in `asten.db` on your disk." },
      { title: "No account", text: "Nothing to sign up for, nothing to log into, nothing to lose access to." },
      { title: "No sync, no telemetry", text: "Asten doesn’t phone home. Verne can’t see your book because Verne never receives it." },
      { title: "Never trained on", text: "Your manuscript will never be used to train a model, by Verne or anyone Verne works with." },
      { title: "Open exits", text: "Export to open formats any time. Copy the database file to back it up." },
      { title: "Works offline", text: "On a plane, in a cabin, during an outage. Your desk is always open." },
    ],
    sections: [
      {
        id: "why-local-first",
        heading: "Why does local-first matter for writers?",
        blocks: [
          { kind: "p", text: "Cloud-only tools ask you to trust that the service stays up, stays affordable, keeps your account in good standing, and keeps its promises about your data. For an unpublished novel, that’s a lot of trust. Local-first software flips the default: your copy is the real one, and anything networked is optional. More in [Why your manuscript should live on your own computer](/journal/local-first-writing)." },
        ],
      },
      {
        id: "backups",
        heading: "How should I back up an Asten project?",
        blocks: [
          { kind: "list", items: [
            "Take named snapshots in the desk before big revisions.",
            "Copy `asten.db` to a backup drive or your own cloud folder, ideally while Asten is closed.",
            "Export to Markdown or .docx now and then, so a readable copy exists outside Asten.",
            "Keep three copies, on two kinds of storage, with one somewhere else.",
          ] },
        ],
      },
      {
        id: "the-website-too",
        heading: "And this website?",
        blocks: [
          { kind: "p", text: "The same principle applies here. This site sets no cookies, runs no analytics, loads no third-party fonts or scripts, and shows no ads. The only thing it stores is your light or dark theme choice, in your own browser, if you press the moon. See the [privacy policy](/privacy)." },
        ],
      },
    ],
    faqs: [
      { q: "Where does Asten store my manuscript?", a: "Asten stores each project in a SQLite file called asten.db in your operating system’s application-data folder. It never leaves your computer unless you copy it." },
      { q: "Does Asten need an internet connection?", a: "No. Asten works fully offline. The optional Word and Google Docs add-ins need those apps, but your Asten data stays local." },
      { q: "Can Verne read my manuscript?", a: "No. Asten has no account, sync, or telemetry, so Verne never receives your manuscript." },
    ],
    demo: "local",
    art: "eye",
    related: ["/privacy", "/journal/local-first-writing", "/ai-policy", "/features/word-and-google-docs"],
  },
];
