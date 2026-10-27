/**
 * The scripted backend of the hosted pre-alpha copy: an in-memory stand-in
 * for `asten-api` that answers the same paths with the same shapes, so every
 * view of the desk runs unchanged with no server and no network call.
 *
 * What is real here: the routes, the shapes, the binder, saving, snapshots,
 * drafts, comments, facts, entities, the world board and the rules about
 * what a repair may change. What is prepared: the marks (`SAMPLE_MARKS`),
 * which the real engine computes and this file only re-locates in the text,
 * dropping a mark once its quoted words are gone; and the hashes, which are
 * FNV-1a placeholders rather than the engine's BLAKE3 chain.
 */
import type {
  AstenDiagnostic,
  AstenDocument,
  Baseline,
  CanonEntity,
  CodeInfo,
  Comment,
  CompileArtifact,
  Draft,
  DocumentSnapshot,
  DocumentSummary,
  EventRecord,
  Fact,
  JsonValue,
  Location,
  MethodPack,
  Project,
  ProjectSettings,
  StoredInvariant,
  StoredRoute,
  StreamEvent,
  Suppression,
  TimelineEvent,
  TrackedChange,
  WorldEntry,
} from "@asten/contracts";
import {
  SAMPLE_CREATED,
  SAMPLE_DOCUMENTS,
  SAMPLE_ENTITIES,
  SAMPLE_EVENTS,
  SAMPLE_FACTS,
  SAMPLE_MARKS,
  SAMPLE_PACKS,
  SAMPLE_PROJECT_ID,
  SAMPLE_PROJECT_NAME,
  SAMPLE_SETTINGS,
  SAMPLE_WORLD,
  blockId,
  type MarkSeed,
} from "./project";

export interface LocalResponse {
  status: number;
  body: unknown;
}

type Json = Record<string, unknown>;

/** FNV-1a 64-bit as hex: a stand-in digest, not the engine's BLAKE3. */
export function fnv64(input: string): string {
  let hash = 0xcbf29ce484222325n;
  const prime = 0x100000001b3n;
  for (let index = 0; index < input.length; index += 1) {
    hash ^= BigInt(input.charCodeAt(index));
    hash = (hash * prime) & 0xffffffffffffffffn;
  }
  return hash.toString(16).padStart(16, "0");
}

interface DocNode {
  type: string;
  attrs?: Json;
  text?: string;
  content?: DocNode[];
}

export function blockText(node: DocNode | undefined): string {
  if (!node) return "";
  if (node.type === "text") return node.text ?? "";
  return (node.content ?? []).map(blockText).join("");
}

function docJson(id: string, paragraphs: string[]): JsonValue {
  return {
    type: "doc",
    content: (paragraphs.length ? paragraphs : [""]).map((text, index) => ({
      type: "paragraph",
      attrs: { id: blockId(id, index) },
      content: text ? [{ type: "text", text }] : [],
    })),
  } as JsonValue;
}

function topBlocks(content: JsonValue): DocNode[] {
  const doc = content as unknown as DocNode;
  return doc && Array.isArray(doc.content) ? doc.content : [];
}

export function wordsOf(content: JsonValue): number {
  return topBlocks(content)
    .map(blockText)
    .join(" ")
    .split(/\s+/)
    .filter(Boolean).length;
}

const stamp = () => new Date().toISOString();
let serial = 0;
const newId = (prefix: string) =>
  `${prefix}-${Date.now().toString(36)}${(serial += 1).toString(36)}`;

const CATALOG: CodeInfo[] = [
  ["CANON-E013", "attributes", "Attribute contradiction", "scribe"],
  ["CANON-E101", "lifecycles", "Acts after a settled death", "lifecycle"],
  [
    "CANON-E201",
    "time",
    "Story time runs backward inside a continuous run",
    "clocks",
  ],
  ["CANON-E202", "time", "The dates cannot all be true", "clocks"],
  ["CANON-E203", "time", "Effect before cause", "clocks"],
  ["CANON-E301", "space", "The journey does not fit the time", "atlas"],
  ["CANON-E302", "space", "In two places at once", "atlas"],
  [
    "CANON-E401",
    "knowledge",
    "Knowledge used before it is learned",
    "knowledge",
  ],
  ["CANON-E501", "relations", "Genealogy cycle", "relations"],
  ["CANON-E601", "rules", "Invariant violated", "invariants"],
  ["NAME-W001", "names", "Unknown name", "scribe"],
  ["NAME-W003", "names", "Confusable names", "scribe"],
  ["PLOT-I001", "ledger", "Setup waiting for its payoff", "invariants"],
].map(([code, area, title, pack]) => ({
  code,
  area: area as CodeInfo["area"],
  title,
  summary: title,
  pack,
}));

export class SampleBackend {
  private projects: Project[];
  private documents = new Map<string, AstenDocument>();
  private events = new Map<string, EventRecord[]>();
  private snapshots: DocumentSnapshot[] = [];
  private drafts: Draft[] = [];
  private comments: Comment[] = [];
  private changes: TrackedChange[] = [];
  private facts: Fact[] = SAMPLE_FACTS.map((fact) => ({ ...fact }));
  private entities: CanonEntity[] = SAMPLE_ENTITIES.map((e) => ({
    ...e,
    aliases: [...e.aliases],
  }));
  private timeline: TimelineEvent[] = SAMPLE_EVENTS.map((e) => ({ ...e }));
  private world: WorldEntry[] = SAMPLE_WORLD.map((w) => ({ ...w }));
  private packs: MethodPack[] = SAMPLE_PACKS.map((p) => ({ ...p }));
  private settings: ProjectSettings = { ...SAMPLE_SETTINGS };
  private suppressions: Suppression[] = [];
  private baselines: Baseline[] = [];
  private invariants: StoredInvariant[] = [];
  private routes: StoredRoute[] = [];
  private resolved = new Set<string>();
  private version = 0;
  private listeners = new Set<(event: StreamEvent) => void>();

