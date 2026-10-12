import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { type FormEvent, useState } from "react";
import type {
  AstenDocument,
  CanonEntity,
  CreateInvariant,
  DiagnosticsSnapshot,
  Fact,
  FactStatus,
  InvariantPattern,
  Proposition,
  Route,
  TraceClock,
} from "@asten/contracts";
import { createAstenClient } from "../lib/asten-client";
import { CONTEXT_CHOICES, entityKey, humanKey, timeAgo } from "../lib/format";

interface Props {
  projectId: string;
  documents: AstenDocument[];
  entities: CanonEntity[];
  snapshot?: DiagnosticsSnapshot;
}

type Tab = "facts" | "proposals" | "invariants" | "routes" | "baselines";

/** Canon: every fact with its status, the proposals queue, your own rules, routes and baselines. */
export function Canon({ projectId, documents, entities, snapshot }: Props) {
  const client = createAstenClient();
  const queryClient = useQueryClient();
  const [tab, setTab] = useState<Tab>("facts");
  const facts = useQuery({
    queryKey: ["facts", projectId],
    queryFn: () => client.listFacts(projectId),
  });
  const invalidate = async () => {
    await queryClient.invalidateQueries({ queryKey: ["facts", projectId] });
    await queryClient.invalidateQueries({
      queryKey: ["diagnostics", projectId],
    });
  };
  const updateFact = useMutation({
    mutationFn: (input: { id: string; status: FactStatus }) =>
      client.updateFact(projectId, input.id, { status: input.status }),
    onSuccess: invalidate,
  });
  const deleteFact = useMutation({
    mutationFn: (id: string) => client.deleteFact(projectId, id),
    onSuccess: invalidate,
  });
  const proposals = (facts.data ?? []).filter(
    (fact) => fact.status === "proposed",
  );

  return (
    <section className="canon register">
      <header>
        <p className="eyebrow mono">Canon</p>
        <h1>What your story says is true</h1>
        <div className="segmented" role="tablist">
          {(
            ["facts", "proposals", "invariants", "routes", "baselines"] as Tab[]
          ).map((item) => (
            <button
              key={item}
              type="button"
              role="tab"
              aria-selected={tab === item}
              className={tab === item ? "active" : ""}
              onClick={() => setTab(item)}
            >
              {item === "proposals"
                ? `Proposals ${proposals.length ? `(${proposals.length})` : ""}`
                : item[0].toUpperCase() + item.slice(1)}
            </button>
          ))}
        </div>
      </header>

      {tab === "facts" && (
        <FactsTable
          facts={facts.data ?? []}
          onStatus={(id, status) => updateFact.mutate({ id, status })}
          onDelete={(id) => deleteFact.mutate(id)}
        />
      )}
      {tab === "proposals" && (
        <div className="proposals">
          <p className="lede">
            Proposed facts come from the page (inline{" "}
            <code>[[entity.predicate=value]]</code> notes, settled world cards)
            and, later, from the importer. Nothing becomes canon until you
            settle it.
          </p>
          <FactsTable
            facts={proposals}
            onStatus={(id, status) => updateFact.mutate({ id, status })}
            onDelete={(id) => deleteFact.mutate(id)}
          />
        </div>
      )}
      {tab === "invariants" && (
        <Invariants
          projectId={projectId}
          entities={entities}
          snapshot={snapshot}
        />
      )}
      {tab === "routes" && <Routes projectId={projectId} entities={entities} />}
      {tab === "baselines" && (
        <Baselines projectId={projectId} snapshot={snapshot} />
      )}
      <p className="hint">{documents.length} documents in the binder.</p>
    </section>
  );
}

