# Margin: Complete Product, Research, and Engineering Handoff

> **Document status:** Foundational handoff  
> **Last updated:** 2026-09-21  
> **Current product stage:** Architectural scaffold / pre-MVP  
> **Primary audience:** Product leadership, design, research, engineering, editorial advisors, and future AI/ML teams  
> **Repository:** `e:\margin`  
> **Tagline:** **Let AI be your editor, not your writer.**

---

## 1. Purpose of This Document

This is the authoritative handoff for Margin. It records:

- the product vision and the problem Margin is intended to solve;
- the philosophical and research foundations behind the product;
- the product modules and expected workflows;
- the proposed canon, consistency, and editorial engine;
- the current repository architecture and exactly what is implemented;
- the intended technical direction for desktop, web, storage, diagnostics,
  collaboration, extension, and AI;
- the decisions that have already been made;
- decisions intentionally left open because they require research, prototyping,
  legal input, user testing, or business input;
- a staged delivery plan;
- quality, security, privacy, performance, accessibility, and reliability
  requirements;
- a detailed bibliography and open-source study list;
- immediate next steps for any team taking over the project.

This document is deliberately broader than a conventional README. A new team
should be able to understand the product and resume work without access to the
original planning conversation.

This is not a claim that every listed module should be built simultaneously.
Margin's long-term vision is intentionally broad, but the implementation must
remain phased, evidence-driven, and centered on the canon engine.

---

## 2. Executive Summary

Margin is a local-first desktop and web writing environment for novels,
screenplays, narrative non-fiction, research-heavy books, and other complex
long-form work.

Most writing tools solve one part of the process:

- word processors handle page-oriented text editing;
- Scrivener-like applications organize long manuscripts;
- Campfire and World Anvil store lore;
- Plottr-like applications visualize structure;
- timeline products manage dates;
- ProWritingAid-like tools inspect prose style;
- map and drawing products handle visual planning;
- citation managers manage research;
- generative AI products produce prose.

Writers currently move between several disconnected applications. Their
manuscript, world bible, timeline, maps, character files, source notes, and
editorial checks do not share a consistent model of the work. Contradictions
are discovered manually, often late in revision.

Margin's central idea is to treat a writing project more like an integrated
development environment:

1. The manuscript, wiki, research, timeline, graph, maps, and visual planning
   artifacts belong to one project.
2. Margin builds a structured, provenance-aware model of what the project says
   is true.
3. A compiler-like engine evaluates the model and reports contradictions,
   impossible timelines, knowledge leaks, travel impossibilities, unresolved
   references, and optional craft observations.
4. Every diagnostic points to evidence and remains subordinate to the author.
5. AI may help extract, compare, explain, and edit, but generating the author's
   prose is not the product's organizing purpose.

The defensible product is not a large checklist of writing tools. The
defensible product is the **canon engine**: a typed, temporal, spatial,
epistemic, provenance-aware knowledge graph connected directly to the
manuscript and surfaced through fast, understandable diagnostics.

---

## 3. Product Thesis

### 3.1 Core thesis

A complex manuscript can be treated as a partially structured program:

- prose and project files are the source;
- names, events, claims, attributes, places, dates, relationships, and rules
  are symbols and facts;
- the project bible is an explicit symbol table and canon store;
- extraction resolves prose mentions against that store;
- constraints encode what must or may be true;
- the editorial engine evaluates constraints;
- contradictions and ambiguities become diagnostics;
- author actions are the equivalent of code actions: revise the text, update
  canon, add an event that explains the change, classify the passage as dream
  or rumor, or suppress the diagnostic with a reason.

This metaphor is useful because it produces clear product behavior. It must not
be allowed to imply that fiction is reducible to formal rules. Formal checks
protect the author's intended experience; they do not determine artistic merit.

### 3.2 Product promise

Margin should eventually let an author:

- draft and revise a major manuscript in one environment;
- organize chapters, scenes, notes, research, and visual material;
- maintain characters, locations, factions, items, events, concepts, rules,
  sources, and arbitrary user-defined entity types;
- connect every kind of project artifact;
- explore those connections as a wiki, graph, timeline, map, outline,
  spreadsheet, corkboard, or freeform canvas;
- receive evidence-backed consistency diagnostics while retaining control over
  canon;
- compile and export professional deliverables;
- work offline and retain durable ownership of project data;
- optionally use local or user-selected AI as an editorial assistant.

### 3.3 What Margin must not become

Margin must not become:

- a prose-generation wrapper whose primary value is writing on the user's
  behalf;
- a rigid embodiment of one plotting doctrine;
- a tool that marks contested craft advice as objective error;
- a mandatory database form that prevents discovery writers from drafting;
- a cloud-only service that can lock authors out of unpublished work;
- a proprietary format with no credible export or recovery path;
- a false-positive machine that trains users to ignore diagnostics;
- a collection of shallow modules with no shared project model;
- a Word clone attempted before the core canon advantage is validated.

---

## 4. Non-Negotiable Product Principles

### 4.1 The author is the authority

Margin may propose, explain, compare, and warn. The author decides what is
canon and whether a diagnostic represents an error, an intentional mystery, an
unreliable account, a draft inconsistency, or an exception.

Every diagnostic must support one or more of these outcomes:

- edit the source;
- update the relevant canon statement;
- add a missing event or transition;
- reclassify modality, time, speaker, source, or point of view;
- accept a suggested resolution;
- suppress the diagnostic locally;
- suppress the rule for a scope;
- explain why the apparent contradiction is intentional.

Suppressions must be reviewable and retain a reason. Silent dismissal loses
valuable editorial context.

### 4.2 Canon is extractable, not mandatory

An architect may build a wiki before writing. A discovery writer may start
with an empty page. Both must be first-class workflows.

The system should obtain structured facts from:

- explicit wiki fields;
- document and scene metadata;
- inline annotations;
- deterministic extraction;
- local machine-learning extraction;
- optional AI-assisted extraction;
- direct author confirmation.

Extracted claims remain proposed until confidence and policy allow otherwise.
The system should never silently convert uncertain prose interpretation into
authoritative canon.

### 4.3 Writing methods are optional packs

Save the Cat, Story Grid, Snowflake, Hero's Journey, Heroine's Journey,
Kishōtenketsu, scene/sequel, Propp functions, romance beats, mystery clue
ledgers, screenplay sequences, and future methods belong in optional packs.

A method pack may provide:

- templates;
- custom entity or scene fields;
- views;
- reports;
- advisory rules;
- examples;
- terminology;
- workflow commands.

No method is the universal project default.

### 4.4 Separate truth from craft advice

Margin should distinguish at least:

- **Canon errors:** incompatible facts, impossible time, impossible presence,
  broken identity, violated user-defined world rules.
- **Warnings:** likely name drift, suspicious unresolved setup, orphaned entity,
  probable knowledge leak, incomplete metadata.
- **Information:** useful project observations and reports.
- **Craft hints:** optional, contestable advice such as adverb density, filter
  words, beat placement, or scene-shape observations.

Craft hints must be opt-in or explicitly selected with a method/editorial pack.

### 4.5 Provenance is mandatory

No structured claim is useful unless Margin can answer:

- where did this claim come from?
- who or what asserted it?
- when is it valid in story time?
- is it world truth, narrator assertion, belief, rumor, hypothesis, dream,
  retcon, or deprecated draft material?
- how confident is the extraction?
- has the author confirmed it?

The engine must preserve source spans as documents change. Stable document and
block identities are therefore architectural requirements.

### 4.6 Local-first and durable

The desktop application must be fully useful offline. The user must retain
access to project data without a subscription or remote service.

Cloud synchronization, collaboration, hosted inference, and web access may be
offered, but they must not become the only route to the user's manuscript.

### 4.7 Incremental by default

Margin should react like an IDE. Editing one paragraph must not reprocess an
entire series. Parsing, mention resolution, fact extraction, rule evaluation,
indexing, and diagnostics must be designed around stable identities and
incremental recomputation.

### 4.8 Accessibility and international writing are core requirements

The product must not assume English-only, left-to-right, Latin-script prose.
Bidirectional text, complex shaping, input method editors, keyboard-only use,
screen readers, reduced motion, high contrast, scalable text, and international
calendars must be considered before editor architecture is locked.

---

## 5. Intended Users and Process Archetypes

Margin should not define authors by one universal process. Research to date
suggests at least these overlapping archetypes.

### 5.1 Architect / plotter

Examples in the craft literature include K.M. Weiland, Randy Ingermanson, and
Larry Brooks.

Needs:

- hierarchical outlines;
- beat and sequence templates;
- expandable summaries;
- dependency and setup/payoff tracking;
- scene metadata;
- planned versus drafted comparisons;
- structure reports.

Failure mode to avoid: making the outline mandatory or treating template
deviation as an error.

### 5.2 Gardener / discovery writer

Examples include Stephen King, Anne Lamott, and Ray Bradbury.

Needs:

- immediate blank-page entry;
- minimal setup;
- automatic names and facts proposed after writing;
- lightweight notes;
- easy retroactive organization;
- revision-time diagnostics;
- no mandatory forms before prose.

Failure mode to avoid: forcing worldbuilding database work before drafting.

### 5.3 Spreadsheet analyst

The Story Grid workflow is a representative example.

Needs:

- scene rows and custom columns;
- value shifts;
- point-of-view, location, date, length, status, and purpose fields;
- sorting, filtering, grouping, formula-like reports, and export;
- visual distributions and pacing reports.

### 5.4 Card shuffler

Associated workflows include Nabokov's cards, Robert Caro's research cards, and
modern corkboard tools.

Needs:

- one card per scene, idea, source, clue, event, or note;
- drag-and-drop reordering;
- stacks, lanes, tags, links, and filters;
- stable references when cards move;
- manuscript order derived from card order where desired.

### 5.5 Worldbuilder-first author

Common among speculative fiction and series authors.

Needs:

- deep typed wiki;
- maps, calendars, languages, cultures, factions, species, technologies,
  magic systems, items, genealogy, and history;
- arbitrary custom schemas;
- time-varying facts and borders;
- series-wide canon with book-specific visibility.

### 5.6 Visual structuralist

Examples include John McPhee's structural diagrams, Jane Alison's non-arc
shapes, comics workflows, and storyboarding.

Needs:

- freeform spatial canvas;
- diagrams that link to real project entities and scenes;
- nonlinear and repeating structures;
- drawing and stylus support;
- storyboard panels and shot metadata.

### 5.7 Researcher

Examples include historical novelists, biographers, investigative writers, and
Robert Caro's research-heavy process.

Needs:

- sources, excerpts, citation metadata, page locators, interview records, and
  archival references;
