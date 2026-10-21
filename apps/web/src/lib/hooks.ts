import { useQueryClient } from "@tanstack/react-query";
import { type RefObject, useEffect, useRef, useState } from "react";
import type { ProjectId, StreamEvent } from "@asten/contracts";
import { createAstenClient } from "./asten-client";

/**
 * Keep the desk live: subscribe to the project's server-sent events and
 * refresh the queries that changed. Each event names what moved, so only
 * those queries refetch.
 */
export function useProjectStream(projectId: ProjectId | undefined) {
  const queryClient = useQueryClient();
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    if (!projectId) return;
    let dispose: (() => void) | undefined;
    let cancelled = false;
    const client = createAstenClient();
    client
      .subscribe(
        projectId,
        (event: StreamEvent) => {
          if (event.type === "document") {
            void queryClient.invalidateQueries({
              queryKey: ["documents", projectId],
            });
            void queryClient.invalidateQueries({ queryKey: ["history"] });
            void queryClient.invalidateQueries({ queryKey: ["blame"] });
            void queryClient.invalidateQueries({ queryKey: ["comments"] });
            void queryClient.invalidateQueries({ queryKey: ["snapshots"] });
            void queryClient.invalidateQueries({ queryKey: ["drafts"] });
          } else if (event.type === "canon") {
            void queryClient.invalidateQueries({
              queryKey: ["facts", projectId],
            });
            void queryClient.invalidateQueries({
              queryKey: ["entities", projectId],
            });
            void queryClient.invalidateQueries({
              queryKey: ["events", projectId],
            });
            void queryClient.invalidateQueries({
              queryKey: ["documents", projectId],
            });
            void queryClient.invalidateQueries({
              queryKey: ["suppressions", projectId],
            });
            void queryClient.invalidateQueries({
              queryKey: ["settings", projectId],
            });
            void queryClient.invalidateQueries({
              queryKey: ["invariants", projectId],
            });
            void queryClient.invalidateQueries({
              queryKey: ["baselines", projectId],
            });
          } else if (
            event.type === "revision" ||
            event.type === "diagnostics"
          ) {
            void queryClient.invalidateQueries({
              queryKey: ["diagnostics", projectId],
            });
            void queryClient.invalidateQueries({ queryKey: ["scene-summary"] });
            void queryClient.invalidateQueries({ queryKey: ["fact-card"] });
          }
        },
        (open) => {
          if (!cancelled) setConnected(open);
        },
      )
      .then((disposer) => {
        if (cancelled) disposer();
        else dispose = disposer;
      })
      .catch(() => setConnected(false));
    return () => {
      cancelled = true;
      dispose?.();
    };
  }, [projectId, queryClient]);

  return connected;
}

/** Whether a CSS media query matches, kept live as the viewport changes. */
export function useMediaQuery(query: string): boolean {
  const [matches, setMatches] = useState(() =>
    typeof window === "undefined" ? false : window.matchMedia(query).matches,
  );
  useEffect(() => {
    const media = window.matchMedia(query);
    const update = () => setMatches(media.matches);
    update();
    media.addEventListener("change", update);
    return () => media.removeEventListener("change", update);
  }, [query]);
  return matches;
}

/** Focus an overlay when opened and return to its visible trigger on dismissal. */
export function usePanelFocus(
  open: boolean,
  triggerRef: RefObject<HTMLButtonElement | null>,
) {
  const panelRef = useRef<HTMLElement>(null);
  useEffect(() => {
    if (!open) return;
    const panel = panelRef.current;
    const trigger = triggerRef.current;
    if (!panel) return;
    panel.focus({ preventScroll: true });
    const onKey = (event: KeyboardEvent) => {
      if (event.key !== "Tab") return;
      const controls = Array.from(
        panel.querySelectorAll<HTMLElement>(
          ":is(button, input, select, textarea, a[href], [tabindex])",
        ),
      ).filter(
        (element) =>
          element.tabIndex >= 0 &&
          !element.matches(":disabled") &&
          element.getClientRects().length > 0 &&
          !element.closest("[inert]"),
      );
      const first = controls[0];
      const last = controls.at(-1);
      if (!first || !last) {
        event.preventDefault();
        panel.focus({ preventScroll: true });
      } else if (
        event.shiftKey &&
        (document.activeElement === first || document.activeElement === panel)
      ) {
        event.preventDefault();
        last.focus({ preventScroll: true });
      } else if (
        !event.shiftKey &&
        (document.activeElement === last || document.activeElement === panel)
      ) {
        event.preventDefault();
        first.focus({ preventScroll: true });
      }
    };
    panel.addEventListener("keydown", onKey);
    return () => {
      panel.removeEventListener("keydown", onKey);
      // Resizing back to desktop hides the toggle; do not move focus to it.
      if (trigger?.isConnected && trigger.getClientRects().length > 0) {
        trigger.focus({ preventScroll: true });
      }
    };
  }, [open, triggerRef]);
  return panelRef;
}

/** Persist a small UI preference per viewer. */
export function useStoredState<T extends string>(
  key: string,
  fallback: T,
): [T, (value: T) => void] {
  const [value, setValue] = useState<T>(() => {
    try {
      return (localStorage.getItem(key) as T | null) ?? fallback;
    } catch {
      return fallback;
    }
  });
  const update = (next: T) => {
    setValue(next);
    try {
      localStorage.setItem(key, next);
    } catch {
      // Fine without storage.
    }
  };
  return [value, update];
}
