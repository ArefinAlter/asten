import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type {
  Calendar,
  DiagnosticsSnapshot,
  Strictness,
  TimeUnit,
  UpdateSettings,
} from "@asten/contracts";
import { type InputHTMLAttributes, useEffect, useState } from "react";
import { createAstenClient } from "../lib/asten-client";

interface Props {
  projectId: string;
  snapshot?: DiagnosticsSnapshot;
}

/** How strictly to weigh, how time is counted, and what the narrator is worth. */
export function Settings({ projectId, snapshot }: Props) {
  const client = createAstenClient();
  const queryClient = useQueryClient();
  const settings = useQuery({
    queryKey: ["settings", projectId],
    queryFn: () => client.settings(projectId),
  });
  const update = useMutation({
    mutationFn: (input: UpdateSettings) =>
      client.updateSettings(projectId, input),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: ["settings", projectId],
      });
      await queryClient.invalidateQueries({
        queryKey: ["diagnostics", projectId],
      });
    },
  });
  const desk = useQuery({
    queryKey: ["desk-info"],
    queryFn: () => client.deskInfo(),
  });
  const [origin, setOrigin] = useState("");
  useEffect(() => {
    client
      .origin()
      .then((value) => setOrigin(value || window.location.origin))
      .catch(() => setOrigin(window.location.origin));
  }, [client]);
  const data = settings.data;
  if (!data) return <p className="status">Loading…</p>;

  return (
    <section className="settings register">
      <header>
        <p className="eyebrow mono">Settings</p>
        <h1>How the scales are set</h1>
      </header>
      <div className="settings-grid">
        <fieldset>
          <legend className="mono">Strictness</legend>
          {(["quiet", "standard", "thorough"] as Strictness[]).map((level) => (
            <label key={level} className="row">
              <input
                type="radio"
                name="strictness"
                checked={data.strictness === level}
                onChange={() => update.mutate({ strictness: level })}
              />
              <span>
                <strong>{level[0].toUpperCase() + level.slice(1)}</strong>
                <small className="hint">
                  {level === "quiet"
                    ? " Only settled facts against settled facts."
                    : level === "standard"
                      ? " Plus what the page says against what you settled, above a confidence threshold."
                      : " Plus proposals against proposals, ranked, for revision passes."}
                </small>
              </span>
            </label>
          ))}
        </fieldset>
        <fieldset>
          <legend className="mono">Narration</legend>
          <label className="row">
            <input
              type="checkbox"
              checked={data.narrator_reliable}
              onChange={(event) =>
                update.mutate({ narrator_reliable: event.target.checked })
              }
            />
            <span>
              <strong>The narrator is reliable</strong>
              <small className="hint">
                {" "}
                Narration counts as world truth. Switch it off for an unreliable
                narrator; then only facts you settle are weighed.
              </small>
            </span>
          </label>
        </fieldset>
        <fieldset>
          <legend className="mono">Story time</legend>
          <label>
            <span>Base unit</span>
            <select
              value={data.clock.unit}
              onChange={(event) =>
                update.mutate({
                  clock: {
                    ...data.clock,
                    unit: event.target.value as TimeUnit,
                  },
                })
              }
            >
              <option value="day">days</option>
              <option value="hour">hours</option>
              <option value="minute">minutes</option>
            </select>
          </label>
          <label>
            <span>Calendar</span>
            <select
              value={data.clock.calendar.kind}
              onChange={(event) => {
                const kind = event.target.value;
                const calendar: Calendar =
                  kind === "gregorian"
                    ? { kind: "gregorian" }
                    : kind === "fictional"
                      ? {
                          kind: "fictional",
                          months: [
                            { name: "First", days: 30 },
                            { name: "Second", days: 30 },
                            { name: "Third", days: 30 },
                            { name: "Fourth", days: 30 },
                          ],
                          epoch_year: 1,
                          eras: [],
                          seasons: [],
                        }
                      : { kind: "day_count" };
                update.mutate({ clock: { ...data.clock, calendar } });
              }}
            >
              <option value="day_count">Day count ("day 14")</option>
              <option value="gregorian">
                Gregorian dates ("1890-03-02", "winter of 1890")
              </option>
              <option value="fictional">
                Fictional calendar (months you define)
              </option>
            </select>
          </label>
          {data.clock.calendar.kind === "fictional" && (
            <div className="months">
              {data.clock.calendar.months.map((month, index) => (
                <div key={index} className="row">
                  <DeferredInput
                    value={month.name}
                    onCommit={(value) => {
                      if (data.clock.calendar.kind !== "fictional") return;
                      const months = data.clock.calendar.months.map(
                        (item, i) =>
                          i === index ? { ...item, name: value } : item,
                      );
                      update.mutate({
                        clock: {
                          ...data.clock,
                          calendar: { ...data.clock.calendar, months },
                        },
                      });
                    }}
                  />
                  <DeferredInput
                    type="number"
                    min={1}
                    value={String(month.days)}
                    onCommit={(value) => {
                      if (data.clock.calendar.kind !== "fictional") return;
                      const days = Math.max(1, Math.floor(Number(value) || 1));
                      const months = data.clock.calendar.months.map(
                        (item, i) => (i === index ? { ...item, days } : item),
                      );
                      update.mutate({
                        clock: {
                          ...data.clock,
                          calendar: { ...data.clock.calendar, months },
                        },
                      });
                    }}
                  />
                </div>
              ))}
              <button
                type="button"
                className="linkish"
                onClick={() => {
                  if (data.clock.calendar.kind !== "fictional") return;
                  update.mutate({
                    clock: {
                      ...data.clock,
                      calendar: {
                        ...data.clock.calendar,
                        months: [
                          ...data.clock.calendar.months,
                          {
                            name: `Month ${data.clock.calendar.months.length + 1}`,
                            days: 30,
                          },
                        ],
                      },
                    },
                  });
                }}
              >
                + month
              </button>
            </div>
          )}
          <label>
            <span>Minimum age of a parent at a child's birth (days)</span>
            <DeferredInput
              type="number"
              min={0}
              placeholder="off"
              value={
                data.min_parent_age_days == null
                  ? ""
                  : String(data.min_parent_age_days)
              }
              onCommit={(value) =>
                update.mutate({
                  min_parent_age_days: value ? Number(value) : null,
                })
              }
            />
          </label>
        </fieldset>
        <fieldset>
          <legend className="mono">Languages in this project</legend>
          {snapshot?.tiers.length ? (
            <ul className="plain-list">
              {snapshot.tiers.map((tier) => (
                <li key={tier.script} className="row">
                  <strong>{tier.script}</strong>
                  <small className="mono">
                    tier {tier.tier} · {tier.blocks} block
                    {tier.blocks === 1 ? "" : "s"}
                  </small>
                </li>
              ))}
            </ul>
          ) : (
            <p className="hint">No prose yet.</p>
          )}
          <p className="hint">
            Tier C: checks between settled facts, any language. Tier B: plus
            names and lexicon attributes read from the page. Tier A: trained
            perception models, planned and local.
          </p>
        </fieldset>
        <fieldset>
          <legend className="mono">This desk</legend>
          {desk.data ? (
            <ul className="plain-list">
              <li className="row">
                <strong>Asten {desk.data.version}</strong>
                <small className="mono">{desk.data.host}</small>
              </li>
              <li className="row">
                <strong>
                  {desk.data.embedded
                    ? "The API runs inside this app"
                    : "The API is a server you started"}
                </strong>
                <small className="mono">{origin || "\u2026"}</small>
              </li>
              <li className="row">
                <strong>Authentication</strong>
                <small className="mono">{desk.data.auth_mode}</small>
              </li>
              <li className="row">
                <strong>Your file</strong>
                {desk.data.database_path ? (
                  <small
                    className="mono desk-path"
                    title={desk.data.database_path}
                  >
                    {desk.data.database_path}
                    <button
                      type="button"
                      className="linkish"
                      onClick={() => {
                        void navigator.clipboard
                          ?.writeText(desk.data?.database_path ?? "")
                          .catch(() => undefined);
                      }}
                    >
                      copy
                    </button>
                  </small>
                ) : (
                  <small className="hint">
                    On the machine that runs the API. Only its desk sees the
                    path.
                  </small>
                )}
              </li>
            </ul>
          ) : (
            <p className="hint">Asking the API about itself\u2026</p>
          )}
          <p className="hint">
            Back up by copying that one file while Asten is closed. Compile to
            Markdown or .docx for a copy that needs no Asten at all.
          </p>
        </fieldset>
      </div>
    </section>
  );
}

/**
 * An input that keeps its own text while you type and commits on blur or
 * Enter, so a setting is saved once per edit rather than once per keystroke.
 */
function DeferredInput({
  value,
  onCommit,
  ...rest
}: Omit<InputHTMLAttributes<HTMLInputElement>, "value" | "onChange"> & {
  value: string;
  onCommit: (value: string) => void;
}) {
  const [draft, setDraft] = useState(value);
  useEffect(() => {
    setDraft(value);
  }, [value]);
  function commit() {
    if (draft !== value) onCommit(draft);
  }
  return (
    <input
      {...rest}
      value={draft}
      onChange={(event) => setDraft(event.target.value)}
      onBlur={commit}
      onKeyDown={(event) => {
        if (event.key === "Enter") {
          event.preventDefault();
          commit();
        }
      }}
    />
  );
}
