import { INITIAL_LAYOUT, LIMITS, type LayoutState } from "./layout";

const finite = (value: number, fallback: number) =>
  Number.isFinite(value) ? value : fallback;
const bound = (value: number, min: number, max: number) =>
  Math.round(Math.max(min, Math.min(max, value)));

/** Fit preferred panel sizes to this workspace without overwriting the preferences on window resize. */
export function panelSpace(
  layout: LayoutState,
  width: number,
  height: number,
  sidebar: boolean,
  secondary: boolean,
) {
  const left = bound(
    finite(layout.sidebarWidth, INITIAL_LAYOUT.sidebarWidth),
    LIMITS.sidebar.min,
    LIMITS.sidebar.max,
  );
  const right = bound(
    finite(layout.secondaryWidth, INITIAL_LAYOUT.secondaryWidth),
    LIMITS.secondary.min,
    LIMITS.secondary.max,
  );
  const budget = Math.max(0, finite(width, 0) - 48 - 320);
  const leftMin = sidebar ? LIMITS.sidebar.min : 0;
  const rightMin = secondary ? LIMITS.secondary.min : 0;
  const desired = (sidebar ? left : 0) + (secondary ? right : 0);
  const floor = leftMin + rightMin;
  let sidebarWidth = left;
  let secondaryWidth = right;
  if (desired > budget && desired > floor) {
    const scale = Math.max(
      0,
      Math.min(1, (budget - floor) / (desired - floor)),
    );
    if (sidebar) sidebarWidth = Math.floor(leftMin + (left - leftMin) * scale);
    if (secondary)
      secondaryWidth = Math.floor(rightMin + (right - rightMin) * scale);
  }
  const usableHeight = Math.max(0, finite(height, 0));
  const panelMax = Math.max(
    0,
    Math.floor(usableHeight - Math.min(160, usableHeight / 2)),
  );
  return {
    sidebarWidth,
    secondaryWidth,
    panelHeight: bound(
      finite(layout.panelHeight, INITIAL_LAYOUT.panelHeight),
      Math.min(LIMITS.panel.min, panelMax),
      panelMax,
    ),
    panelMax,
  };
}
