/**
 * The sample project the hosted pre-alpha copy opens on. Every sentence here
 * is original to this file and written for it; no author's text is used. The
 * marks are prepared by hand to show what the engine reports on a story like
 * this one; the hosted copy has no engine (see `backend.ts`).
 */
import type {
  AstenDiagnostic,
  CanonEntity,
  Context,
  DocumentKind,
  Fact,
  MethodPack,
  ProjectSettings,
  ScenePlacement,
  TimelineEvent,
  WorldEntry,
} from "@asten/contracts";

export const SAMPLE_PROJECT_ID = "sample-salt-and-lantern";
export const SAMPLE_PROJECT_NAME = "Salt and Lantern";
export const SAMPLE_CREATED = "2026-10-01T09:00:00.000Z";

const world: Context = { kind: "world" };

export interface SampleDocument {
  id: string;
  parent_id: string | null;
  kind: DocumentKind;
  title: string;
  position: number;
  scene?: Partial<ScenePlacement>;
  paragraphs: string[];
}

/** Paragraph ids are stable: `<document>-p<n>`. */
export const blockId = (documentId: string, index: number) =>
  `${documentId}-p${index + 1}`;

export const SAMPLE_DOCUMENTS: SampleDocument[] = [
  {
    id: "ms",
    parent_id: null,
    kind: "manuscript",
    title: "Manuscript",
    position: 1,
    paragraphs: [],
  },
  {
    id: "ch1",
    parent_id: "ms",
    kind: "chapter",
    title: "Chapter 1. Low Water",
    position: 1,
    paragraphs: [],
  },
  {
    id: "sc1",
    parent_id: "ch1",
    kind: "scene",
    title: "The lamp at Gull Point",
    position: 1,
    scene: { story_time: "Day 1, evening", tick: 1, continuity_run: "arrival" },
    paragraphs: [
      "Mara Vell climbed the last ninety steps with the oil can knocking against her knee. The lamp room at Gull Point smelled of soot and old rope, and the glass was furred with salt.",
      "She wiped a circle clear and looked down at Harrowmouth: forty roofs, one chapel, and a harbour that emptied twice a day like a lung. Her grey eyes stung in the wind.",
      "Old Wen had left a note pinned under the wick trimmer. Light at dusk. Log every ship. Trust the ledger, not the almanac.",
    ],
  },
  {
    id: "sc2",
    parent_id: "ch1",
    kind: "scene",
    title: "Ferry at dusk",
    position: 2,
    scene: { story_time: "Day 1, night", tick: 2, continuity_run: "arrival" },
    paragraphs: [
      "Tomas Reyne kept the ferry the way other men kept a grudge: carefully, and long past the point of use. He watched the lamp catch on the headland and grunted.",
      "“The new keeper,” he said to nobody. “She will have read the ledger by now, and she will have questions about the brass key.”",
      "The water slapped the hull. Somewhere behind the chapel a dog barked twice and stopped.",
    ],
  },
  {
    id: "ch2",
    parent_id: "ms",
    kind: "chapter",
    title: "Chapter 2. The Ledger",
    position: 2,
    paragraphs: [],
  },
  {
    id: "sc3",
    parent_id: "ch2",
    kind: "scene",
    title: "Ida’s archive",
    position: 1,
    scene: { story_time: "Day 3, morning", tick: 5, continuity_run: "ledger" },
    paragraphs: [
      "Ida Fenn kept the town’s papers in a room above the chandler’s, between a barometer that had not moved in years and a cat that rarely did.",
      "“Wen’s ledger,” she said, turning the pages with a pencil so she would not have to touch them. “Every ship since the flood year. Look at the tides he wrote down.”",
      "Mara looked. The tides in the ledger were an hour later than the almanac’s, every night, for thirty years. She found the brass key taped inside the back cover.",
    ],
  },
  {
    id: "sc4",
    parent_id: "ch2",
    kind: "scene",
    title: "Night crossing",
    position: 2,
    scene: { story_time: "Day 2, night", tick: 4, continuity_run: "ledger" },
    paragraphs: [
      "The crossing took the whole of the slack water. Mara sat in the bow with the ledger wrapped in oilcloth, and Tomas rowed without looking at her.",
      "On the far shore the customs house was dark. Its door had a lock the size of a fist, and the key in her pocket was the right shape for it.",
      "When she turned back, the lamp at Gull Point was out. Her brown eyes found the headland and found nothing there.",
    ],
  },
  {
    id: "ch3",
    parent_id: "ms",
    kind: "chapter",
    title: "Chapter 3. Spring Tide",
    position: 3,
    paragraphs: [],
  },
  {
    id: "sc5",
    parent_id: "ch3",
    kind: "scene",
    title: "What the water kept",
    position: 1,
    scene: { story_time: "Day 4, dawn", tick: 7 },
    paragraphs: [
      "The spring tide came in an hour late, exactly as the ledger said it would, and the harbour filled around the chapel steps.",
      "Ada Fenn stood at the archive window and counted the boats that had not come back. There were none. For once there were none.",
      "Mara relit the lamp at noon, which no keeper had ever done, and logged it in her own hand.",
    ],
  },
  {
    id: "notes",
    parent_id: null,
    kind: "note",
    title: "Notes",
    position: 2,
    paragraphs: [],
  },
  {
    id: "n-mara",
    parent_id: "notes",
    kind: "wiki",
    title: "Mara Vell",
    position: 1,
    paragraphs: [
      "Apprentice keeper at [[Gull Point]], newly arrived in [[Harrowmouth]]. Grey eyes, settled in [[The lamp at Gull Point]].",
      "Trusts paper more than people. Learns the ledger’s secret from [[Ida Fenn]].",
    ],
  },
  {
    id: "n-tomas",
    parent_id: "notes",
    kind: "wiki",
    title: "Tomas Reyne",
    position: 2,
    paragraphs: [
      "Ferryman of [[Harrowmouth]]. Knew [[Old Wen]] for thirty years. What he knows about [[The brass key]], and when he learned it, is open.",
    ],
  },
  {
    id: "n-ida",
    parent_id: "notes",
    kind: "wiki",
    title: "Ida Fenn",
    position: 3,
    paragraphs: [
      "Keeps the town archive above the chandler’s. Holds [[The tide ledger]] after Wen’s death.",
    ],
  },
  {
    id: "n-harrow",
    parent_id: "notes",
    kind: "wiki",
    title: "Harrowmouth",
    position: 4,
    paragraphs: [
      "Harbour town of forty roofs and one chapel. The harbour drains at low water. Reached from the far shore by [[Tomas Reyne]]’s ferry.",
      "Places: [[Gull Point]], the chapel steps, [[The customs house]].",
    ],
  },
  {
    id: "n-gull",
    parent_id: "notes",
    kind: "wiki",
    title: "Gull Point",
    position: 5,
    paragraphs: [
      "Headland lighthouse above [[Harrowmouth]], ninety steps to the lamp room. Kept by [[Old Wen]], then [[Mara Vell]].",
    ],
  },
  {
    id: "n-customs",
    parent_id: "notes",
    kind: "wiki",
    title: "The customs house",
    position: 6,
    paragraphs: [
      "On the far shore from [[Harrowmouth]]. Its lock fits [[The brass key]].",
    ],
  },
  {
    id: "n-ledger",
    parent_id: "notes",
    kind: "note",
    title: "The tide ledger",
    position: 7,
    paragraphs: [
      "Wen’s record of every ship since the flood year. Its tides run an hour behind the almanac. Setup: the hour is explained in chapter 4 (not written yet).",
      "Holds [[The brass key]]. See [[Research: tide tables]].",
    ],
  },
  {
    id: "n-key",
    parent_id: "notes",
    kind: "note",
    title: "The brass key",
    position: 8,
    paragraphs: [
      "Found inside [[The tide ledger]]. Fits the door of [[The customs house]]. Payoff pending.",
    ],
  },
  {
    id: "n-wen",
    parent_id: "notes",
    kind: "wiki",
    title: "Old Wen",
    position: 9,
    paragraphs: [
      "Keeper of [[Gull Point]] before [[Mara Vell]]. Dead before the story opens; appears only in the ledger and in other people’s memories.",
    ],
  },
  {
    id: "n-research",
    parent_id: "notes",
    kind: "source",
    title: "Research: tide tables",
    position: 10,
    paragraphs: [
      "Working notes on how harbour tide tables were kept by hand: high and low water by the clock, corrections for the age of the moon, and why a local table can drift from a printed almanac.",
      "Used for [[The tide ledger]].",
    ],
  },
];

