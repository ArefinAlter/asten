import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { type FormEvent, useEffect, useState } from "react";
import type {
  AstenDocument,
  CanonEntity,
  EntityKind,
  FactStatus,
  Location,
  RenamePreview,
} from "@asten/contracts";
import { createAstenClient } from "../lib/asten-client";
import { entityKey, humanKey } from "../lib/format";

interface Props {
  projectId: string;
  selectedKey?: string;
  onSelectKey: (key: string | undefined) => void;
  documents: AstenDocument[];
  onJump: (location: Location) => void;
}

const KINDS: EntityKind[] = [
  "character",
  "place",
  "item",
  "event",
  "rule",
  "source",
  "other",
];

/** Who lives here: the cast, their settled facts, aliases, mentions, renames. */
export function Cast({
  projectId,
  selectedKey,
  onSelectKey,
  documents,
  onJump,
}: Props) {
  const client = createAstenClient();
  const queryClient = useQueryClient();
  const entities = useQuery({
    queryKey: ["entities", projectId],
    queryFn: () => client.listEntities(projectId),
  });
  const facts = useQuery({
    queryKey: ["facts", projectId],
    queryFn: () => client.listFacts(projectId),
  });
  const [filter, setFilter] = useState("");
  const selected = entities.data?.find(
    (entity) => entityKey(entity.name) === selectedKey,
  );

  const invalidate = async () => {
    await queryClient.invalidateQueries({ queryKey: ["entities", projectId] });
    await queryClient.invalidateQueries({ queryKey: ["facts", projectId] });
    await queryClient.invalidateQueries({
      queryKey: ["diagnostics", projectId],
    });
  };
  const create = useMutation({
    mutationFn: (input: {
      name: string;
      kind: EntityKind;
      notes: string;
      aliases: string[];
    }) => client.createEntity({ project_id: projectId, ...input }),
    onSuccess: async (entity) => {
      onSelectKey(entityKey(entity.name));
      await invalidate();
    },
  });

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    create.mutate({
      name: String(form.get("name") ?? ""),
      kind: String(form.get("kind") ?? "character") as EntityKind,
      notes: String(form.get("notes") ?? ""),
      aliases: String(form.get("aliases") ?? "")
        .split(",")
        .map((alias) => alias.trim())
        .filter(Boolean),
    });
    event.currentTarget.reset();
  }

  const list = (entities.data ?? []).filter(
    (entity) =>
      !filter ||
      entity.name.toLowerCase().includes(filter.toLowerCase()) ||
      entity.aliases.some((alias) =>
        alias.toLowerCase().includes(filter.toLowerCase()),
      ),
  );
  const byKind = KINDS.map((kind) => ({
    kind,
    items: list.filter((entity) => entity.kind === kind),
  })).filter((group) => group.items.length > 0);

  return (
    <section className="cast register">
      <div className="cast-list">
        <header>
          <p className="eyebrow mono">Cast</p>
          <h1>Who lives here</h1>
          <input
            className="filter-input"
            placeholder="Filter the cast"
            value={filter}
            onChange={(event) => setFilter(event.target.value)}
          />
        </header>
        {byKind.map((group) => (
          <div key={group.kind} className="cast-group">
            <h3 className="mono">{group.kind}s</h3>
            <ul className="chip-list">
              {group.items.map((entity) => {
                const key = entityKey(entity.name);
                const count = (facts.data ?? []).filter(
                  (fact) =>
                    fact.entity_key === key && fact.status === "confirmed",
                ).length;
                return (
                  <li key={entity.id}>
                    <button
                      type="button"
                      className={`cartouche-chip${selectedKey === key ? " selected" : ""}`}
                      onClick={() => onSelectKey(key)}
                    >
                      {entity.name}
                      {count > 0 && <small className="mono"> {count}</small>}
                    </button>
                  </li>
                );
              })}
            </ul>
          </div>
        ))}
        <form className="stacked-form cast-add" onSubmit={submit}>
          <h3 className="mono">Add to the cast</h3>
          <input name="name" placeholder="Mira" required />
          <select name="kind" defaultValue="character">
            {KINDS.map((kind) => (
              <option key={kind} value={kind}>
                {kind}
              </option>
            ))}
          </select>
          <input
            name="aliases"
            placeholder="Aliases, comma separated (the dyer's girl, মিরা)"
          />
          <textarea name="notes" rows={2} placeholder="What you know so far" />
          <button
            type="submit"
            className="btn btn--small btn--primary"
            disabled={create.isPending}
          >
            Add
          </button>
        </form>
      </div>
      <div className="cast-detail">
        {selected ? (
          <EntityDetail
            key={selected.id}
            projectId={projectId}
            entity={selected}
            documents={documents}
            entities={entities.data ?? []}
            onJump={onJump}
            onChanged={invalidate}
            onDeleted={() => onSelectKey(undefined)}
          />
        ) : (
          <div className="editor-placeholder">
            <p className="eyebrow mono">Cast</p>
            <h1>Pick a name</h1>
            <p>
              Characters are entities, not a separate bible. Settle facts here
              or confirm the ones Asten proposes from the page. Nothing is
              required before you write.
            </p>
          </div>
        )}
      </div>
    </section>
  );
}

