# Sources and scope

This is a development preview of an incomplete LiquidBounce port to the Horizon
Rust client for Minecraft 26.3. The actual declared module list is generated into
README.zh-CN.md, metadata/build-summary.json and package-manifest.json. An optional
successful runtime validation summary identifies the same executable by SHA-256.
This package does not claim parity with LiquidBounce's complete 236-module list.

- LiquidBounce 0.43.1 / nextgen, original source and original compiled theme:
  https://github.com/CCBlueX/LiquidBounce/tree/ae0eb396c07f22383789b28d5912d714c6ca0a86
  License: GPL-3.0; see LICENSE and licenses/LiquidBounce-GPL-3.0.txt.
- Horizon native client, commit 9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0:
  original https://github.com/horizon-rust/horizon;
  recovered source https://github.com/Jijjy/mc-horizon/tree/9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0.
  License: MIT; see licenses/Horizon-MIT.txt.
- CEF binary 152.0.6+g708dc14+chromium-152.0.7977.83, Windows x64 minimal:
  https://cef-builds.spotifycdn.com/index.html
  CEF uses its BSD license; Chromium dependency notices are reproduced in
  licenses/Chromium-CREDITS.html. See licenses/CEF-BSD.txt.
- Rust cef and cef-dll-sys 152.3.0+152.0.6:
  https://github.com/tauri-apps/cef-rs/tree/4e566e628a361f3ade6f9a211cb33ea3c7a9153f.
  The pinned original LiquidBounce CEF add-on and JCEF references are recorded in
  metadata/cef-source-versions.json. This port invokes CEF directly from Rust.

The source/ directory includes the local port's native/ sources and tests
(event core, module behaviors, Horizon adapter, HTTP/WebSocket interop and CEF
compositor), Cargo manifests and lockfiles, patches/, scripts/, root license and
README, source verification metadata and the integrated build's Cargo.lock.
metadata/build-summary.json and source/native-source-manifest.json record the
individual compiled source hashes and nativeSourceSha256. Packaging refuses a
source tree that changed after the successful integrated build.

source/research/ preserves the exact Horizon, LiquidBounce and Horizon-Mixins
source ZIP archives. source/source-baselines.json provides their full commits,
archive roots, SHA-256 hashes and backup download URLs. The bundled restore helper
uses the included archives first, verifies them and extracts the source baselines;
source/Rebuild-LiquidBounce.ps1 then rebuilds the original theme and local native
client using the recorded lockfile. See source/README-REBUILD.md. Rebuilding needs
the documented compiler tools and network access to Cargo/npm dependencies and
the CEF distribution. The source baselines restore from the included archives,
and the rebuild does not need the original local
workspace. The three included archives were checked against their recorded fixed
SHA-256 hashes before packaging. Other dependency caches and build output are not
copied from the developer's machine. If a source archive is absent, the restore
helper can download the exact pinned archive and verify the same digest.

No Mojang JAR, prepared game resources, pre-existing world, account, session,
configuration or browser cache is included. The launcher requires an explicit
path to resources prepared separately by the user, and creates its own user-data/
for mutable files. It starts the included executable directly and has no source
workspace path in its runtime configuration.
