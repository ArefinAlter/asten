import { useEffect, useRef } from "react";
import { metaFor } from "../routes";
import { headTags } from "./head";

/** Keeps <head> in step with client-side navigation. The server writes the same tags. */
export function Seo({ pathname }: { pathname: string }) {
  const first = useRef(true);
  useEffect(() => {
    // On first load the server has already written this page's head, structured data included.
    if (first.current) {
      first.current = false;
      if (document.head.querySelector("[data-seo]")) return;
    }
    const meta = metaFor(pathname);
    const head = document.head;
    head.querySelectorAll("[data-seo]").forEach((node) => node.remove());
    const anchor = head.querySelector('script[src="/boot.js"]');
    for (const { tag, attrs, text } of headTags(meta)) {
      if (tag === "title") {
        document.title = text ?? "";
        continue;
      }
      const element = document.createElement(tag);
      element.setAttribute("data-seo", "");
      for (const [key, value] of Object.entries(attrs)) element.setAttribute(key, value);
      if (text) element.textContent = text;
      head.insertBefore(element, anchor?.nextSibling ?? null);
    }
  }, [pathname]);
  return null;
}
