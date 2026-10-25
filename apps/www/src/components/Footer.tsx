import { Link } from "react-router-dom";
import { Frieze, LiveLotusFrieze } from "../art/Friezes";
import { Glyph } from "../art/Glyphs";
import { audiences } from "../content/audiences";
import { site } from "../site";
import { BrandLink, CartoucheWordmark } from "./Brand";

const columns: { title: string; links: { to: string; label: string }[] }[] = [
  {
    title: "Product",
    links: [
      { to: "/how-it-works", label: "How it works" },
      { to: "/features", label: "Features" },
      { to: "/ai", label: "Our AI stance" },
      { to: "/roadmap", label: "Roadmap" },
      { to: "/compare", label: "Compare" },
      { to: "/download", label: "Try the pre-alpha" },
    ],
  },
  {
    title: "For writers",
    links: audiences.map((audience) => ({ to: audience.path, label: audience.name })),
  },
  {
    title: "Learn",
    links: [
      { to: "/journal", label: "Journal" },
      { to: "/glossary", label: "Glossary" },
      { to: "/faq", label: "Questions" },
      { to: "/guide", label: "The complete guide" },
      { to: "/name", label: "Why “Asten”" },
    ],
  },
  {
    title: "Company",
    links: [
      { to: "/about", label: "About" },
      { to: "/brand", label: "Brand & press" },
      { to: "/contact", label: "Contact" },
      { to: "/legal", label: "Legal" },
      { to: "/privacy", label: "Privacy" },
      { to: "/terms", label: "Terms" },
    ],
  },
];

export function Footer() {
  return (
    <footer className="colophon">
      <LiveLotusFrieze count={40} />
      <Frieze kind="blocks" />
      <div className="wrap footer-inner">
        <div className="footer-brand">
          <BrandLink />
          <p className="footer-tagline">{site.tagline}</p>
          <p>A local-first writing desk that marks where a draft contradicts the facts you have settled. It never changes your text.</p>
          <div className="footer-cartouche">
            <CartoucheWordmark />
          </div>
        </div>
        <div className="footer-cols">
          {columns.map((column) => (
            <nav key={column.title} className="footer-col" aria-label={column.title}>
              <h2>{column.title}</h2>
              <ul>
                {column.links.map((link) => (
                  <li key={link.to}>
                    <Link to={link.to}>{link.label}</Link>
                  </li>
                ))}
              </ul>
            </nav>
          ))}
        </div>
      </div>
      <div className="wrap footer-base">
        <p>
          <span>© 2026 {site.legalName}. Asten is a Verne product.</span>
          <span className="no-trackers">
            <Glyph name="eye" /> No cookies. No trackers.
          </span>
          <a href="/llms.txt">llms.txt</a>
          <a href="/journal/rss.xml">RSS</a>
          <a href="/sitemap.xml">Sitemap</a>
        </p>
        <a className="verne-credit" href={site.publisherUrl} target="_blank" rel="noopener noreferrer">
          <img src="/verne-mark.png" alt="" width="26" height="26" loading="lazy" />
          <span>
            Designed and built by <strong>{site.creditName}</strong>
          </span>
        </a>
      </div>
    </footer>
  );
}
