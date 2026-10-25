#!/bin/bash
# Runs inside an ubuntu:22.04 container capped at 4 CPUs and 8 GB, on the VPS.
# The source is at /src (a git archive), outputs go to /out. No signing key here:
# the AppImage is signed for the updater on the release machine afterwards.
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive CARGO_HOME=/cache/cargo RUSTUP_HOME=/cache/rustup
export PATH=/cache/cargo/bin:/cache/node/bin:$PATH CARGO_BUILD_JOBS=4 APPIMAGE_EXTRACT_AND_RUN=1 CI=true
export CARGO_TARGET_DIR=/cache/target npm_config_store_dir=/cache/pnpm-store
apt-get update -qq
apt-get install -y -qq --no-install-recommends ca-certificates curl wget file xz-utils build-essential pkg-config patchelf \
  libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev libssl-dev >/dev/null
[ -x /cache/cargo/bin/cargo ] || curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.99.0
if [ ! -x /cache/node/bin/node ]; then
  curl -fsSL https://nodejs.org/dist/v22.22.2/node-v22.22.2-linux-x64.tar.xz -o /tmp/node.tar.xz
  mkdir -p /cache/node && tar -xJf /tmp/node.tar.xz -C /cache/node --strip-components=1
  npm install -g pnpm@11.7.0 >/dev/null
fi
cd /src
pnpm install --frozen-lockfile --store-dir /cache/pnpm-store
pnpm --filter @asten/web build
(cd apps/desktop/src-tauri && ../node_modules/.bin/tauri build --bundles deb,appimage \
  --config '{"build":{"beforeBuildCommand":""},"bundle":{"createUpdaterArtifacts":false}}' < /dev/null)
cargo build --locked --release -p asten-api -p asten-prealpha
pnpm --filter @asten/web build:hosted
mkdir -p /out/release /out/bin /out/desk
node scripts/release-assets.mjs --platform linux-x64 --out /out/release --target /cache/target
cp /cache/target/release/asten-prealpha /out/bin/
cp -r apps/web/dist-hosted/. /out/desk/
ls -la /out/release /out/bin
echo BUILD-OK