  constructor() {
    this.projects = [
      {
        id: SAMPLE_PROJECT_ID,
        name: SAMPLE_PROJECT_NAME,
        kind: "novel",
        description: "A short sample novel written for this release.",
        created_at: SAMPLE_CREATED,
        updated_at: SAMPLE_CREATED,
      },
    ];
    for (const seed of SAMPLE_DOCUMENTS) {
      const content = docJson(seed.id, seed.paragraphs);
      const record = this.eventFor(seed.id, 1, "author", content, "");
      this.events.set(seed.id, [record]);
      this.documents.set(seed.id, {
        id: seed.id,
        project_id: SAMPLE_PROJECT_ID,
        parent_id: seed.parent_id,
        kind: seed.kind,
        title: seed.title,
        content_json: content,
        position: seed.position,
        revision: 1,
        exclude_from_compile:
          seed.kind === "note" ||
          seed.kind === "wiki" ||
          seed.kind === "source",
        scene: {
          story_time: null,
          tick: null,
          context: { kind: "narration" },
          continuity_run: null,
          language: "en",
          introduces: [],
          ...seed.scene,
        },
        head_seq: 1,
        chain_hash: record.chain_hash,
        created_at: SAMPLE_CREATED,
        updated_at: SAMPLE_CREATED,
      });
    }
    // Earlier versions, so Source Control has a history to show: every page
    // committed once, the opening twice, and the last scene changed since.
    for (const document of this.documents.values()) {
      if (document.id === "sc1") continue;
      const older = document.id === "sc5";
      this.snapshots.push({
        id: `snap-init-${document.id}`,
        document_id: document.id,
        revision: 1,
        label: "Initial manuscript",
        content_json: older
          ? docJson("sc5", [
              "The spring tide came in late, as the ledger said it would, and the harbour filled around the chapel steps.",
              "Ida Fenn stood at the archive window and counted the boats that had not come back.",
              "Mara relit the lamp at noon and logged it.",
            ])
          : document.content_json,
        seq: older ? 0 : document.head_seq,
        chain_hash: older ? fnv64("older") : document.chain_hash,
        merkle_root: fnv64(`init-${document.id}`),
        created_at: "2026-10-02T09:00:00.000Z",
      });
    }
    const scene = this.documents.get("sc1")!;
    this.snapshots.push({
      id: "snap-1",
      document_id: "sc1",
      revision: 1,
      label: "First pass of the opening",
      content_json: docJson("sc1", [
        "Mara Vell climbed the steps with the oil can knocking against her knee. The lamp room smelled of soot.",
        "She looked down at Harrowmouth. Her grey eyes stung in the wind.",
      ]),
      seq: 1,
      chain_hash: scene.chain_hash,
      merkle_root: fnv64("snap-1"),
      created_at: "2026-10-02T10:00:00.000Z",
    });
    this.snapshots.push({
      id: "snap-2",
      document_id: "sc1",
      revision: 1,
      label: "Add Wen's note",
      content_json: scene.content_json,
      seq: 1,
      chain_hash: scene.chain_hash,
      merkle_root: fnv64("snap-2"),
      created_at: "2026-10-03T16:30:00.000Z",
    });
    this.drafts.push({
      id: "draft-1",
      document_id: "sc4",
      name: "crossing-at-dawn",
      base_seq: 1,
      base_state_json: this.documents.get("sc4")!.content_json,
      content_json: docJson("sc4", [
        "The crossing took the last of the night. Mara sat in the bow with the ledger wrapped in oilcloth, and Tomas rowed without looking at her.",
        "On the far shore the customs house was grey in the first light. Its door had a lock the size of a fist.",
        "When she turned back, the lamp at Gull Point was out.",
      ]),
      created_at: "2026-10-04T08:00:00.000Z",
      updated_at: "2026-10-04T08:00:00.000Z",
      merged_at: null,
    });
    this.comments.push({
      id: "c-1",
      document_id: "sc2",
      block_id: "sc2-p1",
      from_offset: 0,
      to_offset: 11,
      body: "Is the grudge line too neat? Leave for now.",
      resolved: false,
      created_at: "2026-10-03T12:00:00.000Z",
      updated_at: "2026-10-03T12:00:00.000Z",
    });
  }

  /** Change-stream subscription, the in-memory twin of the SSE stream. */
  subscribe(onEvent: (event: StreamEvent) => void): () => void {
    this.listeners.add(onEvent);
    return () => this.listeners.delete(onEvent);
  }

  private emit(event: StreamEvent) {
    for (const listener of this.listeners) {
      queueMicrotask(() => listener(event));
    }
  }

  private bump(kind: "document" | "canon", document?: AstenDocument) {
    this.version += 1;
    if (kind === "document" && document)
      this.emit({
        type: "document",
        document: document.id,
        seq: document.head_seq,
      });
    else this.emit({ type: "canon", what: "canon" });
    this.emit({ type: "revision", revision: this.revision() });
  }

