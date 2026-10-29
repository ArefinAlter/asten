export interface LegalMeta {
  path: string;
  nav: string;
  title: string;
  metaTitle: string;
  description: string;
}

/** Titles and descriptions of the legal record, apart from the full text. */
export const legalIndex: Record<string, LegalMeta> = {
  "/privacy": {
    path: "/privacy",
    nav: "Privacy",
    title: "Privacy policy",
    metaTitle: "Privacy policy: your manuscript stays on your computer",
    description:
      "How Verne handles personal data for Asten: your manuscript stays on your computer, this site sets no cookies, hosted sandboxes are deleted after 24 hours, and email is used only to reply.",
  },
  "/terms": {
    path: "/terms",
    nav: "Terms",
    title: "Terms of use",
    metaTitle: "Terms of use for the Asten website and builds",
    description:
      "The terms for this website and for Asten builds: you own your writing, a diagnostic is a notice rather than a verdict, and the pre-alpha is provided as is.",
  },
  "/cookies": {
    path: "/cookies",
    nav: "Cookies",
    title: "Cookie notice",
    metaTitle: "Cookie notice: none here, one in the hosted pre-alpha",
    description:
      "This website sets no cookies and runs no trackers; one optional localStorage key remembers your theme. The hosted pre-alpha sets one strictly necessary cookie for 24 hours.",
  },
  "/license": {
    path: "/license",
    nav: "License",
    title: "Asten’s license",
    metaTitle: "Asten is released under the MIT License",
    description:
      "Asten is open source under the MIT License. Your manuscripts are never licensed to Verne, and the Asten and Verne names and logos are not part of the license.",
  },
  "/acceptable-use": {
    path: "/acceptable-use",
    nav: "Acceptable use",
    title: "Acceptable use policy",
    metaTitle: "Acceptable use policy for Asten",
    description:
      "What not to do with Asten and this website: work only with texts you may hold, leave other people’s computers alone, and never attack or overload the site.",
  },
  "/ai-policy": {
    path: "/ai-policy",
    nav: "AI policy",
    title: "AI policy",
    metaTitle: "AI policy: an editor, never a writer",
    description:
      "Asten is an AI product built as an intelligent continuity editor. It pairs deep narrative auditing with local canon rules, while strictly handing creativity back to the writer—it never writes or ghostwrites your prose.",
  },
  "/accessibility": {
    path: "/accessibility",
    nav: "Accessibility",
    title: "Accessibility statement",
    metaTitle: "Accessibility statement",
    description:
      "How the Asten website aims to meet WCAG 2.2 AA, what has been done so far, what is still limited, and how to tell Verne when something gets in your way.",
  },
  "/copyright": {
    path: "/copyright",
    nav: "Copyright",
    title: "Copyright and takedown",
    metaTitle: "Copyright and takedown notices",
    description:
      "You own what you write in Asten. Verne owns this website’s text and original artwork. Here is how to report anything on this site you believe infringes.",
  },
  "/trademark": {
    path: "/trademark",
    nav: "Trademarks",
    title: "Trademark and brand use",
    metaTitle: "Trademark and brand use guidelines",
    description:
      "How to refer to Asten and Verne: what you can say, how to use the logo and press assets from the brand page, and what needs Verne’s written permission first.",
  },
  "/security": {
    path: "/security",
    nav: "Security",
    title: "Security and vulnerability disclosure",
    metaTitle: "Security and vulnerability disclosure",
    description:
      "How to report a security problem in Asten or this website: what to include, what is in scope, the safe harbor for good-faith research, and what happens next.",
  },
  "/subprocessors": {
    path: "/subprocessors",
    nav: "Subprocessors",
    title: "Subprocessors and third parties",
    metaTitle: "Subprocessors and third-party services",
    description:
      "Who else handles data when you use Asten: no one, for your manuscript. Vercel hosts this website, and an email provider carries the messages you send Verne.",
  },
  "/credits": {
    path: "/credits",
    nav: "Credits",
    title: "Credits and open-source notices",
    metaTitle: "Credits and open-source notices",
    description:
      "The fonts, open-source software, and artistic sources behind the Asten website, with their licenses, plus a note on the hieroglyphs and the cartouche name toy.",
  },
  "/notice": {
    path: "/notice",
    nav: "Legal notice",
    title: "Legal notice",
    metaTitle: "Legal notice (imprint) for Asten",
    description:
      "Who publishes Asten and this website: Verne Ltd., registered in Dhaka, Bangladesh, with its registered address, registration, and contact.",
  },
};

export const legalList: LegalMeta[] = Object.values(legalIndex);
