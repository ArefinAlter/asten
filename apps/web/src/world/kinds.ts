import type { IdeaPrompt, WorldKind, WorldStatus } from "@asten/contracts";

export const WORLD_KINDS: WorldKind[] = [
  "place",
  "question",
  "system",
  "culture",
  "creature",
  "item",
  "fragment",
];

export function kindLabel(kind: WorldKind): string {
  switch (kind) {
    case "place":
      return "Place";
    case "culture":
      return "People";
    case "system":
      return "Rule";
    case "creature":
      return "Creature";
    case "item":
      return "Relic";
    case "question":
      return "Question";
    case "fragment":
      return "Scrap";
  }
}

export function kindHint(kind: WorldKind): string {
  switch (kind) {
    case "place":
      return "A harbor, a city, a room that matters";
    case "culture":
      return "How ordinary people live here";
    case "system":
      return "A limit. Magic or tech that can break the story";
    case "creature":
      return "Something that lives here";
    case "item":
      return "An object with weight";
    case "question":
      return "Why? How? What happens if…";
    case "fragment":
      return "A scrap before it knows what it is";
  }
}

export function promptLabel(prompt: IdeaPrompt): string {
  switch (prompt) {
    case "why":
      return "Why?";
    case "how":
      return "How?";
    case "what_result":
      return "Then what?";
  }
}

export function statusLabel(status: WorldStatus): string {
  switch (status) {
    case "spark":
      return "Spark";
    case "sketched":
      return "Sketch";
    case "settled":
      return "Settled";
  }
}

export const DISCOVERY_LABELS: Record<string, string> = {
  first_spark: "First spark",
  asked_why: "Asked why",
  first_place: "A place",
  ugly_triangles: "Ugly map",
  first_rule: "A rule holds",
  scene_anyway: "Wrote anyway",
};

export function icebergCaption(
  scenes: number,
  underwater: number,
  sparks: number,
) {
  if (scenes === 0 && underwater === 0 && sparks === 0) {
    return "The wall is empty. That is allowed.";
  }
  if (scenes === 0) {
    return "Lovely notes. A scene will make them matter.";
  }
  if (underwater === 0) {
    return "You are writing first. The world can catch up.";
  }
  return "Only the tip reaches the page.";
}

export function cardTilt(id: string): number {
  let total = 0;
  for (const character of id) {
    total += character.charCodeAt(0);
  }
  return ((total % 9) - 4) * 1.15;
}