  private revision() {
    return fnv64(`rev:${this.version}`);
  }

  private eventFor(
    documentId: string,
    seq: number,
    actor: string,
    content: JsonValue,
    previous: string,
  ): EventRecord {
    const content_hash = fnv64(JSON.stringify(content));
    return {
      id: `${documentId}-ev${seq}`,
      seq,
      actor,
      at: seq === 1 ? SAMPLE_CREATED : stamp(),
      payload: { type: "MetaEdited", field: "content", value: null },
      content_hash,
      prev_chain_hash: previous,
      chain_hash: fnv64(previous + content_hash),
    };
  }

  // ───────────────────────── diagnostics ─────────────────────────

  private locate(seed: MarkSeed): Location | null {
    const document = this.documents.get(seed.document);
    if (!document) return null;
    const blocks = topBlocks(document.content_json);
    for (const block of blocks) {
      const text = blockText(block);
      const start = text.indexOf(seed.quote);
      if (start >= 0)
        return {
          document: document.id,
          block: (block.attrs?.id as string | undefined) ?? null,
          start,
          end: start + seed.quote.length,
          quote: seed.quote,
          label: document.title,
        };
    }
    return null;
  }

  diagnosticsNow(): AstenDiagnostic[] {
    const enabled = new Set(
      this.packs
        .filter((pack) => pack.enabled)
        .flatMap(
          (pack) =>
            ((pack.manifest as { rules?: string[] }).rules ?? []) as string[],
        ),
    );
    const out: AstenDiagnostic[] = [];
    for (const seed of SAMPLE_MARKS) {
      if (this.resolved.has(seed.id)) continue;
      if (seed.code.startsWith("CANON") || seed.code.startsWith("PLOT")) {
        if (!enabled.has(seed.code)) continue;
      }
      const primary = this.locate(seed);
      if (!primary) continue;
      const suppression = this.suppressions.find(
        (item) => item.diagnostic_id === seed.id,
      );
      out.push({
        id: seed.id,
        code: seed.code,
        severity: seed.severity,
        tier: seed.severity === "error" ? "must" : "may",
        probability: seed.severity === "error" ? 1 : 0.8,
        title: seed.title,
        message: seed.message,
        primary,
        related: [],
        explanation: seed.explanation.map((text, index) => ({
          kind: index === 0 ? "fact" : "observation",
          id: `${seed.id}-s${index}`,
          text,
          location: null,
          status: index === 0 ? "settled" : "proposed",
          fault_prior: index === 0 ? 0.2 : 0.8,
          confidence: 1,
        })),
        culprit: [{ statement_id: `${seed.id}-s1`, probability: 0.8 }],
        repairs: seed.repairs.map((repair, index) => ({
          kind: repair.kind,
          label: repair.label,
          cost: index,
        })),
        provenance: {
          rules: [seed.code],
          packs: [],
          revision: this.revision(),
          engine_version: "sample",
        },
        stale: false,
        suppressed_by: suppression?.id ?? null,
        new_since_baseline: false,
        entity: seed.entity ?? null,
        details: "",
        evidence_fingerprint: fnv64(seed.id + primary.quote),
      });
    }
    return out;
  }

  // ───────────────────────── routing ─────────────────────────

  async handle(
    method: string,
    rawPath: string,
    body?: unknown,
  ): Promise<LocalResponse> {
    try {
      return this.route(method.toUpperCase(), rawPath, (body ?? {}) as Json);
    } catch (error) {
      return fail(500, error instanceof Error ? error.message : "Failed");
    }
  }

