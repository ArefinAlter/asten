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
    h1: "A manuscript editor for long projects",
    lede: "Chapters and scenes in a binder, a clean page in the middle, and Asten’s red marks in the margin. Asten adds marks beside the text and never changes it.",
    answer:
      "The Asten desk is a manuscript editor for long-form writing. It has a binder of chapters, scenes, notes, and sources; a rich-text page; comments; find; word counts; a letter-sized page view; track-change marks; and named snapshots. Diagnostics sit in the margin and never rewrite your text.",
    points: [
      { title: "A binder", text: "Manuscript, chapters, scenes, notes, and sources in one tree. Exclude a scene from compile without deleting it." },
      { title: "The page", text: "Headings, emphasis, and the formatting a manuscript needs. Literata on warm paper. Nothing animated near your words." },
      { title: "Comments and track changes", text: "Leave notes to yourself or an editor. Insertions and deletions show as marks you can read back." },
      { title: "Page view", text: "A letter-sized sheet with a running header, so you can feel how long the chapter is. It is not a typesetter." },
      { title: "Find, counts, snapshots", text: "Find next and previous, live word counts, and named snapshots you can restore without any cloud." },
      { title: "Red in the margin", text: "When a page disagrees with your canon, the mark appears beside the sentence, with the evidence one click away." },
    ],
    sections: [
      {
        id: "what-the-desk-is-for",
        heading: "What is the desk for?",
        blocks: [
          { kind: "p", text: "The desk is where the book gets written. Everything else in Asten (canon, cast, time, world) exists to serve the page, and none of it is required before you start typing." },
          { kind: "p", text: "Think of it as a writer’s subset of a word processor: the parts you use every day on a manuscript, without the layout features you only need for a printed brochure. When the book needs to leave, you [compile](/features/word-and-google-docs) it to Markdown, Fountain, HTML, Word, EPUB, or Final Draft." },
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
      { q: "Can I import an existing manuscript into Asten?", a: "Yes. Asten imports Markdown, plain text, Fountain, Final Draft (.fdx), HTML, and .docx. The .docx import is deliberately conservative: it keeps your text and basic structure, not complex layout." },
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
    h1: "Check the manuscript against settled facts",
    lede: "Canon is what your story says is true. Settle a fact (*Mira · hair · auburn*) and Asten weighs every later page against it. If chapter 19 says black, you get a red mark with both sides, never a rewrite.",
    answer:
      "Canon checks in Asten compare your manuscript against facts you have settled. A fact is an entity, a predicate, and a value, with a status, a modality, an optional span of story time, and a source. When two confirmed facts disagree, or the page says something different from what you settled, Asten marks it in red with code CANON-E013 and shows both sides. Asten reads plain attributes from your prose with fixed rules, not a model, and treats dialogue, belief, memory, and dreams as separate contexts.",
    points: [
      { title: "Facts with a source", text: "Every fact records where it came from, so a mark can show you the sentence that settled it." },
      { title: "Statuses you control", text: "Proposed, confirmed, deprecated, rejected. Confirmed against confirmed is an error; the page against your canon is a warning; the page against itself is information." },
      { title: "Room for lies and legends", text: "Dialogue, belief, memory, dream, vision, and frame stories are separate contexts. What a character says or dreams never contradicts the world, and contested facts stay quiet." },
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
            { term: "Status", text: "Proposed, confirmed, deprecated, or rejected. Only confirmed facts produce errors." },
            { term: "Modality", text: "Established, inferred, contested, or author override. Contested facts stay quiet." },
            { term: "Story time", text: "An optional valid-from and valid-to, for facts that change during the book." },
            { term: "Context", text: "World, narration, dialogue, belief, memory, dream, vision, frame, or hypothetical. Only world and reliable narration count against canon." },
          ] },
        ],
      },
      {
        id: "what-a-mark-looks-like",
        heading: "What does a mark look like?",
        blocks: [
          { kind: "p", text: "A mark is short and specific. It names the rule, quotes both sides, and lists what you can do. It does not tell you which side is right, because only you know that." },
          { kind: "diagnostic", code: "CANON-E013", title: "Mira’s hair was settled as auburn.", lines: ["Settled in chapter 2: Mira · hair_color · auburn", "Chapter 19 says: “Mira pushed her black hair out of her eyes.”", "Keep my sentence · Update canon · Add a change in story time · Silence this rule"] },
          { kind: "p", text: "Two confirmed facts that disagree over overlapping story time are an **error**. The page disagreeing with a confirmed fact is a **warning**: worth a look, not a verdict. The page disagreeing with itself is **information**. Every mark reads as a numbered chain (①②③), names the statement most likely at fault, and lists repairs you own: add a change event, mark the scene a memory, retire the fact, or keep your sentence." },
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
            "Let the page propose. Asten reads names and plain attributes from your prose (hair, eyes, height, age, where someone is, alive or dead, who is married to whom) and lists them as proposals you confirm or reject.",
          ] },
          { kind: "callout", tone: "rubric", title: "Asten reads the page, not your mind", text: "The reading is deterministic: fixed rules over words and cues, no model. It attributes an observation to a name in the same sentence, so a pronoun on its own is not weighed. Anything inside quotation marks counts as something a character *says*, not as world truth. Nothing Asten reads becomes canon until you confirm it." },
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
      { q: "How does Asten know a sentence contradicts my canon?", a: "Asten compares the facts you have settled with each other, and compares them with plain attributes it reads from your prose (hair and eye color, height, age, location, alive or dead, relations), each attributed to a name in the same sentence. It uses deterministic rules, not an AI model." },
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
    title: "Cast: characters, places, and other entities",
    description:
      "The Asten cast keeps characters, places, items, and events as entities with aliases and confirmed facts. Name someone when the draft needs them, not before.",
    h1: "Characters and other entities, added when you need them",
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
          { kind: "p", text: "Egyptian scribes enclosed royal names in a *cartouche*, an oval ring that marks a name as important. Asten borrows the idea: an entity’s name appears in a cartouche wherever it is referenced. The example below draws a name in one, using a playful alphabet rather than real hieroglyphic spelling." },
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
      "Asten keeps two clocks: when events happen (story time) and when readers meet them (reading order). It marks story time that steps backward inside a run of scenes you declared continuous.",
    h1: "Story time and reading order, kept apart",
    lede: "Every story has two timelines: the order things happen, and the order readers find out. Asten keeps both, and marks the moments they cross so you can tell a flashback from a slip.",
    answer:
      "Asten tracks story time (when an event happens in the world) and reading order (when the reader meets it) as two separate clocks. A flashback is not a mistake: Asten argues only inside a run of scenes you declared continuous (CANON-E201), when the dates you gave cannot all be true (CANON-E202), or when an effect is seen before its cause (CANON-E203). Story time can be a day count, a Gregorian date, or a calendar you define.",
    points: [
      { title: "Story time", text: "When it happens in the world: day 14, 1890-03-02, the winter of 1890, three days after the siege. Day counts, Gregorian dates, or a fictional calendar with your own months." },
      { title: "Reading order", text: "When the reader meets it: chapter 3, scene 2." },
      { title: "CANON-E201 to E203", text: "Story time stepping backward inside a run you declared continuous; dates that cannot all be true; an effect seen before its cause." },
      { title: "CANON-E101", text: "Someone acting after a settled death gets a mark, unless the scene is a memory, a flashback, or placed earlier in story time." },
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
          { kind: "diagnostic", code: "CANON-E201", title: "Inside the run “The siege”, story time steps backward.", lines: ["① The harbor burns · day 15 · chapter 9", "② Oren sails · day 14 · chapter 10", "Reorder the binder · Retcon a date · Mark chapter 10 a flashback · Keep my order"] },
          { kind: "p", text: "Asten compares scenes only inside a run you declared continuous, so an ordinary flashback raises nothing. Give scenes dates, or relative placements such as “three days after the siege”, and Asten solves them together: if no placement satisfies them all, CANON-E202 lists the dates in the knot. If a state is seen before the event that causes it, CANON-E203 says so." },
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
      { q: "Does Asten treat every flashback as an error?", a: "No. Asten compares story time only inside a run of scenes you declared continuous, and a scene marked as a memory or flashback is never weighed against the world. CANON-E201 appears only when a run you said was continuous steps backward." },
      { q: "Do I have to date every scene in Asten?", a: "No. Asten only compares scenes that have both a story time and a reading order. Undated scenes are never marked." },
      { q: "Can Asten use a fantasy calendar?", a: "Yes. A project’s clock can count days, use Gregorian dates such as “1890-03-02” or “the winter of 1890”, or use a fictional calendar whose months you define. Relative placements such as “three days after the siege” resolve against the scene they name." },
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
    h1: "Optional world notes, connected to canon",
    lede: "Maps, factions, histories, and rules get a place in Asten, and none of them has to exist before you write.",
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
      { q: "Does Asten’s atlas check travel times?", a: "Yes, once you declare routes. Give two places a route with a travel time, and CANON-E301 marks a journey that does not fit the story time between two scenes; CANON-E302 marks someone placed in two places at once." },
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
      "Under Asten’s red marks is a deterministic engine with stable codes, four severities, explanation chains, repairs you own, suppressions, and a verifiable history of your edits.",
    h1: "Deterministic checks with stable codes",
    lede: "Programmers have compilers that read code and point at the line that breaks. Asten does the same for a story’s facts, with deterministic rules, stable codes, and evidence for every mark.",
    answer:
      "Asten’s compiler is a deterministic rule engine written in Rust. It reads your settled facts, your timeline, and the page itself, and emits diagnostics with stable codes (CANON-E013 to E602, NAME-W001 to W004, PLOT-I001 and I002, and optional CRAFT hints) at four severities: error, warning, information, and hint. Each mark explains itself in a numbered chain and offers repairs. It runs entirely on your machine and never changes your text.",
    points: [
      { title: "Deterministic", text: "Same project, same marks. No model, no randomness, no “it depends on the prompt”." },
      { title: "Stable codes", text: "CANON-E013 always means an attribute contradiction. Search it, discuss it, suppress it." },
      { title: "Four severities", text: "Errors for confirmed against confirmed, warnings for the page against canon, information for the page against itself, hints for optional craft notes." },
      { title: "Evidence attached", text: "Every mark points at a span in a document, reads as a numbered chain (①②③), and names the statement most likely at fault." },
      { title: "Repairs you own", text: "Add an event, mark the scene a memory, retire a fact, silence the mark, or keep your sentence. Bisect finds the edit that raised a mark." },
      { title: "Append-only history", text: "Document operations are recorded, and named snapshots restore a revision on your own disk." },
      { title: "Stable block ids", text: "Paragraphs carry identities, so a mark can follow its sentence when you edit around it." },
    ],
    sections: [
      {
        id: "the-rules-today",
        heading: "Which rules ship today?",
        blocks: [
          { kind: "table", head: ["Code", "Pack", "What it notices", "Default"], rows: [
            ["CANON-E013", "Scribe", "Two values for a one-valued attribute at the same story time in the same context, with no change event between them", "Always on"],
            ["CANON-E014 / E015", "Scribe", "An immutable attribute (a birthplace) changed; an attribute on the wrong kind of thing", "Always on"],
            ["CANON-E101 / E102 / E103", "Lifecycle", "Acting after a settled death; appearing before birth; an item used after it was destroyed", "On"],
            ["CANON-E201 / E202 / E203", "Clocks", "Story time stepping backward inside a continuous run; dates that cannot all be true; an effect before its cause", "On"],
            ["CANON-E301 / E302", "Atlas", "A journey that does not fit the time between scenes; in two places at once", "On"],
            ["CANON-E401", "Knowledge", "A character using knowledge no event has given them yet", "On"],
            ["CANON-E501 / E502 / E503", "Relations", "Their own ancestor; two of an exclusive relation at once; a parent too young", "On"],
            ["CANON-E601 / E602", "Invariants", "A rule you wrote in the invariants panel fails on the scene trace; a pack rule fails", "On"],
            ["NAME-W001 to W004", "Scribe", "A name not in the cast; named before being introduced; confusable names; a fact pointing at a removed entity", "Always on"],
            ["PLOT-I001 / I002", "Invariants", "A setup not yet paid off; set up and never used by the end", "On"],
            ["CRAFT-H001 / H002", "Diction watch", "Filter words such as “very” and “really”, as optional notes", "Off"],
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
          { kind: "p", text: "For the curious: the engine is a set of Rust crates. A story IR holds entities, facts, events, scenes, and contexts; a block model gives every paragraph a stable id and a hash-chained history so marks follow their sentence through edits; perception reads the page with Unicode folding and dictionary matching, no model; the analyses are an event calculus, a lifecycle typestate, a temporal network, shortest routes, and a small temporal-logic checker for your own rules. Storage is SQLite. An Axum server exposes a local API on `127.0.0.1:4317` that answers only to paired clients. The interface is React and TypeScript; the Tauri desktop shell embeds the same API. More in [For AI tinkerers](/for/ai-tinkerers)." },
        ],
      },
    ],
    faqs: [
      { q: "Is the Asten compiler an AI?", a: "No. The Asten compiler is a deterministic rule engine. It produces the same marks for the same project every time, and it runs entirely on your computer." },
      { q: "What do Asten’s diagnostic codes mean?", a: "CANON-E0xx codes are attribute clashes, E1xx lifecycle, E2xx the two clocks, E3xx travel and place, E4xx knowledge, E5xx relations and genealogy, E6xx your own rules. NAME-W codes are name hygiene, PLOT-I codes are setup ledgers, and CRAFT-H codes are optional diction hints. The [glossary](/glossary) defines the common ones." },
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
      "Asten’s checks come in packs: Scribe (always on), Lifecycle, Clocks, Relations, Knowledge, Atlas, Invariants, and the optional Diction watch. Enable what helps you. No craft doctrine is ever a default.",
    h1: "Optional groups of rules",
    lede: "Asten doesn’t believe in one right way to write. Its checks come in packs you turn on and off, and craft advice stays switched off until you ask for it.",
    answer:
      "Method packs are optional groups of rules in Asten. Eight ship today: Scribe (the core canon check and name hygiene, always on), Lifecycle, Clocks, Relations, Knowledge, Atlas, Invariants (your own world rules), and Diction watch (optional filter-word hints, off by default). Packs add marks; they never change your manuscript.",
    points: [
      { title: "Scribe", text: "The core canon compiler. Attribute contradictions and name hygiene, CANON-E013 to E015 and NAME-W001 to W004. Always on." },
      { title: "Lifecycle", text: "A settled death followed by a later act, a birth after a first appearance, an item used after it was destroyed. CANON-E101 to E103." },
      { title: "Clocks", text: "Story time inside continuous runs, dates that cannot all be true, effects before causes. CANON-E201 to E203. Silent until scenes are dated." },
      { title: "Relations, Knowledge, Atlas", text: "Genealogy and exclusive relations (E501 to E503), who knows what and when (E401), routes and travel time (E301, E302)." },
      { title: "Invariants", text: "Rules you write yourself in plain patterns (“after X, never Y”), checked against the scene trace. CANON-E601, and setup ledgers PLOT-I001 and I002." },
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
      { q: "Can I turn off Asten’s checks?", a: "Yes. Every pack except Scribe can be switched off, and a project’s strictness can be set to quiet (settled against settled only), standard, or thorough. Scribe, the core attribute check, stays on, but any individual mark can be silenced with a reason." },
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
    short: "Compile to Markdown, Fountain, HTML, .docx, EPUB, and Final Draft. Optional Word and Docs add-ins talk to your own machine.",
    title: "Word and Google Docs: export, import, and add-ins",
    description:
      "Asten compiles to Markdown, plain text, Fountain, HTML, .docx, EPUB 3, and Final Draft (.fdx), and imports all but EPUB back. Optional Word and Google Docs add-ins show marks from your local Asten.",
    h1: "Export, import, and add-ins for Word and Google Docs",
    lede: "Your editor wants a .docx. Your co-writer lives in Google Docs. Asten sends the manuscript out and takes it back, and two small add-ins let those apps show Asten’s marks without your canon leaving your machine.",
    answer:
      "Asten compiles a manuscript to Markdown, plain text, Fountain, HTML, .docx, EPUB 3, or Final Draft (.fdx), and imports every one of those except EPUB. PDF comes from printing the HTML. Optional Word and Google Docs add-ins pair once with a six-digit code shown in the desk, then list your local projects, show the Scales, and can file a selected passage into Asten; they talk only to the Asten API on 127.0.0.1:4317 and hold a token you can revoke.",
    points: [
      { title: "Compile", text: "Markdown, plain text, Fountain, HTML, .docx, EPUB 3, and Final Draft (.fdx). Excluded scenes stay out; the EPUB’s table of contents follows your binder." },
      { title: "Import", text: "Bring drafts home from the same formats, EPUB aside. The .docx import is conservative on purpose." },
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
            ["EPUB 3", "Yes", "No", "E-readers and reading apps"],
            ["Final Draft (.fdx)", "Yes", "Yes", "Final Draft and other screenwriting tools"],
          ] },
        ],
      },
      {
        id: "how-the-add-ins-work",
        heading: "How do the add-ins work?",
        blocks: [
          { kind: "p", text: "The add-ins are thin windows onto your own computer. They call the Asten API at `127.0.0.1:4317` (that address only exists on your machine) to list projects, show the Scales, and file a selected passage into a project. The API answers only to clients you have paired: the first time a pane opens, it asks for a six-digit code from the desk’s Guests panel, and you can revoke that pairing at any time. Microsoft and Google already have the document you opened; Asten doesn’t send your canon to them or to Verne." },
          { kind: "callout", tone: "note", title: "Sideload only, for now", text: "The add-ins are installed by hand (“sideloaded”) and are not in the Microsoft or Google stores. Early-access writers get instructions." },
        ],
      },
    ],
    faqs: [
      { q: "Can Asten export to Word?", a: "Yes. Asten compiles a manuscript to .docx, as well as Markdown, plain text, Fountain, HTML, EPUB, and Final Draft (.fdx)." },
      { q: "Does the Asten Google Docs add-on send my book to Verne?", a: "No. The Asten Docs sidebar talks only to the Asten API running on your own computer at 127.0.0.1:4317. Verne never receives your text." },
      { q: "Can Asten export EPUB or PDF?", a: "EPUB, yes: Asten compiles the binder to an EPUB 3 file whose table of contents follows your chapters and scenes. PDF comes from printing Asten’s HTML output." },
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
    title: "Local-first: your manuscript stays on your computer",
    description:
      "Asten keeps your whole project in one SQLite file on your computer. No account, no sync, no telemetry, and no training on your words. Export anytime.",
    h1: "Your manuscript stays on your computer",
    lede: "Asten has no account, no cloud storage, and no telemetry, and your manuscript is not used for training. Your project is a single file on your own disk, and you can export it to open formats at any time.",
    answer:
      "Asten is local-first: every project lives in a SQLite database file on your computer (asten.db, in the operating system’s app-data folder). There is no account, no sync service, and no telemetry, and Verne never receives your manuscript. You can export to Markdown, plain text, Fountain, HTML, .docx, EPUB, or Final Draft (.fdx) at any time.",
    points: [
      { title: "One file", text: "Projects, scenes, facts, comments, snapshots, and marks live in `asten.db` on your disk." },
      { title: "No account", text: "Nothing to sign up for, nothing to log into, nothing to lose access to." },
      { title: "No sync, no telemetry", text: "Asten doesn’t phone home. Verne can’t see your book because Verne never receives it." },
      { title: "Never trained on", text: "Your manuscript will never be used to train a model, by Verne or anyone Verne works with." },
      { title: "Open exits", text: "Export to open formats any time. Copy the database file to back it up." },
      { title: "Works offline", text: "The desktop app needs no network connection. The Word and Docs add-ins depend on those apps." },
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
          { kind: "p", text: "The same principle applies here. This site sets no cookies, runs no analytics, loads no third-party fonts or scripts, and shows no ads. The only thing it stores is your light or dark theme choice, in your own browser, if you change it. See the [privacy policy](/privacy)." },
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
