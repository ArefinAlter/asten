# The Asten canon engine: design

Version 0.2 · 8 October 2026 (adds the bootstrap importer, §8.5) · Status: proposal for review · Owner: Verne

This document designs the engine that finds continuity errors in a manuscript the way an IDE finds errors in code. It covers the formal model, the algorithms, the probabilistic reasoning layer, the models we would train, the engineering, and the alternative paths we considered. It is written for the engineers who will build it and the people who will decide what to build first.

Related documents: [`architecture.md`](architecture.md) (current layering), [`PROJECT_HANDOFF.md`](PROJECT_HANDOFF.md) (the September planning archive, especially §9–§14), [`BRAND.md`](BRAND.md) (the "black ink, red ink" rule that the diagnostics surface must respect).

---

## 1. Summary

**The thesis.** A manuscript is a program written in a natural language. A compiler turns source text into a structured representation, runs analyses over it, and reports errors with locations and fixes. The canon engine does the same for a story: it turns prose into a language-independent representation of who and what exists, what is true when, and who knows what, then runs analyses that are, almost one for one, the analyses compilers and verifiers already run on code.

**The shape of the system.**

```text
 Dynodoc blocks, events, drafts                         (inputs, git-like)
          │
          ├──▶ Bootstrap importer, optional, run once     (small local generative model, §8.5)
          │      proposes a starting canon for review ──▶ enters the IR as proposed evidence
          ▼
 Frontends: per-language perception                      (models, never trusted blindly)
   segmentation · mentions · aliases · facts · events · time · framing
          │   probabilistic evidence with provenance
          ▼
 Story IR: entities · fluents · events · contexts · constraints   (language-independent)
          │
          ├──▶ Symbolic analyses (exact, deterministic)
          │      names · sorts and value lattices · event calculus · typestate
          │      data flow · temporal network · travel · knowledge flow · invariants
          │
          ├──▶ Probabilistic reasoning (collective, calibrated)
          │      hinge-loss Markov random field · provenance semirings
          │      diagnosis (who is wrong) · abduction (how to fix)
          ▼
 Diagnostics with evidence, explanations, ranked repairs, guarantees
          │
          ▼
 Story language server ──▶ desk · Word pane · Docs sidebar
```

**Design decisions.**

1. **The reasoning core is symbolic and language-independent.** All logic runs on the Story IR. Language only enters through perception frontends, so adding a language means adding or training a frontend, never touching the analyses.
2. **No generative model in the reasoning or in the per-edit loop.** The models that run while the writer works are small discriminative encoders that turn text into typed, calibrated evidence. Their job is to serve the logic, and some are trained *through* it (§11.8). The single exception is the optional **bootstrap importer** (§8.5): a small local generative model that reads an existing draft once and proposes a starting canon for the writer to review. It never writes prose, never settles a fact, and nothing in the reasoning depends on it.
3. **Exact where the author has spoken, probabilistic where we inferred.** Conflicts between settled facts are decided by exact algorithms. Everything involving extracted evidence is scored by a probabilistic layer and only shown above a calibrated threshold with a statistical bound on false alarms (§10.8).
4. **Incremental like a language server.** Every analysis is a memoised query over Dynodoc's stable block IDs and content fingerprints. An edit to one paragraph recomputes only what depends on it.
5. **Every mark explains itself.** Each diagnostic carries the minimal set of statements that conflict, the most likely culprit, and ranked repairs. The engine never edits prose; it only writes in red in the margin.
6. **Local and private.** All of it runs on the writer's machine. No manuscript is used for training, and learning from a writer's decisions stays on their machine.

**What is new compared with today.** The current engine has three hand-written rules over settled facts, raw character offsets as anchors, story time as free text, and a full re-evaluation on every change. This design replaces that with a formal state model (the event calculus) that handles change over time and flashbacks, a temporal network with explanations, typestate lifecycles, knowledge flow, author invariants, a probabilistic ranking layer, incremental evaluation, and a model family designed for the job. The current rules become special cases: CANON-E013 is a functional-fluent violation (§9.3), CANON-E101 is a typestate error (§9.4), and CANON-E201 is replaced by a temporal-network check that does not punish flashbacks (§9.6).

**Recommended path.** Build the deterministic, language-independent core first (it is useful on day one for settled facts, in every language), then the bootstrap importer as an early bridge for writers who arrive with a draft, then perception models, then probabilistic ranking, then the research models. §15 compares the alternatives and §16 gives the phased plan with decision gates.

---

## 2. Requirements and constraints

### 2.1 Product requirements

| # | Requirement | Consequence for the design |
|---|---|---|
| R1 | **The author is the authority.** The engine proposes; the author decides what is canon. | Statuses (settled, proposed, deprecated), suppressions with reasons, blame defaults to the uncertain side, nothing becomes canon silently. |
| R2 | **Canon is optional.** A gardener with no wiki still gets value; an architect gets more. | Gradual canon (§6.4): checks scale with what has been settled. |
| R3 | **Few false alarms.** A checker that cries wolf trains writers to ignore it. | Calibrated scores, conservative thresholds, statistical false-alarm control, may/must severities. |
| R4 | **Every mark shows its evidence.** | Provenance on every fact, minimal conflict sets, explanation chains. |
| R5 | **Language-agnostic.** Bengali, Arabic, Chinese, Thai, low-resource scripts such as Chakma, mixed-script manuscripts. | Language-independent IR and analyses; per-language frontends; graceful degradation where no model exists. |
| R6 | **Local-first and private.** | All inference on device; no telemetry; no training on manuscripts. |
| R7 | **Feels like an IDE.** Fast, incremental, navigable. | Language-server architecture, incremental queries, rename and references. |
| R8 | **Works with the git-like model.** Drafts, merges, history. | Analyses per draft, merge-preview checks, blame and bisect over the event log. |

### 2.2 Engineering constraints

- **Latency.** The planning archive's target stands: deterministic diagnostics within about **50 ms** of a paragraph edit in a 250,000-word project on a mid-range laptop. Model inference runs in background workers and never blocks typing; its results arrive as soon as they are ready, highest priority for visible blocks.
- **Footprint.** Models must run on a laptop CPU. Budget roughly **150–450 MB** of model files in total after 8-bit quantisation, with an optional smaller "lite" set.
- **Determinism.** The same project state and model versions must produce the same diagnostics. Model outputs are cached by (block fingerprint, model version) and the probabilistic solver is run to a fixed tolerance from a deterministic starting point.
- **Rust.** The core is Rust, as today. Model inference uses ONNX Runtime through the `ort` crate. Training happens offline in Python.
- **Generative models only in the importer.** None in the per-edit path, and none as a required step anywhere in the data pipeline (§11.9). Encoders that produce embeddings, labels, spans or scores are allowed. The bootstrap importer (§8.5) is the one exception: optional, local, downloaded on demand, outside the footprint budget above, and its output enters the IR only as proposed evidence.

### 2.3 Non-goals

- Judging prose quality or style. That stays in optional craft packs, outside this engine.
- Detecting every plot hole. Errors that need open-ended causal or social reasoning ("why would she trust him after that?") are out of scope for the exact core and are, at best, weak evidence for the research models (§11.7).
- Writing or rewriting text, including in quick fixes. Repairs change canon, contexts or metadata; the author changes the prose.
- Automatic canon. The importer (§8.5) proposes; it never settles a fact, and it never resolves a contradiction by choosing one side.
- Collaboration semantics beyond what Dynodoc drafts and merges already provide.

---

## 3. Where we start

### 3.1 The Asten engine today

| Area | Today | Limitation the design removes |
|---|---|---|
| Documents | ProseMirror JSON per document, block ids are UUIDv7 in `attrs.id` (`asten-domain/src/pm.rs`). Append-only `document_ops` and `document_snapshots` in SQLite. | Nested nodes and Dynodoc's ULID block ids are not yet unified (§3.3). |
| Facts | `entity_key · predicate · value` with `FactModality`, `FactStatus`, string `valid_from`/`valid_to`, and an optional `SourceAnchor { document_id, start, end }` (`asten-domain/src/lib.rs`). | Anchors are raw offsets into a whole document's plain text, so any edit earlier in the document silently moves them (§6.1). Values are strings, so "dark" contradicts "black". Validity times are strings compared lexically. |
| Time | `TimelineEvent { story_time: Option<String>, reading_order: Option<i64> }`. `story_times_out_of_order` compares lower-cased strings. | "day 9" sorts after "day 10". No durations, no relative constraints, no calendars. |
| Rules | `asten-diagnostics`: a `Rule` trait evaluated over a full `AnalysisContext` snapshot. Rules: CANON-E013, CANON-E101, CANON-E201, CRAFT-H001/H002. | Not incremental; no explanations beyond one message; E201 marks every flashback; author override has no special meaning. |
| Packs | `method_packs` with a JSON manifest; rules gated per pack. | Packs can switch rules but cannot define new ones. |
| Suppressions | Scoped by code, document, entity. | Not tied to the evidence they silenced, so they never go stale. |

### 3.2 The Dynodoc foundation

Asten's document layer follows Dynodoc, Verne's git-like, block-based document system. The standalone engine (`dynodoc-engine` 0.1.0, MIT, extracted from Dynodoc on 2026-09-22) provides most of what an IDE-style analyser needs from its source-control layer:

| Dynodoc concept | What it is | What the canon engine uses it for |
|---|---|---|
| `NodeId` | A ULID, never changed or reused. Each top-level rich-text block is an `item` node whose ProseMirror JSON lives in `current_fields.content`; plain text in `current_fields.label`. | **The primary key of every incremental query.** Anchors, mentions and facts hang off node ids. |
| `RichTextPatch` | Inline edits as `Retain / Delete / Insert` counted in Unicode scalars, with SHA-256 `before` and `after` fingerprints of canonical JSON. | Mapping anchors through edits (§6.1); fingerprints as cache keys (§7). |
| Event log | `EventPayload` (`NodeCreated`, `NodeMoved`, `NodeDeleted`, `RichTextPatched`, …) in an append-only, hash-chained log with a gap-free `seq` per document. | The change feed that drives invalidation. `seq` and `chain_hash` identify a revision exactly. |
| Snapshots | Full materialised state plus a Merkle root over node hashes and the event chain hash; re-snapshot every 500 events. | Fast "has anything in this document changed?" checks; the base for bisect and for materialising any past revision. |
| Drafts | `workspace_draft` + append-only `draft_edit`: personal branches with a merge base. | Per-draft analysis; canon is checked per branch. |
| Merge and diff | State-based three-way merge per (node, field); disjoint inline edits merge automatically; `diff(from, to)` yields semantic ops, with Myers diff inside blocks. | **Merge preview with continuity checks** (§12.5). |
| `integrity::validate_integrity` | Structural checks (dangling references, type mismatches, cycles) that gate deploy and merge in Dynodoc's questionnaire domain. | The precedent for diagnostics as a merge gate. The canon engine is the narrative version of this hook. |

Three facts about Dynodoc shape the plan:

1. **dynodoc-engine stores everything in PostgreSQL 16.** Asten is local-first on SQLite. The pure parts of the engine (materialiser, rich-text patches and fingerprints, merge, diff, hashing) appear to be storage-independent and can be reused directly. The log append and verification code is written against PostgreSQL and needs a SQLite adapter behind a storage trait. This should be confirmed in a short spike before Phase 0 (§16).
2. **Only top-level blocks have stable ids.** Children inside a block (list items, table cells) are addressed by paths relative to the block. The canon engine treats the top-level block as its unit of identity and caching, which matches how prose is written: a paragraph is the natural unit.
3. **There is no incremental computation layer, no blame algorithm, and no CRDT.** The first two are built here (§7, §12.5). The third is not needed by this design.

### 3.3 Unifying the two block models

Asten (UUIDv7 in `attrs.id`) and Dynodoc (ULID in `blockId`, one `item` node per block) differ only in naming and id format. The engine does not care which id scheme wins, only that ids are **stable, unique, never reused, and assigned once**. The recommendation is to adopt Dynodoc's scheme when Asten's storage moves onto the Dynodoc event model, and to give the engine a single `BlockId` newtype so the change is mechanical.

---

## 4. The compiler analogy, made precise

The analogy is useful only if it is exact enough to borrow algorithms from. This table is the map the rest of the document follows.

| Compiler or IDE concept | Canon engine counterpart | Section |
|---|---|---|
| Source files and edits | Dynodoc blocks and events | §3.2, §6.1 |
| Lexer | Unicode segmentation (graphemes, words, sentences) per script | §8.1 |
| Parser to syntax tree | Perception frontends to Story IR | §8 |
| Intermediate representation (LLVM IR) | Story IR, language-independent | §6.2 |
| Symbol table, name resolution | Cast, aliases, mention linking | §9.1 |
| Undeclared identifier, use before definition | Unknown name, used before introduction | §9.1 |
| Types and type checking | Sorts, predicate signatures, value lattices, units | §9.2 |
| Gradual typing and blame | Gradual canon: settled vs proposed facts | §6.4 |
| Reaching definitions, constant propagation | Which facts hold at a scene; change events | §9.3, §9.5 |
| Typestate, use-after-free | Lifecycles; acting after death | §9.4 |
| May and must analyses | Warnings and errors | §9.3 |
| Information-flow and taint analysis | Knowledge leaks | §9.8 |
| Constraint solving, unsat cores | Temporal network, negative cycles as explanations | §9.6 |
| Assertions, contracts, temporal properties | Author invariants in LTLf | §9.10 |
| Incremental compilation (salsa in rust-analyzer) | Memoised queries over block fingerprints | §7 |
| Diagnostics, quick fixes | Marks, abductive repairs | §10.7, §12 |
| Language Server Protocol | Story language server | §12.4 |
| Rename, references, go to definition, inlay hints | Rename a character, all mentions, where a fact was settled, inferred facts as ghost text | §12.4 |
| `git blame`, `git bisect`, CI baselines | Who settled a fact, which edit introduced a mark, marks new since a snapshot | §12.5 |
| Merge conflicts | Canon conflicts introduced by merging a draft | §12.5 |

Two places where fiction is *not* code drive most of the novel engineering:

1. **The parser is uncertain.** A compiler's parser is exact; ours is a set of models with error rates. Hence a probabilistic layer between perception and verdicts (§10).
2. **Contradictions are sometimes the point.** Lies, dreams and unreliable narrators are legitimate. Hence contexts (§6.3) and defeasible reasoning, and the rule that the author can always overrule.

---

## 5. Architecture

### 5.1 Layers

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Clients: desk (React) · Word task pane · Docs sidebar · CLI · test harness    │
└───────────────▲──────────────────────────────────────────────────────────────┘
                │ story language server protocol (§12.4)