function FactsTable({
  facts,
  onStatus,
  onDelete,
}: {
  facts: Fact[];
  onStatus: (id: string, status: FactStatus) => void;
  onDelete: (id: string) => void;
}) {
  const [filter, setFilter] = useState("");
  const rows = facts.filter(
    (fact) =>
      !filter ||
      `${fact.entity_key} ${fact.predicate} ${fact.value}`
        .toLowerCase()
        .includes(filter.toLowerCase()),
  );
  return (
    <div className="facts-table">
      <input
        className="filter-input"
        placeholder="Filter facts"
        value={filter}
        onChange={(event) => setFilter(event.target.value)}
      />
      <table>
        <thead>
          <tr>
            <th>Entity</th>
            <th>Predicate</th>
            <th>Value</th>
            <th>Context</th>
            <th>Valid</th>
            <th>Status</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {rows.map((fact) => (
            <tr key={fact.id} className={fact.status}>
              <td>{humanKey(fact.entity_key)}</td>
              <td className="mono">{humanKey(fact.predicate)}</td>
              <td>
                <strong>{fact.value}</strong>
              </td>
              <td className="mono">{fact.context.kind}</td>
              <td className="mono">
                {fact.valid_from ?? "…"} – {fact.valid_to ?? "…"}
              </td>
              <td>
                <span className={`chip chip--${fact.status}`}>
                  {fact.status === "confirmed" ? "settled" : fact.status}
                </span>
              </td>
              <td className="row">
                {fact.status !== "confirmed" && (
                  <button
                    type="button"
                    className="linkish settled"
                    onClick={() => onStatus(fact.id, "confirmed")}
                  >
                    settle
                  </button>
                )}
                {fact.status === "proposed" && (
                  <button
                    type="button"
                    className="linkish"
                    onClick={() => onStatus(fact.id, "rejected")}
                  >
                    reject
                  </button>
                )}
                {fact.status === "confirmed" && (
                  <button
                    type="button"
                    className="linkish"
                    onClick={() => onStatus(fact.id, "deprecated")}
                  >
                    retire
                  </button>
                )}
                <button
                  type="button"
                  className="linkish rubric"
                  onClick={() => onDelete(fact.id)}
                >
                  delete
                </button>
              </td>
            </tr>
          ))}
          {rows.length === 0 && (
            <tr>
              <td colSpan={7} className="hint">
                No facts here.
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}

const PATTERNS: {
  value: InvariantPattern["pattern"];
  label: string;
  help: string;
  slots: string[];
}[] = [
  {
    value: "absence_after",
    label: "Once A, never B",
    help: "Once Oren dies, he never speaks.",
    slots: ["after", "never"],
  },
  {
    value: "response",
    label: "Every A is followed by B",
    help: "Every planted clue is eventually paid off.",
    slots: ["trigger", "response"],
  },
  {
    value: "precedence",
    label: "B never before A",
    help: "The reveal never comes before the clue is planted.",
    slots: ["first", "later"],
  },
  {
    value: "universality_between",
    label: "Between A and B, always C",
    help: "Mira stays in Veld between the siege and the treaty.",
    slots: ["from", "until", "always"],
  },
  {
    value: "bounded_existence",
    label: "A happens at most N times",
    help: "The ring changes hands at most three times.",
    slots: ["event"],
  },
];

type PropDraft = {
  kind: "happens" | "mentioned" | "in_context";
  entity: string;
  event: string;
  context: string;
};

function toProposition(draft: PropDraft): Proposition {
  switch (draft.kind) {
    case "happens":
      return {
        kind: "happens",
        subject: draft.entity ? entityKey(draft.entity) : null,
        event: (draft.event || "any") as never,
      };
    case "mentioned":
      return { kind: "mentioned", entity: entityKey(draft.entity) };
    case "in_context":
      return {
        kind: "in_context",
        context: CONTEXT_CHOICES.find(
          (choice) => choice.value === draft.context,
        )?.context ?? { kind: "memory" },
      };
  }
}

function describeProposition(proposition: Proposition): string {
  switch (proposition.kind) {
    case "happens":
      return `${proposition.subject ? humanKey(proposition.subject) : "someone"} ${typeof proposition.event === "string" ? proposition.event : proposition.event.other}`;
    case "mentioned":
      return `${humanKey(proposition.entity)} is mentioned`;
    case "in_context":
      return `the scene is ${proposition.context.kind}`;
    case "holds":
      return `${humanKey(proposition.fluent.subject)} · ${humanKey(proposition.fluent.predicate)}`;
  }
}

function Invariants({
  projectId,
  entities,
  snapshot,
}: {
  projectId: string;
  entities: CanonEntity[];
  snapshot?: DiagnosticsSnapshot;
}) {
  const client = createAstenClient();
  const queryClient = useQueryClient();
  const invariants = useQuery({
    queryKey: ["invariants", projectId],
    queryFn: () => client.listInvariants(projectId),
  });
  const [pattern, setPattern] =
    useState<InvariantPattern["pattern"]>("absence_after");
  const [label, setLabel] = useState("");
  const [clock, setClock] = useState<TraceClock>("story_time");
  const [max, setMax] = useState(3);
  const blank = (): PropDraft => ({
    kind: "happens",
    entity: "",
    event: "dies",
    context: "memory",
  });
  const [slots, setSlots] = useState<PropDraft[]>([blank(), blank(), blank()]);
  const invalidate = async () => {
    await queryClient.invalidateQueries({
      queryKey: ["invariants", projectId],
    });
    await queryClient.invalidateQueries({
      queryKey: ["diagnostics", projectId],
    });
  };
  const create = useMutation({
    mutationFn: (input: CreateInvariant) =>
      client.createInvariant(projectId, input),
    onSuccess: invalidate,
  });
  const remove = useMutation({
    mutationFn: (id: string) => client.deleteInvariant(projectId, id),
    onSuccess: invalidate,
  });
  const toggle = useMutation({
    mutationFn: (input: { id: string; enabled: boolean }) => {
      const stored = invariants.data?.find((item) => item.id === input.id);
      return client.updateInvariant(projectId, input.id, {
        ...stored!.invariant,
        enabled: input.enabled,
      });
    },
    onSuccess: invalidate,
  });
  const definition = PATTERNS.find((item) => item.value === pattern)!;

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const props = slots.map(toProposition);
    let built: InvariantPattern;
    switch (pattern) {
      case "absence_after":
        built = { pattern, after: props[0], never: props[1] };
        break;
      case "response":
        built = { pattern, trigger: props[0], response: props[1] };
        break;
      case "precedence":
        built = { pattern, first: props[0], later: props[1] };
        break;
      case "universality_between":
        built = { pattern, from: props[0], until: props[1], always: props[2] };
        break;
      case "bounded_existence":
        built = { pattern, event: props[0], max };
        break;
    }
    create.mutate({
      label: label || definition.label,
      pattern: built,
      clock,
      enabled: true,
    });
    setLabel("");
  }

  const ledger = snapshot?.ledger ?? [];
  return (
    <div className="invariants">
      <p className="lede">
        Your own rules over the story, written as patterns rather than logic.
        Each is checked over the scenes in story time or reading order. A rule
        whose payoff is unwritten is pending, not broken.
      </p>
      <ul className="plain-list">
        {(invariants.data ?? []).map((stored) => (
          <li key={stored.id} className="invariant-row">
            <label className="row">
              <input
                type="checkbox"
                checked={stored.invariant.enabled}
                onChange={(event) =>
                  toggle.mutate({
                    id: stored.id,
                    enabled: event.target.checked,
                  })
                }
              />
              <strong>{stored.invariant.label}</strong>
              <small className="mono">
                {stored.invariant.pattern.pattern.replace(/_/g, " ")} ·{" "}
                {stored.invariant.clock.replace("_", " ")}
              </small>
            </label>
            <button
              type="button"
              className="linkish rubric"
              onClick={() => remove.mutate(stored.id)}
            >
              remove
            </button>
          </li>
        ))}
        {(invariants.data ?? []).length === 0 && (
          <li className="hint">No rules yet.</li>
        )}
      </ul>
      {ledger.length > 0 && (
        <div className="ledger">
          <h3 className="mono">Ledger: awaiting payoff</h3>
          <ul className="plain-list">
            {ledger.map((entry, index) => (
              <li key={`${entry.invariant}-${index}`}>
                {entry.label}{" "}
                <small className="mono">since {entry.since_title}</small>
              </li>
            ))}
          </ul>
        </div>
      )}
      <form className="stacked-form invariant-form" onSubmit={submit}>
        <h3 className="mono">New rule</h3>
        <label>
          <span>Pattern</span>
          <select
            value={pattern}
            onChange={(event) =>
              setPattern(event.target.value as InvariantPattern["pattern"])
            }
          >
            {PATTERNS.map((item) => (
              <option key={item.value} value={item.value}>
                {item.label}
              </option>
            ))}
          </select>
          <small className="hint">{definition.help}</small>
        </label>
        {definition.slots.map((slot, index) => (
          <fieldset key={slot} className="slot">
            <legend className="mono">{slot}</legend>
            <select
              value={slots[index].kind}
              onChange={(event) =>
                setSlots(
                  slots.map((item, i) =>
                    i === index
                      ? {
                          ...item,
                          kind: event.target.value as PropDraft["kind"],
                        }
                      : item,
                  ),
                )
              }
            >
              <option value="happens">an event happens</option>
              <option value="mentioned">an entity is mentioned</option>
              <option value="in_context">the scene is told as…</option>
            </select>
            {slots[index].kind !== "in_context" && (
              <select
                value={slots[index].entity}
                onChange={(event) =>
                  setSlots(
                    slots.map((item, i) =>
                      i === index
                        ? { ...item, entity: event.target.value }
                        : item,
                    ),
                  )
                }
              >
                <option value="">
                  {slots[index].kind === "happens"
                    ? "anyone"
                    : "choose an entity"}
                </option>
                {entities.map((entity) => (
                  <option key={entity.id} value={entity.name}>
                    {entity.name}
                  </option>
                ))}
              </select>
            )}
            {slots[index].kind === "happens" && (
              <select
                value={slots[index].event}
                onChange={(event) =>
                  setSlots(
                    slots.map((item, i) =>
                      i === index
                        ? { ...item, event: event.target.value }
                        : item,
                    ),
                  )
                }
              >
                {[
                  "dies",
                  "born",
                  "travels",
                  "acquires",
                  "loses",
                  "learns",
                  "tells",
                  "reveals",
                  "marries",
                  "acts",
                  "any",
                ].map((kind) => (
                  <option key={kind} value={kind}>
                    {kind}
                  </option>
                ))}
              </select>
            )}
            {slots[index].kind === "in_context" && (
              <select
                value={slots[index].context}
                onChange={(event) =>
                  setSlots(
                    slots.map((item, i) =>
                      i === index
                        ? { ...item, context: event.target.value }
                        : item,
                    ),
                  )
                }
              >
                {CONTEXT_CHOICES.map((choice) => (
                  <option key={choice.value} value={choice.value}>
                    {choice.label}
                  </option>
                ))}
              </select>
            )}
            <small className="hint">
              {describeProposition(toProposition(slots[index]))}
            </small>
          </fieldset>
        ))}
        {pattern === "bounded_existence" && (
          <label>
            <span>At most</span>
            <input
              type="number"
              min={0}
              value={max}
              onChange={(event) => setMax(Number(event.target.value))}
            />
          </label>
        )}
        <label>
          <span>Checked along</span>
          <select
            value={clock}
            onChange={(event) => setClock(event.target.value as TraceClock)}
          >
            <option value="story_time">story time</option>
            <option value="reading_order">reading order</option>
          </select>
        </label>
        <label>
          <span>Name</span>
          <input
            value={label}
            onChange={(event) => setLabel(event.target.value)}
            placeholder={definition.help}
          />
        </label>
        <button
          type="submit"
          className="btn btn--small btn--primary"
          disabled={create.isPending}
        >
          Add rule
        </button>
      </form>
    </div>
  );
}

function Routes({
  projectId,
  entities,
}: {
  projectId: string;
  entities: CanonEntity[];
}) {
  const client = createAstenClient();
  const queryClient = useQueryClient();
  const routes = useQuery({
    queryKey: ["routes", projectId],
    queryFn: () => client.listRoutes(projectId),
  });
  const invalidate = async () => {
    await queryClient.invalidateQueries({ queryKey: ["routes", projectId] });
    await queryClient.invalidateQueries({
      queryKey: ["diagnostics", projectId],
    });
  };
  const create = useMutation({
    mutationFn: (route: Route) => client.createRoute(projectId, route),
    onSuccess: invalidate,
  });
  const remove = useMutation({
    mutationFn: (id: string) => client.deleteRoute(projectId, id),
    onSuccess: invalidate,
  });
  const places = entities.filter((entity) => entity.kind === "place");

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    create.mutate({
      from: entityKey(String(form.get("from"))),
      to: entityKey(String(form.get("to"))),
      min_ticks: Number(form.get("days") ?? 1),
      mode: String(form.get("mode") ?? ""),
      bidirectional: form.get("both") === "on",
      available: {},
    });
    event.currentTarget.reset();
  }

  return (
    <div className="routes">
      <p className="lede">
        Routes between places with the least time they take. Travel reduces to
        time: two appearances closer together than the shortest route allows
        become a mark (CANON-E301). Without routes nothing is checked.
      </p>
      <ul className="plain-list">
        {(routes.data ?? []).map((stored) => (
          <li key={stored.id} className="row">
            <span>
              {humanKey(stored.route.from)}{" "}
              {stored.route.bidirectional ? "↔" : "→"}{" "}
              {humanKey(stored.route.to)}
            </span>
            <small className="mono">
              {stored.route.min_ticks} day
              {stored.route.min_ticks === 1 ? "" : "s"}
              {stored.route.mode ? ` · ${stored.route.mode}` : ""}
            </small>
            <button
              type="button"
              className="linkish rubric"
              onClick={() => remove.mutate(stored.id)}
            >
              remove
            </button>
          </li>
        ))}
        {(routes.data ?? []).length === 0 && (
          <li className="hint">No routes yet.</li>
        )}
      </ul>
      <form className="row wrap" onSubmit={submit}>
        <input name="from" list="places" placeholder="From" required />
        <input name="to" list="places" placeholder="To" required />
        <datalist id="places">
          {places.map((place) => (
            <option key={place.id} value={place.name} />
          ))}
        </datalist>
        <input
          name="days"
          type="number"
          min={0}
          defaultValue={1}
          aria-label="Days"
        />
        <input name="mode" placeholder="on foot · by ship" />
        <label className="row">
          <input type="checkbox" name="both" defaultChecked /> both ways
        </label>
        <button
          type="submit"
          className="btn btn--small btn--primary"
          disabled={create.isPending}
        >
          Add route
        </button>
      </form>
    </div>
  );
}

function Baselines({
  projectId,
  snapshot,
}: {
  projectId: string;
  snapshot?: DiagnosticsSnapshot;
}) {
  const client = createAstenClient();
  const queryClient = useQueryClient();
  const baselines = useQuery({
    queryKey: ["baselines", projectId],
    queryFn: () => client.listBaselines(projectId),
  });
  const invalidate = async () => {
    await queryClient.invalidateQueries({ queryKey: ["baselines", projectId] });
    await queryClient.invalidateQueries({
      queryKey: ["diagnostics", projectId],
    });
  };
  const create = useMutation({
    mutationFn: (name: string) => client.createBaseline(projectId, name),
    onSuccess: invalidate,
  });
  const activate = useMutation({
    mutationFn: (id: string) => client.activateBaseline(projectId, id),
    onSuccess: invalidate,
  });
  const clear = useMutation({
    mutationFn: () => client.clearBaseline(projectId),
    onSuccess: invalidate,
  });
  const remove = useMutation({
    mutationFn: (id: string) => client.deleteBaseline(projectId, id),
    onSuccess: invalidate,
  });
  const open =
    snapshot?.diagnostics.filter((mark) => !mark.suppressed_by).length ?? 0;
  const fresh =
    snapshot?.diagnostics.filter(
      (mark) => !mark.suppressed_by && mark.new_since_baseline,
    ).length ?? 0;

  return (
    <div className="baselines">
      <p className="lede">
        Freeze today's marks as known. From then on The Scales highlights only
        what is new, the way a checker is adopted on a half-finished novel.
      </p>
      <p className="hint mono">
        {open} open · {fresh} new since the active baseline
      </p>
      <ul className="plain-list">
        {(baselines.data ?? []).map((baseline) => (
          <li key={baseline.id} className="row">
            <strong>{baseline.name}</strong>
            <small className="mono">
              {baseline.diagnostic_ids.length} marks ·{" "}
              {timeAgo(baseline.created_at)}
              {baseline.active ? " · active" : ""}
            </small>
            {!baseline.active && (
              <button
                type="button"
                className="linkish"
                onClick={() => activate.mutate(baseline.id)}
              >
                activate
              </button>
            )}
            <button
              type="button"
              className="linkish rubric"
              onClick={() => remove.mutate(baseline.id)}
            >
              remove
            </button>
          </li>
        ))}
      </ul>
      <div className="row">
        <button
          type="button"
          className="btn btn--small btn--primary"
          onClick={() =>
            create.mutate(`Baseline ${new Date().toLocaleDateString()}`)
          }
          disabled={create.isPending}
        >
          Freeze today's marks
        </button>
        <button
          type="button"
          className="btn btn--small btn--ghost"
          onClick={() => clear.mutate()}
          disabled={clear.isPending}
        >
          Show everything
        </button>
      </div>
    </div>
  );
}