  private route(method: string, rawPath: string, body: Json): LocalResponse {
    const [path, queryString = ""] = rawPath.split("?");
    const query = new URLSearchParams(queryString);
    const parts = path.split("/").filter(Boolean).map(decodeURIComponent);

    if (parts[0] === "auth") return this.auth(method, parts.slice(1));
    if (parts[0] === "desk")
      return ok({
        version: "0.0.1-beta",
        host: "this browser (pre-alpha, sample data)",
        embedded: false,
        auth_mode: "off",
        database_path: null,
        logs_path: null,
        backups_path: null,
      });
    if (parts[0] === "packs") {
      if (method === "GET") return ok(this.packs);
      const pack = this.packs.find((item) => item.id === parts[1]);
      if (!pack) return fail(404, "No such pack");
      if (pack.pack_key === "scribe") return fail(409, "Scribe stays on.");
      pack.enabled = Boolean(body.enabled);
      this.bump("canon");
      return ok(pack);
    }
    if (parts[0] !== "projects") return fail(404, "Not found");
    if (parts.length === 1) {
      if (method === "GET") return ok(this.projects);
      const project: Project = {
        id: newId("project"),
        name: String(body.name ?? "Untitled"),
        kind: (body.kind as Project["kind"]) ?? "novel",
        description: null,
        created_at: stamp(),
        updated_at: stamp(),
      };
      this.projects.push(project);
      return ok(project);
    }
    const projectId = parts[1];
    const rest = parts.slice(2);
    const isSample = projectId === SAMPLE_PROJECT_ID;
    const head = rest[0];

    switch (head) {
      case "stream-ticket":
        return ok({ ticket: "local", expires_at: stamp() });
      case "documents":
        return this.documentsRoute(
          method,
          projectId,
          rest.slice(1),
          body,
          query,
        );
      case "drafts":
        return this.draftsRoute(method, rest.slice(1), body);
      case "snapshots":
        return this.restoreSnapshot(rest[1]);
      case "comments": {
        const comment = this.comments.find((item) => item.id === rest[1]);
        if (!comment) return fail(404, "No such comment");
        Object.assign(comment, body, { updated_at: stamp() });
        return ok(comment);
      }
      case "changes": {
        const change = this.changes.find((item) => item.id === rest[1]);
        if (!change) return fail(404, "No such change");
        change.accepted = Boolean(body.accepted);
        return ok(change);
      }
      case "facts":
        return this.collection<Fact>(
          method,
          rest.slice(1),
          body,
          this.facts,
          (input) => ({
            id: newId("fact"),
            project_id: projectId,
            entity_key: String(input.entity_key ?? ""),
            predicate: String(input.predicate ?? ""),
            value: String(input.value ?? ""),
            modality: (input.modality as Fact["modality"]) ?? "established",
            status: (input.status as Fact["status"]) ?? "confirmed",
            valid_from: null,
            valid_to: null,
            source: null,
            context: { kind: "world" },
            confidence: 1,
            anchor: null,
            provenance: { kind: "author", action: "settle" },
            settled_by: "author",
            settled_at: stamp(),
            created_at: stamp(),
            updated_at: stamp(),
          }),
        );
      case "proposals":
        return ok(
          isSample
            ? this.facts.filter((fact) => fact.status === "proposed")
            : [],
        );
      case "entities":
        if (rest[2] === "merge-into") {
          const source = this.entities.find((e) => e.id === rest[1]);
          const target = this.entities.find((e) => e.id === rest[3]);
          if (!source || !target) return fail(404, "No such entity");
          target.aliases.push(source.name, ...source.aliases);
          this.entities = this.entities.filter((e) => e !== source);
          this.bump("canon");
          return ok(target);
        }
        if (rest[2] === "rename") {
          const entity = this.entities.find((e) => e.id === rest[1]);
          if (!entity) return fail(404, "No such entity");
          return ok({
            entity: entity.id,
            from: entity.name,
            to: body.to,
            changes: [],
          });
        }
        return this.collection(
          method,
          rest.slice(1),
          body,
          this.entities,
          (input) => ({
            id: newId("entity"),
            project_id: projectId,
            kind: (input.kind as CanonEntity["kind"]) ?? "character",
            name: String(input.name ?? ""),
            aliases: (input.aliases as string[]) ?? [],
            notes: String(input.notes ?? ""),
            introduced_at: null,
            created_at: stamp(),
            updated_at: stamp(),
          }),
        );
      case "events":
        return this.collection(
          method,
          rest.slice(1),
          body,
          this.timeline,
          (input) => ({
            ...(SAMPLE_EVENTS[0] as TimelineEvent),
            id: newId("event"),
            project_id: projectId,
            title: String(input.title ?? "Event"),
            document_id: (input.document_id as string) ?? null,
            entity_id: (input.entity_id as string) ?? null,
            participants: (input.participants as string[]) ?? [],
            tick: (input.tick as number) ?? null,
            kind: (input.kind as TimelineEvent["kind"]) ?? { kind: "acts" },
            created_at: stamp(),
            updated_at: stamp(),
          }),
        );
      case "world-entries":
        return this.collection(
          method,
          rest.slice(1),
          body,
          this.world,
          (input) => ({
            id: newId("world"),
            project_id: projectId,
            kind: (input.kind as WorldEntry["kind"]) ?? "fragment",
            title: String(input.title ?? ""),
            summary: String(input.summary ?? ""),
            prompt: null,
            status: (input.status as WorldEntry["status"]) ?? "spark",
            board_x: Number(input.board_x ?? 80),
            board_y: Number(input.board_y ?? 80),
            map_x: null,
            map_y: null,
            hue: "",
            created_at: stamp(),
            updated_at: stamp(),
          }),
        );
      case "world-pulse":
        return ok({
          sparks: this.world.filter((w) => w.status === "spark").length,
          sketched: this.world.filter((w) => w.status === "sketched").length,
          settled: this.world.filter((w) => w.status === "settled").length,
          questions: this.world.filter((w) => w.kind === "question").length,
          places: this.world.filter((w) => w.kind === "place").length,
          systems: this.world.filter((w) => w.kind === "system").length,
          scenes: [...this.documents.values()].filter((d) => d.kind === "scene")
            .length,
          underwater: 0,
          iceberg: 0,
          discoveries: [],
        });
      case "settings":
        if (method === "PUT") {
          Object.assign(this.settings, body, { updated_at: stamp() });
          this.bump("canon");
        }
        return ok(this.settings);
      case "suppressions":
        return this.collection(
          method,
          rest.slice(1),
          body,
          this.suppressions,
          (input) => ({
            id: newId("supp"),
            project_id: projectId,
            rule_code: String(input.rule_code ?? ""),
            document_id: (input.document_id as string) ?? null,
            entity_key: (input.entity_key as string) ?? null,
            reason: String(input.reason ?? ""),
            scope: (input.scope as Suppression["scope"]) ?? "occurrence",
            diagnostic_id: (input.diagnostic_id as string) ?? null,
            evidence_fingerprint: null,
            stale: false,
            created_at: stamp(),
          }),
        );
      case "invariants":
        return this.collection(
          method,
          rest.slice(1),
          body,
          this.invariants,
          (input) => ({
            id: newId("inv"),
            project_id: projectId,
            invariant: {
              ...(input as unknown as StoredInvariant["invariant"]),
              id: newId("i"),
            },
            created_at: stamp(),
          }),
        );
      case "routes":
        return this.collection(
          method,
          rest.slice(1),
          body,
          this.routes,
          (input) => ({
            id: newId("route"),
            project_id: projectId,
            route: input.route as StoredRoute["route"],
            created_at: stamp(),
          }),
        );
      case "baselines":
        return this.baselinesRoute(method, projectId, rest.slice(1), body);
      case "story":
        return this.story(method, projectId, rest.slice(1), body, query);
      case "compile":
        return this.compile(String(body.format ?? "markdown"));
      case "import":
        return this.importDocument(projectId, body);
      default:
        return fail(404, "Not found");
    }
  }

