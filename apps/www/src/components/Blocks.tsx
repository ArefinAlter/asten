import { Fragment } from "react";
import type { Block, QA, Section } from "../content/types";
import { Bloom } from "./Icons";
import { Inline } from "./Inline";

export function BlockView({ block }: { block: Block }) {
  switch (block.kind) {
    case "p":
      return (
        <p>
          <Inline text={block.text} />
        </p>
      );
    case "h3":
      return (
        <h3>
          <Inline text={block.text} />
        </h3>
      );
    case "list": {
      const items = block.items.map((item, i) => (
        <li key={i}>
          <Inline text={item} />
        </li>
      ));
      return block.ordered ? <ol>{items}</ol> : <ul>{items}</ul>;
    }
    case "quote":
      return (
        <blockquote>
          <Inline text={block.text} />
          {block.cite ? <cite>{block.cite}</cite> : null}
        </blockquote>
      );
    case "callout":
      return (
        <aside className={`callout callout--${block.tone ?? "note"}`}>
          {block.title ? <strong className="callout-title">{block.title}</strong> : null}
          <Inline text={block.text} />
        </aside>
      );
    case "diagnostic":
      return <Diagnostic code={block.code} title={block.title} lines={block.lines} />;
    case "terms":
      return (
        <dl className="terms">
          {block.items.map((item) => (
            <Fragment key={item.term}>
              <dt>{item.term}</dt>
              <dd>
                <Inline text={item.text} />
              </dd>
            </Fragment>
          ))}
        </dl>
      );
    case "table":
      return (
        <div className="table-wrap" tabIndex={0} role="region" aria-label={block.caption ?? "Table"}>
          <table>
            {block.caption ? <caption>{block.caption}</caption> : null}
            <thead>
              <tr>
                {block.head.map((cell) => (
                  <th key={cell} scope="col">
                    {cell}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {block.rows.map((row, r) => (
                <tr key={r}>
                  {row.map((cell, c) => (
                    <td key={c}>
                      <Inline text={cell} />
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      );
  }
}

export function Diagnostic({ code, title, lines }: { code: string; title: string; lines: string[] }) {
  return (
    <figure className="diagnostic" aria-label={`Asten diagnostic ${code}`}>
      <div className="diagnostic-head">
        <span className="chip chip--code">{code}</span>
        <span className="diagnostic-title">{title}</span>
      </div>
      <ul>
        {lines.map((line, i) => (
          <li key={i}>{line}</li>
        ))}
      </ul>
    </figure>
  );
}

export function Sections({ sections, level = 2 }: { sections: Section[]; level?: 2 | 3 }) {
  const Heading = level === 2 ? "h2" : "h3";
  return (
    <>
      {sections.map((section) => (
        <section key={section.id} id={section.id} aria-labelledby={`${section.id}-h`}>
          <Heading id={`${section.id}-h`}>{section.heading}</Heading>
          {section.blocks.map((block, i) => (
            <BlockView key={i} block={block} />
          ))}
        </section>
      ))}
    </>
  );
}

export function FaqList({ items, openFirst = false }: { items: QA[]; openFirst?: boolean }) {
  return (
    <div className="faq">
      {items.map((item, i) => (
        <details key={item.q} open={openFirst && i === 0}>
          <summary>
            <span>{item.q}</span>
            <Bloom />
          </summary>
          <div className="faq-answer">
            <Inline text={item.a} />
          </div>
        </details>
      ))}
    </div>
  );
}
