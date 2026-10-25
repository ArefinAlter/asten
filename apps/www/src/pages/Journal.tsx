import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { FaqList, Sections } from "../components/Blocks";
import { Arrow } from "../components/Icons";
import { Answer, ArtFor, Breadcrumbs, CtaBand, Related } from "../components/Page";
import { articles } from "../content/journal";
import { absoluteUrl } from "../site";
import { formatDate } from "./format";


function CopyLink({ path }: { path: string }) {
  const [done, setDone] = useState(false);
  return (
    <button
      type="button"
      className="share-button"
      onClick={async () => {
        try {
          await navigator.clipboard.writeText(absoluteUrl(path));
          setDone(true);
          window.setTimeout(() => setDone(false), 1800);
        } catch {
          // Clipboard can be blocked; the address bar still has the link.
        }
      }}
    >
      {done ? "Link copied" : "Copy link"}
    </button>
  );
}

function useCurrentSection(ids: string[]) {
  const [current, setCurrent] = useState<string | null>(null);
  useEffect(() => {
    const nodes = ids.map((id) => document.getElementById(id)).filter(Boolean) as HTMLElement[];
    if (nodes.length === 0 || !("IntersectionObserver" in window)) return;
    const observer = new IntersectionObserver(
      (entries) => {
        const visible = entries.filter((entry) => entry.isIntersecting).sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top);
        if (visible[0]) setCurrent(visible[0].target.id);
      },
      { rootMargin: "-20% 0px -65% 0px" },
    );
    nodes.forEach((node) => observer.observe(node));
    return () => observer.disconnect();
  }, [ids]);
  return current;
}

export function ArticlePage({ slug }: { slug: string }) {
  const article = articles.find((item) => item.slug === slug)!;
  const index = articles.findIndex((item) => item.slug === article.slug);
  const next = articles[(index + 1) % articles.length];
  const path = `/journal/${article.slug}`;
  const [ids] = useState(() => article.sections.map((section) => section.id));
  const current = useCurrentSection(ids);

  return (
    <>
      <header className="page-hero article-hero">
        <div className="wrap">
          <Breadcrumbs />
          <div className="page-hero-grid">
            <div>
              <p className="eyebrow">{article.tags.join(" · ")}</p>
              <h1>{article.title}</h1>
              <p className="lede">{article.dek}</p>
              <p className="byline">
                <span>By the Asten team at Verne</span>
                <span>
                  <time dateTime={article.published}>{formatDate(article.published)}</time>
                </span>
                <span>{article.minutes} min read</span>
                <CopyLink path={path} />
              </p>
            </div>
            <div className="page-hero-art">
              <ArtFor art={article.art} />
            </div>
          </div>
        </div>
      </header>

      <section className="register register--sunk article-top" aria-label="Summary">
        <div className="wrap article-summary">
          <Answer text={article.answer} label="The short answer" />
          <div className="keyfacts">
            <h2>Key takeaways</h2>
            <ul>
              {article.takeaways.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </div>
        </div>
      </section>

      <div className="register">
        <div className="wrap article-body">
          <nav className="toc" aria-label="Contents">
            <h2>Contents</h2>
            <ol>
              {article.sections.map((section) => (
                <li key={section.id}>
                  <a href={`#${section.id}`} className={current === section.id ? "is-current" : undefined}>
                    {section.heading}
                  </a>
                </li>
              ))}
            </ol>
          </nav>
          <article className="prose">
            <Sections sections={article.sections} />
          </article>
        </div>
      </div>

      {article.faqs.length > 0 ? (
        <section className="register register--sunk" aria-labelledby="art-faq-h">
          <div className="wrap wrap-narrow">
            <h2 id="art-faq-h" style={{ marginBottom: 28 }}>
              Questions about this article
            </h2>
            <FaqList items={article.faqs} />
          </div>
        </section>
      ) : null}

      {next && next.slug !== article.slug ? (
        <section className="register" aria-label="Next article">
          <div className="wrap">
            <Link to={`/journal/${next.slug}`} className="card article-next">
              <span className="article-meta">Next in the journal</span>
              <h2 className="h3">{next.title}</h2>
              <p>{next.dek}</p>
              <span className="card-more">
                Read next <Arrow />
              </span>
            </Link>
          </div>
        </section>
      ) : null}

      <Related paths={article.related} />
      <CtaBand />
    </>
  );
}

