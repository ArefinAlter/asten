import {
  useMutation,
  useQueries,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { type FormEvent, useState } from "react";
import type {
  AstenDocument,
  CanonEntity,
  CreateTimelineEvent,
  SceneSummary,
  TimelineEvent,
} from "@asten/contracts";
import { createAstenClient } from "../lib/asten-client";
import {
  EVENT_KIND_CHOICES,
  buildEventKind,
  eventKindLabel,
} from "../lib/format";

interface Props {
  projectId: string;
  documents: AstenDocument[];
  entities: CanonEntity[];
  onOpenDocument: (id: string) => void;
}

/**
 * Two registers, like two bands on a tomb wall: reading order above, story time
 * below. Lines connect each scene's two positions; crossing lines are
 * flashbacks, and that is not an error.
 */
export function Timeline({
  projectId,
  documents,
  entities,
  onOpenDocument,
}: Props) {
  const client = createAstenClient();
  const queryClient = useQueryClient();
  const prose = documents.filter(
    (document) =>
      (document.kind === "manuscript" ||
        document.kind === "chapter" ||
        document.kind === "scene") &&
      !document.exclude_from_compile,
  );
  const events = useQuery({
    queryKey: ["events", projectId],
    queryFn: () => client.listEvents(projectId),
  });
  const summaries = useQueries({
    queries: prose.map((document) => ({
      queryKey: ["scene-summary", projectId, document.id, document.head_seq],
      queryFn: () => client.sceneSummary(projectId, document.id),
    })),
  });
  const scenes: SceneSummary[] = summaries
    .map((query) => query.data)
    .filter((summary): summary is SceneSummary => Boolean(summary))
    .sort((a, b) => a.reading_pos - b.reading_pos);

  const createEvent = useMutation({
    mutationFn: (input: CreateTimelineEvent) => client.createEvent(input),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["events", projectId] });
      await queryClient.invalidateQueries({
        queryKey: ["diagnostics", projectId],
      });
    },
  });
  const deleteEvent = useMutation({
    mutationFn: (id: string) => client.deleteEvent(projectId, id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["events", projectId] });
      await queryClient.invalidateQueries({
        queryKey: ["diagnostics", projectId],
      });
    },
  });
  const placeScene = useMutation({
    mutationFn: (input: { document: AstenDocument; story_time: string }) =>
      client.updateDocument(projectId, input.document.id, {
        scene: {
          ...input.document.scene,
          story_time: input.story_time || null,
          tick: null,
          tick_lo: null,
          tick_hi: null,
        },
      }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: ["documents", projectId],
      });
      await queryClient.invalidateQueries({ queryKey: ["scene-summary"] });
    },
  });

  const [kind, setKind] = useState("acts");
  const choice = EVENT_KIND_CHOICES.find((item) => item.value === kind)!;

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    createEvent.mutate({
      project_id: projectId,
      title: String(form.get("title") ?? ""),
      story_time: String(form.get("story_time") ?? "") || null,
      entity_id: String(form.get("entity_id") || "") || null,
      document_id: String(form.get("document_id") || "") || null,
      notes: String(form.get("notes") ?? ""),
      kind: buildEventKind(kind, String(form.get("argument") ?? "")),
    });
    event.currentTarget.reset();
  }

  const placed = scenes.filter(
    (scene) => scene.bounds.lo != null || scene.bounds.hi != null,
  );
  const range = (() => {
    const los = placed.map((scene) => scene.bounds.lo ?? scene.bounds.hi ?? 0);
    const his = placed.map((scene) => scene.bounds.hi ?? scene.bounds.lo ?? 0);
    const min = Math.min(...los, 0);
    const max = Math.max(...his, 1);
    return { min, max: max === min ? min + 1 : max };
  })();

  const width = Math.max(640, scenes.length * 110);
  const x = (tick: number) =>
    40 + ((tick - range.min) / (range.max - range.min)) * (width - 80);
  const ordered = [...(events.data ?? [])].sort(
    (a, b) =>
      (a.tick ?? a.reading_order ?? 0) - (b.tick ?? b.reading_order ?? 0),
  );

  return (
    <section className="timeline register">
      <header>
        <p className="eyebrow mono">Two clocks</p>
        <h1>Reading order is not story time</h1>
        <p className="lede">
          A flashback is not a bug. Asten argues only inside a run you declared
          continuous, when the dates cannot all be true, or when an effect is
          seen before its cause.
        </p>
      </header>

      <div
        className="registers"
        role="img"
        aria-label="Scenes in reading order above and in story time below"
      >
        <svg
          viewBox={`0 0 ${width} 220`}
          width="100%"
          preserveAspectRatio="xMidYMid meet"
        >
          <line x1="20" x2={width - 20} y1="48" y2="48" className="ground" />
          <line x1="20" x2={width - 20} y1="172" y2="172" className="ground" />
          <text x="24" y="24" className="register-label">
            READING ORDER
          </text>
          <text x="24" y="206" className="register-label">
            STORY TIME
          </text>
          {scenes.map((scene, index) => {
            const rx =
              60 +
              (index * (width - 120)) / Math.max(1, scenes.length - 1 || 1);
            const isPlaced = scene.bounds.lo != null || scene.bounds.hi != null;
            const lo = scene.bounds.lo ?? scene.bounds.hi ?? 0;
            const hi = scene.bounds.hi ?? scene.bounds.lo ?? 0;
            const sx = isPlaced ? x(lo) : undefined;
            const ex = isPlaced ? x(hi) : undefined;
            const crossing =
              isPlaced &&
              scenes
                .slice(0, index)
                .some(
                  (earlier) =>
                    (earlier.bounds.lo ?? earlier.bounds.hi ?? -Infinity) > hi,
                );
            return (
              <g
                key={scene.document}
                className={`scene-pair${crossing ? " flashback" : ""}`}
              >
                <g
                  className="reading-tick"
                  onClick={() => onOpenDocument(scene.document)}
                  role="button"
                  tabIndex={0}
                >
                  <circle cx={rx} cy="48" r="6" />
                  <text
                    x={rx}
                    y="36"
                    textAnchor="middle"
                    className="scene-label"
                  >
                    {scene.title.length > 16
                      ? `${scene.title.slice(0, 15)}…`
                      : scene.title}
                  </text>
                </g>
                {isPlaced && sx !== undefined && ex !== undefined ? (
                  <>
                    <line
                      x1={rx}
                      y1="54"
                      x2={(sx + ex) / 2}
                      y2="166"
                      className="link"
                    />
                    <rect
                      x={sx - 4}
                      y="166"
                      width={Math.max(8, ex - sx + 8)}
                      height="12"
                      rx="6"
                      className="story-span"
                    />
                    <text
                      x={(sx + ex) / 2}
                      y="194"
                      textAnchor="middle"
                      className="scene-time"
                    >
                      {scene.time_label || scene.bounds_label}
                    </text>
                  </>
                ) : (
                  <text
                    x={rx}
                    y="66"
                    textAnchor="middle"
                    className="scene-time unplaced"
                  >
                    unplaced
                  </text>
                )}
              </g>
            );
          })}
          {ordered
            .filter((event) => event.tick != null)
            .map((event) => (
              <g key={event.id} className="event-tick">
                <path d={`M ${x(event.tick!)} 160 l -5 -8 l 10 0 z`} />
              </g>
            ))}
        </svg>
      </div>

      <div className="timeline-columns">
        <div>
          <h2>Scenes</h2>
          <ul className="plain-list">
            {prose.map((document) => (
              <li key={document.id} className="row">
                <button
                  type="button"
                  className="linkish"
                  onClick={() => onOpenDocument(document.id)}
                >
                  {document.title}
                </button>
                <input
                  className="inline-time"
                  defaultValue={document.scene.story_time ?? ""}
                  placeholder="day 14"
                  onBlur={(event) => {
                    if (
                      event.target.value !== (document.scene.story_time ?? "")
                    )
                      placeScene.mutate({
                        document,
                        story_time: event.target.value.trim(),
                      });
                  }}
                  aria-label={`Story time of ${document.title}`}
                />
                <small className="mono">
                  {scenes.find((scene) => scene.document === document.id)
                    ?.bounds_label ?? ""}
                </small>
              </li>
            ))}
          </ul>
        </div>
        <div>
          <h2>Beats</h2>
          <ol className="plain-list">
            {ordered.map((event: TimelineEvent) => {
              const who = entities.find(
                (entity) => entity.id === event.entity_id,
              );
              return (
                <li key={event.id} className="row">
                  <span>
                    <strong>{event.title}</strong>
                    <small className="mono">
                      {" "}
                      {who ? `${who.name} ` : ""}
                      {eventKindLabel(event.kind)}
                      {event.story_time
                        ? ` · ${event.story_time}`
                        : event.tick != null
                          ? ` · tick ${event.tick}`
                          : event.document_id
                            ? ` · with its scene`
                            : " · unplaced"}
                    </small>
                  </span>
                  <button
                    type="button"
                    className="linkish rubric"
                    onClick={() => deleteEvent.mutate(event.id)}
                  >
                    remove
                  </button>
                </li>
              );
            })}
            {ordered.length === 0 && (
              <li className="hint">
                No beats yet. A death, a birth, a journey, a secret learned.
              </li>
            )}
          </ol>
          <form className="stacked-form" onSubmit={submit}>
            <h3 className="mono">Pin a beat</h3>
            <input
              name="title"
              placeholder="Oren dies at the harbor"
              required
            />
            <select
              value={kind}
              onChange={(event) => setKind(event.target.value)}
              aria-label="What happens"
            >
              {EVENT_KIND_CHOICES.map((item) => (
                <option key={item.value} value={item.value}>
                  {item.label}
                </option>
              ))}
            </select>
            {choice.needs && (
              <input
                name="argument"
                placeholder={
                  choice.needs === "secret"
                    ? "the secret, in a few words"
                    : choice.needs === "label"
                      ? "what happens"
                      : `the ${choice.needs}`
                }
                required
              />
            )}
            <select name="entity_id" defaultValue="">
              <option value="">Who (optional)</option>
              {entities.map((entity) => (
                <option key={entity.id} value={entity.id}>
                  {entity.name}
                </option>
              ))}
            </select>
            <input
              name="story_time"
              placeholder="day 14 · 1890-03-02 · three days after the siege"
            />
            <select name="document_id" defaultValue="">
              <option value="">Or: at the time of a scene…</option>
              {prose.map((document) => (
                <option key={document.id} value={document.id}>
                  {document.title}
                </option>
              ))}
            </select>
            <textarea name="notes" rows={2} placeholder="Notes" />
            <button
              type="submit"
              className="btn btn--small btn--primary"
              disabled={createEvent.isPending}
            >
              Pin
            </button>
          </form>
        </div>
      </div>
    </section>
  );
}