┌───────────────┴──────────────────────────────────────────────────────────────┐
│ asten-ls        requests, notifications, cancellation, priorities             │
├──────────────────────────────────────────────────────────────────────────────┤
│ asten-engine    salsa database: inputs, tracked queries, revisions (§7)       │
│   ├─ perception queries ──▶ asten-perceive (models via ONNX Runtime) (§8, §11)│
│   ├─ IR queries         ──▶ asten-ir (types, anchors, contexts) (§6)          │
│   ├─ analyses           ──▶ asten-logic (Datalog, event calculus, typestate)  │
│   │                         asten-time (calendars, STN, Allen, travel)        │
│   │                         asten-know (knowledge flow), asten-ltl (invariants)│
│   └─ reasoning          ──▶ asten-prob (HL-MRF, provenance, diagnosis) (§10)  │
├──────────────────────────────────────────────────────────────────────────────┤
│ asten-application (use cases) · asten-domain (types)                          │
├──────────────────────────────────────────────────────────────────────────────┤
│ Storage: SQLite canon store + Dynodoc event model (SQLite adapter) + caches   │
└──────────────────────────────────────────────────────────────────────────────┘
```

The existing crates keep their roles. `asten-diagnostics` becomes the registry and data model for diagnostic codes and is fed by the new analysis crates instead of evaluating rules itself. Appendix C lists crates, responsibilities and key interfaces.

### 5.2 Data flow for one edit

1. The author types. The desk produces a Dynodoc `RichTextPatched` event for block $b$.
2. The engine sets the salsa input `block_text(b)` to the new content and fingerprint.
3. Deterministic queries recompute for $b$ only: segmentation, alias matching, settled-fact anchors. If the alias matches are unchanged, propagation stops there (early cutoff, §7.2).
4. Model queries for $b$ are scheduled in the background at high priority because $b$ is visible.
5. Analyses that read $b$'s facts and observations recompute: the event-calculus state from $b$'s story time onward, the temporal network if $b$ added constraints, the typestate traces of the entities $b$ mentions.
6. The probabilistic layer warm-starts from its previous solution and re-optimises the affected factors.
7. Diagnostics whose inputs changed are re-published. The client receives a diff of marks, not the full list.

Steps 2–3 and the exact parts of 5 fit within the 50 ms budget. Steps 4 and 6 are asynchronous and typically complete within a second or two on CPU for a single paragraph.

---

## 6. The formal model

### 6.1 Documents, blocks, revisions, and anchors

**Revisions.** A project revision is the tuple of each document's head `seq` (and draft revision, when working in a draft). Every derived result is a function of a revision. Two analyses are comparable only at the same revision, which is how stale results are detected and never shown as current.

**Anchors must survive editing.** Today's `SourceAnchor { document_id, start, end }` uses offsets into the whole document's plain text, so typing a word in chapter 1 moves every anchor after it. The replacement is a block-relative anchor:

```text
Anchor {
  block:        BlockId          // stable, never reused
  start, end:   u32              // offsets in Unicode scalar values within the block
  revision:     (seq, fingerprint)  // where these offsets are valid
  quote:        String           // the anchored text at that revision, for repair
  context:      (String, String) // up to ~32 scalars before and after
}
```

Anchors are kept current by **position mapping** through each `RichTextPatch`, the same technique editors use for cursors and comments. A patch is a sequence of operations $\langle \mathrm{Retain}(n) \mid \mathrm{Delete}(n) \mid \mathrm{Insert}(s) \rangle$. A position $x$ maps to $\mu(x)$ by walking the operations with two counters (old and new position):

- $\mathrm{Retain}(n)$: positions in the retained range shift by the accumulated difference.
- $\mathrm{Insert}(s)$ at the anchor's position: the start maps to the left of the insertion, the end to the right, so text typed inside a span stays inside it (the "sticky" bias is configurable per anchor type).
- $\mathrm{Delete}(n)$ covering part of the anchor: the anchor shrinks; if the whole anchor is deleted it becomes **orphaned**.

Mapping is $O(k)$ for $k$ operations and exact. When it cannot be applied (the block was replaced wholesale, or the anchor is orphaned), the anchor is **re-attached** by fuzzy search for its quote in the block's new text, scored by edit distance on the quote plus agreement of the surrounding context, accepted only above a threshold. Failing that, the anchor and everything derived from it is marked stale, and any settled fact that depended on it is shown to the author as "source text changed; please confirm". This is the recovery strategy the planning archive asked for (§9.7 there), made concrete.

When a block is split or merged (paragraph break inserted or removed), Dynodoc creates or deletes nodes. The engine treats a split as "anchors in the tail move to the new block": the events that describe the split carry enough information to map offsets across.

**Granularity.** Offsets are in Unicode scalar values to match Dynodoc patches. Everything user-facing (selection, highlighting, distances in §9.1) works in grapheme clusters, converted at the boundary. Mixing those two units is a classic source of bugs in complex scripts, so the conversion lives in one module with property tests.

### 6.2 The Story IR

Everything after the frontends works on one language-independent intermediate representation, the **Story IR**. It plays the role LLVM IR plays for compilers: many language frontends, one set of analyses. Nothing in the IR is a word in a natural language except display labels and the source spans it points back to.

| IR element | Definition | Notes |
|---|---|---|
| Entity $e \in E$ | A thing the story talks about, with a sort $\sigma(e)$ | Sorts are character, place, item, group, event, concept, rule, source, plus pack-defined sorts. Today's `EntityKind` is the seed. |
| Mention $m$ | An anchored span that refers to an entity: $m = \langle \alpha, e, q \rangle$ | $\alpha$ is a block-relative anchor (§6.1), $q$ the link confidence. |
| Predicate $p \in P$ | A typed attribute or relation with a signature $p : S \to D_p$ | Example: `hair_color : Character → Color`. Declares cardinality, mutability, symmetry, inverse, and units. |
| Value $v \in D_p$ | An element of the predicate's value domain | Domains are **concept lattices**, not strings (§9.2). "auburn", "auburn-haired" and the Bengali "তামাটে" can all map to one concept. |
| Fluent $\phi = p(e, v)$ | A property that can hold or stop holding over story time | Event-calculus vocabulary (§9.3). |
| Event $\varepsilon$ | Something that happens at a story time and can initiate or terminate fluents | "Mira dyes her hair", "Oren dies". |
| Scene $s$ | A unit of narration with a reading position and a story-time placement | Usually a binder scene; may be finer. |
| Context $c$ | The epistemic frame a statement lives in | World truth, narration, a character's belief, a dream, a frame story… (§6.3). |
| Fact $f$ | $f = \langle \phi, c, I, \mathit{st}, \pi, q \rangle$ | Fluent, context, validity interval $I \subseteq \mathbb{T}$, status, provenance, confidence. |
| Observation $o$ | A statement in the prose that *reports* a fluent at a time, without causing it | "Mira pushed her black hair…" is an observation of `hair_color(Mira, black)` at that scene's time. |
| Constraint $\kappa$ | A temporal, spatial, or logical constraint with provenance | "three days after the siege". |

Two clocks are first-class:

- **Reading order** $<_R$ is a total order on blocks, derived from binder order and block order inside documents.
- **Story time** $\mathbb{T}$ is integer *ticks* in a project-chosen base unit. Calendars (§9.6) map human labels to ticks. Scene placements and event times are variables $X_i \in \mathbb{T}$ constrained by a temporal network, not single numbers, because most stories only pin time down partially.

**Statuses** keep the author's authority explicit. Every fact is one of `settled` (author-confirmed), `proposed` (extracted or inferred, awaiting the author), `deprecated` (retracted but kept for history), or `rejected`. Today's `FactStatus` already has the first three.

**Provenance** $\pi$ records where a fact came from: an anchor, a wiki field, an author action, a model (with model id and version), or a rule derivation (with the ids of its premises). Provenance is not decoration. The explanation and blame machinery in §10 is computed from it.

### 6.3 Contexts: lies, dreams, and frame stories

Fiction is full of statements that are not world truth. If the engine compared every sentence as an omniscient fact it would drown the author in false alarms. The handoff already lists the modalities (world true, narrator asserted, believed, claimed, rumor, lie, dream, vision, story-within-story, counterfactual). The engine turns them into **contexts**, a lightweight version of possible-worlds semantics.

Each statement is evaluated in a context $c$:

- $W$: world truth, what is actually so in the story world.
- $N$: narration. By default $N$ *projects into* $W$ (what the narrator says is true), unless the project marks the narrator unreliable.
- $\mathit{Say}_x$: what character $x$ claims. Does not project into $W$.
- $B_x$: what $x$ believes. Does not project into $W$; projects into $x$'s knowledge state (§9.8).
- $D_k$, $V_k$, $F_k$, $H_k$: a dream, a vision, frame story $k$, a hypothetical or counterfactual. Each is its own world with its own facts, inheriting from $W$ by default unless overridden (a dream inherits that Mira exists, not that she can fly).

Formally the contexts form an **inheritance hierarchy** $\mathcal{C}$ with a projection relation $\triangleright$. A fact $f$ in context $c$ is *visible* in context $c'$ if $c \triangleright^* c'$. Contradictions are only computed **within** a context, after inheritance:

$$
\mathrm{Conflict}_c(e, p, v_1, v_2, t) \iff \mathrm{HoldsAt}_c(p(e, v_1), t) \wedge \mathrm{HoldsAt}_c(p(e, v_2), t) \wedge \mathrm{functional}(p) \wedge v_1 \sqcap v_2 = \bot
$$

Cross-context rules are where fiction gets interesting, and they are pack material rather than core:

- A **lie** is a claim $\mathit{Say}_x(\phi)$ where $W \models \neg\phi$. Not an error; possibly an annotation ("Oren lies here").
- A **knowledge leak** is when $x$ acts on $\phi$ while $B_x \not\models \phi$ and nothing in $x$'s history could have told them (§9.8).
- An **unreliable narrator** cuts the $N \triangleright W$ edge for flagged passages, so narration no longer forces world truth.

Assigning a context to each clause is a perception problem (model M4, §11), with author override always winning. When the model is unsure, the statement lands in $N$ with lowered confidence, so it can still be checked but is ranked down.

### 6.4 Gradual canon

Asten must work for a gardener who has settled nothing and for an architect with a 400-entry dossier. Programming languages solved the same problem with **gradual typing** (Siek & Taha 2006): untyped code runs unchecked, typed code is checked, and the boundary is explicit. The engine adopts the idea directly.

| Gradual typing | Gradual canon |
|---|---|
| A value of unknown type `?` | Prose with no settled facts behind it |
| Type annotations | Settled facts, wiki fields, author confirmations |
| Type inference | Extraction proposals (shown, not trusted) |
| Static type error | A conflict between **settled** facts: an error |
| Runtime cast failure, with blame | A conflict involving **proposed** facts or observations: a warning, blamed on the less certain side |
| Adding annotations makes more checks possible | Settling facts makes more checks possible |

The **blame** idea (Wadler & Findler 2009) matters. When a settled fact and an unsettled observation disagree, the engine blames the observation by default: the mark lands on the sentence, not on the canon. When two settled facts disagree, both sides are shown and nothing is blamed. §10.6 generalises this into probabilistic diagnosis.

Strictness is a project setting, the same way compilers have warning levels:

- **Quiet:** only settled-versus-settled conflicts.
- **Standard:** plus settled-versus-observed, above a calibrated confidence.
- **Thorough:** plus proposed-versus-proposed, ranked, for revision passes.

---

## 7. Incremental computation

### 7.1 Why salsa

An IDE answers questions like "what is the type of this expression?" thousands of times per second because it **memoises** every intermediate result and **recomputes only what an edit invalidated**. rust-analyzer does this with salsa, a Rust framework for on-demand, incremental computation. The canon engine adopts the same model.

The vocabulary:

- **Inputs** are values the outside world sets: `block_text(b)`, `block_order(doc)`, `settled_facts(project)`, `entity_table(project)`, `pack_config(project)`, `model_version(m)`.
- **Tracked queries** are pure functions of inputs and other queries: `segments(b)`, `mentions(b)`, `observations(b)`, `scene_state(s)`, `stn(project)`, `diagnostics(doc)`.
- The database keeps a global **revision counter**. Setting an input bumps the revision and records which input changed.

### 7.2 The algorithm, briefly

Each memoised query result stores the revision when it was last **verified** and the revision when its value last **changed**, plus the list of queries it read. When a query is requested at revision $r$:

1. If it was verified at $r$, return it.
2. Otherwise, check each dependency recursively. If none changed after this result was last verified, mark it verified at $r$ and return it (**green**).
3. Otherwise, re-execute it (**red**). If the new value equals the old one, keep the old "changed at" revision. This is **backdating**, also called **early cutoff** (Mokhov, Mitchell & Peyton Jones 2018): queries that depend on this one see no change and do not re-run.

Early cutoff is what makes typing fast. Fixing a typo in a paragraph re-runs that block's segmentation and mention detection. Their outputs are the same set of mentions, so nothing downstream re-runs. An edit that changes a fact re-runs the state queries from that scene's story time onward, and nothing else.

Two salsa features matter here:

- **Durability.** Inputs that rarely change (pack definitions, lexicons, model versions) are marked high-durability, so checking dependencies on them is skipped on ordinary edits.
- **Cancellation.** When a new edit arrives, in-flight queries for the old revision are cancelled cooperatively. The server always answers for the newest revision.

### 7.3 Where salsa stops and incremental logic begins

Salsa is ideal for the per-block front half (segmentation, perception, per-block IR). The back half is recursive logic over the whole project (Datalog rules, the event calculus, the temporal network), where the right tools are incremental **fixpoint** algorithms:

| Option | How it works | Fit |
|---|---|---|
| Re-run Datalog per query, semi-naive | Recompute the least fixpoint with only new facts each round | Simple. Good enough while projects are small and rules few. |
| DRed (delete and re-derive) | On deletion, over-delete derived facts, then re-derive what still holds (Gupta, Mumick & Subrahmanian 1993) | Classic, handles recursion, moderate complexity. |
| Counting | Track the number of derivations of each fact | Fast for non-recursive rules. |
| Differential dataflow | Maintain collections as streams of differences, including through iteration (McSherry et al. 2013); DDlog and FlowLog compile Datalog to it | Strongest incrementality; heavier dependency and learning curve. |

**Recommendation.** Start with `ascent` (a Datalog engine embedded in Rust through macros, with lattices and stratified negation) and recompute the logic layer per affected **scope**: per entity for typestate, per story-time suffix for the event calculus, per connected component for the temporal network. These scopes are small, so semi-naive recomputation inside them is cheap. Adopt DRed or differential dataflow only if profiling on series-scale projects shows the need. The temporal network keeps its own incremental algorithm (§9.6).

### 7.4 Persistence and caches

Salsa's memo tables live in memory. Two caches persist to disk so a reopened project is fast:

- **Perception cache**, keyed by (block fingerprint, model id, model version). Model outputs are the expensive part. They are deterministic given these keys, so the cache is always valid and can be shared across drafts and documents.
- **IR snapshot**, keyed by the project revision. On open, the engine loads the last snapshot and replays only the events since, exactly as Dynodoc materialises documents from a snapshot plus a tail.

### 7.5 Cost model

Let $B$ be the number of blocks, $F$ the facts, $S$ the scenes, and $n$ the time points in the temporal network. With early cutoff:

| Edit | Work |
|---|---|
| Typo, no fact change | $O(\lvert b \rvert)$ for the block, nothing downstream |
| New observation in block $b$ | One lookup against $\mathrm{HoldsAt}$ for the observed fluent: $O(\log F)$ with an interval index |
| New or changed fact at story time $t$ | Event-calculus sweep over affected fluents from $t$: $O(F_{\phi} \log F_{\phi})$ for that fluent's history |
| New temporal constraint | Negative-cycle check $O(1)$, matrix update $O(n^2)$ |
| Reorder chapters | Reading-order queries for moved scenes; story time unaffected unless constraints reference reading order |

The planning archive's 50 ms target is plausible on these bounds for the exact layer. It must be measured on synthetic projects at 10k, 100k and 250k words and multi-book scale (§14.5).

---

## 8. Perception: language-agnostic frontends

Perception turns text into IR evidence. Everything here may be wrong, so everything carries a confidence and provenance, and nothing here becomes canon without the author.

### 8.1 Normalisation and segmentation

- **Normalisation.** NFC for storage and display; a separate case-folded, NFKC "matching form" for alias lookup, computed per script (case folding is meaningless in most scripts and harmless there).
- **Graphemes, words, sentences.** Unicode text segmentation (UAX #29) through ICU4X. ICU4X also segments languages without spaces between words: dictionary-based for Chinese and Japanese, LSTM-based for Burmese, Khmer, Lao and Thai. Sentence segmentation feeds clause boundaries for the models.
- **Bidirectional text** affects display, not logic. The engine stores logical order only.
- **Dialogue.** Quotation marks differ by language («», „“, 「」, — at line start in Spanish and Russian). Quote detection is table-driven per locale from CLDR delimiters, with a model fallback (§11.5).

### 8.2 Mentions and linking: deterministic first

The deterministic path works in every language from day one, using only what the author has entered:

1. Build an **Aho–Corasick automaton** over the matching forms of every entity name and alias (`aho-corasick` crate). Matching is $O(\lvert \text{text} \rvert + \text{matches})$, independent of the number of names.
2. **Morphology.** Inflected and agglutinative languages attach endings to names (Bengali *মিরার*, "Mira's"; Turkish *Mira'nın*; Finnish *Miran*). The matcher accepts a name followed by a suffix from a per-language suffix list, or, for languages with no list yet, any match at a word-initial position followed by letters, with lowered confidence. A learned segmentation model can replace the list later (§11.2).
3. **Ambiguity.** "Rose" the character versus "rose" the flower. In scripts with case the matcher uses capitalisation as a feature, not a rule. Everywhere else, ambiguous names are resolved by the mention model, and fall back to "possible mention" with low confidence.

The model path (§11.2) adds unknown names, pronouns and descriptions ("the old captain"), producing candidate links scored against the cast. Linking is then the constrained clustering of §9.1.

### 8.3 Facts, events, time, and framing

The models of §11 produce, per block:

- **Observations and change events** as event-calculus atoms with fluent, polarity, and confidence (§11.3).
- **Temporal anchors**: normalised time expressions, relations between scenes and events, and durations, each as a distribution, not a point (§11.4).
- **Contexts** for clauses: narration, dialogue by speaker, belief, dream, hypothetical, flashback (§11.5).
- **Knowledge events**: who witnessed, told, or read what (§9.8), from the same extraction model with knowledge-specific labels.

Date and number parsing that does not need a model (absolute dates in a real calendar, numerals in any script) uses ICU4X and CLDR data directly and is deterministic.

### 8.4 The contract between perception and reasoning

Every perceptual output enters the IR as an **evidence atom**:

```text
Evidence {
  atom:       IR atom (Obs, Initiates, TimeRel, Context, Mention, …)
  confidence: f32 in [0, 1], calibrated (§10.8)
  provenance: (model id, model version, anchor, input fingerprint)
  status:     proposed
}
```

Reasoning never reads raw text. This boundary is what makes the reasoning core language-independent and testable without models: any analysis can be tested with hand-written evidence atoms.

### 8.5 Bootstrap import from existing drafts

**The problem.** Many writers arrive with a finished or half-finished draft and no canon. Tier C (§13.2) checks nothing until facts are settled, and settling a novel's worth of facts by hand is the friction that makes people give up. The importer reads the draft once and proposes a starting canon. The writer corrects it.

**Its place in the design.** This is the one component allowed to use a generative model (§1, decision 2), and it is fenced:

- **Optional and local.** The model is downloaded on demand, never bundled, never required. Everything else works without it.
- **A frontend under the §8.4 contract.** It emits evidence atoms with status `proposed` and provenance (model id, version, quantisation, schema version). Nothing becomes settled without the writer.
- **Reads only.** It never writes prose. The writer sees structured fields and their own quotes, never generated text.
- **Outside the per-edit loop.** It runs in the background at low priority. Nothing in §9 or §10 depends on it.
- **Not a training source.** Its outputs on a writer's draft are never used to train anything.

**The model.** A small open-weights instruction model, for example Gemma 4 E2B or E4B (Apache 2.0, 128k-token context), run at 4-bit precision through a local runtime behind a trait so it can be swapped. The model does extraction per chunk only. **Reconciliation across the book is deterministic**, for two reasons. First, contradictions in the draft are exactly what the product exists to find: a model asked to tidy the canon tends to pick one value and drop the other, hiding the slip. Second, a 2–4B model is unreliable at reasoning over a whole novel even when the text fits its context, while merging aliases and aligning facts are what §9 already does exactly.

**Pipeline.**

| Step | By | What happens |
|---|---|---|
| 0. Prepare | Rules | Import into Dynodoc blocks (each gets a block id); detect chapters and scene breaks; identify the language per block (§13.3); harvest name candidates (capitalised sequences in cased scripts, frequency and co-occurrence counts) as a seed cast |
| 1. Extract | Model, per chunk | Structured output constrained to a JSON schema, so it always parses |
| 2. Ground | Rules | Every item's quote must be found in its block; items without a matching quote are dropped |
| 3. Reconcile | Rules | Merge entities and aliases; aggregate observations; apply the event calculus; build proposed time constraints |
| 4. Score | Rules | Calibrated confidence per proposal |
| 5. Review | Writer | Triage queue; accepted items become settled |

**Step 1: extraction.** A chunk is a scene, or 1,500–3,000 tokens split at block boundaries with one block of overlap. The prompt contains the schema, the text with block markers (`[b17] …`), and a **running cast sheet**: ids, names, aliases and sorts of the entities that matter for this chunk, chosen by Aho–Corasick matches of known names inside it (§8.2) so the sheet stays short. The cast sheet makes the model link to existing ids instead of inventing new entities. The schema has:

- `entity`: cast id or new temporary id, name, sort, aliases introduced here;
- `observation`: entity, property, value, quote, block;
- `event`: kind (dies, born, travels, acquires, loses, injured, learns, marries, …), participants, quote, block;
- `time_cue`: expression, quote, block;
- `frame`: quote and its context (narration, dialogue, dream, memory, lie, hypothetical).

Properties and event kinds come from a **closed vocabulary**: the IR's predicate signatures and the concept lattices of §9.2. Values of lattice properties are normalised to concept ids by deterministic lookup; values that do not match become text proposals that the writer can map. An `other` slot keeps unexpected facts as hints only. Decoding is greedy at temperature 0, and each field records its token log-probabilities.

**Step 2: grounding.** Each item's quote is searched for in the block it cites, first exactly after NFC normalisation, then fuzzily, accepted below a small normalised edit distance. The match becomes the item's anchor (§6.1). Items whose quote cannot be found are dropped and counted in the import report. **This is the main defence against made-up facts: every proposal the writer sees is backed by their own words.**

**Step 3: reconciliation.**

- **Entities.** Correlation clustering (§9.1) over temporary entities and surface forms. Attracting evidence: same matching form, model links, appositions the text declares ("Oren, whom the crew called the Captain"). Repelling evidence: two names acting as distinct agents in the same sentence, incompatible sorts. Ambiguous pairs may get one short yes/no question to the model with both quote contexts; the answer is one more weighted edge, never a decision.
- **Facts.** Observations are grouped per (entity, property) and run through the event calculus (§9.3) with the extracted change events. There are three outcomes. Consistent observations become one proposed fact with all supporting quotes. Changes explained by an event become a proposed sequence of values. **Unexplained conflicts keep both sides**, and the engine reports them as ordinary diagnostics. The importer never picks a winner.
- **Time.** Absolute time cues become proposed scene anchors; relative cues become proposed constraints in the temporal network (§9.6).

**Step 4: scoring.** Each proposal's confidence combines the number of independent supporting quotes, quote-match quality, the mean token log-probability of its fields, and agreement with deterministic mention matches. The combination is calibrated with isotonic regression on **gold import sets**: public-domain novels with hand-built canon, per language.

**Step 5: review.** The import review queue puts the cast first, sorted by mention count, with merge, split, rename and sort controls, because entity errors spoil everything downstream. Facts follow per entity, sorted by support, each with its quotes: accept, edit, reject, or accept everything with at least $N$ quotes. Contradictions found during import appear as ordinary diagnostics, which is often the first useful thing the writer sees. Accepting settles the fact as an author action (a Dynodoc event). Unreviewed items stay proposed and drive only info-level diagnostics (§12.2). Rejections are remembered by evidence fingerprint so a re-import does not propose them again.

**Re-import and determinism.** Extraction results are cached by (block fingerprint, chunk boundaries, model id, quantisation, schema version). After edits, only changed chunks are re-extracted; reconciliation reruns in full because it is cheap and deterministic. Greedy decoding with a pinned model and runtime is repeatable on the same machine, but floating-point differences across hardware can change outputs. Import results are therefore **stored, not recomputed**: the stored proposals with their provenance are the reproducible artefact.

**Cost and footprint (estimates to measure in Phase 2).** The model is a download of a few gigabytes at 4-bit. A 100,000-word English novel is about 140,000 tokens of input, more in scripts that tokenise poorly, plus cast-sheet overhead, and tens of thousands of tokens of structured output. Expect around an hour or more on a laptop CPU and roughly 10–20 minutes on Apple Silicon or a discrete GPU. The job runs in the background, shows progress, can be paused and resumed, and never blocks editing.

**Languages.** Small generative models are uneven across languages and weakest in low-resource ones. Grounding works in every language, so what survives is still anchored in the text. The import report states per language what was attempted and how much survived grounding. Where the model is too weak, the importer falls back to a cast-only import from step 0.

**Relation to M1–M5.** The importer is broad and slow; the models of §11 are fast, incremental and calibrated. Before Phase 3 the importer is the only source of extracted facts. After Phase 3 the models handle perception as the writer edits, and the importer remains the way to bootstrap a draft and to cover fact types the models do not. Both produce evidence atoms, so the probabilistic layer (§10) treats them as separate sources whose agreement raises confidence.

**Evaluation.** On gold import sets per language: precision of proposals after grounding, recall of the hand-built canon, cast clustering quality (B³ and CoNLL F1), and review effort (writer actions per accepted fact). One test is a hard requirement: inject contradictions into a public-domain novel with the operators of §11.9 and check that **both sides of every injected conflict survive the import**. The importer must surface contradictions, not smooth them.

---

## 9. Symbolic analyses: the type checker

These analyses are deterministic. Given the same IR they produce the same results on every machine. They are exact over settled facts and probabilistic only in the confidence numbers they carry along from perception. §10 adds collective probabilistic reasoning on top; nothing in §9 depends on it.

### 9.1 Names and symbols

This is the cheapest and most IDE-like layer.

**Symbol table.** Every entity has a canonical name and aliases. Aliases may be time-scoped (a married name from chapter 12) and context-scoped (a nickname only one character uses). The table is the analogue of a compiler's scope chain.

**Alias resolution as constrained clustering.** Mentions are linked to entities by clustering. The author's assertions become hard constraints: *must-link* ("the stranger is Oren") and *cannot-link* ("these are two different Toms"). Model scores become soft pairwise similarities $w_{ij}$. The objective is **correlation clustering** (Bansal, Blum & Chawla 2004):

$$
\min_{\mathcal{K}} \; \sum_{(i,j) \text{ same cluster}} \max(0, -w_{ij}) \;+\; \sum_{(i,j) \text{ split}} \max(0, w_{ij})
\quad \text{s.t. must-link and cannot-link}
$$

It is NP-hard in general, but the instances are small and local: mentions only compete with candidates that share a name key. A greedy pivot algorithm with a union-find structure for must-links is fast and good enough. Re-clustering after an edit is restricted to the clusters touched by the edited block.

**A late identity reveal is a feature, not a bug.** "The stranger" turning out to be Oren is an alias whose *link time* is the reveal scene. Before it, cross-checks between the stranger's facts and Oren's are suppressed. After it, they are merged. This is unification with a time stamp.

**Symbol diagnostics:**

| Diagnostic | Compiler analogue | Rule |
|---|---|---|
| Unknown name | Undeclared identifier | A mention with high "is a name" probability that links to no entity. Information level: "add Lord Aric to the cast?" |
| Used before introduction | Use before definition | First reading-order mention of $e$ precedes $e$'s introduction (author-marked, or the first mention with descriptive apposition), and the text presupposes the reader knows $e$. |
| Confusable names | Shadowing, typo-squatting lints | Two entities whose names are close under a script-aware distance (below). |
| Dangling reference | Reference to a deleted symbol | A mention or fact pointing at a removed entity. |
| Unused setup | Unused variable | A planted element with no payoff by the end of the manuscript (§9.10). |

**Script-aware name distance.** English-only tricks (capital letters, Soundex, Double Metaphone) are off the table. The distance between names $a$ and $b$ is a weighted combination:

1. Normalised edit distance over **grapheme clusters** (Unicode UAX #29), not bytes or code points. A Bengali conjunct is one unit of distance, as a reader perceives it.
2. Edit distance between **confusable skeletons** (Unicode UTS #39), which catches look-alike characters across scripts.
3. Edit distance between **Latin transliterations** (ICU transliterators, available for most scripts), which approximates "sounds similar" without language-specific phonetics.
4. Optionally, cosine distance between name embeddings from the mention model.

The weights are fitted on confusable-pair data per script. Jaro–Winkler is a reasonable default for the first three.

### 9.2 Sorts, signatures, and value lattices

**Sort checking.** Each predicate's signature says what it may attach to. `hair_color(the harbor, …)` is a sort error, and so is "the sword said" unless a world rule grants items speech. Checking is a lookup per fact, linear in the number of facts.

**Cardinality and mutability.** A predicate is *functional* (one value at a time, like hair color), *multi-valued* (allies), or *immutable* (birthplace). Functional predicates drive contradiction checks. Immutable ones turn any change event into an error unless the change is explicitly declared (a retcon).

**Values are concepts in a lattice.** Treating values as strings is the most common source of false alarms. "Dark hair" does not contradict "black hair"; "red" and "auburn" overlap; "about thirty" does not contradict "thirty-two". Each value domain $D_p$ is therefore a lattice $(D_p, \sqsubseteq, \sqcap, \bot)$, where $v \sqsubseteq w$ means "$v$ is a more specific case of $w$". Compatibility is a meet:

$$
\mathrm{compatible}(v_1, v_2) \iff v_1 \sqcap v_2 \neq \bot
$$

Examples:

- **Taxonomic domains** (colors, materials, species). A small concept hierarchy: `black ⊑ dark`, `auburn ⊑ red`, `auburn ⊑ brown-ish`. The meet of `auburn` and `black` is $\bot$; the meet of `dark` and `black` is `black`.
- **Numeric domains with uncertainty.** Values are intervals; the meet is intersection. "About thirty" becomes $[27, 33]$, which is compatible with $32$.
- **Quantities with units.** Dimensional analysis, exactly as in languages with units of measure: "six feet" and "183 cm" are the same length; "six feet" and "six metres" are not. Units come from CLDR unit data, so they are language-independent.
- **Entity-valued domains.** Values are entity ids; the meet is identity after alias resolution.

Concept ids are what make the domains language-independent. Lexicalisations come from multilingual lexical resources (the Open Multilingual Wordnet, Wikidata lexemes, CLDR) and from the author's own value lists, which always take precedence. The lattice is shipped per domain and extended by packs and by the author ("in this world, *storm-grey* is a hair color").

### 9.3 Story state: the event calculus

Story state is the heart of the engine, and the **event calculus** (Kowalski & Sergot 1986; the simplified form of Shanahan 1999) is its formal model. It answers one question: *which fluents hold at story time $t$?* It solves the problem Asten's current rules handle poorly, change over time, and it gives flashbacks a principled home.

**Vocabulary.** $\mathrm{Happens}(\varepsilon, t)$: event $\varepsilon$ occurs at $t$. $\mathrm{Initiates}(\varepsilon, \phi, t)$ and $\mathrm{Terminates}(\varepsilon, \phi, t)$: the event starts or ends fluent $\phi$. $\mathrm{Initially}(\phi)$: $\phi$ holds from the story's start. $\mathrm{HoldsAt}(\phi, t)$: the derived state.

**Axioms** (discrete time, per context $c$):

$$
\begin{aligned}
\mathrm{HoldsAt}(\phi, t) &\leftarrow \mathrm{Initially}(\phi) \wedge \neg \mathrm{Clipped}(0, \phi, t) \\
\mathrm{HoldsAt}(\phi, t) &\leftarrow \mathrm{Happens}(\varepsilon, t_1) \wedge \mathrm{Initiates}(\varepsilon, \phi, t_1) \wedge t_1 < t \wedge \neg \mathrm{Clipped}(t_1, \phi, t) \\
\mathrm{Clipped}(t_1, \phi, t_2) &\leftarrow \mathrm{Happens}(\varepsilon, t) \wedge t_1 < t < t_2 \wedge \mathrm{Terminates}(\varepsilon, \phi, t)
\end{aligned}
$$

**Inertia comes for free.** A fluent persists until something terminates it, which is the frame problem solved the standard way. "Mira's hair is auburn" in chapter 2 keeps holding through chapter 19 unless an event changes it.

**Functional fluents auto-terminate.** For a functional predicate, initiating $p(e, v)$ terminates every $p(e, v')$ with $v' \neq v$:

$$
\mathrm{Terminates}(\varepsilon, p(e, v'), t) \leftarrow \mathrm{Initiates}(\varepsilon, p(e, v), t) \wedge \mathrm{functional}(p) \wedge v' \neq v
$$

So "Mira dyes her hair black" in chapter 12 is an event that initiates `hair_color(Mira, black)` and therefore terminates `auburn`. Chapter 19's black hair is then consistent, and chapter 15's auburn hair would now be the error.

**Observations versus causes.** Most prose does not cause change; it reports state. An observation $o = \mathrm{Obs}(\phi, t, c, \alpha)$ asserts that $\phi$ holds at $t$ in context $c$, with anchor $\alpha$. The core contradiction check (the generalisation of CANON-E013) is:

$$
\mathrm{Violation}(o) \leftarrow \mathrm{Obs}(p(e, v), t, c, \alpha) \wedge \mathrm{HoldsAt}_c(p(e, v'), t) \wedge \mathrm{functional}(p) \wedge v \sqcap v' = \bot
$$

with severity decided by the statuses of the facts behind $\mathrm{HoldsAt}$ and by the confidence of the observation (§6.4).

**Evaluating the axioms.** The axioms contain negation ($\neg\mathrm{Clipped}$), and $\mathrm{Initiates}$ may depend on $\mathrm{HoldsAt}$ (an event's effects can depend on state: a key opens a door only if the door is locked). In general that is not stratified Datalog. It is, however, **locally stratified over time**: $\mathrm{HoldsAt}$ at $t$ only depends on things strictly before $t$. The program therefore has a unique perfect model (Przymusinski 1988), computed by sweeping time points in increasing order. That is what the implementation does, and it is also what makes it incremental: an edit at story time $t$ only invalidates state at times $\geq t$.

**Partially known time: may and must.** Story time is usually a partial order (§9.6). Two events with no known order could happen either way round. Rather than enumerate every linear order, the engine uses the temporal network's earliest and latest bounds for each time point:

- A violation is **must** if it occurs under every ordering the constraints allow (the bounds force the overlap).
- A violation is **may** if it occurs under some allowed ordering.

Must-violations over settled facts are errors. May-violations are warnings. This is the same must/may distinction static analysers use to separate definite bugs from possible ones.

### 9.4 Typestate: lifecycles

**Typestate analysis** (Strom & Yemini 1986) checks that operations happen only in valid states: you cannot read a closed file or use freed memory. Characters, items and places have lifecycles, and CANON-E101 ("acts after a settled death") is precisely *use after free*.

Each sort $\sigma$ has a **lifecycle automaton** $A_\sigma = (Q, \Sigma, \delta, q_0, Q_{\mathrm{err}})$. For a character in a default pack:

```text
           born                die                (resurrect, if a world rule allows)
 unborn ─────────▶ alive ─────────────▶ dead ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ▶ alive
   │                  │                     │
   │ act               │ act                 │ act (in context W)
   ▼                  ▼                     ▼
 error             alive                 error   ← CANON-E101
