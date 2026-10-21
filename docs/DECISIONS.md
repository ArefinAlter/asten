# Decisions

The decisions that shape Asten, each with the context it was made in, the
choice, what it costs, and its status. Newer entries sit at the bottom. A
decision is changed by adding a new entry that supersedes it, not by editing
the old one.

Format: **Context** (what was true), **Decision** (what we chose), **Consequences**
(what it buys and what it costs), **Status**.

---

## D1. The engine is deterministic; no generative model, ever

*September 2026*

**Context.** The product promise is "let the machine be the editor, not the
writer". Writers who are curious about AI but protective of their voice are the
audience. A contradiction checker that is sometimes confidently wrong trains
the author to ignore every mark.

**Decision.** Every mark comes from a rule over a Story IR the author can
inspect. Perception reads the page with Unicode folding, dictionaries and cue
words. Trained models, when they arrive, are small local classifiers and
taggers that only propose; the logic that judges contradictions stays
rule-based. No generative model runs anywhere in the product, local or hosted,
and no manuscript is used to train one.

**Consequences.** Same project, same marks, every time; every mark explains
itself. The cost is coverage: a pronoun with no name in the sentence is not
weighed, and languages beyond the lexicon's cues get Tier C checks only.

**Status.** Accepted. Reaffirmed each time an "AI feature" is proposed.

## D2. The block model is Dynodoc's, carried into Rust

*October 2026*