- claims connected to evidence;
- distinction between historical fact, interpretation, fictionalization, and
  unresolved question;
- source comparison;
- citation export;
- auditability.

### 5.8 Screenwriter / playwright

Needs:

- format-aware drafting;
- Fountain and FDX interchange;
- scene headings, character cues, parentheticals, dialogue, transitions, and
  dual dialogue;
- scene numbering;
- page and runtime estimates;
- production revisions;
- character and location reports;
- storyboard and shot links.

### 5.9 Multi-book and collaborative team

Needs:

- series-wide canon;
- project-specific branches or states;
- editor, writer, researcher, and sensitivity-reader roles;
- comments, suggestions, assignments, and review states;
- synchronization that preserves offline work;
- conflict explanations understandable to non-technical users.

---

## 6. Primary Product Use Cases

Margin should eventually support these end-to-end use cases.

### 6.1 Start with prose

1. Create a novel project.
2. Begin writing without defining a wiki.
3. Margin identifies recurring names and proposes entities.
4. The author confirms that aliases refer to the same person.
5. Margin proposes facts with source citations.
6. The author accepts, edits, defers, or rejects them.
7. Later passages are checked against confirmed canon.

### 6.2 Start with a world bible

1. Create custom entity templates and a calendar.
2. Define characters, places, factions, routes, rules, and historical events.
3. Build an outline tied to those entities.
4. Draft scenes from outline cards.
5. Margin checks the prose against the explicit model.

### 6.3 Diagnose a continuity problem

1. An author changes a character's eye color or history.
2. The affected paragraph is parsed incrementally.
3. A new fact conflicts with an overlapping canon fact.
4. Margin emits a diagnostic with both source spans.
5. The author updates canon, edits prose, records an in-story change, marks an
   unreliable assertion, or suppresses the issue.
6. Dependent diagnostics update without a full-project rebuild.

### 6.4 Validate nonlinear chronology

1. Scenes have reading order and story-time placement.
2. Flashbacks and frame narratives use separate clocks.
3. Relative constraints such as "three days after the siege" are entered or
   extracted.
4. The temporal network detects an impossible cycle.
5. Margin shows the smallest useful explanation chain.

### 6.5 Validate travel

1. Locations and routes are placed on a real or invented map.
2. Travel modes define speed ranges and route restrictions.
3. Consecutive character appearances are compared.
4. Margin reports travel that cannot fit the elapsed story time.
5. The author changes time, route, transportation, location, or canon.

### 6.6 Track character knowledge

1. Facts can be world-true, narrator-asserted, or believed by an entity.
2. Scenes record presence, communication, observation, and disclosure.
3. A character refers to information before a plausible acquisition event.
4. Margin raises a knowledge warning with the missing path.
5. The author adds a disclosure, marks off-page knowledge, or confirms an
   intentional reveal.

### 6.7 Research-backed non-fiction

1. Import citation metadata and source files.
2. Capture excerpts with stable locators.
3. Connect manuscript claims to one or more pieces of evidence.
4. Flag uncited or source-disputed claims according to project policy.
5. Export citations through selected CSL styles.

### 6.8 Screenplay workflow

1. Draft in format-aware screenplay mode.
2. Link scenes to cast, locations, props, timeline, and storyboard panels.
3. Run formatting and naming checks.
4. Produce breakdown reports.
5. Export Fountain and FDX without losing semantic structure.

---

## 7. Product Scope by Surface

### 7.1 Project library

Required capabilities:

- create, open, rename, duplicate, archive, import, export, and delete projects;
- support novel, screenplay, non-fiction, and custom project kinds;
- recent projects and pinned projects;
- templates without making templates mandatory;
- safe project recovery and backup status;
- project health summary;
- series/workspace containers above individual books;
- explicit project schema and migration version;
- integrity verification before and after migration.

### 7.2 Manuscript editor

Long-term requirements:

- semantic rich-text document model;
- paragraphs, headings, block quotes, lists, scene breaks, code-like literal
  blocks where required, tables, images, captions, and horizontal rules;
- character and paragraph styles;
- comments, suggestions, tracked changes, accept/reject operations, and review
  authorship;
- footnotes, endnotes, cross-references, bookmarks, and internal links;
- find and replace, including project-wide and regular-expression modes;
- spelling, grammar, custom dictionaries, and language ranges;
- word, character, paragraph, page-estimate, and reading-time statistics;
- writing goals, sessions, and sprints;
- focus mode, split editor, side-by-side revision, and source comparison;
- inline entity hover and navigation;
- diagnostic squiggles and code actions;
- block and span IDs that survive normal edits;
- reliable undo/redo across semantic operations;
- clipboard interoperability;
- IME, bidirectional, and complex-script correctness;
- accessible keyboard and screen-reader behavior;
- crash recovery;
- autosave with visible state;
- document revision history;
- large-document virtualization;
- print/page preview eventually, without making page layout the first editor
  milestone.

The first editor release should target the writer's subset of Word rather than
attempting complete Word parity. Pagination, advanced floating layout, perfect
DOCX round-tripping, and every table feature are expensive long-tail projects.

### 7.3 Binder, outline, corkboard, and grid

The same underlying document hierarchy should support:

- part, chapter, scene, section, note, wiki, and source nodes;
- custom node types;
- arbitrary nesting where valid;
- drag-and-drop reordering;
- multi-selection and bulk metadata editing;
- corkboard cards;
- outliner rows;
- spreadsheet-like custom columns;
- status, label, point of view, location, story time, reading order, goal,
  conflict, outcome, value shift, word target, and revision state;
- saved filters and views;
- derived manuscript order;
- exclusion from compile;
- links to visual boards and method packs.

### 7.4 Wiki and project knowledge base

Initial built-in entity types may include:

- Character
- Place
- Faction
- Item
- Species
- Culture
- Event
- Concept
- Language
- Technology
- Magic System
- Rule
- Source
- Organization
- Relationship

The author must be able to add types and fields without waiting for a software
release.

Field types should eventually include:

- short and long text;
- rich text;
- integer and decimal numbers;
- boolean;
- date, time, interval, fuzzy date, and relative date;
- duration;
- entity reference and list of references;
- enum and multi-select;
- image, file, audio, and external URL;
- geometry and map layer;
- formula or derived field;
- source citation;
- structured claim/fact;
- user-defined object.

Wiki requirements:

- aliases;
- backlinks;
- templates;
- transclusion;
- source citations;
- history;
- custom views;
- time-sliced values;
- series scope;
- import and export;
- bulk edit;
- orphan and unresolved-link reports.

### 7.5 Graph view

The graph is not decorative. It should support:

- typed nodes and edges;
- filters by entity type, relationship, book, confidence, source, and time;
- time slicing;
- neighborhood expansion;
- path finding;
- clustering;
- pinning and saved layouts;
- navigation back to source spans;
- author-created and engine-inferred relations shown distinctly;
- export as image and structured data;
- accessibility alternative as a list/tree;
- large-graph level-of-detail behavior.

### 7.6 Timeline

Requirements:

- separate reading order and story order;
- point events and intervals;
- fuzzy and relative dates;
- multiple custom calendars;
- eras, leap rules, variable month lengths, and calendar conversion;
- character lifelines;
- event dependencies;
- concurrent events;
- nested timescales;
- visual uncertainty;
- source provenance;
- constraint explanations;
- event links to scenes, entities, locations, and sources;
- series-scale time range.

### 7.7 Maps and spatial model

Margin should distinguish:

- real geographic maps with coordinate reference systems;
- invented geographic worlds with scale and projection;
- planar diagrams such as buildings, rooms, ships, and stations;
- maps without known scale, where spatial diagnostics are disabled or
  qualitative.

Capabilities:

- points, lines, polygons, rasters, labels, and layers;
- import and export of common GIS formats where practical;
- routes and travel modes;
- speed ranges and constraints;
- terrain and border changes over time;
- scene and event placement;
- floor plans;
- measurement;
- printable map output;
- optional procedural map generation;
- spatial diagnostics;
- offline map behavior and licensing-aware tile management.

### 7.8 Sketching and storyboards

Requirements:

- mouse, touch, and pressure-sensitive stylus input;
- vector strokes with pressure-derived width;
- eraser, selection, transform, layers, colors, and simple shapes;
- handwriting or sketches attached to any project entity;
- storyboard panels tied to screenplay or prose scenes;
- shot size, angle, movement, duration, dialogue, and notes;
- image import and tracing;
- non-destructive export;
- efficient large-canvas rendering;
- no dependency on the writing editor's text layout engine.

### 7.9 Idea board

A freeform canvas should allow:

- cards, notes, images, document excerpts, entities, links, groups, frames, and
  arrows;
- embedded live references rather than copied snapshots;
- spatial grouping without forcing hierarchy;
- conversion of cards into scenes, wiki entities, events, or tasks;
- board templates;
- zooming and minimap;
- keyboard-accessible structured alternative.

### 7.10 Conlang workbench

Potential capabilities:

- phoneme inventory;
- orthography;
- phonotactic rules;
- syllable templates;
- morphology notes;
- lexicon;
- etymology and derivation links;
- word generation;
- inflection tables;
- sound-change rules;
- validation of invented terms in prose;
- pronunciation data;
- exportable dictionary.

This module is valuable for some speculative-fiction users but must not delay
the canon-engine MVP.

### 7.11 Non-fiction research and citations

Requirements:

- CSL-compatible bibliography;
- Zotero and BibTeX import;
- source records and attachments;
- page, timestamp, section, archive box, URL, and interview locators;
- claim-to-evidence links;
- quotation verification where text is available;
- source status and reliability notes;
- primary/secondary source distinction;
- anonymized or embargoed sources;
- citation style export;
- endnotes and footnotes;
- research questions and unresolved claims.

### 7.12 Screenplay and playwriting

Requirements:

- Fountain as a durable plain-text interchange format;
- FDX import/export;
- format-aware autocomplete;
- scene headings and location normalization;
- character cue normalization;
- dialogue, parenthetical, transition, centered text, lyrics, notes, and
  sections;
- page locking and revision colors in later phases;
- scene numbering;
- production breakdowns;
- cast/location/prop reports;
- runtime estimates;
- dual dialogue;
- title page;
- storyboard and shot integration.

### 7.13 Search and command system

Margin should have:

- command palette;
- quick open;
- global text search;
- structured search over entities and facts;
- filters over metadata;
- saved searches;
- regex search;
- fuzzy symbol search;
- backlinks;
- search result previews with source context;
- keyboard-first navigation;
- eventual natural-language query routed through the canon graph with source
  citations.

### 7.14 Problems and reports

The Problems surface should support:

