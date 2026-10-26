import type { CSSProperties } from "react";
import { Link } from "react-router-dom";
import { Frieze } from "../art/Friezes";
import { Arrow } from "../components/Icons";
import { PagePlate } from "../personality/Plate";
import { CtaBand, PageHero } from "../components/Page";
import { articleList as articles } from "../content/journal/meta";
import { formatDate } from "./format";

export function JournalIndex() {
  const [lead, ...rest] = articles;
  return (
    <>
      <PageHero
        title="Journal"
        lede="Essays and practical guides on keeping long stories consistent, handling time, building worlds without drowning in them, and using AI as an editor rather than a ghostwriter."
      >
        <a href="/journal/rss.xml" className="link-arrow">
          Subscribe by RSS <Arrow />
        </a>
      </PageHero>
      <Frieze kind="water" />
      <section className="register register--sunk" aria-label="Articles">
        <div className="wrap">
          {lead ? (
            <Link to={`/journal/${lead.slug}`} className="card article-lead" data-reveal>
              <div className="art-slot">
                <PagePlate path={`/journal/${lead.slug}`} bare />
              </div>
              <div>
                <span className="article-meta">
                  {formatDate(lead.published)} · {lead.minutes} min read
                </span>
                <h2>{lead.title}</h2>
                <p>{lead.dek}</p>
                <span className="card-more">
                  Read the essay <Arrow />
                </span>
              </div>
            </Link>
          ) : null}
          <div className="card-grid card-grid--2" style={{ marginTop: 22 }}>
            {rest.map((article, i) => (
              <Link
                key={article.slug}
                to={`/journal/${article.slug}`}
                className="card article-card"
                data-reveal
                style={{ "--d": i % 2 } as CSSProperties}
              >
                <div className="art-slot">
                  <PagePlate path={`/journal/${article.slug}`} bare />
                </div>
                <span className="article-meta">
                  {formatDate(article.published)} · {article.minutes} min read
                </span>
                <h2 className="h3">{article.title}</h2>
                <p>{article.dek}</p>
                <span className="card-more">
                  Read <Arrow />
                </span>
              </Link>
            ))}
          </div>
        </div>
      </section>
      <CtaBand />
    </>
  );
}
