import { useQuery } from "@tanstack/react-query";
import { Suspense, lazy, useEffect, useState } from "react";
import { Packs } from "../../packs/Packs";
import { PaletteMark } from "../../brand/Palette";
import { useWorkbench } from "../context";
import { Icon } from "../icons";
import {
  chordOf,
  conflicts,
  DEFAULT_BINDINGS,
  effectiveBindings,
  normalizeChord,
} from "../keys";
import {
  DEFAULT_PREFS,
  resetPrefs,
  setPrefs,
  usePrefs,
  type Prefs,
  type ProviderEntry,
  type Theme,
} from "../prefs";
import { forgetKey, keyHint, storeKey, useKeyVersion } from "../secrets";
import { aiAvailable } from "../ai/bridge";
import { ProvidersDesktop } from "../ai/ProvidersPanel";
import { FIXED_RULES } from "../agent/AgentPanel";
import { SKILLS } from "../agent/skills";
import { TOOLS } from "../agent/tools";
import { CONNECTORS, type Connector } from "./connectors";
import { hostedSession, hostedStartOver, isHosted } from "../../lib/storage";
import { RELEASE } from "../../version";
import { Library } from "./Library";
import { Card, Fixed, Head, SettingRow, Switch } from "./parts";

const ProjectSettings = lazy(() =>
  import("../../settings/Settings").then((m) => ({ default: m.Settings })),
);
const Guests = lazy(() =>
  import("../../guests/Guests").then((m) => ({ default: m.Guests })),
);

export const SECTIONS = [
  { id: "general", label: "General", icon: "settings", group: "Desk" },
  { id: "appearance", label: "Appearance", icon: "theme", group: "Desk" },
  { id: "editor", label: "Page and editor", icon: "page", group: "Desk" },
  { id: "keybindings", label: "Keybindings", icon: "keyboard", group: "Desk" },
  { id: "checks", label: "Continuity checks", icon: "checks", group: "Story" },
  { id: "packs", label: "Method packs", icon: "packs", group: "Story" },
  { id: "assistant", label: "Assistant", icon: "agent", group: "Assistant" },
  { id: "skills", label: "Skills", icon: "skill", group: "Assistant" },
  { id: "providers", label: "AI providers", icon: "key", group: "Assistant" },
  { id: "connectors", label: "Connectors", icon: "plug", group: "Connections" },
  {
    id: "guests",
    label: "Guests and pairing",
    icon: "people",
    group: "Connections",
  },
  { id: "storage", label: "Storage and versions", icon: "save", group: "Data" },
  {
    id: "privacy",
    label: "Privacy and security",
    icon: "shield",
    group: "Data",
  },
  { id: "about", label: "About", icon: "info", group: "Data" },
] as const;

export type SectionId = (typeof SECTIONS)[number]["id"];

export function SettingsView({
  section,
  onSection,
}: {
  section: SectionId;
  onSection: (s: SectionId) => void;
}) {
  let group = "";
  return (
    <div className="settings">
      <nav className="settings-nav" aria-label="Settings sections">
        {SECTIONS.map((s) => {
          const head = s.group !== group ? s.group : null;
          group = s.group;
          return (
            <div key={s.id}>
              {head && <div className="settings-group">{head}</div>}
              <button
                type="button"
                className={s.id === section ? "active" : ""}
                aria-current={s.id === section ? "page" : undefined}
                onClick={() => onSection(s.id)}
              >
                <Icon name={s.icon} /> {s.label}
              </button>
            </div>
          );
        })}
      </nav>
      <div className="settings-body">
        {section === "general" && <General />}
        {section === "appearance" && <Appearance />}
        {section === "editor" && <EditorSection />}
        {section === "keybindings" && <Keybindings />}
        {section === "checks" && <Checks />}
        {section === "packs" && <Packs />}
        {section === "assistant" && <Assistant />}
        {section === "skills" && <Skills />}
        {section === "providers" &&
          (aiAvailable() ? <ProvidersDesktop /> : <Providers />)}
        {section === "connectors" && <Connectors />}
        {section === "guests" && (
          <Suspense fallback={<p className="status">Opening…</p>}>
            <GuestsSection />
          </Suspense>
        )}
        {section === "storage" && <Storage />}
        {section === "privacy" && <Privacy />}
        {section === "about" && <About />}
      </div>
    </div>
  );
}

function Select<K extends keyof Prefs>({
  pref,
  options,
}: {
  pref: K;
  options: [string, string][];
}) {
  const prefs = usePrefs();
  return (
    <select
      className="wb-select"
      value={String(prefs[pref])}
      onChange={(e) =>
        setPrefs({
          [pref]:
            typeof DEFAULT_PREFS[pref] === "number"
              ? Number(e.target.value)
              : e.target.value,
        } as Partial<Prefs>)
      }
    >
      {options.map(([v, t]) => (
        <option key={v} value={v}>
          {t}
        </option>
      ))}
    </select>
  );
}

