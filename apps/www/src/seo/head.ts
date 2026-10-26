import type { RouteMeta } from "../routes";
import { plateForPath, plates } from "../personality/plates";
import { absoluteUrl, site } from "../site";

export interface HeadTag {
  tag: "title" | "meta" | "link" | "script";
  attrs: Record<string, string>;
  text?: string;
}

export function ogSlug(path: string): string {
  return path === "/" ? "home" : path.slice(1).replaceAll("/", "-");
}

export function ogImageFor(meta: RouteMeta): string {
  return `${site.origin}/og/${ogSlug(meta.path)}.png`;
}

/** Head tags for a page. Structured data is passed in by the server only, to keep it out of the client bundle. */
export function headTags(meta: RouteMeta, jsonLd?: unknown): HeadTag[] {
  const url = absoluteUrl(meta.path);
  const image = ogImageFor(meta);
  const isArticle = meta.schema === "article" && meta.kind === "journal";
  const tags: HeadTag[] = [
    { tag: "title", attrs: {}, text: meta.title },
    { tag: "meta", attrs: { name: "description", content: meta.description } },
    {
      tag: "meta",
      attrs: {
        name: "robots",
        content: meta.noindex
          ? "noindex, follow"
          : "index, follow, max-image-preview:large, max-snippet:-1, max-video-preview:-1",
      },
    },
    { tag: "meta", attrs: { name: "author", content: site.publisher } },
    { tag: "link", attrs: { rel: "canonical", href: url } },
    { tag: "link", attrs: { rel: "alternate", hreflang: "en", href: url } },
    { tag: "link", attrs: { rel: "alternate", hreflang: "x-default", href: url } },
    { tag: "meta", attrs: { property: "og:type", content: isArticle ? "article" : "website" } },
    { tag: "meta", attrs: { property: "og:site_name", content: site.name } },
    { tag: "meta", attrs: { property: "og:locale", content: site.locale } },
    { tag: "meta", attrs: { property: "og:title", content: meta.title } },
    { tag: "meta", attrs: { property: "og:description", content: meta.description } },
    { tag: "meta", attrs: { property: "og:url", content: url } },
    { tag: "meta", attrs: { property: "og:image", content: image } },
    { tag: "meta", attrs: { property: "og:image:width", content: "1200" } },
    { tag: "meta", attrs: { property: "og:image:height", content: "630" } },
    { tag: "meta", attrs: { property: "og:image:alt", content: `${meta.og.eyebrow}: ${meta.og.headline}` } },
    { tag: "meta", attrs: { name: "twitter:card", content: "summary_large_image" } },
    { tag: "meta", attrs: { name: "twitter:site", content: "@VerneOf" } },
    { tag: "meta", attrs: { name: "twitter:title", content: meta.title } },
    { tag: "meta", attrs: { name: "twitter:description", content: meta.description } },
    { tag: "meta", attrs: { name: "twitter:image", content: image } },
  ];
  // The hero painting is the largest thing above the fold: fetch it first.
  if (meta.path === "/") {
    tags.push({ tag: "link", attrs: { rel: "preload", as: "image", href: "/art/hero-window.webp", type: "image/webp", fetchpriority: "high" } });
  } else {
    // Every other page opens with its own painted plate, which is its largest image.
    const pp = plateForPath(meta.path);
    const spec = pp ? plates[pp.plate] : undefined;
    if (spec) tags.push({ tag: "link", attrs: { rel: "preload", as: "image", href: spec.src, type: "image/webp", fetchpriority: "high" } });
  }
  if (isArticle && meta.published) {
    tags.push(
      { tag: "meta", attrs: { property: "article:published_time", content: meta.published } },
      { tag: "meta", attrs: { property: "article:modified_time", content: meta.modified ?? meta.published } },
      { tag: "meta", attrs: { property: "article:publisher", content: site.publisherUrl } },
    );
  }
  if (jsonLd) {
    tags.push({
      tag: "script",
      attrs: { type: "application/ld+json" },
      text: JSON.stringify(jsonLd).replaceAll("<", "\\u003c"),
    });
  }
  return tags;
}

function escapeAttr(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

function escapeText(value: string): string {
  return value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
}

export function headFor(meta: RouteMeta, jsonLd?: unknown): string {
  return headTags(meta, jsonLd)
    .map(({ tag, attrs, text }) => {
      const attributes = Object.entries(attrs)
        .map(([key, value]) => ` ${key}="${escapeAttr(value)}"`)
        .join("");
      if (tag === "meta" || tag === "link") return `<${tag} data-seo${attributes} />`;
      const body = tag === "script" ? (text ?? "") : escapeText(text ?? "");
      return `<${tag} data-seo${attributes}>${body}</${tag}>`;
    })
    .join("\n    ");
}
