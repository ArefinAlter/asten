/**
 * Prose content model shared by the journal, legal record, glossary, FAQ, and
 * guide. Strings support a tiny inline markup, rendered by `Inline`:
 *
 *   *italic*   **bold**   `code`   [label](/internal/path)   [label](https://…)
 */

export type Block =
  | { kind: "p"; text: string }
  | { kind: "h3"; text: string }
  | { kind: "list"; items: string[]; ordered?: boolean }
  | { kind: "quote"; text: string; cite?: string }
  | {
      kind: "callout";
      text: string;
      title?: string;
      tone?: "note" | "rubric" | "settled";
    }
  | {
      /** A red-ink Asten mark, shown the way the desk would show it. */
      kind: "diagnostic";
      code: string;
      title: string;
      lines: string[];
    }
  | { kind: "terms"; items: { term: string; text: string }[] }
  | { kind: "table"; head: string[]; rows: string[][]; caption?: string };

export interface Section {
  /** Stable anchor id, kebab-case. */
  id: string;
  heading: string;
  blocks: Block[];
}

export interface QA {
  q: string;
  a: string;
}

export type ArtKey =
  | "scales"
  | "moon"
  | "pyramid"
  | "eye"
  | "palette"
  | "ibis"
  | "scroll"
  | "lotus";

export interface Article {
  slug: string;
  /** The H1. */
  title: string;
  /** The <title>, 60 characters or fewer, without the site suffix. */
  metaTitle: string;
  /** Meta description, 140–160 characters. */
  description: string;
  /** One-sentence standfirst under the headline. */
  dek: string;
  /** A 40–60 word direct answer to the article's core question (AEO). */
  answer: string;
  /** ISO dates. */
  published: string;
  updated: string;
  minutes: number;
  tags: string[];
  takeaways: string[];
  sections: Section[];
  faqs: QA[];
  /** Internal paths to related pages. */
  related: string[];
  art: ArtKey;
}

export interface LegalDoc {
  path: string;
  nav: string;
  title: string;
  metaTitle: string;
  description: string;
  /** Plain-language summary shown first. */
  summary: string[];
  sections: Section[];
}

export interface GlossaryTerm {
  slug: string;
  term: string;
  /** One-sentence definition, quotable on its own. */
  short: string;
  /** Optional longer explanation. */
  long?: string;
  /** Internal path where the idea is shown in context. */
  see?: string;
  related?: string[];
}

export interface FaqGroup {
  id: string;
  title: string;
  items: QA[];
}
