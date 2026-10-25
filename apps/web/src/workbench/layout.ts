/**
 * The workbench's panel state, as a pure reducer: which panels are open,
 * their sizes (clamped), the bottom panel's tab and maximised state, the
 * ribbon's collapsed state, and the view mode of the page. Kept free of React
 * so the rules are tested directly (`layout.test.ts`).
 */

export type SidePane =
  "manuscript" | "knowledge" | "search" | "scm" | "checks" | "packs";

export type PanelTab =
  "problems" | "scales" | "output" | "terminal" | "timeline";
export type SecondaryTab = "agent" | "inspector" | "links";
export type PageMode = "print" | "draft" | "focus" | "read";

export interface LayoutState {
  sidebar: boolean;
  sidePane: SidePane;
  sidebarWidth: number;
  panel: boolean;
  panelTab: PanelTab;
  panelHeight: number;
  panelMaximized: boolean;
  secondary: boolean;
  secondaryTab: SecondaryTab;
  secondaryWidth: number;
  ribbonCollapsed: boolean;
  pageMode: PageMode;
  zoom: number;
  ruler: boolean;
  navigationPane: boolean;
  split: boolean;
}

export const LIMITS = {
  sidebar: { min: 200, max: 520 },
  secondary: { min: 280, max: 640 },
  panel: { min: 120, max: 720 },
  zoom: { min: 50, max: 200 },
} as const;

export const INITIAL_LAYOUT: LayoutState = {
  sidebar: true,
  sidePane: "manuscript",
  sidebarWidth: 272,
  panel: true,
  panelTab: "problems",
  panelHeight: 220,
  panelMaximized: false,
  secondary: true,
  secondaryTab: "agent",
  secondaryWidth: 340,
  ribbonCollapsed: false,
  pageMode: "print",
  zoom: 100,
  ruler: false,
  navigationPane: false,
  split: false,
};

export type LayoutAction =
  | { type: "toggleSidebar" }
  | { type: "showPane"; pane: SidePane }
  | { type: "togglePanel" }
  | { type: "showPanel"; tab: PanelTab }
  | { type: "toggleMaximizePanel" }
  | { type: "toggleSecondary" }
  | { type: "showSecondary"; tab: SecondaryTab }
  | { type: "resize"; target: "sidebar" | "secondary" | "panel"; size: number }
  | { type: "toggleRibbon" }
  | { type: "pageMode"; mode: PageMode }
  | { type: "zoom"; zoom: number }
  | { type: "zoomBy"; delta: number }
  | { type: "toggle"; key: "ruler" | "navigationPane" | "split" };

export const clamp = (value: number, min: number, max: number) =>
  Math.min(max, Math.max(min, Math.round(value)));

export function layoutReducer(
  state: LayoutState,
  action: LayoutAction,
): LayoutState {
  switch (action.type) {
    case "toggleSidebar":
      return { ...state, sidebar: !state.sidebar };
    case "showPane":
      // Pressing the active pane's icon again closes the side bar, as VS Code does.
      if (state.sidebar && state.sidePane === action.pane)
        return { ...state, sidebar: false };
      return { ...state, sidebar: true, sidePane: action.pane };
    case "togglePanel":
      return state.panel
        ? { ...state, panel: false, panelMaximized: false }
        : { ...state, panel: true };
    case "showPanel":
      return { ...state, panel: true, panelTab: action.tab };
    case "toggleMaximizePanel":
      return { ...state, panel: true, panelMaximized: !state.panelMaximized };
    case "toggleSecondary":
      return { ...state, secondary: !state.secondary };
    case "showSecondary":
      if (state.secondary && state.secondaryTab === action.tab)
        return { ...state, secondary: false };
      return { ...state, secondary: true, secondaryTab: action.tab };
    case "resize": {
      if (action.target === "sidebar")
        return {
          ...state,
          sidebarWidth: clamp(
            action.size,
            LIMITS.sidebar.min,
            LIMITS.sidebar.max,
          ),
        };
      if (action.target === "secondary")
        return {
          ...state,
          secondaryWidth: clamp(
            action.size,
            LIMITS.secondary.min,
            LIMITS.secondary.max,
          ),
        };
      return {
        ...state,
        panelHeight: clamp(action.size, LIMITS.panel.min, LIMITS.panel.max),
        panelMaximized: false,
      };
    }
    case "toggleRibbon":
      return { ...state, ribbonCollapsed: !state.ribbonCollapsed };
    case "pageMode":
      return { ...state, pageMode: action.mode };
    case "zoom":
      return {
        ...state,
        zoom: clamp(action.zoom, LIMITS.zoom.min, LIMITS.zoom.max),
      };
    case "zoomBy":
      return {
        ...state,
        zoom: clamp(
          state.zoom + action.delta,
          LIMITS.zoom.min,
          LIMITS.zoom.max,
        ),
      };
    case "toggle":
      return { ...state, [action.key]: !state[action.key] };
    default:
      return state;
  }
}

/**
 * Focus mode hides every panel and the ribbon without forgetting them; the
 * shell derives what to draw from the stored state plus the page mode.
 */
export function visiblePanels(state: LayoutState) {
  const focus = state.pageMode === "focus";
  return {
    sidebar: state.sidebar && !focus,
    panel: state.panel && !focus,
    secondary: state.secondary && !focus,
    ribbon: !state.ribbonCollapsed && !focus,
  };
}
