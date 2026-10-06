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
              <span className="dot" /> Early access · writing desk with a memory
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
                Get early access
              </Link>
              <Link to="/how-it-works" className="link-arrow">
                See how it works <Arrow />
              </Link>
            </div>
            <ul className="trust" aria-label="Promises">
              <li>No AI writes your prose</li>
              <li>No account</li>
              <li>No cloud</li>
              <li>Never trained on</li>
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
            <p className="stela-caption">Hover or tap the scales. Thoth records; you decide.</p>
          </div>
        </div>
      </section>

      <Frieze kind="khekher" />

      <section className="register register--sunk" aria-labelledby="problem-h">
        <div className="wrap">
          <SectionHead
            id="problem-h"
            eyebrow="The problem"
            title={
              <>
                Long stories <em>forget</em> themselves.
              </>
            }
            text="Somewhere past chapter twenty, every long book starts to contradict itself. Not because you’re careless, but because nobody can hold 100,000 words in their head."
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
            Readers notice. Editors notice. Asten notices first, and tells only you.
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
            <p className="eyebrow">The scribe’s palette</p>
            <h2 id="inks-h">
              Black ink is yours. <em>Red ink</em> is ours.
            </h2>
            <p className="lede" style={{ marginTop: 18 }}>
              An Egyptian scribe’s palette held two inks: black for the text, red to mark it. Asten works the same way. Everything
              you write stays in black and untouched. Asten only ever adds red: a mark in the margin, a code, both sides of the
              disagreement, and a choice that is always yours.
            </p>
            <div className="contrast">
              <div className="them">
                <h3>AI writing tools often</h3>
                <ul>
                  <li>offer to write the scene</li>
                  <li>smooth every voice into one</li>
                  <li>send your draft to a server</li>
                </ul>
              </div>
              <div className="us">
                <h3>Asten</h3>
                <ul>
                  <li>never writes a word</li>
                  <li>points at evidence, then steps back</li>
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
            eyebrow="Try it"
            title="Weigh a sentence."
            text="Mira’s hair was settled as auburn in chapter 2. Change the word in chapter 19 and watch what Asten does. And what it doesn’t."
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
            eyebrow="How it works"
            title="Write. Settle. Notice. You decide."
            text="No setup ritual, no forms. Asten grows a record of your story as you write, and only speaks when the record and the page disagree."
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
              The longer tour <Arrow />
            </Link>
          </p>
        </div>
      </section>

      <section className="register register--sunk" aria-labelledby="desk-h">
        <div className="wrap">
          <SectionHead
            id="desk-h"
            eyebrow="The desk"
            title="A desk you’d want to sit at."
            text="Binder on the left, a plain page in the middle, Asten’s red notes in the margin. Click a scene in the binder."
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
            eyebrow="Features"
            title="Everything a long story needs. Nothing that writes it."
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
            <p className="eyebrow">Two clocks</p>
            <h2 id="time-h">The order things happen, and the order you tell them.</h2>
            <p className="lede" style={{ marginTop: 18 }}>
              Asten keeps story time and reading order apart, so a flashback is a choice and not an accident. Nudge the days and
              watch the threads cross.
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
            eyebrow="World, optional"
            title={
              <>
                Most of the world stays <em>below the sand</em>.
              </>
            }
            text="Readers see the pyramid. Your maps, histories, and rules live in the chambers underneath, as sparks, sketches, or settled canon. Build as much or as little as the book needs."
          />
          <div data-reveal>
            <PyramidExplorer />
          </div>
        </div>
      </section>

      <section className="register register--sunk" aria-labelledby="who-h">
        <div className="wrap">
          <SectionHead id="who-h" eyebrow="Who it’s for" title="Made for people who write long, and care how." />
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
            <p className="eyebrow">Local-first</p>
            <h2 id="privacy-h">Protected the oldest way: it never leaves home.</h2>
            <p className="lede" style={{ marginTop: 18 }}>
              The wedjat eye was worn for protection. Asten’s protection is simpler: your whole project is one file on your own
              disk, and nothing in Asten sends it anywhere.
            </p>
            <div className="promises">
              {[
                { glyph: "house" as const, title: "One file, yours", text: "A SQLite file on your computer." },
                { glyph: "door" as const, title: "No account", text: "Nothing to sign up for or lose." },
                { glyph: "eye" as const, title: "No telemetry", text: "Asten doesn’t phone home. Neither does this site." },
                { glyph: "feather" as const, title: "Never trained on", text: "Not by Verne, not by anyone." },
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
            <p className="eyebrow">For the tinkerers</p>
            <h2 id="hood-h">Deterministic, inspectable, and on your machine.</h2>
            <p className="lede" style={{ marginTop: 18 }}>
              No model runs inside Asten today. The checks are rules with stable codes that produce the same marks every time. When
              AI arrives, it will only propose facts for you to confirm, locally first, with your own key if you choose.
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
          <figure className="terminal" data-reveal aria-label="An example of Asten’s diagnostics">
            <div className="terminal-bar" aria-hidden="true">
              <i />
              <i />
              <i />
            </div>
            <pre>
              <span className="t-red">error[CANON-E013]</span>: attribute contradiction{"\n"}
              <span className="t-dim">  --&gt; book-one/ch19-veld: paragraph 2</span>{"\n"}
              {"\n"}
              {"   “Mira pushed her black hair out of her eyes.”\n"}
              <span className="t-red">{`${" ".repeat(20)}^^^^^`}</span>
              {"\n\n"}
              {"   settled: Mira · hair_color · auburn\n"}
              <span className="t-dim">{"   source:  ch02-arrival, paragraph 7\n"}</span>
              {"\n"}
              <span className="t-gold">   actions</span>
              {": keep · update canon · add change · silence\n\n"}
              <span className="t-green">✓</span>
              {" 0 words written by Asten"}
            </pre>
          </figure>
        </div>
      </section>

      <section className="register" aria-labelledby="status-h">
        <div className="wrap">
          <SectionHead
            id="status-h"
            eyebrow="Honest status"
            title="Early, and honest about it."
            text="Asten is young. Here is exactly where it stands, so nobody is surprised."
          />
          <div className="status-grid" data-reveal>
            <div className="status-col">
              <h3>Works today</h3>
              <ul>
                <li>The desk: binder, comments, page view, track changes, snapshots</li>
                <li>Canon with three continuity rules</li>
                <li>Cast, two clocks, optional world</li>
                <li>Compile and import: Markdown, Fountain, HTML, .docx</li>
              </ul>
            </div>
            <div className="status-col" style={{ "--status": "var(--gold)" } as CSSProperties}>
              <h3>Not yet</h3>
              <ul>
                <li>A public download or signed installer</li>
                <li>Facts proposed automatically from prose</li>
                <li>Any AI model, local or cloud</li>
                <li>EPUB, FDX, and collaboration</li>
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
          <SectionHead id="journal-h" eyebrow="From the journal" title="Craft notes from the scribe’s desk." />
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
          <SectionHead id="faq-h" eyebrow="Questions" title="Asked, and answered plainly." />
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