- grouping by severity, file, entity, rule, method pack, or scope;
- stable diagnostic codes;
- full explanation and evidence;
- related locations;
- quick fixes;
- suppressions with reasons and scopes;
- stale-result indication;
- filter presets;
- baseline comparison;
- export for editorial review.

Reports may include:

- character presence by chapter;
- point-of-view distribution;
- scene and chapter length;
- pacing proxies;
- location distribution;
- dialogue distribution;
- setup/payoff ledger;
- unresolved clues;
- character knowledge state;
- relationship changes;
- timeline uncertainty;
- travel pressure;
- source coverage;
- name confusability;
- word and phrase repetition;
- custom method-pack reports.

### 7.15 Compile and export

Target formats:

- DOCX;
- PDF;
- EPUB;
- Markdown;
- plain text;
- HTML;
- Fountain;
- FDX;
- static series-bible website;
- JSON or project archive for full-fidelity backup.

Compile should support:

- inclusion and exclusion rules;
- front matter and back matter;
- section breaks;
- style mapping;
- templates;
- metadata;
- named compile profiles;
- preview;
- deterministic output where possible;
- validation and actionable errors;
- no mutation of source documents.

---

## 8. UX Model

### 8.1 Main shell

The current scaffold already presents the intended regions:

- title bar and global actions;
- activity bar for major workspaces;
- left project/binder sidebar;
- central tabbed workspace;
- right contextual inspector;
- bottom Problems panel;
- status bar.

This layout is a structural placeholder, not a final visual design.

### 8.2 Suggested activity areas

- Write
- Wiki
- Graph
- Timeline
- Map
- Board
- Research
- Reports
- Compile

Users should be able to hide, reorder, or simplify these areas. A first-time
novelist should not be confronted with every advanced module at once.

### 8.3 Progressive disclosure

Margin should reveal complexity in layers:

1. writing and basic organization;
2. lightweight metadata and links;
3. explicit wiki and timeline;
4. diagnostics;
5. advanced maps, rules, and method packs;
6. automation and AI.

### 8.4 Modes without silos

Planning, drafting, revising, and reviewing are recursive cognitive processes,
not a one-way wizard. Views may optimize for a mode, but users must move freely
and retain context.

### 8.5 Explainability

Any automated result must show:

- what Margin observed;
- which source spans and canon statements are involved;
- what rule or model produced the result;
- confidence and status where relevant;
- available actions;
- why a result changed or disappeared when practical.

---

## 9. Canon Engine Overview

### 9.1 Compiler-style pipeline

The target pipeline is:

```text
Project sources
  manuscript, wiki, scene metadata, maps, timeline, research, rules
        │
        ▼
Incremental parsing
  blocks, spans, document structure, explicit annotations
        │
        ▼
Mention detection and resolution
  names, aliases, pronouns, places, dates, references
        │
        ▼
Claim and fact extraction
  value, modality, source, confidence, valid time
        │
        ▼
Canon store and indexes
  typed graph, temporal index, spatial index, full-text index
        │
        ▼
Rule and constraint evaluation
  deterministic rules, Datalog, temporal/spatial solvers, optional ML/AI
        │
        ▼
Diagnostics and reports
  evidence, related locations, explanations, code actions
```

Every stage must accept a change set and invalidate only dependent results.

### 9.2 Entity model

An entity should eventually contain at least:

- stable identifier;
- entity type;
- canonical display name;
- aliases and validity periods;
- project/series scope;
- lifecycle status;
- custom fields;
- source references;
- creation and update metadata;
- optional geometry;
- optional external identifiers;
- optional visibility and collaboration metadata.

Entity types are schema-driven. Built-in types are conveniences, not hard
limits.

### 9.3 Fact model

The recommended fact shape is inspired by Wikidata statements while remaining
optimized for a local writing project:

```text
Fact
  id
  subject
  predicate
  object/value
  valid_story_time
  asserted_at_reading_position
  source_span(s)
  source_entity/source_document
  modality
  status
  confidence
  qualifiers
  scope
  created_by
  supersedes / superseded_by
```

Example:

```text
subject: Mira
predicate: hair_color
object: auburn
valid_story_time: [412-03-01, 412-08-12)
source: wiki/characters/mira, field hair_color
modality: world_true
status: canon
confidence: 1.0
```

### 9.4 Predicate schema

Each predicate may declare:

- human label and description;
- subject type constraints;
- object/value type;
- cardinality;
- whether it is functional during overlapping valid time;
- whether it can change;
- inverse predicate;
- symmetry, asymmetry, transitivity, or acyclicity where appropriate;
- units;
- default modality;
- validation rules;
- method-pack ownership;
- user-visible editor control.

### 9.5 Modality and epistemic state

Fiction routinely contains statements that are not world truth. The model must
represent at least:

- world true;
- narrator asserted;
- believed by an entity;
- claimed by an entity;
- rumor;
- hypothesis;
- lie;
- dream;
- vision;
- story within the story;
- counterfactual;
- planned but not canon;
- deprecated or retconned.

The exact vocabulary should be extensible. The key requirement is that Margin
must not compare every sentence as though it were an omniscient factual
statement.

### 9.6 Status and confidence

Candidate statuses:

- proposed;
- confirmed;
- canon;
- draft-only;
- disputed;
- deprecated;
- rejected;
- suppressed.

Confidence should reflect extraction confidence, not artistic confidence.
Explicit author input normally starts at full confidence, while extraction
results retain calibrated scores and model/rule provenance.

### 9.7 Source spans

A source span must not be only a raw byte offset. Raw offsets become invalid
after edits. The final editor and engine need:

- document ID;
- block/node ID;
- local offsets or anchors;
- revision;
- context fingerprint for recovery;
- original text snapshot when necessary;
- migration logic when blocks split or merge.

### 9.8 Closed-world versus open-world behavior

Full OWL-style open-world semantics are not a natural default for fiction
editing. Absence can be editorially meaningful, but absence is not always
false.

Margin should support rule-specific policies:

- known true;
- known false;
- unknown;
- expected but missing;
- assumed under a project policy.

Do not force one global assumption.

---

## 10. Extraction Tiers

### Tier 0: Explicit structure

Sources:

- wiki fields;
- scene metadata;
- links;
- inline entity tags;
- explicit dates;
- author-entered facts;
- map and timeline entries.

This tier should be deterministic, fast, offline, and available in the MVP.

### Tier 1: Deterministic text extraction

Potential capabilities:

- Aho-Corasick matching over names and aliases;
- title and honorific handling;
- date and duration parsing;
- simple attribute patterns;
- capitalization and screenplay formatting cues;
- dialogue attribution heuristics;
- direct relation patterns;
- user-defined extraction patterns.

Results should normally be proposed rather than silently promoted.

### Tier 2: Local machine learning

Potential capabilities:

- named entity recognition;
- coreference resolution;
- quotation attribution;
- temporal relation extraction;
- relation and event extraction;
- embedding-assisted alias suggestions;
- local classifiers for passage modality or point of view.

Models should run in background workers and never block typing.

### Tier 3: AI/agent-assisted editorial analysis

Potential capabilities:

- claim extraction from complex prose;
- subtle contradiction proposals;
- diagnostic explanations;
- source-aware question answering;
- investigation plans;
- batch review of uncertain extractions;
- editorial summaries with citations.

Requirements:

- the graph and project services are tools available to the agent;
- output is grounded in cited source spans;
- no manuscript is sent to a remote provider without explicit policy and user
  consent;
- model output never becomes canon silently;
- local models are supported where practical;
- cloud access should support user-controlled providers/keys;
- the AI role is editorial, not automatic authorship.

---

## 11. Diagnostic Catalogue

Diagnostic codes should be stable, searchable, documented, and namespaced.
The catalogue below is a target taxonomy, not yet implemented.

### 11.1 Attribute diagnostics: `CANON-E0xx`

- incompatible functional attributes over overlapping story time;
- alias or identity collision;
- impossible immutable-property change;
- unit inconsistency;
- custom field validation failure.

Example:

```text
error[CANON-E013]: attribute contradiction
  --> book1/ch07: paragraph 42

  "Mira pushed her black hair out of her eyes."
                        ^^^^^

  Conflicts with confirmed canon:
  Mira · hair_color · auburn
  Source: Character/Mira, hair_color

  Possible actions:
  - revise this passage;
  - update canon;
  - add a dye/disguise/change event;
  - mark the assertion as mistaken or deceptive;
  - suppress with a reason.
```

### 11.2 Existence and lifecycle: `CANON-E1xx`

- dead character acts without resurrection/flashback explanation;
- character appears before birth;
- destroyed item reappears;
- place exists outside its valid interval;
- entity is used outside project/series scope.

### 11.3 Timeline: `CANON-E2xx`

- impossible ordering;
- negative temporal constraint cycle;
- age mismatch;
- duration mismatch;
- effect before cause;
- overlapping events declared mutually exclusive;
- calendar conversion inconsistency;
- ambiguous relative date requiring confirmation.

### 11.4 Space and travel: `CANON-E3xx`

- entity in incompatible places at once;
- travel cannot fit elapsed time;
- route unavailable during the relevant period;
- border or terrain state contradicts scene time;
- location geometry or scale is insufficient for a requested check.

### 11.5 Knowledge and disclosure: `CANON-E4xx`

- character references information before acquisition;
- point-of-view narration reveals inaccessible internal state under selected
  rules;
- secret is treated as public without a disclosure path;
- clue reveal occurs without planting under an enabled mystery pack.

### 11.6 Relationship and genealogy: `CANON-E5xx`

- self-ancestor or genealogy cycle;
- incompatible parentage dates;
- inverse relationship missing or inconsistent;
- asymmetric relationship used symmetrically;
- relationship status overlaps incompatibly.

### 11.7 World rules: `CANON-E6xx`

- magic cost or prerequisite violation;
- technology availability violation;
- social/legal rule violation where explicitly modeled;
- species/biology constraint violation;
- conlang phonotactic violation;
- user-defined Datalog or plugin rule violation.

### 11.8 Format and evidence: `CANON-E7xx`

- screenplay character-name drift;
- malformed scene heading;
- citation missing or unresolved;
- quotation mismatch with available source;
- cited statistic differs from captured evidence;
- compile-blocking format issue.

### 11.9 Warnings

- near-duplicate names;
- confusable cast names;
- probable typo in an entity mention;
- orphaned wiki entity;
- unresolved prose mention;
- unresolved setup or story question;
- possible unplanted clue;
- stale fact after source revision;
- extraction awaiting confirmation;
- source without rights/provenance metadata.

### 11.10 Optional craft hints

Only through explicit packs:

