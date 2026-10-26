# The HTTP API

Asten has one transport. The browser desk, the desktop shell's webview, the
Word task pane and the Docs sidebar all call the same axum service, normally
at `http://127.0.0.1:4317`, under `/api/v1`. Nothing it does reaches any other
server.

The API describes itself. `GET /api/v1/openapi.json` serves an OpenAPI 3.1
document generated at compile time from the annotation on every handler and
the types in the contract crates; the same document is committed as
[`docs/openapi.json`](openapi.json) and `cargo test -p asten-api` fails when
the committed copy falls behind the code. `node scripts/openapi-export.mjs`
regenerates it and the route table at the end of this page. Load either copy
into any OpenAPI viewer or client generator.

`docs/SECURITY.md` is the threat model behind everything below.

## Start the API

```sh
cargo run -p asten-api
```

The API binds `127.0.0.1:4317`, opens `data/asten.db`, and, when no client is
paired yet, prints a six-digit pairing code that is valid for ten minutes.
`ASTEN_BIND`, `ASTEN_DATABASE_PATH`, `ASTEN_AUTH` and the other variables are
listed in the README. The desktop shell starts the same API on a loopback port
of its own and pairs its webview itself.

Two routes answer without a token. `/health` says the service is up;
`/api/v1/auth/status` says which mode it runs in and whether anyone is paired:

```sh
curl -s http://127.0.0.1:4317/health
curl -s http://127.0.0.1:4317/api/v1/auth/status
```

```json
{ "mode": "pairing", "owner_configured": false, "paired_clients": 0, "requires_auth": true }
```

## Pair

Exchange the code for a bearer token. The label is what the desk's Guests
panel shows for this client.

```sh
curl -s -X POST http://127.0.0.1:4317/api/v1/auth/pair \
  -H 'content-type: application/json' \
  -d '{ "code": "123456", "label": "my laptop" }'
```

```json
{
  "token": "ast_…",
  "token_id": "019a…",
  "kind": "desk",
  "label": "my laptop",
  "cookie": false
}
```

The token is shown once; only its hash is stored. Keep it somewhere private
(an environment variable for the examples below). A code grants a token of a
stated kind: the bootstrap code printed at first start grants `desk`; codes
minted under **Guests** grant `guest` unless the desk asked for another desk.

When the request is same-origin (the page was served by the API itself, or by
a proxy in front of both) the response also sets an `asten_session` cookie
(`HttpOnly`, `SameSite=Strict`) and `cookie` is `true`; a browser desk on
another origin keeps its bearer token instead.

```sh
export ASTEN_TOKEN='ast_…'
```

## Call

Every other route wants the token in the `Authorization` header (or the
session cookie). Responses are JSON; lists come back as bare arrays; creation
answers `201`, deletion `204`.

```sh
API=http://127.0.0.1:4317/api/v1
AUTH="authorization: Bearer $ASTEN_TOKEN"

# Who am I, and what does this API run on?
curl -s $API/auth/me -H "$AUTH"
curl -s $API/desk -H "$AUTH"

# A project and a scene.
PROJECT=$(curl -s -X POST $API/projects -H "$AUTH" -H 'content-type: application/json' \
  -d '{ "name": "The Harbor", "kind": "novel", "description": null }' | jq -r .id)
CREATED=$(curl -s -X POST $API/projects/$PROJECT/documents -H "$AUTH" -H 'content-type: application/json' \
  -d '{ "kind": "scene", "title": "Arrival", "parent_id": null }')
DOCUMENT=$(echo "$CREATED" | jq -r .id)
HEAD=$(echo "$CREATED" | jq -r .head_seq)   # 1: creating a document is its first event

# Write, with optimistic concurrency: the commit is refused with 409 when the
# document moved past the head sequence you last saw (the response carries the
# new `head_seq`).
curl -s -X PUT $API/projects/$PROJECT/documents/$DOCUMENT -H "$AUTH" -H 'content-type: application/json' \
  -d "{ \"content_json\": { \"type\": \"doc\", \"content\": [ { \"type\": \"paragraph\",
        \"content\": [ { \"type\": \"text\", \"text\": \"Mira had black hair.\" } ] } ] },
        \"expected_head_seq\": $HEAD }"

# What the engine thinks.
curl -s $API/projects/$PROJECT/story/diagnostics -H "$AUTH"
# A fact card needs an entity in the cast (create one under Cast, or POST /entities).
curl -s "$API/projects/$PROJECT/story/fact-card?entity=mira" -H "$AUTH"
```

