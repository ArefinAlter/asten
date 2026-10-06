import { audiences } from "./content/audiences";
import { features } from "./content/features";
import { articleList as articles } from "./content/journal/meta";
import { legalList as legalDocs } from "./content/legal-index";

export type RouteKind =
  | "home"
  | "product"
  | "feature"
  | "audience"
  | "journal"
  | "learn"
  | "company"
  | "legal"
  | "error";

export type SchemaKind =
  | "web"
  | "article"
  | "faq"
  | "glossary"
  | "howto"
  | "about"
  | "contact"
  | "collection";

export interface RouteMeta {
  path: string;
  /** Full <title>. */
  title: string;
  description: string;
  /** Short name used in breadcrumbs, menus, and llms.txt. */
  label: string;
  kind: RouteKind;
  parent?: string;
  schema?: SchemaKind;
  published?: string;
  modified?: string;
  priority?: number;
  noindex?: boolean;
  og: { eyebrow: string; headline: string };
}

const suffix = " · Asten";

function page(meta: Omit<RouteMeta, "title"> & { title: string; bare?: boolean }): RouteMeta {
  const { bare, ...rest } = meta;
  return { ...rest, title: bare ? meta.title : `${meta.title}${suffix}` };
}

const product: RouteMeta[] = [
  page({
    path: "/",
    bare: true,
    title: "Asten: let AI be your editor, not your writer",
    description:
      "Asten is a local-first writing app that remembers your story’s facts and marks contradictions in red. It never writes your prose. No account, no cloud.",
    label: "Home",
    kind: "home",
    schema: "web",
    priority: 1,
    og: { eyebrow: "A writing desk with a memory", headline: "Let AI be your editor, not your writer." },
  }),
  page({
    path: "/how-it-works",
    title: "How Asten works: write, settle, notice",
    description:
      "Write freely, settle the facts that matter, and Asten marks the page that disagrees. A plain-language tour of canon, diagnostics, and the author’s veto.",
    label: "How it works",
    kind: "product",
    schema: "howto",
    priority: 0.9,
    og: { eyebrow: "How it works", headline: "Write. Settle. Notice. You decide." },
  }),
  page({
    path: "/features",
    title: "Features: everything a long story needs",
    description:
      "The desk, canon checks, cast, two clocks, an optional world, method packs, Word and Google Docs guests, and local-first files. Everything except a ghostwriter.",
    label: "Features",
    kind: "product",
    schema: "collection",
    priority: 0.9,
    og: { eyebrow: "Features", headline: "Everything a long story needs. Nothing that writes it." },
  }),
  ...features.map((feature) =>
    page({
      path: feature.path,
      title: feature.title,
      description: feature.description,
      label: feature.name,
      kind: "feature",
      parent: "/features",
      priority: 0.8,
      og: { eyebrow: `Feature · ${feature.name}`, headline: feature.h1 },
    }),
  ),
  page({
    path: "/ai",
    title: "Our AI stance: editor, never writer",
    description:
      "Where Asten stands on AI: no model writes your prose, nothing trains on your manuscript, and any future AI only proposes facts you confirm, locally first.",
    label: "AI stance",
    kind: "product",
    schema: "article",
    priority: 0.9,
    og: { eyebrow: "Our AI stance", headline: "The pen stays in your hand." },
  }),
  ...audiences.map((audience) =>
    page({
      path: audience.path,
      title: audience.title,
      description: audience.description,
      label: audience.name,
      kind: "audience",
      parent: "/features",
      priority: 0.8,
      og: { eyebrow: `For ${audience.name.toLowerCase()}`, headline: audience.h1 },
    }),
  ),
  page({
    path: "/compare",
    title: "Asten compared: generators, word processors, wikis",
    description:
      "How Asten differs from AI story generators, word processors, manuscript organizers, and worldbuilding wikis, and when you might want one of those instead.",
    label: "Compare",
    kind: "product",
    schema: "article",
    priority: 0.7,
    og: { eyebrow: "Compare", headline: "Where Asten fits, and where it doesn’t." },
  }),
  page({
    path: "/download",
    title: "Get Asten: early access",
    description:
      "Asten is in early access. There’s no public download yet. Ask for an invitation, see what runs today, and what you need on your machine.",
    label: "Get Asten",
    kind: "product",
    schema: "howto",
    priority: 0.9,
    og: { eyebrow: "Early access", headline: "Get Asten on your own machine." },
  }),
  page({
    path: "/roadmap",
    title: "Roadmap: shipped, next, later, never",
    description:
      "What Asten does today, what is planned next, what comes later, and the things it will never do. Honest status, no dates we can’t keep.",
    label: "Roadmap",
    kind: "product",
    schema: "article",
    priority: 0.7,
    og: { eyebrow: "Roadmap", headline: "Shipped, next, later, and never." },
  }),
];