  private auth(method: string, parts: string[]): LocalResponse {
    const what = parts[0];
    if (what === "status")
      return ok({
        mode: "off",
        owner_configured: false,
        paired_clients: 0,
        requires_auth: false,
      });
    if (what === "me")
      return ok({ token_id: null, kind: "desk", label: "This browser" });
    if (what === "tokens") return ok(method === "GET" ? [] : undefined);
    if (what === "pairing-codes")
      return fail(
        501,
        "Pairing needs the desktop app or a local API. This release runs on sample data in the browser.",
      );
    return fail(501, "Not available in this release.");
  }

  private collection<T extends { id: string }>(
    method: string,
    rest: string[],
    body: Json,
    list: T[],
    make: (input: Json) => NoInfer<T>,
  ): LocalResponse {
    if (!rest.length) {
      if (method === "GET") return ok(list);
      const item = make(body);
      list.push(item);
      this.bump("canon");
      return ok(item);
    }
    const index = list.findIndex((item) => item.id === rest[0]);
    if (index < 0) return fail(404, "Not found");
    if (method === "DELETE") {
      list.splice(index, 1);
      this.bump("canon");
      return { status: 204, body: undefined };
    }
    if (method === "PUT") {
      const patch = Object.fromEntries(
        Object.entries(body).filter(
          ([, value]) => value !== null && value !== undefined,
        ),
      );
      Object.assign(list[index], patch, { updated_at: stamp() });
      this.bump("canon");
    }
    return ok(list[index]);
  }

  private summary(document: AstenDocument): DocumentSummary {
    const { content_json, ...rest } = document;
    return { ...rest, words: wordsOf(content_json) };
  }

  private projectDocuments(projectId: string) {
    return [...this.documents.values()].filter(
      (d) => d.project_id === projectId,
    );
  }

  private documentsRoute(
    method: string,
    projectId: string,
    rest: string[],
    body: Json,
    query: URLSearchParams,
  ): LocalResponse {
    if (!rest.length) {
      if (method === "GET") {
        const list = this.projectDocuments(projectId);
        return ok(
          query.get("fields") === "summary"
            ? list.map((d) => this.summary(d))
            : list,
        );
      }
      const id = newId("doc");
      const content = (body.content_json as JsonValue) ?? docJson(id, []);
      const record = this.eventFor(id, 1, "author", content, "");
      const document: AstenDocument = {
        id,
        project_id: projectId,
        parent_id: (body.parent_id as string) ?? null,
        kind: (body.kind as AstenDocument["kind"]) ?? "scene",
        title: String(body.title ?? "Untitled"),
        content_json: content,
        position: Number(body.position ?? this.documents.size + 1),
        revision: 1,
        exclude_from_compile: false,
        scene: {
          story_time: null,
          tick: null,
          context: { kind: "narration" },
          continuity_run: null,
          language: "en",
          introduces: [],
        },
        head_seq: 1,
        chain_hash: record.chain_hash,
        created_at: stamp(),
        updated_at: stamp(),
      };
      this.documents.set(id, document);
      this.events.set(id, [record]);
      this.bump("document", document);
      return ok(document);
    }
    const document = this.documents.get(rest[0]);
    if (!document) return fail(404, "That page does not exist.");
    const sub = rest[1];
    if (!sub) {
      if (method === "GET") return ok(document);
      if (method === "DELETE") {
        this.documents.delete(document.id);
        this.bump("document", document);
        return { status: 204, body: undefined };
      }
      if (
        typeof body.expected_head_seq === "number" &&
        body.content_json &&
        body.expected_head_seq !== document.head_seq
      )
        return fail(409, "This page changed elsewhere.");
      if (body.content_json) {
        const log = this.events.get(document.id) ?? [];
        const previous = log.at(-1)?.chain_hash ?? "";
        const record = this.eventFor(
          document.id,
          document.head_seq + 1,
          String(body.actor ?? "author"),
          body.content_json as JsonValue,
          previous,
        );
        log.push(record);
        this.events.set(document.id, log);
        document.content_json = body.content_json as JsonValue;
        document.head_seq = record.seq;
        document.chain_hash = record.chain_hash;
        document.revision += 1;
      }
      for (const key of [
        "title",
        "position",
        "parent_id",
        "exclude_from_compile",
        "scene",
      ] as const) {
        if (body[key] !== undefined && body[key] !== null)
          (document as unknown as Json)[key] = body[key];
      }
      if (
        body.parent_id === null &&
        "parent_id" in body &&
        body.position !== undefined
      )
        document.parent_id = null;
      document.updated_at = stamp();
      this.bump("document", document);
      return ok(document);
    }
    switch (sub) {
      case "history":
        return ok(this.events.get(document.id) ?? []);
      case "verify": {
        const log = this.events.get(document.id) ?? [];
        return ok({
          document_id: document.id,
          events: log.length,
          head_seq: document.head_seq,
          chain_hash: document.chain_hash,
          merkle_root: fnv64(log.map((e) => e.chain_hash).join("")),
          ok: true,
          error: null,
        });
      }
      case "at":
        return ok(document.content_json);
      case "blame": {
        const out: Record<
          string,
          {
            start: number;
            end: number;
            seq: number;
            actor: string;
            at: string;
          }[]
        > = {};
        for (const block of topBlocks(document.content_json)) {
          const id = block.attrs?.id as string | undefined;
          if (id)
            out[id] = [
              {
                start: 0,
                end: blockText(block).length,
                seq: document.head_seq,
                actor: "author",
                at: document.updated_at,
              },
            ];
        }
        return ok(out);
      }
      case "drafts":
        if (method === "GET")
          return ok(this.drafts.filter((d) => d.document_id === document.id));
        {
          const draft: Draft = {
            id: newId("draft"),
            document_id: document.id,
            name: String(body.name ?? "draft"),
            base_seq: document.head_seq,
            base_state_json: document.content_json,
            content_json: document.content_json,
            created_at: stamp(),
            updated_at: stamp(),
            merged_at: null,
          };
          this.drafts.push(draft);
          return ok(draft);
        }
      case "snapshots":
        if (method === "GET")
          return ok(
            this.snapshots.filter((s) => s.document_id === document.id),
          );
        {
          const snapshot: DocumentSnapshot = {
            id: newId("snap"),
            document_id: document.id,
            revision: document.revision,
            label: (body.label as string) ?? null,
            content_json: document.content_json,
            seq: document.head_seq,
            chain_hash: document.chain_hash,
            merkle_root: fnv64(document.chain_hash + "m"),
            created_at: stamp(),
          };
          this.snapshots.push(snapshot);
          this.bump("document", document);
          return ok(snapshot);
        }
      case "comments":
        if (method === "GET")
          return ok(this.comments.filter((c) => c.document_id === document.id));
        {
          const comment: Comment = {
            id: newId("comment"),
            document_id: document.id,
            block_id: (body.block_id as string) ?? null,
            from_offset: Number(body.from_offset ?? 0),
            to_offset: Number(body.to_offset ?? 0),
            body: String(body.body ?? ""),
            resolved: false,
            created_at: stamp(),
            updated_at: stamp(),
          };
          this.comments.push(comment);
          return ok(comment);
        }
      case "changes":
        if (method === "GET")
          return ok(this.changes.filter((c) => c.document_id === document.id));
        {
          const change: TrackedChange = {
            id: newId("change"),
            document_id: document.id,
            kind: (body.kind as TrackedChange["kind"]) ?? "insert",
            block_id: (body.block_id as string) ?? null,
            text: String(body.text ?? ""),
            actor: "author",
            accepted: null,
            created_at: stamp(),
          };
          this.changes.push(change);
          return ok(change);
        }
      default:
        return fail(404, "Not found");
    }
  }