Errors share one shape, with the HTTP status repeated in `error.code`:

```json
{ "error": { "code": "404", "message": "project was not found" } }
```

| Status | Meaning                                                                 |
| ------ | ----------------------------------------------------------------------- |
| `400`  | A path parameter is not a valid id. The body is plain text, not the envelope above. |
| `401`  | No valid token, cookie or ticket. `WWW-Authenticate: Bearer` is set.    |
| `403`  | A guest token asked for something only a desk or owner session may do.  |
| `404`  | The project, document or other record does not exist.                   |
| `409`  | A concurrency or state conflict: a stale `expected_head_seq`, a draft already merged, a passphrase already set. |
| `422`  | The input did not validate; the message says what.                      |
| `408`  | The request took longer than thirty seconds.                            |
| `413`  | The body is larger than 10 MiB.                                         |

Every response carries an `x-request-id`, `Cache-Control: no-store` and the
security headers from `docs/SECURITY.md`. A browser may call the API only from
an origin on the allowlist (`ASTEN_ALLOWED_ORIGINS`).

## Token kinds and what they may do

| Kind      | Issued by                              | May manage pairing, tokens, passphrase | Reads and writes the story |
| --------- | -------------------------------------- | -------------------------------------- | -------------------------- |
| `desk`    | The bootstrap code, or a desk code     | yes                                    | yes                        |
| `session` | `POST /auth/login` in owner mode       | yes                                    | yes                        |
| `guest`   | A code minted under **Guests**         | no (`403`); may revoke only itself     | yes                        |

A desk mints codes for guests and lists or revokes tokens:

```sh
curl -s -X POST $API/auth/pairing-codes -H "$AUTH" -H 'content-type: application/json' \
  -d '{ "label": "Word on the study machine", "grants": "guest" }'
curl -s "$API/auth/tokens?include_revoked=true" -H "$AUTH"
curl -s -X DELETE $API/auth/tokens/$TOKEN_ID -H "$AUTH"
curl -s -X POST $API/auth/logout -H "$AUTH"
```

## The change stream and its ticket

`GET /projects/{project_id}/stream` is a Server-Sent Events stream of
`revision`, `diagnostics`, `document`, `canon` and `heartbeat` events, each
carrying a `StreamEvent` as JSON data, plus `lagged` (data `{}`) when the
client fell behind and should refetch, and a `ping` comment every twenty
seconds. The first event is a `heartbeat`, so the connection is known to be up
before the first change.

With a client that can set headers, the bearer token works as everywhere else:

```sh
curl -N $API/projects/$PROJECT/stream -H "$AUTH"
```

A browser `EventSource` cannot set headers, and a token must never travel in a
URL. So an authenticated call first mints a **stream ticket**: random, hashed
at rest, bound to one project, valid for sixty seconds and spent on first use.
Only the ticket goes in the query string, and a proxy's access log holds
nothing worth stealing.

```sh
TICKET=$(curl -s -X POST $API/projects/$PROJECT/stream-ticket -H "$AUTH" | jq -r .ticket)
curl -N "$API/projects/$PROJECT/stream?ticket=$TICKET"
```

```js
const { ticket } = await fetch(`${api}/projects/${project}/stream-ticket`, {
  method: "POST",
  headers: { authorization: `Bearer ${token}` },
}).then((r) => r.json());
const events = new EventSource(`${api}/projects/${project}/stream?ticket=${ticket}`);
events.addEventListener("diagnostics", (e) => render(JSON.parse(e.data)));
```

A page served by the API itself can open the stream with the session cookie
alone. A spent or expired ticket is answered with `401`; mint another.

## Owner mode

`ASTEN_AUTH=owner` adds a passphrase for a browser that has no way to pair,
typically the API hosted for yourself behind TLS (`ASTEN_COOKIE_SECURE=true`).

