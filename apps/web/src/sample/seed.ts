/**
 * Creates the sample novel as an ordinary project through the API, so a
 * desktop or local desk can open it and let the real engine weigh it. The
 * marks it shows are then the engine's own, not the prepared ones.
 */
import type { CreateCanonEntity, ProjectId } from "@asten/contracts";
import type { AstenClient } from "../lib/asten-client";
import {
  SAMPLE_DOCUMENTS,
  SAMPLE_ENTITIES,
  SAMPLE_FACTS,
  SAMPLE_PROJECT_NAME,
  blockId,
} from "./project";

export async function createSampleProject(
  client: AstenClient,
): Promise<ProjectId> {
  const project = await client.createProject({
    name: SAMPLE_PROJECT_NAME,
    kind: "novel",
    description: "A short sample novel written for Asten's pre-alpha release.",
  });
  const ids = new Map<string, string>();
  // Parents before children: the list is already in that order.
  for (const seed of SAMPLE_DOCUMENTS) {
    const created = await client.createDocument({
      project_id: project.id,
      parent_id: seed.parent_id ? (ids.get(seed.parent_id) ?? null) : null,
      kind: seed.kind,
      title: seed.title,
      position: seed.position,
      content_json: {
        type: "doc",
        content: (seed.paragraphs.length ? seed.paragraphs : [""]).map(
          (text, index) => ({
            type: "paragraph",
            attrs: { id: blockId(seed.id, index) },
            content: text ? [{ type: "text", text }] : [],
          }),
        ),
      },
      scene: seed.scene
        ? {
            story_time: seed.scene.story_time ?? null,
            tick: seed.scene.tick ?? null,
            context: { kind: "narration" },
            continuity_run: seed.scene.continuity_run ?? null,
            language: "en",
            introduces: [],
          }
        : null,
    });
    ids.set(seed.id, created.id);
  }
  const entityIds = new Map<string, string>();
  for (const entity of SAMPLE_ENTITIES) {
    const input: CreateCanonEntity = {
      project_id: project.id,
      kind: entity.kind,
      name: entity.name,
      aliases: entity.aliases,
      notes: entity.notes,
    };
    const created = await client.createEntity(input);
    entityIds.set(entity.id, created.id);
  }
  for (const fact of SAMPLE_FACTS.filter((f) => f.status === "confirmed")) {
    await client.createFact({
      project_id: project.id,
      entity_key: entityIds.get(fact.entity_key) ?? fact.entity_key,
      predicate: fact.predicate,
      value: fact.value,
      status: "confirmed",
    });
  }
  return project.id;
}
