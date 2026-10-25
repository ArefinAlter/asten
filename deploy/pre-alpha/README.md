# Hosted pre-alpha

The public pre-alpha of the workbench at <https://asten.103.191.50.32.sslip.io/>.
It runs the real Asten API and engine; nothing on it is scripted. A copy of
this file sits at `/opt/asten-prealpha/README` on the host.

## What runs

| Piece | Where | What it does |
| --- | --- | --- |
| `asten-prealpha-gateway` | container, network `asten-prealpha-inner` (internal: no route out) | `services/asten-prealpha`. One real Asten API ([`asten_api::router`]) per visitor, each on its own SQLite file; the `/svc/v1` state store; the hosted desk build (`apps/web`, `--mode hosted`). |
| `asten-prealpha-edge` | container, `inner` + `outer` | nginx; forwards `127.0.0.1:18640` to the gateway. The only published port. |
| host nginx | `/etc/nginx/sites-available/asten-prealpha` | TLS, rate limits (`limit_req`), 10 MB bodies, `X-Robots-Tag: noindex, nofollow`. Source: `nginx/asten-prealpha.site.conf`. |
| data | volume `asten-prealpha-data` | `visitors.db` (visitor hashes, stored state) and `sandboxes/<hash>.db`. |

Ports: 18640 (edge, loopback only). 18641 to 18649 are reserved and unused.
Limits: gateway 1.5 GB and 2 CPUs, edge 128 MB and half a CPU; logs rotate at
10 MB x 3.

## Visitors

- The first request without a live `asten_visitor` cookie creates a visitor:
  32 random bytes in an `HttpOnly; Secure; SameSite=Strict` cookie. Only the
  SHA-256 is stored. The browser never holds an API token.
- The gateway opens the visitor's own database with the real API in
  `pairing` mode and, as the desktop shell does at launch, revokes earlier
  desk tokens and issues a fresh one, which it presents on the visitor's
  behalf (`docs/SECURITY.md`, "Hosted pre-alpha").
- The desk creates the sample novel through the API on the first visit
  (`POST /svc/v1/claim/sample-novel` makes that happen once).
- Requests whose `Origin` is not this host, or that a browser marks
  `Sec-Fetch-Site: same-site`/`cross-site`, are refused with 403.
- **Expiry:** a visitor, their sandbox and their stored state are deleted 24
  hours after the first visit. The sweep runs every five minutes; there is no
  separate nightly reset and no backup. About -> Start over deletes them at once.
- **Quotas:** 400 live visitors; 20 new visitors per client address per hour (offices share one);
  48 MB per sandbox (writes beyond answer 507); 512 KB per state value, 4 MB
  and 64 keys per visitor; 48 sandboxes open in memory, idle ones closed after
  15 minutes.

## `/svc/v1` (the pre-alpha service)

What the API does not keep: workbench preferences (appearance, keybindings,
ribbon, page display), assistant threads, skills and access level, provider and
connector settings, console history, the last open page and folded binder
nodes. Provider keys are never sent; they stay in the tab's memory.

| Route | Meaning |
| --- | --- |
| `GET /svc/v1/session` | Created and expiry times, sandbox size, limits, and which parts run where |
| `GET /svc/v1/state` | `{ "items": { key: value } }` |
| `PUT /svc/v1/state/{key}` | Store one JSON value |
| `DELETE /svc/v1/state/{key}` | Forget one value |
| `POST /svc/v1/claim/{name}` | `{ "first": true }` once per visitor and name |
| `POST /svc/v1/reset` | Delete the sandbox and stored state now |

Everything else under `/api/v1` is the product's API, unchanged (`docs/API.md`).

## Build and deploy

Images and binaries are built off the VPS, with Docker on the release machine:

```sh
docker build -f deploy/release/linux.Dockerfile \
  --secret id=tauri_key,src=<updater key> \
  --output type=local,dest=deploy/release/out/linux .
mkdir -p deploy/pre-alpha/artifacts
cp deploy/release/out/linux/bin/asten-prealpha deploy/pre-alpha/artifacts/
cp -r deploy/release/out/linux/desk deploy/pre-alpha/artifacts/desk
docker build -t asten-prealpha:0.0.1a deploy/pre-alpha
docker save asten-prealpha:0.0.1a | gzip > asten-prealpha-0.0.1a.tar.gz
```

On the host, in `/opt/asten-prealpha` (holds `compose.yaml`, `nginx/edge.conf`,
this README):

```sh
gunzip -c asten-prealpha-0.0.1a.tar.gz | docker load
docker compose up -d
docker compose ps
docker compose logs --tail 50 gateway
```

Restart: `docker compose restart`. Stop: `docker compose down` (the volume,
and so live sandboxes, stays; `docker volume rm asten-prealpha-data` deletes
them all). nginx edits, `nginx -t`, reloads and certbot runs go under
`flock /tmp/verne-nginx.lock`.

## Local run

```sh
pnpm --filter @asten/web build:hosted
PREALPHA_DESK=apps/web/dist-hosted PREALPHA_DATA=data/prealpha cargo run -p asten-prealpha
# http://127.0.0.1:4318
```

or `pnpm --filter @asten/web dev:hosted` against a gateway on 4318.
