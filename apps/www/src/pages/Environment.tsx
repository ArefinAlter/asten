import { HarborStroke } from "../art/Ink";
import { Reveal } from "../components/Reveal";

export function Environment() {
  return (
    <main className="page">
      <p className="eyebrow">The desk</p>
      <h1>A writing environment, not a typesetter.</h1>
      <p className="lede">
        Scenes, a chapter tree, italics, comments, find, counts, a manuscript
        page view, and track changes. Draft in flow. Paginate when you ask
        how long the book feels.
      </p>
      <HarborStroke className="spread-art" />

      <Reveal className="cards">
        <article>
          <h2>The page</h2>
          <p>
            Georgia on paper. Headings, emphasis, scene breaks. Diagnostics
            sit in the gutter the way an editor’s pencil would.
          </p>
        </article>
        <article>
          <h2>Page view</h2>
          <p>
            Letter, one-inch margins, a running header. Honest page count.
            Not InDesign. Print houses still get a file.
          </p>
        </article>
        <article>
          <h2>Track changes</h2>
          <p>
            Accept, reject, filter by who touched the line. Built on the
            same operations that keep provenance. Export a redline if an
            editor only has Word.
          </p>
        </article>
        <article>
          <h2>Compile</h2>
          <p>
            The binder becomes a manuscript. .docx, PDF, Fountain. Take it
            with you. Bring the draft home.
          </p>
        </article>
      </Reveal>
    </main>
  );
}
