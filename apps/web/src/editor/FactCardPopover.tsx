import { useQuery } from "@tanstack/react-query";
import { createAstenClient } from "../lib/asten-client";

interface Props {
  projectId: string;
  entity: string;
  documentId: string;
  x: number;
  y: number;
  onOpen: () => void;
  onClose: () => void;
}

/** Hovering a name shows what is settled about them at this point in the story. */
export function FactCardPopover({
  projectId,
  entity,
  documentId,
  x,
  y,
  onOpen,
  onClose,
}: Props) {
  const client = createAstenClient();
  const card = useQuery({
    queryKey: ["fact-card", projectId, entity, documentId],
    queryFn: () => client.factCard(projectId, entity, documentId),
  });
  const data = card.data;
  return (
    <div
      className="fact-card"
      style={{ left: Math.max(0, x), top: y + 6 }}
      onMouseLeave={onClose}
      role="dialog"
      aria-label={`Facts about ${data?.name ?? entity}`}
    >
      {!data ? (
        <p className="status">{card.isError ? "Not in the cast." : "…"}</p>
      ) : (
        <>
          <header>
            <span className="cartouche-chip">{data.name}</span>
            <span className="mono">{data.sort}</span>
          </header>
          {data.at_label && <p className="hint">At {data.at_label}</p>}
          {data.facts.length === 0 ? (
            <p className="hint">Nothing settled yet.</p>
          ) : (
            <ul>
              {data.facts.slice(0, 6).map((fact) => (
                <li
                  key={fact.id}
                  className={fact.status === "settled" ? "settled" : "proposed"}
                >
                  <span>{fact.predicate}</span>
                  <strong>{fact.value}</strong>
                  {fact.valid_label && <small>{fact.valid_label}</small>}
                </li>
              ))}
            </ul>
          )}
          <footer>
            <span>
              {data.mention_count} mention{data.mention_count === 1 ? "" : "s"}
              {data.open_marks > 0
                ? ` · ${data.open_marks} open mark${data.open_marks === 1 ? "" : "s"}`
                : ""}
            </span>
            <button type="button" className="linkish" onClick={onOpen}>
              Open in cast
            </button>
          </footer>
        </>
      )}
    </div>
  );
}
