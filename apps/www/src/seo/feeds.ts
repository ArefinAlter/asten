import { articleList as articles } from "../content/journal/meta";
import { indexable, type RouteKind } from "../routes";
import { absoluteUrl, site } from "../site";

const groups: { title: string; kinds: RouteKind[] }[] = [
  { title: "Product", kinds: ["home", "product"] },
  { title: "Features", kinds: ["feature"] },
  { title: "Who it is for", kinds: ["audience"] },
  { title: "Journal", kinds: ["journal"] },
  { title: "Reference", kinds: ["learn"] },
  { title: "Company", kinds: ["company"] },
  { title: "Legal", kinds: ["legal"] },
];

/** llms.txt: a curated map of the site for language models. */
export function llmsText(): string {
  const lines = [
    `# ${site.name}`,
    "",
    `> ${site.description}`,
    "",
    "Key facts:",
    "- Asten is a writing environment for novels, screenplays, and other long fiction. Its slogan is “Let AI be your editor, not your writer.”",
    "- It never generates prose. It runs no AI model: its checks are fifteen deterministic rules over facts the author confirms and plain attributes read from the page (for example CANON-E013 attribute contradiction, CANON-E101 life after a settled death, CANON-E201 story time stepping backward inside a continuous run).",
    "- It is local-first: the project is a SQLite file on the author’s computer. No account, no sync, no telemetry. Manuscripts are never used to train models.",
    "- Asten 0.0.1a is a public pre-alpha: unsigned desktop builds for Windows x64 and Linux x64 at https://github.com/ArefinAlter/asten/releases/tag/v0.0.1a, and a hosted copy at https://asten.103.191.50.32.sslip.io/ that keeps a private sandbox per visitor for 24 hours. macOS comes later.",
    `- Asten is published by ${site.publisher} (${site.legalName}, Dhaka, Bangladesh): ${site.publisherUrl}`,
    "- The name Asten is another name for Thoth, the Egyptian god of writing.",
    "",
  ];
  for (const group of groups) {
    const items = indexable.filter((route) => group.kinds.includes(route.kind));
    if (items.length === 0) continue;
    lines.push(`## ${group.title}`, "");
    for (const route of items) {
      lines.push(`- [${route.label}](${absoluteUrl(route.path)}): ${route.description}`);
    }
    lines.push("");
  }
  lines.push(
    "## Optional",
    "",
    `- [Full text of every page](${site.origin}/llms-full.txt): the whole site as plain Markdown.`,
    `- [Sitemap](${site.origin}/sitemap.xml)`,
    "",
  );
  return lines.join("\n");
}

export function llmsFullIntro(): string {
  return [
    `# ${site.name}: the complete site text`,
    "",
    `> ${site.description}`,
    "",
    `Publisher: ${site.publisher} (${site.legalName}), ${site.publisherUrl}. Contact: ${site.email}. Updated ${site.updated}.`,
    "",
    "Every public page follows, separated by horizontal rules. Each section starts with its canonical URL.",
    "",
  ].join("\n");
}

function xml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

export function rssFeed(): string {
  const items = [...articles]
    .sort((a, b) => b.published.localeCompare(a.published))
    .map((article) => {
      const url = absoluteUrl(`/journal/${article.slug}`);
      return [
        "    <item>",
        `      <title>${xml(article.title)}</title>`,
        `      <link>${url}</link>`,
        `      <guid isPermaLink="true">${url}</guid>`,
        `      <pubDate>${new Date(`${article.published}T09:00:00Z`).toUTCString()}</pubDate>`,
        `      <description>${xml(article.description)}</description>`,
        ...article.tags.map((tag) => `      <category>${xml(tag)}</category>`),
        "    </item>",
      ].join("\n");
    })
    .join("\n");
  return [
    '<?xml version="1.0" encoding="UTF-8"?>',
    '<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">',
    "  <channel>",
    "    <title>The Asten Journal</title>",
    `    <link>${absoluteUrl("/journal")}</link>`,
    `    <atom:link href="${site.origin}/journal/rss.xml" rel="self" type="application/rss+xml" />`,
    "    <description>Craft notes on continuity, story time, story bibles, local-first writing, and AI as an editor.</description>",
    "    <language>en</language>",
    `    <lastBuildDate>${new Date(`${site.updated}T09:00:00Z`).toUTCString()}</lastBuildDate>`,
    items,
    "  </channel>",
    "</rss>",
    "",
  ].join("\n");
}
