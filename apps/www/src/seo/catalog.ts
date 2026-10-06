export const site = {
  origin: "https://asten.vercel.app",
  publisher: "Verne",
  product: "Asten",
  updated: "2026-10-06",
  updatedLabel: "6 October 2026",
  repository: "https://github.com/ArefinAlter/asten",
  research: "https://github.com/ArefinAlter/asten-research-ingest",
  tagline:
    "Asten is a local-first writing environment by Verne. It notices contradictions in world, character, and time, and it leaves the sentence yours.",
} as const;

export interface Block {
  heading: string;
  paragraphs: string[];
}

export interface Question {
  question: string;
  answer: string;
}

export interface PageRecord {
  path: string;
  nav: string | null;
  title: string;
  description: string;
  h1: string;
  answer: string;
  kind: "story" | "record" | "legal";
  blocks: Block[];
  questions?: Question[];
}

export const pages: PageRecord[] = [
  {
    path: "/",
    nav: "Notice",
    kind: "story",
    title: "Asten — a writing environment by Verne",
    description:
      "Asten is a local-first writing environment by Verne. It notices contradictions in world, character, and time, and it does not write the manuscript.",
    h1: "Asten notices a contradiction and leaves the sentence yours.",
    answer:
      "Asten is a Verne product: a local-first writing environment named for Thoth in another tongue. It can point at a contradiction. It does not pick up the pen.",
    blocks: [
      {
        heading: "What Asten is",
        paragraphs: [
          "Asten is a desk for a manuscript, plus a compiler for facts you have settled. The files stay on the machine. There is no account.",
          "World notes are optional. A gardener can start a scene. An architect can pin a harbor first. Neither path is the gate.",
        ],
      },
      {
        heading: "What Asten refuses",
        paragraphs: [
          "It is not a generator, not a typesetter, and not a clone of Word or InDesign. Word and Google Docs remain guests.",
        ],
      },
    ],
  },
  {
    path: "/environment",
    nav: "Desk",
    kind: "story",
    title: "The desk — Asten by Verne",
    description:
      "Asten’s desk is a manuscript editor with a binder, comments, find, a letter-sized page view, and track changes. Paginate only when you ask.",
    h1: "A writing environment, not a typesetter.",
    answer:
      "The Asten desk is a binder and a page: headings, emphasis, comments, find, counts, a letter view, and track changes. It is not a layout program.",
    blocks: [
      {
        heading: "What the desk includes",
        paragraphs: [
          "Scenes live in a chapter tree. The page uses Georgia on paper. Diagnostics sit beside the sentence.",
          "Page view is a letter sheet with a running header, for the feel of length. Print houses still receive a file. Asten does not replace a typesetter.",
        ],
      },
    ],
  },
  {
    path: "/continuity",
    nav: "Canon",
    kind: "story",
    title: "Canon — how Asten checks continuity",
    description:
      "Asten treats canon as settled facts with modality, time, and a source. Contradictions are diagnostics you can suppress. The author override wins.",
    h1: "A fact is established, inferred, contested, or overridden.",
    answer:
      "Canon in Asten is a fact with an entity, a predicate, a value, a modality, a status, a valid interval, and a source. The compiler compares settled facts. It does not invent them.",
    blocks: [
      {
        heading: "What is checked",
        paragraphs: [
          "Confirmed attributes that clash raise CANON-E013. A life that continues after a settled death raises CANON-E101. Reading order that fights story time raises CANON-E201.",
          "You can suppress a rule. An author override wins. Craft diction stays in an optional pack.",
        ],
      },
    ],
  },
  {
    path: "/cast",
    nav: "Cast",
    kind: "story",
    title: "Cast — characters in Asten",
    description:
      "Asten keeps characters as canon entities with facts you confirm. The cast is a view of the book, not a form you must finish before the first scene.",
    h1: "Name someone when the draft needs them.",
    answer:
      "The cast is a list of entities — characters and the rest — with the facts you have confirmed. It is not a character-sheet gate.",
    blocks: [
      {
        heading: "How a person enters the book",
        paragraphs: [
          "You can write the scene first. You can name the person when a fact is worth keeping. Settling a fact is what the compiler can see.",
        ],
      },
    ],
  },
  {
    path: "/time",
    nav: "Time",
    kind: "story",
    title: "Time — story time and reading order in Asten",
    description:
      "Asten keeps two clocks: story time and reading order. It warns when a later page happens earlier in the tale, and it does not force a timeline before you draft.",
    h1: "Two clocks, and neither is mandatory.",
    answer:
      "Asten records story time and reading order separately. CANON-E201 fires when those clocks disagree. You can draft before you date a scene.",
    blocks: [
      {
        heading: "What the timeline is for",
        paragraphs: [
          "Travel, delay, and a death all need a when. The timeline is a view of events you chose to place. It is not a prerequisite for chapter one.",
        ],
      },
    ],
  },
  {
    path: "/world",
    nav: "World",
    kind: "story",
    title: "World — optional notes in Asten",
    description:
      "Worldbuilding in Asten is a wall, an atlas, and a dossier you can ignore. Settling a place mints a fact. Starting a scene never waits on a wiki.",
    h1: "Most of the world stays underwater.",
    answer:
      "World tools in Asten are optional. A scrap, a place, or a rule can sit on the wall. Settling it mints a fact. Nothing in the world is required before you write.",
    blocks: [
      {
        heading: "What stays optional",
        paragraphs: [
          "Places, cultures, and invented systems can be stored when you want them consistent. The novel shows the tip. The rest can remain notes.",
          "A gardener starts a scene. An architect may pin a harbor first. Asten does not rank those temperaments.",
        ],
      },
    ],
  },
  {
    path: "/engine",
    nav: "Engine",
    kind: "story",
    title: "The compiler and provenance — Asten",
    description:
      "Asten keeps append-only document operations, stable block ids, snapshots, and a compiler over settled facts. History stays on the machine.",
    h1: "Stable nodes, an append-only history, named snapshots.",
    answer:
      "Every save can write an operation. Block identifiers live in the document. Snapshots restore a revision without a cloud. The compiler reads settled facts and emits diagnostics.",
    blocks: [
      {
        heading: "What the compiler reports",
        paragraphs: [
          "CANON-E013 is an attribute contradiction. CANON-E101 is a life after a settled death. CANON-E201 is reading order against story time. Diction hints ship in an optional pack and can be turned off.",
        ],
      },
    ],
  },
  {
    path: "/packs",
    nav: "Packs",
    kind: "story",
    title: "Method packs — Asten",
    description:
      "Asten method packs are optional rules for craft, clocks, and diction. The core compiler stays small. You enable a pack; it does not become a doctrine.",
    h1: "A pack is a set of rules you can decline.",
    answer:
      "Method packs add checks — scribe, lifecycle, clocks, diction — without changing the manuscript by themselves. Disable a pack and its hints go quiet.",
    blocks: [
      {
        heading: "What a pack is not",
        paragraphs: [
          "A pack is not a house style you must adopt, and it is not an AI that rewrites the paragraph. It notices. You decide.",
        ],
      },
    ],
  },
  {
    path: "/interop",
    nav: "Guests",
    kind: "story",
    title: "Word and Docs as guests — Asten",
    description:
      "Asten compiles to Markdown, Fountain, HTML, and Word, and imports prose back. Optional Word and Docs add-ins file a selection to the local API. Canon stays on the machine.",
    h1: "Word and Docs remain guests.",
    answer:
      "Asten sends a manuscript out and takes prose back. It does not replace Word or Google Docs. Optional add-ins show a problem from the local compiler. They do not move canon to Microsoft or Google.",
    blocks: [
      {
        heading: "Formats",
        paragraphs: [
          "Compile targets include Markdown, plain text, Fountain, HTML, and docx. Import accepts the same family. Nested tables and fields are not the promise.",
          "The Word task pane and the Docs sidebar live with the source tree, under apps/addin-word and apps/addin-docs. Both call http://127.0.0.1:4317 on your machine.",
        ],
      },
    ],
  },
  {
    path: "/local",
    nav: "Local",
    kind: "story",
    title: "Local-first — Asten keeps the files",
    description:
      "Asten is desktop-first and offline-first. Projects live in SQLite on disk. Verne does not receive the manuscript. Export is a file you can open elsewhere.",
    h1: "The files stay on this machine.",
    answer:
      "Asten stores the project in a local SQLite database. There is no account and no sync service. If the application disappeared, the export would still be a file.",
    blocks: [
      {
        heading: "Where the bytes are",
        paragraphs: [
          "The desktop build uses asten.db in the operating system’s application-data folder. The development API uses data/asten.db unless you set ASTEN_DATABASE_PATH.",
          "Provenance is an append-only history and a source on settled facts. Character offsets are not identities. Block ids are.",
        ],
      },
    ],
  },
  {
    path: "/name",
    nav: "Name",
    kind: "story",
    title: "The name Asten — a Verne product",
    description:
      "Asten is another name for Thoth, the Egyptian scribe of the gods. The product is published by Verne. The name is not a costume and not a generator.",
    h1: "Asten is another name for Thoth.",
    answer:
      "Asten is another name for Thoth, scribe of the gods, and the name of this writing environment. Asten is a Verne product.",
    blocks: [
      {
        heading: "What the name is for",
        paragraphs: [
          "Writing, wisdom, the moon, the weighing of words. The product kept paper, ink, and ochre. It did not build a temple theme or a purple suite.",
          "Verne publishes Asten. The manuscript remains the author’s.",
        ],
      },
    ],
  },
  {
    path: "/download",
    nav: null,
    kind: "story",
    title: "Open Asten on this machine — Verne",
    description:
      "Run Asten locally from the repository: the API on port 4317 and the desk on port 1420. No account. The desktop build will be listed here when it is cut.",
    h1: "Open it on this machine.",
    answer:
      "Asten runs locally. Install dependencies, start the API, then the desk. No account is required. A packaged desktop build is not posted on this site yet.",
    blocks: [
      {
        heading: "How to run the current tree",
        paragraphs: [
          "Clone https://github.com/ArefinAlter/asten, run pnpm install, then pnpm dev:api and pnpm dev. The foyer on this website is separate and does not hold your project.",
          "Read the license and the privacy policy before you treat a diagnostic as a verdict. It is a notice.",
        ],
      },
    ],
  },
  {
    path: "/guide",
    nav: "Record",
    kind: "record",
    title: "What Asten is — the record, by Verne",
    description:
      "A complete description of Asten, a Verne product: the desk, canon, cast, time, optional world, compiler, packs, Word and Docs guests, and local storage.",
    h1: "The record of what Asten is.",
    answer:
      "Asten is a local-first writing environment published by Verne. AI may edit as a critic. It may not be the writer. The manuscript stays in SQLite on your machine.",
    blocks: [
      {
        heading: "Who makes Asten?",
        paragraphs: [
          "Verne publishes Asten and this website. There is no Asten account. Verne does not receive the manuscript from the desk.",
          "The source tree is https://github.com/ArefinAlter/asten. A separate research-ingest repository holds lawful source notes for product research. It is not the writing environment.",
        ],
      },
      {
        heading: "What does the desk do?",
        paragraphs: [
          "The desk is a binder of manuscripts, chapters, scenes, notes, and sources; a rich-text page; comments; find; word counts; a letter-sized page view; and track changes as marks.",
          "You can start a scene without filling a world bible. Page view answers how long the book feels. It is not a pagination engine for print production.",
        ],
      },
      {
        heading: "How does canon work?",
        paragraphs: [
          "A fact is an entity, a predicate, a value, a modality, a status, an optional valid-from and valid-to, and a source. Modalities include established, inferred, contested, and author override. Status moves from proposed to confirmed to deprecated.",
          "Only settled facts are worth a hard contradiction. Contested material can stay quiet. The author override wins.",
        ],
      },
      {
        heading: "What do cast, time, and world require?",
        paragraphs: [
          "Nothing, at the start. The cast names entities when you want facts to hang on them. Time stores story time and reading order as two clocks. World is a wall, an atlas, and a dossier.",
          "Settling a world entry can mint a confirmed fact. Leaving it as a scrap does not. Gardeners are not blocked.",
        ],
      },
      {
        heading: "What does the compiler report?",
        paragraphs: [
          "CANON-E013: two confirmed values for the same attribute. CANON-E101: a life that continues after a settled death. CANON-E201: reading order against story time. Optional diction hints live in a pack.",
          "A diagnostic points at a span. It does not rewrite the span. You can suppress a code. Snapshots and an append-only operation log keep provenance on disk.",
        ],
      },
      {
        heading: "How do Word and Google Docs fit?",
        paragraphs: [
          "Compile writes Markdown, plain text, Fountain, HTML, or docx. Import reads those formats back, with a conservative extract from docx. PDF is a printed HTML page, not a separate typesetting program.",
          "The Word add-in and the Docs add-on are guests. They list local projects, show diagnostics, and can file the selected prose into the local project. Canon is not copied to a Verne server, because there is no such server for canon.",
        ],
      },
      {
        heading: "What does Asten refuse?",
        paragraphs: [
          "It refuses to be the writer. It refuses a cloud lock on an unpublished draft. It refuses to require worldbuilding. It refuses a mythological skin, a purple suite, and a doctrine named after a craft book.",
          "It also refuses to pretend the current download is a signed installer. Today you run it from source. The license page says what that permission is.",
        ],
      },
    ],
  },
  {
    path: "/faq",
    nav: "Questions",
    kind: "record",
    title: "Questions about Asten — answered by Verne",
    description:
      "Direct answers: what Asten is, who publishes it, where manuscripts live, whether AI writes, whether worldbuilding is required, and how Word and Docs are used.",
    h1: "Questions, answered in one breath each.",
    answer:
      "Asten is a local-first writing environment by Verne. It notices. It does not write the book, require an account, or keep a copy of the manuscript.",
    blocks: [],
    questions: [
      {
        question: "What is Asten?",
        answer:
          "Asten is a local-first writing environment. It combines a manuscript desk with a compiler that notices contradictions in world, character, and time.",
      },
      {
        question: "Who makes Asten?",
        answer:
          "Asten is a Verne product. Verne publishes the environment and this website.",
      },
      {
        question: "Does Asten write the manuscript?",
        answer:
          "No. The rule is that AI may be the editor and may not be the writer. Asten can point at a span. It does not pick up the pen.",
      },
      {
        question: "Where is the manuscript stored?",
        answer:
          "In a SQLite file on the machine running Asten. The desktop name is asten.db. Verne does not receive it.",
      },
      {
        question: "Does Asten require an account?",
        answer:
          "No. There is no sign-in on the desk and no sign-in on this website.",
      },
      {
        question: "Does this website track visitors?",
        answer:
          "The site sets no cookies and loads no analytics. The host, Vercel, sees ordinary connection data when a page is requested. See the privacy policy.",
      },
      {
        question: "Do I have to build the world before I write?",
        answer:
          "No. World notes, the cast, and the timeline are optional. A scene can be first.",
      },
      {
        question: "What does the compiler check?",
        answer:
          "Settled contradictions: clashing attributes, a life after a settled death, and story time against reading order. Extra craft hints ship in packs you can turn off.",
      },
      {
        question: "Can I silence a diagnostic?",
        answer:
          "Yes. A suppression is stored with the project. The rule can stay quiet without deleting the fact.",
      },
      {
        question: "Can I use Word or Google Docs?",
        answer:
          "Yes, as guests. Compile or import a file, or use the optional add-in to file a selection to the local API. Canon stays on the machine.",
      },
      {
        question: "Which formats can I compile?",
        answer:
          "Markdown, plain text, Fountain, HTML, and docx. A letter view can be printed. Asten is not a print-production suite.",
      },
      {
        question: "Is Asten open source?",
        answer:
          "The source is public on GitHub. The permission to run it is on the license page. Do not assume a blanket open-source grant unless a LICENSE file in that repository says so.",
      },
      {
        question: "How do I contact Verne?",
        answer:
          "Through the source repository listed on the legal notice. No postal address or dedicated legal inbox is published on this site.",
      },
    ],
  },
  legal(
    "/privacy",
    "Privacy",
    "Privacy policy — Asten by Verne",
    "Asten stores the manuscript on your machine. This website does not run an account, and Verne does not receive the book.",
    "Privacy policy",
    "The manuscript stays on the computer running Asten. Verne does not receive it. This website does not set cookies or run analytics.",
    [
      [
        "What this policy covers",
        "This policy covers the Asten writing environment and the public Asten website. Both are published by Verne.",
        "It does not govern Microsoft Word, Google Docs, GitHub, Vercel, or a machine you do not control. Those have their own terms.",
      ],
      [
        "The manuscript",
        "The desk keeps projects, scenes, facts, comments, snapshots, tracked changes, and diagnostics in a SQLite database on the computer where Asten is running.",
        "That file is asten.db in the application-data folder, or data/asten.db when the local API is used in development, unless ASTEN_DATABASE_PATH points elsewhere.",
        "Verne does not operate sync. The desk does not upload the manuscript, the world notes, or the diagnostics to Verne.",
        "If you copy the database, export a file, or hand a document to Word or Docs, that copy is yours to place, under the terms of wherever you send it.",
      ],
      [
        "The public website",
        "This website describes the product. It does not ask you to sign in, and it does not store a project.",
        "It does not set cookies. It does not use browser storage to recognize you. It does not load analytics, advertising, or fonts from another site. Type is taken from fonts already on your machine.",
        "The site is hosted on Vercel. A request carries what a web server needs in order to answer: IP address, time, the path, and ordinary browser headers. Verne does not add a tracker on top of that.",
        "This page does not claim a retention period for Vercel’s own logs. Their hosting terms describe their systems.",
      ],
      [
        "Add-ins",
        "The Word task pane and the Docs sidebar are optional. They call the Asten API at 127.0.0.1 on your machine. A selection you file is written into the local project. It is not sent to Verne.",
        "The Word task pane loads Office.js from Microsoft, because Word requires that library. That request is between your computer and Microsoft.",
        "The Docs sidebar is shown by Google Apps Script. Google already has the document open in Docs. Asten does not make an extra copy of it on a Verne server.",
      ],
      [
        "Accounts, children, and sale",
        "There is no Asten account and no newsletter on this site.",
        "Asten is not directed at children under 16, and it does not knowingly collect information from them.",
        "Verne does not sell personal information. There is no manuscript on Verne’s side to share with an advertiser, and the website has no advertising partner.",
      ],
      [
        "Requests you can make",
        "Verne cannot delete or export a book it does not hold. Delete a local project by deleting it in the desk, or delete the database file.",
        "For this website, you may ask what this policy means. Write through the repository named on the legal notice. Verne has not published a postal address or a dedicated privacy inbox here.",
        "Where a privacy law gives you rights against a website operator, those rights concern the public site. They do not create a copy of your manuscript in Verne’s hands.",
        "If you are in the EEA or the United Kingdom, Verne has not appointed a representative on this page. The desk is not a remote service processing the manuscript for Verne.",
      ],
      [
        "Changes",
        "The date below is the date of this text. If a later build syncs, signs in, or phones home, this page has to say so, and the software has to say so in the interface before it does.",
      ],
    ],
  ),
  legal(
    "/terms",
    "Terms",
    "Terms of use — Asten by Verne",
    "You may read this site and run Asten locally. You keep the manuscript. A diagnostic is a notice, not a verdict.",
    "Terms of use",
    "You may read this website and run Asten on machines you control. You keep the copyright in what you write. Verne does not promise that a diagnostic is correct.",
    [
      [
        "The site and the software",
        "The website explains Asten. It is not an offer to host your book, and it is not a contract to finish a feature that the software does not yet contain.",
        "You may run the build you have for your own writing and research. The license page states the permission. These terms cover the website and the use of the product name in public.",
      ],
      [
        "Your manuscript",
        "You own your text, or you hold it under someone else’s rights. Importing a file is your statement that you may use that file.",
        "Diagnostics are notices. You may suppress one. You remain the author of what you later publish. Asten does not take a share of the work because it pointed at a sentence.",
      ],
      [
        "No warranty",
        "The software and the site are provided as they stand. To the extent the law allows, Verne disclaims warranties of merchantability, fitness for a particular purpose, accuracy of a diagnostic, and uninterrupted operation.",
        "A lost draft, a missed contradiction, or a line you changed because a notice fired or stayed quiet is not a defect Verne has agreed to insure.",
      ],
      [
        "Liability",
        "To the extent the law allows, Verne is not liable for indirect or consequential loss arising from the site or the software, including a lost manuscript that existed only on your disk.",
        "Some places do not allow that limit. Where they do not, the limit does not apply. Nothing on this page removes liability that the law refuses to let a publisher waive.",
      ],
      [
        "Law and changes",
        "Verne has not chosen a court or a national law on this page. Mandatory protections where you live still apply.",
        "These terms change when this page changes. There is no account in which to click “accept.” Using a later build is not a waiver of rights the law keeps for you.",
      ],
    ],
  ),
  legal(
    "/cookies",
    "Cookies",
    "Cookies — Asten by Verne",
    "The Asten website does not set cookies. The desk does not use a cookie to sign you in.",
    "Cookie notice",
    "The Asten website does not set cookies. It does not use local storage or session storage to recognize a visitor.",
    [
      [
        "What is not in use",
        "No analytics cookie. No advertising cookie. No consent banner for a tracker that is not there.",
        "The desk has no sign-in cookie. The project is a SQLite file on disk, which is not a cookie and is not sent to Verne.",
        "If a later version needs a cookie, this page will name the cookie, who sets it, and how long it lasts, before it is set.",
      ],
    ],
  ),
  legal(
    "/license",
    "License",
    "License — running Asten, a Verne product",
    "You may run Asten on your own machines. This page is not, by itself, an open-source grant. Your manuscript is not licensed to Verne.",
    "License to run Asten",
    "Verne grants a personal, non-exclusive, non-transferable permission to run Asten for your own writing and research on computers you control.",
    [
      [
        "What you may do",
        "You may install and run the software, including the local API, the desk, the desktop shell, and the Word and Docs guests, for your own work.",
        "You may keep, copy, export, and publish your manuscripts, notes, and the local database. Those are not licensed to Verne, and Verne claims no copyright in them by this page.",
      ],
      [
        "What this page does not grant",
        "This page does not grant a right to offer Asten as a hosted service, to strip the Asten and Verne names and ship the environment as another product, or to present Verne’s site copy as yours.",
        "If a LICENSE file in the Asten repository grants broader rights to the code, that file governs the code it covers. This page still governs the product name and the website text.",
        "There is no support contract, uptime promise, or indemnity on this page.",
      ],
      [
        "Ending",
        "The permission to run the software for your own work continues with the copy you have. Your manuscripts are unaffected by any argument about the software license.",
      ],
    ],
  ),
  legal(
    "/acceptable-use",
    "Use",
    "Acceptable use — Asten by Verne",
    "Use Asten on text you have a right to hold. Do not point the local API at someone else’s machine.",
    "Acceptable use",
    "Use Asten on works you are allowed to possess and adapt. Do not use the local API to read or change a project on a computer you do not control.",
    [
      [
        "Texts",
        "Do not import, index, or compile a work you are not allowed to hold. Asten will not police that for you, and it will not launder the question by calling the file a research note.",
        "The separate research-ingest tools are not a license to scrape, bypass access controls, or redistribute copyrighted books.",
      ],
      [
        "Machines and names",
        "Do not attack, scan, or alter another person’s local Asten API. The default bind is loopback for a reason.",
        "Do not impersonate Verne, or send a false copyright notice about a file this website does not host.",
        "Verne can change or withdraw a public page that is being abused. Verne cannot remotely switch off a database on your disk.",
      ],
    ],
  ),
  legal(
    "/accessibility",
    "Access",
    "Accessibility — Asten by Verne",
    "The Asten website is text with headings, named links, and a reduced-motion path. It has not had an independent conformance audit.",
    "Accessibility",
    "The public pages use a single main heading, real subheadings, and links with names. Motion yields when the system asks for less of it. This is not a certified audit.",
    [
      [
        "The website",
        "Ink on paper is the palette. Ochre marks a link or a rule; it is not the only carrier of meaning in a sentence.",
        "Pages are in English. A skip past decoration is provided by ordinary heading structure rather than a hidden duplicate of the page.",
      ],
      [
        "The desk",
        "The desk is a local application. The editor follows the browser and its own commands for keyboard input. Diagnostics are words and codes, not color alone.",
        "Verne does not claim a named conformance level for the desk or the website. If a page blocks you, write through the repository on the legal notice. Name the page and what failed.",
      ],
    ],
  ),
  legal(
    "/copyright",
    "Copyright",
    "Copyright — Asten by Verne",
    "You own what you write. Verne owns the Asten name and this website’s text. The website does not host manuscripts.",
    "Copyright",
    "Copyright in your manuscript, notes, and exports stays with you, or with whoever already owned them. Verne’s copyright covers the Asten and Verne names as used here, the site’s prose, and the expression of the software.",
    [
      [
        "What is not claimed",
        "A fact, an idea, or a diagnostic that points at your sentence is not a claim on the sentence. Filing a selection through an add-in does not assign the selection to Verne.",
      ],
      [
        "Notices",
        "This website does not store visitor uploads. A notice about an infringing manuscript will usually be about a file that is not here. A notice about the site’s own copy is in scope.",
        "Send notices through the repository on the legal notice. Include the URL, the work you say is infringed, and a way to reply. Verne has not registered a separate designated agent on this page. A notice is still read.",
        "If you believe site copy was removed in error, reply the same way and say what should be restored.",
      ],
    ],
  ),
  legal(
    "/subprocessors",
    "Processors",
    "Subprocessors — Asten by Verne",
    "The desk has no subprocessors for the manuscript. The public website is hosted on Vercel. Word and Docs are involved only if you open them.",
    "Subprocessors",
    "The manuscript is not processed by a Verne vendor, because it is not sent to Verne. The public website has one host.",
    [
      [
        "Desk",
        "No subprocessor. SQLite is on disk. This version does not ship a cloud model. If a later build calls a model, the interface and this page have to name it first.",
      ],
      [
        "Website",
        "Vercel, Inc. hosts the public pages and receives connection data as described in the privacy policy. No analytics vendor is added.",
      ],
      [
        "Only if you choose them",
        "GitHub hosts the source repository. Visiting it or cloning it is your use of GitHub.",
        "Microsoft supplies Office.js and the Word document you already opened, if you sideload the task pane.",
        "Google supplies Apps Script and the Docs file you already opened, if you install the sidebar.",
        "Verne does not use a payment processor, an email vendor, or an advertising network for Asten.",
      ],
    ],
  ),
  legal(
    "/notice",
    "Notice",
    "Legal notice — Asten is a Verne product",
    "Asten is a Verne product. Verne publishes the writing environment and this website. No postal address is printed here.",
    "Legal notice",
    "Asten is a Verne product. Verne publishes the writing environment and this website. This page is the imprint.",
    [
      [
        "Identity",
        "Product: Asten, a local-first writing environment.",
        "Publisher: Verne.",
        `Website: ${site.origin}`,
        "Source: https://github.com/ArefinAlter/asten",
        "Separate research tools: https://github.com/ArefinAlter/asten-research-ingest",
      ],
      [
        "What is not on this page",
        "No postal address is published here. No company number, VAT number, or dedicated legal inbox is published here.",
        "Contact is the source repository above. This notice does not state a country of incorporation.",
      ],
    ],
  ),
];