function General() {
  const prefs = usePrefs();
  return (
    <>
      <Head title="General">How the desk opens and behaves.</Head>
      <Card>
        <SettingRow
          title="When Asten opens"
          note="Return to the page you left, or start on the project overview."
        >
          <Select
            pref="openOn"
            options={[
              ["last", "Last page"],
              ["overview", "Overview"],
            ]}
          />
        </SettingRow>
        <SettingRow
          title="Page language"
          note="Spell checking and perception follow it."
        >
          <Select
            pref="language"
            options={[
              ["en-GB", "English (UK)"],
              ["en-US", "English (US)"],
              ["fr-FR", "French"],
              ["de-DE", "German"],
              ["es-ES", "Spanish"],
            ]}
          />
        </SettingRow>
        <SettingRow
          title="Check spelling as you type"
          note="Your system's spell checker."
        >
          <Switch
            label="Spelling"
            value={prefs.spellcheck}
            onChange={(v) => setPrefs({ spellcheck: v })}
          />
        </SettingRow>
        <SettingRow title="Confirm before deleting a page">
          <Switch
            label="Confirm delete"
            value={prefs.confirmDelete}
            onChange={(v) => setPrefs({ confirmDelete: v })}
          />
        </SettingRow>
        <Fixed
          title="Saving"
          note="Every pause commits the page; there is no Save to forget."
        />
      </Card>
      <Card title="Reset">
        <SettingRow
          title="Restore every preference to its default"
          note="Keybindings, appearance, the toolbar and assistant settings. Your projects are not touched."
        >
          <button type="button" className="wb-btn" onClick={resetPrefs}>
            Reset
          </button>
        </SettingRow>
      </Card>
    </>
  );
}

const ACCENTS = [
  "#b0351f",
  "#1f3f77",
  "#2e6b4e",
  "#b98419",
  "#6b4fa0",
  "#3d3d3d",
];

function Appearance() {
  const prefs = usePrefs();
  const themes: { id: Theme; label: string }[] = [
    { id: "light", label: "Light" },
    { id: "dark", label: "Dark" },
    { id: "sepia", label: "Sepia" },
    { id: "system", label: "System" },
  ];
  return (
    <>
      <Head title="Appearance">
        The shell follows a neutral scheme with one accent; the page keeps
        Asten's typography.
      </Head>
      <Card title="Theme">
        <div className="theme-cards">
          {themes.map((t) => (
            <button
              key={t.id}
              type="button"
              className={`theme-card theme-${t.id}${prefs.theme === t.id ? " active" : ""}`}
              aria-pressed={prefs.theme === t.id}
              onClick={() => setPrefs({ theme: t.id })}
            >
              <span className="theme-preview">
                <span />
                <span />
                <span />
              </span>
              {t.label}
            </button>
          ))}
        </div>
      </Card>
      <Card>
        <SettingRow
          title="Accent"
          note="One colour for the active icon, the primary button and links. Asten's red by default."
        >
          <div className="accent-row">
            {ACCENTS.map((c) => (
              <button
                key={c}
                type="button"
                className={`accent-swatch${prefs.accent === c ? " active" : ""}`}
                style={{ background: c }}
                aria-label={`Accent ${c}`}
                onClick={() => setPrefs({ accent: c })}
              />
            ))}
            <input
              type="color"
              value={prefs.accent}
              onChange={(e) => setPrefs({ accent: e.target.value })}
              aria-label="Custom accent"
            />
          </div>
        </SettingRow>
        <SettingRow title="Density" note="Row and control height.">
          <Select
            pref="density"
            options={[
              ["comfortable", "Comfortable"],
              ["compact", "Compact"],
            ]}
          />
        </SettingRow>
        <SettingRow title="Interface text size">
          <Select
            pref="uiFontSize"
            options={[
              ["12", "12 px"],
              ["13", "13 px"],
              ["14", "14 px"],
            ]}
          />
        </SettingRow>
        <SettingRow
          title="Page scheme"
          note="Publisher-style colour schemes for the page surface."
        >
          <Select
            pref="scheme"
            options={[
              ["papyrus", "Papyrus"],
              ["lapis", "Lapis"],
              ["malachite", "Malachite"],
              ["ink", "Plain ink"],
            ]}
          />
        </SettingRow>
        <SettingRow title="Reduce motion">
          <Switch
            label="Reduce motion"
            value={prefs.reduceMotion}
            onChange={(v) => setPrefs({ reduceMotion: v })}
          />
        </SettingRow>
        <SettingRow title="Increase contrast">
          <Switch
            label="Increase contrast"
            value={prefs.highContrast}
            onChange={(v) => setPrefs({ highContrast: v })}
          />
        </SettingRow>
      </Card>
    </>
  );
}

