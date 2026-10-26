import type { QA } from "./types";

export interface Step {
  title: string;
  text: string;
}

/** The four steps shown on the home page and /how-it-works. */
export const howSteps: Step[] = [
  {
    title: "Write",
    text: "Open a scene and type. No setup, no character forms, no world bible required. The binder keeps chapters, scenes, notes, and sources in order.",
  },
  {
    title: "Settle",
    text: "When a detail matters, settle it as a fact: *Mira · hair · auburn*. It joins your canon, with a status, an optional span of story time, and a source.",
  },
  {
    title: "Notice",
    text: "When a later page disagrees with what you settled, Asten marks it in red in the margin with a code, both sides of the disagreement, and your options.",
  },
  {
    title: "Decide",
    text: "Keep the sentence, update canon, record a change, or silence the rule. Asten records your decision. It never rewrites a word.",
  },
];

/** How to get Asten today, used on /download and in HowTo structured data. */
export const accessSteps: Step[] = [
  {
    title: "Try it in the browser",
    text: "Open the hosted pre-alpha. It runs the real engine in a private sandbox of your own, starts on a short sample novel, and is deleted 24 hours after your first visit. Do not put your manuscript there.",
  },
  {
    title: "Install the desktop app",
    text: "Download release 0.0.1a for Windows x64 or Linux x64. It runs the engine on your own computer, with your projects in one file on your disk, and offers “Open the sample novel”. The installers are not signed yet, so your system warns before the first run.",
  },
  {
    title: "Start a project",
    text: "Create a novel, screenplay, or non-fiction project. Write the first scene before you settle anything.",
  },
  {
    title: "Keep a copy",
    text: "Your project is one file on your disk. Take snapshots as you go, and export to Markdown, Fountain, HTML, .docx, EPUB, or Final Draft whenever you like.",
  },
];

export const homeFaqs: QA[] = [
  {
    q: "Does Asten write for me?",
    a: "No. Asten never generates prose. It remembers the facts you settle and marks the places where your draft disagrees with them. Every word on the page is yours.",
  },
  {
    q: "Does Asten use AI?",
    a: "No model runs, local or remote. Asten reads names and plain attributes from the page with fixed rules and weighs them against facts you confirm. Planned perception models are small local classifiers that only propose facts for you to confirm; they do not write text.",
  },
  {
    q: "Will my writing be used to train AI?",
    a: "Never. Your manuscript stays in a file on your computer. Verne never receives it, and Asten will never send it to train a model.",
  },
  {
    q: "Do I have to build a world or a story bible first?",
    a: "No. You can start with a blank scene. Settle facts only when they matter, and the cast, timeline, and world notes stay optional.",
  },
  {
    q: "What does Asten check today?",
    a: "Seven families of rules, all deterministic: attributes that disagree with your canon or with each other (CANON-E013 to E015), lifecycle such as acting after a settled death (E101 to E103), the two clocks and dates that cannot all be true (E201 to E203), a journey that does not fit the time (E301), who knows what and when (E401), relations and genealogy (E501 to E503), and rules you write yourself (E601). Name hygiene and a setup ledger come along. Optional diction hints are off until you turn them on.",
  },
  {
    q: "How do I get Asten?",
    a: "Try the pre-alpha: open the hosted copy in your browser, or download release 0.0.1a for Windows x64 or Linux x64 from the [download page](/download). macOS comes later. The builds are unsigned, so your system warns before the first run.",
  },
];
