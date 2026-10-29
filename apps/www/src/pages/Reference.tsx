import { useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { Glyph } from "../art/Glyphs";
import { Diagnostic, FaqList } from "../components/Blocks";
import { Arrow } from "../components/Icons";
import { Inline, plainText } from "../components/Inline";
import { Answer, CtaBand, PageHero, Related } from "../components/Page";
import { audiences } from "../content/audiences";
import { faq } from "../content/faq";
import { features } from "../content/features";
import { glossary } from "../content/glossary";
import { roadmap } from "../content/roadmap";
import { site } from "../site";

export function Glossary() {
  const [query, setQuery] = useState("");
  const shown = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return glossary;
    return glossary.filter((term) => `${term.term} ${plainText(term.short)} ${plainText(term.long ?? "")}`.toLowerCase().includes(q));
  }, [query]);
  const letters = Array.from(new Set(glossary.map((term) => term.term[0]!.toUpperCase())));

  return (
    <>
      <PageHero
        title="Glossary"
        lede="Plain definitions of the terms used in Asten and in conversations about continuity, story structure, and local-first software."
      />
      <section className="register register--sunk" aria-label="Glossary">
        <div className="wrap glossary-layout">
          <div className="glossary-tools">
            <label className="clocks-label" htmlFor="glossary-filter">
              Filter terms
            </label>
            <input
              id="glossary-filter"
              className="cartouche-input"
              type="search"
              placeholder="canon, story time…"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
            />
            <nav className="letters" aria-label="Jump to letter">
              {letters.map((letter) => {
                const first = glossary.find((term) => term.term[0]!.toUpperCase() === letter);
                return (
                  <a key={letter} href={`#${first?.slug}`}>
                    {letter}
                  </a>
                );
              })}
            </nav>
            <p className="fine">
              {shown.length} of {glossary.length} terms
            </p>
          </div>
          <dl className="glossary">
            {shown.map((term) => (
              <div key={term.slug} id={term.slug} className="term">
                <dt>
                  <a href={`#${term.slug}`} className="term-anchor" aria-label={`Link to ${term.term}`}>
                    #
                  </a>
                  {term.term}
                </dt>
                <dd>
                  <p className="term-short">
                    <Inline text={term.short} />
                  </p>
                  {term.long ? (
                    <p className="term-long">
                      <Inline text={term.long} />
                    </p>
                  ) : null}
                  {term.see || term.related?.length ? (
                    <p className="term-links">
                      {term.see ? (
                        <Link to={term.see} className="link">
                          See it in context
                        </Link>
                      ) : null}
                      {term.related?.map((slug) => {
                        const other = glossary.find((item) => item.slug === slug);
                        return other ? (
                          <a key={slug} href={`#${slug}`} className="chip">
                            {other.term}
                          </a>
                        ) : null;
                      })}
                    </p>
                  ) : null}
                </dd>
              </div>
            ))}
            {shown.length === 0 ? <p className="fine">No term matches “{query}”.</p> : null}
          </dl>
        </div>
      </section>
      <Related paths={["/faq", "/guide", "/how-it-works", "/journal"]} />
    </>
  );
}

export function Faq() {
  return (
    <>
      <PageHero
        title="Questions and answers"
        lede="Everything people ask about Asten: what it is, whether it uses AI, where your files live, what it checks, how to get it, and who makes it."
      >
        <nav className="faq-jump" aria-label="Question groups">
          {faq.map((group) => (
            <a key={group.id} href={`#${group.id}`} className="chip">
              {group.title}
            </a>
          ))}
        </nav>
      </PageHero>
      {faq.map((group, i) => (
        <section key={group.id} id={group.id} className={`register ${i % 2 === 0 ? "register--sunk" : ""}`} aria-labelledby={`${group.id}-h`}>
          <div className="wrap wrap-narrow">
            <h2 id={`${group.id}-h`} style={{ marginBottom: 26 }}>
              {group.title}
            </h2>
            <FaqList items={group.items} />
          </div>
        </section>
      ))}
      <CtaBand title="Other questions" text={`Write to ${site.email}.`} />
    </>
  );
}