const at = (minutes: number) =>
  new Date(Date.parse(SAMPLE_CREATED) + minutes * 60_000).toISOString();

export const SAMPLE_ENTITIES: CanonEntity[] = (
  [
    ["e-mara", "character", "Mara Vell", ["Mara"], "Apprentice keeper."],
    ["e-tomas", "character", "Tomas Reyne", ["Tomas"], "Ferryman."],
    ["e-ida", "character", "Ida Fenn", ["Ida"], "Archivist."],
    ["e-wen", "character", "Old Wen", ["Wen"], "The previous keeper."],
    ["e-harrow", "place", "Harrowmouth", [], "Harbour town."],
    ["e-gull", "place", "Gull Point", [], "Headland lighthouse."],
    ["e-customs", "place", "The customs house", [], "On the far shore."],
    ["e-ledger", "item", "The tide ledger", ["the ledger"], "Wen's ledger."],
    ["e-key", "item", "The brass key", ["the key"], "Taped in the ledger."],
  ] as const
).map(([id, kind, name, aliases, notes], index) => ({
  id,
  project_id: SAMPLE_PROJECT_ID,
  kind,
  name,
  aliases: [...aliases],
  notes,
  introduced_at: null,
  created_at: at(index),
  updated_at: at(index),
}));

function fact(
  id: string,
  entity_key: string,
  predicate: string,
  value: string,
  status: Fact["status"],
  document?: string,
): Fact {
  return {
    id,
    project_id: SAMPLE_PROJECT_ID,
    entity_key,
    predicate,
    value,
    modality: "established",
    status,
    valid_from: null,
    valid_to: null,
    source: document ? { document_id: document, start: 0, end: 0 } : null,
    context: world,
    confidence: status === "confirmed" ? 1 : 0.62,
    anchor: null,
    provenance:
      status === "confirmed"
        ? { kind: "author", action: "settle" }
        : { kind: "rule", code: "perceive", premises: [] },
    settled_by: status === "confirmed" ? "author" : null,
    settled_at: status === "confirmed" ? at(60) : null,
    created_at: at(30),
    updated_at: at(30),
  };
}

