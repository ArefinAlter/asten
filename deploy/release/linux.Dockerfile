# syntax=docker/dockerfile:1.7
#
# Linux x64 release build, run locally with Docker (no CI minutes needed):
#
#   docker build -f deploy/release/linux.Dockerfile \
#     --secret id=tauri_key,src=<path to asten-updater.key> \
#     --output type=local,dest=deploy/release/out/linux .
#
# Ubuntu 22.04 keeps the glibc floor at 2.35, as the CI release job does. The
# output holds release/ (the .deb, the AppImage and its updater signature, and
# asten-api as a .tar.gz, named by scripts/release-assets.mjs),
# bin/asten-prealpha (the hosted gateway) and desk/ (the hosted desk build).
# The updater key arrives as a BuildKit secret: it is read for one command and
# never lands in a layer.

FROM ubuntu:22.04 AS build
ENV DEBIAN_FRONTEND=noninteractive \
    CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup \
    PATH=/usr/local/cargo/bin:/usr/local/node/bin:$PATH \
    CARGO_BUILD_JOBS=4 \
    APPIMAGE_EXTRACT_AND_RUN=1 \
    CI=true
# dpkg without fsync: a throwaway build stage, and far faster on Docker Desktop.
RUN echo force-unsafe-io > /etc/dpkg/dpkg.cfg.d/unsafe-io \
    && apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates curl wget file xz-utils build-essential pkg-config patchelf \
      libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev libssl-dev \
    && rm -rf /var/lib/apt/lists/*
RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.99.0
RUN curl -fsSL https://nodejs.org/dist/v22.22.2/node-v22.22.2-linux-x64.tar.xz -o /tmp/node.tar.xz \
    && mkdir -p /usr/local/node && tar -xJf /tmp/node.tar.xz -C /usr/local/node --strip-components=1 \
    && rm /tmp/node.tar.xz && npm install -g pnpm@11.7.0

WORKDIR /src
COPY . .
RUN --mount=type=cache,id=asten-pnpm,target=/root/.local/share/pnpm/store \
    pnpm install --frozen-lockfile
RUN --mount=type=cache,id=asten-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=asten-linux-target,target=/src/target \
    --mount=type=secret,id=tauri_key,required=true \
    set -e; \
    export TAURI_SIGNING_PRIVATE_KEY="$(cat /run/secrets/tauri_key)"; \
    export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""; \
    pnpm --filter @asten/web build; \
    (cd apps/desktop/src-tauri && ../node_modules/.bin/tauri build --bundles deb,appimage \
      --config '{"build":{"beforeBuildCommand":""}}' < /dev/null); \
    unset TAURI_SIGNING_PRIVATE_KEY; \
    cargo build --locked --release -p asten-api -p asten-prealpha; \
    pnpm --filter @asten/web build:hosted; \
    mkdir -p /out/release /out/bin /out/desk; \
    node scripts/release-assets.mjs --platform linux-x64 --out /out/release; \
    cp target/release/asten-prealpha /out/bin/; \
    cp -r apps/web/dist-hosted/. /out/desk/

FROM scratch AS out
COPY --from=build /out/ /