export function Guide() {
  return (
    <>
      <PageHero
        title="The complete guide to Asten"
        lede="A complete description of Asten for readers, reviewers, and answer engines: what it does, what it refuses, how it works, where it stands, and who makes it."
      />
      <section className="register register--sunk" aria-label="Summary">
        <div className="wrap">
          <Answer text={`Asten is an AI-powered local-first writing environment for long fiction and other long-form work, published by ${site.legalName} (Dhaka, Bangladesh). It combines a manuscript desk with an intelligent continuity engine that marks contradictions between the manuscript and facts the author has settled. It hands creativity back to the writer and never generates prose, pairs with an AI assistant that supports BYOK and direct subscriptions (Claude Code, Codex, Antigravity, Cursor, Grok), stores projects in a SQLite file on the author’s computer, and is available as an alpha release (0.1.0-alpha.1) for Windows and Linux.`} />
        </div>
      </section>
      <div className="register">
        <div className="wrap guide">
          <section aria-labelledby="g-what">
            <h2 id="g-what">What Asten is</h2>
            <div className="prose">
              <p>
                Asten is a writing app with a memory. You write in a calm manuscript desk; you settle the details that matter as
                facts; Asten’s compiler marks any page that disagrees with them. Its slogan is the product rule:{" "}
                <strong>{site.tagline}</strong>
              </p>
              <p>
                The name is another name for Thoth, the Egyptian god of writing, who recorded the verdict at the weighing of the
                heart without deciding it. Asten keeps the record; the author decides.{" "}
                <Link to="/name" className="link">
                  More on the name
                </Link>
                .
              </p>
            </div>
          </section>

          <section aria-labelledby="g-features">
            <h2 id="g-features">What it does</h2>
            <div className="guide-features">
              {features.map((feature) => (
                <article key={feature.slug}>
                  <h3>
                    <Glyph name={feature.glyph} /> <Link to={feature.path}>{feature.name}</Link>
                  </h3>
                  <p>
                    <Inline text={feature.answer} />
                  </p>
                </article>
              ))}
            </div>
          </section>

          <section aria-labelledby="g-codes">
            <h2 id="g-codes">The rules that ship today</h2>
            <div className="card-grid card-grid--3">
              <Diagnostic code="CANON-E013" title="Attribute contradiction" lines={["Two values for one attribute at the same story time, no change between", "Settled against settled is an error; the page against canon a warning", "Scribe pack · always on · siblings E014, E015, NAME-W001 to W004"]} />
              <Diagnostic code="CANON-E101" title="Life after a settled death" lines={["A later beat in story time for someone settled as dead", "A scene placed earlier in story time is not marked", "Lifecycle pack · on · siblings E102, E103"]} />
              <Diagnostic code="CANON-E201" title="Backward inside a continuous run" lines={["Dated scenes in a run you declared continuous step backward", "E202 dates that cannot all be true · E203 effect before cause", "Clocks pack · on · also Atlas E301, Knowledge E401, Relations E501–E503, Invariants E601"]} />
            </div>
          </section>

          <section aria-labelledby="g-refuses">
            <h2 id="g-refuses">What it refuses</h2>
            <ul className="honest-list">
              {roadmap
                .find((lane) => lane.id === "never")
                ?.items.map((item) => (
                  <li key={item.title}>
                    <strong>{item.title}.</strong> {item.text}
                  </li>
                ))}
            </ul>
          </section>

          <section aria-labelledby="g-who">
            <h2 id="g-who">Who it is for</h2>
            <ul className="honest-list">
              {audiences.map((audience) => (
                <li key={audience.slug}>
                  <Link to={audience.path} className="link">
                    {audience.name}
                  </Link>
                  : {audience.short}
                </li>
              ))}
            </ul>
          </section>

          <section aria-labelledby="g-status">
            <h2 id="g-status">Where it stands</h2>
            <div className="lanes lanes--compact">
              {roadmap
                .filter((lane) => lane.id !== "never")
                .map((lane) => (
                  <section key={lane.id} className={`lane lane--${lane.id}`}>
                    <header>
                      <h3>{lane.title}</h3>
                      <p>{lane.note}</p>
                    </header>
                    <ul>
                      {lane.items.map((item) => (
                        <li key={item.title}>
                          <strong>{item.title}</strong>
                          <span>{item.text}</span>
                        </li>
                      ))}
                    </ul>
                  </section>
                ))}
            </div>
          </section>

          <section aria-labelledby="g-pub">
            <h2 id="g-pub">Who makes it</h2>
            <div className="prose">
              <p>
                Asten is designed and built by {site.legalName}, registered in Dhaka, Bangladesh.
                Contact: <a href={`mailto:${site.email}`}>{site.email}</a>. Company details:{" "}
                <a href={site.companyDetails} target="_blank" rel="noopener noreferrer">
                  vernesystems.com/legal/company
                </a>
                .
              </p>
              <p>
                For language models: a plain-text map of this site is at <a href="/llms.txt">/llms.txt</a>, and the full text of every
                page is at <a href="/llms-full.txt">/llms-full.txt</a>.
              </p>
            </div>
          </section>
          <p>
            <Link to="/faq" className="link-arrow">
              Every question, answered <Arrow />
            </Link>
          </p>
        </div>
      </div>
      <CtaBand />
    </>
  );
}

