const KEY = "asten-theme";

function currentTheme(): "day" | "night" {
  const set = document.documentElement.getAttribute("data-theme");
  if (set === "day" || set === "night") return set;
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "night" : "day";
}

/**
 * A sun by day, a crescent moon by night. The icon is drawn purely by CSS from
 * the theme attribute, so the server and the browser render the same markup.
 */
export function ThemeToggle() {
  function toggle() {
    const next = currentTheme() === "night" ? "day" : "night";
    document.documentElement.setAttribute("data-theme", next);
    try {
      localStorage.setItem(KEY, next);
    } catch {
      // Storage can be unavailable (private windows). The choice still applies to this page.
    }
  }

  return (
    <button type="button" className="theme-toggle" onClick={toggle} aria-label="Switch between day and night themes">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <defs>
          <mask id="tt-mask">
            <rect x="-12" y="-12" width="48" height="48" fill="white" />
            <circle className="tt-shadow" cx="17" cy="8" r="6.4" fill="black" />
          </mask>
        </defs>
        <g className="tt-rays">
          {Array.from({ length: 8 }, (_, i) => {
            const a = (i * Math.PI) / 4;
            const x1 = 12 + Math.cos(a) * 9.2;
            const y1 = 12 + Math.sin(a) * 9.2;
            const x2 = 12 + Math.cos(a) * 11.4;
            const y2 = 12 + Math.sin(a) * 11.4;
            return <line key={i} x1={x1.toFixed(2)} y1={y1.toFixed(2)} x2={x2.toFixed(2)} y2={y2.toFixed(2)} />;
          })}
        </g>
        <circle className="tt-disk" cx="12" cy="12" r="6.8" mask="url(#tt-mask)" />
      </svg>
    </button>
  );
}
