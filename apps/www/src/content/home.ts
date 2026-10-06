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
    a: "Not today. Asten’s checks are deterministic rules over facts you confirm, and no model runs, local or remote. Planned AI features will only propose facts for you to confirm, will run locally first, and can be switched off entirely.",
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
    a: "Three things: a character attribute that disagrees with your canon (CANON-E013), someone acting after a settled death (CANON-E101), and scenes whose story time runs backward against reading order (CANON-E201). Optional diction hints are off until you turn them on.",
  },
  {
    q: "How do I get Asten?",
    a: "Asten is in early access and there is no public download yet. Ask for an invitation at info@vernesystems.com with the subject “Asten early access”.",
  },
];
