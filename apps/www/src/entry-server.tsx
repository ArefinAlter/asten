import { StrictMode } from "react";
import { renderToString } from "react-dom/server";
import { prerender } from "react-dom/static";
import { StaticRouter } from "react-router-dom";
import App from "./App";
import { breadcrumbsFor, metaFor, type RouteMeta } from "./routes";
import { headFor, ogImageFor } from "./seo/head";
import { schemaFor } from "./seo/schema";

export { routes } from "./routes";
export { site } from "./site";
export { llmsText, llmsFullIntro, rssFeed } from "./seo/feeds";
export { artMarkup, ogArtFor } from "./seo/art-static";
export { metaFor } from "./routes";

export interface Rendered {
  html: string;
  head: string;
  meta: RouteMeta;
}

/** Renders a route to complete static HTML. */
export async function render(path: string): Promise<Rendered> {
  const meta = metaFor(path);
  const tree = (
    <StrictMode>
      <StaticRouter location={path}>
        <App />
      </StaticRouter>
    </StrictMode>
  );
  // First pass resolves the page's lazy chunk. The second renders it inline, so the
  // HTML carries the whole page with no streaming scripts or hidden segments.
  await prerender(tree);
  const html = renderToString(tree);
  const jsonLd = schemaFor(meta, breadcrumbsFor(meta), ogImageFor(meta));
  return { html, head: headFor(meta, jsonLd), meta };
}
