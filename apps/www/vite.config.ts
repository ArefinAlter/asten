import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import react from "@vitejs/plugin-react";
import { defineConfig, type Plugin } from "vite";
import {
  crawlHtml,
  headHtml,
  llmsFullText,
  llmsText,
  pages,
  site,
} from "./src/seo/catalog.ts";

function seoFiles(): Plugin {
  const dist = path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    "dist",
  );
  return {
    name: "asten-seo-files",
    configurePreviewServer(server) {
      server.middlewares.use((req, _res, next) => {
        const url = req.url?.split("?")[0] ?? "";
        const page = pages.find((item) => item.path === url);
        if (page && page.path !== "/") req.url = `${page.path}/index.html`;
        next();
      });
    },
    closeBundle() {
      const template = fs.readFileSync(path.join(dist, "index.html"), "utf8");
      for (const page of pages) {
        const html = template
          .replace(/<!--seo-head-->[\s\S]*?<!--\/seo-head-->/, headHtml(page))
          .replace(
            /<!--seo-body-->[\s\S]*?<!--\/seo-body-->/,
            crawlHtml(page),
          );
        if (page.path === "/") {
          fs.writeFileSync(path.join(dist, "index.html"), html);
          continue;
        }
        const directory = path.join(dist, page.path.slice(1));
        fs.mkdirSync(directory, { recursive: true });
        fs.writeFileSync(path.join(directory, "index.html"), html);
      }
      const urls = pages
        .map((page) => {
          const loc =
            page.path === "/"
              ? `${site.origin}/`
              : `${site.origin}${page.path}`;
          return `  <url><loc>${loc}</loc><lastmod>${site.updated}</lastmod></url>`;
        })
        .join("\n");
      fs.writeFileSync(
        path.join(dist, "sitemap.xml"),
        `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls}\n</urlset>\n`,
      );
      fs.writeFileSync(
        path.join(dist, "robots.txt"),
        [
          "User-agent: *",
          "Allow: /",
          "",
          "User-agent: GPTBot",
          "Allow: /",
          "",
          "User-agent: OAI-SearchBot",
          "Allow: /",
          "",
          "User-agent: ChatGPT-User",
          "Allow: /",
          "",
          "User-agent: Google-Extended",
          "Allow: /",
          "",
          "User-agent: PerplexityBot",
          "Allow: /",
          "",
          "User-agent: ClaudeBot",
          "Allow: /",
          "",
          "User-agent: anthropic-ai",
          "Allow: /",
          "",
          "User-agent: Applebot-Extended",
          "Allow: /",
          "",
          `Sitemap: ${site.origin}/sitemap.xml`,
          "",
        ].join("\n"),
      );
      fs.writeFileSync(path.join(dist, "llms.txt"), llmsText());
      fs.writeFileSync(path.join(dist, "llms-full.txt"), llmsFullText());
      fs.writeFileSync(
        path.join(dist, "humans.txt"),
        [
          "/* TEAM */",
          "Publisher: Verne",
          "Product: Asten",
          `Contact: ${site.repository}`,
          "",
          "/* SITE */",
          "Language: English",
          `Updated: ${site.updated}`,
          "",
        ].join("\n"),
      );
      fs.mkdirSync(path.join(dist, ".well-known"), { recursive: true });
      fs.writeFileSync(
        path.join(dist, ".well-known", "security.txt"),
        [
          `Contact: ${site.repository}`,
          "Expires: 2027-10-06T00:00:00.000Z",
          "Preferred-Languages: en",
          `Canonical: ${site.origin}/.well-known/security.txt`,
          `Policy: ${site.origin}/notice`,
          "",
        ].join("\n"),
      );
      fs.writeFileSync(
        path.join(dist, "manifest.webmanifest"),
        `${JSON.stringify(
          {
            name: "Asten",
            short_name: "Asten",
            description: site.tagline,
            start_url: "/",
            display: "browser",
            background_color: "#f6f4ef",
            theme_color: "#f6f4ef",
            lang: "en",
          },
          null,
          2,
        )}\n`,
      );
    },
  };
}

export default defineConfig({
  plugins: [react(), seoFiles()],
  clearScreen: false,
  server: {
    port: 4320,
    strictPort: true,
  },
});
