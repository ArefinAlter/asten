import type { FormEvent } from "react";
import type { CanonEntity, EntityKind, Fact } from "@asten/contracts";

interface Props {
  entities: CanonEntity[];
  facts: Fact[];
  onCreate: (input: { name: string; kind: EntityKind; notes: string }) => void;
  creating?: boolean;
}

export function Cast({ entities, facts, onCreate, creating }: Props) {
  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    onCreate({
      name: String(form.get("name") ?? ""),
      kind: String(form.get("kind") ?? "character") as EntityKind,
      notes: String(form.get("notes") ?? ""),
    });
    event.currentTarget.reset();
  }

  return (
    <section className="editor-placeholder">
      <p className="eyebrow">Cast and canon people</p>
      <h1>Who lives here</h1>
      <p>
        Characters are entities, not a separate bible. Confirm facts in the
        inspector. Write first if you want; this drawer is optional.
      </p>
      <form className="stacked-form" onSubmit={submit}>
        <input name="name" placeholder="Mira" required />
        <select name="kind" defaultValue="character">
          <option value="character">Character</option>
          <option value="place">Place</option>
          <option value="item">Item</option>
          <option value="rule">Rule</option>
          <option value="source">Source</option>
          <option value="other">Other</option>
        </select>
        <textarea name="notes" rows={3} placeholder="What you know so far" />
        <button type="submit" disabled={creating}>
          Add to the cast
        </button>
      </form>
      <ul className="fact-list">
        {entities.map((entity) => (
          <li key={entity.id}>
            <strong>
              {entity.name} · {entity.kind}
            </strong>
            <span>{entity.notes || "No notes yet."}</span>
            <span>
              {facts
                .filter(
                  (fact) =>
                    fact.entity_key ===
                    entity.name.toLowerCase().replace(/\s+/g, "_"),
                )
                .map((fact) => `${fact.predicate}=${fact.value}`)
                .join(" · ") || "No confirmed facts."}
            </span>
          </li>
        ))}
      </ul>
    </section>
  );
}
