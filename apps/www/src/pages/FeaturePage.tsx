import type { CSSProperties } from "react";
import { Link } from "react-router-dom";
import { Glyph } from "../art/Glyphs";
import { FaqList, Sections } from "../components/Blocks";
import { Arrow } from "../components/Icons";
import { Inline } from "../components/Inline";
import { Answer, ArtFor, CtaBand, PageHero, Related, SectionHead } from "../components/Page";
import { features, type DemoKey, type Feature } from "../content/features";
import { CartoucheName } from "../demos/CartoucheName";
import { ClocksDemo } from "../demos/ClocksDemo";
import { DeskMock } from "../demos/DeskMock";
import { CompileDemo, LocalDemo, PacksDemo } from "../demos/MiniDemos";
import { PyramidExplorer } from "../demos/PyramidExplorer";
import { WeighDemo } from "../demos/WeighDemo";

const demoTitles: Record<DemoKey, { eyebrow: string; title: string }> = {
  weigh: { eyebrow: "Try it", title: "Weigh a sentence." },
  clocks: { eyebrow: "Try it", title: "Nudge the days. Watch the threads." },
  pyramid: { eyebrow: "Explore", title: "Open the chambers." },
  cartouche: { eyebrow: "Play", title: "Put a name in a cartouche." },
  packs: { eyebrow: "Try it", title: "Switch packs on and off." },
  compile: { eyebrow: "Try it", title: "One scene, five formats." },
  local: { eyebrow: "Try it", title: "Switch the Wi-Fi off." },
  desk: { eyebrow: "Look around", title: "Click through the binder." },
};

function Demo({ demo }: { demo: DemoKey }) {
  switch (demo) {
    case "weigh":
      return <WeighDemo />;
    case "clocks":
      return <ClocksDemo />;
    case "pyramid":
      return <PyramidExplorer />;
    case "cartouche":
      return <CartoucheName />;
    case "packs":
      return <PacksDemo />;
    case "compile":
      return <CompileDemo />;
    case "local":
      return <LocalDemo />;
    case "desk":
      return <DeskMock />;
  }
}

export function FeaturePage({ feature }: { feature: Feature }) {
  const others = features.filter((item) => item.slug !== feature.slug);
  return (
    <>
      <PageHero
        eyebrow={`Feature · ${feature.name}`}
        title={feature.h1}
        lede={<Inline text={feature.lede} />}
        art={<ArtFor art={feature.art} />}
      >
        <div className="btn-row">
          <Link to="/download" className="btn btn--primary" data-magnetic>
            Get early access
          </Link>
          <Link to="/how-it-works" className="link-arrow">
            How Asten works <Arrow />
          </Link>
        </div>
      </PageHero>

      <section className="register register--sunk" aria-label="Summary">
        <div className="wrap">
          <div data-reveal>
            <Answer text={feature.answer} />
          </div>
          <div className="card-grid point-grid" style={{ marginTop: 28 }}>
            {feature.points.map((point, i) => (
              <div key={point.title} className="point" data-reveal style={{ "--d": i % 3 } as CSSProperties}>
                <h3>{point.title}</h3>
                <p>
                  <Inline text={point.text} />
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {feature.demo ? (
        <section className="register register--raised" aria-labelledby="demo-h">
          <div className="wrap">
            <SectionHead id="demo-h" eyebrow={demoTitles[feature.demo].eyebrow} title={demoTitles[feature.demo].title} />
            <div className={feature.demo === "desk" ? undefined : "demo-frame"} data-reveal data-llms="skip">
              <Demo demo={feature.demo} />
            </div>
          </div>
        </section>
      ) : null}

      <section className="register" aria-label="Details">
        <div className="wrap">
          <div className="prose" data-reveal>
            <Sections sections={feature.sections} />
          </div>
        </div>
      </section>

      {feature.faqs.length > 0 ? (
        <section className="register register--sunk" aria-labelledby="ffaq-h">
          <div className="wrap wrap-narrow">
            <SectionHead id="ffaq-h" eyebrow="Questions" title={`About ${feature.name.toLowerCase()}`} />
            <FaqList items={feature.faqs} />
          </div>
        </section>
      ) : null}

      <section className="register" aria-labelledby="more-h">
        <div className="wrap">
          <SectionHead id="more-h" eyebrow="More features" title="The rest of the desk." />
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

      <Related paths={feature.related} />
      <CtaBand />
    </>
  );
}
