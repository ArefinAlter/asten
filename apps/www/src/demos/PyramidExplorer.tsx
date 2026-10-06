import { useState } from "react";
import { chambers, PyramidSection } from "../art/Pyramid";

const notes: Record<string, { title: string; text: string; status: string }> = {
  maps: { title: "The atlas", text: "Veld sits east of the reef, two days’ sail from the capital. Pinned on a rough map.", status: "settled" },
  history: { title: "The dossier: history", text: "The siege of 412. Three lines in the book, three pages in your notes.", status: "sketched" },
  factions: { title: "The dossier: cultures", text: "The harbor guild distrusts the capital. Nobody says why, yet.", status: "spark" },
  rules: { title: "The dossier: systems", text: "Tide-reading costs the reader a memory. Settle it, and Asten can hold you to it later.", status: "sketched" },
  families: { title: "The wall", text: "A scrap: “Oren had a sister?” A question pinned for later.", status: "spark" },
};

export function PyramidExplorer() {
  const [active, setActive] = useState<string | null>("maps");
  const note = active ? notes[active] : null;
  return (
    <div className="pyramid-explorer">
      <PyramidSection active={active} onEnter={(id) => id && setActive(id)} />
      <div className="pyramid-panel">
        <p className="clocks-label">Below the sand</p>
        <div className="pyramid-buttons" role="tablist" aria-label="Chambers of the world">
          {chambers.map((chamber) => (
            <button
              key={chamber.id}
              type="button"
              role="tab"
              aria-selected={active === chamber.id}
              className={active === chamber.id ? "is-on" : undefined}
              onClick={() => setActive(chamber.id)}
              onMouseEnter={() => setActive(chamber.id)}
              onFocus={() => setActive(chamber.id)}
            >
              {chamber.label}
            </button>
          ))}
        </div>
        {note ? (
          <div className="pyramid-note" role="tabpanel" aria-live="polite">
            <span className={`chip ${note.status === "settled" ? "chip--settled" : ""}`}>{note.status}</span>
            <h3>{note.title}</h3>
            <p>{note.text}</p>
          </div>
        ) : null}
      </div>
    </div>
  );
}