  private draftsRoute(
    method: string,
    rest: string[],
    body: Json,
  ): LocalResponse {
    const draft = this.drafts.find((d) => d.id === rest[0]);
    if (!draft) return fail(404, "No such draft");
    if (method === "DELETE") {
      this.drafts = this.drafts.filter((d) => d !== draft);
      return { status: 204, body: undefined };
    }
    if (rest[1] === "merge-preview")
      return ok({
        draft_id: draft.id,
        merged_content: draft.content_json,
        conflicts: [],
        ops: topBlocks(draft.content_json).length,
        new_marks: [],
        resolved_marks: [],
      });
    if (rest[1] === "merge") {
      const document = this.documents.get(draft.document_id);
      if (!document) return fail(404, "No such page");
      draft.merged_at = stamp();
      return this.documentsRoute(
        "PUT",
        document.project_id,
        [document.id],
        { content_json: draft.content_json, actor: `draft:${draft.name}` },
        new URLSearchParams(),
      );
    }
    if (body.name) draft.name = String(body.name);
    if (body.content_json) draft.content_json = body.content_json as JsonValue;
    draft.updated_at = stamp();
    return ok(draft);
  }

  private restoreSnapshot(id: string | undefined): LocalResponse {
    const snapshot = this.snapshots.find((s) => s.id === id);
    if (!snapshot) return fail(404, "No such snapshot");
    const document = this.documents.get(snapshot.document_id);
    if (!document) return fail(404, "No such page");
    return this.documentsRoute(
      "PUT",
      document.project_id,
      [document.id],
      { content_json: snapshot.content_json, actor: "restore" },
      new URLSearchParams(),
    );
  }

  private baselinesRoute(
    method: string,
    projectId: string,
    rest: string[],
    body: Json,
  ): LocalResponse {
    if (!rest.length) {
      if (method === "GET") return ok(this.baselines);
      const baseline: Baseline = {
        id: newId("base"),
        project_id: projectId,
        name: String(body.name ?? "Baseline"),
        revision: this.revision(),
        diagnostic_ids: this.diagnosticsNow().map((d) => d.id),
        active: false,
        created_at: stamp(),
      };
      this.baselines.push(baseline);
      return ok(baseline);
    }
    if (rest[0] === "clear") {
      this.baselines.forEach((b) => (b.active = false));
      return ok(this.baselines);
    }
    if (rest[1] === "activate") {
      this.baselines.forEach((b) => (b.active = b.id === rest[0]));
      return ok(this.baselines);
    }
    if (method === "DELETE") {
      this.baselines = this.baselines.filter((b) => b.id !== rest[0]);
      return { status: 204, body: undefined };
    }
    return fail(404, "Not found");
  }

