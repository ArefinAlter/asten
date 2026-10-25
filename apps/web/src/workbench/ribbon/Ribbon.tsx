import { useEffect, useReducer, useState } from "react";
import { useWorkbench } from "../context";
import { Icon } from "../icons";
import { getPrefs, setPrefs, usePrefs, type Prefs } from "../prefs";
import { ACTIVE_STATE, resolveCommand } from "./commands";
import { RIBBON_TABS, type RibbonControl } from "./model";

/** Whether an app-level toggle is on, from layout and preferences. */
export function useAppToggle(): (id: string) => boolean {
  const api = useWorkbench();
  const prefs = usePrefs();
  return (id: string) => {
    const { layout } = api;
    switch (id) {
      case "view.mode.read":
        return layout.pageMode === "read";
      case "view.mode.print":
        return layout.pageMode === "print";
      case "view.mode.draft":
        return layout.pageMode === "draft";
      case "view.mode.focus":
        return layout.pageMode === "focus";
      case "view.ruler":
        return layout.ruler;
      case "view.navigationPane":
        return layout.navigationPane;
      case "view.split":
        return layout.split;
      case "view.secondary":
        return layout.secondary;
      case "panel.toggle":
        return layout.panel;
      default:
        if (id.startsWith("app.scheme.")) return prefs.scheme === id.slice(11);
        if (id.startsWith("app.master.")) return prefs.master === id.slice(11);
        return false;
    }
  };
}

export function Ribbon({
  trackChanges,
  blame,
  onFile,
}: {
  trackChanges: boolean;
  blame: boolean;
  onFile: () => void;
}) {
  const api = useWorkbench();
  const prefs = usePrefs();
  const appToggle = useAppToggle();
  const tabs = RIBBON_TABS.filter((tab) => !prefs.hiddenTabs.includes(tab.id));
  const [tabId, setTabId] = useState("home");
  const [peek, setPeek] = useState(false);
  const tab = tabs.find((t) => t.id === tabId) ?? tabs[0];
  const collapsed = api.layout.ribbonCollapsed;
  const editor = api.editor;
  // Re-render on selection changes so toggles reflect the caret.
  const [, bump] = useReducer((n: number) => n + 1, 0);
  useEffect(() => {
    if (!editor) return;
    editor.on("transaction", bump);
    return () => {
      editor.off("transaction", bump);
    };
  }, [editor]);

  const pressed = (control: RibbonControl): boolean => {
    if (control.id === "app.trackChanges") return trackChanges;
    if (control.id === "app.blame") return blame;
    const check = ACTIVE_STATE[control.id];
    if (check && editor) {
      try {
        return check.name
          ? editor.isActive(check.name, check.attrs)
          : editor.isActive(check.attrs ?? {});
      } catch {
        return false;
      }
    }
    return appToggle(control.id);
  };

  const disabledReason = (control: RibbonControl): string | undefined => {
    const resolved = resolveCommand(control.id);
    if (resolved.kind === "unavailable") return resolved.reason;
    if (resolved.kind === "editor" && !editor)
      return "Open a page in the binder to use this.";
    if (
      [
        "app.comment",
        "app.link",
        "app.crossReference",
        "app.insertCharacter",
        "app.insertPlace",
        "app.citation",
        "app.paste",
        "app.cut",
        "app.copy",
        "app.find",
        "app.trackChanges",
        "app.blame",
      ].includes(control.id) &&
      !editor
    )
      return "Open a page in the binder to use this.";
    if (
      !api.projectId &&
      /^app\.(new|comment|openCanon|openCast|openTimeline|proposals|sources|runChecks|compare)/.test(
        control.id,
      )
    )
      return "Open a project first.";
    return undefined;
  };

  return (
    <div
      className={`ribbon${collapsed ? " ribbon--collapsed" : ""}${collapsed && peek ? " ribbon--peek" : ""}`}
    >
      <div className="ribbon-tabs" role="tablist" aria-label="Ribbon">
        <button
          type="button"
          className="ribbon-file"
          onClick={onFile}
          title="File (Alt+F)"
        >
          File
        </button>
        {tabs.map((t) => (
          <button
            key={t.id}
            type="button"
            role="tab"
            aria-selected={t.id === tab.id}
            className={t.id === tab.id && (!collapsed || peek) ? "active" : ""}
            onClick={() => {
              setTabId(t.id);
              if (collapsed)
                setPeek((open) => (t.id === tab.id ? !open : true));
            }}
            onDoubleClick={() => {
              api.dispatch({ type: "toggleRibbon" });
              setPeek(false);
            }}
          >
            {t.label}
          </button>
        ))}
        <span className="ribbon-tabs-spacer" />
        <button
          type="button"
          className="ribbon-collapse"
          title={
            collapsed
              ? "Pin the ribbon (Ctrl+F1)"
              : "Collapse the ribbon (Ctrl+F1)"
          }
          aria-label={collapsed ? "Pin the ribbon" : "Collapse the ribbon"}
          onClick={() => {
            api.dispatch({ type: "toggleRibbon" });
            setPeek(false);
          }}
        >
          <Icon name={collapsed ? "chevronDown" : "chevronUp"} />
        </button>
      </div>
      {(!collapsed || peek) && (
        <div
          className="ribbon-body"
          role="toolbar"
          aria-label={`${tab.label} commands`}
          onClick={() => collapsed && setPeek(false)}
        >
          {tab.groups.map((group) => {
            const big = group.controls.filter((c) => c.size === "big");
            const small = group.controls.filter(
              (c) => c.size !== "big" && !c.select,
            );
            const selects = group.controls.filter((c) => c.select);
            return (
              <section
                key={group.label}
                className="ribbon-group"
                aria-label={group.label}
              >
                <div className="ribbon-group-body">
                  {selects.length > 0 && (
                    <div className="ribbon-selects">
                      {selects.map((control) => (
                        <label
                          key={control.label}
                          className="ribbon-select"
                          title={control.hint ?? control.label}
                        >
                          <span>{control.label}</span>
                          <select
                            value={String(
                              prefs[control.select!.pref as keyof Prefs] ?? "",
                            )}
                            onChange={(event) =>
                              setPrefs({
                                [control.select!.pref]: event.target.value,
                              } as Partial<Prefs>)
                            }
                          >
                            {control.select!.options.map((option) => (
                              <option key={option.value} value={option.value}>
                                {option.label}
                              </option>
                            ))}
                          </select>
                        </label>
                      ))}
                    </div>
                  )}
                  {big.map((control) => (
                    <RibbonButton
                      key={control.id + control.label}
                      control={control}
                      big
                      pressed={pressed(control)}
                      disabled={disabledReason(control)}
                    />
                  ))}
                  {small.length > 0 && (
                    <div
                      className={`ribbon-smalls${small.length > 6 ? " ribbon-smalls--grid" : ""}`}
                    >
                      {small.map((control) => (
                        <RibbonButton
                          key={control.id + control.label}
                          control={control}
                          pressed={pressed(control)}
                          disabled={disabledReason(control)}
                        />
                      ))}
                    </div>
                  )}
                </div>
                <div className="ribbon-group-label">{group.label}</div>
              </section>
            );
          })}
        </div>
      )}
    </div>
  );
}