- tense drift;
- repeated word proximity;
- adverb density;
- filter words;
- dialogue tag patterns;
- scene without selected goal/conflict/outcome fields;
- beat location outside a method's recommended range;
- cliché lexicon matches;
- point-of-view pattern observations;
- pacing and paragraph variation.

Margin must phrase these as observations or selected-method guidance, not
universal rules.

---

## 12. Temporal Engine

### 12.1 Two clocks

Each scene has at least:

- **reading order** or discourse position;
- **story time** or fabula position.

This allows:

- flashbacks;
- flashforwards;
- frame stories;
- parallel narratives;
- repeated events from different viewpoints;
- nonlinear chapter order.

### 12.2 Internal representation

Use integer ticks in a project-selected base unit. Calendars convert human
labels to and from ticks.

Do not bake Gregorian assumptions into the core.

### 12.3 Custom calendars

Calendar definitions may include:

- units;
- month/season names and lengths;
- weekdays;
- leap rules;
- eras;
- epoch;
- intercalary periods;
- multiple concurrent calendar systems;
- display formatting;
- uncertain conversion.

### 12.4 Fuzzy and relative time

Examples:

- "that winter";
- "three days after the siege";
- "before Mira arrived";
- "between the coronation and first snow";
- "around 1890";
- "late afternoon."

Represent these as ranges and constraints rather than fake precise timestamps.

### 12.5 Solvers

Recommended foundations:

- Simple Temporal Networks for difference constraints;
- Bellman-Ford negative-cycle detection for impossible networks;
- incremental STN variants if profiling justifies them;
- Allen interval algebra for qualitative relations;
- path consistency on bounded local subgraphs;
- interval trees for overlap and range queries.

Explanations should identify the small set of constraints forming the conflict,
not merely state that the global network is invalid.

---

## 13. Spatial Engine

### 13.1 Geometry

Locations may have:

- point geometry;
- line geometry;
- polygon or multipolygon;
- parent/containment relation;
- vertical level;
- uncertainty;
- valid-time interval;
- multiple representations at different scales.

### 13.2 Route graph

Routes should carry:

- endpoints;
- geometry;
- directionality;
- allowed travel modes;
- speed or duration range;
- cost;
- availability interval;
- prerequisites;
- source and confidence.

### 13.3 Travel checks

For consecutive appearances of an entity:

1. determine the relevant locations and time ranges;
2. select routes and travel modes available at that story time;
3. compute shortest feasible travel duration using Dijkstra or A*;
4. compare duration range with elapsed story time;
5. emit a diagnostic only when evidence exceeds the configured certainty
   threshold.

Results must tolerate unknown scale, hidden transportation, teleportation, and
other author-defined exceptions.

### 13.4 Indexing

- R-tree for geometry;
- cached location-pair route results;
- invalidation when route, mode, map scale, or story-time state changes;
- level-of-detail rendering for large maps.

---

## 14. Algorithm and Data-Structure Toolkit

Candidate techniques:

- entity mentions: Aho-Corasick over canonical names and aliases;
- alias groups: union-find where merge semantics are appropriate;
- name similarity: Jaro-Winkler and Levenshtein;
- fuzzy lookup: BK-tree or finite-state indexes;
- phonetic similarity: Double Metaphone or language-specific alternatives;
- rule evaluation: Datalog with semi-naive evaluation;
- incremental queries: Salsa-style demand-driven memoization;
- graph representation: adjacency structures through `petgraph` or a
  purpose-built index;
- graph cycles: Tarjan strongly connected components;
- reachability and dependency explanation: graph traversal with provenance;
- temporal constraints: STN and Bellman-Ford;
- interval relations: Allen algebra;
- time ranges: interval tree;
- spatial ranges: R-tree;
- routes: Dijkstra/A*;
- character prominence: PageRank or degree-based summaries;
- communities: Louvain or Leiden, report-only;
- repetition: sliding-window n-grams;
- approximate duplicate passages: MinHash or locality-sensitive hashing;
- full-text search: Tantivy;
- prefix and exact symbol indexes: FST;
- text editing: rope or tree-based text storage where native processing needs
  it;
- freehand simplification: Ramer-Douglas-Peucker;
- freehand fitting: Bézier fitting and pressure-to-outline techniques;
- generated maps: Poisson-disc sampling, Voronoi/Delaunay, and noise-based
  terrain, optional only.

Algorithms are candidates, not commitments. Benchmark at realistic series
scale before finalizing.

---

## 15. Performance Targets

Initial engineering targets:

- normal paragraph input remains visually responsive at 60 Hz or better;
- deterministic diagnostics update within approximately 50 ms after a
  paragraph edit in a 250,000-word project on a mid-range laptop;
- expensive local ML runs in background workers;
- remote AI never blocks typing or save;
- opening a large project shows usable structure before all optional indexes
  finish;
- autosave does not introduce visible typing stalls;
- project-wide search begins returning results quickly and streams the rest;
- the editor virtualizes content where necessary;
- diagnostics retain revision identity so stale results are not displayed as
  current;
- memory use is bounded and measured with multi-book projects;
- cold start, warm start, database migration, and index rebuild are benchmarked
  separately.

The 50 ms diagnostic target is a design target, not a currently measured
result.

---

## 16. Current Repository State

### 16.1 What is implemented

The repository currently contains a functioning architectural scaffold:

- Rust workspace;
- React 19 / TypeScript 6 / Vite 8 frontend;
- Tauri 2 desktop shell;
- Axum HTTP API;
- domain, application, storage, and diagnostic crates;
- SQLite schema and migration runner;
- shared TypeScript contracts;
- runtime frontend transport selection between HTTP and Tauri IPC;
- create/list/get project use cases;
- create/list document use cases;
- basic project and document validation;
- HTTP middleware;
- Docker files for web and API;
- Docker Compose topology;
- generated desktop icons;
- CI workflow;
- formatting, linting, type-check, build, and test commands;
- architecture and setup documentation;
- IDE-like placeholder application shell;
- an empty diagnostic rule registry ready for research-dependent rules.

### 16.2 What is not implemented

The scaffold is not an MVP. It does not yet include:

- a real manuscript editor;
- updates, deletion, transactions spanning multiple domain operations, or
  revision history;
- plain-file project bundle;
- knowledge entities or facts;
- mention resolution;
- extraction;
- canon diagnostics;
- timeline, graph, map, board, screenplay, citation, or compile modules;
- authentication or multi-user authorization;
- collaboration or synchronization;
- generated API contracts;
- plugin runtime;
- production CORS configuration;
- telemetry policy;
- cloud infrastructure;
- AI or local ML;
- final visual design;
- product license or distribution terms.

### 16.3 Repository map

```text
margin/
├── .github/workflows/ci.yml
├── apps/
│   ├── desktop/
│   │   ├── package.json
│   │   └── src-tauri/
│   │       ├── capabilities/default.json
│   │       ├── icons/
│   │       ├── src/lib.rs
│   │       ├── src/main.rs
│   │       ├── build.rs
│   │       ├── Cargo.toml
│   │       └── tauri.conf.json
│   └── web/
│       ├── src/App.tsx
│       ├── src/lib/margin-client.ts
│       ├── src/main.tsx
│       ├── src/margin.css
│       ├── Dockerfile
│       ├── nginx.conf
│       ├── package.json
│       └── vite.config.ts
├── assets/icon.svg
├── crates/
│   ├── margin-domain/
│   ├── margin-application/
│   ├── margin-storage/
│   │   ├── migrations/0001_initial.sql
│   │   └── tests/repository.rs
│   └── margin-diagnostics/
├── docs/
│   ├── architecture.md
│   └── PROJECT_HANDOFF.md
├── packages/contracts/
├── services/margin-api/
├── books/                         ignored research material
├── Cargo.toml
├── Cargo.lock
├── package.json
├── pnpm-workspace.yaml
├── pnpm-lock.yaml
├── compose.yaml
└── README.md
```

### 16.4 Git status

The repository has been initialized but the scaffold was not committed at the
time this handoff was created. Review the working tree before making the first
commit.

The `books/` directory is intentionally ignored and must not be committed.

---

## 17. Current Architecture

### 17.1 Dependency direction

```text
React UI ── HTTP ──► Axum API ───────┐
    │                                │
    └── Tauri invoke ─► Desktop ─────┤
                                     ▼
                           margin-application
                             │             │
                             ▼             ▼
                       margin-domain   Repository trait
                                             ▲
                                             │
                                      margin-storage
                                          SQLite

margin-diagnostics ── consumes domain snapshots; owns no UI or storage adapter
```

Rules:

1. Domain code must not depend on Tauri, Axum, SQLite, React, or AI providers.
2. Application use cases depend on domain types and repository interfaces.
3. Storage implements repository interfaces.
4. HTTP and Tauri are transport adapters.
5. The same application service serves desktop and web.
6. Diagnostics should consume stable domain snapshots and publish transport-
   neutral results.

### 17.2 Rust crates

#### `margin-domain`

Currently defines:

- `ProjectId`;
- `DocumentId`;
- `DiagnosticId`;
- `Project`;
- `ProjectKind`;
- `Document`;
- `DocumentKind`;
- `Severity`;
- `SourceRange`;
- `Diagnostic`.

IDs use UUID version 7 to retain uniqueness with useful creation ordering.

#### `margin-application`

Currently defines:

- `Application`;
- `Repository` trait;
- `ApplicationError`;
- `CreateProject`;
- `CreateDocument`;
- project and document use cases.

Validation currently covers non-empty project and document names and verifies
project existence before document operations.

#### `margin-storage`

Currently provides:

- `SqliteRepository`;
- SQLite connection setup;
- write-ahead logging;
- foreign-key enforcement;
- embedded migrations;
- mapping between SQLite rows and domain types;
- integration test for project/document persistence.

#### `margin-diagnostics`

Currently provides:

- `AnalysisContext`;
- `Rule` trait;
- `DiagnosticEngine`;
- rule registration;
- synchronous evaluation over a snapshot.

This is a minimal extension boundary. It is not yet incremental and should not
be mistaken for the final engine architecture.

#### `margin-api`

Currently provides:

- Axum router;
- API state;
- project and document endpoints;
- error mapping;
- database-backed application construction;
- process startup and graceful shutdown.

### 17.3 Frontend

The frontend:

- uses React and TanStack Query;
- defines a `MarginClient` interface;
- selects `TauriClient` when Tauri internals are present;
- otherwise uses `HttpClient`;
- lists and creates projects;
- lists project documents;
- displays the intended IDE shell regions.

The TypeScript contracts are currently handwritten. They must be generated
from an authoritative schema before the API becomes large.

### 17.4 Desktop

The Tauri shell:

- initializes a database in the operating system application-data directory;
- constructs the shared `Application`;
- exposes Tauri commands for project and document operations;
- uses the same built frontend as the browser application;
- grants only baseline Tauri core capabilities.

### 17.5 Web deployment

The scaffold includes:

- Nginx serving the built React application;
- `/api` reverse proxy to the Axum service;
- API container with persistent `/app/data`;
- Compose network and volume.

This is a deployment scaffold, not production infrastructure.

---

## 18. Current Database Schema

SQLite currently uses strict tables.

### 18.1 `projects`

Fields:

- `id`: text UUID primary key;
- `name`: required text;
- `kind`: `novel`, `screenplay`, `non_fiction`, or `other`;
- `description`: optional text;
- `created_at`;
- `updated_at`.

### 18.2 `documents`

Fields:

- `id`: text UUID primary key;
- `project_id`: foreign key with cascading delete;
- `parent_id`: self-reference with cascading delete;
- `kind`: manuscript, chapter, scene, wiki, note, or source;
- `title`;
- `content_json`;
- `position`;
- `revision`;
- `created_at`;
- `updated_at`.

The hierarchy index covers project, parent, and position.

### 18.3 `schema_metadata`

Currently records:

```text
content_format = prosemirror-json-v1
```

This records architectural intent. A ProseMirror editor has not yet been
installed and the final schema remains a design task.

### 18.4 Future schema areas

Do not add all of these in one migration. Introduce them behind proven domain
models:

- entity types and field schemas;
- entities and aliases;
- predicates;
- facts and qualifiers;
- source anchors;
- temporal constraints;
- events and calendars;
- geometries, routes, and travel modes;
- diagnostics and suppressions;
- extraction proposals;
- tags and saved views;
- comments and change tracking;
- sources and citation records;
- attachments and asset metadata;
- compile profiles;
- project settings and feature packs;
- sync/change-log data;
- background job state;
- plugin metadata;
- schema/index versioning.

---

## 19. Current API and Middleware

### 19.1 Endpoints

Current HTTP endpoints:

- `GET /health`
- `GET /api/v1/projects`
- `POST /api/v1/projects`
- `GET /api/v1/projects/{project_id}`
- `GET /api/v1/projects/{project_id}/documents`
- `POST /api/v1/projects/{project_id}/documents`

### 19.2 Middleware

Current middleware:

- request ID generation;
- request ID propagation;
- structured HTTP tracing;
- panic containment;
- gzip compression;
- 30-second request timeout;
- 10 MiB request-body limit;
- permissive development CORS.

### 19.3 Error mapping

- not found → HTTP 404;
- validation → HTTP 422;
- storage failure → HTTP 500.

Response shape:

```json
{
  "error": {
    "code": "422",
    "message": "invalid input: project name cannot be empty"
  }
}
```

### 19.4 Required API work

- generated OpenAPI or equivalent authoritative contract;
- pagination;
- filtering and stable sorting;
- optimistic concurrency/revision preconditions;
- idempotency where appropriate;
- update/delete endpoints;
- problem details with stable product error codes;
- authentication and authorization for hosted use;
- explicit deployment CORS allowlist;
- rate limiting for public deployments;
- asset upload/download policy;
- streaming for large exports and background tasks;
- WebSocket/SSE or sync protocol if required;
- endpoint and migration compatibility policy.

---

## 20. Storage and Project Format Strategy

### 20.1 Present state

SQLite is currently the source of truth for the scaffold.

### 20.2 Long-term requirement

Writers need durable ownership and credible recovery. The final design should
support a portable project bundle that can be validated, backed up, exported,
and opened without a cloud service.

### 20.3 Candidate strategies

#### SQLite-first bundle

Pros:

- transactions;
- indexes;
- compact;
- simple desktop implementation.

Cons:

- not human-readable;
- poor manual version-control diffs;
- corruption or migration fear requires excellent backup/export tooling.

#### Plain-file source plus rebuildable index

Pros:

- durable and inspectable;
- diffable;
- easier recovery;
- aligns with author trust.

Cons:

- transactions and multi-file consistency are harder;
- filenames and moves complicate identity;
- rich text and collaborative updates require careful representation.

#### Hybrid

Potential direction:

- durable project files or an archive as authoritative content;
- SQLite as transactional working store and rebuildable index;
- explicit snapshot/export process;
- stable IDs independent of filenames.

Do not decide based on ideology. Prototype:

- crash recovery;
- migration;
- large projects;
- backup;
- external edits;
- sync;
- rich-text fidelity;
- performance.

---

## 21. Collaboration and Synchronization

Collaboration is deferred but architecture must avoid blocking it.

Potential requirements:

- offline edits;
- deterministic merges;
- rich text;
- movable tree/binder;
- comments and suggestions;
- presence;
- asset synchronization;
- partial project access;
- encryption;
- understandable conflict handling;
- migration across schema versions.

Candidate CRDTs:

- Loro, especially because rich text and movable trees align with Margin;
- Yrs/Yjs, with mature ProseMirror integration;
- Automerge.

Do not add a CRDT merely for future-proofing. Prototype the real editor schema,
binder moves, comments, and offline conflict cases first.

---

## 22. Plugin and Method-Pack Architecture

### 22.1 Goals

Plugins should eventually allow:

- method packs;
- custom diagnostics;
- entity templates and fields;
- import/export adapters;
- reports;
- commands;
- controlled UI contributions;
- custom calendars and world rules.

### 22.2 Security boundary

Third-party code must not receive unrestricted filesystem or network access by
default. Candidate runtime:

- WebAssembly through Wasmtime or Extism;
- declared capabilities;
- versioned host API;
- resource limits;
- deterministic execution where practical;
- signed packages and transparent permissions.

### 22.3 Method-pack manifest

A conceptual manifest may include:

```text
id
name
version
author
license
supported Margin API version
entity schemas
scene fields
templates
rules
reports
commands
localization
permissions
```

### 22.4 Pack governance

Margin must distinguish:

- official packs;
- verified third-party packs;
- local project packs;
- untrusted packs.

Method packs based on books or proprietary systems require rights review. Do
not copy protected explanatory text or branded templates without permission.

---

## 23. AI Editorial Layer

### 23.1 Positioning

The tagline is not merely marketing:

> Let AI be your editor, not your writer.

The primary AI jobs are:

- identify;
- compare;
- trace;
- question;
- explain;
- summarize evidence;
- propose structured facts;
- help investigate inconsistencies;
- support revision.

### 23.2 Agent tools

An editorial agent may need tools such as:

- find entity;
- retrieve source spans;
- query facts with valid time and modality;
- inspect scene metadata;
- traverse relationship paths;
- solve temporal constraints;
- calculate travel feasibility;
- list unresolved extractions;
- create a proposed fact;
- create a proposed diagnostic explanation;
- never directly change canon without an explicit application command and user
  review.

### 23.3 Grounding

Every substantive answer should cite project evidence. For example:

> "Has Mira visited Veld?"

The answer should distinguish:

- explicit scene presence;
- statement by a narrator or character;
- inferred travel;
- unknown;
- conflicting evidence.

### 23.4 Privacy

- local inference should be offered for sensitive workflows;
- cloud requests require clear provider and data-policy disclosure;
- unpublished manuscripts must not be used for model training by default;
- secrets and provider keys must use operating-system secure storage;
- redact or scope context sent to providers;
- maintain a request audit view where practical;
- allow complete AI disablement.

### 23.5 Evaluation

AI features require project-specific evaluation sets:

- extraction precision and recall;
- contradiction false-positive rate;
- citation correctness;
- unsupported-claim rate;
- modality handling;
- long-context retrieval quality;
- multilingual behavior;
- latency and cost;
- author trust and action rate.

---

## 24. Security, Privacy, and Trust

### 24.1 Threat model categories

- loss or corruption of unpublished manuscripts;
- accidental cloud upload;
- malicious plugin;
- compromised sync account;
- unauthorized collaborator;
- path traversal through assets/imports;
- unsafe document parsing;
- oversized or decompression-bomb imports;
- prompt injection in imported research for AI features;
- remote content tracking;
- secret leakage in logs;
- unsafe HTML rendering;
- vulnerable native dependencies.

### 24.2 Baseline requirements

- local data by default;
- atomic saves and tested recovery;
- versioned migrations with backups;
- path normalization;
- file-type and size validation;
- sandboxed previews;
- content security policy;
- no secrets in project files or logs;
- dependency auditing;
- least-privilege Tauri capabilities;
- encrypted transport for hosted services;
- authentication and authorization before public multi-user deployment;
- export and deletion tools;
- clear data retention policy;
- opt-in telemetry only after explicit product decision.

### 24.3 Backups

Margin should provide:

- automatic local snapshots;
- retention policy visible to users;
- restore preview;
- integrity verification;
- backup location choice;
- project archive export;
- migration backup;
- warnings when no successful backup exists;
- no implication that Git is an appropriate manuscript backup for all users.

---

## 25. Accessibility and Internationalization

Requirements:

- WCAG-aligned keyboard and screen-reader behavior;
- visible focus;
- semantic controls;
- resizable panels;
- high-contrast support;
- reduced motion;
- color-independent diagnostic severity;
- localized UI;
- pluralization and locale-aware formatting;
- right-to-left UI strategy;
- bidirectional document content;
- IME correctness;
- Unicode grapheme-aware selection and counts;
- script-appropriate word boundaries;
- font fallback;
- locale and project-specific dictionaries;
- custom calendars and era names;
- accessible alternatives for graph, map, canvas, and timeline.

Test with Bengali, Arabic, Devanagari, CJK, mixed-direction text, emoji,
combining marks, and long names before claiming international readiness.

---

## 26. Observability and Background Work

### 26.1 Logging

Use structured tracing. Logs should include:

- request/job ID;
- project ID only where policy allows;
- operation;
- duration;
- result category;
- engine stage;
- cache hit/miss where useful.

Never log manuscript text by default.

### 26.2 Background jobs

Potential jobs:

- index build;
- extraction;
- embeddings;
- OCR/import conversion;
- export;
- static site generation;
- map processing;
- sync;
- backups;
- integrity checks.

Each job should expose:

- status;
- progress;
- cancellation;
- retry policy;
- error explanation;
- revision/input identity;
- stale-result handling.

### 26.3 Metrics

Local developer metrics may include:

- edit-to-diagnostic latency;
- index rebuild time;
- extraction throughput;
- memory;
- project open time;
- save duration;
- database query latency;
- diagnostic count and suppression rate in test corpora.

Production telemetry requires an explicit privacy decision.

---

## 27. Testing Strategy

### 27.1 Unit tests

- domain invariants;
- validation;
- fact and modality semantics;
- calendar conversions;
- temporal constraints;
- spatial calculations;
- rule behavior;
- serialization;
- source-anchor updates.