export const SAMPLE_FACTS: Fact[] = [
  fact("f1", "mara_vell", "eye_colour", "grey", "confirmed", "sc1"),
  fact("f2", "mara_vell", "role", "apprentice keeper", "confirmed", "sc1"),
  fact("f3", "tomas_reyne", "occupation", "ferryman", "confirmed", "sc2"),
  fact("f4", "old_wen", "alive", "false", "confirmed", "sc1"),
  fact("f5", "gull_point", "steps", "ninety", "confirmed", "sc1"),
  fact(
    "f6",
    "ida_fenn",
    "works_at",
    "the archive above the chandler's",
    "confirmed",
    "sc3",
  ),
  fact(
    "f7",
    "the_tide_ledger",
    "tide_offset",
    "one hour behind the almanac",
    "proposed",
    "sc3",
  ),
  fact("f8", "the_brass_key", "fits", "The customs house", "proposed", "sc4"),
  fact("f9", "harrowmouth", "roofs", "forty", "proposed", "sc1"),
];

function event(
  id: string,
  title: string,
  document: string,
  tick: number,
  participants: string[],
  kind: TimelineEvent["kind"] = { kind: "acts" },
): TimelineEvent {
  return {
    id,
    project_id: SAMPLE_PROJECT_ID,
    entity_id: participants[0] ?? null,
    document_id: document,
    title,
    story_time: null,
    reading_order: null,
    duration: null,
    notes: "",
    kind,
    participants,
    context: world,
    status: "confirmed",
    confidence: 1,
    anchor: null,
    tick,
    tick_lo: null,
    tick_hi: null,
    provenance: { kind: "author", action: "timeline" },
    created_at: at(40),
    updated_at: at(40),
  };
}

export const SAMPLE_EVENTS: TimelineEvent[] = [
  event("t1", "Mara lights the lamp", "sc1", 1, ["mara_vell"], {
    kind: "arrives",
    place: "gull_point",
  }),
  event("t2", "Tomas watches the lamp catch", "sc2", 2, ["tomas_reyne"]),
  event("t3", "The night crossing", "sc4", 4, ["mara_vell", "tomas_reyne"], {
    kind: "travels",
    from: "harrowmouth",
    to: "the_customs_house",
  }),
  event("t4", "Mara finds the brass key", "sc3", 5, ["mara_vell"], {
    kind: "acquires",
    item: "the_brass_key",
  }),
  event("t5", "Spring tide, an hour late", "sc5", 7, ["harrowmouth"]),
];

