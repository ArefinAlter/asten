import { useEffect, useState } from "react";
import type { WorldStatus } from "@asten/contracts";
import {
  icebergCaption,
  kindLabel,
  promptLabel,
  statusLabel,
} from "./kinds";
import { useWorld } from "./useWorld";

const STATUSES: WorldStatus[] = ["spark", "sketched", "settled"];

interface WorldInspectorProps {
  projectId: string;
  selectedId?: string;
}

export function WorldInspector({ projectId, selectedId }: WorldInspectorProps) {
  const { entries, pulse, update } = useWorld(projectId);
  const entry = entries.data?.find((item) => item.id === selectedId);
  const [title, setTitle] = useState(entry?.title ?? "");
  const [summary, setSummary] = useState(entry?.summary ?? "");

  useEffect(() => {
    setTitle(entry?.title ?? "");
    setSummary(entry?.summary ?? "");
  }, [entry?.id, entry?.title, entry?.summary]);

  const meter = pulse.data;

  if (!entry) {
    return (
      <>
        <div className="pane-heading">The iceberg</div>
        <div className="world-aside">
          <p>
            {icebergCaption(
              meter?.scenes ?? 0,
              meter?.underwater ?? 0,
              meter?.sparks ?? 0,
            )}
          </p>
          <dl>
            <div>
              <dt>Sparks</dt>
              <dd>{meter?.sparks ?? 0}</dd>
            </div>
            <div>
              <dt>Sketches</dt>
              <dd>{meter?.sketched ?? 0}</dd>
            </div>
            <div>
              <dt>Settled</dt>
              <dd>{meter?.settled ?? 0}</dd>
            </div>
            <div>
              <dt>On the page</dt>
              <dd>{meter?.scenes ?? 0}</dd>
            </div>
          </dl>
          <p className="hint">
            Nothing on the wall is canon until you settle it. A gardener can
            ignore this pane entirely.
          </p>
        </div>
      </>
    );
  }

  return (
    <>
      <div className="pane-heading">{kindLabel(entry.kind)}</div>
      <form
        className="world-aside stacked-form"
        onSubmit={(event) => {
          event.preventDefault();
          update.mutate({ id: entry.id, title, summary });
        }}
      >
        {entry.prompt && (
          <p className="eyebrow">{promptLabel(entry.prompt)}</p>
        )}
        <label>
          Name
          <input
            value={title}
            onChange={(event) => setTitle(event.target.value)}
            onBlur={() => {
              if (title.trim() && title !== entry.title) {
                update.mutate({ id: entry.id, title });
              }
            }}
          />
        </label>
        <label>
          More, if you want
          <textarea
            rows={5}
            value={summary}
            placeholder="Leave it thin. The story will ask for more."
            onChange={(event) => setSummary(event.target.value)}
            onBlur={() => {
              if (summary !== entry.summary) {
                update.mutate({ id: entry.id, summary });
              }
            }}
          />
        </label>
        <div className="status-row">
          {STATUSES.map((status) => (
            <button
              key={status}
              type="button"
              className={entry.status === status ? "active" : ""}
              onClick={() => update.mutate({ id: entry.id, status })}
            >
              {statusLabel(status)}
            </button>
          ))}
        </div>
        <p className="hint">
          {entry.status === "settled"
            ? "Settled. Continuity can watch this now."
            : "Still a spark. The engine will not nag you about it."}
        </p>
        {update.error && <p className="error">{update.error.message}</p>}
      </form>
    </>
  );
}