### 27.2 Property tests

Useful for:

- calendar round trips;
- interval relations;
- graph invariants;
- project serialization;
- migration idempotence;
- route bounds;
- undo/redo operations.

### 27.3 Integration tests

- application with SQLite;
- migration from every supported version;
- HTTP and Tauri transport parity;
- export/import round trips;
- crash-recovery simulation;
- plugin sandbox permissions;
- AI provider redaction boundaries.

### 27.4 Golden tests

- compiled DOCX/PDF/EPUB structure;
- Fountain/FDX conversion;
- deterministic diagnostics;
- map rendering where stable;
- editor schema transformations.

### 27.5 End-to-end tests

- create project;
- create hierarchy;
- edit and reopen;
- run diagnostics;
- resolve/suppress;
- export;
- restore backup;
- offline desktop startup;
- browser/API workflow.

### 27.6 Performance tests

Create synthetic and licensed/public-domain fixtures at:

- 10,000 words;
- 100,000 words;
- 250,000 words;
- multi-book series scale;
- thousands of entities and events;
- dense relationships;
- long timelines;
- large maps and assets.

### 27.7 Corpus policy

Do not commit copyrighted book text. Tests should use:

- generated fixtures;
- public-domain texts;
- explicitly licensed excerpts;
- transformed metadata that does not reproduce protected text;
- small hand-authored examples.

---

## 28. Build, Run, and Verification

### 28.1 Prerequisites

- Rust 1.85 or newer;
- Node.js 22 or newer;
- pnpm 11;
- WebView2 and Tauri prerequisites on Windows;
- Docker only for container workflows.

### 28.2 Install

```powershell
cd e:\margin
pnpm install
```

### 28.3 Browser development

Terminal 1:

```powershell
pnpm dev:api
```

Terminal 2:

```powershell
pnpm dev
```

Addresses:

- API: `http://127.0.0.1:4317`
- Vite: `http://localhost:1420`

### 28.4 Desktop development

```powershell
pnpm dev:desktop
```

The desktop database is placed in the operating system application-data
directory.

### 28.5 Containers

```powershell
docker compose up --build
```

The web container is exposed on port 8080 and proxies API requests internally.

### 28.6 Quality commands

```powershell
pnpm check
pnpm lint
pnpm test
pnpm format:check
```

### 28.7 Verification state at handoff

Verified:

- frontend dependencies installed;
- frontend type-check passed;
- frontend lint passed;
- frontend production build passed;
- frontend package tests passed;
- Rust workspace check passed, including desktop compilation;
- strict Clippy passed;
- core/API/database tests passed;
- API health and project-list read endpoints passed;
- IDE linter reported no errors.

An all-workspace Rust test build was intentionally replaced by core/API tests
plus desktop compilation because compiling Tauri's Windows dependency graph as
test targets exhausted the temporary build volume. The CI workflow follows the
same separation.

---

## 29. CI/CD

The GitHub Actions scaffold has:

### Frontend job

- pnpm install from lockfile;
- formatting check;
- TypeScript check;
- lint;
- tests;
- production build.

### Rust job

- formatting;
- strict Clippy;
- tests for domain, application, storage, diagnostics, and API.

### Desktop job

- Windows runner;
- frontend build;
- desktop Cargo check.

Before release automation is added, decide:

- repository host;
- branch policy;
- license;
- signing identities;
- desktop update channel;
- artifact retention;
- SBOM and provenance requirements;
- release cadence;
- web hosting;
- database compatibility policy.

---

## 30. Recommended Technology Direction

These are current recommendations, not all installed dependencies.

### 30.1 Application shell and UI

- Tauri 2 for desktop;
- React/TypeScript for shared UI;
- ProseMirror or Tiptap as the leading editor candidate;
- WebGPU-oriented rendering for advanced canvases, with fallbacks;
- avoid implementing a Word-class rich-text engine natively in Rust during the
  first product phases.

### 30.2 Rust core

Potential crates:

- `salsa` for incremental queries;
- `petgraph` for graph algorithms;
- `ascent`, `datafrog`, or `crepe` for Datalog experiments;
- `rusqlite` or existing `sqlx` depending measured needs;
- `redb` for embedded key/value cases;
- `serde`;
- `ropey`;
- `unicode-segmentation`;
- `icu4x`;
- `tantivy`;
- `fst`;
- `aho-corasick`;
- `strsim`;
- `rstar`;
- `geo`, `geojson`, and optional `proj`;
- `vello`, `kurbo`, `lyon`, `tiny-skia`, and `wgpu`;
- `typst` as a PDF/layout candidate;
- `docx-rs`;
- `epub-builder`;
- `comrak` or `pulldown-cmark`;
- `hayagriva` and CSL-related libraries;
- `wasmtime` or `extism`;
- `ort`, `candle`, or other measured local inference runtimes;
- vector indexes only when a concrete retrieval benchmark requires them.

### 30.3 Collaboration candidates

- Loro;
- Yrs/Yjs;
- Automerge.

### 30.4 Contract generation

Evaluate:

- OpenAPI generation;
- `specta`;
- `ts-rs`;
- a dedicated schema package.

One format must become authoritative. Do not maintain large duplicate Rust and
TypeScript contract sets manually.

---

## 31. Open-Source Projects to Study

Study architecture and interaction patterns; do not copy code without license
review.

- **rust-analyzer:** incremental query and diagnostic architecture.
- **Zed:** high-performance editing structures and GPUI techniques.
- **Typst:** incremental document compilation and Rust layout.
- **AppFlowy:** Rust-backed document application architecture.
- **AFFiNE:** local-first workspace and CRDT experience.
- **Rnote:** Rust stylus and freehand implementation.
- **novelWriter:** open-source novel workflow and project format.
- **Manuskript:** novel-planning feature scope.
- **bibisco:** character and location questionnaires.
- **Fantasia Archive:** worldbuilding taxonomy.
- **Logseq:** block references and linked knowledge.
- **Obsidian ecosystem:** local files, plugins, graphs, and writer trust.
- **Excalidraw:** approachable freeform diagram interaction.
- **tldraw:** extensible canvas architecture.
- **Azgaar's Fantasy Map Generator:** fantasy map capabilities.
- **Martin O'Leary's map generator:** procedural techniques.
- **Beat:** screenplay workflow.
- **Trelby:** screenplay format and reports.
- **Afterwriting:** Fountain processing.
- **BookNLP:** literary entity, coreference, quotation, and event extraction.
- **LitBank:** literary NLP annotation.
- **Inform 7:** natural-language world modeling and rules.
- **Wikidata:** statement, qualifier, rank, and provenance model.

---

## 32. Research Program

### 32.1 Why book research is insufficient by itself

Books describe stated methods. Actual working behavior may differ. Product
research must combine:

- craft books;
- author essays and interviews;
- direct contextual interviews;
- observation of active projects;
- artifact studies of outlines, cards, spreadsheets, maps, notebooks, and
  revision files;
- competitor testing;
- prototype usability studies;
- longitudinal use during real drafting and revision.

### 32.2 Interview sample

Aim for at least 20–30 contextual interviews across:

- discovery and outline-heavy novelists;
- science-fiction and fantasy series writers;
- romance;
- mystery and thriller;
- historical fiction;
- literary fiction;
- horror;
- children's and young-adult;
- screenwriters;
- playwrights;
- comics and graphic novels;
- memoir;
- narrative non-fiction;
- academic or research-heavy trade writing;
- editors;
- co-authors and writing rooms;
- multilingual writers;
- disabled writers using assistive technology.

### 32.3 Interview topics

- project setup;
- first idea capture;
- outline behavior;
- drafting;
- revision;
- world and character notes;
- chronology;
- maps;
- source management;
- continuity mistakes;
- collaboration;
- backup anxiety;
- export/submission;
- abandoned tools;
- privacy;
- AI attitudes;
- accessibility;
- terminology;
- moments where information is duplicated;
- moments where the writer stops trusting software.

### 32.4 Artifact coding

Tag findings by:

- archetype;
- phase;
- artifact;
- pain;
- workaround;
- frequency;
- severity;
- project scale;
- genre;
- evidence source;
- contradiction with other methods;
- product implication;
- confidence.

### 32.5 Research knowledge base

The separate ingestion project at `e:\margin-research-ingest` can inventory and
convert lawfully acquired PDF, EPUB, Markdown, and text into a local,
citation-preserving corpus.

Research policy:

- do not commit source books;
- record rights and provenance;
- preserve page/chapter locators;
- create human-authored synthesis notes;
- link every synthesis claim back to a source span;
- separate quotations from paraphrases;
- avoid treating embeddings as the sole index;
- retain bibliographic metadata;
- deduplicate editions using hashes and identifiers;
- verify quotations against source;
- comply with copyright and source terms.

Suggested knowledge layers:

1. source metadata;
2. source text segmented by page/chapter;
3. summaries and claim notes;
4. concepts and process archetypes;
5. product implications;
6. decisions linked to evidence;
7. embeddings as secondary retrieval.

---

## 33. Research Canon

The following bibliography is a starting research map, not an endorsement of
every prescription. Bibliographic details should be verified before formal
citation.

### 33.1 General craft and process philosophy

- Stephen King, *On Writing*.
- Anne Lamott, *Bird by Bird*.
- Ursula K. Le Guin, *Steering the Craft*.
- John Gardner, *The Art of Fiction*.
- John Gardner, *On Becoming a Novelist*.
- E.M. Forster, *Aspects of the Novel*.
- Francine Prose, *Reading Like a Writer*.
- George Saunders, *A Swim in a Pond in the Rain*.
- Ray Bradbury, *Zen in the Art of Writing*.
- Annie Dillard, *The Writing Life*.
- Dorothea Brande, *Becoming a Writer*.
- Natalie Goldberg, *Writing Down the Bones*.
- Steven Pressfield, *The War of Art*.
- Matthew Salesses, *Craft in the Real World*.
- Jane Alison, *Meander, Spiral, Explode*.
- James Wood, *How Fiction Works*.
- Milan Kundera, *The Art of the Novel*.
- Italo Calvino, *Six Memos for the Next Millennium*.
- Margaret Atwood, *Negotiating with the Dead*.
- Flannery O'Connor, *Mystery and Manners*.
- Haruki Murakami, *Novelist as a Vocation*.
- Umberto Eco, *Confessions of a Young Novelist*.
- Umberto Eco, *Postscript to The Name of the Rose*.
- Vladimir Nabokov, *Lectures on Literature*.
- Mason Currey, *Daily Rituals*.
- *The Paris Review Interviews / Writers at Work* series.
- Liu Xie, *The Literary Mind and the Carving of Dragons*.
- Aristotle, *Poetics*.
- Gustav Freytag, *Technique of the Drama*.

