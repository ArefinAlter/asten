import type { MethodPack } from "@asten/contracts";

interface Props {
  packs: MethodPack[];
  onToggle: (packId: string, enabled: boolean) => void;
}

export function Packs({ packs, onToggle }: Props) {
  return (
    <section className="editor-placeholder">
      <p className="eyebrow">Method packs</p>
      <h1>Extensions, not a doctrine</h1>
      <p>
        Packs add rules and optional craft hints. They never replace the page.
        Scribe stays on. Diction is a hint pack, off until you ask.
      </p>
      <ul className="fact-list">
        {packs.map((pack) => (
          <li key={pack.id}>
            <strong>
              {pack.name} · {pack.version}
            </strong>
            <span>{pack.summary}</span>
            <button
              type="button"
              disabled={pack.pack_key === "scribe"}
              onClick={() => onToggle(pack.id, !pack.enabled)}
            >
              {pack.enabled ? "On" : "Off"}
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}