export const SAMPLE_WORLD: WorldEntry[] = (
  [
    [
      "w1",
      "place",
      "The chapel steps",
      "Where the spring tide stops.",
      "settled",
      120,
      80,
    ],
    [
      "w2",
      "system",
      "Tide reckoning",
      "Why a local table drifts from the almanac.",
      "sketched",
      360,
      140,
    ],
    [
      "w3",
      "question",
      "Why is the ledger an hour late?",
      "Open. Answered in chapter 4.",
      "spark",
      220,
      300,
    ],
    [
      "w4",
      "culture",
      "Keepers' customs",
      "Light at dusk, log every ship.",
      "sketched",
      480,
      320,
    ],
  ] as const
).map(([id, kind, title, summary, status, x, y]) => ({
  id,
  project_id: SAMPLE_PROJECT_ID,
  kind,
  title,
  summary,
  prompt: null,
  status,
  board_x: x,
  board_y: y,
  map_x: null,
  map_y: null,
  hue: "",
  created_at: at(50),
  updated_at: at(50),
}));

export const SAMPLE_PACKS: MethodPack[] = (
  [
    [
      "scribe",
      "Scribe",
      "Core canon compiler. Attribute contradiction is always on.",
      true,
      { rules: ["CANON-E013"], locked: true },
    ],
    [
      "lifecycle",
      "Lifecycle",
      "Notices a settled death followed by a later clocked act.",
      true,
      { rules: ["CANON-E101"] },
    ],
    [
      "clocks",
      "Clocks",
      "Compares reading order with stated story time. Silent until both clocks exist.",
      true,
      { rules: ["CANON-E201"] },
    ],
    [
      "diction",
      "Diction watch",
      "Optional craft hints. Never a canon error. Off until you enable it.",
      false,
      { hints: [] },
    ],
    [
      "relations",
      "Relations",
      "Genealogy cycles, exclusive relations, and parent ages.",
      true,
      { rules: ["CANON-E501", "CANON-E502", "CANON-E503"] },
    ],
    [
      "knowledge",
      "Knowledge",
      "Who knows what, and when they learned it.",
      true,
      { rules: ["CANON-E401"] },
    ],
    [
      "atlas",
      "Atlas",
      "Travel that fits the time, over the routes you declare.",
      true,
      { rules: ["CANON-E301", "CANON-E302"] },
    ],
    [
      "invariants",
      "Invariants",
      "Your own rules over the story, and the setup/payoff ledger.",
      true,
      { rules: ["CANON-E601", "PLOT-I001", "PLOT-I002"] },
    ],
  ] as const
).map(([key, name, summary, enabled, manifest], index) => ({
  id: `pack-${index + 1}`,
  pack_key: key,
  name,
  version: "0.1.0",
  summary,
  enabled,
  manifest: manifest as never,
  created_at: SAMPLE_CREATED,
}));

export const SAMPLE_SETTINGS: ProjectSettings = {
  project_id: SAMPLE_PROJECT_ID,
  strictness: "standard",
  narrator_reliable: true,
  clock: { unit: "day", calendar: { kind: "day_count" } },
  min_parent_age_days: null,
  resurrection_allowed: [],
  updated_at: SAMPLE_CREATED,
};

export interface MarkSeed {
  id: string;
  code: string;
  severity: AstenDiagnostic["severity"];
  title: string;
  message: string;
  document: string;
  paragraph: number;
  quote: string;
  entity?: string;
  repairs: {
    label: string;
    kind: AstenDiagnostic["repairs"][number]["kind"];
  }[];
  explanation: string[];
}

