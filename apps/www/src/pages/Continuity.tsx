import { Link } from "react-router-dom";
import { DiagnosticDemo } from "../components/DiagnosticDemo";
import { Reveal } from "../components/Reveal";

export function Continuity() {
  return (
    <main className="page">
      <p className="eyebrow">The differentiator</p>
      <h1>It notices what four hundred pages hide.</h1>
      <p className="lede">
        Confirmed facts, settled world cards, and a compiler. Hair color,
        a date, a place that was never invented. Problems point at evidence.
        You can suppress a rule.
      </p>
      <DiagnosticDemo />

      <Reveal className="cards">
        <article>
          <h2>Canon</h2>
          <p>
            A fact is established, inferred, contested, or yours to
            override. Nothing on the wall is canon until you settle it.
          </p>
        </article>
        <article>
          <h2>World</h2>
          <p>
            A cork wall for scraps. A rough map. Ugly triangles are enough.
            The dossier can wait.
          </p>
        </article>
        <article>
          <h2>Time</h2>
          <p>
            Story time and reading order are different clocks. Flashbacks
            are not bugs. Travel becomes an error only after you state a
            constraint.
          </p>
        </article>
      </Reveal>

      <Reveal className="closer">
        <p>
          Craft advice stays a method pack. Asten will not mark “show,
          don’t tell” as a defect.
        </p>
        <Link to="/environment" className="quiet">
          See the desk
        </Link>
      </Reveal>
    </main>
  );
}
