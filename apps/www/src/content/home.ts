import type { QA } from "./types";

export interface Step {
  title: string;
  text: string;
}

/** The three-step story told on the home page and /how-it-works. */
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
    title: "Ask for an invitation",
    text: "Email info@vernesystems.com with the subject “Asten early access”. Tell us what you write and which computer you use.",
  },
  {
    title: "Install the desk",
    text: "Invited writers receive a build and setup notes. Asten runs as a desktop shell, or in your browser against a small server on your own machine.",
  },
  {
    title: "Start a project",
    text: "Create a novel, screenplay, or non-fiction project. Write the first scene before you settle anything.",
  },
  {
    title: "Keep a copy",
    text: "Your project is one file on your disk. Take snapshots as you go, and export to Markdown, Fountain, HTML, or .docx whenever you like.",
  },
];

export const homeFaqs: QA[] = [
  {
    q: "Does Asten write for me?",
    a: "No. Asten never generates prose. It remembers the facts you settle and marks the places where your draft disagrees with them. Every word on the page is yours.",
  },
  {
    q: "Does Asten use AI?",
    a: "No model runs, local or remote. Asten reads names and plain attributes from the page with fixed rules and weighs them against facts you confirm. Planned perception models are small local classifiers that only propose, and any generative AI, if it ever comes, will be optional, local first, and switchable off entirely.",
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
    a: "Seven families of rules, all deterministic: attributes that disagree with your canon or with the page itself (CANON-E013 to E015), lifecycle such as acting after a settled death (E101 to E103), the two clocks and dates that cannot all be true (E201 to E203), travel and place (E301, E302), who knows what and when (E401), relations and genealogy (E501 to E503), and rules you write yourself (E601, E602). Name hygiene and setup ledgers come along. Optional diction hints are off until you turn them on.",
  },
  {
    q: "How do I get Asten?",
    a: "Asten is in early access and there is no public download yet. Ask for an invitation at info@vernesystems.com with the subject “Asten early access”.",
  },
];
