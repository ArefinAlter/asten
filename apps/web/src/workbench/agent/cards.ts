/**
 * The shell sends cards as small descriptors (ids, never objects); the panel
 * resolves them against the desk's own data and renders them with the same
 * card views the offline assistant uses. Nothing here writes: a card only
 * offers buttons, and the person presses them.
 */
import type { QueryClient } from "@tanstack/react-query";
import type { AstenClient } from "../../lib/asten-client";
import { backlinks, buildVault, outgoing } from "../knowledge/vault";
import type { CardDesc } from "../ai/bridge";
import { extractivePoints, type Card } from "./script";
import type { Access } from "./tools";

export interface CardContext {
  client: AstenClient;
  projectId: string;
  queryClient: QueryClient;
  access: Access;
}

/** Turn a descriptor into the card the panel draws, or nothing if it is gone. */
export async function resolveCard(
  desc: CardDesc,
  ctx: CardContext,
): Promise<Card | undefined> {
  const { client, projectId, queryClient } = ctx;
  const snapshot = () =>
    queryClient.fetchQuery({
      queryKey: ["diagnostics", projectId],
      queryFn: () => client.diagnostics(projectId),
      staleTime: 0,
    });
  const titles = async () => {
    const documents = await client.listDocumentSummaries(projectId);
    return Object.fromEntries(documents.map((d) => [d.id, d.title]));
  };
  switch (desc.type) {
    case "marks": {
      const { diagnostics } = await snapshot();
      const marks = desc.ids
        .map((id) => diagnostics.find((m) => m.id === id))
        .filter((m): m is NonNullable<typeof m> => Boolean(m));
      if (!marks.length) return undefined;
      return { type: "marks", marks, titles: await titles() };
    }
    case "explain": {
      const { diagnostics } = await snapshot();
      const mark = diagnostics.find((m) => m.id === desc.mark_id);
      if (!mark) return undefined;
      const names = await titles();
      return {
        type: "explain",
        mark,
        title: names[mark.primary?.document ?? ""] ?? "the project",
        canRepair: ctx.access === "propose",
      };
    }
    case "canon": {
      const [entities, facts] = await Promise.all([
        client.listEntities(projectId),
        client.listFacts(projectId),
      ]);
      const entity = entities.find((e) => e.id === desc.entity_id);
      if (!entity) return undefined;
      const key = entity.name.trim().toLowerCase().split(/\s+/).join("_");
      return {
        type: "canon",
        entity,
        facts: facts.filter(
          (f) =>
            (f.entity_key === entity.id || f.entity_key === key) &&
            f.status !== "rejected",
        ),
      };
    }
    case "graph":
    case "source": {
      const [documents, entities] = await Promise.all([
        client.listDocuments(projectId),
        client.listEntities(projectId),
      ]);
      const vault = buildVault(documents, entities);
      const note = vault.byId.get(desc.note_id);
      if (!note) return undefined;
      if (desc.type === "graph")
        return {
          type: "graph",
          note,
          out: outgoing(vault, note.id),
          back: backlinks(vault, note.id).map((b) => b.note),
        };
      return {
        type: "source",
        note,
        points: extractivePoints(note.text, 3),
        links: outgoing(vault, note.id).map((n) => n.title),
      };
    }
    case "timeline": {
      const [events, names] = await Promise.all([
        client.listEvents(projectId),
        titles(),
      ]);
      const sorted = [...events].sort((a, b) => (a.tick ?? 0) - (b.tick ?? 0));
      return { type: "timeline", events: sorted, titles: names };
    }
    case "proposed_facts":
      return { type: "proposed", items: desc.items };
  }
}
