import type { ReactNode } from "react";
import { Link, useLocation } from "react-router-dom";
import { IbisStandard } from "../art/Figures";
import { Glyph } from "../art/Glyphs";
import { Plate } from "../personality/Plate";
import type { PlateKey } from "../personality/plates";
import type { ArtKey } from "../content/types";
import { breadcrumbsFor, metaFor } from "../routes";
import { release } from "../content/release";
import { Arrow } from "./Icons";
import { Inline } from "./Inline";

const plateFor: Record<ArtKey, PlateKey> = {
  scroll: "scribes",
  scales: "weighing",
  ibis: "scribe",
  moon: "dusk",
  pyramid: "cliffs",
  palette: "palette",
  lotus: "ibis",
  eye: "lamp",
};

/** The painted plate for a page: a museum label under it, or none (`bare`) when it sits as a thumbnail on a card. */
export function ArtFor({ art, className, bare = false }: { art: ArtKey; className?: string; bare?: boolean }) {
  return <Plate name={plateFor[art]} className={`${bare ? "plate--thumb " : ""}${className ?? ""}`.trim()} label={!bare} />;
}

/** A rolled papyrus, tied, with a red rubric seal. */
export function ScrollArt({ className }: { className?: string }) {
  return (
    <svg className={`art scroll-art ${className ?? ""}`} viewBox="0 0 260 200" aria-hidden="true">
      <path className="f-papyrus o" d="M40 46 H 214 V 168 H 40 Z" />
      {[70, 84, 98, 112, 126, 140].map((y, i) => (
        <path key={y} className="f-none s-line w15" d={`M62 ${y} H ${i === 3 ? 140 : 192}`} opacity="0.7" />
      ))}
      <path className="f-none s-red w3" d="M146 112 C 156 108, 168 116, 180 110" />
      <rect className="f-sand o" x="24" y="34" width="26" height="146" rx="13" />
      <rect className="f-sand o" x="204" y="34" width="26" height="146" rx="13" />
      <ellipse className="f-shadow o-thin" cx="37" cy="34" rx="13" ry="5" />
      <ellipse className="f-shadow o-thin" cx="217" cy="34" rx="13" ry="5" />
      <circle className="f-red o" cx="217" cy="108" r="9" />
    </svg>
  );
}

export function Breadcrumbs() {
  const { pathname } = useLocation();
  const trail = breadcrumbsFor(metaFor(pathname));
  if (trail.length < 2) return null;
  return (
    <nav className="breadcrumbs" aria-label="Breadcrumb">
      <ol>
        {trail.map((crumb, i) => (
          <li key={crumb.path}>
            {i < trail.length - 1 ? <Link to={crumb.path}>{crumb.label}</Link> : <span aria-current="page">{crumb.label}</span>}
          </li>
        ))}
      </ol>
    </nav>
  );
}

export function PageHero({
  title,
  lede,
  art,
  children,
}: {
  title: ReactNode;
  lede?: ReactNode;
  art?: ReactNode;
  children?: ReactNode;
}) {
  return (
    <header className="page-hero">
      <div className="wrap">
        <Breadcrumbs />
        <div className={art ? "page-hero-grid" : undefined}>
          <div>
            <h1>{title}</h1>
            {lede ? <p className="lede">{lede}</p> : null}
            {children}
          </div>
          {art ? <div className="page-hero-art">{art}</div> : null}
        </div>
      </div>
    </header>
  );
}

export function Answer({ text, label = "In short" }: { text: string; label?: string }) {
  return (
    <section className="answer" aria-label={label}>
      <Glyph name="feather" />
      <div>
        <h2>{label}</h2>
        <p>
          <Inline text={text} />
        </p>
      </div>
    </section>
  );
}

export function Related({ paths, title = "Keep reading" }: { paths: string[]; title?: string }) {
  return (
    <section className="register" aria-labelledby="related-h">
      <div className="wrap">
        <div className="section-head">
          <h2 id="related-h">{title}</h2>
        </div>
        <div className="related">
          {paths.map((path) => {
            const meta = metaFor(path.split("#")[0] ?? path);
            return (
              <Link key={path} to={path}>
                <span>{meta.label}</span>
                <Arrow />
              </Link>
            );
          })}
        </div>
      </div>
    </section>
  );
}

export function CtaBand({
  title = "Try the pre-alpha",
  text = "Open the workbench in your browser on a sample novel, or download release 0.0.1a for Windows or Linux.",
}: {
  title?: string;
  text?: string;
}) {
  return (
    <section className="register register--sunk" aria-labelledby="cta-h">
      <div className="wrap cta-band" data-reveal>
        <IbisStandard />
        <div>
          <h2 id="cta-h">{title}</h2>
          <p>{text}</p>
        </div>
        <div className="btn-row">
          <a className="btn btn--primary" href={release.hostedUrl} data-magnetic>
            Try the pre-alpha
          </a>
          <Link className="link-arrow" to="/download">
            Download for Windows or Linux <Arrow />
          </Link>
        </div>
      </div>
    </section>
  );
}

export function SectionHead({
  title,
  text,
  id,
  center = false,
}: {
  title: ReactNode;
  text?: ReactNode;
  id?: string;
  center?: boolean;
}) {
  return (
    <div className={`section-head${center ? " center" : ""}`} data-reveal>
      <h2 id={id}>{title}</h2>
      {text ? <p>{text}</p> : null}
    </div>
  );
}
