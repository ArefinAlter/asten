import { Link } from "react-router-dom";
import { BrushWash, IbisMark, IcebergMark, ReedPen, WaxSeal, Wordmark } from "../art/Ink";
import { DiagnosticDemo } from "../components/DiagnosticDemo";
import { Reveal } from "../components/Reveal";

export function Home() {
  return (
    <main>
      <section className="hero">
        <BrushWash className="hero-wash" />
        <p className="eyebrow">A Verne product</p>
        <h1>
          <Wordmark large />
        </h1>
        <p className="lede">
          Named for Thoth in another tongue. Asten, published by Verne,
          notices a contradiction in world, character, or time — and leaves
          the sentence yours.
        </p>
        <ReedPen className="hero-pen" />
        <div className="actions">
          <Link to="/download" className="cta">
            Open the library
          </Link>
          <Link to="/continuity" className="quiet">
            See a continuity check
          </Link>
        </div>
      </section>

      <Reveal>
        <DiagnosticDemo />
      </Reveal>

      <Reveal className="split">
        <div>
          <p className="eyebrow">Not a generator</p>
          <h2>AI as the editor, not the writer.</h2>
          <p>
            Asten keeps the manuscript on this machine. It can point at a
            span. It does not pick up the pen. World tools stay optional. A
            gardener starts a scene. An architect may pin a harbor first.
            Neither path is the gate.
          </p>
        </div>
        <div className="iceberg-block">
          <IcebergMark className="iceberg-art" />
          <p>Most of the world stays underwater. The novels show only the tip.</p>
        </div>
      </Reveal>

      <Reveal className="cards">
        <article>
          <h2>The desk</h2>
          <p>
            A binder, a page, comments, find, counts, a letter-sized view, and
            track changes. Paginate when you ask how long the book feels.
          </p>
        </article>
        <article>
          <h2>The compiler</h2>
          <p>
            Confirmed facts, two clocks, a settled death. Problems point at
            evidence. You can suppress a rule.
          </p>
        </article>
        <article>
          <h2>The guests</h2>
          <p>
            Compile to Markdown, Fountain, HTML, and Word. Import the draft
            home. Word and Docs can file a selection to the machine. Canon
            does not leave it.
          </p>
        </article>
      </Reveal>

      <Reveal className="seals-row">
        <WaxSeal label="Asked why" />
        <WaxSeal label="A place" />
        <WaxSeal label="A rule holds" />
        <WaxSeal label="Wrote anyway" />
      </Reveal>

      <Reveal className="split">
        <div>
          <p className="eyebrow">The name</p>
          <h2>Asten, another name for Thoth.</h2>
          <p>
            The ibis-headed scribe. Writing, the moon, the weighing of words.
            We kept paper, ink, and ochre. We did not build a temple theme.
          </p>
          <Link to="/name" className="quiet">
            Why the name
          </Link>
        </div>
        <IbisMark className="ibis-art" />
      </Reveal>
    </main>
  );
}