```

Items get `intact → damaged → destroyed` and `held / lost / found`; places get `standing → ruined`. Packs can define new automata (a vampire sort whose `act` is legal in `dead`).

**The check** runs each entity's events in **story-time order** through its automaton, per context. This is what fixes the flashback problem properly. A scene that is late in reading order but early in story time feeds its events at the early time, so a dead captain acting in a flashback is in state `alive` at that time and no error fires. Today's E101 skips events whose notes say "flashback"; the typestate version needs no such hint, only a story-time placement.

Product automata handle interacting states (a character who is both `imprisoned` and `alive`). Their size is the product of a few small automata per entity, so it stays tiny.

### 9.5 The data-flow view

Section 9.3 is the logic view. The same computation has a **data-flow analysis** view, which is useful for incremental implementation and for reasoning about concurrency, such as two story threads that run in parallel and later meet.

Take the **story-time graph** $G$: scenes as nodes, edges for known precedence. On each pair $(e, p)$ use the flat lattice

$$
L_{e,p} = \{\bot\} \cup D_p \cup \{\top\}, \qquad v \sqcup w = \begin{cases} v & v \sqcap w \neq \bot \text{ (take the more specific)} \\ \top & \text{otherwise} \end{cases}
$$

where $\bot$ means no information and $\top$ means conflicting information. The state is $X = \prod_{(e,p)} L_{e,p}$. Each scene $s$ has a monotone transfer function $f_s$ that overwrites the pairs its events initiate (the *gen/kill* of reaching definitions), and the analysis computes the least fixpoint

$$
X_{\mathrm{in}}(s) = \bigsqcup_{s' \to s} f_{s'}(X_{\mathrm{in}}(s'))
$$

with Kildall's worklist algorithm (Kildall 1973). Because the lattice has finite height and the transfer functions are monotone, it terminates. On a DAG it is a single pass in topological order.

What the view buys:

- **Join points.** Where two threads meet, a $\top$ at the join is a may-conflict: one thread says auburn, the other black, and the scene where both groups reunite inherits the conflict. That is precisely the "concurrent threads" case authors get wrong.
- **Incrementality.** An edit to scene $s$ only re-runs the worklist from $s$'s successors, and stops early wherever the recomputed state equals the old one.
- **Cycles.** Time loops and repeated days create cycles in $G$. The worklist handles them where a single sweep cannot.

This is the theory of monotone frameworks (Kam & Ullman 1977) and abstract interpretation (Cousot & Cousot 1977). The interval domain of §9.2 is literally the abstract interval domain.

### 9.6 Time: two clocks and a temporal network

**Calendars.** Each project chooses a base tick (minute, hour, day) and an epoch. A calendar is a pair of total functions between labels and ticks. Real calendars come from ICU4X (Gregorian, Hebrew, Islamic, Chinese, Ethiopian, Persian, and others), which already handles their arithmetic. Fictional calendars are defined declaratively: month lengths, leap cycles (a leap rule is a periodic predicate on the year), intercalary days, and eras. Fuzzy labels ("that winter", "around 1890") become intervals, not points.

**Simple Temporal Networks.** Most temporal statements are difference constraints:

$$
a_{ij} \;\leq\; X_j - X_i \;\leq\; b_{ij}
$$

"Three days after the siege" gives $X_{\mathrm{feast}} - X_{\mathrm{siege}} \in [3\mathrm{d}, 3\mathrm{d}]$; "a few days later" gives $[2\mathrm{d}, 6\mathrm{d}]$; "before" gives $[1, \infty)$. This is a Simple Temporal Network (Dechter, Meiri & Pearl 1991). Its **distance graph** has an edge $i \to j$ with weight $b_{ij}$ and an edge $j \to i$ with weight $-a_{ij}$. Then:

- The network is **consistent if and only if the distance graph has no negative cycle.** Bellman–Ford decides it in $O(|V||E|)$.
- All-pairs shortest paths $d_{ij}$ give the **minimal network**: the tightest bound each constraint can have. With a fixed origin $X_0$, each time point's earliest and latest values are $X_i \in [-d_{i0}, d_{0i}]$. Those bounds drive the may/must logic of §9.3.
- **The negative cycle is the explanation.** Its edges are exactly a set of statements that cannot all be true. Each edge carries provenance, so the diagnostic reads: *chapter 3 says the feast is three days after the siege; chapter 5 dates the siege to day 10; chapter 4 puts the feast on day 11.* The shortest negative cycle is a minimal conflict, the analogue of an unsat core.

**Incremental updates.** Adding a constraint $(u, v, w)$ creates a negative cycle if and only if $w + d_{vu} < 0$, an $O(1)$ check against the maintained distance matrix. Updating the matrix after an addition costs $O(n^2)$ with incremental full path consistency (Planken, de Weerdt & van der Krogt 2010). Removals are rarer and can trigger a local recompute. Networks per project are modest: hundreds to low thousands of time points.

**Qualitative relations.** Some statements are qualitative: "during the siege", "while she slept". Allen's interval algebra (Allen 1983) has 13 basic relations between intervals. Reasoning in the full algebra is NP-complete, but **ORD-Horn** relations form the maximal tractable subclass (Nebel & Bürckert 1995), solved by path consistency in $O(n^3)$. Most narrative relations ("before", "during", "overlaps", "meets") are convex and translate into point constraints, which drop straight into the STN. Rare disjunctive cases ("before or after, but not during") are kept in a small separate network and solved by backtracking over few variables.

**Uncertain constraints.** Constraints that come from extraction models carry probabilities. They enter the network as **soft** constraints. A hard network (settled facts only) is solved exactly. The soft layer is handled in §10, either as weighted potentials or by sampling bounds from the model's distributions and estimating the probability that a conflict exists. Probabilistic STNs and their chance-constrained variants (Tsamardinos 2002; Fang, Yu & Williams 2014) are the formal background.

**Reading order versus story time.** CANON-E201 today fires whenever neighbouring dated scenes run backward. In the new design the two clocks are independent by default: a scene that steps backward in story time is a flashback, and that is not an error. The engine only reports a clock problem when:

- the author declared a run of scenes **continuous** ("chapters 4–9 are one night"), and story time jumps backward inside it; or
- a scene's story-time placement makes the STN inconsistent; or
- an event's effects are observed before the event in story time (effect before cause).

### 9.7 Space and travel

Locations form a graph. Routes are edges carrying a travel mode, a duration range per mode, and an availability interval (a bridge that burns in chapter 7). For consecutive appearances of an entity at places $a$ and $b$, the shortest feasible travel time $\tau(a, b, t)$ comes from Dijkstra or A* on the time-dependent graph.

The elegant part is that **travel reduces to time**. Each pair of consecutive appearances adds a difference constraint

$$
X_{\mathrm{arrive}(b)} - X_{\mathrm{depart}(a)} \;\geq\; \tau_{\min}(a, b, t)
$$

to the same STN. An impossible journey therefore shows up as a negative cycle, with the travel edge in the explanation, and there is no separate travel solver to keep consistent. Spatial queries over map geometry (containment, distance) use an R-tree (`rstar`). If a map has no scale, travel constraints are simply not generated; qualitative adjacency still works.

### 9.8 Knowledge and information flow

Knowledge leaks, where a character knows something they could not know yet, are among the hardest continuity errors for humans and for language models. Formally this is **epistemic logic**: $K_x \phi$ means "$x$ knows $\phi$", and dynamic epistemic logic models how announcements change knowledge. Full dynamic epistemic logic is expensive and more expressive than stories need. The engine implements a decidable fragment as information flow over time, in the tradition of Denning's lattice model of secure information flow (Denning 1976).

```text
Learns(x, φ, t)  ← Witnesses(x, ε, t) ∧ Reveals(ε, φ)
Learns(x, φ, t)  ← Told(y, x, φ, t) ∧ Knows(y, φ, t)
Learns(x, φ, t)  ← Reads(x, d, t) ∧ Contains(d, φ)
Knows(x, φ, t)   ← Learns(x, φ, t₀) ∧ t₀ ≤ t ∧ ¬Forgets(x, φ, t₀, t)
Knows(x, φ, t)   ← Public(φ, t)
Leak(x, φ, t)    ← UsesKnowledge(x, φ, t) ∧ ¬Knows(x, φ, t) ∧ ¬Deduces(x, φ, t)
```

Each secret carries a label, the set of agents who know it, which grows only along communication edges in story time. A leak is a flow that violates the label. Nesting ("Mira knows that Oren knows") is supported to depth two, which covers most mystery and intrigue plots. Deeper nesting is rare and can be modelled explicitly by the author.

The difficult inputs are perceptual: which clause shows a character *using* a piece of knowledge, and which event *reveals* it. Those come from models (§11) and are therefore probabilistic. The flow logic itself is exact.

### 9.9 Relations and genealogy

Relations declare algebraic properties: symmetric (`married_to`), inverse (`parent_of` / `child_of`), transitive (`ancestor_of`), acyclic (`ancestor_of`, `contains`). The engine derives the implied facts and checks:

- **Acyclicity** with strongly connected components (Tarjan 1972). Anyone in an SCC of size above one in `ancestor_of` is their own ancestor.
- **Temporal constraints between relatives**, added to the STN: a parent is born at least a minimum age before a child, and a child is conceived before the parent's death (with a pack-defined gap for fathers).
- **Exclusive relations over overlapping intervals**: two simultaneous marriages, unless the world rules allow them.

### 9.10 World rules and author invariants

Authors and packs need to state their own rules. Two languages cover almost everything.

**Datalog rules** for structural laws: "magic costs blood", "only members of the guild can enter the archive". They compile into the same rule engine as the core analyses (§7.3, Appendix B).

**Temporal-logic invariants** for behaviour over the course of the story. The right logic is **linear temporal logic on finite traces**, LTLf (De Giacomo & Vardi 2013). A manuscript is a finite trace of scene states $\pi = s_0 s_1 \dots s_n$, in story time or reading order (the invariant chooses). Syntax:

$$
\varphi ::= a \;|\; \neg\varphi \;|\; \varphi \wedge \varphi \;|\; \mathsf{X}\varphi \;|\; \varphi\, \mathsf{U}\, \varphi, \qquad \mathsf{F}\varphi \equiv \top\, \mathsf{U}\, \varphi, \quad \mathsf{G}\varphi \equiv \neg\mathsf{F}\neg\varphi
$$

Authors should never see this syntax. The interface offers the **specification patterns** of Dwyer, Avrunin & Corbett (1999), which cover most real requirements:

| Pattern | LTLf | Story example |
|---|---|---|
| Absence after | $\mathsf{G}(q \rightarrow \mathsf{G}\neg p)$ | Once Oren dies, he never speaks (in $W$). |
| Response | $\mathsf{G}(p \rightarrow \mathsf{F}q)$ | Every planted clue is eventually paid off. |
| Precedence | $\neg q\, \mathsf{W}\, p$ | The reveal never comes before the clue is planted. |
| Universality between | $\mathsf{G}((q \wedge \neg r) \rightarrow (p\, \mathsf{W}\, r))$ | Mira stays in Veld between the siege and the treaty. |
| Bounded existence | (counting) | The ring changes hands at most three times. |

Each formula compiles to a deterministic finite automaton. The construction is doubly exponential in the worst case, but author invariants are tiny, so the automata have a handful of states. The automaton runs incrementally over the scene trace, so checking is linear in the number of scenes.

**Drafts are unfinished, so verdicts have three values.** A response obligation ("the clue is eventually paid off") is not violated just because the payoff has not been written yet. Following runtime verification practice (Bauer, Leucker & Schallhart 2011), each invariant reports *satisfied*, *violated*, or *pending*. Pending obligations appear in a ledger, not as errors, until the author marks the manuscript complete. This is the **setup/payoff ledger** from the planning archive, and the "unused setup" diagnostic of §9.1 is its end-of-book check.

---

## 10. Probabilistic reasoning

### 10.1 Why a probabilistic layer

The symbolic analyses of §9 are exact over their inputs, but perception is not. A typical extracted observation might be: *"Mira pushed her black hair"* → `Obs(hair_color(Mira, black), t₁₉, N)` with confidence 0.81 for the fluent, 0.93 that the mention is Mira, 0.88 that the clause is plain narration. A hard yes-or-no verdict throws that information away. Three things need a principled combination:

1. **Uncertain evidence** from several models, with correlated errors.
2. **Hard rules** that must not be violated (a functional fluent has one value per context and time).
3. **Soft rules** that usually hold (narration usually reflects world truth; characters usually stay where they were; a death usually ends a lifecycle).

The output the product needs is a **ranked list of marks with honest probabilities**, the "weighted decision ranking" asked for earlier, plus **which statement is most likely wrong** and **how it could be fixed**.

### 10.2 Options

| Framework | Semantics | Inference | Strengths | Weaknesses for us |
|---|---|---|---|---|
| Markov logic networks (Richardson & Domingos 2006) | Weighted first-order formulas over Boolean worlds: $P(x) \propto \exp(\sum_i w_i n_i(x))$ | MAP by weighted MaxSAT; marginals by MC-SAT | Expressive, well studied | Discrete inference is NP-hard and slow to converge; hard to make incremental |
| Probabilistic soft logic / hinge-loss MRFs (Bach et al. 2017) | Soft truth values in $[0,1]$, Łukasiewicz logic, hinge-loss potentials | MAP is a **convex** problem, solved by consensus ADMM | Fast, scalable, warm-startable (incremental), weights learnable | Truth values are not probabilities; needs calibration |
| ProbLog (De Raedt et al. 2007) | Possible worlds over probabilistic facts | Knowledge compilation to circuits; exact but #P-hard | Exact marginals, clean semantics | Exponential in bad cases |
| Scallop (Li, Huang & Naik 2023) | Datalog over **provenance semirings**, including top-$k$ proofs | Semi-naive evaluation; differentiable | Explanations and probabilities from one evaluation; gradients for training | Approximate (top-$k$); a research-grade dependency |
| Factor graphs with belief propagation | Arbitrary factors | Loopy BP | Flexible | Convergence not guaranteed on loopy graphs |
| Bayesian networks | Directed generative model | Exact or sampling | Interpretable causal story | Wrong shape for symmetric constraints |

### 10.3 The recommendation: three cooperating pieces

1. **Provenance-semiring Datalog** for derivations: one rule engine, evaluated under different semirings to get truth, explanations, and best-derivation scores (§10.4).
2. **A hinge-loss Markov random field** for collective ranking over uncertain atoms, solved by ADMM and warm-started on every edit (§10.5).
3. **Consistency-based diagnosis and abduction** on top, for blame and repairs (§10.6–§10.7).

Settled-versus-settled conflicts bypass the probabilistic layer entirely: they are errors with probability 1, explained by exact provenance.

### 10.4 Provenance semirings: one evaluation, many answers

Green, Karvounarakis & Tannen (2007) showed that if every tuple in a database is annotated with an element of a **commutative semiring** $(K, \oplus, \otimes, 0, 1)$, then evaluating positive queries combines annotations with $\otimes$ along a derivation (joins, rule bodies) and $\oplus$ across alternative derivations (unions). Change the semiring, and the same evaluation answers a different question:

| Semiring | $K$, $\oplus$, $\otimes$ | What the annotation of a derived fact means |
|---|---|---|
| Boolean | $\{0,1\}$, $\vee$, $\wedge$ | Is it derivable? |
| Why-provenance | sets of sets of base facts, $\cup$, pairwise $\cup$ | **Every minimal set of statements that derives it.** This is the explanation. |
| Viterbi | $[0,1]$, $\max$, $\times$ | Probability of the single most probable derivation |
| Top-$k$ proofs (as in Scallop) | sets of at most $k$ proofs | The $k$ best explanations; probability by weighted model counting over them |
| Tropical | $\mathbb{R}_{\geq 0} \cup \{\infty\}$, $\min$, $+$ | Cost of the cheapest derivation; used for repair costs (§10.7) |

For a conflict atom $\kappa$, one evaluation gives:

- Its **minimal witness sets** (why-provenance), from which the explanation chain is drawn: "auburn (chapter 2, settled) and black (chapter 19, observed); no change event between them".
- Its **best-derivation probability** (Viterbi), $\prod_i q_i$ over the confidences in the best witness, a fast lower bound on the probability the conflict is real.
- The **top-$k$ probability** for a better estimate when several weak derivations support the same conflict.

Semi-naive evaluation stays correct for these semirings when they are idempotent in $\oplus$ (Boolean, Viterbi, tropical) or bounded (top-$k$), which covers all the ones listed. Recursive rules terminate because the base facts are finite and the semirings are absorptive or bounded.

### 10.5 Collective ranking with a hinge-loss MRF

Some judgments are collective. If three mentions of "the stranger" link to Oren with middling confidence, and one of them is observed acting after Oren's death, the right probability for the lifecycle violation depends on all three links at once, and on whether "the stranger" was revealed as someone else. A hinge-loss Markov random field (the semantics of probabilistic soft logic) handles this.

**Atoms.** Each uncertain ground atom (a mention link, an observation, a context label, a time relation) gets a soft truth value $y_j \in [0,1]$. Observed evidence fixes or anchors values; latent atoms are free.

**Łukasiewicz relaxation.** Logical connectives become linear functions:

$$
a \tilde{\wedge} b = \max(0,\, a + b - 1), \qquad a \tilde{\vee} b = \min(1,\, a + b), \qquad \tilde{\neg} a = 1 - a
$$

**Potentials.** A weighted rule $w_r : \mathrm{body} \Rightarrow \mathrm{head}$ incurs a penalty equal to its *distance to satisfaction*:

$$
d_r(\mathbf{y}) = \max\bigl(0,\; I(\mathrm{body}) - I(\mathrm{head})\bigr)
$$

**Distribution.**

$$
P(\mathbf{y} \mid \mathbf{x}) = \frac{1}{Z(\mathbf{x})} \exp\Bigl( -\sum_r w_r \, d_r(\mathbf{y})^{\rho_r} \Bigr), \qquad \rho_r \in \{1, 2\}
$$

with hard rules as linear constraints on $\mathbf{y}$. Because every $d_r$ is a hinge of a linear function, the **MAP problem is convex**:

$$
\min_{\mathbf{y} \in [0,1]^n} \; \sum_r w_r \, d_r(\mathbf{y})^{\rho_r} \quad \text{subject to the hard constraints}
$$

**Solver.** Consensus ADMM splits the objective into one small subproblem per potential, each with a closed-form solution, plus a consensus averaging step. It parallelises trivially and, crucially for an editor, **warm-starts** from the previous solution: after an edit, only potentials near the changed atoms move, and convergence takes a few iterations.

**Example rules** (weights learned, §10.9):

```text
w=∞ : Functional(p) ∧ Holds(p(e,v1),t,c) ∧ Holds(p(e,v2),t,c) ∧ Incompatible(v1,v2) ⇒ ⊥
w=4 : Obs(φ,t,N) ∧ ReliableNarration(t) ⇒ Holds(φ,t,W)
w=2 : Link(m,e) ∧ Link(m',e) ∧ SameScene(m,m') ⇒ Coherent(m,m')
w=3 : Dies(e,t₀) ∧ t > t₀ ∧ Acts(e,t,W) ⇒ Violation(lifecycle,e,t)
w=1 : Obs(φ,t,Say_x) ⇒ Holds(φ,t,W)          // characters are believed weakly
```

**From soft truth to probability.** HL-MRF values are not probabilities. The violation atoms' MAP values, together with features (statuses, Viterbi scores, number of independent witnesses, distance in reading order), go through a calibration map trained on labelled data (§10.8). The product shows only calibrated probabilities.

### 10.6 Diagnosis: which statement is wrong?

A conflict says *something* is wrong. The author wants to know *what*. This is **consistency-based diagnosis** (Reiter 1987; de Kleer & Williams 1987).

- The **system description** $\mathit{SD}$ is the rule set.
- The **components** $\mathit{COMPS}$ are the statements that might be wrong: observations, extracted facts, links, context labels, and, with low prior, settled facts.
- A **conflict** is a set $C \subseteq \mathit{COMPS}$ such that $\mathit{SD}$ is inconsistent when all of $C$ is assumed correct. The minimal conflicts are exactly the minimal why-provenance witness sets of the violation atoms (§10.4).
- A **diagnosis** is a minimal set $\Delta \subseteq \mathit{COMPS}$ such that assuming everything outside $\Delta$ correct is consistent. Diagnoses are the **minimal hitting sets** of the minimal conflicts, computed with Reiter's HS-tree or, at our sizes, a simple incremental enumeration.

Each component $c$ has a prior probability of being at fault $p_c$, derived from its status and source:

| Component | Typical fault prior |
|---|---|
| Settled fact (author-confirmed) | 0.01 |
| Wiki field | 0.03 |
| Observation in narration, high-confidence extraction | $1 - q$ after calibration, clipped to at least 0.05 |
| Observation in a character's speech | 0.3 or more (characters misremember and lie) |
| Mention link, context label | $1 - q$ |

The probability of diagnosis $\Delta$ is proportional to

$$
P(\Delta) \propto \prod_{c \in \Delta} p_c \prod_{c \notin \Delta} (1 - p_c)
$$

restricted to the components that appear in some conflict. The best-ranked diagnosis decides where the red mark goes. For Mira, the chapter 19 observation (fault prior around 0.15) outranks the settled chapter 2 fact (0.01), so the mark lands on chapter 19 and the explanation points back to chapter 2. This is the probabilistic form of the blame rule in §6.4. Assumption-based truth maintenance (de Kleer 1986) is the classical data structure for keeping these environments up to date incrementally; the semiring evaluation already gives us its labels.

### 10.7 Abduction: proposing repairs

Quick fixes in an IDE are local code transformations. Story repairs are **abductive explanations**: minimal additions or changes to canon that would make the observation consistent (abductive logic programming; Kakas, Kowalski & Toni 1992).

Given a violation, find $\Delta \subseteq \mathcal{A}$ from a set of **abducibles** such that $\mathit{SD} \cup \mathit{Canon} \cup \Delta$ entails the observation and is consistent, minimising a cost $\mathrm{cost}(\Delta)$. The abducibles are the repair vocabulary:

| Abducible | Example quick fix | Typical cost |
|---|---|---|
| Change event in a gap | "Add an event: Mira dyes her hair (between ch. 12 and ch. 19)" | Low if a gap exists in story time |
| Context reclassification | "Mark this passage as a dream / as Oren's lie" | Low for dialogue, higher for narration |
| Time adjustment | "Move scene 'The harbor burns' to day 13" | Proportional to how far it moves and how many constraints it touches |
| Validity bound | "Auburn holds until ch. 12" | Low |
| Retract or deprecate a fact | "Deprecate: Mira's hair is auburn" | High for settled facts |
| Suppress with reason | "This is intentional" | Always available, never auto-chosen |

The tropical semiring computes minimal-cost repairs for single violations directly. For interacting violations, a small optimisation over candidate repairs (an answer-set program with weak constraints, or a bespoke branch-and-bound over at most a few dozen candidates) finds a jointly minimal set. The author sees the top three. **No repair edits prose**, consistent with the product rule that Asten only writes in red.

### 10.8 Calibration and statistical guarantees

**Model calibration.** Each model's raw scores are calibrated on held-out data per language: temperature scaling for softmax outputs (Guo et al. 2017), isotonic regression where miscalibration is not monotone in temperature. Reliability is tracked with expected calibration error and reliability diagrams per language and genre.

**Diagnostic calibration.** The final mark score (from §10.5) is mapped to a probability with isotonic regression on labelled marks.

**Controlling false alarms with a guarantee.** Choosing the display threshold by eye is how checkers end up noisy. Instead, choose it so that the expected fraction of false alarms among shown marks is bounded, with a statistical guarantee. Let $\lambda$ be the threshold and $R(\lambda)$ the false-discovery proportion of marks scoring at least $\lambda$. Using a labelled calibration set of $n$ manuscripts:

- **Learn then Test** (Angelopoulos, Bates, Candès, Jordan & Lei 2021) treats each candidate threshold $\lambda_j$ as a null hypothesis $H_j : R(\lambda_j) > \alpha$, computes a valid p-value for each from the calibration set, and tests them in a fixed sequence from strict to lenient, stopping at the first hypothesis it cannot reject. Every threshold whose hypothesis was rejected is certified; the most lenient of them is deployed. With probability at least $1 - \delta$, the deployed threshold satisfies $R(\hat\lambda) \leq \alpha$. It works for risks that are not monotone in $\lambda$, like the false-discovery proportion.
- **Conformal risk control** (Angelopoulos, Bates, Fisch, Lei & Schuster 2022) gives the bound in expectation for losses that are monotone in the parameter (parameterised so that the loss does not increase as $\lambda$ grows, as with a miss rate measured against a widening set of shown marks). With losses bounded by $B$,

$$
\hat\lambda = \inf\Bigl\{\lambda : \tfrac{n}{n+1}\hat R_n(\lambda) + \tfrac{B}{n+1} \leq \alpha \Bigr\} \;\Longrightarrow\; \mathbb{E}\bigl[R(\hat\lambda)\bigr] \leq \alpha
$$

Severity tiers get different targets: for example $\alpha = 0.05$ for marks shown as errors, $0.20$ for warnings, none for the "thorough" revision mode. The guarantees assume calibration data exchangeable with real use, so calibration sets are kept per language and genre, and the guarantee is reported with the population it was measured on.

**Learning from the author, locally.** Suppressions and accepted repairs are feedback. They are not ground-truth labels (an author suppresses intentional contradictions too), so they train a separate per-project "is this useful to me" adjustment, not the global models. The adjustment is a small logistic model over rule code, entity, section and score, kept on the author's machine. It changes ranking, never correctness. This is the "learn from suppressions" idea from the earlier discussion, made safe.

### 10.9 Learning rule weights

Soft-rule weights should not be hand-tuned forever. With labelled conflicts, HL-MRF weights are learned by maximising pseudo-likelihood or by structured perceptron updates on the MAP state (Bach et al. 2017). Weights are per pack and can be specialised by genre (a fantasy pack weights "characters return from death" violations lower when a resurrection rule exists). Learned weights ship with the pack; per-project adjustments stay local (§10.8).

---

## 11. Models

### 11.1 Principles

1. **Perception, not judgment.** Models report what the text says, relative to the IR: this span mentions Mira; this clause asserts a hair color; this sentence is in a dream; this scene comes three days after that one. **The logic decides whether that is a contradiction.** This separation is the main lesson of the FlawedFictions results: a plain entailment classifier (DeBERTa trained on MNLI), asked to judge contradictions directly from text pairs, scored 0.53 accuracy and 0.04 on localisation, barely above chance. It was being asked to do the reasoning implicitly, without time, context, or state.
2. **Discriminative encoders only.** Every model maps text to labels, spans, scores or embeddings. None generates text.
3. **Schema-conditioned.** Predicates, sorts and context labels are passed to the model as inputs (as in GLiNER and GLiNER2), so an author-defined predicate ("blood debt owed to") works without retraining.
4. **Calibrated outputs with uncertainty.** Every output carries a probability that is calibrated per language (§10.8). Durations and times are distributions, not points.
5. **Multilingual or byte-level backbones**, small enough for a laptop CPU after distillation and 8-bit quantisation.
6. **Trained to serve the logic.** After ordinary supervised training, models are fine-tuned through the reasoning layer so that what improves is the quality of the final diagnostics, not a proxy metric (§11.8).

### 11.2 M1: mentions and aliases

**Task.** Find spans that refer to entities (names, pronouns, descriptions such as "the old captain") and score links to cast entries.

**Architecture.** A schema-conditioned span model in the GLiNER family. The encoder reads `[labels] [SEP] [text]`, where labels are the sort names and the canonical names of nearby cast members. Span representations $h_{ij} = \mathrm{MLP}([h_i; h_j; \phi(j-i)])$ for spans up to length $L$ are scored against label representations $u_\ell$ with $s_{ij\ell} = \sigma(h_{ij}^\top u_\ell)$. Linking uses a bi-encoder (span embedding versus entity embedding built from the entity's mentions and facts) to retrieve candidates, then a small cross-encoder to re-rank the top few.

**Pronouns and descriptions** are linked within a window by a coreference head (mention-pair scoring with a learned antecedent distribution), whose scores feed the correlation clustering of §9.1 rather than deciding clusters on their own.

**Morphology.** For languages where names take suffixes, the span head learns name-plus-affix spans directly, which replaces the suffix lists of §8.2 once trained.

### 11.3 M2: the event-calculus parser

**Task, and what is new.** Most extractors output entities and relations. M2 outputs **event-calculus structure** directly: for each clause, which fluents it *observes*, which it *initiates* or *terminates*, with polarity and a link to a time anchor. Its output plugs into §9.3 without any hand-written mapping. Tracking entity state changes has prior art in procedural text (ProPara; Dalvi et al. 2018) and in physical-commonsense stories (TRIP; Storks et al. 2021, which also asks models to find the conflicting sentences). M2 extends that line to open, author-defined predicates, to contexts (dreams, lies, flashbacks), and to links into a temporal network. That combination is the part worth publishing.

**Output schema per clause.**

```text
{ entity_span, predicate (schema id), value_span | value_concept,
  role ∈ {observes, initiates, terminates, none},
  polarity ∈ {asserted, negated, uncertain},
  time_link → event or scene anchor (from M3),
  confidence per field }
```

**Architecture.** Shared encoder; span enumeration as in M1; a **triaffine** scorer over (entity span, value span, predicate label) to form candidate fluents; a role classifier over the clause representation concatenated with the fluent representation; and a polarity head. Predicate labels are provided as text (localised labels) for author-defined predicates and as learned embeddings for core predicates, so both open and closed schemas work.

**Value grounding.** The value span is mapped to a concept in the predicate's lattice (§9.2) by nearest-neighbour search in a multilingual lexicon embedding space, with the author's own value list searched first. Unmapped values become new candidate concepts for the author to confirm.

**Loss.** Multi-task cross-entropy for spans, roles and polarity, plus the **semantic loss** of §11.8 to keep outputs well-formed (one value per functional fluent per clause; a termination needs a fluent that could hold).

### 11.4 M3: temporal anchoring

**Tasks.** (a) Find and normalise time expressions; (b) relate scenes and events in story time; (c) estimate durations.

- **Normalisation without per-language grammars.** Instead of writing time grammars for each language, M3 tags a time expression and fills structured slots: kind (absolute, relative, duration, frequency, vague), quantity, unit, direction, and anchor ("the siege"). Numerals in any script are parsed with CLDR data; the slot values are language-independent, so one downstream normaliser converts them into STN constraints in ticks.
- **Relations.** A pair classifier over scene or event representations outputs a distribution over point relations $\{<, =, >, \text{overlap}, \text{vague}\}$, the convex core of Allen's algebra that maps straight into the STN (§9.6).
- **Durations.** Quantile regression on log-duration, trained with the pinball loss $\ell_\tau(y, \hat{y}) = \max(\tau(y - \hat{y}), (\tau - 1)(y - \hat{y}))$ for $\tau \in \{0.1, 0.5, 0.9\}$. The 10% and 90% quantiles become the soft bounds $[a_{ij}, b_{ij}]$ of the constraint. Vague phrases ("a while later") therefore produce wide intervals automatically instead of fake precision.

### 11.5 M4: framing and attribution

**Tasks.** Assign each clause a context (§6.3) and attribute dialogue to speakers.

- **Context classifier.** Multi-label over {narration, dialogue, internal thought, reported belief, dream or vision, memory or flashback, hypothetical, story-within-story}, with the speaker or believer as a linked argument. Scene-level cues (a scene that begins "Years earlier…") propagate down through a small sequence model over clause labels.
- **Speaker attribution.** Pairwise scoring between each quote and candidate speaker mentions, with features for distance and turn-taking, in the spirit of BookNLP's quotation attribution, trained multilingually.
- **Flashback detection** from M4 is what lets the temporal network place a scene correctly, and it is the automatic replacement for today's "notes contain the word flashback" heuristic in E101.

### 11.6 M5: the canon-conditioned verifier

**Task.** Given a window of text and the canon facts relevant to it, say what the text asserts **about each fact**: the same value, a different value, a change event, a negation, or nothing. The verifier does not decide contradictions. It reads the relationship; the event calculus decides whether that relationship is a violation at that story time.

**Why it is different from entailment models.**

1. **Structured, scoped premises.** Facts arrive as typed records with context, validity interval and status, not as free-text premises.
2. **Many facts at once.** The model sees up to $K$ facts together and can tell which one the text is about, which plain sentence-pair models cannot.
3. **"Different" is not "contradicts".** Separating *asserts a different value* from *contradicts* lets time scopes and change events do their job in the logic.
4. **Evidence pointers.** For each fact, the model points at the span that bears on it, which gives localisation for free.
5. **Story-local hard negatives** in training: same entity with a different predicate, same predicate with a different entity, the same fact in another context.

**Input encoding.**

```text
[CLS] text window (≤ 384 tokens, target block centred)
[SEP] [F1] <ent> Mira <pred:hair_color> <val> auburn <ctx:W> <from> day 3 <to> — <st:settled>
      [F2] <ent> Mira <pred:location> <val> Veld   <ctx:W> <from> day 10 …
      … up to K facts
```

Predicate, context and status are **language-independent special tokens** with learned embeddings. Entity names and values are rendered in the manuscript's own language from the canon, so the text and the facts share a language. A fact-index embedding is added to each fact's tokens.

**Heads.** For each fact $k$, a pooled representation $g_k$ (its marker token plus attention over its tokens) feeds:

- a relation head $\mathrm{softmax}(W g_k) \in \Delta^{5}$ over {same, different, change, negated, unrelated};
- an evidence pointer: biaffine start and end scores over text tokens conditioned on $g_k$;
- a listwise "aboutness" score across the $K$ facts, trained with a softmax ranking loss.

**Training data** comes from the perturbation pipeline (§11.9): unperturbed text gives "same" and "unrelated", value substitutions give "different", and lifecycle and negation operators give the rest. Change events come from stories where M2 and the author agree a change happens.

**Size.** Trained as a base-size multilingual encoder and distilled to roughly 100–150M parameters for deployment.

### 11.7 M6: the narrative state predictive model (research track)

The symbolic engine catches what can be stated as rules. Some continuity breaks resist rules: a character who could not swim crossing a river with no comment, a sworn enemy suddenly acting as an old friend. A complementary signal is **surprise**: how badly the next scene's state of an entity defies prediction from the previous scenes.

**Design.** A joint-embedding predictive architecture in the JEPA family, applied to entity state:

- An encoder $E_\theta$ maps the mentions of entity $e$ in scene $s$ (and its settled facts) to a latent state $z_{e,s} \in \mathbb{R}^d$.
- A predictor $g_\psi$ forecasts the next state from the current state, the events involving $e$ in between, and the elapsed story time: $\hat z_{e,s'} = g_\psi(z_{e,s}, a_{s \to s'}, \Delta t)$.
- Training minimises prediction error in latent space, with a regulariser that prevents collapse to a constant representation. LeJEPA's SIGReg (Balestriero & LeCun 2025) does this without a teacher network, by pushing the embedding distribution towards an isotropic Gaussian through random one-dimensional projections:

$$
\mathcal{L} = \sum_{(e,s,s')} \bigl\lVert g_\psi(z_{e,s}, a_{s\to s'}, \Delta t) - z_{e,s'} \bigr\rVert^2 + \lambda \cdot \mathrm{SIGReg}(Z)
$$

**Why it suits our constraints.** It is self-supervised, so it trains on any public-domain fiction in any language with no labels and no generative model. It predicts in latent space rather than generating text.

**From surprise to evidence.** On clean text, residuals $r = \hat z - z$ have an empirical distribution; with its covariance $\Sigma$, the surprise score is the Mahalanobis distance $u = r^\top \Sigma^{-1} r$, converted to a calibrated p-value against held-out clean scenes. **Interpretable surprise:** linear probes trained from $z$ to known predicates (alive, location, hair color) decompose the residual, so a large residual along the "alive" probe with no death event points at a likely lifecycle problem the extractor missed.

**How it is used.** Surprise enters the HL-MRF (§10.5) as a low-weight atom that can raise a candidate's rank only when another, independent signal (a symbolic rule, the verifier) points at the same place. It never produces a mark on its own. Plot twists are surprising by design; the corroboration rule keeps them from turning into false alarms.

**Status.** This is research. Its value must be shown on held-out data, especially the common-sense subset of FlawedFictions, before it ships.

### 11.8 Training through the logic

Supervised training teaches each model its local task. The final metric, however, is the quality of the diagnostics after reasoning. Three techniques connect the two.

**Semantic loss** (Xu, Zhang, Friedman, Liang & Van den Broeck 2018). For a constraint $\alpha$ over Boolean outputs with predicted probabilities $p$, penalise the probability mass outside the constraint:

$$
L_{\mathrm{sem}}(\alpha, p) = -\log \sum_{x \models \alpha} \;\prod_{i : x_i = 1} p_i \prod_{i : x_i = 0} (1 - p_i)
$$

Used for well-formedness: exactly one value per functional fluent per clause, transitivity of coreference within a window, a termination only of something that can hold. Small constraints are compiled to circuits once and evaluated cheaply.

**Differentiable provenance.** Evaluating the rules under a top-$k$ proofs semiring (as Scallop does) makes the probability of each derived diagnostic a differentiable function of the model outputs. The loss on the final diagnostics, against gold labels from the perturbation pipeline, back-propagates into M1–M5.

**Rewards from proper scoring rules.** When part of the pipeline is not differentiable (clustering, the STN), fine-tune with policy gradients (REINFORCE with a group-mean baseline), using as reward a **strictly proper scoring rule** of the final diagnostic probabilities, such as the log score or the Brier score (Gneiting & Raftery 2007). A strictly proper rule is maximised in expectation only by reporting the true probability, so the training pressure is towards honest confidence, not over-flagging. This is the same principle the Laya decision model uses in training, applied to diagnostics rather than single decisions.

**Consistency as free supervision.** Unperturbed public-domain stories are almost always consistent on the facts the extractor can see. A pipeline that derives must-violations on them is wrong somewhere. That gives a large, label-free negative signal in every language.

The schedule is: supervised training of each model; then semantic-loss fine-tuning; then end-to-end fine-tuning through the logic on perturbed and clean stories; then calibration.

### 11.9 Data, without generative models

**Sources.** Public-domain fiction with clear provenance: Project Gutenberg (many languages, English-heavy), Wikisource (wide language coverage, including Bengali), and public-domain sections of national digital libraries. LitBank (CC BY 4.0) gives English annotations for entities, coreference and quotations. FlawedFictions is used **only for evaluation**, never for training. Every training document carries its licence and source. Copyrighted books are not used without a licence, and writers' manuscripts are never used.

**Perturbation operators.** The engine's own pipeline locates facts, events and time expressions in a public-domain story. Edits then inject errors with known gold locations, using only operations that work in any language:

| Operator | What it does | Gold label |
|---|---|---|
| Value substitution | Replace an established value's span with a sibling concept from the same lattice, lexicalised in the same language | Attribute violation at the edited span, conflicting with the establishing span |
| Lifecycle move | Move a sentence in which a character acts to after their death scene, or move the death earlier | Lifecycle violation |
| Time edit | Change a numeral in a time expression, or swap two scenes with anchors | Temporal violation with the negative cycle as explanation |
| Identity swap | Replace a mention with another cast member's name | Presence or knowledge violation |
| Reveal shift | Move a revelation after the first use of the knowledge | Knowledge leak |
| Route edit | Replace the destination of a journey with a distant place | Travel violation |

**Matched controls, the crucial part.** Every operator is also applied where it does **not** create a contradiction: substituting a value nobody established, moving a sentence with no fluent content, swapping scenes without temporal anchors. Without these controls a model learns to detect the edit (a seam, an odd word) instead of the contradiction. This is exactly the shortcut risk raised for the FlawedFictions generator.

**Fluency filtering without generation.** Substitutions can break agreement in inflected languages. Each candidate edit is scored with masked-language-model pseudo-log-likelihood (Salazar et al. 2020), using the same encoders, and edits that drop fluency sharply are discarded.

**Adversarial filtering.** AFLite (Le Bras et al. 2020) trains many weak models on cheap features and removes examples they solve too easily, so that what remains needs the intended reasoning.

**Human gold, per language.** 500–2,000 examples per supported language, double-annotated with adjudication, for evaluation and calibration. To keep annotation affordable, examples are chosen by **active learning**: uncertainty sampling and disagreement between models (query by committee). Annotation guidelines live with the data and are versioned.

**Self-supervised pretraining.** M6, and optional continued masked-language-model pretraining of the backbone, use raw public-domain text only.

**The importer as a teacher?** The importer's model (§8.5) could label public-domain fiction to produce silver training data for M1–M5, cutting annotation cost in Phase 3. That would make a generative model part of the training pipeline, a second exception to decision 2, so it is left as an open question (§17). Nothing in this section depends on it.

### 11.10 Backbones and languages

- **Multilingual encoders.** mmBERT (a multilingual ModernBERT, about 300M parameters at base size, most of them in the embedding table) or XLM-R, chosen per language by evaluation.
- **Byte-level fallback.** Scripts poorly covered by subword vocabularies (Chakma and many other low-resource scripts) use a byte- or character-level encoder (CANINE, or the encoder half of ByT5). Byte-level models also make offset mapping back to Unicode scalars trivial.
- **Vocabulary extension.** For a language that matters but is poorly tokenised, extend the vocabulary and continue masked-language-model pretraining on public-domain text in that language.
- **Shared trunk, small heads.** One backbone per family with small task heads and optional per-language adapters, so installing a language adds megabytes, not gigabytes.
- **Distillation and quantisation.** Each task is trained on a larger teacher and distilled into a student of roughly 100–150M parameters, exported to ONNX with 8-bit weights.

### 11.11 Running the models

- **Runtime.** ONNX Runtime through the `ort` crate, one session per model with a shared thread pool. Optional hardware acceleration through DirectML on Windows and Core ML on macOS; CPU is the baseline every feature must meet.
- **Scheduling.** A priority queue of blocks: visible blocks first, then the rest of the open document, then the project in the background. Blocks are batched per model.
- **Caching.** Results are cached by (block fingerprint, model id, version) (§7.4), so re-opening a project costs nothing and an unchanged paragraph is never re-run.
- **Budgets.** Targets to verify in Phase 3: under 60 ms per block per model on a mid-range laptop CPU at 8-bit precision, and a full 100,000-word manuscript processed in the background within a few minutes.
- **Determinism.** Fixed thread counts and deterministic kernels where available; results are cached, so repeated analysis of the same input reuses identical outputs.
- **Shipping.** Each model ships with a model card: training data and licences, languages, calibration parameters, known weaknesses, and evaluation results. Model versions are salsa inputs, so upgrading a model invalidates exactly its cached outputs.

### 11.12 Model summary

| Model | Input | Output to the IR | Deployed size (target) | Priority |
|---|---|---|---|---|
| M1 mentions and aliases | Block text, nearby cast | Mentions, links, coreference scores | ~100–150M, shared trunk | Phase 2 |
| M2 event-calculus parser | Clauses, predicate schema | Observes, initiates and terminates atoms with polarity | Head on shared trunk | Phase 3 |
| M3 temporal anchoring | Time expressions, scene pairs | Normalised slots, relation distributions, duration quantiles | Head on shared trunk | Phase 3 |
| M4 framing and attribution | Clauses, quotes | Contexts, speakers, flashback flags | Head on shared trunk | Phase 3 |
| M5 canon-conditioned verifier | Text window plus up to $K$ facts | Per-fact relation and evidence span | ~100–150M, separate | Phase 3 |
| M6 narrative state predictor | Entity mentions per scene | Calibrated surprise atoms | Research | Phase 5 |

---

## 12. Diagnostics, explanations, and the story language server

### 12.1 The diagnostic record

```text
Diagnostic {
  id:            stable hash of (code, evidence set)      // survives re-analysis
  code:          "CANON-E013"                              // namespaced, never reused
  severity:      error | warning | info | hint
  tier:          must | may                                // §9.3
  probability:   calibrated P(real problem)                // §10.8
  primary:       Anchor                                    // where the red mark goes
  related:       [Anchor + role]                           // the other side(s) of the conflict
  explanation:   minimal conflict set, ordered as a chain  // §10.4, §10.6
  culprit:       ranked diagnoses with probabilities       // §10.6
  repairs:       ranked abductive fixes                    // §10.7
  provenance:    rules, packs, model versions, revision (seq, chain hash)
  stale:         bool                                      // revision moved on
}
```

The **id** is derived from the code and the identities of the evidence (fact ids and anchor block ids), not from offsets, so the same problem keeps the same id as text moves around it. That stability is what lets suppressions, baselines and "new since snapshot" work.

### 12.2 Severity and codes

Severity is computed, not hand-assigned per rule:

| | Must (forced by constraints) | May (possible under some ordering) |
|---|---|---|
| Settled versus settled | **Error** | Warning |
| Settled versus observed or proposed | Warning (error in strict mode) | Info |
| Proposed versus proposed | Info, shown only in "thorough" mode | Hidden |

Codes follow the namespaces of the planning archive, kept stable forever:

| Range | Area | Examples |
|---|---|---|
| CANON-E0xx | Attributes and values | E013 functional-fluent violation (today's rule, generalised) |
| CANON-E1xx | Lifecycles | E101 action in an invalid lifecycle state (generalised to items and places) |
| CANON-E2xx | Time | E201 backward jump inside a run declared continuous (redefined, §9.6); E202 inconsistent temporal network; E203 effect before cause |
| CANON-E3xx | Space and travel | E301 impossible journey |
| CANON-E4xx | Knowledge | E401 knowledge used before acquisition |
| CANON-E5xx | Relations | E501 genealogy cycle; E502 exclusive relations overlap |
| CANON-E6xx | World rules and invariants | E601 invariant violated; E602 pack rule violated |
| NAME-W0xx | Symbols | W001 unknown name; W002 used before introduction; W003 confusable names |
| PLOT-I0xx | Ledger | I001 pending setup (not yet paid off) |

Changing E201's meaning is a deliberate break: today it marks every flashback. The new definition keeps the number for "reading order fights story time" but only where the author declared continuity, and adds E202 for genuine temporal inconsistency.

### 12.3 Explanations that read like a good editor

An explanation is the minimal conflict set (§10.4) rendered as a short chain, most authoritative statement first:

```text
CANON-E013 · Mira's hair was settled as auburn.            (error, must, p = 0.97)
  ① Settled in chapter 2: "Mira · hair · auburn"
  ② Chapter 19 says: "Mira pushed her black hair out of her eyes."
  ③ No event between day 3 and day 19 changes her hair.
  Most likely slip: ② (91%)
  Fixes: Add an event · Auburn holds until a date · Mark ② as a dream · Keep my sentence
```

Explanation text is produced from **templates** per code, localised with ICU MessageFormat into the interface language. No text is generated by a model, so explanations are identical across runs and translatable by ordinary localisation.

### 12.4 The story language server

The engine exposes itself through a protocol shaped like the Language Server Protocol, adapted from text documents to Dynodoc blocks. Desk, Word task pane, Docs sidebar and the CLI are all clients. On desktop the transport is in-process through Tauri commands; the local HTTP API on `127.0.0.1:4317` carries the same messages for the browser and the add-ins, with server-sent events for push notifications, the pattern Dynodoc already uses for its event stream.

| LSP method | Story counterpart | What the author sees |
|---|---|---|
| `publishDiagnostics` | `story/diagnostics` (per block, as diffs) | Red marks in the margin |
| `codeAction` | `story/repairs` | "Add an event · Mark as dream · Keep my sentence" |
| `hover` | `story/factCard` | Hovering "Mira" shows her settled facts at this point in story time |
| `definition` | `story/whereSettled` | Jump to where a fact was settled |
| `references` | `story/mentions` | Every mention of an entity, aliases included |
| `rename` | `story/rename` | Rename a character everywhere, previewed, applied only when the author confirms |
| `inlayHint` | `story/inferredFacts` | Optional ghost text: inferred facts the author has not settled |
| `codeLens` | `story/sceneSummary` | Above a scene: 4 facts settled, 1 conflict, day 17 |
| `semanticTokens` | `story/entityTokens` | Subtle colouring of names by sort |
| `workspace/symbol` | `story/castSearch` | Jump to any character, place or item |
| `$/progress`, `$/cancelRequest` | Same | Background analysis progress; cancellation on new edits |

Rename is the one operation that changes prose, and only as an explicit, previewed author action, like find-and-replace. It produces ordinary Dynodoc patch events, so it is undoable and appears in history.

### 12.5 Git-like features

Dynodoc's event log, hash chain and snapshots make several version-control features straightforward. Dynodoc today has no blame algorithm (its "who changed this" is a client-side filter), so the engine builds the pieces it needs.

**Blame for prose.** For each block, maintain an interval map from text ranges to the event that last wrote them. Each `RichTextPatched` event updates the map by mapping existing intervals through the patch (§6.1) and inserting the new interval for inserted text. Building it for a block is linear in the block's events; it is cached per block fingerprint.

**Blame for canon.** Every settled fact records the event (`seq`, actor, time) that settled or last changed it. "Who decided Mira's hair was auburn, and when?" is a lookup.

**Bisect for diagnostics.** When a mark exists now but not at an earlier revision, find the event that introduced it by binary search over `seq`:

1. Materialise the project at the midpoint revision from the nearest snapshot plus the event tail, as Dynodoc already does.
2. Run the engine at that revision. Perception outputs come from the cache (§7.4) for every block whose fingerprint is unchanged, so a probe is cheap.
3. Narrow the interval; after $O(\log \Delta\mathit{seq})$ probes, report the introducing event: "introduced by your edit to chapter 19 on 3 October".

**Baselines.** Freeze the diagnostic ids at a named snapshot (a Dynodoc `document_version`) and show only marks that are new since. This is how CI tools adopt a checker on an existing codebase without drowning in old warnings, and it is how an author starts using Asten on a half-finished novel.

**Drafts as canon branches.** A draft is analysed against canon as it exists in that draft. Facts settled inside a draft are scoped to it until merged.

**Semantic merge conflicts.** Two drafts can each be consistent and still contradict each other once merged: one moves Oren's death to chapter 7, the other gives him a speech in chapter 8. Text merge cannot see this; software engineers call it a semantic merge conflict. When Dynodoc previews a merge (its `sync` preview returns the merged state and its operations), the engine analyses the merged state and reports diagnostics that exist after the merge but in neither draft. Optionally, as with Dynodoc's `validate_integrity` gate, a project can require these to be resolved before merging.

**Reproducibility.** Every diagnostic records the revision as `(seq, chain_hash)`. A report about a wrong mark can be reproduced exactly by materialising that revision.

### 12.6 Suppressions that know when to come back

Today a suppression hides a rule for a document or entity forever. The new suppression binds to:

- the diagnostic **code**;
- the **scope**: this occurrence, this entity, this rule in this document, or this rule in the project;
- a **fingerprint of the evidence**: the ids of the facts and anchors in the minimal conflict set;
- the author's **reason**.

If the evidence changes (the settled fact is edited, the sentence is rewritten, a new conflicting statement appears), the suppression becomes **stale** and the mark returns for review, labelled as previously suppressed with the old reason. This is the narrative version of linters that report unused suppression comments, and it keeps "I meant that" from silently hiding a real slip introduced later.

---

## 13. Language agnosticism

### 13.1 What depends on language

| Layer | Language-dependent? | How it generalises |
|---|---|---|
| Story IR, all analyses (§9), probabilistic layer (§10), diagnostics (§12) | **No** | Work on ids, concepts, ticks and anchors |
| Calendars, numbers, units, quotation marks | Data-driven | ICU4X and CLDR data per locale |
| Segmentation | Data-driven | UAX #29 plus ICU4X dictionary and LSTM segmenters |
| Deterministic mention matching | Mostly no | Aho–Corasick over author-entered names; per-language affix tables or a learned segmenter |
| Value concepts and lexicalisations | Data | Multilingual lexicons plus the author's own value lists |
| Models M1–M6 | Yes, through training data | Multilingual or byte-level backbones, per-language calibration |
| Explanation text | Interface language, not manuscript language | Localised templates |

### 13.2 Support tiers

Every language gets the exact layer on day one, because it runs on what the author has settled. Models add coverage as each language is trained.

| Tier | What works | Requirement |
|---|---|---|
| **C: canon** | All checks between settled facts, structured dates and author-entered events: attributes, lifecycles, timelines, travel, genealogy, invariants | Unicode text. Any language. |
| **B: deterministic perception** | Plus mentions of known names and aliases with morphology handling, unknown-name hints in scripts with case | An affix table or a trained segmenter for the language |
| **A: full perception** | Plus extracted observations, events, time relations, contexts, the verifier, ranking | Trained and calibrated models with a gold evaluation set |

The interface shows the tier for each language in a project, so a writer in Chakma knows that settled-fact checks are active and extraction is not, rather than discovering silence.

The bootstrap importer (§8.5) cuts across the tiers: in a language without tier A models it can still propose a starting canon, at whatever quality the importer's model reaches in that language. The import report states that quality per language, and anything the writer accepts is checked at tier C from then on.

### 13.3 Text-handling checklist

- Offsets in Unicode scalars internally (to match Dynodoc patches), grapheme clusters at every user-facing boundary, converted in one module with property tests.
- NFC storage; matching forms per script; never case-fold scripts without case.
- Word and sentence segmentation through ICU4X, including dictionary and LSTM segmenters for Chinese, Japanese, Burmese, Khmer, Lao and Thai.
- Native digits in every script; numerals parsed with CLDR numbering systems.
- Quotation conventions per locale; dialogue dashes.
- Mixed-script names and transliteration-aware name distance (§9.1).
- Right-to-left scripts handled purely at display; logic sees logical order.
- **Per-block language identification**, so a manuscript in Bengali with English dialogue routes each block to the right frontend. A small character n-gram classifier is enough and is itself a tiny discriminative model.

### 13.4 Adding a language

1. Collect public-domain fiction in the language, with licences recorded.
2. Run the perturbation pipeline (§11.9) with lexicon entries for the language's value concepts.
3. Annotate a gold set of 500–2,000 examples, chosen by active learning.
4. Train per-language adapters or heads on the shared backbone, or extend the vocabulary if tokenisation is poor; fall back to the byte-level backbone otherwise.
5. Calibrate, set thresholds with Learn then Test, and publish a model card with the language's numbers.
6. Move the language from tier B to tier A.

---

## 14. Evaluation

### 14.1 Metrics

| Metric | Why | Target to set in Phase 3 |
|---|---|---|
| False-discovery proportion of shown marks, per severity | Trust is the product | Bounded by $\alpha$ with probability $1 - \delta$ (§10.8) |
| Recall on injected errors, per error class and language | Coverage | Reported per class; no single aggregate hides weak classes |
| Localisation: both spans of the conflict | The mark must land on the right sentence | Exact block match and token overlap |
| Explanation minimality and correctness | Explanations must be the real reason | Checked against gold conflict sets |
| Repair acceptance (local telemetry is off; measured in studies) | Repairs must be useful | User studies with consenting writers |
| Calibration error per language | Honest probabilities | Expected calibration error under 0.05 after calibration |
| Latency, p50 and p95, per edit type | The IDE feel | 50 ms exact layer; model results within seconds |
| Import quality: grounded precision, canon recall, cast clustering, review effort | A bad import costs the writer hours | Measured per language on gold import sets; injected conflicts must all survive (§8.5) |

### 14.2 Datasets

- **Perturbation suites** per language from held-out public-domain stories, with matched controls.
- **Human gold sets** per language for calibration and thresholds.
- **FlawedFictions and FlawedFictionsLong** (Ahuja et al. 2025), external, evaluation only. Their published bars: best model 0.76 accuracy and 0.67 CEEval-Full on short stories (the same as human annotators); about 0.61 and 0.53 on long stories; a plain MNLI entailment baseline at 0.53 and 0.04.
- **TRIP** (Storks et al. 2021) for physical-state conflicts in short stories, if its licence allows.
- **LitBank** for extraction quality in English (entities, coreference, quotations).

### 14.3 Testing the exact core

- **Unit tests** per analysis, written against hand-made evidence atoms (§8.4).
- **Property-based tests** (`proptest`): the STN solver against brute force on small random networks; the event-calculus sweep against naive simulation; lattice laws (associativity, commutativity and idempotence of meet); anchor mapping against "apply the patch, then find the quote".
- **Metamorphic tests.** Diagnostics must not change, except for names in messages, when: entities are renamed; every story time is shifted by a constant; an irrelevant paragraph is inserted; the canon's display labels are translated into another language while concept ids stay the same.
- **Golden tests** for diagnostic output on fixture projects.
- **Fuzzing** of patch application and anchor mapping with random edit sequences.

### 14.4 Ablations

Each component must earn its place on the evaluation sets:

1. Exact layer on settled facts only.
2. Plus deterministic mentions.
3. Plus M2–M5 with exact rules, no probabilistic layer.
4. Plus the HL-MRF ranking and diagnosis.
5. Plus end-to-end training through the logic.
6. Plus M6 surprise atoms.

### 14.5 Performance

Synthetic projects at 10k, 100k and 250k words and at multi-book scale, generated with realistic distributions of entities, facts and scenes. Recorded edit traces (sequences of Dynodoc events from real writing sessions, with consent, content stripped and replaced by synthetic text) are replayed to measure per-edit latency distributions rather than single runs.

### 14.6 The FlawedFictions experiment

1. Re-run current frontier models on both splits with the public evaluation code to establish today's bar.
2. **Oracle ceiling:** hand-extract the facts of 50 stories into the IR and run the exact engine. If localisation is high with perfect facts, invest in extraction. If it is low, the benchmark's plot holes need common sense beyond the rules, and M6 or a different approach is required for that slice.
3. Run the full pipeline. Report accuracy and CEEval-Full on both splits, with the long-story split as the headline.

---

## 15. Pathways compared

| Pathway | What it is | Strengths | Weaknesses | Verdict |
|---|---|---|---|---|
| **P1. Exact core only** | Settled facts plus deterministic mentions, all of §9 | Every language on day one; deterministic; explainable; fast to ship | Only checks what authors settle; low recall for gardeners | **Build first** |
| **P2. Neuro-symbolic with probabilistic logic** | P1 plus models M1–M5 as perception, HL-MRF ranking, diagnosis, abduction | High recall with bounded false alarms; explainable; local; language-scalable | Per-language data and models; more moving parts | **Target architecture** |
| **P3. End-to-end encoder detector** | One encoder reads a story and flags contradictions | Simple to train | Long-context weakness; poor localisation; opaque; language-hungry | Baseline and auxiliary ranking feature only |
| **P4. Learned reasoning over the IR** | Graph neural networks or neural theorem proving over the entity-event graph instead of hand-written rules | Learns soft patterns rules miss | Weaker explanations; needs much data | Research, after P2 (§16 Phase 5) |
| **P5. Small generative model trained for the task** | For example, a 7B model trained with reinforcement learning, as in LiteReason (about 0.87 accuracy on FlawedFictions short stories) | Strong on short stories today | Generative, heavy for a laptop, non-deterministic, uneven language coverage | Excluded as a detector; used only as a comparison number (contrast P7, where a generative model only proposes facts) |
| **P6. Cloud language models** | API calls | Strong general reasoning | Sends manuscripts off the machine; cost; non-deterministic; language-uneven | Excluded |
| **P7. Bootstrap importer** | A small local generative model (for example Gemma 4 E2B or E4B) reads an existing draft once, chunk by chunk, and proposes a starting canon; grounding and reconciliation are deterministic and the writer reviews (§8.5) | A starting canon on day one for writers who arrive with a draft; many fact types without per-type training data; available before M1–M5 | Slow (minutes to an hour or more per novel); multi-gigabyte optional download; crude and uneven across languages | **Adopted** as an optional, one-time bridge; never in the reasoning or the per-edit loop |

The recommended order is P1, then P7 as an early bridge for existing drafts, then P2, with P4 and M6 as research tracks.

---

## 16. Roadmap

Effort figures assume two to three engineers (Rust and logic; machine learning) plus part-time annotators. They are estimates to be revised at each gate.

### Phase 0: Foundations (4–6 weeks)

- Spike: run dynodoc-engine's pure modules (materialiser, rich-text patches, merge, diff) on SQLite behind a storage trait; decide the adoption path.
- One `BlockId` type; block-relative anchors with patch mapping and quote re-attachment; migrate existing `SourceAnchor`s.
- `asten-ir` types; the salsa database skeleton; existing rules ported as queries.
- Synthetic project generator and the benchmark harness.
- **Gate:** today's three rules run incrementally with stable anchors; latency measured at 250k words.

### Phase 1: Exact core (8–10 weeks)

- Calendars and ticks; the temporal network with negative-cycle explanations; Allen relations in point form.
- Event calculus with change events and contexts; typestate lifecycles; value lattices and units.
- Symbol diagnostics; Datalog rules through `ascent`; Dwyer-pattern invariants (first five patterns).
- Suppressions 2.0; explanations; story language server v1 (diagnostics, hover, definition, references, rename).
- **Gate:** tier C works in any language; flashbacks no longer marked; 50 ms target met.

### Phase 2: Deterministic perception, the importer, and the data pipeline (8–11 weeks)

- Aho–Corasick matching with affix tables for the first languages (English and Bengali, plus one more script family); alias clustering with author constraints.
- Perturbation pipeline with matched controls; first gold sets; evaluation harness; M1 v1.
- Bootstrap importer (§8.5): document import into Dynodoc blocks, chunking, schema-constrained extraction, quote grounding, deterministic reconciliation, the import review queue; gold import sets from public-domain novels; runtime choice benchmarked.
- **Gate:** tier B in the first languages; mention precision and recall reported per language; importer grounded precision and cast clustering measured on gold import sets, every injected conflict surviving import, and a 100,000-word novel importing in the background without blocking editing.

### Phase 3: Models and probabilistic ranking (12–16 weeks)

- M2–M5 trained, distilled and exported; scheduling and caching.
- HL-MRF ranking; diagnosis; abductive repairs; calibration and Learn-then-Test thresholds.
- **Gate:** tier A in the first languages; shown-mark false-discovery proportion within $\alpha$ on gold sets; latency budgets met on CPU.

### Phase 4: Breadth (8–12 weeks)

- Knowledge flow; travel and maps; genealogy; the full invariant interface; merge-preview checks; bisect and blame; baselines.
- More languages through §13.4.

### Phase 5: Research (ongoing)

- M6 narrative state predictor; end-to-end training through the logic; learned reasoning over the IR (P4).
- The FlawedFictions experiment (§14.6) and publication of the event-calculus parser, the verifier and the evaluation suites.

---

## 17. Risks and open questions

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Extraction quality is poor in low-resource languages | High | Tier A slips for those languages | Tier C and B still deliver value; byte-level backbones; active learning to make annotation cheap |
| False alarms erode trust | Medium | High | Severity from evidence, calibrated thresholds with guarantees, may/must, stale-aware suppressions |
| Models learn perturbation artefacts | Medium | High | Matched controls, adversarial filtering, human gold sets, FlawedFictions as an external check |
| Too many solvers to maintain | Medium | Medium | One IR, one Datalog engine, one temporal network; travel and genealogy reduce to time constraints |
| Porting the Dynodoc event model to SQLite is harder than expected | Medium | Medium | Phase 0 spike; the canon engine depends only on block ids, fingerprints and patches |
| Series-scale performance | Medium | Medium | Scoped recomputation first; differential dataflow if profiling demands |
| Authors rarely settle facts | Medium | Medium | One-click settle from proposals; inlay hints; the probabilistic layer still ranks proposals |
| Model footprint is too large for some machines | Low | Medium | Shared trunk, distillation, 8-bit weights, per-language installation |
| Licensing | Low | High | Use the MIT dynodoc-engine, not AGPL Dynodoc code; record licences for every corpus |
| Research models do not pay off | Medium | Low | They are additive, behind evaluation gates |
| Importer proposals are wrong often enough to make review a chore | Medium | Medium | Quote grounding drops unsupported facts; cast review first; bulk accept by support; rejections remembered; cast-only fallback per language |
| Importer smooths over contradictions in the draft | Low | High | Reconciliation is deterministic and never picks a winner; injected-conflict test is a Phase 2 gate (§8.5) |
| Importer is too slow or too large for some machines | Medium | Low | Optional download; background, resumable job; cast-only import needs no model |

**Open questions** for review:

1. What is the scene unit: binder documents, scene breaks inside them, or both?
2. How do authors express story time in the interface without friction (dates, "day N", relative to another scene)?
3. Who curates the default concept lattices, and how do packs extend them?
4. How deep should epistemic nesting go before it becomes author-modelled only?
5. Answer-set programming for joint repairs: depend on a C++ solver through FFI, or keep repairs to bespoke search?
6. When, if ever, to adopt differential dataflow for the logic layer?
7. Evaluation with real manuscripts: is a consented, evaluation-only programme compatible with the AI policy, and how is consent recorded?
8. Should the importer's model also act as a teacher, labelling public-domain fiction to train M1–M5 (§11.9)? It would cut annotation cost in Phase 3 but adds a generative model to the training pipeline.
9. Importer runtime: llama.cpp through Rust bindings, or ONNX Runtime's generative extension to stay on one runtime with `ort`? Decide by benchmark in Phase 2.
10. How do we word the importer for writers? It fits the brand pillars as written ([`BRAND.md`](BRAND.md): Asten never generates prose, anything AI touches is a proposal the author confirms, local, no training on your words), but it is the first model in the product. The copy must say it reads the draft and proposes facts, writes nothing, and is optional, and the site must call it planned until it ships.

---

## 18. References

- Aho, A. V., & Corasick, M. J. (1975). Efficient string matching: an aid to bibliographic search. *Communications of the ACM* 18(6).
- Ahuja, K., et al. (2025). *Finding Flawed Fictions: Evaluating Complex Reasoning in Language Models via Plot Hole Detection.* arXiv:2504.11900.
- Allen, J. F. (1983). Maintaining knowledge about temporal intervals. *Communications of the ACM* 26(11).
- Angelopoulos, A. N., Bates, S., Candès, E. J., Jordan, M. I., & Lei, L. (2021). *Learn then Test: Calibrating predictive algorithms to achieve risk control.* arXiv:2110.01052.
- Angelopoulos, A. N., Bates, S., Fisch, A., Lei, L., & Schuster, T. (2022). *Conformal risk control.* arXiv:2208.02814.
- Bach, S. H., Broecheler, M., Huang, B., & Getoor, L. (2017). Hinge-loss Markov random fields and probabilistic soft logic. *JMLR* 18.
- Balestriero, R., & LeCun, Y. (2025). *LeJEPA: Provable and scalable self-supervised learning without the heuristics.* arXiv:2511.08544.
- Bamman, D., Popat, S., & Shen, S. (2019). An annotated dataset of literary entities (LitBank). *NAACL*.
- Bansal, N., Blum, A., & Chawla, S. (2004). Correlation clustering. *Machine Learning* 56.
- Bauer, A., Leucker, M., & Schallhart, C. (2011). Runtime verification for LTL and TLTL. *ACM TOSEM* 20(4).
- Clark, J. H., Garrette, D., Turc, I., & Wieting, J. (2022). CANINE: pre-training an efficient tokenization-free encoder for language representation. *TACL* 10.
- Conneau, A., et al. (2020). Unsupervised cross-lingual representation learning at scale (XLM-R). *ACL*.
- Cousot, P., & Cousot, R. (1977). Abstract interpretation: a unified lattice model for static analysis of programs. *POPL*.
- Dalvi, B., et al. (2018). Tracking state changes in procedural text (ProPara). *NAACL*.
- De Giacomo, G., & Vardi, M. Y. (2013). Linear temporal logic and linear dynamic logic on finite traces. *IJCAI*.
- de Kleer, J. (1986). An assumption-based TMS. *Artificial Intelligence* 28(2).
- de Kleer, J., & Williams, B. C. (1987). Diagnosing multiple faults. *Artificial Intelligence* 32(1).
- De Raedt, L., Kimmig, A., & Toivonen, H. (2007). ProbLog: a probabilistic Prolog and its application in link discovery. *IJCAI*.
- Dechter, R., Meiri, I., & Pearl, J. (1991). Temporal constraint networks. *Artificial Intelligence* 49.
- Denning, D. E. (1976). A lattice model of secure information flow. *Communications of the ACM* 19(5).
- Dwyer, M. B., Avrunin, G. S., & Corbett, J. C. (1999). Patterns in property specifications for finite-state verification. *ICSE*.
- Fang, C., Yu, P., & Williams, B. C. (2014). Chance-constrained probabilistic simple temporal problems. *AAAI*.
- Gneiting, T., & Raftery, A. E. (2007). Strictly proper scoring rules, prediction, and estimation. *JASA* 102.
- Google DeepMind (2026). Gemma 4 open models: E2B, E4B, 26B mixture-of-experts, 31B dense; Apache 2.0.
- Green, T. J., Karvounarakis, G., & Tannen, V. (2007). Provenance semirings. *PODS*.
- Guo, C., Pleiss, G., Sun, Y., & Weinberger, K. Q. (2017). On calibration of modern neural networks. *ICML*.
- Gupta, A., Mumick, I. S., & Subrahmanian, V. S. (1993). Maintaining views incrementally. *SIGMOD*.
- Gurung, A., Malkin, N., & Lapata, M. (2025). *Lightweight Latent Reasoning for Narrative Tasks* (LiteReason). arXiv:2512.02240.
- Kakas, A. C., Kowalski, R. A., & Toni, F. (1992). Abductive logic programming. *Journal of Logic and Computation* 2(6).
- Kam, J. B., & Ullman, J. D. (1977). Monotone data flow analysis frameworks. *Acta Informatica* 7.
- Kildall, G. A. (1973). A unified approach to global program optimization. *POPL*.
- Kowalski, R., & Sergot, M. (1986). A logic-based calculus of events. *New Generation Computing* 4.
- Le Bras, R., et al. (2020). Adversarial filters of dataset biases. *ICML*.
- Li, Z., Huang, J., & Naik, M. (2023). Scallop: a language for neurosymbolic programming. *PLDI*.
- Marone, M., Weller, O., Fleshman, W., Yang, E., Lawrie, D., & Van Durme, B. (2025). *mmBERT: a modern multilingual encoder with annealed language learning.* arXiv:2509.06888.
- McSherry, F., Murray, D. G., Isaacs, R., & Isard, M. (2013). Differential dataflow. *CIDR*.
- Mokhov, A., Mitchell, N., & Peyton Jones, S. (2018). Build systems à la carte. *ICFP*.
- Myers, E. W. (1986). An O(ND) difference algorithm and its variations. *Algorithmica* 1.
- Nebel, B., & Bürckert, H.-J. (1995). Reasoning about temporal relations: a maximal tractable subclass of Allen's interval algebra. *JACM* 42(1).
- Planken, L., de Weerdt, M., & van der Krogt, R. (2010). Incrementally solving STNs by enforcing partial path consistency. *ICAPS*.
- Przymusinski, T. C. (1988). On the declarative semantics of deductive databases and logic programs. In *Foundations of Deductive Databases and Logic Programming*.
- Reiter, R. (1987). A theory of diagnosis from first principles. *Artificial Intelligence* 32(1).
- Richardson, M., & Domingos, P. (2006). Markov logic networks. *Machine Learning* 62.
- Sahebolamri, A., Gilray, T., & Micinski, K. (2022). Seamless deductive inference via macros (the `ascent` Datalog engine). *CC*.
- Salazar, J., Liang, D., Nguyen, T. Q., & Kirchhoff, K. (2020). Masked language model scoring. *ACL*.
- Shanahan, M. (1999). The event calculus explained. In *Artificial Intelligence Today*, LNCS 1600.
- Siek, J. G., & Taha, W. (2006). Gradual typing for functional languages. *Scheme and Functional Programming Workshop*.
- Storks, S., Gao, Q., Zhang, Y., & Chai, J. (2021). Tiered reasoning for intuitive physics (TRIP). *Findings of EMNLP*.
- Strom, R. E., & Yemini, S. (1986). Typestate: a programming language concept for enhancing software reliability. *IEEE TSE* 12(1).
- Tang, L., Laban, P., & Durrett, G. (2024). MiniCheck: efficient fact-checking of LLMs on grounding documents. *EMNLP*.
- Tarjan, R. (1972). Depth-first search and linear graph algorithms. *SIAM Journal on Computing* 1(2).
- Tsamardinos, I. (2002). A probabilistic approach to robust execution of temporal plans with uncertainty. *SETN*.
- Wadler, P., & Findler, R. B. (2009). Well-typed programs can't be blamed. *ESOP*.
- Williams, R. J. (1992). Simple statistical gradient-following algorithms for connectionist reinforcement learning (REINFORCE). *Machine Learning* 8.
- Xu, J., Zhang, Z., Friedman, T., Liang, Y., & Van den Broeck, G. (2018). A semantic loss function for deep learning with symbolic knowledge. *ICML*.
- Xue, L., et al. (2022). ByT5: towards a token-free future with pre-trained byte-to-byte models. *TACL* 10.
- Zadrozny, B., & Elkan, C. (2002). Transforming classifier scores into accurate multiclass probability estimates (isotonic calibration). *KDD*.
- Zaratiana, U., et al. (2024). GLiNER: generalist model for named entity recognition using bidirectional transformer. *NAACL*. And GLiNER2 (2025), arXiv:2507.18546.
- Unicode Consortium. UAX #29 (Text Segmentation), UTS #39 (Security Mechanisms, confusables). ICU4X and CLDR.
- Software: `salsa` (salsa-rs, the incremental engine behind rust-analyzer); `ascent`; ONNX Runtime and the `ort` crate; BookNLP (Bamman).
- Verne. dynodoc-engine 0.1.0 (MIT): `README.md`, `docs/ARCHITECTURE.md`, `VERSIONING-CONFORMANCE.md`.

---

## Appendix A. A worked example

The numbers below are illustrative; they show the mechanics, not measured results.

**The project.** Chapter 2 settles *Mira · hair_color · auburn* (status settled, context $W$, valid from day 3). Chapter 9 settles *Captain Oren · status · dead* at day 14. Chapter 12, set on day 16, contains: *"Mira cut and blackened her hair in the dyer's yard."* Chapter 14, set on day 20, contains: *"Oren poured the tea and said nothing."* The author has not dated chapter 14 relative to Oren's death in any other way.

**Perception (tier A).**

| Atom | Source | Confidence |
|---|---|---|
| `Initiates(ε₁₂, hair_color(Mira, black), day 16)` | M2 on chapter 12 | 0.86 |
| `Context(ch14 scene, memory)` | M4 | 0.22 |
| `Link("Oren", Captain Oren)` | M1 | 0.97 |
| `Acts(Oren, pours tea, day 20, N)` | M2 | 0.91 |
| `TimeRel(ch14 scene, ch9 death scene, >)` | M3 | 0.74 |

**Exact layer.** The event calculus terminates `auburn` at day 16 and initiates `black`, so a chapter 19 observation of black hair (day 22) is consistent. **No mark.** Without chapter 12 there would have been a must-violation of E013.

Oren's lifecycle automaton is in `dead` from day 14. The `Acts` atom at day 20 in narration projects into $W$, so the typestate check derives an E101 candidate. Because the time placement comes from a model with confidence 0.74, the violation is **may** under the hard network and its strength is left to the probabilistic layer.

**Probabilistic layer.** The HL-MRF combines the act's confidence (0.91), the link (0.97), the time relation (0.74), and the low memory-context probability (0.22, so narration is likely). The MAP violation value is about 0.64. The calibration map trained on gold data turns it into $P = 0.71$.

**Threshold.** The project uses standard strictness; the warning threshold certified by Learn then Test for English is 0.55. The mark is shown as a **warning**.

**Diagnosis.** Minimal conflict: {settled death at day 14; act at day 20; time relation}. Fault priors: settled death 0.01, act in narration 0.09, time relation 0.26. The best diagnosis blames the **time relation** (perhaps the scene is earlier than it looks), then the act.

**Repairs** (tropical semiring costs):

1. Mark the chapter 14 scene as a memory (cost low; M4 already gave it 0.22).
2. Place chapter 14 before day 14 (cost moderate; moves one scene).
3. Move Oren's death later (cost high; settled).
4. Keep my sentence (always available).

**What the author sees.**

```text
CANON-E101 · Oren acts after a settled death.             (warning, may, p = 0.71)
  ① Settled in chapter 9: Captain Oren dies on day 14.
  ② Chapter 14 (placed around day 20): "Oren poured the tea and said nothing."
  Most likely cause: the scene's time is uncertain.
  Fixes: Mark as a memory · Set the scene's day · Move the death · Keep my sentence
```

---

## Appendix B. Rule sketches

Illustrative Datalog in an `ascent`-like notation. Relations ending in `_w` carry weights or semiring annotations in the probabilistic layer; the exact layer reads only settled inputs.

```text
// Inertia and change (event calculus, §9.3), evaluated per context in time order
holds(F, T)      <-- initially(F), !clipped(0, F, T), tick(T).
holds(F, T)      <-- happens(E, T1), initiates(E, F, T1), T1 < T, !clipped(T1, F, T).
clipped(T1,F,T2) <-- happens(E, T), terminates(E, F, T), T1 < T, T < T2.

// Functional fluents auto-terminate other values
terminates(E, fl(P, X, V2), T) <-- initiates(E, fl(P, X, V1), T), functional(P),
                                    value_of(P, V2), V1 != V2.

// Observation violations (E013), with lattice compatibility
violation_e013(O, X, P, V1, V2, T) <-- obs(O, fl(P, X, V1), T, C), holds_in(C, fl(P, X, V2), T),
                                        functional(P), incompatible(V1, V2).

// Lifecycle (E101) via typestate
state_at(X, dead, T) <-- happens(E, T0), dies(E, X), T0 <= T, !resurrected_between(X, T0, T).
violation_e101(X, A, T) <-- acts(A, X, T, C), projects_to_world(C), state_at(X, dead, T).

// Knowledge leak (E401)
knows(X, S, T) <-- learns(X, S, T0), T0 <= T.
knows(X, S, T) <-- public(S, T).
violation_e401(X, S, T) <-- uses_knowledge(X, S, T), !knows(X, S, T), !deduces(X, S, T).

// Genealogy cycle (E501), with the transitive closure
ancestor(A, B) <-- parent_of(A, B).
ancestor(A, C) <-- parent_of(A, B), ancestor(B, C).
violation_e501(A) <-- ancestor(A, A).
```

---

## Appendix C. Crates and interfaces

| Crate | Responsibility |
|---|---|
| `asten-ir` | Story IR types: ids, anchors, entities, fluents, events, contexts, evidence, constraints |
| `asten-engine` | Salsa database: inputs, tracked queries, revisions, cancellation |
| `asten-perceive` | Segmentation, deterministic matching, model sessions through `ort`, perception cache |
| `asten-logic` | Datalog rules (`ascent`), event calculus sweep, typestate, symbols, invariants |
| `asten-time` | Calendars, ticks, STN with explanations, Allen point relations, travel constraints |
| `asten-prob` | Provenance semirings, HL-MRF with ADMM, diagnosis, abduction, calibration |
| `asten-ls` | Story language server protocol, transports, scheduling |
| `asten-diagnostics` | Diagnostic codes, records, severities, suppressions (refactored from today's crate) |
| `asten-storage` | SQLite canon store, Dynodoc event model adapter, caches |
| `asten-import` | Bootstrap importer: document import, chunking, schema-constrained extraction through a swappable local runtime, grounding, reconciliation, the review queue (§8.5) |

Key interfaces (illustrative Rust):

```rust
/// A language frontend turns one block into evidence. Implemented per language family.
pub trait Frontend: Send + Sync {
    fn languages(&self) -> &[LanguageTag];
    fn tier(&self) -> SupportTier; // A, B or C
    fn perceive(&self, block: &BlockText, cast: &CastView) -> Vec<Evidence>;
}

/// Every analysis reads the IR and emits candidate violations with provenance.
pub trait Analysis: Send + Sync {
    fn codes(&self) -> &[DiagnosticCode];
    fn run(&self, ir: &IrView, scope: Scope) -> Vec<Violation>;
}

/// The probabilistic layer ranks, blames and repairs.
pub trait Reasoner {
    fn rank(&mut self, violations: &[Violation], evidence: &EvidenceView) -> Vec<RankedViolation>;
    fn diagnose(&self, violation: &RankedViolation) -> Vec<Diagnosis>;
    fn repairs(&self, violation: &RankedViolation) -> Vec<Repair>;
}
```