function EntityDetail({
  projectId,
  entity,
  documents,
  entities,
  onJump,
  onChanged,
  onDeleted,
}: {
  projectId: string;
  entity: CanonEntity;
  documents: AstenDocument[];
  entities: CanonEntity[];
  onJump: (location: Location) => void;
  onChanged: () => Promise<void>;
  onDeleted: () => void;
}) {
  const client = createAstenClient();
  const key = entityKey(entity.name);
  const facts = useQuery({
    queryKey: ["facts", projectId],
    queryFn: () => client.listFacts(projectId),
  });
  const mentions = useQuery({
    queryKey: ["mentions", projectId, key],
    queryFn: () => client.mentions(projectId, key),
  });
  const [notes, setNotes] = useState(entity.notes);
  const [alias, setAlias] = useState("");
  const [renameTo, setRenameTo] = useState("");
  const [preview, setPreview] = useState<RenamePreview>();
  const [mergeTarget, setMergeTarget] = useState("");

  useEffect(() => setNotes(entity.notes), [entity.id, entity.notes]);

  const update = useMutation({
    mutationFn: (input: Parameters<typeof client.updateEntity>[2]) =>
      client.updateEntity(projectId, entity.id, input),
    onSuccess: onChanged,
  });
  const updateFact = useMutation({
    mutationFn: (input: { id: string; status: FactStatus }) =>
      client.updateFact(projectId, input.id, { status: input.status }),
    onSuccess: onChanged,
  });
  const addFact = useMutation({
    mutationFn: (input: {
      predicate: string;
      value: string;
      valid_from?: string;
    }) =>
      client.createFact({
        project_id: projectId,
        entity_key: key,
        predicate: input.predicate,
        value: input.value,
        valid_from: input.valid_from || null,
        status: "confirmed",
      }),
    onSuccess: onChanged,
  });
  const rename = useMutation({
    mutationFn: (apply: boolean) =>
      client.renameEntity(projectId, entity.id, renameTo, apply),
    onSuccess: async (result, apply) => {
      if (apply) {
        setPreview(undefined);
        setRenameTo("");
        await onChanged();
      } else {
        setPreview(result);
      }
    },
  });
  const merge = useMutation({
    mutationFn: (targetId: string) =>
      client.mergeEntity(projectId, entity.id, targetId),
    onSuccess: async () => {
      onDeleted();
      await onChanged();
    },
  });
  const remove = useMutation({
    mutationFn: () => client.deleteEntity(projectId, entity.id),
    onSuccess: async () => {
      onDeleted();
      await onChanged();
    },
  });

  const own = (facts.data ?? []).filter((fact) => fact.entity_key === key);
  const proseDocs = documents.filter(
    (document) =>
      document.kind !== "note" &&
      document.kind !== "wiki" &&
      document.kind !== "source",
  );

  return (
    <div className="entity-detail">
      <header className="entity-head">
        <span className="cartouche-chip large">{entity.name}</span>
        <select
          value={entity.kind}
          onChange={(event) =>
            update.mutate({ kind: event.target.value as EntityKind })
          }
          aria-label="Kind"
        >
          {KINDS.map((kind) => (
            <option key={kind} value={kind}>
              {kind}
            </option>
          ))}
        </select>
      </header>

      <section>
        <h3 className="mono">Settled facts</h3>
        <ul className="fact-list">
          {own.map((fact) => (
            <li key={fact.id} className={fact.status}>
              <span className="mono">{humanKey(fact.predicate)}</span>
              <strong>{fact.value}</strong>
              <small>
                {fact.status}
                {fact.valid_from ? ` · from ${fact.valid_from}` : ""}
                {fact.valid_to ? ` · until ${fact.valid_to}` : ""}
                {fact.context.kind !== "world" ? ` · ${fact.context.kind}` : ""}
                {fact.anchor?.stale ? " · source text changed" : ""}
              </small>
              <span className="row">
                {fact.status !== "confirmed" && (
                  <button
                    type="button"
                    className="linkish settled"
                    onClick={() =>
                      updateFact.mutate({ id: fact.id, status: "confirmed" })
                    }
                  >
                    settle
                  </button>
                )}
                {fact.status === "proposed" && (
                  <button
                    type="button"
                    className="linkish"
                    onClick={() =>
                      updateFact.mutate({ id: fact.id, status: "rejected" })
                    }
                  >
                    reject
                  </button>
                )}
                {fact.status === "confirmed" && (
                  <button
                    type="button"
                    className="linkish"
                    onClick={() =>
                      updateFact.mutate({ id: fact.id, status: "deprecated" })
                    }
                  >
                    retire
                  </button>
                )}
                {fact.anchor && (
                  <button
                    type="button"
                    className="linkish"
                    onClick={() =>
                      fact.anchor &&
                      onJump({
                        document: fact.anchor.document,
                        block: fact.anchor.block,
                        start: fact.anchor.start,
                        end: fact.anchor.end,
                      })
                    }
                  >
                    where
                  </button>
                )}
              </span>
            </li>
          ))}
          {own.length === 0 && (
            <li className="hint">
              Nothing settled. Settle a fact and Asten weighs every page against
              it.
            </li>
          )}
        </ul>
        <form
          className="row wrap"
          onSubmit={(event) => {
            event.preventDefault();
            const form = new FormData(event.currentTarget);
            addFact.mutate({
              predicate: String(form.get("predicate") ?? ""),
              value: String(form.get("value") ?? ""),
              valid_from: String(form.get("from") ?? ""),
            });
            event.currentTarget.reset();
          }}
        >
          <input
            name="predicate"
            placeholder="hair color"
            list="predicates"
            required
          />
          <datalist id="predicates">
            {[
              "hair_color",
              "eye_color",
              "height",
              "age",
              "birthplace",
              "location",
              "material",
              "parent_of",
              "child_of",
              "married_to",
              "sibling_of",
              "ally_of",
              "enemy_of",
            ].map((predicate) => (
              <option key={predicate} value={predicate} />
            ))}
          </datalist>
          <input name="value" placeholder="auburn" required />
          <input name="from" placeholder="from (day 3)" />
          <button
            type="submit"
            className="btn btn--small btn--primary"
            disabled={addFact.isPending}
          >
            Settle
          </button>
        </form>
      </section>

      <section>
        <h3 className="mono">Aliases</h3>
        <ul className="chip-list">
          {entity.aliases.map((item) => (
            <li key={item}>
              <span className="chip">
                {item}
                <button
                  type="button"
                  className="linkish"
                  aria-label={`Remove alias ${item}`}
                  onClick={() =>
                    update.mutate({
                      aliases: entity.aliases.filter((alias) => alias !== item),
                    })
                  }
                >
                  ×
                </button>
              </span>
            </li>
          ))}
        </ul>
        <form
          className="row"
          onSubmit={(event) => {
            event.preventDefault();
            if (alias.trim())
              update.mutate({ aliases: [...entity.aliases, alias.trim()] });
            setAlias("");
          }}
        >
          <input
            value={alias}
            onChange={(event) => setAlias(event.target.value)}
            placeholder="the stranger · Capt. Oren · ওরেন"
          />
          <button type="submit" className="btn btn--small">
            Add alias
          </button>
        </form>
      </section>

      <section>
        <h3 className="mono">Mentions</h3>
        <ul className="mention-list">
          {(mentions.data ?? []).slice(0, 40).map((mention, index) => (
            <li key={`${mention.block}-${mention.start}-${index}`}>
              <button
                type="button"
                className="linkish"
                onClick={() =>
                  onJump({
                    document: mention.document,
                    block: mention.block,
                    start: mention.start,
                    end: mention.end,
                  })
                }
              >
                <small className="mono">{mention.document_title}</small>{" "}
                {mention.quote}
              </button>
              {mention.context !== "narration" && (
                <small className="chip">{mention.context}</small>
              )}
            </li>
          ))}
          {(mentions.data ?? []).length === 0 && (
            <li className="hint">Not mentioned on any page yet.</li>
          )}
          {(mentions.data ?? []).length > 40 && (
            <li className="hint">
              … and {(mentions.data ?? []).length - 40} more
            </li>
          )}
        </ul>
      </section>

      <section>
        <h3 className="mono">Introduced in</h3>
        <select
          value={entity.introduced_at ?? ""}
          onChange={(event) =>
            update.mutate({
              introduced_at: event.target.value
                ? Number(event.target.value)
                : null,
            })
          }
        >
          <option value="">Not marked</option>
          {proseDocs.map((document, index) => (
            <option key={document.id} value={index + 1}>
              {document.title}
            </option>
          ))}
        </select>
        <p className="hint">
          Mentions before this scene get a note (NAME-W002). Fine for a teaser;
          worth a look otherwise.
        </p>
      </section>

      <section>
        <h3 className="mono">Notes</h3>
        <textarea
          rows={4}
          value={notes}
          onChange={(event) => setNotes(event.target.value)}
          onBlur={() => notes !== entity.notes && update.mutate({ notes })}
        />
      </section>

      <section>
        <h3 className="mono">Rename everywhere</h3>
        <form
          className="row"
          onSubmit={(event) => {
            event.preventDefault();
            if (renameTo.trim()) rename.mutate(false);
          }}
        >
          <input
            value={renameTo}
            onChange={(event) => setRenameTo(event.target.value)}
            placeholder={`New name for ${entity.name}`}
          />
          <button
            type="submit"
            className="btn btn--small"
            disabled={rename.isPending || !renameTo.trim()}
          >
            Preview
          </button>
        </form>
        {preview && (
          <div className="rename-preview">
            <p className="hint">
              {preview.changes.length} change
              {preview.changes.length === 1 ? "" : "s"} across{" "}
              {new Set(preview.changes.map((change) => change.document)).size}{" "}
              page(s). The old name stays as an alias.
            </p>
            <ul className="mention-list">
              {preview.changes.slice(0, 12).map((change, index) => (
                <li key={`${change.block}-${change.start}-${index}`}>
                  <small className="mono">{change.document_title}</small>{" "}
                  <s>{change.before}</s> → <strong>{change.after}</strong>
                </li>
              ))}
            </ul>
            <div className="row">
              <button
                type="button"
                className="btn btn--small btn--primary"
                onClick={() => rename.mutate(true)}
                disabled={rename.isPending}
              >
                Apply rename
              </button>
              <button
                type="button"
                className="btn btn--small btn--ghost"
                onClick={() => setPreview(undefined)}
              >
                Cancel
              </button>
            </div>
          </div>
        )}
      </section>

      <section className="danger-zone">
        <h3 className="mono">Merge or remove</h3>
        <div className="row">
          <select
            value={mergeTarget}
            onChange={(event) => setMergeTarget(event.target.value)}
          >
            <option value="">Merge into…</option>
            {entities
              .filter((other) => other.id !== entity.id)
              .map((other) => (
                <option key={other.id} value={other.id}>
                  {other.name}
                </option>
              ))}
          </select>
          <button
            type="button"
            className="btn btn--small"
            disabled={!mergeTarget || merge.isPending}
            onClick={() => mergeTarget && merge.mutate(mergeTarget)}
          >
            Merge
          </button>
          <button
            type="button"
            className="btn btn--small btn--ghost rubric"
            onClick={() => {
              if (
                window.confirm(
                  `Remove ${entity.name} from the cast? Facts keep the key; marks will point at a missing entity.`,
                )
              )
                remove.mutate();
            }}
          >
            Remove
          </button>
        </div>
        <p className="hint">
          Merging turns this name into an alias of the target and moves facts
          and events across. A late reveal ("the stranger is Oren") is a merge.
        </p>
      </section>
    </div>
  );
}
