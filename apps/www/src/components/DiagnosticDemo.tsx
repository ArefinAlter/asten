import { useEffect, useState } from "react";
import { BrushUnderline } from "../art/Ink";

export function DiagnosticDemo() {
  const [phase, setPhase] = useState<"idle" | "mark" | "note" | "suppressed">(
    "idle",
  );

  useEffect(() => {
    const mark = window.setTimeout(() => setPhase("mark"), 700);
    const note = window.setTimeout(() => setPhase("note"), 1400);
    return () => {
      window.clearTimeout(mark);
      window.clearTimeout(note);
    };
  }, []);

  return (
    <figure className={`demo ${phase}`}>
      <div className="demo-page">
        <p className="eyebrow">From the manuscript</p>
        <p className="demo-prose">
          Mira waited at the dusk harbor. She pushed her{" "}
          <span className="fault">
            black
            <BrushUnderline className="fault-stroke" />
          </span>{" "}
          hair out of her eyes and listened for a tide that spoke.
        </p>
      </div>
      <aside className="demo-gutter" aria-live="polite">
        {phase !== "suppressed" ? (
          <button
            type="button"
            className="gutter-note"
            onClick={() => setPhase("suppressed")}
          >
            <span className="eyebrow">CANON-E013</span>
            <strong>Settled: auburn</strong>
            <em>The sentence stays yours. Click to suppress.</em>
          </button>
        ) : (
          <p className="suppressed">Suppressed. The harbor is still there.</p>
        )}
      </aside>
    </figure>
  );
}
