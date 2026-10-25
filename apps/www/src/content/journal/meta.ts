import type { ArtKey } from "../types";

export interface ArticleMeta {
  slug: string;
  title: string;
  metaTitle: string;
  description: string;
  dek: string;
  published: string;
  updated: string;
  minutes: number;
  tags: string[];
  art: ArtKey;
}

/** Light article metadata, kept apart from the full text so lists and the router stay small. */
export const journalMeta: Record<string, ArticleMeta> = {
  "ai-editor-not-writer": {
    slug: "ai-editor-not-writer",
    title: "Let AI be your editor, not your writer",
    metaTitle: "Let AI be your editor, not your writer",
    description:
      "Why drafting with AI changes the voice of a book, what a good editor does, why that job suits software better, and how Asten leaves every word of your draft to you.",
    dek: "The tedious half of writing is remembering what you already decided; the half that makes a book yours is everything else.",
    published: "2026-10-06",
    updated: "2026-10-06",
    minutes: 7,
    tags: ["AI", "Craft", "Editing", "Voice"],
    art: "palette",
  },
  "continuity-in-a-long-novel": {
    slug: "continuity-in-a-long-novel",
    title: "How to keep continuity in a long novel",
    metaTitle: "How to keep continuity in a long novel",
    description:
      "Hair colors change, the dead walk, Tuesday comes twice. A practical guide to catching continuity errors in a long novel, by hand and with a canon checker.",
    dek: "Readers forgive a lot, but they notice when the dead captain gives an order.",
    published: "2026-10-06",
    updated: "2026-10-06",
    minutes: 7,
    tags: ["Continuity", "Revision", "Novels", "Canon"],
    art: "scales",
  },
  "story-time-and-reading-order": {
    slug: "story-time-and-reading-order",
    title: "Story time vs. reading order: flashbacks without contradictions",
    metaTitle: "Story time vs. reading order: flashbacks that hold up",
    description:
      "Every story has two clocks: when things happen and when the reader learns them. How to use flashbacks, frames, and parallel timelines without contradictions.",
    dek: "Your story has two clocks, and most timeline mistakes happen when you only watch one of them.",
    published: "2026-10-06",
    updated: "2026-10-06",
    minutes: 7,
    tags: ["Structure", "Timeline", "Flashbacks", "Craft"],
    art: "moon",
  },
  "do-you-need-a-story-bible": {
    slug: "do-you-need-a-story-bible",
    title: "Do you need a story bible? Gardeners, architects, and the space between",
    metaTitle: "Do you need a story bible? Gardeners and architects",
    description:
      "A story bible can save a series or swallow a draft. Which kind of writer you are, what belongs in a bible, and how to stop worldbuilding long enough to write.",
    dek: "A story bible is a tool for finishing the book, which is easy to forget once the bible gets more fun than the book.",
    published: "2026-10-06",
    updated: "2026-10-06",
    minutes: 7,
    tags: ["Worldbuilding", "Planning", "Story bible", "Craft"],
    art: "pyramid",
  },
  "local-first-writing": {
    slug: "local-first-writing",
    title: "Why your manuscript should live on your own computer",
    metaTitle: "Why your manuscript should live on your own computer",
    description:
      "Cloud-only writing tools can lock you out, go down, or change their terms. What local-first software means, how to back up a book, and which formats last.",
    dek: "The safest home for a book you’ve spent years on is a file you can see, copy, and open without anyone’s permission.",
    published: "2026-10-06",
    updated: "2026-10-06",
    minutes: 7,
    tags: ["Local-first", "Privacy", "Backups", "Ownership"],
    art: "eye",
  },
};

export const articleList: ArticleMeta[] = [journalMeta["ai-editor-not-writer"]!, journalMeta["continuity-in-a-long-novel"]!, journalMeta["story-time-and-reading-order"]!, journalMeta["do-you-need-a-story-bible"]!, journalMeta["local-first-writing"]!];