/** Prepared marks: what the engine reports on this text (see About). */
export const SAMPLE_MARKS: MarkSeed[] = [
  {
    id: "d1",
    code: "CANON-E013",
    severity: "error",
    title: "Attribute contradiction",
    message:
      "① Mara Vell’s eye colour is settled as grey in “The lamp at Gull Point”. ② “Night crossing” reads brown. ③ No change event explains it.",
    document: "sc4",
    paragraph: 2,
    quote: "brown eyes",
    entity: "mara_vell",
    repairs: [
      {
        label: "Retire the settled fact",
        kind: { kind: "deprecate_fact", fact: "f1" },
      },
      { label: "Keep my sentence", kind: { kind: "keep" } },
      { label: "Suppress with a reason", kind: { kind: "suppress" } },
    ],
    explanation: [
      "Settled: Mara Vell eye_colour = grey (The lamp at Gull Point)",
      "Observed: Mara Vell eye_colour = brown (Night crossing)",
    ],
  },
  {
    id: "d2",
    code: "CANON-E201",
    severity: "warning",
    title: "Story time runs backward inside a continuous run",
    message:
      "① “Ida’s archive” (Day 3) and “Night crossing” (Day 2) are in the continuity run “ledger”. ② In reading order the second steps back a day.",
    document: "sc4",
    paragraph: 0,
    quote: "The crossing took the whole of the slack water.",
    repairs: [
      {
        label: "Take the scene out of the run",
        kind: { kind: "clear_continuity", document: "sc4" },
      },
      {
        label: "Mark the scene a memory",
        kind: {
          kind: "mark_context",
          document: "sc4",
          context: { kind: "memory" },
        },
      },
      { label: "Keep my sentence", kind: { kind: "keep" } },
    ],
    explanation: [
      "Scene: Ida’s archive, Day 3, run “ledger”",
      "Scene: Night crossing, Day 2, run “ledger”",
    ],
  },
  {
    id: "d3",
    code: "CANON-E401",
    severity: "warning",
    title: "Knowledge used before it is learned",
    message:
      "① Tomas Reyne speaks of the brass key in “Ferry at dusk”. ② Nobody tells him of it, and the key is found later, in “Ida’s archive”.",
    document: "sc2",
    paragraph: 1,
    quote: "questions about the brass key",
    entity: "tomas_reyne",
    repairs: [
      {
        label: "Add an event: Tomas learns of the key earlier",
        kind: { kind: "keep" },
      },
      { label: "Keep my sentence", kind: { kind: "keep" } },
    ],
    explanation: [
      "Event: Mara finds the brass key (Ida’s archive, Day 3)",
      "Use: Tomas mentions the key (Ferry at dusk, Day 1)",
    ],
  },
  {
    id: "d4",
    code: "NAME-W003",
    severity: "warning",
    title: "Confusable names",
    message:
      "① “Ada Fenn” is one letter from “Ida Fenn”. ② No entity is called Ada. A slip, or a new character?",
    document: "sc5",
    paragraph: 1,
    quote: "Ada Fenn",
    entity: "ida_fenn",
    repairs: [
      {
        label: "Add “Ada” as an alias of Ida Fenn",
        kind: { kind: "add_alias", entity: "ida_fenn", alias: "Ada" },
      },
      {
        label: "Add Ada Fenn as a character",
        kind: { kind: "add_entity", name: "Ada Fenn", sort: "character" },
      },
      { label: "Keep my sentence", kind: { kind: "keep" } },
    ],
    explanation: ["Known: Ida Fenn (character)", "Read: Ada Fenn, no match"],
  },
  {
    id: "d5",
    code: "PLOT-I001",
    severity: "information",
    title: "Setup waiting for its payoff",
    message:
      "① The brass key is set up in “Ida’s archive”. ② Nothing has opened the customs house yet. Information only.",
    document: "sc3",
    paragraph: 2,
    quote: "She found the brass key taped inside the back cover.",
    entity: "the_brass_key",
    repairs: [{ label: "Keep it open", kind: { kind: "keep" } }],
    explanation: ["Setup: The brass key (Ida’s archive)"],
  },
  {
    id: "d6",
    code: "CANON-E301",
    severity: "information",
    title: "The journey does not fit the time",
    message:
      "① No route is declared between Harrowmouth and the customs house. ② The crossing is placed in one slack water. Declare a route to weigh it.",
    document: "sc4",
    paragraph: 1,
    quote: "the customs house was dark",
    entity: "the_customs_house",
    repairs: [{ label: "Keep my sentence", kind: { kind: "keep" } }],
    explanation: ["Travel: Harrowmouth → the customs house, no route declared"],
  },
];
