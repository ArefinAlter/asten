export const site = {
  origin: "https://asten-writing.vercel.app",
  name: "Asten",
  publisher: "Verne",
  legalName: "Verne Ltd.",
  creditName: "Verne Ltd.",
  publisherUrl: "https://vernesystems.com",
  companyDetails: "https://vernesystems.com/legal/company",
  email: "info@vernesystems.com",
  accessSubject: "Asten early access",
  address: "8/A, Sher-e-Bangla Road, Zighatola, Dhaka-1209, Bangladesh",
  registration: "C-201411-1",
  social: {
    x: "https://x.com/VerneOf",
    linkedin: "https://www.linkedin.com/company/verneoff",
  },
  updated: "2026-10-06",
  updatedLabel: "6 October 2026",
  tagline: "Let AI be your editor, not your writer.",
  description:
    "Asten is a local-first writing app that remembers the facts of your story and marks contradictions in red. It never writes a word for you.",
  locale: "en_US",
  themeColor: { day: "#f5eedf", night: "#0f1a2e" },
} as const;

export const accessHref = `mailto:${site.email}?subject=${encodeURIComponent(site.accessSubject)}`;

export function absoluteUrl(path: string): string {
  return path === "/" ? `${site.origin}/` : `${site.origin}${path}`;
}