function legal(
  path: string,
  nav: string,
  title: string,
  description: string,
  h1: string,
  answer: string,
  blocks: string[][],
): PageRecord {
  return {
    path,
    nav,
    title,
    description,
    h1,
    answer,
    kind: "legal",
    blocks: blocks.map(([heading, ...paragraphs]) => ({
      heading: heading ?? "",
      paragraphs,
    })),
  };
}

export function pageByPath(pathname: string): PageRecord | undefined {
  const path = pathname.length > 1 ? pathname.replace(/\/$/, "") : "/";
  return pages.find((page) => page.path === path);
}

export function absoluteUrl(path: string): string {
  if (path === "/") return `${site.origin}/`;
  return `${site.origin}${path}`;
}

export function jsonLdFor(page: PageRecord): Record<string, unknown> {
  const url = absoluteUrl(page.path);
  const orgId = `${site.origin}/#verne`;
  const appId = `${site.origin}/#asten`;
  const graph: Record<string, unknown>[] = [
    {
      "@type": "Organization",
      "@id": orgId,
      name: site.publisher,
      url: absoluteUrl("/notice"),
      brand: { "@type": "Brand", name: site.product },
    },
    {
      "@type": "SoftwareApplication",
      "@id": appId,
      name: site.product,
      applicationCategory: "WritingApplication",
      operatingSystem: "Windows, macOS, Linux",
      isAccessibleForFree: true,
      description: site.tagline,
      url: absoluteUrl("/"),
      publisher: { "@id": orgId },
      offers: {
        "@type": "Offer",
        price: "0",
        priceCurrency: "USD",
        description: "Run from the public source tree. No account.",
      },
    },
    {
      "@type": "SoftwareSourceCode",
      name: site.product,
      codeRepository: site.repository,
      publisher: { "@id": orgId },
      targetProduct: { "@id": appId },
    },
    {
      "@type": "WebSite",
      "@id": `${site.origin}/#website`,
      name: site.product,
      url: absoluteUrl("/"),
      description: site.tagline,
      publisher: { "@id": orgId },
      inLanguage: "en",
    },
    {
      "@type": "WebPage",
      "@id": `${url}#webpage`,
      url,
      name: page.title,
      description: page.description,
      isPartOf: { "@id": `${site.origin}/#website` },
      about: { "@id": appId },
      inLanguage: "en",
      dateModified: site.updated,
    },
    {
      "@type": "BreadcrumbList",
      itemListElement: [
        {
          "@type": "ListItem",
          position: 1,
          name: "Asten",
          item: absoluteUrl("/"),
        },
        ...(page.path === "/"
          ? []
          : [
              {
                "@type": "ListItem",
                position: 2,
                name: page.nav ?? page.h1,
                item: url,
              },
            ]),
      ],
    },
  ];
  if (page.questions && page.questions.length > 0) {
    graph.push({
      "@type": "FAQPage",
      "@id": `${url}#faq`,
      url,
      mainEntity: page.questions.map((item) => ({
        "@type": "Question",
        name: item.question,
        acceptedAnswer: { "@type": "Answer", text: item.answer },
      })),
    });
  }
  return { "@context": "https://schema.org", "@graph": graph };
}

