import { Reveal } from "../components/Reveal";

export function Interop() {
  return (
    <main className="page">
      <p className="eyebrow">Guests</p>
      <h1>Word and Docs remain in the house.</h1>
      <p className="lede">
        Asten does not replace them. It sends a clean manuscript out and
        takes prose back. Add-ins come after the file path works.
      </p>

      <Reveal className="cards">
        <article>
          <h2>Export</h2>
          <p>.docx, PDF, Fountain. Enough for an agent who still lives in Word.</p>
        </article>
        <article>
          <h2>Import</h2>
          <p>
            Body, headings, italics, comments. Nested tables and fields stay
            behind. We will publish the subset.
          </p>
        </article>
        <article>
          <h2>Add-ins</h2>
          <p>
            A Word add-in and a Docs add-on that send a selection to Asten
            and show a problem. Canon does not move to Microsoft or Google.
          </p>
        </article>
      </Reveal>
    </main>
  );
}
