import { useState } from "react";
import { useWorkbench } from "../context";
import { Icon } from "../icons";
import { getPrefs, setPrefs, type Prefs, type Theme } from "../prefs";
import { RIBBON_TABS } from "./model";
import { QAT_CHOICES } from "./Ribbon";
import { CONNECTORS } from "../settings/connectors";
import { aiAvailable } from "../ai/bridge";

export const OPTION_PAGES = [
  "General",
  "Display",
  "Proofing",
  "Save",
  "Language",
  "Accessibility",
  "Advanced",
  "Customize Ribbon",
  "Quick Access Toolbar",
  "Add-ins",
  "Trust Center",
] as const;

export type OptionPage = (typeof OPTION_PAGES)[number];

/** Options, laid out like Word's: pages on the left, OK and Cancel at the foot. Edits apply on OK. */
export function OptionsDialog({
  initial = "General",
  onClose,
}: {
  initial?: OptionPage;
  onClose: () => void;
}) {
  const api = useWorkbench();
  const [page, setPage] = useState<OptionPage>(initial);
  const [draft, setDraft] = useState<Prefs>(() => getPrefs());
  const set = (patch: Partial<Prefs>) => setDraft((d) => ({ ...d, ...patch }));

  const check = (label: string, key: keyof Prefs, note?: string) => (
    <label className="opt-check">
      <input
        type="checkbox"
        checked={Boolean(draft[key])}
        onChange={(e) => set({ [key]: e.target.checked } as Partial<Prefs>)}
      />
      <span>
        {label}
        {note && <small>{note}</small>}
      </span>
    </label>
  );
  const select = (
    label: string,
    key: keyof Prefs,
    options: [string, string][],
  ) => (
    <label className="opt-field">
      <span>{label}</span>
      <select
        value={String(draft[key])}
        onChange={(e) =>
          set({
            [key]:
              isNaN(Number(e.target.value)) ||
              key === "pageFontSize" ||
              key === "lineSpacing"
                ? e.target.value
                : Number(e.target.value),
          } as Partial<Prefs>)
        }
      >
        {options.map(([value, text]) => (
          <option key={value} value={value}>
            {text}
          </option>
        ))}
      </select>
    </label>
  );
  const fixed = (label: string, note: string) => (
    <div className="opt-fixed">
      <Icon name="lock" />
      <span>
        {label}
        <small>{note}</small>
      </span>
      <span className="wb-pill">fixed</span>
    </div>
  );

  return (
    <div
      className="wb-modal-backdrop"
      onMouseDown={(e) => e.target === e.currentTarget && onClose()}
    >
      <div
        className="options-dialog"
        role="dialog"
        aria-modal="true"
        aria-label="Asten Options"
      >
        <header>
          <strong>Asten Options</strong>
          <button
            type="button"
            className="wb-icon-btn"
            aria-label="Close"
            onClick={onClose}
          >
            <Icon name="x" />
          </button>
        </header>
        <div className="options-main">
          <nav aria-label="Option pages">
            {OPTION_PAGES.map((p) => (
              <button
                key={p}
                type="button"
                className={p === page ? "active" : ""}
                onClick={() => setPage(p)}
              >
                {p}
              </button>
            ))}
          </nav>
          <div className="options-page">
            <h2>{pageTitle(page)}</h2>
            {page === "General" && (
              <>
                <h3>User interface options</h3>
                {select("Office theme", "theme", [
                  ["system", "Use system setting"],
                  ["light", "Light"],
                  ["dark", "Dark"],
                  ["sepia", "Sepia"],
                ] satisfies [Theme, string][])}
                {select("Density", "density", [
                  ["comfortable", "Comfortable"],
                  ["compact", "Compact"],
                ])}
                {select("Interface text size", "uiFontSize", [
                  ["12", "12 px"],
                  ["13", "13 px"],
                  ["14", "14 px"],
                ])}
                <h3>Start up options</h3>
                {select("When Asten opens", "openOn", [
                  ["last", "Return to the last page"],
                  ["overview", "Show the project overview"],
                ])}
                {fixed(
                  "Personalize your copy",
                  "Asten has no account and no user name to set.",
                )}
              </>
            )}
            {page === "Display" && (
              <>
                <h3>Page display options</h3>
                {select("Page size", "pageSize", [
                  ["letter", "Letter"],
                  ["a4", "A4"],
                  ["a5", "A5"],
                  ["trade", "Trade 6 × 9 in"],
                ])}
                {select("Margins", "pageMargins", [
                  ["normal", "Normal"],
                  ["narrow", "Narrow"],
                  ["wide", "Wide"],
                ])}
                {select("Page font", "pageFont", [
                  ["literata", "Literata"],
                  ["fraunces", "Fraunces"],
                  ["mono", "IBM Plex Mono"],
                  ["system", "System"],
                ])}
                {select("Line spacing", "lineSpacing", [
                  ["1.5", "1.5"],
                  ["1.7", "1.7"],
                  ["2", "Double"],
                ])}
                <h3>Always show these marks on the page</h3>
                {check("Continuity marks under the words", "showMarksInline")}
                {check("Mark cards in the margin", "showMargin")}
              </>
            )}
            {page === "Proofing" && (
              <>
                <h3>When correcting spelling</h3>
                {check(
                  "Check spelling as you type",
                  "spellcheck",
                  "Your system's spell checker, in the page's language.",
                )}
                <h3>When weighing the story</h3>
                <p className="muted">
                  Strictness, narrator reliability and the story clock are
                  project settings.
                </p>
                <button
                  type="button"
                  className="wb-btn"
                  onClick={() => {
                    onClose();
                    api.openSettings("checks");
                  }}
                >
                  Continuity settings…
                </button>
                {fixed(
                  "Grammar and style suggestions",
                  "Craft hints come only from the optional Diction pack, off until you enable it.",
                )}
              </>
            )}
            {page === "Save" && (
              <>
                <h3>Save documents</h3>
                {fixed(
                  "Save as you write",
                  "Every pause commits the page to the project file, with a hash-chained history.",
                )}
                {check(
                  "Snapshot the open page when compiling",
                  "snapshotOnCompile",
                )}
                {select("Save a version automatically", "autoSnapshotMinutes", [
                  ["0", "Never"],
                  ["15", "Every 15 minutes"],
                  ["30", "Every 30 minutes"],
                ])}
                {fixed(
                  "Where the project lives",
                  api.sample
                    ? "This release: the page's memory (sample data)."
                    : "One SQLite file in the app-data folder.",
                )}
              </>
            )}
            {page === "Language" && (
              <>
                <h3>Editing language</h3>
                {select("Page language", "language", [
                  ["en-GB", "English (United Kingdom)"],
                  ["en-US", "English (United States)"],
                  ["fr-FR", "French"],
                  ["de-DE", "German"],
                  ["es-ES", "Spanish"],
                ])}
                <p className="muted">
                  Perception reads Latin script at Tier B today; other scripts
                  get name checks (Tier C). The interface is in English.
                </p>
              </>
            )}
            {page === "Accessibility" && (
              <>
                {check("Reduce motion", "reduceMotion")}
                {check("Increase contrast", "highContrast")}
                {fixed(
                  "States in words",
                  "Every mark says its severity in words; colour is a second channel.",
                )}
                {fixed(
                  "Keyboard",
                  "Every control is reachable by keyboard; see Keybindings in Settings.",
                )}
              </>
            )}
            {page === "Advanced" && (
              <>
                {check("Confirm before deleting a page", "confirmDelete")}
                {check("Assistant panel available", "agentEnabled")}
                <button
                  type="button"
                  className="wb-btn"
                  onClick={() => {
                    onClose();
                    api.openSettings("keybindings");
                  }}
                >
                  Keyboard shortcuts…
                </button>
              </>
            )}
            {page === "Customize Ribbon" && (
              <>
                <p className="muted">
                  Choose the tabs the ribbon shows. File is always there.
                </p>
                {RIBBON_TABS.map((tab) => (
                  <label key={tab.id} className="opt-check">
                    <input
                      type="checkbox"
                      checked={!draft.hiddenTabs.includes(tab.id)}
                      disabled={tab.id === "home"}
                      onChange={(e) =>
                        set({
                          hiddenTabs: e.target.checked
                            ? draft.hiddenTabs.filter((t) => t !== tab.id)
                            : [...draft.hiddenTabs, tab.id],
                        })
                      }
                    />
                    <span>
                      {tab.label}
                      <small>
                        {tab.groups.map((g) => g.label).join(" · ")}
                      </small>
                    </span>
                  </label>
                ))}
              </>
            )}
            {page === "Quick Access Toolbar" && (
              <div className="qat-editor">
                <div>
                  <h3>Choose commands</h3>
                  {QAT_CHOICES.filter((c) => !draft.qat.includes(c.id)).map(
                    (c) => (
                      <button
                        key={c.id}
                        type="button"
                        className="wb-row clickable"
                        onClick={() => set({ qat: [...draft.qat, c.id] })}
                      >
                        <Icon name={c.icon} />
                        <span className="wb-row-main">{c.label}</span>
                        <span className="muted">Add</span>
                      </button>
                    ),
                  )}
                </div>
                <div>
                  <h3>On the toolbar</h3>
                  {draft.qat.map((id) => {
                    const c = QAT_CHOICES.find((x) => x.id === id);
                    return c ? (
                      <button
                        key={id}
                        type="button"
                        className="wb-row clickable"
                        onClick={() =>
                          set({ qat: draft.qat.filter((x) => x !== id) })
                        }
                      >
                        <Icon name={c.icon} />
                        <span className="wb-row-main">{c.label}</span>
                        <span className="muted">Remove</span>
                      </button>
                    ) : null;
                  })}
                </div>
              </div>
            )}
            {page === "Add-ins" && (
              <>
                <table className="wb-table">
                  <thead>
                    <tr>
                      <th>Add-in or connector</th>
                      <th>Kind</th>
                      <th>Status</th>
                    </tr>
                  </thead>
                  <tbody>
                    {CONNECTORS.map((c) => (
                      <tr key={c.id}>
                        <td>{c.name}</td>
                        <td className="muted">{c.category}</td>
                        <td>
                          <span
                            className={`wb-pill wb-pill--${c.status === "Built" ? "ok" : "muted"}`}
                          >
                            {c.status}
                          </span>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                <button
                  type="button"
                  className="wb-btn"
                  onClick={() => {
                    onClose();
                    api.openSettings("connectors");
                  }}
                >
                  Manage connectors…
                </button>
              </>
            )}
            {page === "Trust Center" && (
              <>
                {fixed(
                  "No telemetry",
                  "Asten sends nothing about you or your work anywhere.",
                )}
                {fixed(
                  "No training",
                  "No manuscript trains a model, by Verne or anyone.",
                )}
                {fixed(
                  "Editor, not writer",
                  "Nothing in Asten writes, rewrites or continues your prose.",
                )}
                {fixed(
                  aiAvailable()
                    ? "Keys stay in the credential store"
                    : "Keys stay in memory",
                  aiAvailable()
                    ? "Provider keys live in your operating system's credential store, never in the page, and are shown only as the last four characters."
                    : "Provider keys are never stored on disk in this copy and never shown again.",
                )}
                {check(
                  "Allow AI providers (off by default)",
                  "aiEnabled",
                  "Local first. A cloud provider needs your key and your consent, per provider.",
                )}
                <button
                  type="button"
                  className="wb-btn"
                  onClick={() => {
                    onClose();
                    api.openSettings("providers");
                  }}
                >
                  Providers and consent…
                </button>
              </>
            )}
          </div>
        </div>
        <footer>
          <button
            type="button"
            className="wb-btn wb-btn--primary"
            onClick={() => {
              setPrefs(draft);
              onClose();
            }}
          >
            OK
          </button>
          <button type="button" className="wb-btn" onClick={onClose}>
            Cancel
          </button>
        </footer>
      </div>
    </div>
  );
}

function pageTitle(page: OptionPage) {
  return {
    General: "General options for working with Asten.",
    Display: "Change how the page is displayed on the screen.",
    Proofing: "Change how Asten checks spelling and weighs the story.",
    Save: "Customize how pages and versions are saved.",
    Language: "Set the page's language.",
    Accessibility: "Make Asten easier to use.",
    Advanced: "Advanced options for working with Asten.",
    "Customize Ribbon": "Customize the Ribbon.",
    "Quick Access Toolbar": "Customize the Quick Access Toolbar.",
    "Add-ins": "View and manage add-ins and connectors.",
    "Trust Center": "Privacy, AI and the rules that do not change.",
  }[page];
}
