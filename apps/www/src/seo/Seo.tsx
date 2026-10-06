import { useEffect } from "react";
import { jsonLdFor, pageByPath, site, type PageRecord } from "./catalog";

function setMeta(
  attribute: "name" | "property",
  key: string,
  content: string,
) {
  const selector = `meta[${attribute}="${key}"]`;
  let element = document.head.querySelector(selector);
  if (!element) {
    element = document.createElement("meta");
    element.setAttribute(attribute, key);
    document.head.append(element);
  }
  element.setAttribute("content", content);
}

function setLink(rel: string, href: string, hreflang?: string) {
  const selector = hreflang
    ? `link[rel="${rel}"][hreflang="${hreflang}"]`
    : `link[rel="${rel}"]:not([hreflang])`;
  let element = document.head.querySelector(selector);
  if (!element) {
    element = document.createElement("link");
    element.setAttribute("rel", rel);
    if (hreflang) element.setAttribute("hreflang", hreflang);
    document.head.append(element);
  }
  element.setAttribute("href", href);
}

function apply(page: PageRecord) {
  const url =
    page.path === "/" ? `${site.origin}/` : `${site.origin}${page.path}`;
  document.title = page.title;
  document.documentElement.lang = "en";
  setMeta("name", "description", page.description);
  setMeta("name", "author", site.publisher);
  setMeta("name", "publisher", site.publisher);
  setMeta(
    "name",
    "robots",
    "index, follow, max-snippet:-1, max-image-preview:large",
  );
  setMeta("property", "og:type", "website");
  setMeta("property", "og:site_name", site.product);
  setMeta("property", "og:title", page.title);
  setMeta("property", "og:description", page.description);
  setMeta("property", "og:url", url);
  setMeta("property", "og:locale", "en_US");
  setMeta("name", "twitter:card", "summary");
  setMeta("name", "twitter:title", page.title);
  setMeta("name", "twitter:description", page.description);
  setLink("canonical", url);
  setLink("alternate", url, "en");
  setLink("alternate", url, "x-default");

  document.getElementById("seo-ld")?.remove();
  const script = document.createElement("script");
  script.id = "seo-ld";
  script.type = "application/ld+json";
  script.textContent = JSON.stringify(jsonLdFor(page));
  document.head.append(script);
}

export function Seo({ pathname }: { pathname: string }) {
  const page = pageByPath(pathname) ?? pageByPath("/")!;
  useEffect(() => {
    apply(page);
  }, [page]);
  return null;
}