export function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

export function crawlHtml(page: PageRecord): string {
  const blocks = page.blocks
    .map(
      (block) =>
        `<section><h2>${escapeHtml(block.heading)}</h2>${block.paragraphs
          .map((paragraph) => `<p>${escapeHtml(paragraph)}</p>`)
          .join("")}</section>`,
    )
    .join("");
  const questions = (page.questions ?? [])
    .map(
      (item) =>
        `<section><h2>${escapeHtml(item.question)}</h2><p>${escapeHtml(item.answer)}</p></section>`,
    )
    .join("");
  return `<article class="crawl" id="crawl"><p>Asten is a Verne product.</p><h1>${escapeHtml(page.h1)}</h1><p>${escapeHtml(page.answer)}</p>${blocks}${questions}</article>`;
}

export function headHtml(page: PageRecord): string {
  const url = absoluteUrl(page.path);
  const ld = JSON.stringify(jsonLdFor(page)).replaceAll("<", "\\u003c");
  return [
    `<title>${escapeHtml(page.title)}</title>`,
    `<meta name="description" content="${escapeHtml(page.description)}" />`,
    `<meta name="author" content="${site.publisher}" />`,
    `<meta name="publisher" content="${site.publisher}" />`,
    `<meta name="robots" content="index, follow, max-snippet:-1, max-image-preview:large" />`,
    `<link rel="canonical" href="${url}" />`,
    `<link rel="alternate" hreflang="en" href="${url}" />`,
    `<link rel="alternate" hreflang="x-default" href="${url}" />`,
    `<meta property="og:type" content="website" />`,
    `<meta property="og:site_name" content="Asten" />`,
    `<meta property="og:title" content="${escapeHtml(page.title)}" />`,
    `<meta property="og:description" content="${escapeHtml(page.description)}" />`,
    `<meta property="og:url" content="${url}" />`,
    `<meta property="og:locale" content="en_US" />`,
    `<meta name="twitter:card" content="summary" />`,
    `<meta name="twitter:title" content="${escapeHtml(page.title)}" />`,
    `<meta name="twitter:description" content="${escapeHtml(page.description)}" />`,
    `<script type="application/ld+json">${ld}</script>`,
  ].join("");
}

