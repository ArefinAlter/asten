/**
 * The knowledge view's model, in the manner of an Obsidian vault: notes
 * (wiki, note and source pages, and the cast and places from the canon),
 * [[wikilinks]] between them, backlinks, properties from settled facts, and
 * a graph of it all. Pure; built from what the API already returns.
 */
import type { AstenDocument, CanonEntity, Fact } from "@asten/contracts";

export type NoteKind =
  "character" | "place" | "item" | "note" | "source" | "scene" | "unresolved";

export interface VaultNote {
  id: string;
  title: string;
  kind: NoteKind;
  folder: string;
  documentId?: string;
  entityId?: string;
  /** The desk's key for the entity (facts and marks name it this way). */
  entityKey?: string;
  aliases: string[];
  text: string;
}

export interface VaultLink {
  from: string;
  to: string;
  /** The sentence the link sits in, for backlinks. */
  context: string;
  /** "wikilink" from note text; "mention" when a scene names a cast member. */
  via: "wikilink" | "mention";
}

export interface Vault {
  notes: VaultNote[];
  links: VaultLink[];
  byId: Map<string, VaultNote>;
}

const WIKILINK = /\[\[([^\]|#]+)(?:#[^\]|]*)?(?:\|([^\]]+))?\]\]/g;

/** Every `[[Target]]` / `[[Target|shown]]` in a text, with its target. */
export function parseWikilinks(
  text: string,
): { target: string; shown: string; index: number }[] {
  const out = [];
  for (const match of text.matchAll(WIKILINK)) {
    const target = match[1].trim();
    out.push({
      target,
      shown: (match[2] ?? target).trim(),
      index: match.index ?? 0,
    });
  }
  return out;
}

/** Split text into plain runs and wikilinks, for rendering a note. */
export function splitWikilinks(
  text: string,
): ({ text: string } | { link: string; shown: string })[] {
  const parts: ({ text: string } | { link: string; shown: string })[] = [];
  let last = 0;
  for (const link of parseWikilinks(text)) {
    if (link.index > last) parts.push({ text: text.slice(last, link.index) });
    parts.push({ link: link.target, shown: link.shown });
    const raw = text.slice(link.index).match(/^\[\[[^\]]*\]\]/)?.[0] ?? "";
    last = link.index + raw.length;
  }
  if (last < text.length) parts.push({ text: text.slice(last) });
  return parts;
}

interface Node {
  type?: string;
  text?: string;
  content?: Node[];
}

export function documentText(content: unknown): string {
  const walk = (node: Node | undefined): string => {
    if (!node) return "";
    if (node.type === "text") return node.text ?? "";
    const inner = (node.content ?? []).map(walk);
    return node.type === "doc" ? inner.join("\n") : inner.join("");
  };
  return walk(content as Node);
}

const FOLDER: Record<NoteKind, string> = {
  character: "Characters",
  place: "Places",
  item: "Items",
  note: "Notes",
  source: "Research",
  scene: "Manuscript",
  unresolved: "Unresolved",
};

function sentenceAround(text: string, index: number): string {
  const start = Math.max(
    text.lastIndexOf(".", index - 1) + 1,
    text.lastIndexOf("\n", index - 1) + 1,
    0,
  );
  const endDot = text.indexOf(".", index);
  const end = endDot < 0 ? text.length : endDot + 1;
  return text
    .slice(start, end)
    .trim()
    .replace(
      /\[\[([^\]|#]+)(?:#[^\]|]*)?(?:\|([^\]]+))?\]\]/g,
      (_, target: string, shown?: string) => shown ?? target,
    );
}

export function buildVault(
  documents: AstenDocument[],
  entities: CanonEntity[],
  options: { mentions?: boolean } = {},
): Vault {
  const notes: VaultNote[] = [];
  const byKey = new Map<string, VaultNote>();
  const key = (title: string) => title.trim().toLowerCase();
  const add = (note: VaultNote) => {
    notes.push(note);
    byKey.set(key(note.title), note);
    for (const alias of note.aliases)
      if (!byKey.has(key(alias))) byKey.set(key(alias), note);
  };

  for (const entity of entities) {
    const kind: NoteKind =
      entity.kind === "character"
        ? "character"
        : entity.kind === "place"
          ? "place"
          : "item";
    add({
      id: `entity:${entity.id}`,
      title: entity.name,
      kind,
      folder: FOLDER[kind],
      entityId: entity.id,
      entityKey: entity.name.trim().toLowerCase().split(/\s+/).join("_"),
      aliases: entity.aliases,
      text: entity.notes,
    });
  }
  for (const document of documents) {
    if (!["wiki", "note", "source", "scene"].includes(document.kind)) continue;
    const text = documentText(document.content_json);
    const existing = byKey.get(key(document.title));
    if (existing && existing.entityId && !existing.documentId) {
      // A wiki page named after a cast member is that member's note.
      existing.documentId = document.id;
      existing.text = text;
      continue;
    }
    const kind: NoteKind =
      document.kind === "scene"
        ? "scene"
        : document.kind === "source"
          ? "source"
          : "note";
    add({
      id: `doc:${document.id}`,
      title: document.title,
      kind,
      folder: FOLDER[kind],
      documentId: document.id,
      aliases: [],
      text,
    });
  }

  const links: VaultLink[] = [];
  const seen = new Set<string>();
  const unresolved = new Map<string, VaultNote>();
  for (const note of [...notes]) {
    if (note.kind === "scene") continue;
    for (const link of parseWikilinks(note.text)) {
      let target = byKey.get(key(link.target));
      if (!target) {
        target = unresolved.get(key(link.target));
        if (!target) {
          target = {
            id: `unresolved:${key(link.target)}`,
            title: link.target,
            kind: "unresolved",
            folder: FOLDER.unresolved,
            aliases: [],
            text: "",
          };
          unresolved.set(key(link.target), target);
          notes.push(target);
        }
      }
      if (target.id === note.id) continue;
      const id = `${note.id}->${target.id}`;
      if (seen.has(id)) continue;
      seen.add(id);
      links.push({
        from: note.id,
        to: target.id,
        context: sentenceAround(note.text, link.index),
        via: "wikilink",
      });
    }
  }
  if (options.mentions !== false) {
    const cast = notes.filter((n) => n.entityId);
    for (const scene of notes.filter((n) => n.kind === "scene")) {
      for (const member of cast) {
        const names = [member.title, ...member.aliases].filter(
          (n) => n.length > 2,
        );
        const at = names.map((n) => scene.text.indexOf(n)).find((i) => i >= 0);
        if (at === undefined) continue;
        const id = `${scene.id}->${member.id}`;
        if (seen.has(id)) continue;
        seen.add(id);
        links.push({
          from: scene.id,
          to: member.id,
          context: sentenceAround(scene.text, at),
          via: "mention",
        });
      }
    }
  }
  return { notes, links, byId: new Map(notes.map((n) => [n.id, n])) };
}

export function backlinks(
  vault: Vault,
  id: string,
): (VaultLink & { note: VaultNote })[] {
  return vault.links
    .filter((link) => link.to === id)
    .map((link) => ({ ...link, note: vault.byId.get(link.from)! }))
    .filter((link) => link.note);
}

export function outgoing(vault: Vault, id: string): VaultNote[] {
  return vault.links
    .filter((l) => l.from === id)
    .map((l) => vault.byId.get(l.to)!)
    .filter(Boolean);
}

/** Properties for the right sidebar: facts about the entity, then the basics. */
export function properties(
  note: VaultNote,
  facts: Fact[],
): { key: string; value: string; status?: string }[] {
  const out: { key: string; value: string; status?: string }[] = [
    { key: "type", value: note.kind },
  ];
  if (note.aliases.length)
    out.push({ key: "aliases", value: note.aliases.join(", ") });
  if (note.entityId)
    for (const fact of facts.filter(
      (f) =>
        (f.entity_key === note.entityKey || f.entity_key === note.entityId) &&
        f.status !== "rejected",
    ))
      out.push({ key: fact.predicate, value: fact.value, status: fact.status });
  const words = note.text.split(/\s+/).filter(Boolean).length;
  if (words) out.push({ key: "words", value: String(words) });
  return out;
}

export interface GraphFilter {
  search: string;
  showOrphans: boolean;
  showUnresolved: boolean;
  showScenes: boolean;
  kinds: NoteKind[];
}

export const DEFAULT_FILTER: GraphFilter = {
  search: "",
  showOrphans: true,
  showUnresolved: true,
  showScenes: true,
  kinds: [
    "character",
    "place",
    "item",
    "note",
    "source",
    "scene",
    "unresolved",
  ],
};

/** The nodes and edges a graph shows, after filters; `center` gives a local graph. */
export function graphOf(
  vault: Vault,
  filter: GraphFilter,
  center?: { id: string; depth: number },
): { nodes: VaultNote[]; edges: VaultLink[] } {
  let allowed = new Set(
    vault.notes
      .filter((n) => filter.kinds.includes(n.kind))
      .filter((n) => filter.showUnresolved || n.kind !== "unresolved")
      .filter((n) => filter.showScenes || n.kind !== "scene")
      .map((n) => n.id),
  );
  if (center) {
    const keep = new Set([center.id]);
    let frontier = [center.id];
    for (let depth = 0; depth < center.depth; depth += 1) {
      const next: string[] = [];
      for (const link of vault.links) {
        if (frontier.includes(link.from) && !keep.has(link.to))
          next.push(link.to);
        if (frontier.includes(link.to) && !keep.has(link.from))
          next.push(link.from);
      }
      next.forEach((id) => keep.add(id));
      frontier = next;
    }
    allowed = new Set([...allowed].filter((id) => keep.has(id)));
    allowed.add(center.id);
  }
  const query = filter.search.trim().toLowerCase();
  if (query)
    allowed = new Set(
      [...allowed].filter((id) =>
        vault.byId.get(id)!.title.toLowerCase().includes(query),
      ),
    );
  const edges = vault.links.filter(
    (l) => allowed.has(l.from) && allowed.has(l.to),
  );
  let nodes = vault.notes.filter((n) => allowed.has(n.id));
  if (!filter.showOrphans) {
    const linked = new Set(edges.flatMap((e) => [e.from, e.to]));
    nodes = nodes.filter((n) => linked.has(n.id) || n.id === center?.id);
  }
  return { nodes, edges };
}
