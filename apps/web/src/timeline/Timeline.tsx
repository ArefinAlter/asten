import type { FormEvent } from "react";
import type { CanonEntity, AstenDocument, TimelineEvent } from "@asten/contracts";

interface Props {
  events: TimelineEvent[];
  entities: CanonEntity[];
  documents: AstenDocument[];
  onCreate: (input: {
    title: string;
    story_time?: string;
    reading_order?: number;
    notes?: string;
    entity_id?: string | null;
    document_id?: string | null;
  }) => void;
  creating?: boolean;
}

export function Timeline({
  events,
  entities,
  documents,
  onCreate,
  creating,
}: Props) {
  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const order = String(form.get("reading_order") ?? "");
    onCreate({
      title: String(form.get("title") ?? ""),
      story_time: String(form.get("story_time") ?? "") || undefined,
      reading_order: order ? Number(order) : undefined,
      notes: String(form.get("notes") ?? ""),
      entity_id: String(form.get("entity_id") || "") || null,
      document_id: String(form.get("document_id") || "") || null,
    });
    event.currentTarget.reset();
  }

  const ordered = [...events].sort(
    (left, right) => (left.reading_order ?? 999) - (right.reading_order ?? 999),
  );

  return (
    <section className="editor-placeholder">
      <p className="eyebrow">Two clocks</p>
      <h1>Reading order is not story time</h1>
      <p>
        A flashback is not a bug. The compiler only argues once both clocks are
        stated.
      </p>
      <ol className="timeline-list">
        {ordered.map((item) => (
          <li key={item.id}>
            <strong>{item.title}</strong>
            <span>
              Read {item.reading_order ?? "—"} · Story {item.story_time ?? "unsaid"}
            </span>
            {item.notes ? <span>{item.notes}</span> : null}
          </li>
        ))}
      </ol>
      <form className="stacked-form" onSubmit={submit}>
        <input name="title" placeholder="Harbor arrival" required />
        <input name="story_time" placeholder="Year 3, dusk" />
        <input name="reading_order" type="number" placeholder="Reading order" />
        <select name="entity_id" defaultValue="">
          <option value="">No character</option>
          {entities.map((entity) => (
            <option key={entity.id} value={entity.id}>
              {entity.name}
            </option>
          ))}
        </select>
        <select name="document_id" defaultValue="">
          <option value="">No scene</option>
          {documents.map((document) => (
            <option key={document.id} value={document.id}>
              {document.title}
            </option>
          ))}
        </select>
        <textarea name="notes" rows={2} placeholder="Flashback? Say so here." />
        <button type="submit" disabled={creating}>
          Pin a beat
        </button>
      </form>
    </section>
  );
}
