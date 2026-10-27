import type { GlyphKey } from "../art/Glyphs";
import type { ArtKey, QA } from "./types";

export interface Audience {
  slug: string;
  path: string;
  name: string;
  glyph: GlyphKey;
  short: string;
  title: string;
  description: string;
  h1: string;
  lede: string;
  answer: string;
  pains: { title: string; text: string }[];
  helps: { title: string; text: string; href: string }[];
  scenario: { title: string; lines: string[] };
  honest: string[];
  faqs: QA[];
  art: ArtKey;
  related: string[];
}

export const audiences: Audience[] = [
  {
    slug: "novelists",
    path: "/for/novelists",
    name: "Novelists",
    glyph: "scroll",
    short: "For long manuscripts where details drift between chapters.",
    title: "Asten for novelists: continuity without losing your voice",
    description:
      "For novelists with a long draft: Asten remembers hair colors, deaths, and dates, and marks contradictions in red. It never writes your prose or trains on it.",
    h1: "Asten for novelists",
    lede: "In a long draft, details drift: an eye color changes, a dead man pours tea, a flashback lands in the wrong week. Asten keeps a record of what you settled and marks the pages that disagree with it.",
    answer:
      "Asten helps novelists keep a long manuscript consistent without handing their prose to an AI. You settle the facts that matter, and Asten marks later pages that disagree, such as a changed hair color, a character acting after their death, or scenes out of story order. It never writes or rewrites a sentence.",
    pains: [
      { title: "The style sheet you stopped updating", text: "You meant to keep a list of who has which scar. It stopped at chapter six." },
      { title: "Search-and-squint revision", text: "Checking every mention of a name by hand, hoping you didn’t miss the one that matters." },
      { title: "AI that wants the pen", text: "You tried an AI tool for help and it offered to write the scene. In a voice that wasn’t yours." },
    ],
    helps: [
      { title: "Settle it once", text: "Mira’s hair, Oren’s death, the day the harbor burned. Facts with sources.", href: "/features/canon" },
      { title: "Red in the margin", text: "When chapter 19 disagrees with chapter 2, you see both sides at once.", href: "/how-it-works" },
      { title: "Two clocks", text: "Flashbacks, frame stories, and parallel threads, with story time and reading order kept apart.", href: "/features/time" },
      { title: "A desk you can live in", text: "Binder, comments, find, page view, track changes, snapshots.", href: "/features/desk" },
    ],
    scenario: {
      title: "A Tuesday in revision",
      lines: [
        "You rewrite chapter 19 in a rush. Mira “pushes her black hair out of her eyes.”",
        "A red mark appears in the margin: CANON-E013. Chapter 2 settled her hair as auburn.",
        "You remember: she dyed it in chapter 12 to slip past the harbor guards.",
        "You give the auburn fact an end date in story time, add the black one from chapter 12 onward, and the mark disappears. The sentence never changed.",
      ],
    },
    honest: [
      "Asten reads only plain attributes from your prose (hair, eyes, age, location, alive or dead, relations), and only when a name is in the same sentence. Anything subtler, you settle yourself.",
      "It is a pre-alpha: release 0.0.1b for Windows and Linux, unsigned, with macOS later.",
      "It won’t typeset your book. Compile to .docx for your editor or publisher.",
    ],
    faqs: [
      { q: "Is Asten good for writing a novel?", a: "Asten is built for long fiction. It combines a manuscript desk (binder, comments, page view, track changes) with continuity checks that remember the facts you settle, without ever writing prose for you." },
      { q: "How is Asten different from Scrivener?", a: "Both organize a long manuscript in a binder. Asten adds a canon model and continuity checks that mark contradictions, and it is local-first with no account. It does not try to replicate every Scrivener compile option." },
    ],
    art: "scroll",
    related: ["/features/canon", "/journal/continuity-in-a-long-novel", "/features/desk", "/compare"],
  },
  {
    slug: "worldbuilders",
    path: "/for/worldbuilders",
    name: "Worldbuilders",
    glyph: "hills",
    short: "For fantasy and science-fiction writers whose world is bigger than the book.",
    title: "Asten for worldbuilders: a world that keeps its own rules",
    description:
      "For fantasy and sci-fi writers: keep places, cultures, systems, and creatures in an optional wall, atlas, and dossier, and let Asten catch contradictions in your canon.",
    h1: "Asten for worldbuilders",
    lede: "Asten gives your notes on places, cultures, and rules a home, connects the parts you settle to the manuscript, and keeps the rest out of the way until you need it.",
    answer:
      "Asten gives fantasy and science-fiction writers optional worldbuilding tools (a wall for scraps, an atlas for places, and a dossier grouped by kind) connected to a canon of settled facts. When the manuscript contradicts a settled fact, Asten marks it. Nothing in the world is required before you write.",
    pains: [
      { title: "The wiki that ate the novel", text: "Three years of lore, eleven pages of chapter one." },
      { title: "Facts scattered across apps", text: "Maps in one tool, a wiki in another, the manuscript in a third, and none of them talk." },
      { title: "Your own rules, broken", text: "Magic costs blood in book one. By book two, nobody bleeds." },
    ],
    helps: [
      { title: "Wall, atlas, dossier", text: "Scraps, places, and entries by kind: places, cultures, systems, creatures, items, questions, fragments.", href: "/features/world" },
      { title: "Spark → sketched → settled", text: "Ideas grow at their own pace. Only settled entries become canon.", href: "/features/world" },
      { title: "Canon checked against the manuscript", text: "Settled facts are checked against the manuscript, with sources.", href: "/features/canon" },
      { title: "Two clocks", text: "Histories, prophecies, and flashbacks with story time and reading order kept apart.", href: "/features/time" },
    ],
    scenario: {
      title: "The harbor at Veld",
      lines: [
        "You pin a spark on the wall: “Veld: harbor town, eats the reef’s fish, distrusts the capital.”",
        "Two chapters later you sketch it in the atlas, east of the reef.",
        "When the siege chapter needs it, you settle it. Asten mints the facts.",
        "Chapter 31 has Mira sail from Veld to the capital in two days. You declared that route as five, so CANON-E301 marks the journey.",
      ],
    },
    honest: [
      "The atlas is a rough map for pinning places. Travel-time checks work once you declare routes with a travel time.",
      "World rules (“magic costs blood”) are written as plain patterns in the invariants panel and checked against the scene trace.",
      "Story time can count days, use Gregorian dates, or use a fictional calendar whose months you define.",
    ],
    faqs: [
      { q: "Can Asten replace World Anvil or Campfire?", a: "Asten is a writing app with optional worldbuilding, not a dedicated worldbuilding platform. Its world tools are lighter, but they connect directly to your manuscript’s continuity checks and stay on your own computer." },
      { q: "Does Asten support magic systems and invented cultures?", a: "Yes. Asten’s dossier has kinds for systems, cultures, creatures, places, items, questions, and fragments. Settled entries become facts that Asten can check." },
    ],
    art: "pyramid",
    related: ["/features/world", "/journal/do-you-need-a-story-bible", "/features/canon", "/roadmap"],
  },
  {
    slug: "screenwriters",
    path: "/for/screenwriters",
    name: "Screenwriters",
    glyph: "mouth",
    short: "For scripts with timelines, reveals, and a cast that has to stay straight.",
    title: "Asten for screenwriters: Fountain in, continuity checked",
    description:
      "For screenwriters: import and compile Fountain, keep story time and reading order apart for non-linear scripts, and catch continuity slips before the table read.",
    h1: "Asten for screenwriters",
    lede: "Cold opens, flash-forwards, “three weeks earlier.” Asten keeps the order audiences see scenes separate from the order events happen, imports and compiles Fountain and Final Draft, and marks story time that steps backward inside a run of scenes you declared continuous.",
    answer:
      "Asten supports screenwriters by importing and compiling Fountain, the plain-text screenplay format, and by tracking story time and reading order as two clocks for non-linear scripts. It marks continuity contradictions in your settled facts. It is not a full screenplay formatting suite.",
    pains: [
      { title: "Non-linear structure", text: "A cold open, two timelines, and a reveal that depends on both lining up." },
      { title: "Character drift across drafts", text: "The limp moved from the left leg to the right somewhere in draft four." },
      { title: "Format lock-in", text: "A script trapped in one app’s file format." },
    ],
    helps: [
      { title: "Fountain in and out", text: "Import and compile the open plain-text screenplay format.", href: "/features/word-and-google-docs" },
      { title: "Two clocks", text: "Screen order and story order, kept separate. CANON-E201 only inside a run you declared continuous.", href: "/features/time" },
      { title: "Canon checks", text: "Scars, cars, and who knows what, settled once.", href: "/features/canon" },
      { title: "Local and private", text: "An unproduced script stays on your machine.", href: "/features/local-first" },
    ],
    scenario: {
      title: "Three weeks earlier",
      lines: [
        "Your script opens on the fire, then cuts to “THREE WEEKS EARLIER.”",
        "The cut steps backward in story time. It is outside the run you declared continuous, so Asten says nothing.",
        "In act three a scene you dated day 15 lands before day 14, and this time you didn’t mean it.",
        "You swap the scenes in the binder. Fountain compiles clean.",
      ],
    },
    honest: [
      "Asten is not a full screenplay editor: no production revisions or page locking.",
      "Fountain and Final Draft (.fdx) are supported for import and compile.",
      "A flashback is never marked on its own. CANON-E201 appears only inside a run of scenes you declared continuous.",
    ],
    faqs: [
      { q: "Does Asten support Fountain?", a: "Yes. Asten imports and compiles Fountain, the plain-text screenplay format, alongside Final Draft (.fdx), Markdown, plain text, HTML, and .docx." },
      { q: "Can Asten replace Final Draft?", a: "No. Asten is not a production screenplay suite: it exchanges .fdx files with Final Draft, but has no page locking or revision colors. It is for keeping a script’s story and continuity straight." },
    ],
    art: "moon",
    related: ["/features/time", "/journal/story-time-and-reading-order", "/features/word-and-google-docs", "/roadmap"],
  },
  {
    slug: "fan-fiction",
    path: "/for/fan-fiction",
    name: "Fan-fiction writers",
    glyph: "heart",
    short: "For writers keeping their own version of an existing canon.",
    title: "Asten for fan-fiction writers: keep your canon straight",
    description:
      "For fan-fiction writers: settle canon facts (and your own AU changes), get marks when a chapter drifts, and keep your writing on your own computer, never trained on.",
    h1: "Asten for fan-fiction writers",
    lede: "Asten uses canon the way fandom does. Settle what is true in your version, keep source canon you have changed on file as contested, and get a mark when chapter 40 contradicts chapter 4.",
    answer:
      "Asten lets fan-fiction writers record canon as settled facts, keep source canon they’ve changed on file as contested, and get notified when the story contradicts what they settled. Everything stays in a file on your own computer; Asten never generates prose and never trains on your writing.",
    pains: [
      { title: "Long fics, longer memories", text: "A 300,000-word work-in-progress and readers who notice everything." },
      { title: "AU rules you invented", text: "You changed one thing about canon. Now you have to remember its consequences." },
      { title: "AI scraping worries", text: "You want your writing to stay yours, not become someone’s training data." },
    ],
    helps: [
      { title: "Canon, literally", text: "Settle facts from the source and from your own story.", href: "/features/canon" },
      { title: "Contested, not deleted", text: "Keep source canon on file as contested. Your AU facts are the ones Asten checks.", href: "/glossary#contested" },
      { title: "Local-first", text: "Your fic lives in a file on your computer. No account, no upload, no training.", href: "/features/local-first" },
      { title: "Export anywhere", text: "Compile to HTML or Markdown for posting.", href: "/features/word-and-google-docs" },
    ],
    scenario: {
      title: "The AU where he lived",
      lines: [
        "In source canon, Captain Oren dies. In your AU, he doesn’t.",
        "You keep the source fact “Oren · status · dead” on file as contested, so it never raises a mark, and settle “alive” for your story.",
        "Months later an old outline note settles him as dead again. Asten marks the clash (CANON-E013) before you post the chapter.",
        "Throughout, the fic stays in a file on your laptop.",
      ],
    },
    honest: [
      "Asten doesn’t import a franchise’s canon for you. You settle the facts that matter.",
      "There’s no posting integration; compile to HTML or Markdown and post as usual.",
      "It is a pre-alpha (release 0.0.1b); expect rough edges.",
    ],
    faqs: [
      { q: "Is Asten good for writing fan fiction?", a: "Yes. Asten’s canon model fits fan fiction naturally: settle the facts of your version, keep changed source canon on file as contested, and get marks when your canon contradicts itself. Your writing stays on your computer." },
      { q: "Will Asten use my fan fiction to train AI?", a: "No. Asten keeps your writing in a local file that Verne never receives, and Verne’s AI policy forbids training on manuscripts." },
    ],
    art: "ibis",
    related: ["/features/canon", "/features/local-first", "/ai-policy", "/glossary"],
  },
  {
    slug: "ai-tinkerers",
    path: "/for/ai-tinkerers",
    name: "AI tinkerers",
    glyph: "kit",
    short: "For people who use AI daily and want a writing tool that draws a clear line.",
    title: "Asten for AI tinkerers: a clear line, a plain architecture",
    description:
      "For AI enthusiasts who write: Asten is a local-first, deterministic canon engine with stable diagnostic codes, a loopback API, and a clear rule that AI edits and never writes.",
    h1: "Asten for AI tinkerers",
    lede: "If you use models every day, you know where they are unreliable. Asten’s checks are deterministic rules. The assistant is optional: bring your own key or run a local model, read the exact request before it is sent, and know that it never writes your prose and your manuscript is never used for training.",
    answer:
      "Asten is a local-first writing environment with a deterministic canon engine: Rust, SQLite, a loopback HTTP API on 127.0.0.1:4317, and stable diagnostic codes. The checks run no model: perception is deterministic. The optional assistant in the desktop app (bring your own key, or a local model) only explains, searches and proposes; it never writes prose and never trains on manuscripts.",
    pains: [
      { title: "Voice flattening", text: "Every AI draft sounds like every other AI draft." },
      { title: "Confident nonsense", text: "A model “remembers” a plot point that never happened." },
      { title: "Opaque data flows", text: "Where did your chapter go when you pasted it in?" },
    ],
    helps: [
      { title: "Deterministic first", text: "Rules you can read, test, and argue with. Same project, same marks.", href: "/features/compiler" },
      { title: "Stable codes", text: "CANON-E013 to E601, NAME-W, PLOT-I, CRAFT-H. Searchable, suppressible, documented.", href: "/glossary#diagnostic" },
      { title: "Plain architecture", text: "Rust crates, SQLite, Axum on loopback, React, Tauri.", href: "/features/compiler" },
      { title: "A written AI policy", text: "Proposals only, local first, your key, full switch-off, no training.", href: "/ai-policy" },
    ],
    scenario: {
      title: "How the assistant fits",
      lines: [
        "You finish a chapter and ask the assistant what the draft implies. It reads what you attach and proposes three facts: “Mira · hair_color · black?”",
        "Each proposal cites the sentence it came from. None of them is canon yet.",
        "You confirm one, reject one, and mark one as a lie Mira tells. Only the confirmed one joins canon.",
        "The compiler, still deterministic, does the checking. The model never touches your prose.",
      ],
    },
    honest: [
      "The assistant is new in 0.0.1b and has only been tested against mock providers that speak each vendor’s wire format; live providers need your key and are untested by Verne. The checks that judge contradictions stay rule-based.",
      "Asten is open source under the MIT License; the source is on GitHub.",
      "The local API binds to loopback by default and answers only to paired clients. Don’t expose it to a network without TLS and an owner passphrase.",
    ],
    faqs: [
      { q: "Does Asten run a local LLM?", a: "Asten does not bundle one. Its checks are deterministic rules. The optional assistant can call a local model through Ollama (or any OpenAI-compatible endpoint on your machine), or a cloud provider with your own key and consent, and everything AI-related switches off in one place." },
      { q: "Does Asten have an API?", a: "Asten runs a local HTTP API on 127.0.0.1:4317 that its desk and add-ins use. It is a loopback API on your own machine, not a public cloud API." },
      { q: "Is Asten open source?", a: "Yes. Asten is released under the MIT License, and the source is on GitHub (ArefinAlter/asten). Its rules and diagnostic codes are documented on this site." },
    ],
    art: "palette",
    related: ["/ai", "/ai-policy", "/features/compiler", "/journal/ai-editor-not-writer"],
  },
];
