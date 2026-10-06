import { legalList } from "../content/legal-index";

/** One loader per page chunk. The client preloads the current route's chunk before hydrating. */
export const loaders = {
  home: () => import("./Home"),
  how: () => import("./HowItWorks"),
  features: () => import("./Features"),
  feature: () => import("./FeaturePage"),
  ai: () => import("./AiStance"),
  audience: () => import("./AudiencePage"),
  compare: () => import("./Compare"),
  download: () => import("./Download"),
  roadmap: () => import("./Roadmap"),
  journalIndex: () => import("./JournalIndex"),
  article: () => import("./Journal"),
  reference: () => import("./Reference"),
  name: () => import("./Name"),
  company: () => import("./Company"),
  brand: () => import("./Brand"),
  legal: () => import("./Legal"),
  notFound: () => import("./NotFound"),
};

const exact: Record<string, keyof typeof loaders> = {
  "/": "home",
  "/how-it-works": "how",
  "/features": "features",
  "/ai": "ai",
  "/compare": "compare",
  "/download": "download",
  "/roadmap": "roadmap",
  "/journal": "journalIndex",
  "/glossary": "reference",
  "/faq": "reference",
  "/guide": "reference",
  "/name": "name",
  "/about": "company",
  "/contact": "company",
  "/brand": "brand",
  "/legal": "legal",
};

const legalPaths = new Set(legalList.map((doc) => doc.path));

export function loaderFor(pathname: string): () => Promise<unknown> {
  const path = pathname.length > 1 ? pathname.replace(/\/+$/, "") : "/";
  const key = exact[path];
  if (key) return loaders[key];
  if (path.startsWith("/features/")) return loaders.feature;
  if (path.startsWith("/for/")) return loaders.audience;
  if (path.startsWith("/journal/")) return loaders.article;
  if (legalPaths.has(path)) return loaders.legal;
  return loaders.notFound;
}
