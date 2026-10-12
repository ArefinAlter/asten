import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
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
