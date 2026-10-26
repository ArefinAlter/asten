import { release } from "../content/release";
import type { CSSProperties } from "react";
import { Link } from "react-router-dom";
import { Wedjat } from "../art/Figures";
import { Frieze } from "../art/Friezes";
import { Glyph } from "../art/Glyphs";
import { LunarEmblem, StarField } from "../art/Night";
import { PaletteMark } from "../art/Palette";
import { Scales } from "../art/Scales";
import { Thoth } from "../art/Thoth";
import { FaqList } from "../components/Blocks";
import { Arrow } from "../components/Icons";
import { ArtFor, CtaBand, SectionHead } from "../components/Page";
import { audiences } from "../content/audiences";
import { features } from "../content/features";
import { homeFaqs, howSteps } from "../content/home";
import { articleList as articles } from "../content/journal/meta";
import { ClocksDemo } from "../demos/ClocksDemo";
import { DeskMock } from "../demos/DeskMock";
import { PyramidExplorer } from "../demos/PyramidExplorer";
import { WeighDemo } from "../demos/WeighDemo";
import { Inline } from "../components/Inline";

const column = ["\u{1315D}", "\u{133DE}", "\u{13000}", "\u{1315D}", "\u{133DE}", "\u{13000}", "\u{1315D}", "\u{133DE}", "\u{13000}", "\u{1315D}", "\u{133DE}"];