  private story(
    method: string,
    projectId: string,
    rest: string[],
    body: Json,
    query: URLSearchParams,
  ): LocalResponse {
    const what = rest[0];
    const marks = projectId === SAMPLE_PROJECT_ID ? this.diagnosticsNow() : [];
    switch (what) {
      case "diagnostics":
        return ok({
          revision: this.revision(),
          diagnostics: marks,
          ledger: [],
          tiers: [{ script: "Latin", tier: "B", blocks: 40 }],
          stale_suppressions: [],
          timings_ms: { perceive: 0, analyze: 0, total: 0 },
          cached_blocks: 0,
          total_blocks: 0,
        });
      case "catalog":
        return ok(CATALOG);
      case "fact-card": {
        const entity = this.entity(query.get("entity") ?? "");
        if (!entity) return fail(404, "No such name");
        return ok({
          entity: entity.id,
          name: entity.name,
          sort:
            entity.kind === "character"
              ? "character"
              : entity.kind === "place"
                ? "place"
                : "item",
          aliases: entity.aliases,
          facts: this.facts
            .filter((f) => f.entity_key === keyOf(entity.name))
            .map((f) => ({
              id: f.id,
              predicate: f.predicate,
              value: f.value,
              status: f.status,
              context: "world",
              valid_label: "always",
              settled_in: f.source?.document_id ?? null,
            })),
          at_label: "now",
          mention_count: this.mentionsOf(entity).length,
          open_marks: marks.filter((m) => m.entity === keyOf(entity.name))
            .length,
        });
      }
      case "mentions": {
        const entity = this.entity(query.get("entity") ?? "");
        return ok(entity ? this.mentionsOf(entity) : []);
      }
      case "where-settled": {
        const fact = this.facts.find((f) => f.id === query.get("fact"));
        if (!fact) return fail(404, "No such fact");
        return ok({
          fact: fact.id,
          document: fact.source?.document_id ?? null,
          block: null,
          start: 0,
          end: 0,
          quote: "",
          settled_by: fact.settled_by ?? "author",
          settled_at: fact.settled_at ?? fact.created_at,
        });
      }
      case "scene-summary": {
        const document = this.documents.get(rest[1] ?? "");
        if (!document) return fail(404, "No such page");
        const text = topBlocks(document.content_json).map(blockText).join(" ");
        const names = this.entities.filter(
          (e) =>
            text.includes(e.name) || e.aliases.some((a) => text.includes(a)),
        );
        return ok({
          document: document.id,
          title: document.title,
          reading_pos: document.position,
          time_label: document.scene.story_time ?? "unplaced",
          bounds: {
            lo: document.scene.tick ?? null,
            hi: document.scene.tick ?? null,
          },
          bounds_label: document.scene.story_time ?? "unplaced",
          context: document.scene.context.kind,
          continuity_run: document.scene.continuity_run ?? null,
          mentions: names.length,
          entities: names.map((e) => e.id),
          events: this.timeline.filter((e) => e.document_id === document.id)
            .length,
          observations: 0,
          marks: marks.filter((m) => m.primary?.document === document.id)
            .length,
          script: "Latin",
          tier: "B",
          words: wordsOf(document.content_json),
        });
      }
      case "cast-search": {
        const q = (query.get("q") ?? "").toLowerCase();
        return ok(
          this.entities
            .filter((e) => e.name.toLowerCase().includes(q))
            .map((e) => ({
              entity: e.id,
              name: e.name,
              sort: e.kind,
              score: 1,
              mentions: this.mentionsOf(e).length,
            })),
        );
      }
      case "repairs":
        return this.applyRepair(body);
      case "bisect": {
        const mark = marks.find((m) => m.id === rest[1]);
        return ok({
          diagnostic_id: rest[1],
          found: Boolean(mark),
          document: mark?.primary?.document ?? null,
          seq: 1,
          actor: "author",
          at: SAMPLE_CREATED,
          event_type: "RichTextPatched",
          probes: 1,
        });
      }
      default:
        return method === "GET" ? ok([]) : fail(404, "Not found");
    }
  }

  private entity(key: string) {
    return this.entities.find(
      (e) =>
        e.id === key ||
        e.name.toLowerCase() === key.toLowerCase() ||
        keyOf(e.name) === key,
    );
  }

  private mentionsOf(entity: CanonEntity) {
    const names = [entity.name, ...entity.aliases].filter((n) => n.length > 2);
    const out = [];
    for (const document of this.documents.values()) {
      if (document.kind !== "scene") continue;
      for (const block of topBlocks(document.content_json)) {
        const text = blockText(block);
        for (const name of names) {
          const start = text.indexOf(name);
          if (start < 0) continue;
          out.push({
            document: document.id,
            document_title: document.title,
            block: (block.attrs?.id as string) ?? "",
            start,
            end: start + name.length,
            quote: name,
            reading_pos: document.position,
            confidence: 1,
            context: "narration",
          });
          break;
        }
      }
    }
    return out;
  }

