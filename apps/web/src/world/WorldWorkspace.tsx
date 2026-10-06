import { type FormEvent, type PointerEvent, useMemo, useRef, useState } from "react";
import type { IdeaPrompt, WorldEntry, WorldKind } from "@margin/contracts";
import {
  DISCOVERY_LABELS,
  WORLD_KINDS,
  cardTilt,
  icebergCaption,
  kindHint,
  kindLabel,
  promptLabel,
  statusLabel,
} from "./kinds";
import { type WorldLens, useWorld } from "./useWorld";
import "./world.css";

const PROMPTS: IdeaPrompt[] = ["why", "how", "what_result"];

interface WorldWorkspaceProps {
  projectId: string;
  projectName: string;
  lens: WorldLens;
  selectedId?: string;
  onLensChange: (lens: WorldLens) => void;
  onSelect: (id: string | undefined) => void;
  onWriteScene: () => void;
}

export function WorldWorkspace({
  projectId,
  projectName,
  lens,
  selectedId,
  onLensChange,
  onSelect,
  onWriteScene,
}: WorldWorkspaceProps) {
  const { entries, pulse, create, update } = useWorld(projectId);
  const [kind, setKind] = useState<WorldKind>("fragment");
  const [prompt, setPrompt] = useState<IdeaPrompt>("why");
  const [draft, setDraft] = useState("");
  const [positions, setPositions] = useState<Record<string, { x: number; y: number }>>(
    {},
  );
  const boardRef = useRef<HTMLDivElement>(null);
  const dragRef = useRef<{
    id: string;
    startX: number;
    startY: number;
    origX: number;
    origY: number;
    moved: boolean;
  } | null>(null);

  const cards = entries.data ?? [];
  const selected = cards.find((entry) => entry.id === selectedId);
  const grouped = useMemo(() => {
    const groups = new Map<WorldKind, WorldEntry[]>();
    for (const entry of cards) {
      const list = groups.get(entry.kind) ?? [];
      list.push(entry);
      groups.set(entry.kind, list);
    }
    return WORLD_KINDS.map((item) => ({
      kind: item,
      items: groups.get(item) ?? [],
    })).filter((group) => group.items.length > 0);
  }, [cards]);

  function pinScrap(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const title = draft.trim();
    if (!title) {
      return;
    }
    create.mutate(
      {
        kind,
        title,
        prompt: kind === "question" ? prompt : null,
      },
      {
        onSuccess: (entry) => {
          setDraft("");
          onSelect(entry.id);
          if (lens === "atlas" && kind === "place") {
            onLensChange("atlas");
          }
        },
      },
    );
  }

  function startDrag(event: PointerEvent<HTMLButtonElement>, entry: WorldEntry) {
    const live = positions[entry.id];
    dragRef.current = {
      id: entry.id,
      startX: event.clientX,
      startY: event.clientY,
      origX: live?.x ?? entry.board_x,
      origY: live?.y ?? entry.board_y,
      moved: false,
    };
    event.currentTarget.setPointerCapture(event.pointerId);
  }

  function moveDrag(event: PointerEvent<HTMLButtonElement>) {
    const drag = dragRef.current;
    const board = boardRef.current;
    if (!drag || !board) {
      return;
    }
    const dx = event.clientX - drag.startX;
    const dy = event.clientY - drag.startY;
    if (Math.abs(dx) + Math.abs(dy) < 4) {
      return;
    }
    drag.moved = true;
    const x = clamp(
      drag.origX + (dx / board.clientWidth) * 100,
      0,
      86,
    );
    const y = clamp(
      drag.origY + (dy / board.clientHeight) * 100,
      0,
      78,
    );
    setPositions((current) => ({ ...current, [drag.id]: { x, y } }));
  }

  function endDrag(entry: WorldEntry) {
    const drag = dragRef.current;
    dragRef.current = null;
    if (!drag) {
      return;
    }
    if (!drag.moved) {
      onSelect(entry.id);
      return;
    }
    const next = positions[entry.id];
    if (!next) {
      return;
    }
    update.mutate({ id: entry.id, board_x: next.x, board_y: next.y });
  }

  function dropOnMap(event: PointerEvent<HTMLDivElement>) {
    const rect = event.currentTarget.getBoundingClientRect();
    const map_x = clamp(((event.clientX - rect.left) / rect.width) * 100, 4, 96);
    const map_y = clamp(((event.clientY - rect.top) / rect.height) * 100, 6, 94);
    if (selected) {
      update.mutate({ id: selected.id, map_x, map_y });
      return;
    }
    create.mutate(
      {
        kind: "place",
        title: "A place without a name",
        map_x,
        map_y,
      },
      { onSuccess: (entry) => onSelect(entry.id) },
    );
  }

  const meter = pulse.data;
  const iceberg = Math.round((meter?.iceberg ?? 0) * 100);

  return (
    <section className="world-stage">
      <div className="world-ribbon">
        <div>
          <p className="eyebrow">World · optional</p>
          <h1>{projectName}</h1>
        </div>
        <div className="iceberg" title="How much of the world stays off the page">
          <div className="iceberg-mark" style={{ height: `${Math.max(18, iceberg)}%` }} />
          <div>
            <strong>{iceberg}%</strong>
            <span>underwater</span>
            <small>
              {icebergCaption(
                meter?.scenes ?? 0,
                meter?.underwater ?? 0,
                meter?.sparks ?? 0,
              )}
            </small>
          </div>
        </div>
      </div>

      <div className="world-toolbar">
        <div className="world-lenses" role="tablist" aria-label="World views">
          {(
            [
              ["wall", "Wall"],
              ["atlas", "Map"],
              ["dossier", "Dossier"],
            ] as const
          ).map(([value, label]) => (
            <button
              key={value}
              type="button"
              role="tab"
              aria-selected={lens === value}
              className={lens === value ? "active" : ""}
              onClick={() => onLensChange(value)}
            >
              {label}
            </button>
          ))}
        </div>
        <div className="world-seals" aria-label="Discoveries">
          {(meter?.discoveries ?? []).map((item) => (
            <span key={item} className="seal">
              {DISCOVERY_LABELS[item] ?? item}
            </span>
          ))}
          {!meter?.discoveries.length && (
            <span className="seal ghost">Pin a scrap to begin</span>
          )}
        </div>
      </div>

      {lens === "wall" && (
        <div className="cork-wall" ref={boardRef}>
          {entries.isPending && (
            <div className="wall-empty">
              <p>Gathering scraps…</p>
            </div>
          )}
          {!entries.isPending && cards.length === 0 && (
            <div className="wall-empty">
              <p>Baur kept napkins on a magnetic wall until they wanted an order.</p>
              <div className="invites">
                <button
                  type="button"
                  onClick={() => {
                    setKind("place");
                    setDraft("A harbor that only appears at dusk");
                  }}
                >
                  <small>Place</small>
                  Drop a harbor
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setKind("question");
                    setPrompt("why");
                    setDraft("Why do the tides speak?");
                  }}
                >
                  <small>Idea net</small>
                  Ask why
                </button>
                <button type="button" onClick={onWriteScene}>
                  <small>Gardener</small>
                  Write a scene anyway
                </button>
              </div>
            </div>
          )}
          {cards.map((entry) => {
            const live = positions[entry.id];
            return (
              <button
                key={entry.id}
                type="button"
                className={`world-card hue-${entry.hue} ${entry.status} ${
                  entry.id === selectedId ? "selected" : ""
                }`}
                style={{
                  left: `${live?.x ?? entry.board_x}%`,
                  top: `${live?.y ?? entry.board_y}%`,
                  transform: `rotate(${cardTilt(entry.id)}deg)`,
                }}
                onPointerDown={(event) => startDrag(event, entry)}
                onPointerMove={moveDrag}
                onPointerUp={() => endDrag(entry)}
              >
                <span className="card-kind">
                  {entry.prompt ? promptLabel(entry.prompt) : kindLabel(entry.kind)}
                </span>
                <strong>{entry.title}</strong>
                {entry.summary && <p>{entry.summary}</p>}
                <em>{statusLabel(entry.status)}</em>
              </button>
            );
          })}
        </div>
      )}

      {lens === "atlas" && (
        <div className="atlas-shell">
          <div
            className="atlas-map"
            onPointerUp={dropOnMap}
            role="application"
            aria-label="Rough map. Click to drop a place."
          >
            <svg viewBox="0 0 800 520" className="atlas-drawing" aria-hidden="true">
              <path
                d="M20 80 C 140 40, 220 130, 340 90 S 520 40, 680 110 790 240 760 360 620 500 400 470 40 430 30 280 Z"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.4"
              />
              <path
                d="M210 220 C 260 260, 300 300, 280 380"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.2"
              />
              <polygon points="470,210 500,140 530,210" />
              <polygon points="520,230 545,170 575,230" />
              <polygon points="560,220 590,155 620,220" />
            </svg>
            <p className="atlas-hint">
              Ugly triangles are enough. Click to pin a place — or pin the card
              you already chose.
            </p>
            {cards
              .filter((entry) => entry.map_x != null && entry.map_y != null)
              .map((entry) => (
                <button
                  key={entry.id}
                  type="button"
                  className={`map-pin hue-${entry.hue} ${
                    entry.id === selectedId ? "selected" : ""
                  }`}
                  style={{ left: `${entry.map_x}%`, top: `${entry.map_y}%` }}
                  onPointerUp={(event) => {
                    event.stopPropagation();
                    onSelect(entry.id);
                  }}
                >
                  <span />
                  {entry.title}
                </button>
              ))}
          </div>
          <aside className="unplaced">
            <p className="eyebrow">Not on the map</p>
            {cards.filter((entry) => entry.map_x == null).length === 0 && (
              <p>Everything has a pin, or the atlas is still empty.</p>
            )}
            {cards
              .filter((entry) => entry.map_x == null)
              .map((entry) => (
                <button
                  key={entry.id}
                  type="button"
                  className={entry.id === selectedId ? "selected" : ""}
                  onClick={() => onSelect(entry.id)}
                >
                  {kindLabel(entry.kind)} · {entry.title}
                </button>
              ))}
          </aside>
        </div>
      )}

      {lens === "dossier" && (
        <div className="dossier">
          {cards.length === 0 && (
            <p>No dossier yet. The wall and the manuscript can stay ahead of this list.</p>
          )}
          {grouped.map((group) => (
            <section key={group.kind}>
              <h2>{kindLabel(group.kind)}</h2>
              <ul>
                {group.items.map((entry) => (
                  <li key={entry.id}>
                    <button
                      type="button"
                      className={entry.id === selectedId ? "selected" : ""}
                      onClick={() => onSelect(entry.id)}
                    >
                      <strong>{entry.title}</strong>
                      <span>{statusLabel(entry.status)}</span>
                      {entry.summary && <p>{entry.summary}</p>}
                    </button>
                  </li>
                ))}
              </ul>
            </section>
          ))}
        </div>
      )}

      <form className="world-capture" onSubmit={pinScrap}>
        <div className="capture-kinds">
          {WORLD_KINDS.map((item) => (
            <button
              key={item}
              type="button"
              className={kind === item ? "active" : ""}
              onClick={() => setKind(item)}
            >
              {kindLabel(item)}
            </button>
          ))}
        </div>
        {kind === "question" && (
          <div className="capture-kinds prompts">
            {PROMPTS.map((item) => (
              <button
                key={item}
                type="button"
                className={prompt === item ? "active" : ""}
                onClick={() => setPrompt(item)}
              >
                {promptLabel(item)}
              </button>
            ))}
          </div>
        )}
        <label>
          <span>{kindHint(kind)}</span>
          <div className="capture-line">
            <input
              value={draft}
              onChange={(event) => setDraft(event.target.value)}
              placeholder={
                kind === "question"
                  ? "Why does the gate have no entrance?"
                  : "A scrap, a place, a rule…"
              }
            />
            <button type="submit" disabled={create.isPending || !draft.trim()}>
              Pin
            </button>
          </div>
        </label>
        <button type="button" className="quiet skip-write" onClick={onWriteScene}>
          Skip and write
        </button>
        {create.error && <p className="error">{create.error.message}</p>}
      </form>
    </section>
  );
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}
