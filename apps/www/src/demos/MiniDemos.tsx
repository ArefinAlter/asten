import { useEffect, useState } from "react";
import { Glyph } from "../art/Glyphs";

/* ───────── Packs: a switchboard of rules ───────── */

const packList = [
  { key: "scribe", name: "Scribe", code: "CANON-E013", locked: true, text: "Core canon check. Always on." },
  { key: "lifecycle", name: "Lifecycle", code: "CANON-E101", locked: false, text: "Life after a settled death." },
  { key: "clocks", name: "Clocks", code: "CANON-E201", locked: false, text: "Story time against reading order." },
  { key: "diction", name: "Diction watch", code: "CRAFT-H001", locked: false, text: "Optional filter-word notes." },
] as const;

export function PacksDemo() {
  const [on, setOn] = useState<Record<string, boolean>>({ scribe: true, lifecycle: true, clocks: true, diction: false });
  const count = packList.filter((pack) => on[pack.key]).length;
  return (
    <div className="packs-demo">
      <ul className="packs-list">
        {packList.map((pack) => (
          <li key={pack.key} className={on[pack.key] ? "is-on" : undefined}>
            <div>
              <strong>{pack.name}</strong>
              <span>{pack.text}</span>
            </div>
            <span className="chip chip--code">{pack.code}</span>
            <button
              type="button"
              role="switch"
              aria-checked={on[pack.key]}
              aria-label={`${pack.name} pack`}
              disabled={pack.locked}
              className="switch"
              onClick={() => setOn({ ...on, [pack.key]: !on[pack.key] })}
            >
              <span />
            </button>
          </li>
        ))}
      </ul>
      <div className="packs-page">
        <p>
          <span className={on.scribe ? "mk mk-red" : "mk"} data-code="E013">Mira pushed her black hair</span> back as the bell rang. Oren{" "}
          <span className={on.lifecycle ? "mk mk-red" : "mk"} data-code="E101">poured the tea</span>, which was{" "}
          <span className={on.diction ? "mk mk-hint" : "mk"} data-code="H001">very</span> strong.{" "}
          <span className={on.clocks ? "mk mk-red" : "mk"} data-code="E201">Three days earlier</span>, the harbor had burned.
        </p>
        <p className="packs-count" aria-live="polite">
          {count} {count === 1 ? "pack" : "packs"} on · the page itself never changes
        </p>
      </div>
    </div>
  );
}

/* ───────── Compile: one scene, many formats ───────── */

const outputs: Record<string, string> = {
  Markdown: `## Chapter 19: Veld

The tide came in grey over the harbor at Veld.
Mira pushed her auburn hair out of her eyes.

*“You’re late,”* Oren said.`,
  Fountain: `EXT. HARBOR AT VELD - DAWN

The tide comes in grey. MIRA (30s) pushes
auburn hair out of her eyes.

OREN
You're late.`,
  HTML: `<h2>Chapter 19: Veld</h2>
<p>The tide came in grey over the harbor at Veld.
Mira pushed her auburn hair out of her eyes.</p>
<p><em>“You’re late,”</em> Oren said.</p>`,
  "Plain text": `CHAPTER 19: VELD

The tide came in grey over the harbor at Veld.
Mira pushed her auburn hair out of her eyes.

"You're late," Oren said.`,
  ".docx": `chapter-19-veld.docx

  Heading 2   Chapter 19: Veld
  Normal      The tide came in grey over the harbor…
  Normal      “You’re late,” Oren said.   [italic]`,
};

export function CompileDemo() {
  const [format, setFormat] = useState("Markdown");
  return (
    <div className="compile-demo">
      <div className="compile-tabs" role="tablist" aria-label="Compile format">
        {Object.keys(outputs).map((name) => (
          <button
            key={name}
            type="button"
            role="tab"
            aria-selected={format === name}
            className={format === name ? "is-on" : undefined}
            onClick={() => setFormat(name)}
          >
            {name}
          </button>
        ))}
      </div>
      <pre className="compile-out" role="tabpanel" key={format}>
        <code>{outputs[format]}</code>
      </pre>
    </div>
  );
}

/* ───────── Local: switch the network off, keep writing ───────── */

export function LocalDemo() {
  const [online, setOnline] = useState(true);
  const [words, setWords] = useState(81240);
  useEffect(() => {
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduced) return;
    const id = window.setInterval(() => setWords((w) => w + 1 + Math.floor(Math.random() * 3)), 900);
    return () => window.clearInterval(id);
  }, []);
  return (
    <div className={`local-demo ${online ? "is-online" : "is-offline"}`}>
      <div className="local-house">
        <Glyph name="house" />
        <div>
          <p className="clocks-label">Your computer</p>
          <p className="local-file">
            <code>asten.db</code> · {words.toLocaleString("en-US")} words · saved
          </p>
        </div>
      </div>
      <div className="local-wire" aria-hidden="true">
        <span className="local-cut">nothing sent</span>
      </div>
      <div className="local-verne">
        <p className="clocks-label">Verne</p>
        <p>receives: nothing</p>
      </div>
      <button type="button" role="switch" aria-checked={online} className="local-toggle" onClick={() => setOnline(!online)}>
        <span className="switch" aria-hidden="true">
          <span />
        </span>
        {online ? "Wi-Fi on" : "Wi-Fi off: still writing"}
      </button>
    </div>
  );
}