const learn: RouteMeta[] = [
  page({
    path: "/journal",
    title: "Journal: craft notes on continuity and AI",
    description:
      "Essays and guides on keeping a long story consistent, flashbacks and story time, story bibles, local-first writing, and using AI as an editor.",
    label: "Journal",
    kind: "journal",
    schema: "collection",
    priority: 0.8,
    og: { eyebrow: "The Journal", headline: "Craft notes from the scribe’s desk." },
  }),
  ...articles.map((article) =>
    page({
      path: `/journal/${article.slug}`,
      title: article.metaTitle,
      description: article.description,
      label: article.title,
      kind: "journal",
      parent: "/journal",
      schema: "article",
      published: article.published,
      modified: article.updated,
      priority: 0.7,
      og: { eyebrow: "The Journal", headline: article.title },
    }),
  ),
  page({
    path: "/glossary",
    title: "Glossary of writing and continuity terms",
    description:
      "Plain definitions of canon, facts, modality, diagnostics, story time, reading order, story bibles, method packs, and other words used in Asten.",
    label: "Glossary",
    kind: "learn",
    schema: "glossary",
    priority: 0.7,
    og: { eyebrow: "Glossary", headline: "The words of the scribe’s trade." },
  }),
  page({
    path: "/faq",
    title: "Questions and answers",
    description:
      "Straight answers about Asten: what it is, whether it uses AI, where your files live, what it checks, how to get it, and who makes it.",
    label: "FAQ",
    kind: "learn",
    schema: "faq",
    priority: 0.8,
    og: { eyebrow: "Questions", headline: "Asked, and answered plainly." },
  }),
  page({
    path: "/guide",
    title: "The complete guide to Asten",
    description:
      "One page that describes all of Asten: the desk, canon, the compiler, cast, time, world, packs, guests, local storage, AI stance, status, and who publishes it.",
    label: "The record",
    kind: "learn",
    schema: "article",
    priority: 0.8,
    og: { eyebrow: "The record", headline: "Everything Asten is, on one page." },
  }),
  page({
    path: "/name",
    title: "Why Asten? Thoth, the god of writing",
    description:
      "Asten is another name for Thoth, the ibis-headed Egyptian god of writing, the moon, and reckoning, who recorded the verdict at the Weighing of the Heart.",
    label: "The name",
    kind: "learn",
    schema: "article",
    priority: 0.6,
    og: { eyebrow: "The name", headline: "Asten is another name for Thoth." },
  }),
];

const company: RouteMeta[] = [
  page({
    path: "/about",
    title: "About Asten and Verne",
    description:
      "Asten is made by Verne, an AI engineering company in Dhaka. Why we built a writing app where AI is the editor and never the writer.",
    label: "About",
    kind: "company",
    schema: "about",
    priority: 0.6,
    og: { eyebrow: "About", headline: "Made by Verne, for writers who keep the pen." },
  }),
  page({
    path: "/brand",
    title: "Brand guidelines and press kit",
    description:
      "The Asten brand: the Palette mark, the scribe’s pigments, typography, registers, motion, voice, and downloadable logo files for press.",
    label: "Brand",
    kind: "company",
    schema: "web",
    priority: 0.5,
    og: { eyebrow: "Brand & press", headline: "Black ink for the writer. Red for the editor." },
  }),
  page({
    path: "/contact",
    title: "Contact Verne about Asten",
    description:
      "Write to Verne about Asten: early access, press, partnerships, security reports, accessibility problems, or legal notices.",
    label: "Contact",
    kind: "company",
    schema: "contact",
    priority: 0.6,
    og: { eyebrow: "Contact", headline: "Write to the scribes." },
  }),
  page({
    path: "/legal",
    title: "Legal: policies and notices",
    description:
      "Every Asten policy in one place: privacy, terms, cookies, license, acceptable use, AI policy, accessibility, copyright, trademark, security, and the imprint.",
    label: "Legal",
    kind: "legal",
    schema: "collection",
    priority: 0.4,
    og: { eyebrow: "Legal", headline: "Policies written to be read." },
  }),
  ...legalDocs.map((doc) =>
    page({
      path: doc.path,
      title: doc.metaTitle,
      description: doc.description,
      label: doc.nav,
      kind: "legal",
      parent: "/legal",
      schema: "web",
      priority: 0.3,
      og: { eyebrow: "Legal", headline: doc.title },
    }),
  ),
];

export const notFound: RouteMeta = page({
  path: "/404",
  title: "Page not found",
  description: "This page was lost to the sand. The rest of the Asten site is still standing.",
  label: "Not found",
  kind: "error",
  noindex: true,
  og: { eyebrow: "404", headline: "Lost to the sand." },
});

export const routes: RouteMeta[] = [...product, ...learn, ...company, notFound];

const byPath = new Map(routes.map((route) => [route.path, route]));

export function normalizePath(pathname: string): string {
  const clean = pathname.split(/[?#]/)[0] ?? "/";
  return clean.length > 1 ? clean.replace(/\/+$/, "") : "/";
}

export function metaFor(pathname: string): RouteMeta {
  return byPath.get(normalizePath(pathname)) ?? notFound;
}

export function breadcrumbsFor(meta: RouteMeta): RouteMeta[] {
  const trail: RouteMeta[] = [];
  let current: RouteMeta | undefined = meta;
  while (current) {
    trail.unshift(current);
    current = current.parent ? byPath.get(current.parent) : undefined;
  }
  if (trail[0]?.path !== "/") trail.unshift(byPath.get("/")!);
  return trail;
}

export const indexable = routes.filter((route) => !route.noindex);
