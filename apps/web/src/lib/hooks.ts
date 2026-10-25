import { useQueryClient } from "@tanstack/react-query";
import { type RefObject, useEffect, useRef, useState } from "react";
import type { AstenDocument, ProjectId, StreamEvent } from "@asten/contracts";
import { createAstenClient } from "./asten-client";
import { deskStore } from "./storage";

/** The mutation key of the desk's own document saves (see `App.tsx`). */
export const SAVE_DOCUMENT_KEY = ["save-document"] as const;

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
    const pendingBodies = new Map<string, number>();
    const savingDocument = (documentId: string) =>
      queryClient.isMutating({
        mutationKey: SAVE_DOCUMENT_KEY,
        predicate: (mutation) =>
          (mutation.state.variables as { documentId?: string })?.documentId ===
          documentId,
      }) > 0;
    // Events can arrive while our save is pending. Keep their sequence until
    // it settles: the save response may precede a second client's write.
    const unsubscribe = queryClient.getMutationCache().subscribe(() => {
      for (const [documentId, seq] of pendingBodies) {
        if (savingDocument(documentId)) continue;
        pendingBodies.delete(documentId);
        const key = ["document", projectId, documentId];
        const held = queryClient.getQueryData<AstenDocument>(key);
        if (!held || held.head_seq < seq)
          void queryClient.invalidateQueries({ queryKey: key });
      }
    });
    const client = createAstenClient();
    client
      .subscribe(
        projectId,
        (event: StreamEvent) => {
          if (event.type === "document") {
            // The summaries are cheap; the body is refetched only for the
            // one document that moved, and not when the desk already holds
            // that head or is the client saving it (its response carries
            // the new body).
            void queryClient.invalidateQueries({
              queryKey: ["documents", projectId],
            });
            const bodyKey = ["document", projectId, event.document];
            const held = queryClient.getQueryData<AstenDocument>(bodyKey);
            if (savingDocument(event.document)) {
              pendingBodies.set(
                event.document,
                Math.max(pendingBodies.get(event.document) ?? 0, event.seq),
              );
            } else if (!held || held.head_seq < event.seq) {
              void queryClient.invalidateQueries({ queryKey: bodyKey });
            }
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
      unsubscribe();
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
  const [value, setValue] = useState<T>(
    () => (deskStore.get(key) as T | null) ?? fallback,
  );
  const update = (next: T) => {
    setValue(next);
    deskStore.set(key, next);
  };
  return [value, update];
}