**Context.** Anchors (a fact's link to the sentence that settled it) must
survive edits. Diffs of whole ProseMirror documents cannot map offsets.
`dynodoc-engine` (MIT) had the right primitive: a stable id per block, a
hash-chained log of block events, rich-text patches with retain/delete/insert
ops. `dynodoc` the application is AGPL and off limits.

**Decision.** Reimplement the block model as `asten-blocks` from the MIT
reference: canonical JSON fingerprints, fractional positions, rich-text
patches with a deterministic diff, a hash-chained event log per document,
materialisation, three-way merge, anchor remapping, blame. No AGPL code is
read or copied.

**Consequences.** Anchors follow text through edits, history is verifiable, and
drafts merge. The cost is a diff on every commit; D11 bounds it.

**Status.** Accepted.

## D3. Gradual canon with epistemic contexts

*October 2026*

**Context.** Authors contradict themselves on purpose: lies, dreams, unreliable
narrators, frame stories. A checker that calls a lie a contradiction is useless
for fiction.

**Decision.** Facts carry a status (settled, proposed, deprecated, rejected)
and every statement lives in a context (world, narration, dialogue, belief,
memory, dream, vision, frame, hypothetical). Only world and reliable narration
project onto the world; quoted speech is what a character says, not what is
true. Perception's readings arrive as proposals and never become canon
silently.

**Consequences.** A dream chapter raises nothing; a narrator can be marked
unreliable; a rumour can be recorded as a rumour. The cost is a context field
on everything and a settings switch for narrator reliability.

**Status.** Accepted.

## D4. Severity by pairing, tiers, single-fault priors, explanation chains

*October 2026*

**Context.** Settled against settled is the author's own contradiction; the
page against canon may be a slip or a planned change; the page against itself
is a hint. Treating all three alike buries the first.

**Decision.** Severity follows the pairing (error, warning, information). Rules
have a must tier (certain) and a may tier (depends on confidence). Each
conflict names the statement most likely at fault using provenance priors.
Messages read as a numbered chain (①②③) and end with repairs the author owns
(add an event, mark the scene a memory, retire a fact, keep my sentence).

**Consequences.** Marks are ranked and legible without a glossary. The cost is
a catalogue to maintain and priors to tune.

**Status.** Accepted.

## D5. Flashbacks are not errors

*October 2026*

**Context.** The first clocks rule compared neighbouring dated scenes in
reading order and marked every step backward. Fiction is full of deliberate
steps backward.

**Decision.** Story time is argued only inside a run of scenes the author
declared continuous (`CANON-E201`), when the dates cannot all be true
(`CANON-E202`, from a temporal network), or when an effect is seen before its
cause (`CANON-E203`). A scene marked as a memory or flashback is never weighed
against the world.

**Consequences.** No suppressing a mark on every flashback. The cost is one more
field on a scene (its continuity run) and an STN solve per analysis.

**Status.** Accepted. Supersedes the neighbour comparison described on the
public site before 12 October.

## D6. One transport: the desktop embeds the HTTP API

*October 2026*

**Context.** The scaffold planned Tauri commands for the desktop and HTTP for
the browser, with "transport parity" as a goal. Two transports drift.

**Decision.** The desktop (and mobile) shell starts the same axum API on
loopback, pairs its own webview with a desk token, and the React client uses
HTTP everywhere. Exactly two Tauri commands remain: `desk_session` and
`start_pairing`. Anything else the desk needs about the machine comes through
a route (`GET /desk`).

**Consequences.** A feature that works in the browser works on the desktop by
construction; the Word and Docs guests use the same API. The cost is a loopback
server inside the app and the authentication layer it needs (D7).

**Status.** Accepted. Reaffirmed by the client parity work (PR 3).

## D7. Pairing codes by default; token kinds carry authority

*October 2026*

**Context.** A loopback API with no authentication is readable by any web
page the author visits. The product has no accounts and no cloud.

**Decision.** `ASTEN_AUTH=pairing` by default: a six-digit single-use code,
exchanged once for a bearer token hashed at rest. Desk and owner tokens may
manage pairing; guest tokens read and write the story only. The SSE stream
uses a sixty-second single-use ticket instead of a token in the URL. Session
cookies are set only same-origin. `off` is refused on any bind beyond loopback.
An optional argon2id passphrase serves self-hosting.

**Consequences.** Word, Docs and a browser pair once and are revocable from
the Guests panel. The cost is a pairing step on first use and a rate limiter.

**Status.** Accepted. See `docs/SECURITY.md`.

## D8. Local-first: one SQLite file, no sync, no accounts, no telemetry

*September 2026*

**Context.** Verne should never receive a manuscript. Writers must be able to
leave with their words.

**Decision.** Every project lives in one SQLite file in the OS app-data
folder (or wherever the API is pointed). Compile exports to open formats at
any time. No account, no sync service, no analytics anywhere in the product or
the public site.

**Consequences.** Backup is copying a file; collaboration beyond drafts waits
for a design that keeps offline work. The cost is no cross-device sync.

**Status.** Accepted.

## D9. TypeScript contracts are generated from Rust and drift-checked

*October 2026*

**Context.** The handwritten contracts had drifted from the Rust types in a
dozen places (missing fields, wrong optionality, `unknown` payloads).

**Decision.** Every serde-derived contract type derives `ts_rs::TS` behind a
`ts` feature; `pnpm contracts:gen` writes `packages/contracts/src/generated`;
`drift.ts` asserts each handwritten desk contract is mutually assignable with
its generated twin; CI regenerates and fails on a diff. An OpenAPI document
for the routes is deferred.

**Consequences.** A field changed on one side fails the build. The cost is a
generation step after Rust type changes and `bigint` rewritten to `number`.

**Status.** Accepted.

## D10. Incrementality by fingerprint cache, not a query framework

*October 2026*

**Context.** The engine design proposed salsa for incremental computation.
The analyses are whole-project and fast; perception is the expensive part and
is per block.

**Decision.** Perception results are cached by block fingerprint plus the
inputs that affect them (cast, lexicon, context). One edit re-perceives one
block. Analyses run whole-project on every refresh.

**Consequences.** Under a second per keystroke-commit on a 250k-word project
in a debug build, with no framework dependency. Revisit if an analysis becomes
quadratic in project size.

**Status.** Accepted. Revisit when Tier A models arrive.

## D11. Bounded algorithms on author input

*October 2026*

**Context.** Review found a Myers diff with memory quadratic in paragraph
length, a calendar that walked year by year on a twelve-digit year, a
zero-day month that looped forever, and arithmetic that could overflow.

**Decision.** Every algorithm fed by author input has a bound: the diff caps
edit distance and falls back to a rewrite; calendar years are closed-form and
calendars are validated; story-time numbers are capped at nine digits; ticks
from the API are clamped; fuzzy anchor reattachment caps the quote and the
block.

**Consequences.** Pathological input degrades a result instead of taking the
process down. The cost is that an extreme rewrite is recorded as a replace
rather than a minimal patch.

**Status.** Accepted.

## D12. One desk for every screen; the shell builds for every processor

*October 2026*

**Context.** The desktop built for Windows x64 only and the layout assumed
960px of width. Writers use Macs on Apple silicon, Linux laptops, Windows on
ARM, tablets and phones.

**Decision.** The release matrix builds Windows x64 and ARM64, one universal
macOS binary, and Linux x86_64 and aarch64, with per-platform Tauri config
files. The same shell crate compiles for Android and iOS (`cdylib`,
`staticlib`); the platform projects are generated on a machine with the SDK.
The desk is one React app with a responsive shell: a four-column desk above
1280px, a sheet for the inspector below it, and a single column with a drawer
and bottom bar below 960px. No separate mobile app.

**Consequences.** Every view exists on every screen; a fix lands everywhere
once. The cost is layout work per view and the absence, so far, of device
testing for ARM and phones.

**Status.** Accepted (PRs 1 and 2).

## D13. An installable web app before store apps

*October 2026*

**Context.** Store listings take review cycles and the mobile shells need SDKs
the project has not run yet. Writers already own a computer that can host the
API on the home network.

**Decision.** The browser desk ships a web-app manifest, icons and a service
worker that caches only the shell, so a phone can install the desk from the
browser and pair with a code. The API is never cached. Store apps follow when
the mobile shells are built and tested.

**Consequences.** Phones and tablets reach the whole desk today, on the LAN.
The cost is that the phone needs the home machine awake; the mobile shell (D12)
is the offline answer.

**Status.** Accepted (PR 3).

## D14. Guests do not edit and do not administer

*October 2026*

**Context.** The Word pane and Docs sidebar sit beside a document that Word or
Docs owns. A guest token lives in an iframe or task pane the author does not
fully control.

**Decision.** Guests list projects, show the Scales and file a selection as a
new scene. They never edit the manuscript in Asten and never manage pairing,
tokens or the passphrase; they can revoke only themselves.

**Consequences.** One source of truth for the open document; a compromised pane
cannot mint persistence. The cost is that repairs and bisect stay in the desk.

**Status.** Accepted. See `docs/PARITY.md`.

## D15. Perception attributes within the sentence; no pronoun resolution yet

*October 2026*

**Context.** "Mara looked up. Her blue eyes caught the light." reads the eye
colour in a sentence with no name.

**Decision.** An observation is attributed to a name in the same sentence (the
nearest mention). Pronouns are not resolved. The limit is documented on the
public site and in the handoff rather than papered over with a guess.

**Consequences.** Fewer false attributions; some true ones missed. Pronoun
resolution is a Tier A model's job when models arrive (D1).

**Status.** Accepted, with the limit stated.

## D16. The public site says what ships and what is planned, never blurring them

*October 2026*

**Context.** After the engine landed, the site described three rules and no
reading of the page.

**Decision.** Features, FAQ, glossary, audience pages, roadmap and reference
are updated in the same change as the code, and the roadmap keeps four lanes:
Shipped, Next, Later, Never. "Never" lists promises, not plans.

**Consequences.** Readers can trust the site; every claim has a shipped
counterpart. The cost is site edits in feature PRs.

**Status.** Accepted.