function EditorSection() {
  const prefs = usePrefs();
  return (
    <>
      <Head title="Page and editor">
        How the page looks while you write. The manuscript stores structure, not
        fonts.
      </Head>
      <Card>
        <SettingRow title="Page font">
          <Select
            pref="pageFont"
            options={[
              ["literata", "Literata"],
              ["fraunces", "Fraunces"],
              ["mono", "IBM Plex Mono"],
              ["system", "System"],
            ]}
          />
        </SettingRow>
        <SettingRow title="Size">
          <Select
            pref="pageFontSize"
            options={["15", "16", "17", "18", "19", "20", "22"].map((v) => [
              v,
              `${v} px`,
            ])}
          />
        </SettingRow>
        <SettingRow title="Line spacing">
          <Select
            pref="lineSpacing"
            options={[
              ["1.5", "1.5"],
              ["1.7", "1.7"],
              ["2", "Double"],
            ]}
          />
        </SettingRow>
        <SettingRow title="Page size">
          <Select
            pref="pageSize"
            options={[
              ["letter", "Letter"],
              ["a4", "A4"],
              ["a5", "A5"],
              ["trade", "Trade 6 × 9 in"],
            ]}
          />
        </SettingRow>
        <SettingRow title="Margins">
          <Select
            pref="pageMargins"
            options={[
              ["normal", "Normal"],
              ["narrow", "Narrow"],
              ["wide", "Wide"],
            ]}
          />
        </SettingRow>
        <SettingRow
          title="Marks under the words"
          note="Continuity marks drawn in red on the page."
        >
          <Switch
            label="Marks inline"
            value={prefs.showMarksInline}
            onChange={(v) => setPrefs({ showMarksInline: v })}
          />
        </SettingRow>
        <SettingRow title="Mark cards in the margin">
          <Switch
            label="Margin cards"
            value={prefs.showMargin}
            onChange={(v) => setPrefs({ showMargin: v })}
          />
        </SettingRow>
        <Fixed
          title="Black ink and red ink"
          note="Your words are black. Asten only ever writes in red: marks, codes and notes, never prose."
        />
      </Card>
    </>
  );
}

