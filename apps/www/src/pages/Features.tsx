import type { CSSProperties } from "react";
import { Link } from "react-router-dom";
import { Frieze } from "../art/Friezes";
import { Glyph } from "../art/Glyphs";
import { Arrow } from "../components/Icons";
import { Answer, CtaBand, PageHero, SectionHead } from "../components/Page";
import { audiences } from "../content/audiences";
import { features } from "../content/features";

export function Features() {
  return (
    <>
      <PageHero
        title="Features"
        lede="A manuscript desk, canon checks, two clocks for story time, optional world tools, and add-ins for Word and Google Docs. Everything runs on your own computer."
      />
      <Frieze kind="lotus" />
      <section className="register register--sunk" aria-label="All features">
        <div className="wrap">
          <div data-reveal style={{ marginBottom: 36 }}>
            <Answer text="Asten’s features are the desk (binder, comments, find, page view, track changes, snapshots), canon checks with stable codes, a cast of entities, two clocks for story time and reading order, optional world tools, method packs, compile and import for Markdown, Fountain, HTML, .docx, EPUB (compile only), and Final Draft, Word and Google Docs add-ins, and local-first storage." />
          </div>
          <div className="card-grid">
            {features.map((feature, i) => (
              <Link key={feature.slug} to={feature.path} className="card card--carved" data-reveal style={{ "--d": i % 3 } as CSSProperties}>
                <Glyph name={feature.glyph} />
                <h2 className="h3">{feature.name}</h2>
                <p>{feature.short}</p>
                <span className="card-more">
                  {feature.name} <Arrow />
                </span>
              </Link>
            ))}
          </div>
        </div>
      </section>
      <section className="register" aria-labelledby="fa-h">
        <div className="wrap">
          <SectionHead id="fa-h" title="Features by kind of writer" />
          <div className="card-grid">
            {audiences.map((audience, i) => (
              <Link key={audience.slug} to={audience.path} className="card" data-reveal style={{ "--d": i } as CSSProperties}>
                <Glyph name={audience.glyph} />
                <h3>{audience.name}</h3>
                <p>{audience.short}</p>
              </Link>
            ))}
          </div>
        </div>
      </section>
      <CtaBand />
    </>
  );
}
