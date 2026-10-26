import { audiences } from "../content/audiences";
import { faq } from "../content/faq";
import { features } from "../content/features";
import { glossary } from "../content/glossary";
import { accessSteps, homeFaqs, howSteps } from "../content/home";
import { release } from "../content/release";
import { articles } from "../content/journal";
import type { QA } from "../content/types";
import type { RouteMeta } from "../routes";
import { absoluteUrl, site } from "../site";

type Node = Record<string, unknown>;

const ORG = `${site.origin}/#verne`;
const APP = `${site.origin}/#asten`;
const WEBSITE = `${site.origin}/#website`;

function organization(): Node {
  return {
    "@type": "Organization",
    "@id": ORG,
    name: site.publisher,
    legalName: site.legalName,
    url: site.publisherUrl,
    email: site.email,
    logo: { "@type": "ImageObject", url: `${site.origin}/verne-mark.png` },
    address: {
      "@type": "PostalAddress",
      streetAddress: "8/A, Sher-e-Bangla Road, Zighatola",
      addressLocality: "Dhaka",
      postalCode: "1209",
      addressCountry: "BD",
    },
    sameAs: [site.publisherUrl, site.social.x, site.social.linkedin],
    brand: { "@id": APP },
  };
}

function application(): Node {
  return {
    "@type": "SoftwareApplication",
    "@id": APP,
    name: site.name,
    alternateName: "Asten writing environment",
    description: site.description,
    slogan: site.tagline,
    url: absoluteUrl("/"),
    applicationCategory: "UtilitiesApplication",
    applicationSubCategory: "Creative writing software",
    operatingSystem: "Windows 10 and 11 (x64), Linux (x86_64); a hosted copy in the browser",
    softwareVersion: "0.0.1b (pre-alpha)",
    downloadUrl: release.releaseUrl,
    releaseNotes: release.releaseUrl,
    image: `${site.origin}/og/home.png`,
    publisher: { "@id": ORG },
    author: { "@id": ORG },
    featureList: features.map((feature) => `${feature.name}: ${feature.short}`),
    isAccessibleForFree: true,
  };
}

function website(): Node {
  return {
    "@type": "WebSite",
    "@id": WEBSITE,
    name: site.name,
    url: absoluteUrl("/"),
    description: site.description,
    publisher: { "@id": ORG },
    inLanguage: "en",
  };
}

const pageType: Record<string, string> = {
  web: "WebPage",
  article: "WebPage",
  faq: "FAQPage",
  glossary: "WebPage",
  howto: "WebPage",
  about: "AboutPage",
  contact: "ContactPage",
  collection: "CollectionPage",
};

export function schemaFor(meta: RouteMeta, trail: RouteMeta[], image: string): Node {
  const url = absoluteUrl(meta.path);
  const webpage: Node = {
    "@type": pageType[meta.schema ?? "web"] ?? "WebPage",
    "@id": `${url}#webpage`,
    url,
    name: meta.title,
    description: meta.description,
    isPartOf: { "@id": WEBSITE },
    about: { "@id": APP },
    publisher: { "@id": ORG },
    inLanguage: "en",
    dateModified: meta.modified ?? site.updated,
    breadcrumb: { "@id": `${url}#breadcrumb` },
    primaryImageOfPage: { "@type": "ImageObject", url: image, width: 1200, height: 630 },
  };
  const graph: Node[] = [organization(), website(), application(), webpage];

  graph.push({
    "@type": "BreadcrumbList",
    "@id": `${url}#breadcrumb`,
    itemListElement: trail.map((crumb, index) => ({
      "@type": "ListItem",
      position: index + 1,
      name: crumb.label,
      item: absoluteUrl(crumb.path),
    })),
  });

  const faqs = faqsFor(meta.path);
  if (faqs.length > 0) {
    const questions = faqs.map((item) => ({
      "@type": "Question",
      name: item.q,
      acceptedAnswer: { "@type": "Answer", text: plain(item.a) },
    }));
    if (meta.schema === "faq") {
      webpage.mainEntity = questions;
    } else {
      graph.push({ "@type": "FAQPage", "@id": `${url}#faq`, url, mainEntity: questions });
    }
  }

  if (meta.kind === "journal" && meta.schema === "article") {
    const article = articles.find((item) => `/journal/${item.slug}` === meta.path);
    if (article) {
      graph.push({
        "@type": "BlogPosting",
        "@id": `${url}#article`,
        headline: article.title,
        description: article.description,
        abstract: article.answer,
        datePublished: article.published,
        dateModified: article.updated,
        author: { "@id": ORG },
        publisher: { "@id": ORG },
        image,
        mainEntityOfPage: { "@id": `${url}#webpage` },
        keywords: article.tags.join(", "),
        timeRequired: `PT${article.minutes}M`,
        inLanguage: "en",
      });
    }
  } else if (meta.schema === "article") {
    graph.push({
      "@type": "Article",
      "@id": `${url}#article`,
      headline: meta.og.headline,
      description: meta.description,
      dateModified: meta.modified ?? site.updated,
      datePublished: meta.published ?? site.published,
      author: { "@id": ORG },
      publisher: { "@id": ORG },
      image,
      mainEntityOfPage: { "@id": `${url}#webpage` },
      inLanguage: "en",
    });
  }

  if (meta.schema === "glossary") {
    graph.push({
      "@type": "DefinedTermSet",
      "@id": `${url}#terms`,
      name: "Asten glossary",
      url,
      hasDefinedTerm: glossary.map((term) => ({
        "@type": "DefinedTerm",
        "@id": `${url}#${term.slug}`,
        name: term.term,
        description: plain(term.short),
        url: `${url}#${term.slug}`,
        inDefinedTermSet: { "@id": `${url}#terms` },
      })),
    });
  }

  if (meta.schema === "howto") {
    const steps = meta.path === "/download" ? accessSteps : howSteps;
    graph.push({
      "@type": "HowTo",
      "@id": `${url}#howto`,
      name: meta.og.headline,
      description: meta.description,
      step: steps.map((step, index) => ({
        "@type": "HowToStep",
        position: index + 1,
        name: step.title,
        text: plain(step.text),
        url: `${url}#step-${index + 1}`,
      })),
    });
  }

  if (meta.schema === "contact") {
    graph.push({
      "@type": "ContactPoint",
      "@id": `${url}#contact`,
      email: site.email,
      contactType: "customer support",
      availableLanguage: ["English"],
    });
  }

  return { "@context": "https://schema.org", "@graph": graph };
}

/** Questions answered visibly on a page, for FAQPage structured data. */
function faqsFor(path: string): QA[] {
  if (path === "/") return homeFaqs;
  if (path === "/faq") return faq.flatMap((group) => group.items);
  return (
    features.find((item) => item.path === path)?.faqs ??
    audiences.find((item) => item.path === path)?.faqs ??
    articles.find((item) => `/journal/${item.slug}` === path)?.faqs ??
    []
  );
}

/** Strip the inline markup used in content strings. */
export function plain(text: string): string {
  return text
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
    .replace(/\*\*([^*]+)\*\*/g, "$1")
    .replace(/\*([^*]+)\*/g, "$1")
    .replace(/`([^`]+)`/g, "$1");
}
