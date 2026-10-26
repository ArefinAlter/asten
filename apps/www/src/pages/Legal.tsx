import { Link } from "react-router-dom";
import { Sections } from "../components/Blocks";
import { Arrow } from "../components/Icons";
import { Breadcrumbs, PageHero } from "../components/Page";
import { PagePlate } from "../personality/Plate";
import { legalDocs, legalUpdated } from "../content/legal";
import { legalList } from "../content/legal-index";
import { site } from "../site";
import { formatDate } from "./format";


export function LegalIndex() {
  return (
    <>
      <PageHero
        title="Legal"
        lede="Every Asten policy in one place. Each starts with a plain-language summary. If something you need is missing, write to us."
      />
      <section className="register register--sunk" aria-label="Policies">
        <div className="wrap">
          <div className="related legal-grid">
            {legalList.map((doc) => (
              <Link key={doc.path} to={doc.path} className="legal-card">
                <span>
                  <strong>{doc.title}</strong>
                  <small>{doc.description}</small>
                </span>
                <Arrow />
              </Link>
            ))}
          </div>
          <p className="fine" style={{ marginTop: 26 }}>
            Updated {formatDate(legalUpdated)}. Publisher: {site.legalName}, {site.address}.
          </p>
        </div>
      </section>
    </>
  );
}

export function LegalPage({ path }: { path: string }) {
  const doc = legalDocs.find((item) => item.path === path)!;
  return (
    <>
      <header className="page-hero legal-hero">
        <div className="wrap">
          <Breadcrumbs />
          <div className="page-hero-grid">
            <div>
              <h1>{doc.title}</h1>
              <p className="byline">
                <span>
                  Updated <time dateTime={legalUpdated}>{formatDate(legalUpdated)}</time>
                </span>
                <span>{site.legalName}</span>
              </p>
            </div>
            <div className="page-hero-art page-hero-art--plate">
              <PagePlate path={doc.path} eager />
            </div>
          </div>
        </div>
      </header>
      <section className="register register--sunk legal-summary-reg" aria-labelledby="short-h">
        <div className="wrap wrap-narrow">
          <div className="keyfacts">
            <h2 id="short-h">The short version</h2>
            <ul>
              {doc.summary.map((line) => (
                <li key={line}>{line}</li>
              ))}
            </ul>
          </div>
        </div>
      </section>
      <div className="register">
        <div className="wrap article-body">
          <nav className="toc" aria-label="Sections">
            <h2>Sections</h2>
            <ol>
              {doc.sections.map((section) => (
                <li key={section.id}>
                  <a href={`#${section.id}`}>{section.heading}</a>
                </li>
              ))}
            </ol>
          </nav>
          <article className="prose legal-prose">
            <Sections sections={doc.sections} />
            <hr />
            <p className="fine">
              Questions about this page? Write to <a href={`mailto:${site.email}`}>{site.email}</a>. All policies:{" "}
              <Link to="/legal">Legal</Link>.
            </p>
          </article>
        </div>
      </div>
    </>
  );
}