export function llmsText(): string {
  const lines = [
    `# ${site.product}`,
    "",
    `> ${site.tagline}`,
    "",
    "Asten is a Verne product. It is not a generator and not a typesetter. Manuscripts stay in SQLite on the author’s machine. Word and Google Docs are guests. There is no account.",
    "",
    "## Product",
    ...pages
      .filter((page) => page.kind === "story")
      .map(
        (page) =>
          `- [${page.nav ?? page.h1}](${absoluteUrl(page.path)}): ${page.description}`,
      ),
    "",
    "## Record",
    ...pages
      .filter((page) => page.kind === "record")
      .map(
        (page) =>
          `- [${page.nav ?? page.h1}](${absoluteUrl(page.path)}): ${page.description}`,
      ),
    "",
    "## Legal",
    ...pages
      .filter((page) => page.kind === "legal")
      .map(
        (page) =>
          `- [${page.nav ?? page.h1}](${absoluteUrl(page.path)}): ${page.description}`,
      ),
    "",
    `Source: ${site.repository}`,
    `Updated: ${site.updated}`,
    "",
  ];
  return lines.join("\n");
}

export function llmsFullText(): string {
  const parts = [
    `# ${site.product}`,
    "",
    site.tagline,
    "",
    "Publisher: Verne.",
    "",
  ];
  for (const page of pages) {
    parts.push(`# ${page.h1}`, "", page.answer, "");
    for (const block of page.blocks) {
      parts.push(`## ${block.heading}`, "", ...block.paragraphs, "");
    }
    for (const item of page.questions ?? []) {
      parts.push(`## ${item.question}`, "", item.answer, "");
    }
    parts.push(`Source: ${absoluteUrl(page.path)}`, "");
  }
  return parts.join("\n");
}
