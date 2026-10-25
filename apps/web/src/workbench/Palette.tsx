import { useEffect, useMemo, useRef, useState } from "react";
import { Icon } from "./icons";

export interface PaletteItem {
  id: string;
  label: string;
  group: string;
  detail?: string;
  icon?: string;
  keys?: string;
  disabled?: string;
  run: () => void;
}

/** Subsequence match with a bonus for word starts; null when it does not match. */
export function fuzzyScore(query: string, text: string): number | null {
  const q = query.toLowerCase().trim();
  if (!q) return 0;
  const t = text.toLowerCase();
  const direct = t.indexOf(q);
  if (direct >= 0) return 1000 - direct - (direct === 0 ? -100 : 0);
  let score = 0;
  let at = 0;
  for (const char of q) {
    if (char === " ") continue;
    const found = t.indexOf(char, at);
    if (found < 0) return null;
    score += found === at ? 3 : 1;
    if (found === 0 || /[\s\-_./›]/.test(t[found - 1] ?? "")) score += 4;
    at = found + 1;
  }
  return score;
}

export function Palette({
  items,
  placeholder,
  initialQuery = "",
  onClose,
}: {
  items: PaletteItem[];
  placeholder: string;
  initialQuery?: string;
  onClose: () => void;
}) {
  const [query, setQuery] = useState(initialQuery);
  const [index, setIndex] = useState(0);
  const input = useRef<HTMLInputElement>(null);
  const list = useRef<HTMLDivElement>(null);
  useEffect(() => input.current?.focus(), []);

  const commandMode = query.startsWith(">");
  const q = commandMode ? query.slice(1) : query;
  const shown = useMemo(() => {
    const pool = commandMode
      ? items.filter((i) => i.group === "Commands" || i.group === "Ribbon")
      : items;
    return pool
      .map((item) => ({
        item,
        score: fuzzyScore(q, `${item.label} ${item.detail ?? ""}`),
      }))
      .filter(
        (x): x is { item: PaletteItem; score: number } => x.score !== null,
      )
      .sort((a, b) => (q ? b.score - a.score : 0))
      .slice(0, 80)
      .map((x) => x.item);
  }, [items, q, commandMode]);

  useEffect(() => setIndex(0), [query]);
  useEffect(() => {
    list.current
      ?.querySelector<HTMLElement>(`[data-index="${index}"]`)
      ?.scrollIntoView({ block: "nearest" });
  }, [index]);

  const pick = (item: PaletteItem | undefined) => {
    if (!item || item.disabled) return;
    onClose();
    item.run();
  };

  let lastGroup = "";
  return (
    <div
      className="wb-modal-backdrop palette-backdrop"
      onMouseDown={(e) => e.target === e.currentTarget && onClose()}
    >
      <div
        className="palette"
        role="dialog"
        aria-modal="true"
        aria-label="Command palette"
      >
        <div className="palette-input">
          <Icon name="search" />
          <input
            ref={input}
            value={query}
            placeholder={placeholder}
            aria-label={placeholder}
            role="combobox"
            aria-expanded="true"
            aria-controls="wb-palette-list"
            aria-activedescendant={shown[index] ? `pal-${index}` : undefined}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "ArrowDown") {
                e.preventDefault();
                setIndex((i) => Math.min(shown.length - 1, i + 1));
              } else if (e.key === "ArrowUp") {
                e.preventDefault();
                setIndex((i) => Math.max(0, i - 1));
              } else if (e.key === "Enter") {
                e.preventDefault();
                pick(shown[index]);
              } else if (e.key === "Escape") {
                e.preventDefault();
                onClose();
              }
            }}
          />
        </div>
        <div
          className="palette-list"
          id="wb-palette-list"
          role="listbox"
          ref={list}
        >
          {shown.length === 0 && <p className="pane-empty">No matches.</p>}
          {shown.map((item, i) => {
            const head = item.group !== lastGroup && !q ? item.group : null;
            lastGroup = item.group;
            return (
              <div key={`${item.group}:${item.id}`}>
                {head && <div className="palette-group">{head}</div>}
                <button
                  id={`pal-${i}`}
                  data-index={i}
                  type="button"
                  role="option"
                  aria-selected={i === index}
                  aria-disabled={item.disabled ? true : undefined}
                  className={`palette-item${i === index ? " active" : ""}${item.disabled ? " disabled" : ""}`}
                  title={item.disabled}
                  onMouseMove={() => setIndex(i)}
                  onClick={() => pick(item)}
                >
                  <Icon name={item.icon ?? "chevronRight"} />
                  <span className="palette-label">{item.label}</span>
                  {item.detail && (
                    <span className="palette-detail">{item.detail}</span>
                  )}
                  {item.keys && <kbd>{item.keys}</kbd>}
                </button>
              </div>
            );
          })}
        </div>
        <div className="palette-foot">
          <span>
            <kbd>↑</kbd>
            <kbd>↓</kbd> move
          </span>
          <span>
            <kbd>Enter</kbd> open
          </span>
          <span>
            <kbd>&gt;</kbd> commands only
          </span>
          <span>
            <kbd>Esc</kbd> close
          </span>
        </div>
      </div>
    </div>
  );
}
