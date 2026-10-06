export interface Comparison {
  id: string;
  category: string;
  examples: string;
  theyDo: string;
  astenDoes: string;
  chooseThem: string;
}

/** Category-level comparisons. Fair, and specific about when the other tool is the better choice. */
export const comparisons: Comparison[] = [
  {
    id: "ai-generators",
    category: "AI story generators",
    examples: "Apps built around a model that drafts, continues, or rewrites prose for you.",
    theyDo: "Produce sentences, scenes, and ideas on demand. Fast drafts, brainstorming, and getting unstuck.",
    astenDoes: "Never writes a word. Remembers the facts you settle and marks contradictions, so the voice on the page stays entirely yours.",
    chooseThem: "Choose a generator if you want help producing text. Choose Asten if you want help keeping your own text consistent.",
  },
  {
    id: "chat-assistants",
    category: "General AI chat assistants",
    examples: "ChatGPT, Claude, Gemini, and similar assistants.",
    theyDo: "Answer questions, brainstorm, and critique a passage you paste in, within the limits of one conversation.",
    astenDoes: "Keeps a permanent, structured record of your story on your own machine and checks every page against it, deterministically.",
    chooseThem: "Use a chat assistant for open-ended conversation about craft. Use Asten as the memory and the checker. They can sit side by side, and Asten never connects to them.",
  },
  {
    id: "word-processors",
    category: "Word processors",
    examples: "Microsoft Word, Google Docs, Pages.",
    theyDo: "Rich formatting, review workflows, real-time co-editing, page layout.",
    astenDoes: "A manuscript-focused desk with a binder and a canon. It compiles to .docx and invites Word and Docs in as guests.",
    chooseThem: "Stay in Word or Docs for heavy review cycles and layout. Bring the manuscript into Asten when continuity is the problem.",
  },
  {
    id: "manuscript-organizers",
    category: "Manuscript organizers",
    examples: "Binder-style writing apps for long projects.",
    theyDo: "Organize chapters, scenes, and research; flexible compile to many formats.",
    astenDoes: "Organizes the same way, and adds a canon model with continuity checks and two clocks for story time.",
    chooseThem: "If you need very detailed compile templates today, a mature organizer may suit you better. If you want your project to notice its own contradictions, try Asten.",
  },
  {
    id: "worldbuilding-wikis",
    category: "Worldbuilding platforms",
    examples: "Online wikis and apps for lore, maps, and timelines.",
    theyDo: "Deep, shareable encyclopedias of a setting, often with publishing and community features.",
    astenDoes: "Lighter, optional world tools (wall, atlas, dossier) wired directly to the manuscript’s continuity checks, and kept offline.",
    chooseThem: "If your world is the product (a campaign setting you publish), a dedicated platform fits. If the novel is the product, Asten keeps the world in service of it.",
  },
  {
    id: "spreadsheets",
    category: "Spreadsheets and style sheets",
    examples: "The trusty story-bible spreadsheet.",
    theyDo: "Anything you can type, sorted any way you like. Free and flexible.",
    astenDoes: "Turns the style sheet into facts that check themselves against the manuscript, with sources and story time.",
    chooseThem: "A spreadsheet is fine until you stop updating it. Asten is for when you’d like the sheet to do some of the remembering.",
  },
];
