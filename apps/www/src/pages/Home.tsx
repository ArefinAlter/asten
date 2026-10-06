import { Link } from "react-router-dom";
import { BrushWash, IcebergMark, WaxSeal, Wordmark } from "../art/Ink";
import { DiagnosticDemo } from "../components/DiagnosticDemo";
import { Reveal } from "../components/Reveal";

export function Home() {
  return (
    <main>
      <section className="hero">
        <BrushWash className="hero-wash" />
        <p className="eyebrow">A writing environment</p>
        <h1>
          <Wordmark large />
        </h1>
        <p className="lede">
          The environment that notices. It underlines a contradiction in
          world, character, or time — and leaves the sentence yours.
        </p>
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
            Margin keeps the manuscript on this machine. It can point at a
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

      <Reveal className="seals-row">
        <WaxSeal label="Asked why" />
        <WaxSeal label="A place" />
        <WaxSeal label="A rule holds" />
        <WaxSeal label="Wrote anyway" />
      </Reveal>

      <Reveal className="closer">
        <h2>Take it to Word. Bring it back.</h2>
        <p>
          Margin is the desk. Word and Docs are guests. Page view, track
          changes, and compile live here. The agency template can wait in
          another window.
        </p>
        <Link to="/interop" className="quiet">
          How the fence works
        </Link>
      </Reveal>
    </main>
  );
}