function RibbonButton({
  control,
  big,
  pressed,
  disabled,
}: {
  control: RibbonControl;
  big?: boolean;
  pressed: boolean;
  disabled?: string;
}) {
  const api = useWorkbench();
  const title = [
    control.label,
    control.keys ? `(${control.keys})` : "",
    control.hint ? `\n${control.hint}` : "",
    disabled ? `\n${disabled}` : "",
  ]
    .join(" ")
    .trim();
  return (
    <button
      type="button"
      className={`ribbon-btn${big ? " ribbon-btn--big" : ""}${pressed ? " pressed" : ""}`}
      aria-pressed={control.toggle ? pressed : undefined}
      aria-disabled={disabled ? true : undefined}
      title={title}
      onMouseDown={(event) => {
        // Keep the caret in the page when pressing a formatting button.
        if (control.id.startsWith("editor.")) event.preventDefault();
      }}
      onClick={() => {
        if (disabled) {
          api.toast(disabled, "info");
          return;
        }
        api.run(control.id);
      }}
    >
      <Icon name={control.icon ?? "dot"} size={big ? 22 : 16} />
      <span className="ribbon-btn-label">{control.label}</span>
    </button>
  );
}

/** Commands the Quick Access Toolbar can hold, with labels and icons. */
export const QAT_CHOICES: { id: string; label: string; icon: string }[] = [
  { id: "app.save", label: "Save a version", icon: "save" },
  { id: "editor.undo", label: "Undo", icon: "undo" },
  { id: "editor.redo", label: "Redo", icon: "redo" },
  { id: "app.runChecks", label: "Run checks", icon: "check" },
  { id: "app.compileQuick", label: "Compile", icon: "download" },
  { id: "app.newScene", label: "New scene", icon: "filePlus" },
  { id: "app.comment", label: "New comment", icon: "comment" },
  { id: "view.mode.focus", label: "Focus mode", icon: "focus" },
  { id: "app.theme", label: "Theme", icon: "theme" },
  { id: "view.graph", label: "Graph", icon: "graph" },
];

export function qatItems() {
  const ids = getPrefs().qat;
  return ids
    .map((id) => QAT_CHOICES.find((c) => c.id === id))
    .filter(Boolean) as typeof QAT_CHOICES;
}
