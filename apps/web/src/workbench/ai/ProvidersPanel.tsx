import { useState } from "react";
import { useWorkbench } from "../context";
import { Icon } from "../icons";
import { Card, Fixed, Head, SettingRow, Switch } from "../settings/parts";
import {
  ai,
  newRunId,
  refreshStatus,
  spendLine,
  useAi,
  type AiStatus,
  type LedgerView,
  type Preview,
  type ProviderView,
} from "./bridge";
import { PreviewDialog } from "./PreviewDialog";

const NIL_PROJECT = "00000000-0000-0000-0000-000000000000";

function describe(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return "Something went wrong";
}

const BACKEND_LABEL: Record<AiStatus["vault"]["backend"], string> = {
  os: "your operating system's credential store",
  file: "an encrypted file on this machine, guarded by a passphrase",
  memory: "memory only (gone when Asten closes)",
};

/**
 * The AI providers screen of the desktop app. Everything it shows comes from
 * the shell; keys go in through `ai.setKey` and never come back.
 */
export function ProvidersDesktop() {
  const { status, error } = useAi();
  const api = useWorkbench();
  const [busy, setBusy] = useState<string>();
  const [problem, setProblem] = useState<string>();
  const [consentFor, setConsentFor] = useState<ProviderView>();
  const [preview, setPreview] = useState<Preview>();
  const [keyDraft, setKeyDraft] = useState<Record<string, string>>({});
  const [passphrase, setPassphrase] = useState("");
  const [tests, setTests] = useState<Record<string, string>>({});
  const [models, setModels] = useState<Record<string, string[]>>({});
  const [caps, setCaps] = useState<{
    daily_tokens: string;
    thread_tokens: string;
    daily_usd: string;
  }>();
  const [ledger, setLedger] = useState<LedgerView>();
  const [added, setAdded] = useState({ name: "", endpoint: "", model: "" });

  if (error && !status) {
    return (
      <>
        <Head title="AI providers">
          Model providers run in the desktop app on Windows, macOS and Linux.
          This build has no provider layer, so the assistant answers from
          Asten's own tools.
        </Head>
        <Card>
          <Fixed
            title="This build"
            note="No request goes to any provider from here."
          />
        </Card>
      </>
    );
  }
  if (!status)
    return <p className="status">Reading the assistant's settings…</p>;

  const guarded = async (label: string, run: () => Promise<unknown>) => {
    setBusy(label);
    setProblem(undefined);
    try {
      await run();
    } catch (failure) {
      setProblem(describe(failure));
    } finally {
      setBusy(undefined);
    }
  };

  const samplePreview = async (provider: ProviderView) => {
    try {
      const sample = await ai.preview({
        run_id: newRunId(),
        thread_id: "settings",
        provider: provider.id,
        project_id: api.projectId ?? NIL_PROJECT,
        access: "propose",
        disabled_tools: [],
        text: "Explain the selected mark",
        history: [],
        attachments: [],
      });
      setPreview(sample);
    } catch (failure) {
      setProblem(describe(failure));
    }
  };

  const move = (index: number, by: -1 | 1) => {
    const order = status.providers.map((p) => p.id);
    const to = index + by;
    if (to < 0 || to >= order.length) return;
    [order[index], order[to]] = [order[to], order[index]];
    void guarded("order", () => ai.update({ order }));
  };

  const capValues = caps ?? {
    daily_tokens: String(status.caps.daily_tokens),
    thread_tokens: String(status.caps.thread_tokens),
    daily_usd: String(status.caps.daily_usd),
  };

  return (
    <>
      <Head title="AI providers">
        Use a local model, an API key or an installed agent's existing sign-in.
        Cloud connections require your consent. Your agent's plan limits and
        extra-usage settings apply. Grok Build needs its own official sign-in in a
        dedicated profile; test the connection for setup instructions.
      </Head>
      {problem && (
        <p className="settings-problem" role="alert">
          {problem}
        </p>
      )}
      <Card>
        <SettingRow
          title="Allow AI providers"
          note="Off by default. Off means no provider is ever contacted, whatever is set below."
        >
          <Switch
            label="Allow AI providers"
            value={status.enabled}
            disabled={busy === "enabled"}
            onChange={(value) =>
              void guarded("enabled", () => ai.update({ enabled: value }))
            }
          />
        </SettingRow>
        <SettingRow
          title="Offline mode"
          note="Nothing leaves this machine. A model running on this computer still works; every cloud provider is blocked until you turn this off."
        >
          <Switch
            label="Offline mode"
            value={status.offline}
            onChange={(value) =>
              void guarded("offline", () => ai.update({ offline: value }))
            }
          />
        </SettingRow>
        <SettingRow
          title="Local first"
          note="Try a model on this machine before any cloud provider when the assistant is set to Auto."
        >
          <Switch
            label="Local first"
            value={status.local_first}
            onChange={(value) =>
              void guarded("local", () => ai.update({ local_first: value }))
            }
          />
        </SettingRow>
        <Fixed
          title="What a provider may be asked"
          note="Explain a mark, search the canon, read the timeline, summarise a research note you point at, and propose facts you settle yourself. Never to write, rewrite or continue prose."
        />
        <Fixed
          title="What is sent"
          note="Your question; what the tools return (marks with the author's wording left out, canon facts, the research note you ask about); and, only when you pick one, a passage. Never the whole manuscript, never another project, never your key to anyone but that provider."
        />
        <Fixed
          title="Training"
          note="Verne receives nothing, and no manuscript is used to train a model by Asten. What a provider does with a request is set by its own terms and your account with it."
        />
      </Card>

      <Card title="Keys">
        <SettingRow
          title="Where keys are kept"
          note={
            <>
              In {BACKEND_LABEL[status.vault.backend]}. The page types a key
              once and never gets it back; it shows the last four characters and
              nothing more.
              {status.vault.note && status.vault.backend === "file" && (
                <> ({status.vault.note})</>
              )}
            </>
          }
        >
          <span
            className={`wb-pill ${status.vault.backend === "os" ? "wb-pill--ok" : ""}`}
          >
            {status.vault.backend === "os"
              ? "credential store"
              : status.vault.backend === "file"
                ? "encrypted file"
                : "memory"}
          </span>
        </SettingRow>
        {status.vault.backend === "file" && status.vault.locked && (
          <SettingRow
            title={
              status.vault.needs_setup
                ? "Choose a passphrase"
                : "Unlock the key file"
            }
            note="At least eight characters. It is not stored anywhere; without it the keys cannot be read."
          >
            <input
              type="password"
              className="wb-input"
              autoComplete="off"
              aria-label="Key file passphrase"
              value={passphrase}
              onChange={(event) => setPassphrase(event.target.value)}
            />
            <button
              type="button"
              className="wb-btn"
              disabled={passphrase.length < 8 || busy === "unlock"}
              onClick={() =>
                void guarded("unlock", async () => {
                  await ai.unlock(passphrase);
                  setPassphrase("");
                })
              }
            >
              {status.vault.needs_setup ? "Create" : "Unlock"}
            </button>
          </SettingRow>
        )}
      </Card>

      <Card title="Spend">
        <SettingRow
          title="Today"
          note={
            status.totals.day_has_unpriced
              ? "Some calls have no known price (edit prices below); they count in tokens only."
              : "Estimated from the price table below. The provider's own bill is the one that counts."
          }
        >
          <span className="mono small">
            {spendLine(status.totals, status.caps)}
          </span>
        </SettingRow>
        <SettingRow
          title="Caps"
          note="A cap of zero means no model call. Checked before every request."
        >
          <div className="cap-fields">
            <label>
              Tokens per day
              <input
                className="wb-input"
                inputMode="numeric"
                value={capValues.daily_tokens}
                onChange={(event) =>
                  setCaps({ ...capValues, daily_tokens: event.target.value })
                }
              />
            </label>
            <label>
              Tokens per thread
              <input
                className="wb-input"
                inputMode="numeric"
                value={capValues.thread_tokens}
                onChange={(event) =>
                  setCaps({ ...capValues, thread_tokens: event.target.value })
                }
              />
            </label>
            <label>
              Dollars per day
              <input
                className="wb-input"
                inputMode="decimal"
                value={capValues.daily_usd}
                onChange={(event) =>
                  setCaps({ ...capValues, daily_usd: event.target.value })
                }
              />
            </label>
            <button
              type="button"
              className="wb-btn"
              disabled={!caps || busy === "caps"}
              onClick={() =>
                void guarded("caps", async () => {
                  const next = {
                    daily_tokens: Math.max(
                      0,
                      Math.floor(Number(capValues.daily_tokens)),
                    ),
                    thread_tokens: Math.max(
                      0,
                      Math.floor(Number(capValues.thread_tokens)),
                    ),
                    daily_usd: Math.max(0, Number(capValues.daily_usd)),
                  };
                  if (
                    [
                      next.daily_tokens,
                      next.thread_tokens,
                      next.daily_usd,
                    ].some((n) => !Number.isFinite(n))
                  )
                    throw new Error("Caps must be numbers");
                  await ai.update({ caps: next });
                  setCaps(undefined);
                })
              }
            >
              Save caps
            </button>
          </div>
        </SettingRow>
        <SettingRow
          title="History"
          note="Every provider call: tokens in and out, and the estimated cost. Kept on this machine."
        >
          <button
            type="button"
            className="wb-btn wb-btn--small"
            onClick={() => void ai.ledger().then(setLedger)}
          >
            Show recent calls
          </button>
          <button
            type="button"
            className="wb-btn wb-btn--small wb-btn--ghost"
            onClick={() =>
              void guarded("ledger", async () => {
                await ai.clearLedger();
                setLedger(undefined);
                await ai.update({});
              })
            }
          >
            Clear
          </button>
        </SettingRow>
        {ledger && (
          <table className="wb-table compact ledger-table">
            <thead>
              <tr>
                <th>When</th>
                <th>Provider</th>
                <th>In</th>
                <th>Out</th>
                <th>Cost</th>
              </tr>
            </thead>
            <tbody>
              {ledger.recent.slice(0, 10).map((entry, index) => (
                <tr key={`${entry.run_id}-${index}`}>
                  <td>{new Date(entry.at).toLocaleString()}</td>
                  <td>
                    {entry.provider} · {entry.model}
                  </td>
                  <td>{entry.input_tokens}</td>
                  <td>{entry.output_tokens}</td>
                  <td>
                    {entry.cost_usd == null
                      ? "unpriced"
                      : `$${entry.cost_usd.toFixed(5)}`}
                    {entry.estimated ? " (estimated)" : ""}
                  </td>
                </tr>
              ))}
              {ledger.recent.length === 0 && (
                <tr>
                  <td colSpan={5} className="muted">
                    No calls yet.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        )}
      </Card>

      <Card title="Installed agent connections">
        <p className="muted small">
          Asten detects supported programs on this computer without reading their credentials.
          Sign in using the agent's own app, then give consent and test its connection below.
          Finding a program does not verify its sign-in or subscription.
        </p>
        <button className="wb-btn" disabled={busy === "connections"}
          onClick={() => void guarded("connections", () => refreshStatus())}>
          Refresh installed agents
        </button>
      </Card>
      <section className={`wb-card${status.enabled ? "" : " is-off"}`}>
        <h2>Ladder</h2>
        <p className="muted small">
          Tried in this order when the assistant is set to Auto. Choosing one
          provider in the assistant panel skips the ladder.
        </p>
        <div className="wb-rows">
          {status.providers.map((p, index) => (
            <div key={p.id} className="provider-row provider-row--rich">
              <span className="ladder-n">{index + 1}</span>
              <div className="setting-text">
                <strong>
                  {p.name}{" "}
                  <span
                    className={`wb-pill wb-pill--${p.local ? "ok" : "muted"}`}
                  >
                    {p.endpoint.includes(".local-agent.invalid/")
                      ? "existing agent sign-in · plan limits apply"
                      : p.local
                        ? "this computer"
                        : p.kind}
                  </span>
                  {p.consent_valid && (
                    <span className="wb-pill wb-pill--ok">consent given</span>
                  )}
                  {!p.not_ready && (
                    <span className="wb-pill wb-pill--ok">ready</span>
                  )}
                </strong>
                <small className="mono">
                  {p.endpoint.includes(".local-agent.invalid/")
                    ? p.installed === true ? "Agent program found · sign-in verified when tested" : p.installed === false ? "Agent program not found · install it and sign in first" : "Uses your installed agent"
                    : p.host}
                </small>
                {p.enabled && p.not_ready && (
                  <small className="not-ready">{p.not_ready}</small>
                )}
                <div className="provider-fields">
                  <label>
                    Model
                    <input
                      className="wb-input"
                      list={`models-${p.id}`}
                      defaultValue={p.model}
                      key={`${p.id}-${p.model}`}
                      aria-label={`${p.name} model`}
                      onBlur={(event) => {
                        const value = event.target.value.trim();
                        if (value && value !== p.model)
                          void guarded("model", () =>
                            ai.update({
                              providers: [{ id: p.id, model: value }],
                            }),
                          );
                      }}
                    />
                    <datalist id={`models-${p.id}`}>
                      {[...p.suggested_models, ...(models[p.id] ?? [])].map(
                        (m) => (
                          <option key={m} value={m} />
                        ),
                      )}
                    </datalist>
                  </label>
                  {!p.endpoint.includes(".local-agent.invalid/") && (
                    <label>
                      Base URL
                      <input
                        className="wb-input mono"
                        defaultValue={p.endpoint}
                        key={`${p.id}-${p.endpoint}`}
                        aria-label={`${p.name} base URL`}
                        onBlur={(event) => {
                          const value = event.target.value.trim();
                          if (value && value !== p.endpoint)
                            void guarded("endpoint", () =>
                              ai.update({
                                providers: [{ id: p.id, endpoint: value }],
                              }),
                            );
                        }}
                      />
                    </label>
                  )}
                </div>
                {(p.kind === "cloud" || p.custom) &&
                  !p.endpoint.includes(".local-agent.invalid/") && (
                    <div className="key-row">
                      {p.key_stored ? (
                        <>
                          <span className="wb-pill">
                            key stored · ••••{p.key_last4}
                          </span>
                          <button
                            type="button"
                            className="wb-btn wb-btn--small wb-btn--ghost"
                            onClick={() =>
                              void guarded("forget", () => ai.forgetKey(p.id))
                            }
                          >
                            Forget
                          </button>
                        </>
                      ) : (
                        <>
                          <input
                            type="password"
                            className="wb-input"
                            autoComplete="off"
                            spellCheck={false}
                            placeholder="Paste your key"
                            aria-label={`${p.name} key`}
                            value={keyDraft[p.id] ?? ""}
                            onChange={(event) =>
                              setKeyDraft({
                                ...keyDraft,
                                [p.id]: event.target.value,
                              })
                            }
                          />
                          <button
                            type="button"
                            className="wb-btn wb-btn--small"
                            disabled={
                              !keyDraft[p.id]?.trim() ||
                              (status.vault.backend === "file" &&
                                status.vault.locked)
                            }
                            onClick={() => {
                              const key = keyDraft[p.id] ?? "";
                              setKeyDraft({ ...keyDraft, [p.id]: "" });
                              void guarded("key", () => ai.setKey(p.id, key));
                            }}
                          >
                            Store key
                          </button>
                        </>
                      )}
                    </div>
                  )}
                <div className="provider-fields provider-fields--small">
                  <label>
                    Tool calling
                    <select
                      className="wb-select"
                      value={p.tool_mode}
                      onChange={(event) =>
                        void guarded("tools", () =>
                          ai.update({
                            providers: [
                              {
                                id: p.id,
                                tool_mode: event.target.value as
                                  "native" | "json",
                              },
                            ],
                          }),
                        )
                      }
                    >
                      <option value="native">
                        Native (the provider's own)
                      </option>
                      <option value="json">
                        JSON protocol (no native tools)
                      </option>
                    </select>
                  </label>
                  <label>
                    Price per million tokens (in / out, $)
                    <span className="price-pair">
                      <input
                        className="wb-input"
                        inputMode="decimal"
                        aria-label={`${p.name} input price`}
                        defaultValue={p.price?.input_per_mtok ?? ""}
                        key={`${p.id}-${p.model}-in-${p.price?.input_per_mtok}`}
                        placeholder="unknown"
                        onBlur={(event) =>
                          savePrice(p, event.target.value, undefined)
                        }
                      />
                      <input
                        className="wb-input"
                        inputMode="decimal"
                        aria-label={`${p.name} output price`}
                        defaultValue={p.price?.output_per_mtok ?? ""}
                        key={`${p.id}-${p.model}-out-${p.price?.output_per_mtok}`}
                        placeholder="unknown"
                        onBlur={(event) =>
                          savePrice(p, undefined, event.target.value)
                        }
                      />
                    </span>
                  </label>
                </div>
                <div className="provider-actions">
                  <button
                    type="button"
                    className="wb-btn wb-btn--small"
                    disabled={!!p.not_ready || busy === `test-${p.id}`}
                    title={p.not_ready ?? "Send one tiny request"}
                    onClick={() =>
                      void guarded(`test-${p.id}`, async () => {
                        setTests({ ...tests, [p.id]: "Asking…" });
                        try {
                          const reply = await ai.test(p.id);
                          setTests((t) => ({
                            ...t,
                            [p.id]: `Answered: “${reply}”`,
                          }));
                        } catch (failure) {
                          setTests((t) => ({
                            ...t,
                            [p.id]: describe(failure),
                          }));
                        }
                        await ai.update({});
                      })
                    }
                  >
                    Test
                  </button>
                  {p.kind === "local" && (
                    <button
                      type="button"
                      className="wb-btn wb-btn--small"
                      onClick={() =>
                        void ai
                          .localModels(p.id)
                          .then((found) =>
                            setModels({ ...models, [p.id]: found }),
                          )
                          .catch((failure) => setProblem(describe(failure)))
                      }
                    >
                      Detect models
                    </button>
                  )}
                  <button
                    type="button"
                    className="wb-btn wb-btn--small"
                    onClick={() => void samplePreview(p)}
                  >
                    Show the request
                  </button>
                  {p.custom && (
                    <button
                      type="button"
                      className="wb-btn wb-btn--small wb-btn--ghost"
                      onClick={() =>
                        void guarded("remove", () => ai.removeProvider(p.id))
                      }
                    >
                      Remove
                    </button>
                  )}
                  {tests[p.id] && (
                    <span className="muted small">{tests[p.id]}</span>
                  )}
                </div>
              </div>
              <div className="setting-control">
                <button
                  type="button"
                  className="wb-icon-btn"
                  aria-label="Move up"
                  disabled={index === 0}
                  onClick={() => move(index, -1)}
                >
                  <Icon name="chevronUp" />
                </button>
                <button
                  type="button"
                  className="wb-icon-btn"
                  aria-label="Move down"
                  disabled={index === status.providers.length - 1}
                  onClick={() => move(index, 1)}
                >
                  <Icon name="chevronDown" />
                </button>
                <Switch
                  label={`Use ${p.name}`}
                  value={p.enabled}
                  onChange={(value) => {
                    if (value && !p.consent_valid) setConsentFor(p);
                    else
                      void guarded("use", () =>
                        ai.update({
                          providers: [{ id: p.id, enabled: value }],
                        }),
                      );
                  }}
                />
              </div>
            </div>
          ))}
        </div>
      </section>

      <Card title="Add an OpenAI-compatible service">
        <SettingRow
          title="Any service that speaks the Chat Completions format"
          note="A base URL such as https://api.example.org/v1, or http://127.0.0.1:8080/v1 for a server on this computer. Keys travel in a header, never in a URL."
        >
          <div className="add-service">
            <input
              className="wb-input"
              placeholder="Name"
              aria-label="Service name"
              value={added.name}
              onChange={(event) =>
                setAdded({ ...added, name: event.target.value })
              }
            />
            <input
              className="wb-input mono"
              placeholder="Base URL"
              aria-label="Service base URL"
              value={added.endpoint}
              onChange={(event) =>
                setAdded({ ...added, endpoint: event.target.value })
              }
            />
            <input
              className="wb-input"
              placeholder="Model"
              aria-label="Service model"
              value={added.model}
              onChange={(event) =>
                setAdded({ ...added, model: event.target.value })
              }
            />
            <button
              type="button"
              className="wb-btn"
              disabled={
                !added.name.trim() ||
                !added.endpoint.trim() ||
                !added.model.trim()
              }
              onClick={() =>
                void guarded("add", async () => {
                  await ai.addProvider(added.name, added.endpoint, added.model);
                  setAdded({ name: "", endpoint: "", model: "" });
                })
              }
            >
              Add
            </button>
          </div>
        </SettingRow>
      </Card>

      <Card>
        <SettingRow
          title="Switch everything off"
          note="Withdraws every consent, turns every provider and the master switch off, and deletes every stored key."
        >
          <button
            type="button"
            className="wb-btn"
            onClick={() => void guarded("withdraw", () => ai.withdrawAll())}
          >
            Withdraw all
          </button>
        </SettingRow>
      </Card>

      {consentFor && (
        <div className="wb-modal-backdrop">
          <div
            className="wb-dialog"
            role="dialog"
            aria-modal="true"
            aria-label={`Consent for ${consentFor.name}`}
          >
            <h2>Use {consentFor.name}?</h2>
            <p>
              {consentFor.endpoint.includes(".local-agent.invalid/")
                ? "Asten sends this request through your installed agent and its existing sign-in. Content may leave this computer. That provider controls model access, plan limits and extra-usage billing."
                : consentFor.local
                  ? "This model runs on this machine. Nothing leaves it."
                  : `Requests go from this computer straight to ${consentFor.host}, under your own key and that provider's terms. Asten sends nothing to Verne.`}
            </p>
            <h3>What is sent</h3>
            <ul>
              <li>
                Your question, Asten's fixed instructions, and the names of the
                tools it may use.
              </li>
              <li>
                What a tool returns when the model asks for one: the list of
                marks (the author's quoted wording left out unless you attach
                the mark), canon facts, the research note you ask about.
              </li>
              <li>
                A passage of your writing only when you pick one, and only that
                passage. Never the whole manuscript, never another project.
              </li>
            </ul>
            <h3>What it may do</h3>
            <ul>
              <li>
                Explain, search, summarise a note and propose facts for you to
                settle. It cannot write, rewrite or continue your prose, settle
                canon, apply a repair, change a setting or read a key.
              </li>
            </ul>
            <p className="muted small">
              You can read the exact request before any call, withdraw consent
              here at any time, and the master switch above turns every provider
              off.
            </p>
            <div className="dialog-actions">
              <button
                type="button"
                className="wb-btn"
                onClick={() => void samplePreview(consentFor)}
              >
                Show the exact request
              </button>
              <button
                type="button"
                className="wb-btn"
                onClick={() => setConsentFor(undefined)}
              >
                Cancel
              </button>
              <button
                type="button"
                className="wb-btn wb-btn--primary"
                onClick={() =>
                  void guarded("consent", async () => {
                    await ai.consent(consentFor.id, true);
                    setConsentFor(undefined);
                  })
                }
              >
                I consent for {consentFor.name}
              </button>
            </div>
          </div>
        </div>
      )}
      {preview && (
        <PreviewDialog
          preview={preview}
          title="The exact request"
          onClose={() => setPreview(undefined)}
        />
      )}
    </>
  );

  function savePrice(p: ProviderView, input?: string, output?: string) {
    const parse = (text: string | undefined, fallback: number) => {
      if (text === undefined) return fallback;
      const n = Number(text);
      return Number.isFinite(n) && n >= 0 ? n : fallback;
    };
    if (input === "" && output === undefined) return;
    if (output === "" && input === undefined) return;
    const next = {
      input_per_mtok: parse(input, p.price?.input_per_mtok ?? 0),
      output_per_mtok: parse(output, p.price?.output_per_mtok ?? 0),
    };
    if (
      p.price &&
      next.input_per_mtok === p.price.input_per_mtok &&
      next.output_per_mtok === p.price.output_per_mtok
    )
      return;
    void guarded("price", () =>
      ai.update({ prices: { [`${p.id}/${p.model}`]: next } }),
    );
  }
}