export function Home() {
  return (
    <>
      <section className="hero" aria-labelledby="hero-h">
        <div className="hero-column hero-column--left" aria-hidden="true">
          {column.map((sign, i) => (
            <span key={i}>{sign}</span>
          ))}
        </div>
        <div className="hero-column hero-column--right" aria-hidden="true">
          {column.map((sign, i) => (
            <span key={i}>{sign}</span>
          ))}
        </div>
        <div className="wrap hero-grid">
          <div className="hero-intro">
            <span className="status-pill">
              <span className="dot" /> Pre-alpha {release.version}
            </span>
            <h1 id="hero-h">
              Let AI be your <em>editor</em>, not your writer.
            </h1>
            <p className="lede">
              Asten remembers the facts you settle about your story, like a hair color, a death, or the day the harbor burned,
              and marks any page that disagrees, in red, in the margin. It never writes a word for you. Your manuscript never
              leaves your computer.
            </p>
            <div className="btn-row">
              <Link to="/download" className="btn btn--primary" data-magnetic>
                Try the pre-alpha
              </Link>
              <Link to="/how-it-works" className="link-arrow">
                See how it works <Arrow />
              </Link>
            </div>
            <ul className="trust" aria-label="Promises">
              <li>No AI writes your prose</li>
              <li>No account</li>
              <li>No cloud</li>
              <li>Not used for training</li>
            </ul>
          </div>

          <div>
            <div className="stela" tabIndex={0} role="img" aria-label="Thoth, the ibis-headed scribe, records a weighing: your page in one pan of the scales, the feather of canon in the other.">
              <div className="stela-sky">
                <StarField rows={3} cols={9} seed={11} />
              </div>
              <Scales className="stela-scales" />
              <Thoth className="stela-thoth" flip />
              <span className="chip chip--code stela-mark">CANON-E013</span>
              <div className="stela-ground">
                <Frieze kind="blocks" />
              </div>
            </div>
          </div>
        </div>
      </section>

      <Frieze kind="khekher" />

      <section className="register register--sunk" aria-labelledby="problem-h">
        <div className="wrap">
          <SectionHead
            id="problem-h"
            title="Continuity slips in a long manuscript"
            text="A long manuscript is written over months, and a detail set down in chapter 2 is hard to keep in mind by chapter 19. Three common kinds of slip:"
          />
          <div className="card-grid card-grid--3">
            {[
              { glyph: "feather" as const, code: "CANON-E013", quote: <>Chapter 2: Mira’s hair is auburn. Chapter 19: she pushes back her <mark>black</mark> hair.</>, text: "An attribute you settled, quietly drifting." },
              { glyph: "seated" as const, code: "CANON-E101", quote: <>Captain Oren dies in chapter 9. In chapter 14 he <mark>pours the tea</mark>.</>, text: "A life that continues after a settled death." },
              { glyph: "moon" as const, code: "CANON-E201", quote: <>The harbor burns on day 17. The next scene is <mark>day 4</mark>, and it isn’t a flashback.</>, text: "Reading order and story time, crossing by accident." },
            ].map((item, i) => (
              <article key={item.code} className="vignette" data-reveal style={{ "--d": i } as CSSProperties}>
                <Glyph name={item.glyph} />
                <blockquote>{item.quote}</blockquote>
                <p>{item.text}</p>
                <p>
                  <span className="chip chip--code">{item.code}</span>
                </p>
              </article>
            ))}
          </div>
          <p className="problem-note" data-reveal>
            Once the facts involved are settled, Asten marks each of these in the margin with the code shown. The marks stay on your computer.
          </p>
        </div>
      </section>

      <section className="register" aria-labelledby="inks-h">
        <div className="wrap inks">
          <div className="inks-art" data-reveal>
            <PaletteMark />
            <p className="inks-legend" aria-hidden="true">
              <span className="l-black">your ink</span>
              <span className="l-red">Asten’s ink</span>
            </p>
          </div>
          <div data-reveal>
            <h2 id="inks-h">Black ink for your text, red for Asten’s marks</h2>
            <p className="lede" style={{ marginTop: 18 }}>
              An Egyptian scribe’s palette held two inks: black for the text, red to mark it. Asten follows the same split. It never
              changes what you write. It adds marks in the margin, each with a code, both sides of the disagreement, and the actions
              you can take.
            </p>
            <div className="contrast">
              <div className="them">
                <h3>AI writing tools often</h3>
                <ul>
                  <li>offer to write the scene</li>
                  <li>suggest rewrites of your sentences</li>
                  <li>send your draft to a server</li>
                </ul>
              </div>
              <div className="us">
                <h3>Asten</h3>
                <ul>
                  <li>does not generate or rewrite text</li>
                  <li>shows the evidence for each mark</li>
                  <li>keeps your files on your machine</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      </section>

      <section className="register register--raised" aria-labelledby="weigh-h">
        <div className="wrap">
          <SectionHead
            id="weigh-h"
            title="Example: an attribute check"
            text="Mira’s hair was settled as auburn in chapter 2. The example shows the mark Asten raises when chapter 19 gives a different color, and the actions it offers. The sentence itself is never changed."
          />
          <div className="demo-frame" data-reveal data-llms="skip">
            <WeighDemo />
          </div>
        </div>
      </section>

      <section className="register" aria-labelledby="how-h">
        <div className="wrap">
          <SectionHead
            id="how-h"
            title="How Asten works"
            text="There is no setup step. Asten builds a record of your story from the facts you settle, and adds a mark only when a page disagrees with that record."
          />
          <ol className="steps" data-reveal>
            {howSteps.map((step, i) => (
              <li key={step.title} className="step">
                <span className="step-num" aria-hidden="true">
                  {Array.from({ length: i + 1 }, (_, k) => (
                    <i key={k} />
                  ))}
                </span>
                <h3>{step.title}</h3>
                <p>
                  <Inline text={step.text} />
                </p>
              </li>
            ))}
          </ol>
          <p style={{ marginTop: 28 }}>
            <Link to="/how-it-works" className="link-arrow">
              How it works in detail <Arrow />
            </Link>
          </p>
        </div>
      </section>

      <section className="register register--sunk" aria-labelledby="desk-h">
        <div className="wrap">
          <SectionHead
            id="desk-h"
            title="The writing desk"
            text="The binder on the left, the page in the middle, and Asten’s marks in the margin."
          />
          <div data-reveal>
            <DeskMock />
          </div>
        </div>
      </section>

      <section className="register" aria-labelledby="features-h">
        <div className="wrap">
          <SectionHead
            id="features-h"
            title="What Asten includes"
          />
          <div className="card-grid card-grid--3">
            {features.map((feature, i) => (
              <Link
                key={feature.slug}
                to={feature.path}
                className="card card--carved"
                data-reveal
                style={{ "--d": i % 3 } as CSSProperties}
              >
                <Glyph name={feature.glyph} />
                <h3>{feature.name}</h3>
                <p>{feature.short}</p>
                <span className="card-more">
                  Read more <Arrow />
                </span>
              </Link>
            ))}
          </div>
        </div>
      </section>

      <section className="register register--night" aria-labelledby="time-h">
        <StarField rows={7} cols={16} seed={3} />
        <div className="wrap night-grid">
          <div data-reveal>
            <LunarEmblem />
            <h2 id="time-h">Story time and reading order</h2>
            <p className="lede" style={{ marginTop: 18 }}>
              Asten records when each scene happens in the story separately from where it sits in the book. It compares the two
              only inside a run of scenes you have declared continuous, so a flashback raises no mark.
            </p>
            <p className="myth">
              <strong>Thoth, keeper of time</strong>A story retold by Plutarch says Thoth won five extra days from the moon, so
              that a birth forbidden on every day of the year could happen on days that belonged to no month.
            </p>
          </div>
          <div className="demo-frame" data-reveal data-llms="skip">
            <ClocksDemo />
          </div>
        </div>
      </section>

      <section className="register" aria-labelledby="world-h">
        <div className="wrap">
          <SectionHead
            id="world-h"
            title="Optional world notes"
            text="Places, histories, and rules can be kept as sparks, sketches, or settled entries. Only settled entries become canon, and none of it is required before you write."
          />
          <div data-reveal>
            <PyramidExplorer />
          </div>
        </div>
      </section>

      <section className="register register--sunk" aria-labelledby="who-h">
        <div className="wrap">
          <SectionHead id="who-h" title="Who Asten is for" />
          <div className="card-grid">
            {audiences.map((audience, i) => (
              <Link key={audience.slug} to={audience.path} className="card" data-reveal style={{ "--d": i } as CSSProperties}>
                <Glyph name={audience.glyph} />
                <h3>{audience.name}</h3>
                <p>{audience.short}</p>
                <span className="card-more">
                  For {audience.name.toLowerCase()} <Arrow />
                </span>
              </Link>
            ))}
          </div>
        </div>
      </section>

      <section className="register" aria-labelledby="privacy-h">
        <div className="wrap privacy-grid">
          <div data-reveal>
            <Wedjat />
          </div>
          <div data-reveal>
            <h2 id="privacy-h">Your project stays on your computer</h2>
            <p className="lede" style={{ marginTop: 18 }}>
              Each project is one SQLite file on your own disk. Asten has no account, no sync, and no telemetry, so nothing in it
              sends your manuscript anywhere.
            </p>
            <div className="promises">
              {[
                { glyph: "house" as const, title: "One file, yours", text: "A SQLite file on your computer." },
                { glyph: "door" as const, title: "No account", text: "Nothing to sign up for or lose." },
                { glyph: "eye" as const, title: "No telemetry", text: "Asten sends no usage data. This site runs no analytics." },
                { glyph: "feather" as const, title: "No training", text: "Your manuscript is not used to train any model." },
              ].map((item) => (
                <div key={item.title} className="promise">
                  <Glyph name={item.glyph} />
                  <strong>{item.title}</strong>
                  <span>{item.text}</span>
                </div>
              ))}
            </div>
            <p style={{ marginTop: 24 }}>
              <Link to="/features/local-first" className="link-arrow">
                How local-first works <Arrow />
              </Link>
            </p>
          </div>
        </div>
      </section>

      <section className="register register--sunk" aria-labelledby="hood-h">
        <div className="wrap hood">
          <div data-reveal>
            <h2 id="hood-h">Deterministic rules, run locally</h2>
            <p className="lede" style={{ marginTop: 18 }}>
              No model runs inside Asten. The checks are rules with stable codes, and the same project always produces the same
              marks. The planned perception models are small local classifiers that propose facts for you to confirm; they do not
              generate text.
            </p>
            <ul className="stack" aria-label="Built with">
              {["Rust", "SQLite", "Axum on 127.0.0.1", "React", "TypeScript", "Tauri"].map((item) => (
                <li key={item} className="chip">
                  {item}
                </li>
              ))}
            </ul>
            <div className="btn-row">
              <Link to="/for/ai-tinkerers" className="link-arrow">
                Asten for AI tinkerers <Arrow />
              </Link>
              <Link to="/ai" className="link-arrow">
                Our AI stance <Arrow />
              </Link>
            </div>
          </div>
          <figure className="terminal" data-reveal aria-label="A diagnostic returned by Asten’s local API">
            <div className="terminal-bar" aria-hidden="true">
              <i />
              <i />
              <i />
            </div>
            <pre tabIndex={0}>
              <span className="t-dim">{"$ curl -s -H \"authorization: Bearer $ASTEN_TOKEN\" \\\n    127.0.0.1:4317/api/v1/projects/$PROJECT/story/diagnostics\n\n"}</span>
              <span className="t-red">CANON-E013</span>
              {"  warning\n"}
              {"Mira's hair was settled as auburn.\n\n"}
              {"① Settled: Mira · hair · auburn\n"}
              {"② Chapter 19 says: “black” (Chapter 19)\n"}
              {"③ No event between an unplaced time and day 40\n   changes Mira's hair.\n"}
              {"Most likely slip: ② (93%)\n"}
              <span className="t-gold">Fixes</span>
              {": Add an event · Mark the scene as a\n   memory · auburn holds until day 39 · Retire\n   “auburn” · Silence this mark · Keep my sentence"}
            </pre>
            <figcaption className="terminal-note">
              Message text as the local API returns it for a sample project, with line breaks added and the other fields left out.
            </figcaption>
          </figure>
        </div>
      </section>

      <section className="register" aria-labelledby="status-h">
        <div className="wrap">
          <SectionHead
            id="status-h"
            title="Current status"
            text="What works in the current code, what does not exist yet, and what Asten will not do."
          />
          <div className="status-grid" data-reveal>
            <div className="status-col">
              <h3>Works today</h3>
              <ul>
                <li>The desk: binder, comments, page view, track changes, snapshots</li>
                <li>Canon with fifteen continuity rules, read from the page and from what you settle</li>
                <li>Cast, two clocks, optional world</li>
                <li>Compile to Markdown, plain text, Fountain, HTML, .docx, EPUB, and FDX; import of all but EPUB</li>
              </ul>
            </div>
            <div className="status-col" style={{ "--status": "var(--gold)" } as CSSProperties}>
              <h3>Not yet</h3>
              <ul>
                <li>Signed installers, and a macOS build</li>
                <li>Android and iOS apps tested on devices</li>
                <li>Trained perception models (the reading is rule-based)</li>
                <li>Collaboration</li>
              </ul>
            </div>
            <div className="status-col" style={{ "--status": "var(--rubric)" } as CSSProperties}>
              <h3>Never</h3>
              <ul>
                <li>Writing your prose</li>
                <li>Training on your manuscript</li>
                <li>Locking your files behind a server</li>
              </ul>
            </div>
          </div>
          <p style={{ marginTop: 26 }}>
            <Link to="/roadmap" className="link-arrow">
              Read the roadmap <Arrow />
            </Link>
          </p>
        </div>
      </section>

      <section className="register register--sunk" aria-labelledby="journal-h">
        <div className="wrap">
          <SectionHead id="journal-h" title="From the journal" />
          <div className="card-grid card-grid--3">
            {articles.slice(0, 3).map((article, i) => (
              <Link
                key={article.slug}
                to={`/journal/${article.slug}`}
                className="card article-card"
                data-reveal
                style={{ "--d": i } as CSSProperties}
              >
                <div className="art-slot">
                  <ArtFor art={article.art} />
                </div>
                <span className="article-meta">{article.minutes} min read</span>
                <h3>{article.title}</h3>
                <p>{article.dek}</p>
                <span className="card-more">
                  Read <Arrow />
                </span>
              </Link>
            ))}
          </div>
        </div>
      </section>

      <section className="register" aria-labelledby="faq-h">
        <div className="wrap wrap-narrow">
          <SectionHead id="faq-h" title="Common questions" />
          <div data-reveal>
            <FaqList items={homeFaqs} />
          </div>
          <p style={{ marginTop: 24 }}>
            <Link to="/faq" className="link-arrow">
              All questions <Arrow />
            </Link>
          </p>
        </div>
      </section>

      <CtaBand />
    </>
  );
}
