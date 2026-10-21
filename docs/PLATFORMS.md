# Platforms

Asten is one desk served three ways: a desktop shell that embeds the local
API, a browser desk that talks to that API, and two guests (Word, Google Docs)
that pair with it. This page is the support matrix and the build recipe for
each platform and processor.

## Support matrix

| Platform                   | Processor         | Desktop shell                 | Browser desk                     | Status                                     |
| -------------------------- | ----------------- | ----------------------------- | -------------------------------- | ------------------------------------------ |
| Windows 10 1809+ / 11      | x64               | NSIS and MSI installers       | Edge, Chrome, Firefox            | Built and checked on CI                    |
| Windows 11 on ARM          | ARM64             | NSIS installer                | Edge, Chrome                     | Built on CI (cross-compiled, untested on hardware) |
| macOS 11 Big Sur and later | Apple silicon, Intel | Universal `.app` and `.dmg` | Safari, Chrome, Firefox          | Built and checked on CI (unsigned)         |
| Linux (glibc 2.35+)        | x86_64            | `.deb`, `.rpm`, AppImage      | Firefox, Chromium                | Built and checked on CI                    |
| Linux                      | aarch64           | `.deb`, `.rpm`, AppImage      | Firefox, Chromium                | Release job present; needs an ARM runner   |
| Android 8+                 | arm64, x86_64     | Tauri app (see below)         | Chrome, as an installed web app  | Shell scaffold; project files generated on a machine with the SDK |
| iOS and iPadOS 15+         | arm64             | Tauri app (see below)         | Safari, as an installed web app  | Shell scaffold; project files generated on a Mac with Xcode |

The browser desk works anywhere a modern browser can reach the API. On a phone
or tablet it is the same desk in a one-column layout; see `docs/PARITY.md` for
what each client can and cannot do.

## What the desktop shell does

The shell opens `asten.db` in the operating system's application-data folder,
starts the API on `127.0.0.1:4317` (or any free port), pairs its own webview
with a desk token, and serves the built desk from the bundle. There is no
platform-specific code in the shell: paths come from Tauri's path resolver, the
API binds loopback the same way on every OS, and SQLite is bundled with the
binary. Processor differences do not reach the code; the Rust crates have no
SIMD or inline-assembly paths.

Per-platform packaging lives beside the main config:

- `apps/desktop/src-tauri/tauri.conf.json`: window, security policy, icons.
- `tauri.windows.conf.json`: NSIS per-user install, WebView2 bootstrapper, MSI.
- `tauri.macos.conf.json`: minimum macOS 11, DMG layout.
- `tauri.linux.conf.json`: Debian and RPM dependencies, AppImage.

## Building

All platforms:

```sh
pnpm install
pnpm --filter @asten/desktop build        # tauri build for the host platform
```

Cross-targets on the host that supports them:

```sh
# Windows ARM64 from an x64 Windows machine
rustup target add aarch64-pc-windows-msvc
pnpm --filter @asten/desktop exec tauri build --target aarch64-pc-windows-msvc --bundles nsis

# macOS universal binary (Apple silicon + Intel) from any Mac
rustup target add aarch64-apple-darwin x86_64-apple-darwin
pnpm --filter @asten/desktop exec tauri build --target universal-apple-darwin
```

Linux build dependencies (Debian and Ubuntu):

```sh
sudo apt-get install -y libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
  librsvg2-dev patchelf libxdo-dev libssl-dev build-essential
```

Fedora: `webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel openssl-devel`.

### Release workflow

`.github/workflows/release.yml` runs on a `v*` tag and builds every row of the
matrix above, attaching the installers to a draft GitHub release. The Linux
aarch64 job runs on `ubuntu-24.04-arm`, which private repositories must enable
under larger runners; until then that job is allowed to fail.

Signing is off by default. Add these repository secrets to turn it on:

| Secret                                                                              | Effect                                   |
| ----------------------------------------------------------------------------------- | ---------------------------------------- |
| `WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD`                               | Authenticode-signs the Windows installers |
| `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`         | Signs the macOS app                      |
| `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`                                       | Notarises the macOS app                  |

Unsigned builds install; Windows shows a SmartScreen warning and macOS asks
the user to allow the app under Privacy and Security.

## Android and iOS

Tauri 2 builds the same shell for phones. The Rust crate already exposes the
mobile entry point and the library types mobile needs (`cdylib` for Android,
`staticlib` for iOS). The generated platform projects are not committed because
they can only be created on a machine with the platform SDK:

```sh
# Android: Android Studio, SDK 34+, NDK, and JAVA_HOME set
pnpm --filter @asten/desktop android:init     # writes apps/desktop/src-tauri/gen/android
pnpm --filter @asten/desktop android:dev
pnpm --filter @asten/desktop android:build    # .apk and .aab

# iOS: a Mac with Xcode and an Apple developer team
pnpm --filter @asten/desktop ios:init         # writes apps/desktop/src-tauri/gen/apple
pnpm --filter @asten/desktop ios:dev
pnpm --filter @asten/desktop ios:build
```

On a phone the embedded API still binds loopback inside the app's sandbox, the
database lives in the app's data directory, and the desk opens in its
one-column layout. Word and Docs guests cannot pair with a phone, because the
phone's API is not reachable from another device; see `docs/PARITY.md`.

### Installed web app instead of a store app

Writers who host the API on their own computer can open the browser desk from
a phone on the same network and install it from the browser menu. The desk
ships a web-app manifest and icons for this. Pairing works the same way: type
the code from the desk's Guests panel once.

## Checks

CI checks the Tauri shell on Windows and Ubuntu on every push, and on macOS
weekly and on demand (Actions → CI → Run workflow), because macOS minutes bill
at ten times the Linux rate (`.github/workflows/ci.yml`, job `desktop`). Release
builds always include macOS. The Rust workspace and the desk are
checked on Ubuntu. Hardware testing for Windows on ARM and Linux aarch64 has not
been done; the builds compile and the code has no architecture-specific paths.
