/**
 * Skills: named, reusable instructions for the assistant, each limited to the
 * tools it lists. The person enables, inspects and edits them in Settings. A
 * skill can only narrow what the assistant does; it cannot open a closed tool.
 */

export interface Skill {
  id: string;
  name: string;
  summary: string;
  tools: string[];
  prompt: string;
  body: string;
}

export const SKILLS: Skill[] = [
  {
    id: "explain-mark",
    name: "Explain a mark",
    summary: "Walk through a mark's evidence chain and the repairs it offers.",
    tools: ["marks.list", "marks.explain", "repairs.propose"],
    prompt: "Explain the selected mark",
    body: "Read the mark. Say which statements disagree and where each was settled. List the repairs the rule offers, keeping 'keep my sentence' among them. Do not suggest new wording.",
  },
  {
    id: "sweep",
    name: "Continuity sweep",
    summary: "Run the checks and report the worst marks first.",
    tools: ["checks.run", "marks.list"],
    prompt: "Run the checks",
    body: "Weigh the project. Report errors first, then warnings, by reading order. Offer to explain any one.",
  },
  {
    id: "canon-lookup",
    name: "Canon lookup",
    summary: "What is settled about a name, and where.",
    tools: ["canon.search"],
    prompt: "What is settled about Mara?",
    body: "Find the name in the cast. List settled facts with the scene that settled them; list proposals separately.",
  },
  {
    id: "fact-triage",
    name: "Fact triage",
    summary: "Gather proposals into one card you settle or reject.",
    tools: ["canon.search", "facts.propose"],
    prompt: "Which proposals are pending?",
    body: "List proposed facts read from the page. Prepare a card; the person settles each one. Never settle.",
  },
  {
    id: "source-digest",
    name: "Source digest",
    summary: "Summarise a research or source note (never a scene).",
    tools: ["graph.search", "sources.summarise"],
    prompt: "Summarise the research on tide tables",
    body: "Summarise the source note in its own words, opening sentence per paragraph, and list what links to it. Never read or summarise manuscript scenes.",
  },
  {
    id: "graph-walk",
    name: "Graph walk",
    summary: "Follow wikilinks and backlinks from a note.",
    tools: ["graph.search"],
    prompt: "What links to The brass key?",
    body: "Start at the note. List outgoing links and backlinks with the sentence each sits in.",
  },
  {
    id: "timeline",
    name: "Timeline check",
    summary: "Events in story time, next to reading order.",
    tools: ["timeline.list"],
    prompt: "Show the timeline",
    body: "List events by story time and note where reading order steps back.",
  },
];
