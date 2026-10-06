import { useState } from "react";
import { PaletteMark } from "../art/Palette";

const scenes = [
  {
    id: "harbor",
    title: "The harbor at dawn",
    chapter: "Chapter 19",
    text: [
      "The tide came in grey over the harbor at Veld, carrying the smell of last night’s fires.",
      "Mira pushed her black hair out of her eyes and listened for the bell.",
      "“You’re late,” Oren said, though he had only just arrived himself.",
    ],
    mark: 1,
  },
  {
    id: "storm",
    title: "The storm",
    chapter: "Chapter 18",
    text: [
      "By the third bell the sky had turned the color of old slate.",
      "Oren lashed the tiller and counted the swells under his breath.",
      "Nobody on deck mentioned the capital. Nobody needed to.",
    ],
    mark: -1,
  },
  {
    id: "sails",
    title: "Oren sails",
    chapter: "Chapter 17",
    text: [
      "He left on the morning tide, with salt in the rigging and no farewell worth the name.",
      "Mira watched from the sea wall until the sail was a thumbnail.",
      "Then she went home and wrote it down, because someone should.",
    ],
    mark: -1,
  },
];

/**
 * A still of the desk, with the brand applied inside the product: papyrus
 * chrome, a plain page, and Asten's voice only in red, in the margin.
 */
export function DeskMock() {
  const [active, setActive] = useState("harbor");
  const scene = scenes.find((item) => item.id === active) ?? scenes[0]!;
  return (
    <figure className="desk" aria-label="A preview of the Asten desk" data-llms="skip">
      <div className="desk-bar">
        <span className="desk-dots" aria-hidden="true">
          <i />
          <i />
          <i />
        </span>
        <span className="desk-title">
          <PaletteMark /> The Tide-Reader · Asten
        </span>
        <span className="desk-status">Saved on this computer</span>
      </div>
      <div className="desk-body">
        <nav className="desk-binder" aria-label="Binder (preview)">
          <p className="desk-label">Binder</p>
          <p className="desk-folder">Part two: the reef</p>
          <ul>
            {scenes.map((item) => (
              <li key={item.id}>
                <button type="button" className={item.id === active ? "is-on" : undefined} onClick={() => setActive(item.id)}>
                  <span>{item.title}</span>
                  {item.mark >= 0 ? <i className="desk-dot" aria-label="has a mark" /> : null}
                </button>
              </li>
            ))}
          </ul>
          <p className="desk-label">Cast</p>
          <ul className="desk-cast">
            <li>
              <span className="desk-cart">Mira</span>
            </li>
            <li>
              <span className="desk-cart">Oren</span>
            </li>
          </ul>
        </nav>
        <div className="desk-page" key={scene.id}>
          <p className="desk-chapter">{scene.chapter}</p>
          <h3>{scene.title}</h3>
          {scene.text.map((line, i) => (
            <p key={i} className={i === scene.mark ? "has-mark" : undefined}>
              {i === scene.mark ? (
                <>
                  Mira pushed her <span className="desk-flag">black</span> hair out of her eyes and listened for the bell.
                </>
              ) : (
                line
              )}
            </p>
          ))}
        </div>
        <aside className="desk-margin" aria-label="Margin">
          {scene.mark >= 0 ? (
            <div className="desk-note">
              <span className="chip chip--code">CANON-E013</span>
              <p>Mira’s hair was settled as auburn in chapter 2.</p>
              <p className="desk-note-actions">Keep · Update canon · Add change</p>
            </div>
          ) : (
            <div className="desk-quiet">
              <p>Nothing disagrees with your canon here.</p>
            </div>
          )}
        </aside>
      </div>
    </figure>
  );
}