  private applyRepair(body: Json): LocalResponse {
    const seed = SAMPLE_MARKS.find((m) => m.id === body.diagnostic_id);
    if (!seed) return fail(404, "That mark is gone.");
    const repair = seed.repairs[Number(body.repair_index ?? 0)];
    if (!repair) return fail(400, "No such repair");
    const kind = repair.kind;
    switch (kind.kind) {
      case "deprecate_fact": {
        const fact = this.facts.find((f) => f.id === kind.fact);
        if (fact) fact.status = "deprecated";
        break;
      }
      case "add_alias": {
        this.entity(kind.entity)?.aliases.push(kind.alias);
        break;
      }
      case "add_entity":
        this.entities.push({
          id: newId("entity"),
          project_id: SAMPLE_PROJECT_ID,
          kind: "character",
          name: kind.name,
          aliases: [],
          notes: "",
          introduced_at: null,
          created_at: stamp(),
          updated_at: stamp(),
        });
        break;
      case "clear_continuity":
      case "mark_context": {
        const document = this.documents.get(kind.document);
        if (document) {
          if (kind.kind === "clear_continuity")
            document.scene.continuity_run = null;
          else document.scene.context = kind.context;
        }
        break;
      }
      case "suppress":
        this.suppressions.push({
          id: newId("supp"),
          project_id: SAMPLE_PROJECT_ID,
          rule_code: seed.code,
          document_id: seed.document,
          entity_key: seed.entity ?? null,
          reason: String(body.reason ?? "Suppressed by the author"),
          scope: "occurrence",
          diagnostic_id: seed.id,
          evidence_fingerprint: null,
          stale: false,
          created_at: stamp(),
        });
        this.bump("canon");
        return ok({
          applied: { kind, label: repair.label, cost: 0 },
          message: "Suppressed.",
        });
      default:
        break;
    }
    this.resolved.add(seed.id);
    this.bump("canon");
    return ok({
      applied: { kind, label: repair.label, cost: 0 },
      message: `${repair.label}: done.`,
    });
  }

  private compile(format: string): LocalResponse {
    const ordered = this.readingOrder().filter(
      (d) => !d.exclude_from_compile && d.kind === "scene",
    );
    const title = SAMPLE_PROJECT_NAME;
    const paragraphs = (d: AstenDocument) =>
      topBlocks(d.content_json).map(blockText).filter(Boolean);
    let text: string;
    let mime = "text/plain";
    let ext = "txt";
    if (format === "markdown") {
      text =
        `# ${title}\n\n` +
        ordered
          .map((d) => `## ${d.title}\n\n${paragraphs(d).join("\n\n")}`)
          .join("\n\n");
      mime = "text/markdown";
      ext = "md";
    } else if (format === "plain") {
      text =
        `${title}\n\n` +
        ordered
          .map((d) => `${d.title}\n\n${paragraphs(d).join("\n\n")}`)
          .join("\n\n# # #\n\n");
    } else if (format === "html") {
      const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;");
      text =
        `<!doctype html><meta charset="utf-8"><title>${esc(title)}</title><h1>${esc(title)}</h1>` +
        ordered
          .map(
            (d) =>
              `<h2>${esc(d.title)}</h2>` +
              paragraphs(d)
                .map((p) => `<p>${esc(p)}</p>`)
                .join(""),
          )
          .join("");
      mime = "text/html";
      ext = "html";
    } else if (format === "fountain") {
      text =
        `Title: ${title}\n\n` +
        ordered
          .map(
            (d) => `.${d.title.toUpperCase()}\n\n${paragraphs(d).join("\n\n")}`,
          )
          .join("\n\n");
      ext = "fountain";
    } else {
      return fail(
        501,
        `${format.toUpperCase()} is written by the engine's compiler, which runs in the desktop app. This release compiles Markdown, plain text, HTML and Fountain in the browser.`,
      );
    }
    const artifact: CompileArtifact = {
      filename: `salt-and-lantern.${ext}`,
      mime,
      encoding: "utf-8",
      body: text,
    };
    return ok(artifact);
  }

  /** Binder order: depth-first by position. */
  readingOrder(): AstenDocument[] {
    const all = [...this.documents.values()];
    const out: AstenDocument[] = [];
    const walk = (parent: string | null) => {
      all
        .filter((d) => d.parent_id === parent)
        .sort((a, b) => a.position - b.position)
        .forEach((d) => {
          out.push(d);
          walk(d.id);
        });
    };
    walk(null);
    return out;
  }

  private importDocument(projectId: string, body: Json): LocalResponse {
    const format = String(body.format ?? "plain");
    if (format === "docx" || format === "fdx")
      return fail(
        501,
        `${format.toUpperCase()} import runs in the desktop app's engine.`,
      );
    const text = String(body.body ?? "");
    const paragraphs = text
      .split(/\n\s*\n/)
      .map((p) => p.replace(/^#+\s*/, "").trim())
      .filter(Boolean);
    const id = newId("doc");
    return this.documentsRoute(
      "POST",
      projectId,
      [],
      {
        title: String(body.filename ?? "Imported").replace(/\.[^.]+$/, ""),
        kind: "scene",
        content_json: docJson(id, paragraphs),
      },
      new URLSearchParams(),
    );
  }
}

/** The desk's entity key: the name, lower case, words joined by underscores. */
function keyOf(name: string): string {
  return name.trim().toLowerCase().split(/\s+/).join("_");
}

function ok(body: unknown): LocalResponse {
  // A copy, as from the wire: the desk must never hold the store's own objects.
  return {
    status: 200,
    body: body === undefined ? undefined : structuredClone(body),
  };
}

function fail(status: number, message: string): LocalResponse {
  return { status, body: { error: { code: String(status), message } } };
}
