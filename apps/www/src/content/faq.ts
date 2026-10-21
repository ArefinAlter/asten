import type { FaqGroup } from "./types";

/**
 * Frequently asked questions. Every answer is self-contained so it can be
 * quoted on its own by a search or answer engine.
 */
export const faq: FaqGroup[] = [
  {
    id: "basics",
    title: "The basics",
    items: [
      {
        q: "What is Asten?",
        a: "Asten is a local-first writing app for creative writers: a writing desk plus a rule engine that remembers the facts of your story and marks the moment a draft contradicts them. Asten never writes prose. Your words are black ink, and Asten only ever writes in red. See [how it works](/how-it-works).",
      },
      {
        q: "Who makes Asten?",
        a: "Asten is made by Verne (Verne Systems, trading as Verne), a company registered in Dhaka, Bangladesh. You can reach the team at info@vernesystems.com, and there’s more [about Verne](/about) on this site.",
      },
      {
        q: "Why is the app called Asten?",
        a: "Asten is another name for Thoth, the ancient Egyptian god of writing, wisdom, the moon, and reckoning, who recorded the verdict at the Weighing of the Heart. The app takes his job, not his authority: it keeps the record of your story and weighs each page against it, and you decide. Read [the story behind the name](/name).",
      },
      {
        q: "Who is Asten for?",
        a: "Asten is for creative writers who are curious about AI but protective of their voice: novelists, fan-fiction writers, worldbuilders, screenwriters, and game and campaign writers. It also suits AI tinkerers who care that the checks run locally and are deterministic. Start with the pages for [novelists](/for/novelists), [worldbuilders](/for/worldbuilders), [screenwriters](/for/screenwriters), [fan-fiction writers](/for/fan-fiction), or [AI tinkerers](/for/ai-tinkerers).",
      },
      {
        q: "Is Asten a word processor?",
        a: "Asten includes a writing desk: a binder of chapters and scenes, a rich-text editor, comments, find, word counts, a page view, track-change marks, and named snapshots. Asten isn’t a print-layout tool or a full replacement for Word’s review features, so you can compile to .docx when you need Word. What a word processor lacks, and Asten has, is a memory of your canon. See [the desk](/features/desk).",
      },
      {
        q: "Can I write a screenplay in Asten?",
        a: "Asten can compile your project to Fountain, the plain-text screenplay format, and import Fountain files, and its canon checks work the same way on a script as on a novel. Export to Final Draft’s FDX format is planned but not available yet. See [Asten for screenwriters](/for/screenwriters).",
      },
      {
        q: "Can I use Asten for non-fiction?",
        a: "Asten is built for stories: its checks compare facts about characters, places, events, and time. You can still use the desk, binder, snapshots, and compile for any long document, and entities can include sources, but research and citation tools are only planned so far.",
      },
    ],
  },
  {
    id: "ai",
    title: "AI and your words",
    items: [
      {
        q: "Does Asten use AI to write my book?",
        a: "No, Asten never writes prose, and that’s a product principle, not a setting. Asten keeps the record of your story and marks where a draft contradicts it; every sentence in your manuscript is yours. Read [Let AI be your editor, not your writer](/journal/ai-editor-not-writer).",
      },
      {
        q: "Does Asten use AI right now?",
        a: "No AI model runs in Asten today, local or cloud. Asten reads names and plain attributes from the page with fixed rules, and its checks are deterministic over those readings and the facts you confirm, so the same manuscript and the same canon always produce the same marks. See [Asten and AI](/ai).",
      },
      {
        q: "Will Asten add AI features in the future?",
        a: "Asten plans optional fact proposals drawn from your prose: deterministic first, then local machine learning, then optional AI, always as proposals you confirm and never as silent canon. The plan puts local models first, allows a cloud model only with your own provider key and your explicit consent, and includes a complete AI switch-off. None of this has a date; see [the roadmap](/roadmap).",
      },
      {
        q: "Will my writing be used to train an AI model?",
        a: "Never: no manuscript written in Asten is used to train any model. Asten doesn’t send your manuscript anywhere, and Verne never receives it. See the [AI policy](/ai-policy).",
      },
      {
        q: "How is Asten different from AI writing tools?",
        a: "AI story generators are designed to produce prose; Asten is designed never to. Asten remembers what you’ve decided about your story, like Mira’s auburn hair or Captain Oren’s death in chapter 9, and points at the place where a draft disagrees. Your words stay black ink, and Asten only ever writes in red. See [the comparison](/compare).",
      },
      {
        q: "Can I use ChatGPT or Claude alongside Asten?",
        a: "Yes, you can use ChatGPT or Claude alongside Asten; that’s your choice. Asten doesn’t connect to ChatGPT, Claude, or any other AI service, so nothing passes between them unless you copy it across yourself. Asten’s checks work the same way however a draft was written.",
      },
      {
        q: "Does Asten understand my story?",
        a: "No, and Asten doesn’t claim to. Asten reads plain attributes from the page (hair, eyes, height, age, where someone is, alive or dead, who is married to whom) with fixed rules, compares them with facts you’ve confirmed such as Mira · hair color · auburn, and notices when they disagree. It doesn’t read for meaning, and it never writes.",
      },
    ],
  },
  {
    id: "canon",
    title: "Continuity and canon",
    items: [
      {
        q: "What is canon in Asten?",
        a: "Canon in Asten is the set of facts you’ve confirmed about your story. Each fact is an entity, a predicate, and a value, like Mira · hair color · auburn, with a modality and a status, and only confirmed facts produce hard contradictions. See [canon](/features/canon).",
      },
      {
        q: "What continuity errors can Asten catch?",
        a: "Asten ships seventeen canon rules in seven families. `CANON-E013` marks two values for one attribute, such as auburn and black hair, whether both are confirmed or one is on the page; `CANON-E101` marks a character acting after a confirmed death, such as Captain Oren after chapter 9; `CANON-E201` to `E203` cover story time stepping backward inside a continuous run, dates that cannot all be true, and an effect before its cause; `E301` and `E302` cover travel time and being in two places at once; `E401` knowledge a character cannot have yet; `E501` to `E503` genealogy and exclusive relations; `E601` rules you write yourself. See [the compiler](/features/compiler).",
      },
      {
        q: "Does Asten find contradictions in my prose automatically?",
        a: "Yes, for plain attributes. Asten reads names, aliases, hair and eye color, height, age, location, alive or dead, and relations from the page with fixed rules, no model, and weighs them against your canon as warnings. What it reads arrives as proposals you confirm or reject, never as silent canon. Deeper reading, such as a pronoun with no name in the sentence, is not weighed.",
      },
      {
        q: "How do I silence a warning in Asten?",
        a: "Any mark in Asten can be suppressed with a reason, and the suppression is stored with your project and knows when its evidence changes. Diction watch is off unless you turn it on; Lifecycle, Clocks, Relations, Knowledge, Atlas, and Invariants are on by default; and the Scribe pack, Asten’s core canon compiler, is always on. A project’s strictness can also be set to quiet, standard, or thorough. Diagnostics only point at your text, so silencing one never changes a word.",
      },
      {
        q: "What if a contradiction in my story is intentional?",
        a: "Asten gives you several ways to keep an intentional contradiction. Anything inside quotation marks counts as something a character says, not as world truth; a scene can be marked a memory, dream, vision, or frame story and nothing in it is weighed against the world; a narrator can be marked unreliable; a fact can be marked contested or deprecated; or you can suppress the single mark with a reason and keep your sentence. The canon is yours; Asten only records it.",
      },
      {
        q: "Do I have to fill out a story bible before I start writing in Asten?",
        a: "No, Asten doesn’t ask for a story bible, a character form, or a wiki before you write: the cast isn’t a mandatory character sheet, the world views are optional, and a scene never waits on a wiki. Canon grows from facts you confirm whenever you like; read [more on story bibles](/journal/do-you-need-a-story-bible).",
      },
      {
        q: "What are craft hints in Asten?",
        a: "Craft hints in Asten are optional style notes, and they’re never canon errors. Today they come from the Diction watch pack, which notes words like “very” and “really” and is off by default. Asten makes no craft doctrine a default; see [method packs](/features/packs).",
      },
      {
        q: "Does Asten work with flashbacks?",
        a: "Yes, Asten allows flashbacks: it keeps two clocks, story time and reading order, so a scene can happen early and be told late, and it argues only inside a run of scenes you declared continuous. A scene marked as a memory or flashback is never weighed against the world. `CANON-E201` appears only when a continuous run steps backward in story time. See [time](/features/time).",
      },
    ],
  },
  {
    id: "privacy",
    title: "Privacy and your files",
    items: [
      {
        q: "Where does Asten store my writing?",
        a: "Asten stores your projects on your own computer, in a SQLite database; with the desktop shell, that’s a file called `asten.db` in your system’s application-data folder. Verne never receives your manuscript. See [local-first](/features/local-first).",
      },
      {
        q: "Do I need an account to use Asten?",
        a: "No, Asten has no accounts and no sign-in; your projects live on your computer, and using them doesn’t depend on a Verne server.",
      },
      {
        q: "Does Asten sync my writing to the cloud?",
        a: "No, Asten has no sync and no cloud storage, so your project stays on your computer unless you copy it somewhere yourself. Collaboration is planned for later, without giving up offline work.",
      },
      {
        q: "Does Asten collect telemetry or usage data?",
        a: "No, Asten sends no telemetry or usage data. Your manuscript stays on your computer, and Verne never receives it.",
      },
      {
        q: "What file formats can Asten export?",
        a: "Asten compiles to Markdown, plain text, Fountain for screenplays, HTML, and .docx. For a PDF, compile to HTML and print it. Import reads the same family of formats, with a conservative .docx extract, and EPUB and FDX export are planned.",
      },
      {
        q: "How do I back up my Asten projects?",
        a: "Asten keeps your work in an ordinary file on your computer, so you back it up by copying that file, ideally while Asten is closed, to a drive or folder you trust. Inside Asten, named snapshots mark points in your draft and an append-only history records each change. Compiling to Markdown or .docx also gives you a readable copy that doesn’t need Asten at all.",
      },
      {
        q: "Does the Asten website use cookies?",
        a: "No, the Asten website sets no cookies, runs no analytics or ads, and self-hosts its fonts. If you toggle light or dark mode, one `localStorage` key, `asten-theme`, remembers the choice in your browser and never leaves it. The site is hosted on Vercel, which processes ordinary request data such as IP address and user agent to serve pages and guard against abuse; see the [cookie notice](/cookies).",
      },
      {
        q: "What does Verne receive when I use Asten?",
        a: "Verne receives nothing from your Asten desk: there is no account, no sync, and no telemetry, and your manuscript never leaves your computer. If you email info@vernesystems.com, Verne receives your address and message and uses them to reply; there is no mailing list. See the [privacy notice](/privacy).",
      },
    ],
  },
  {
    id: "access",
    title: "Getting Asten",
    items: [
      {
        q: "Can I download Asten?",
        a: "Asten isn’t available to download publicly yet. Asten is in early development, with no public download and no signed installer. Early access is by request: email info@vernesystems.com with “Asten early access” as the subject. See [download](/download).",
      },
      {
        q: "How much does Asten cost?",
        a: "Asten has nothing to buy today, and pricing hasn’t been decided. Whatever comes, the local-first promise holds: your files must stay usable without a subscription or a server.",
      },
      {
        q: "What platforms does Asten run on?",
        a: "Asten runs as a desktop app on Windows (x64 and ARM), macOS 11 or later (Apple silicon and Intel, one universal build) and Linux (x86_64 and aarch64), or in your web browser against a local server on your own computer; one interface serves both. Builds for every platform exist and are unsigned while early access lasts. Either way, your projects stay on your machine.",
      },
      {
        q: "Is there a mobile app for Asten?",
        a: "Not in the stores yet. The desk has a one-column layout for phones and tablets, and you can install it from your phone's browser against the Asten running on your own computer at home. Android and iOS apps are built from the same shell and come after the desktop installers are signed.",
      },
      {
        q: "Is Asten open source?",
        a: "Asten isn’t open source; its source code is private. Asten’s rules are deterministic, though, and the site describes what each one checks; see [the compiler](/features/compiler).",
      },
      {
        q: "Does Asten work with Microsoft Word or Google Docs?",
        a: "Asten works with Word and Google Docs through a Word task pane and a Google Docs sidebar. Both pair once with a six-digit code from the desk, then list your local Asten projects, show the Scales, and can file a selected passage into a local project, talking to Asten’s local API on your computer. They are sideload only and aren’t in the Microsoft or Google stores; see [Word and Google Docs](/features/word-and-google-docs).",
      },
      {
        q: "What’s on the Asten roadmap?",
        a: "Asten’s roadmap includes a signed desktop installer; small local perception models that propose, never generate; generated API contracts; EPUB and FDX export; a graph, maps, boards, research, and citations; a runtime for plugins and packs; and, later, collaboration that keeps offline work. Everything on it is planned, with no promised dates. See [the roadmap](/roadmap).",
      },
      {
        q: "How do I report a bug or security issue in Asten?",
        a: "To report a bug in Asten, email info@vernesystems.com with what you saw and how to reproduce it. For a security issue, read [the security page](/security) first, and please keep the details private until Verne has had a chance to respond.",
      },
    ],
  },
  {
    id: "company",
    title: "Verne and the company",
    items: [
      {
        q: "Who is Verne?",
        a: "Verne is the company that makes Asten: Verne Systems, trading as Verne, registered in Dhaka, Bangladesh. Verne’s website is [vernesystems.com](https://vernesystems.com).",
      },
      {
        q: "How do I contact the Asten team?",
        a: "You can reach the Asten team at info@vernesystems.com. Verne uses your address and message only to reply, and there is no mailing list. See [contact](/contact).",
      },
      {
        q: "Where can I find Asten’s logo and brand guidelines?",
        a: "Asten’s logo, colors, type, and voice are set out on [the brand page](/brand). The short version: your words are black ink, and Asten only ever writes in red.",
      },
      {
        q: "Who do I contact for press inquiries about Asten?",
        a: "Press inquiries about Asten go to info@vernesystems.com, and [the brand page](/brand) covers the logo and how to use it. When describing Asten, please keep to what exists today: a local-first writing desk with deterministic canon checks, no AI model, and access by request.",
      },
      {
        q: "Where can I find Verne’s company details?",
        a: "Verne Systems, trading as Verne, is registered in Dhaka, Bangladesh (company registration C-201411-1), at 8/A, Sher-e-Bangla Road, Zighatola, Dhaka-1209. The full details are on Verne’s [company page](https://vernesystems.com/legal/company) and this site’s [legal page](/legal).",
      },
    ],
  },
];
