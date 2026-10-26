import type { CSSProperties } from "react";
import { Answer, CtaBand, PageHero, Related } from "../components/Page";
import { roadmap } from "../content/roadmap";

export function Roadmap() {
  return (
    <>
      <PageHero
        title="Shipped, next, later, and never."
        lede="What Asten does today, what is being built next, what comes later, and what Asten will not do. There are no dates."
      />
      <section className="register register--sunk" aria-label="Summary">
        <div className="wrap">
          <div data-reveal>
            <Answer text="The current code includes the desk, canon with fifteen rules in seven families, rule-based reading of the page, method packs, cast, two clocks with calendars, optional world tools, compile and import, the Word and Docs add-ins, generated API contracts, and local-first storage. Next: verified and signed installers, tested Android and iOS shells, and verified ARM builds. Never: writing your prose or training on it." />
          </div>
        </div>
      </section>
      <section className="register" aria-label="Roadmap lanes">
        <div className="wrap lanes">
          {roadmap.map((lane, i) => (
            <section key={lane.id} className={`lane lane--${lane.id}`} aria-labelledby={`lane-${lane.id}`} data-reveal style={{ "--d": i } as CSSProperties}>
              <header>
                <h2 id={`lane-${lane.id}`}>{lane.title}</h2>
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
      <Related paths={["/download", "/ai", "/features", "/faq"]} />
      <CtaBand />
    </>
  );
}
