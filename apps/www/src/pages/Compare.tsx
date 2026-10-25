import { Scales } from "../art/Scales";
import { Answer, CtaBand, PageHero, Related } from "../components/Page";
import { comparisons } from "../content/compare";

export function Compare() {
  return (
    <>
      <PageHero
        title="Where Asten fits, and where it doesn’t."
        lede="Asten is one tool among many on a writer’s desk. Here is how it differs from the kinds of tools you may already use, and when one of those is the better choice."
        art={<Scales tilt={4} left="heart" />}
      />
      <section className="register register--sunk" aria-label="Summary">
        <div className="wrap">
          <div data-reveal>
            <Answer text="Asten is a local-first writing desk with continuity checks. Unlike AI story generators it never writes prose; unlike word processors it keeps a canon and checks the manuscript against it; unlike worldbuilding platforms its world tools are optional and wired to the manuscript; and unlike cloud tools it keeps every file on your computer." />
          </div>
        </div>
      </section>
      <section className="register" aria-label="Comparisons">
        <div className="wrap compare-list">
          {comparisons.map((item) => (
            <article key={item.id} id={item.id} className="compare-row" data-reveal>
              <header>
                <h2>Asten vs. {item.category.toLowerCase()}</h2>
                <p>{item.examples}</p>
              </header>
              <div className="compare-cols">
                <div>
                  <h3>What they do well</h3>
                  <p>{item.theyDo}</p>
                </div>
                <div className="compare-asten">
                  <h3>What Asten does</h3>
                  <p>{item.astenDoes}</p>
                </div>
              </div>
              <p className="compare-choose">{item.chooseThem}</p>
            </article>
          ))}
          <p className="fine" data-reveal>
            Product names belong to their owners and are mentioned only to describe categories. Comparisons are general and
            reflect how these kinds of tools are typically designed, not the details of any particular version.
          </p>
        </div>
      </section>
      <Related paths={["/ai", "/features", "/for/novelists", "/journal/ai-editor-not-writer"]} />
      <CtaBand />
    </>
  );
}
