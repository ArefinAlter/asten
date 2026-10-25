import type { CSSProperties } from "react";
import { Link } from "react-router-dom";
import { Glyph } from "../art/Glyphs";
import { FaqList } from "../components/Blocks";
import { Arrow } from "../components/Icons";
import { Answer, ArtFor, CtaBand, PageHero, Related, SectionHead } from "../components/Page";
import { audiences, type Audience } from "../content/audiences";
import { CartoucheName } from "../demos/CartoucheName";
import { ClocksDemo } from "../demos/ClocksDemo";
import { PacksDemo } from "../demos/MiniDemos";
import { PyramidExplorer } from "../demos/PyramidExplorer";
import { WeighDemo } from "../demos/WeighDemo";

function AudienceDemo({ slug }: { slug: string }) {
  switch (slug) {
    case "worldbuilders":
      return <PyramidExplorer />;
    case "screenwriters":
      return <ClocksDemo />;
    case "ai-tinkerers":
      return <PacksDemo />;
    case "fan-fiction":
      return <CartoucheName initial="Oren" />;
    default:
      return <WeighDemo />;
  }
}

export function AudiencePage({ audience }: { audience: Audience }) {
  const others = audiences.filter((item) => item.slug !== audience.slug);
  return (
    <>
      <PageHero title={audience.h1} lede={audience.lede} art={<ArtFor art={audience.art} />}>
        <div className="btn-row">
          <Link to="/download" className="btn btn--primary" data-magnetic>
            Try the pre-alpha
          </Link>
          <Link to="/how-it-works" className="link-arrow">
            How it works <Arrow />
          </Link>
        </div>
      </PageHero>

      <section className="register register--sunk" aria-labelledby="pain-h">
        <div className="wrap">
          <div data-reveal style={{ marginBottom: 40 }}>
            <Answer text={audience.answer} />
          </div>
          <SectionHead id="pain-h" title="Common problems" />
          <div className="card-grid card-grid--3">
            {audience.pains.map((pain, i) => (
              <div key={pain.title} className="card pain" data-reveal style={{ "--d": i } as CSSProperties}>
                <h3>{pain.title}</h3>
                <p>{pain.text}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="register" aria-labelledby="help-h">
        <div className="wrap">
          <SectionHead id="help-h" title="How Asten helps" />
          <div className="card-grid card-grid--2">
            {audience.helps.map((help, i) => (
              <Link key={help.title} to={help.href} className="card card--carved" data-reveal style={{ "--d": i % 2 } as CSSProperties}>
                <h3>{help.title}</h3>
                <p>{help.text}</p>
                <span className="card-more">
                  Learn more <Arrow />
                </span>
              </Link>
            ))}
          </div>
        </div>
      </section>

      <section className="register register--raised" aria-labelledby="scene-h">
        <div className="wrap scenario-grid">
          <div data-reveal>
            <h2 id="scene-h">{audience.scenario.title}</h2>
            <ol className="scenario">
              {audience.scenario.lines.map((line, i) => (
                <li key={i}>{line}</li>
              ))}
            </ol>
          </div>
          <div className="demo-frame" data-reveal data-llms="skip">
            <AudienceDemo slug={audience.slug} />
          </div>
        </div>
      </section>

      <section className="register register--sunk" aria-labelledby="honest-h">
        <div className="wrap wrap-narrow">
          <SectionHead id="honest-h" title="Limits" />
          <ul className="honest-list" data-reveal>
            {audience.honest.map((line) => (
              <li key={line}>{line}</li>
            ))}
          </ul>
        </div>
      </section>

      <section className="register" aria-labelledby="afaq-h">
        <div className="wrap wrap-narrow">
          <SectionHead id="afaq-h" title={`Asten for ${audience.name.toLowerCase()}`} />
          <FaqList items={audience.faqs} />
        </div>
      </section>

      <section className="register register--sunk" aria-labelledby="others-h">
        <div className="wrap">
          <SectionHead id="others-h" title="Asten for other writers" />
          <div className="feature-strip">
            {others.map((item) => (
              <Link key={item.slug} to={item.path} className="feature-chip">
                <Glyph name={item.glyph} />
                <span>{item.name}</span>
              </Link>
            ))}
          </div>
        </div>
      </section>

      <Related paths={audience.related} />
      <CtaBand />
    </>
  );
}
