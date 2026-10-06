import type { CSSProperties } from "react";
import { LunarEmblem } from "../art/Night";
import { Answer, CtaBand, PageHero, Related } from "../components/Page";
import { roadmap } from "../content/roadmap";

export function Roadmap() {
  return (
    <>
      <PageHero
        eyebrow="Roadmap"
        title="Shipped, next, later, and never."
        lede="What Asten does today, what we’re building, and the lines we won’t cross. No dates until they’re real."
        art={<LunarEmblem />}
      />
      <section className="register register--sunk" aria-label="Summary">
        <div className="wrap">
          <div data-reveal>
            <Answer text="Asten’s early-access builds include the desk, canon with three continuity rules, method packs, cast, two clocks, optional world tools, compile and import, the Word and Docs add-ins, and local-first storage. Next: a signed installer, flashback-aware clocks, fact proposals, and custom calendars. Never: writing your prose or training on it." />
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
