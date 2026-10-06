import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type {
  CreateWorldEntry,
  ProjectId,
  UpdateWorldEntry,
} from "@margin/contracts";
import { createMarginClient } from "../lib/margin-client";

const client = createMarginClient();

export function useWorld(projectId?: ProjectId) {
  const queryClient = useQueryClient();

  const entries = useQuery({
    queryKey: ["world-entries", projectId],
    queryFn: () => client.listWorldEntries(projectId!),
    enabled: Boolean(projectId),
  });

  const pulse = useQuery({
    queryKey: ["world-pulse", projectId],
    queryFn: () => client.worldPulse(projectId!),
    enabled: Boolean(projectId),
  });

  const create = useMutation({
    mutationFn: (input: Omit<CreateWorldEntry, "project_id">) =>
      client.createWorldEntry({ project_id: projectId!, ...input }),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["world-entries"] }),
        queryClient.invalidateQueries({ queryKey: ["world-pulse"] }),
        queryClient.invalidateQueries({ queryKey: ["facts"] }),
      ]);
    },
  });

  const update = useMutation({
    mutationFn: (input: { id: string } & UpdateWorldEntry) => {
      const { id, ...patch } = input;
      return client.updateWorldEntry(projectId!, id, patch);
    },
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["world-entries"] }),
        queryClient.invalidateQueries({ queryKey: ["world-pulse"] }),
        queryClient.invalidateQueries({ queryKey: ["facts"] }),
        queryClient.invalidateQueries({ queryKey: ["diagnostics"] }),
      ]);
    },
  });

  return { entries, pulse, create, update };
}

export type WorldLens = "wall" | "atlas" | "dossier";
