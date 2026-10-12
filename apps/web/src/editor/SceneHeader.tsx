import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import type { AstenDocument, ScenePlacement } from "@asten/contracts";
import { createAstenClient } from "../lib/asten-client";
import { CONTEXT_CHOICES, contextValue } from "../lib/format";

interface Props {
  projectId: string;
  document: AstenDocument;
}

/**
 * The line above a scene: where it sits in story time, what frame it is told
 * in, and what the engine knows about it. The code lens of the desk.
 */
export function SceneHeader({ projectId, document }: Props) {
  const client = createAstenClient();
  const queryClient = useQueryClient();
  const [time, setTime] = useState(document.scene.story_time ?? "");
  const [run, setRun] = useState(document.scene.continuity_run ?? "");
  const [open, setOpen] = useState(false);

  useEffect(() => {
    setTime(document.scene.story_time ?? "");
    setRun(document.scene.continuity_run ?? "");
  }, [document.id, document.scene.story_time, document.scene.continuity_run]);

  const summary = useQuery({
    queryKey: ["scene-summary", projectId, document.id, document.head_seq],
    queryFn: () => client.sceneSummary(projectId, document.id),
    enabled:
      document.kind === "manuscript" ||
      document.kind === "chapter" ||
      document.kind === "scene",
  });

  const place = useMutation({
    mutationFn: (scene: ScenePlacement) =>
      client.updateDocument(projectId, document.id, { scene }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: ["documents", projectId],
      });
      await queryClient.invalidateQueries({ queryKey: ["scene-summary"] });
    },
  });

  function commitTime() {
    if ((document.scene.story_time ?? "") === time) return;
    place.mutate({
      ...document.scene,
      story_time: time.trim() || null,
      tick: null,
      tick_lo: null,
      tick_hi: null,
    });
  }

  function commitRun() {
    if ((document.scene.continuity_run ?? "") === run) return;
    place.mutate({ ...document.scene, continuity_run: run.trim() || null });
  }

  if (!(
    document.kind === "manuscript" ||
    document.kind === "chapter" ||
    document.kind === "scene"
  )) {
    return null;
  }

  const data = summary.data;
  return (
    <div className="scene-header">
      <div className="scene-lens">
        <button
          type="button"
          className="lens-toggle"
          onClick={() => setOpen((value) => !value)}
          aria-expanded={open}
        >
          <span className="mono">
            {data?.time_label || "unplaced"}
            {data &&
            data.bounds_label &&
            data.bounds_label !== "unplaced" &&
            data.bounds_label.toLowerCase() !==
              (data.time_label || "").toLowerCase()
              ? ` (${data.bounds_label})`
              : ""}
          </span>
          <span className="dot">·</span>
          <span>
            {data
              ? `${data.mentions} mention${data.mentions === 1 ? "" : "s"}`
              : "…"}
          </span>
          <span className="dot">·</span>
          <span>
            {data ? `${data.events} event${data.events === 1 ? "" : "s"}` : ""}
          </span>
          <span className="dot">·</span>
          <span className={data && data.marks > 0 ? "rubric" : "settled"}>
            {data
              ? data.marks > 0
                ? `${data.marks} mark${data.marks === 1 ? "" : "s"}`
                : "balanced"
              : ""}
          </span>
          {data?.context && data.context !== "narration" ? (
            <>
              <span className="dot">·</span>
              <span className="chip">{data.context}</span>
            </>
          ) : null}
          {data?.tier ? (
            <>
              <span className="dot">·</span>
              <span
                className="chip chip--tier"
                title={`Script ${data.script}: tier ${data.tier} checks are active`}
              >
                {data.script} · tier {data.tier}
              </span>
            </>
          ) : null}
        </button>
      </div>
      {open && (
        <div className="scene-controls">
          <label>
            <span>Story time</span>
            <input
              value={time}
              placeholder="day 14 · 1890-03-02 · three days after the siege"
              onChange={(event) => setTime(event.target.value)}
              onBlur={commitTime}
              onKeyDown={(event) => event.key === "Enter" && commitTime()}
            />
          </label>
          <label>
            <span>Told as</span>
            <select
              value={contextValue(document.scene.context)}
              onChange={(event) => {
                const choice = CONTEXT_CHOICES.find(
                  (item) => item.value === event.target.value,
                );
                if (choice)
                  place.mutate({ ...document.scene, context: choice.context });
              }}
            >
              {CONTEXT_CHOICES.map((choice) => (
                <option key={choice.value} value={choice.value}>
                  {choice.label}
                </option>
              ))}
            </select>
          </label>
          <label>
            <span>Continuous run</span>
            <input
              value={run}
              placeholder="one night"
              onChange={(event) => setRun(event.target.value)}
              onBlur={commitRun}
              onKeyDown={(event) => event.key === "Enter" && commitRun()}
            />
          </label>
          <p className="hint">
            A flashback is not an error. Only scenes you place in the same
            continuous run are checked for running backward.
          </p>
        </div>
      )}
    </div>
  );
}