```sh
# Once, while no passphrase exists. In other modes this needs a desk token.
curl -s -X POST $API/auth/owner/setup -H 'content-type: application/json' \
  -d '{ "passphrase": "at least twelve characters" }'

# A thirty-day session token; same-origin callers also get the cookie.
curl -s -X POST $API/auth/login -H 'content-type: application/json' \
  -d '{ "passphrase": "at least twelve characters" }'

# Change it: the current passphrase and a desk or session token.
curl -s -X PUT $API/auth/owner/passphrase -H "$AUTH" -H 'content-type: application/json' \
  -d '{ "current": "…", "next": "…" }'
```

`/auth/pair`, `/auth/login`, `/auth/owner/setup` and `/auth/owner/passphrase`
are rate limited per client and per route; a client that guesses too fast
waits a minute.

## Routes

Generated from `docs/openapi.json` by `node scripts/openapi-export.mjs`.
`Auth` is `token` for routes behind the bearer token or cookie, `public` for
the few that need none, and `optional` for the one that is open only until a
passphrase exists. The schemas behind each body are in the document.

<!-- routes:start -->
<!-- Generated by scripts/openapi-export.mjs from docs/openapi.json: 97 operations, 167 schemas. Do not edit by hand. -->

| Tag | Method | Path | Summary | Auth |
| --- | --- | --- | --- | --- |
| desk | `GET` | `/api/v1/desk` | What this API runs on | token |
| desk | `GET` | `/api/v1/openapi.json` | This document | public |
| desk | `GET` | `/health` | Liveness check | public |
| auth | `POST` | `/api/v1/auth/login` | Log in with the owner passphrase | public |
| auth | `POST` | `/api/v1/auth/logout` | Log out | token |
| auth | `GET` | `/api/v1/auth/me` | Who am I | token |
| auth | `PUT` | `/api/v1/auth/owner/passphrase` | Change the owner passphrase | token |
| auth | `POST` | `/api/v1/auth/owner/setup` | Set the owner passphrase for the first time | optional |
| auth | `POST` | `/api/v1/auth/pair` | Redeem a pairing code | public |
| auth | `POST` | `/api/v1/auth/pairing-codes` | Mint a pairing code | token |
| auth | `GET` | `/api/v1/auth/status` | Authentication status | public |
| auth | `GET` | `/api/v1/auth/tokens` | List paired clients | token |
| auth | `DELETE` | `/api/v1/auth/tokens/{token_id}` | Revoke a token | token |
| auth | `POST` | `/api/v1/projects/{project_id}/stream-ticket` | Mint a stream ticket | token |
| projects | `GET` | `/api/v1/projects` | List projects | token |
| projects | `POST` | `/api/v1/projects` | Create a project | token |
| projects | `GET` | `/api/v1/projects/{project_id}` | Fetch one project | token |
| projects | `PUT` | `/api/v1/projects/{project_id}` | Rename a project | token |
| projects | `DELETE` | `/api/v1/projects/{project_id}` | Delete a project | token |
| projects | `GET` | `/api/v1/projects/{project_id}/export` | Export a project file | token |
| projects | `GET` | `/api/v1/projects/{project_id}/settings` | Read project settings | token |
| projects | `PUT` | `/api/v1/projects/{project_id}/settings` | Update project settings | token |
| projects | `POST` | `/api/v1/projects/bundle` | Open a project file | token |
| documents | `POST` | `/api/v1/projects/{project_id}/changes/{change_id}/resolve` | Accept or reject a tracked change | token |
| documents | `PUT` | `/api/v1/projects/{project_id}/comments/{comment_id}` | Edit or resolve a comment | token |
| documents | `GET` | `/api/v1/projects/{project_id}/documents` | List a project's documents | token |
| documents | `POST` | `/api/v1/projects/{project_id}/documents` | Create a document | token |
| documents | `GET` | `/api/v1/projects/{project_id}/documents/{document_id}` | Fetch one document with its body | token |
| documents | `PUT` | `/api/v1/projects/{project_id}/documents/{document_id}` | Update a document | token |
| documents | `DELETE` | `/api/v1/projects/{project_id}/documents/{document_id}` | Delete a document | token |
| documents | `GET` | `/api/v1/projects/{project_id}/documents/{document_id}/changes` | List tracked changes | token |
| documents | `POST` | `/api/v1/projects/{project_id}/documents/{document_id}/changes` | Record a tracked change | token |
| documents | `GET` | `/api/v1/projects/{project_id}/documents/{document_id}/comments` | List comments | token |
| documents | `POST` | `/api/v1/projects/{project_id}/documents/{document_id}/comments` | Add a comment | token |
| canon | `GET` | `/api/v1/projects/{project_id}/entities` | List canon entities | token |
| canon | `POST` | `/api/v1/projects/{project_id}/entities` | Create a canon entity | token |
| canon | `PUT` | `/api/v1/projects/{project_id}/entities/{entity_id}` | Update a canon entity | token |
| canon | `DELETE` | `/api/v1/projects/{project_id}/entities/{entity_id}` | Delete a canon entity | token |
| canon | `POST` | `/api/v1/projects/{project_id}/entities/{entity_id}/merge-into/{target_id}` | Merge one entity into another | token |
| canon | `GET` | `/api/v1/projects/{project_id}/events` | List timeline events | token |
| canon | `POST` | `/api/v1/projects/{project_id}/events` | Create a timeline event | token |
| canon | `PUT` | `/api/v1/projects/{project_id}/events/{event_id}` | Update a timeline event | token |
| canon | `DELETE` | `/api/v1/projects/{project_id}/events/{event_id}` | Delete a timeline event | token |
| canon | `GET` | `/api/v1/projects/{project_id}/facts` | List a project's facts | token |
| canon | `POST` | `/api/v1/projects/{project_id}/facts` | Create a fact | token |
| canon | `PUT` | `/api/v1/projects/{project_id}/facts/{fact_id}` | Update a fact | token |
| canon | `DELETE` | `/api/v1/projects/{project_id}/facts/{fact_id}` | Delete a fact | token |
| canon | `GET` | `/api/v1/projects/{project_id}/invariants` | List invariants | token |
| canon | `POST` | `/api/v1/projects/{project_id}/invariants` | Create an invariant | token |
| canon | `PUT` | `/api/v1/projects/{project_id}/invariants/{invariant_id}` | Replace an invariant | token |
| canon | `DELETE` | `/api/v1/projects/{project_id}/invariants/{invariant_id}` | Delete an invariant | token |
| canon | `GET` | `/api/v1/projects/{project_id}/proposals` | List proposed facts | token |
| canon | `GET` | `/api/v1/projects/{project_id}/routes` | List travel routes | token |
| canon | `POST` | `/api/v1/projects/{project_id}/routes` | Create a travel route | token |
| canon | `DELETE` | `/api/v1/projects/{project_id}/routes/{route_id}` | Delete a travel route | token |
| story | `GET` | `/api/v1/projects/{project_id}/baselines` | List baselines | token |
| story | `POST` | `/api/v1/projects/{project_id}/baselines` | Freeze the current marks as a baseline | token |
| story | `DELETE` | `/api/v1/projects/{project_id}/baselines/{baseline_id}` | Delete a baseline | token |
| story | `POST` | `/api/v1/projects/{project_id}/baselines/{baseline_id}/activate` | Activate a baseline | token |
| story | `POST` | `/api/v1/projects/{project_id}/baselines/clear` | Deactivate the baseline | token |
| story | `GET` | `/api/v1/projects/{project_id}/diagnostics` | Analyse a project | token |
| story | `POST` | `/api/v1/projects/{project_id}/entities/{entity_id}/rename` | Rename an entity | token |
| story | `GET` | `/api/v1/projects/{project_id}/story/bisect/{diagnostic_id}` | Find the edit that introduced a mark | token |
| story | `GET` | `/api/v1/projects/{project_id}/story/cast-search` | Search the cast | token |
| story | `GET` | `/api/v1/projects/{project_id}/story/catalog` | The diagnostic code catalogue | token |
| story | `GET` | `/api/v1/projects/{project_id}/story/diagnostics` | The full diagnostics snapshot | token |
| story | `GET` | `/api/v1/projects/{project_id}/story/fact-card` | An entity's fact card | token |
| story | `GET` | `/api/v1/projects/{project_id}/story/mentions` | Where an entity is mentioned | token |
| story | `POST` | `/api/v1/projects/{project_id}/story/repairs/apply` | Apply a repair | token |
| story | `GET` | `/api/v1/projects/{project_id}/story/scene-summary/{document_id}` | Summarise a scene | token |
| story | `GET` | `/api/v1/projects/{project_id}/story/where-settled` | Where a fact was settled | token |
| story | `GET` | `/api/v1/projects/{project_id}/stream` | Subscribe to a project's change stream | token |
| story | `GET` | `/api/v1/projects/{project_id}/suppressions` | List suppressions | token |
| story | `POST` | `/api/v1/projects/{project_id}/suppressions` | Silence a rule | token |
| story | `DELETE` | `/api/v1/projects/{project_id}/suppressions/{suppression_id}` | Lift a suppression | token |
| history | `GET` | `/api/v1/projects/{project_id}/documents/{document_id}/at/{seq}` | The document at a sequence number | token |
| history | `GET` | `/api/v1/projects/{project_id}/documents/{document_id}/blame` | Blame | token |
| history | `GET` | `/api/v1/projects/{project_id}/documents/{document_id}/drafts` | List drafts | token |
| history | `POST` | `/api/v1/projects/{project_id}/documents/{document_id}/drafts` | Branch a draft | token |
| history | `GET` | `/api/v1/projects/{project_id}/documents/{document_id}/history` | The document's event log | token |
| history | `GET` | `/api/v1/projects/{project_id}/documents/{document_id}/ops` | Legacy document operations | token |
| history | `GET` | `/api/v1/projects/{project_id}/documents/{document_id}/snapshots` | List snapshots | token |
| history | `POST` | `/api/v1/projects/{project_id}/documents/{document_id}/snapshots` | Take a snapshot | token |
| history | `GET` | `/api/v1/projects/{project_id}/documents/{document_id}/verify` | Verify the event log | token |
| history | `PUT` | `/api/v1/projects/{project_id}/drafts/{draft_id}` | Update a draft | token |
| history | `DELETE` | `/api/v1/projects/{project_id}/drafts/{draft_id}` | Delete a draft | token |
| history | `POST` | `/api/v1/projects/{project_id}/drafts/{draft_id}/merge` | Merge a draft | token |
| history | `POST` | `/api/v1/projects/{project_id}/drafts/{draft_id}/merge-preview` | Preview a three-way merge | token |
| history | `POST` | `/api/v1/projects/{project_id}/snapshots/{snapshot_id}/restore` | Restore a snapshot | token |
| world | `GET` | `/api/v1/projects/{project_id}/world-entries` | List world entries | token |
| world | `POST` | `/api/v1/projects/{project_id}/world-entries` | Create a world entry | token |
| world | `PUT` | `/api/v1/projects/{project_id}/world-entries/{entry_id}` | Update a world entry | token |
| world | `GET` | `/api/v1/projects/{project_id}/world-pulse` | The world's pulse | token |
| packs | `GET` | `/api/v1/packs` | List method packs | token |
| packs | `PUT` | `/api/v1/packs/{pack_id}` | Enable or disable a method pack | token |
| compile | `POST` | `/api/v1/projects/{project_id}/compile` | Compile the manuscript | token |
| compile | `POST` | `/api/v1/projects/{project_id}/import` | Import a document | token |

<!-- routes:end -->

## Keeping the document true

Every handler carries a `#[utoipa::path]` annotation beside the route it
serves, and every type on the wire derives `utoipa::ToSchema` behind the
`openapi` cargo feature of its crate, next to the `ts_rs::TS` derive that
feeds the TypeScript contracts. Three tests in `services/asten-api/src/openapi.rs`
hold the document to the code: one parses every `.route(…)` literal from the
handler modules and fails when a route lacks an annotation or an annotation
lacks a route; one checks that every protected route names the bearer scheme
and documents `401`; and one validates hand-written schemas (the internally
tagged enums utoipa cannot derive) against real serialised values. A fourth
compares `docs/openapi.json` with the generated document, so a change to a
route or a schema ends with `node scripts/openapi-export.mjs` and a commit of
the result.