function Keybindings() {
  const prefs = usePrefs();
  const [recording, setRecording] = useState<string>();
  const [filter, setFilter] = useState("");
  const bindings = effectiveBindings(prefs.keybindings);
  const clash = conflicts(bindings);
  useEffect(() => {
    if (!recording) return;
    const onKey = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();
      if (event.key === "Escape") return setRecording(undefined);
      const chord = chordOf(event);
      if (!chord) return;
      setPrefs((p) => ({
        keybindings: { ...p.keybindings, [recording]: chord },
      }));
      setRecording(undefined);
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [recording]);
  return (
    <>
      <Head title="Keybindings">
        Press Record, then the keys. Conflicts are flagged; Escape cancels.
      </Head>
      <input
        className="wb-input settings-filter"
        placeholder="Search keybindings"
        value={filter}
        onChange={(e) => setFilter(e.target.value)}
        aria-label="Search keybindings"
      />
      <section className="wb-card">
        <table className="wb-table">
          <thead>
            <tr>
              <th>Command</th>
              <th>Keybinding</th>
              <th>When</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {bindings
              .filter(
                (b) =>
                  !filter ||
                  `${b.title} ${b.keys}`
                    .toLowerCase()
                    .includes(filter.toLowerCase()),
              )
              .map((b) => {
                const conflict = clash.get(normalizeChord(b.keys));
                const changed = prefs.keybindings[b.command] !== undefined;
                return (
                  <tr key={b.command} className={conflict ? "conflict" : ""}>
                    <td>
                      {b.title}
                      <div className="muted mono small">{b.command}</div>
                    </td>
                    <td>
                      {recording === b.command ? (
                        <span className="recording">Press keys…</span>
                      ) : (
                        <kbd className="chord">{b.keys || "—"}</kbd>
                      )}
                      {conflict && (
                        <div className="wb-error small">
                          Also bound to{" "}
                          {conflict.filter((c) => c !== b.command).join(", ")}
                        </div>
                      )}
                    </td>
                    <td className="muted small">
                      {b.when === "notEditing" ? "page not focused" : "always"}
                    </td>
                    <td className="row-actions">
                      <button
                        type="button"
                        className="wb-btn wb-btn--small"
                        onClick={() => setRecording(b.command)}
                      >
                        Record
                      </button>
                      {changed && (
                        <button
                          type="button"
                          className="wb-btn wb-btn--small wb-btn--ghost"
                          onClick={() =>
                            setPrefs((p) => {
                              const k = { ...p.keybindings };
                              delete k[b.command];
                              return { keybindings: k };
                            })
                          }
                        >
                          Reset
                        </button>
                      )}
                    </td>
                  </tr>
                );
              })}
          </tbody>
        </table>
      </section>
      <p className="muted small">
        While the caret is in the page, Ctrl+B, Ctrl+I, Ctrl+U, Ctrl+Z and
        Ctrl+Y belong to the editor, as in Word. {DEFAULT_BINDINGS.length}{" "}
        commands.
      </p>
    </>
  );
}

function Checks() {
  const api = useWorkbench();
  return (
    <>
      <Head title="Continuity checks">
        How strictly the engine weighs this project, the story clock, and the
        narrator.
      </Head>
      {api.projectId ? (
        <Suspense fallback={<p className="status">Opening…</p>}>
          <div className="embedded-view">
            <ProjectSettings
              projectId={api.projectId}
              snapshot={api.diagnostics}
            />
          </div>
        </Suspense>
      ) : (
        <p className="muted">Open a project to change its checks.</p>
      )}
    </>
  );
}

function Assistant() {
  const prefs = usePrefs();
  const toggleTool = (name: string, on: boolean) =>
    setPrefs((p) => ({
      disabledTools: on
        ? p.disabledTools.filter((t) => t !== name)
        : [...p.disabledTools, name],
    }));
  return (
    <>
      <Head title="Assistant">
        An editor's assistant in the right panel: it checks, explains, searches
        and proposes. It never writes.
      </Head>
      <Card>
        <SettingRow
          title="Show the assistant"
          note="Off removes it from the right panel. Everything else keeps working."
        >
          <Switch
            label="Assistant"
            value={prefs.agentEnabled}
            onChange={(v) => setPrefs({ agentEnabled: v })}
          />
        </SettingRow>
        <SettingRow
          title="Access for new threads"
          note="Read only can look; Propose can also prepare cards you press. The assistant never raises its own access."
        >
          <Select
            pref="agentAccess"
            options={[
              ["read", "Read only"],
              ["propose", "Propose"],
            ]}
          />
        </SettingRow>
        <SettingRow
          title="Model"
          note={
            aiAvailable()
              ? "Pick who answers in the assistant panel: Asten's own tools (no model), or a provider you switched on under AI providers. Providers are off until you allow them."
              : "No model is called in this copy: the assistant runs Asten's own tools and writes its answers from what they return. The desktop app can use a provider with your own key."
          }
        >
          <span className="wb-pill">
            {aiAvailable() && prefs.agentModel !== "rules"
              ? prefs.agentModel
              : "Asten rules"}
          </span>
        </SettingRow>
      </Card>
      <Card title="Rules that no setting changes">
        {FIXED_RULES.map((rule) => (
          <Fixed key={rule} title={rule} note="Fixed." />
        ))}
      </Card>
      <section className="wb-card">
        <h2>Tools</h2>
        <table className="wb-table">
          <thead>
            <tr>
              <th>Tool</th>
              <th>Needs</th>
              <th>What it does</th>
              <th>On</th>
            </tr>
          </thead>
          <tbody>
            {TOOLS.map((tool) => (
              <tr
                key={tool.name}
                className={tool.needs === "never" ? "locked" : ""}
              >
                <td>
                  <code>{tool.name}</code>
                </td>
                <td className="muted">
                  {tool.needs === "never"
                    ? "closed"
                    : tool.needs === "read"
                      ? "Read only"
                      : "Propose"}
                </td>
                <td>
                  {tool.needs === "never"
                    ? tool.closedBecause
                    : tool.description}
                </td>
                <td>
                  {tool.needs === "never" ? (
                    <span className="wb-pill">
                      <Icon name="lock" size={12} /> never
                    </span>
                  ) : (
                    <Switch
                      label={tool.title}
                      value={!prefs.disabledTools.includes(tool.name)}
                      onChange={(v) => toggleTool(tool.name, v)}
                    />
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>
    </>
  );
}

function Skills() {
  const prefs = usePrefs();
  const [open, setOpen] = useState<string>();
  return (
    <>
      <Head title="Skills">
        Named instructions the assistant follows. A skill can only narrow what
        the assistant does; it cannot open a closed tool.
      </Head>
      <section className="wb-card wb-rows">
        {SKILLS.map((skill) => {
          const state = prefs.skills[skill.id];
          const enabled = state?.enabled !== false;
          return (
            <div key={skill.id} className="skill-row">
              <div className="setting-row">
                <div className="setting-text">
                  <strong>{skill.name}</strong>
                  <small>{skill.summary}</small>
                  <small className="mono">{skill.tools.join(" · ")}</small>
                </div>
                <div className="setting-control">
                  <button
                    type="button"
                    className="wb-btn wb-btn--small"
                    onClick={() =>
                      setOpen(open === skill.id ? undefined : skill.id)
                    }
                  >
                    {open === skill.id ? "Close" : "Inspect"}
                  </button>
                  <Switch
                    label={skill.name}
                    value={enabled}
                    onChange={(v) =>
                      setPrefs((p) => ({
                        skills: {
                          ...p.skills,
                          [skill.id]: { ...p.skills[skill.id], enabled: v },
                        },
                      }))
                    }
                  />
                </div>
              </div>
              {open === skill.id && (
                <div className="skill-edit">
                  <label className="field">
                    <span>Instructions</span>
                    <textarea
                      rows={4}
                      value={state?.body ?? skill.body}
                      onChange={(e) =>
                        setPrefs((p) => ({
                          skills: {
                            ...p.skills,
                            [skill.id]: { enabled, body: e.target.value },
                          },
                        }))
                      }
                    />
                  </label>
                  <p className="muted small">
                    Starting prompt: “{skill.prompt}”. The fixed rules apply on
                    top of anything written here.
                  </p>
                  {state?.body !== undefined && (
                    <button
                      type="button"
                      className="wb-btn wb-btn--small wb-btn--ghost"
                      onClick={() =>
                        setPrefs((p) => ({
                          skills: { ...p.skills, [skill.id]: { enabled } },
                        }))
                      }
                    >
                      Restore the default
                    </button>
                  )}
                </div>
              )}
            </div>
          );
        })}
        <div className="setting-row locked">
          <div className="setting-text">
            <strong>Write or rewrite prose</strong>
            <small>Not a skill Asten has, or will have.</small>
          </div>
          <div className="setting-control">
            <span className="wb-pill">
              <Icon name="lock" size={12} /> never
            </span>
          </div>
        </div>
      </section>
    </>
  );
}

function Providers() {
  const prefs = usePrefs();
  useKeyVersion();
  const [consentFor, setConsentFor] = useState<ProviderEntry>();
  const [keyDraft, setKeyDraft] = useState<Record<string, string>>({});
  const update = (id: string, patch: Partial<ProviderEntry>) =>
    setPrefs((p) => ({
      providers: p.providers.map((x) => (x.id === id ? { ...x, ...patch } : x)),
    }));
  return (
    <>
      <Head title="AI providers">
        Local models first; a cloud provider only with your own key and your
        consent; and a switch that turns all of it off.
      </Head>
      <Card>
        <SettingRow
          title="Allow AI providers"
          note="Off by default. Off means no provider is ever contacted, whatever is set below."
        >
          <Switch
            label="Allow AI providers"
            value={prefs.aiEnabled}
            onChange={(v) => setPrefs({ aiEnabled: v })}
          />
        </SettingRow>
        <SettingRow
          title="Local first"
          note="Try a model on this machine before any cloud provider in the ladder."
        >
          <Switch
            label="Local first"
            value={prefs.aiLocalFirst}
            disabled={!prefs.aiEnabled}
            onChange={(v) => setPrefs({ aiLocalFirst: v })}
          />
        </SettingRow>
        <Fixed
          title="What a provider may be asked"
          note="Explain a mark, summarise a source note, search the canon. Never to write, rewrite or continue prose."
        />
        <Fixed
          title="This release"
          note={
            isHosted()
              ? "Providers are configured here but not called: the assistant answers from Asten's own tools. These settings are kept with your sandbox; keys never leave this tab's memory."
              : "Providers are configured here but not called: the assistant answers from Asten's own tools. No request goes to any provider."
          }
        />
      </Card>
      <section className={`wb-card${prefs.aiEnabled ? "" : " is-off"}`}>
        <h2>Ladder</h2>
        <p className="muted small">
          Tried in this order. Keys stay in this window's memory: never saved,
          never shown again, gone when you close it.
        </p>
        <div className="wb-rows">
          {prefs.providers.map((p, index) => {
            const hint = keyHint(p.id);
            return (
              <div key={p.id} className="provider-row">
                <span className="ladder-n">{index + 1}</span>
                <div className="setting-text">
                  <strong>
                    {p.name}{" "}
                    <span
                      className={`wb-pill wb-pill--${p.kind === "local" ? "ok" : "muted"}`}
                    >
                      {p.kind}
                    </span>
                    {p.consent && (
                      <span className="wb-pill wb-pill--ok">consent given</span>
                    )}
                  </strong>
                  <small className="mono">
                    {p.model} · {p.endpoint}
                  </small>
                  {p.kind === "cloud" && (
                    <div className="key-row">
                      {hint.stored ? (
                        <>
                          <span className="wb-pill">
                            key stored · ••••{hint.last4}
                          </span>
                          <button
                            type="button"
                            className="wb-btn wb-btn--small wb-btn--ghost"
                            onClick={() => forgetKey(p.id)}
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
                            placeholder="Paste your key"
                            value={keyDraft[p.id] ?? ""}
                            disabled={!prefs.aiEnabled}
                            onChange={(e) =>
                              setKeyDraft({
                                ...keyDraft,
                                [p.id]: e.target.value,
                              })
                            }
                            aria-label={`${p.name} key`}
                          />
                          <button
                            type="button"
                            className="wb-btn wb-btn--small"
                            disabled={!keyDraft[p.id]?.trim()}
                            onClick={() => {
                              storeKey(p.id, keyDraft[p.id] ?? "");
                              setKeyDraft({ ...keyDraft, [p.id]: "" });
                            }}
                          >
                            Store in memory
                          </button>
                        </>
                      )}
                    </div>
                  )}
                </div>
                <div className="setting-control">
                  <button
                    type="button"
                    className="wb-icon-btn"
                    aria-label="Move up"
                    disabled={index === 0}
                    onClick={() =>
                      setPrefs((s) => {
                        const list = [...s.providers];
                        [list[index - 1], list[index]] = [
                          list[index],
                          list[index - 1],
                        ];
                        return { providers: list };
                      })
                    }
                  >
                    <Icon name="chevronUp" />
                  </button>
                  <Switch
                    label={`Use ${p.name}`}
                    value={p.enabled}
                    disabled={!prefs.aiEnabled}
                    onChange={(v) => {
                      if (v && !p.consent) setConsentFor(p);
                      else update(p.id, { enabled: v });
                    }}
                  />
                </div>
              </div>
            );
          })}
        </div>
      </section>
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
              {consentFor.kind === "local"
                ? "This model runs on this machine. Nothing leaves it."
                : `Requests would go to ${consentFor.endpoint}, under your own key and that provider's terms.`}
            </p>
            <h3>What is sent</h3>
            <ul>
              <li>
                The mark, fact or source note you ask about, and the question
                you typed.
              </li>
              <li>
                Never a whole manuscript, never your other projects, never your
                key to anyone but this provider.
              </li>
            </ul>
            <h3>What it may do</h3>
            <ul>
              <li>
                Explain, summarise and search. It cannot write, rewrite or
                continue your prose, and it cannot settle canon.
              </li>
            </ul>
            <p className="muted small">
              In this release providers are not called. Consent is recorded per
              provider and can be withdrawn here at any time.
            </p>
            <div className="dialog-actions">
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
                onClick={() => {
                  update(consentFor.id, { enabled: true, consent: true });
                  setConsentFor(undefined);
                }}
              >
                I consent for {consentFor.name}
              </button>
            </div>
          </div>
        </div>
      )}
      <Card>
        <SettingRow
          title="Withdraw all consent"
          note="Turns every provider off and forgets every key held in memory."
        >
          <button
            type="button"
            className="wb-btn"
            onClick={() => {
              prefs.providers.forEach((p) => forgetKey(p.id));
              setPrefs((s) => ({
                providers: s.providers.map((p) => ({
                  ...p,
                  enabled: false,
                  consent: false,
                })),
              }));
            }}
          >
            Withdraw
          </button>
        </SettingRow>
      </Card>
    </>
  );
}

function Connectors() {
  const [open, setOpen] = useState<Connector>();
  const [filter, setFilter] = useState<"All" | "Built" | "Planned">("All");
  const shown = CONNECTORS.filter(
    (c) => filter === "All" || c.status === filter,
  );
  return (
    <>
      <Head title="Connectors">
        What Asten reaches beyond its window. “Built” ships today; “Planned” is
        a design, not a feature.
      </Head>
      <div className="seg">
        {(["All", "Built", "Planned"] as const).map((f) => (
          <button
            key={f}
            type="button"
            className={filter === f ? "active" : ""}
            onClick={() => setFilter(f)}
          >
            {f}
          </button>
        ))}
      </div>
      <div className="connector-grid">
        {shown.map((c) => (
          <button
            key={c.id}
            type="button"
            className="connector-card"
            onClick={() => setOpen(c)}
          >
            <div className="connector-top">
              <span className="connector-icon">
                <Icon
                  name={
                    c.category === "Add-in"
                      ? "plug"
                      : c.category === "Storage"
                        ? "folder"
                        : c.category === "Versions"
                          ? "branch"
                          : c.category === "Research"
                            ? "source"
                            : "file"
                  }
                />
              </span>
              <span
                className={`wb-pill wb-pill--${c.status === "Built" ? "ok" : "muted"}`}
              >
                {c.status}
              </span>
            </div>
            <strong>{c.name}</strong>
            <small className="muted">{c.category}</small>
            <p>{c.summary}</p>
          </button>
        ))}
      </div>
      {open && (
        <div
          className="wb-modal-backdrop"
          onMouseDown={(e) =>
            e.target === e.currentTarget && setOpen(undefined)
          }
        >
          <div
            className="wb-dialog"
            role="dialog"
            aria-modal="true"
            aria-label={open.name}
          >
            <h2>
              {open.name}{" "}
              <span
                className={`wb-pill wb-pill--${open.status === "Built" ? "ok" : "muted"}`}
              >
                {open.status}
              </span>
            </h2>
            <p>{open.summary}</p>
            <div className="wb-rows">
              <div className="wb-row">
                <span className="wb-row-label">Authentication</span>
                <span className="wb-row-value">{open.auth}</span>
              </div>
              <div className="wb-row">
                <span className="wb-row-label">Hosts</span>
                <span className="wb-row-value mono">
                  {open.hosts.length
                    ? open.hosts.join(", ")
                    : "none: files only"}
                </span>
              </div>
              <div className="wb-row">
                <span className="wb-row-label">Kind</span>
                <span className="wb-row-value">{open.category}</span>
              </div>
            </div>
            <p className="muted">{open.detail}</p>
            <div className="dialog-actions">
              <button
                type="button"
                className="wb-btn"
                onClick={() => setOpen(undefined)}
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}

function GuestsSection() {
  const api = useWorkbench();
  return (
    <>
      <Head title="Guests and pairing">
        Word, Google Docs and other browsers pair with a code and can be revoked
        here.
      </Head>
      {api.sample ? (
        <Card>
          <Fixed
            title="Pairing"
            note="Pairing needs the desktop app or a local API. This release runs on sample data in the browser, so there is nothing to pair with."
          />
        </Card>
      ) : (
        <div className="embedded-view">
          <Guests />
        </div>
      )}
    </>
  );
}

function Storage() {
  const api = useWorkbench();
  const prefs = usePrefs();
  const desk = useQuery({
    queryKey: ["desk-info"],
    queryFn: () => api.client.deskInfo(),
    retry: false,
  });
  return (
    <>
      <Head title="Storage and versions">
        One file per desk, a hash-chained history per page, and versions you
        name.
      </Head>
      <Card>
        <SettingRow
          title="Database"
          note={
            desk.data?.database_path ??
            (api.sample
              ? "This page's memory: the sample project resets when you reload."
              : "Reported by the API")
          }
        >
          <span className="wb-pill">
            {desk.data?.embedded ? "embedded" : api.sample ? "sample" : "API"}
          </span>
        </SettingRow>
        {desk.data?.backups_path && (
          <SettingRow title="Backups" note={desk.data.backups_path} />
        )}
        {desk.data?.logs_path && (
          <SettingRow title="Logs" note={desk.data.logs_path} />
        )}
        <SettingRow title="Snapshot the open page when compiling">
          <Switch
            label="Snapshot on compile"
            value={prefs.snapshotOnCompile}
            onChange={(v) => setPrefs({ snapshotOnCompile: v })}
          />
        </SettingRow>
        <SettingRow title="Save a version automatically">
          <Select
            pref="autoSnapshotMinutes"
            options={[
              ["0", "Never"],
              ["15", "Every 15 minutes"],
              ["30", "Every 30 minutes"],
            ]}
          />
        </SettingRow>
        <Fixed
          title="No sync service"
          note="Asten keeps no copy anywhere else (D8). Put the file where you like; compile any time."
        />
      </Card>
      <Library />
    </>
  );
}

function Privacy() {
  return (
    <>
      <Head title="Privacy and security">
        The rules that hold whatever else you set.
      </Head>
      <Card>
        <Fixed
          title="No telemetry, no analytics"
          note="Nothing about you or your work is sent anywhere."
        />
        <Fixed
          title="No account"
          note="Nothing to sign in to; pairing codes for guests only."
        />
        <Fixed
          title="No training"
          note="No manuscript trains a model, by Verne or anyone Verne works with."
        />
        <Fixed
          title="Provider keys"
          note={
            aiAvailable()
              ? "Kept in your operating system's credential store (or an encrypted file you unlock with a passphrase), by the desktop app itself. The page never holds a key after you paste it; it shows the last four characters and nothing more."
              : "Held in this page's memory only, shown as the last four characters, never written to disk."
          }
        />
        <Fixed
          title="What leaves this machine"
          note={
            aiAvailable()
              ? "Nothing, unless you switch on an AI provider and consent to it. Then each request goes straight from this computer to that provider, and you can read it first. Update checks happen only when you press the button."
              : "Nothing. This copy calls no provider."
          }
        />
        <Fixed
          title="Guests cannot edit"
          note="Word and Docs guests read marks and file a selection as a scene; they never edit or administer (D14)."
        />
        <Fixed
          title="Marks explain themselves"
          note="Every mark names its rule, its sentence and the settled fact it disagrees with."
        />
      </Card>
    </>
  );
}

function About() {
  const api = useWorkbench();
  return (
    <>
      <div className="about-head">
        <PaletteMark />
        <div>
          <h1>Asten</h1>
          <p className="muted">
            Pre-alpha release {RELEASE} · a writing desk with a memory · Verne
            Ltd.
          </p>
        </div>
      </div>
      {isHosted() ? (
        <HostedAbout />
      ) : api.sample ? (
        <Card title="This release">
          <p className="about-text">
            It runs on sample data with no backend: a short sample novel written
            for it, kept in this page's memory and reset when you reload. It
            makes no network requests and keeps nothing in your browser.
          </p>
          <div className="about-split">
            <div>
              <h3>Real</h3>
              <ul>
                <li>
                  The interface: ribbon, panels, Problems, console, Source
                  Control, knowledge view and graph, settings.
                </li>
                <li>
                  The editor, saving, comments, snapshots, drafts, facts, the
                  cast, the world board and the binder, against an in-page store
                  with the API's shapes.
                </li>
                <li>
                  The assistant's rules: the closed tool table, refused calls,
                  access levels and cards you press.
                </li>
              </ul>
            </div>
            <div>
              <h3>Prepared</h3>
              <ul>
                <li>
                  The continuity marks: written by hand to show what the engine
                  reports; they disappear when you change the words they quote
                  or apply a repair.
                </li>
                <li>
                  The assistant's words: composed from its tools' results by
                  rules; no model is called.
                </li>
                <li>History hashes: placeholders, not the engine's chain.</li>
                <li>
                  DOCX, EPUB and FDX: written by the engine in the desktop app.
                </li>
              </ul>
            </div>
          </div>
        </Card>
      ) : (
        <Card title="This build">
          <p className="about-text">
            Connected to the Asten API on this machine: marks, history and
            compile come from the engine.{" "}
            {aiAvailable()
              ? "The assistant runs Asten's own tools unless you switch on a model provider with your own key (Settings, AI providers)."
              : "The assistant runs Asten's own tools; no model is called."}
          </p>
        </Card>
      )}
      <Card title="Made by">
        <p className="about-text">
          Verne Ltd. Your words in black. Our marks in red.
        </p>
      </Card>
    </>
  );
}

/** About, on the hosted pre-alpha: what runs where, and this visit's limits. */
function HostedAbout() {
  const session = useQuery({
    queryKey: ["hosted-session"],
    queryFn: hostedSession,
  });
  const [confirming, setConfirming] = useState(false);
  const data = session.data;
  const expires = data ? new Date(data.expires_at) : undefined;
  return (
    <>
      <Card title="This release">
        <p className="about-text">
          This is the real workbench with the real engine, running on Verne's
          server in a sandbox of your own. Other visitors cannot see it. It
          started on a short sample novel written for Asten; change anything.
          Everything here is deleted{" "}
          {expires
            ? `on ${expires.toLocaleString(undefined, {
                dateStyle: "medium",
                timeStyle: "short",
              })}`
            : "24 hours after your first visit"}
          . For your own manuscripts, use the desktop app: it keeps everything
          on your machine.
        </p>
        <div className="about-split">
          <div>
            <h3>The Asten API (the product server)</h3>
            <ul>
              {(data?.served_by.api ?? []).map((line) => (
                <li key={line}>{line}</li>
              ))}
            </ul>
          </div>
          <div>
            <h3>The pre-alpha service</h3>
            <ul>
              {(data?.served_by.service ?? []).map((line) => (
                <li key={line}>{line}</li>
              ))}
            </ul>
            <h3>This tab's memory only</h3>
            <ul>
              {(data?.served_by.browser_memory ?? []).map((line) => (
                <li key={line}>{line}</li>
              ))}
            </ul>
          </div>
        </div>
        <p className="about-text muted">
          The assistant answers from Asten's own tools; no model is called and
          it never writes prose. Word and Docs guests pair with the desktop app,
          not with this sandbox.
          {data
            ? ` Sandbox size: ${Math.ceil(data.sandbox_bytes / 1024)} KB of ${Math.round(
                data.limits.max_sandbox_bytes / 1024 / 1024,
              )} MB.`
            : ""}
        </p>
      </Card>
      <Card title="Start over">
        <p className="about-text">
          Delete this sandbox and its settings now and begin again on the sample
          novel.
        </p>
        {confirming ? (
          <div className="about-actions">
            <button
              type="button"
              className="wb-btn wb-btn--danger"
              onClick={() => void hostedStartOver()}
            >
              Delete and start over
            </button>
            <button
              type="button"
              className="wb-btn"
              onClick={() => setConfirming(false)}
            >
              Keep it
            </button>
          </div>
        ) : (
          <button
            type="button"
            className="wb-btn"
            onClick={() => setConfirming(true)}
          >
            Start over…
          </button>
        )}
      </Card>
    </>
  );
}

export type { Theme };
