import { deskStore } from "../lib/storage";

const KEY = "asten-theme";

export function currentTheme(): "day" | "night" {
  const set = document.documentElement.getAttribute("data-theme");
  if (set === "day" || set === "night") return set;
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "night"
    : "day";
}

export function applyStoredTheme() {
  const stored = deskStore.get(KEY);
  if (stored === "day" || stored === "night")
    document.documentElement.setAttribute("data-theme", stored);
}

/** A sun by day, a crescent moon by night. Thoth is also a moon god. */
export function ThemeToggle() {
  function toggle() {
    const next = currentTheme() === "night" ? "day" : "night";
    document.documentElement.setAttribute("data-theme", next);
    deskStore.set(KEY, next);
  }

  return (
    <button
      type="button"
      className="theme-toggle"
      onClick={toggle}
      aria-label="Switch between day and night themes"
      title="Day / night"
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <defs>
          <mask id="desk-tt-mask">
            <rect x="-12" y="-12" width="48" height="48" fill="white" />
            <circle className="tt-shadow" cx="17" cy="8" r="6.4" fill="black" />
          </mask>
        </defs>
        <g className="tt-rays">
          {Array.from({ length: 8 }, (_, i) => {
            const a = (i * Math.PI) / 4;
            return (
              <line
                key={i}
                x1={(12 + Math.cos(a) * 9.2).toFixed(2)}
                y1={(12 + Math.sin(a) * 9.2).toFixed(2)}
                x2={(12 + Math.cos(a) * 11.4).toFixed(2)}
                y2={(12 + Math.sin(a) * 11.4).toFixed(2)}
              />
            );
          })}
        </g>
        <circle
          className="tt-disk"
          cx="12"
          cy="12"
          r="6.8"
          mask="url(#desk-tt-mask)"
        />
      </svg>
    </button>
  );
}
