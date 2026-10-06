import { Navigate, useLocation } from "react-router-dom";
import { pageByPath, site } from "../seo/catalog";

export function Record() {
  const { pathname } = useLocation();
  const page = pageByPath(pathname);
  if (!page || page.kind === "story") return <Navigate to="/" replace />;

  return (
    <main className="page record">
      <p className="eyebrow">
        {page.kind === "legal" ? "Verne · legal" : "Verne · Asten"}
      </p>
      <h1>{page.h1}</h1>
      <p className="lede">{page.answer}</p>
      {page.blocks.map((block) => (
        <section key={block.heading}>
          <h2>{block.heading}</h2>
          {block.paragraphs.map((paragraph, index) => (
            <p key={`${block.heading}-${index}`}>{paragraph}</p>
          ))}
        </section>
      ))}
      {page.questions?.map((item) => (
        <section key={item.question} className="question">
          <h2>{item.question}</h2>
          <p>{item.answer}</p>
        </section>
      ))}
      <p className="colophon">
        Updated {site.updatedLabel}. Asten is a Verne product.
      </p>
    </main>
  );
}