### 33.2 Plot and structure

- Ronald B. Tobias, *20 Master Plots: And How to Build Them*.
- Christopher Vogler, *The Writer's Journey*.
- Gail Carriger, *The Heroine's Journey*.
- Maureen Murdock, *The Heroine's Journey*.
- Robert McKee, *Story*.
- John Truby, *The Anatomy of Story*.
- Jessica Brody, *Save the Cat! Writes a Novel*.
- Blake Snyder, *Save the Cat!*.
- Shawn Coyne, *The Story Grid*.
- Randy Ingermanson, *How to Write a Novel Using the Snowflake Method*.
- K.M. Weiland, *Structuring Your Novel*.
- K.M. Weiland, *Outlining Your Novel*.
- Larry Brooks, *Story Engineering*.
- James Scott Bell, *Plot & Structure*.
- James Scott Bell, *Write Your Novel from the Middle*.
- Jack Bickham, *Scene & Structure*.
- Dwight Swain, *Techniques of the Selling Writer*.
- John Yorke, *Into the Woods*.
- Christopher Booker, *The Seven Basic Plots*.
- Joseph Campbell, *The Hero with a Thousand Faces*.
- Vladimir Propp, *Morphology of the Folktale*.
- Libbie Hawker, *Take Off Your Pants!*.
- Gwen Hayes, *Romancing the Beat*.
- Donald Maass, *The Emotional Craft of Fiction*.
- Donald Maass, *Writing the Breakout Novel*.
- Lisa Cron, *Wired for Story*.
- Lisa Cron, *Story Genius*.
- John McPhee, *Draft No. 4*.
- Scholarship on Kishōtenketsu and non-conflict-centered structure.

### 33.3 Character

- Orson Scott Card, *Characters and Viewpoint*.
- Brandilyn Collins, *Getting Into Character*.
- K.M. Weiland, *Creating Character Arcs*.
- Nancy Kress, *Characters, Emotion & Viewpoint*.
- Linda Seger, *Creating Unforgettable Characters*.
- Robert McKee, *Character* and related dialogue work.
- Angela Ackerman and Becca Puglisi, *The Emotion Thesaurus*.
- Angela Ackerman and Becca Puglisi, *The Positive Trait Thesaurus*.
- Angela Ackerman and Becca Puglisi, *The Negative Trait Thesaurus*.
- Konstantin Stanislavski, *An Actor Prepares*.

### 33.4 Setting, worldbuilding, maps, and language

- Jeff VanderMeer, *Wonderbook*.
- A. Trevena, *30 Days of Worldbuilding*.
- David J. Peterson, *The Art of Language Invention*.
- Mark Rosenfelder, *The Planet Construction Kit*.
- Mark Rosenfelder, *The Language Construction Kit*.
- Mark Rosenfelder, *Advanced Language Construction*.
- Wolfgang Baur, editor, *The Kobold Guide to Worldbuilding*.
- Stephen L. Gillett, *World-Building*.
- Diana Wynne Jones, *The Tough Guide to Fantasyland*.
- Brandon Sanderson's Laws of Magic essays and BYU lectures.
- Randy Ellefson, *The Art of World Building* series.
- Jared Blando, *How to Draw Fantasy Art and RPG Maps*.
- Angela Ackerman and Becca Puglisi, *The Urban Setting Thesaurus*.
- Angela Ackerman and Becca Puglisi, *The Rural Setting Thesaurus*.

### 33.5 Narrative theory and point of view

- Alida Winternheimer, *The Story Works Guide to Writing Point of View*.
- Christopher Castellani, *The Art of Perspective*.
- Gérard Genette, *Narrative Discourse*.
- Seymour Chatman, *Story and Discourse*.
- Wayne C. Booth, *The Rhetoric of Fiction*.
- Mieke Bal, *Narratology*.
- H. Porter Abbott, *The Cambridge Introduction to Narrative*.
- Dorrit Cohn, *Transparent Minds*.
- Joan Silber, *The Art of Time in Fiction*.
- Charles Baxter, *The Art of Subtext*.
- Mark Doty, *The Art of Description*.
- Ellen Bryant Voigt, *The Art of Syntax*.
- Stacey D'Erasmo, *The Art of Intimacy*.

### 33.6 Genre

General and thriller:

- Benjamin Percy, *Thrill Me*.
- James N. Frey, *How to Write a Damn Good Thriller*.

Science fiction, fantasy, and horror:

- Crawford Kilian, *Writing Speculative Fiction*.
- Orson Scott Card, *How to Write Science Fiction & Fantasy*.
- Ursula K. Le Guin, *The Language of the Night*.
- David Gerrold, *Worlds of Wonder*.
- Tim Waggoner, *Writing in the Dark*.
- Mort Castle, editor, *On Writing Horror*.
- Stephen King, *Danse Macabre*.

Historical fiction:

- Historical Novel Society, *Crafting Stories From the Past*.
- Emma Darwin, *Get Started in Writing Historical Fiction*.
- Georg Lukács, *The Historical Novel*.

Western:

- Wayword Writers, *Crafting the Modern Western*.

Mystery:

- Patricia Highsmith, *Plotting and Writing Suspense Fiction*.
- James N. Frey, *How to Write a Damn Good Mystery*.
- Hallie Ephron, *Writing and Selling Your Mystery Novel*.
- Ronald Knox's Decalogue.
- S.S. Van Dine's Twenty Rules.

Romance:

- Leigh Michaels, *On Writing Romance*.
- Jayne Ann Krentz, editor, *Dangerous Men and Adventurous Women*.

Children's and young adult:

- Cheryl Klein, *Second Sight*.
- Mary Kole, *Writing Irresistible Kidlit*.
- Regina Brooks, *Writing Great Books for Young Adults*.

Comics and graphic novels:

- Scott McCloud, *Understanding Comics*.
- Scott McCloud, *Making Comics*.
- Will Eisner, *Comics and Sequential Art*.
- Will Eisner, *Graphic Storytelling and Visual Narrative*.
- Dennis O'Neil, *The DC Comics Guide to Writing Comics*.

### 33.7 Screenwriting, playwriting, and storyboarding

- Syd Field, *Screenplay*.
- David Trottier, *The Screenwriter's Bible*.
- Christopher Riley, *The Hollywood Standard*.
- Paul Gulino, *Screenwriting: The Sequence Approach*.
- Linda Seger, *Making a Good Script Great*.
- William Goldman, *Adventures in the Screen Trade*.
- Karl Iglesias, *Writing for Emotional Impact*.
- Dara Marks, *Inside Story*.
- Stuart Spencer, *The Playwright's Guidebook*.
- David Ball, *Backwards and Forwards*.
- Steven Katz, *Film Directing Shot by Shot*.
- Giuseppe Cristiano, *The Storyboard Artist*.
- Fountain plain-text screenplay specification.

### 33.8 Non-fiction, memoir, and research writing

- William Zinsser, *On Writing Well*.
- Roy Peter Clark, *Writing Tools*.
- Jack Hart, *Storycraft*.
- Vivian Gornick, *The Situation and the Story*.
- Mary Karr, *The Art of Memoir*.
- Susan Rabiner and Alfred Fortunato, *Thinking Like Your Editor*.
- Robert Caro, *Working*.
- Wayne Booth, Gregory Colomb, and Joseph Williams, *The Craft of Research*.
- Gerald Graff and Cathy Birkenstein, *They Say / I Say*.
- Sönke Ahrens, *How to Take Smart Notes*.

### 33.9 Revision, editing, and style

- Renni Browne and Dave King, *Self-Editing for Fiction Writers*.
- *The Chicago Manual of Style*.
- Benjamin Dreyer, *Dreyer's English*.
- Carol Fisher Saller, *The Subversive Copy Editor*.
- Joseph M. Williams, *Style: Lessons in Clarity and Grace*.
- Steven Pinker, *The Sense of Style*.
- Verlyn Klinkenborg, *Several Short Sentences About Writing*.
- Noah Lukeman, *The First Five Pages*.
- James Scott Bell, *Revision and Self-Editing*.

### 33.10 Cognitive science of writing and story

- Linda Flower and John Hayes, "A Cognitive Process Theory of Writing" (1981).
- Carl Bereiter and Marlene Scardamalia, *The Psychology of Written
  Composition*.
- Will Storr, *The Science of Storytelling*.
- Jonathan Gottschall, *The Storytelling Animal*.

### 33.11 Interactive and procedural narrative

- Janet Murray, *Hamlet on the Holodeck*.
- Tanya Short and Tarn Adams, editors, *Procedural Storytelling in Game
  Design*.
- Inform 7 documentation.

### 33.12 Technical papers and datasets

- BookNLP research and documentation.
- LitBank.
- TimeML and TimeBank.
- James Allen, "Maintaining Knowledge about Temporal Intervals" (1983).
- Rina Dechter, Itay Meiri, and Judea Pearl, "Temporal Constraint Networks"
  (1991).
- Wikidata data model documentation.
- current GraphRAG research, evaluated at implementation time.
- current plot-hole and flawed-fiction benchmarks, verified when the AI phase
  begins.

---

## 34. Competitive Context

Competitor categories and their lessons:

### Scrivener

Strengths:

- manuscript hierarchy;
- corkboard;
- research alongside prose;
- compile flexibility;
- trusted offline workflow.

Gap Margin targets:

- structured connected canon;
- live consistency diagnostics;
- time, space, and epistemic reasoning.

### Campfire and World Anvil

Strengths:

- structured lore;
- visual relationships;
- worldbuilding templates;
- timelines and maps.

Gap Margin targets:

- deeper manuscript editing;
- local-first trust;
- incremental diagnostics grounded in prose;
- one coherent editor/engine.

### Obsidian

Strengths:

- local Markdown;
- links;
- extensibility;
- user trust;
- graph.

Gap Margin targets:

- purpose-built manuscript semantics;
- structured story time and canon;
- writer-focused diagnostics and exports.

### Plottr and NovelPad

Strengths:

- visual planning;
- approachable cards and timelines.

Gap Margin targets:

- rich knowledge graph;
- manuscript-aware consistency engine;
- extensible methods without one prescriptive system.

### ProWritingAid, AutoCrit, and grammar tools

Strengths:

- prose-level reports;
- editorial feedback.

Gap Margin targets:

- project/world model;
- cross-document canon;
- temporal, spatial, and knowledge reasoning.

### Generative writing products

Strengths:

- ideation and prose generation;
- accessible AI interaction.

Margin differentiation:

- authorial ownership;
- evidence-backed editorial intelligence;
- privacy and local-first operation;
- graph-grounded consistency rather than generic continuation.

Competitive claims must be revalidated before marketing.

---

## 35. Delivery Roadmap

Dates are intentionally absent. Estimate after spikes and user research.

### Phase 0: Research and engine spike

Goals:

- validate the core diagnostic proposition;
- validate performance and false-positive controls;
- avoid building broad UI before proving the engine.

Deliverables:

- documented research ontology for process findings;
- source-span and stable-ID prototype;
- entity and fact model prototype;
- explicit and deterministic extraction;
- Aho-Corasick mention index;
- incremental query spike;
- three diagnostics:
  - attribute contradiction;
  - impossible lifecycle appearance;
  - name drift;
- public-domain or generated evaluation corpus;
- latency and false-positive report;
- author confirmation/rejection flow prototype.

Exit criteria:

- paragraph-level updates meet a credible latency budget;
- diagnostics retain evidence after common edits;
- test authors understand and trust the proposed-fact flow;
- false-positive policy is documented.

### Phase 1: Margin Draft MVP

Target:

- novelists with continuity pain, especially series and speculative-fiction
  writers.

Deliverables:

- production project format and backup;
- manuscript editor;
- binder/outliner/corkboard;
- wiki templates and custom fields;
- explicit links and aliases;
- graph view;
- dual-clock timeline and custom calendars;
- Tier 0 and conservative Tier 1 extraction;
- proposal inbox;
- Problems panel;
- suppression and explanation workflow;
- DOCX, PDF, EPUB, and Markdown output;
- desktop-first release;
- import from common manuscript formats where feasible.

Exit criteria:

- users can complete a real drafting/revision cycle;
- no data loss in recovery tests;
- exports are usable for editorial workflows;
- continuity diagnostics provide measurable value without being disabled.

### Phase 2: Space, visual planning, and scripts

Deliverables:

- map layers;
- routes and travel checks;
- idea canvas;
- stylus sketches;
- storyboard;
- screenplay editor;
- Fountain/FDX;
- initial collaboration prototype;
- web version beyond local development.

### Phase 3: Breadth and ecosystem

Deliverables:

- non-fiction sources and citations;
- claim-to-evidence links;
- local ML extraction;
- method-pack and plugin SDK;
- package verification/permissions;
- static series-bible export;
- advanced reports;
- mature collaboration and sync if validated.

### Phase 4: AI editor

Deliverables:

- graph-grounded editorial agent;
- source-cited project questions;
- subtle contradiction proposals;
- investigation workflows;
- local-model option;
- user-selected cloud providers;
- privacy dashboard;
- evaluation harness;
- no silent canon changes.

---

## 36. Immediate Engineering Backlog

The next team should not begin with maps or AI. Recommended order:

### 36.1 Repository baseline

- review all generated files;
- choose product license with legal/business input;
- create first commit;
- configure remote and branch protection;
- add contribution and security policies;
- pin supported toolchain versions;
- add dependency update automation.

### 36.2 Contract generation

- select the authoritative API schema path;
- generate TypeScript contracts;
- add drift detection to CI;
- standardize error codes and date/ID serialization.

### 36.3 Persistence hardening

- add project/document update and delete;
- add optimistic concurrency with revisions;
- define transaction boundaries;
- add migration tests;
- add database integrity and recovery tests;
- decide attachment storage;
- prototype portable project archive;
- add automatic backup before migration.

### 36.4 Editor spike

- evaluate ProseMirror directly versus Tiptap;
- define minimal document schema;
- define stable block IDs;
- test comments and change tracking;
- test copy/paste and DOCX import assumptions;
- test IME, bidi, screen readers, and large-document behavior;
- measure Tauri webview behavior on target platforms.

### 36.5 Canon spike

- define entity, predicate, fact, modality, status, and source-anchor types;
- write invariants and serialization tests;
- build deterministic mention index;
- implement proposed-fact review;
- implement one attribute contradiction end to end;
- connect diagnostic to editor evidence.

### 36.6 Product research

- establish research repository taxonomy;
- complete rights/provenance metadata for sources;
- create interview guide;
- recruit diverse working writers;
- code workflows into archetypes;
- link product requirements to evidence.

---

## 37. Major Risks and Mitigations

### False positives

Risk:

Users disable the Problems panel.

Mitigations:

- conservative rules;
- proposed facts;
- modality;
- confidence thresholds;
- clear evidence;
- easy correction;
- scoped suppression;
- measure precision before adding breadth.

### Editor scope

Risk:

Word-class ambition consumes the project.

Mitigations:

- writer-focused subset;
- web editor foundation;
- staged pagination and DOCX work;
- prototype difficult international input early;
- avoid native editor rewrite before proven need.

### Product breadth

Risk:

Maps, drawing, GIS, conlangs, screenplay, collaboration, and citations become
separate unfinished products.

Mitigations:

- canon engine first;
- each module must feed the shared model;
- staged delivery;
- explicit exit criteria;
- plugins/method packs for specialized breadth.

### Data loss and lock-in fear

Risk:

Writers do not trust the tool.

Mitigations:

- local-first;
- backups;
- durable export;
- visible save state;
- recovery testing;
- migration transparency;
- no proprietary-only dead end.

### Prescriptive craft ideology

Risk:

Margin alienates writers and encodes cultural bias.

Mitigations:

- optional method packs;
- craft hints separated from canon errors;
- diverse research traditions;
- editable rules;
- transparent sources;
- advisory language.

### AI privacy and hallucination

Risk:

Unpublished text leaks or unsupported claims become editorial authority.

Mitigations:

- local option;
- explicit providers;
- grounded tool access;
- citations;
- proposed status;
- evaluation;
- no silent writes;
- complete disablement.

### Cross-platform webview differences

Risk:

Editor, WebGPU, IME, or accessibility behavior differs by platform.

Mitigations:

- platform test matrix;
- fallback rendering;
- early Linux/macOS/Windows prototypes;
- avoid depending on one browser-only feature without fallback.

### Research copyright

Risk:

Source books or proprietary methods are redistributed or copied into packs.

Mitigations:

- source files remain outside Git;
- record provenance and rights;
- paraphrase research findings;
- license packs and examples appropriately;
- legal review before distribution.

---

## 38. Decisions Already Made

- Product name: **Margin**.
- Positioning: **Let AI be your editor, not your writer.**
- Local-first desktop capability is mandatory.
- Web and desktop should share a frontend and application layer.
- Rust owns the core, backend, and performance-sensitive engine.
- Tauri is the current desktop shell.
- React/TypeScript is the current shared UI.
- Axum is the current web API.
- SQLite is the current scaffold database.
- Domain/application layers remain transport- and storage-independent.
- Writing methods are optional packs.
- Canon and craft diagnostics have different authority.
- Provenance and modality are required for serious consistency checking.
- Incremental architecture is required.
- Research books and corpora are not committed.
- AI is a later editorial layer, not the foundation.

---

## 39. Decisions Explicitly Deferred

- final product license;
- commercial model;
- repository hosting;
- final project file format;
- authoritative API contract generator;
- ProseMirror versus Tiptap;
- exact editor schema;
- exact fact and ontology schema;
- Salsa adoption and query boundaries;
- Datalog engine;
- graph database versus application-maintained graph indexes;
- CRDT;
- plugin runtime;
- native/WebGPU canvas stack;
- GIS scope;
- local inference runtime and models;
- cloud provider policy;
- telemetry;
- authentication provider;
- web hosting topology;
- release/updater infrastructure;
- branding and final visual design;
- final method-pack catalogue;
- marketplace.

Deferred does not mean unimportant. Each requires evidence or stakeholder input.

---

## 40. Definition of Done by Capability

### A persistence feature is done when

- data survives restart;
- migration is tested;
- backup/recovery is defined;
- errors are actionable;
- concurrency/revision behavior is defined;
- API and desktop behavior match;
- export consequences are known.

### A diagnostic is done when

- inputs and semantics are documented;
- evidence is displayed;
- false-positive tests exist;
- severity is justified;
- revision/staleness is handled;
- quick actions are safe;
- suppression works;
- performance is measured;
- the author can understand why it fired.

### An editor feature is done when

- keyboard, mouse, clipboard, undo, redo, save, and recovery work;
- IME and Unicode behavior are tested;
- accessibility is tested;
- serialization and migration are tested;
- large-document behavior is measured;
- export behavior is documented.

### An AI feature is done when

- data flow is disclosed;
- grounding and citations are tested;
- user approval boundaries are explicit;
- failure and cancellation work;
- local/disabled behavior is valid;
- evaluation results meet a declared threshold;
- manuscript text is not logged or trained on by default.

---

## 41. Glossary

- **Canon:** Information accepted by the author as authoritative within a
  project or series scope.
- **Claim:** A statement extracted or entered from a source; it may or may not
  be accepted as canon.
- **Fact:** A structured subject-predicate-object/value statement with
  qualifiers, provenance, time, modality, and status.
- **Entity:** A stable project object such as a character, place, event, item,
  source, or user-defined type.
- **Predicate:** The typed relationship or attribute connecting a subject to a
  value/object.
- **Modality:** The truth/knowledge context of a claim, such as world truth,
  belief, rumor, lie, or dream.
- **Valid time:** Story-world interval during which a fact applies.
- **Reading order:** Position where the reader encounters material.
- **Story time:** Chronological placement in the represented world.
- **Source span:** Stable link from structured data or a diagnostic to the
  originating document content.
- **Diagnostic:** Evidence-backed engine output with severity, code, locations,
  explanation, and possible actions.
- **Method pack:** Optional schemas, templates, rules, reports, and guidance for
  a writing method or genre workflow.
- **Proposed fact:** Extracted structured information awaiting author review.
- **Suppression:** Recorded decision not to show a diagnostic for a defined
  scope, ideally with a reason.
- **Fabula:** Chronological story events.
- **Syuzhet/discourse:** The order and manner in which events are presented.
- **Local-first:** The local device is fully useful and owns a primary usable
  copy of data; network services are additive.

---

## 42. Final Handoff Guidance

The central implementation discipline is:

> Build one coherent system of evidence, canon, constraints, and author
> decisions. Do not build a pile of disconnected writing widgets.

The first proof should be small:

- one real editor document;
- stable source anchors;
- explicit and extracted entities;
- one useful contradiction;
- one understandable explanation;
- one safe resolution;
- fast incremental updates;
- no loss of author control.

If that loop is trustworthy, the graph, timeline, map, screenplay, citations,
plugins, and AI can become increasingly powerful views and contributors to the
same model.

If that loop is not trustworthy, adding more modules will not create the Margin
described in this handoff.
