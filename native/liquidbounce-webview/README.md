# Original Chromium OSR integration

This crate renders actual CEF OnPaint pixels into Horizon's final GUI layer. It
does not draw a replacement ClickGUI or create a WebView2 overlay window. The
theme URL must come from the real `liquidbounce-interop` server and game backend.

`source-versions.json` records the pinned dependency chain: LiquidBounce CEF
add-on `v1.1.0+26.3`, its JCEF commit, and CEF
`152.0.6+g708dc14+chromium-152.0.7977.83`. Both Rust `cef` and `cef-dll-sys` are
fixed to `152.3.0+152.0.6`; Cargo.lock prevents a different native minor. The
stock CEF C ABI replaces JCEF's Java layer. CPU OSR avoids cef-rs's accelerated
texture importer and its different wgpu dependency. Chromium still renders the
original page; shared GPU handles are disabled and the full OnPaint image is
copied before CEF releases its buffer.

Implemented: transparent top-down BGRA premultiplied images, popup image/position
and removal, independent GPU textures without a 1024 atlas cap, resizing to the
real render target, per-device pipeline recreation, gamma-preserving alpha
blending, and Horizon's layered HDR opacity attachment. The compositor loads
and stores the supplied targets and preserves the retained world/HDR pass.
CEF initialization, separate subprocess helper, context-ready browser creation,
message pump, close callback and shutdown run on their owning thread.

InputQueue forwards mouse moves/buttons/wheel, raw Windows key events, all UTF16
text code units, focus, visibility and navigation to CefBrowserHost. The native
game adapter must intercept its own screen's inputs and supply Windows virtual
key/scancode values and CEF modifier bits. `gui_to_browser` converts Horizon GUI
coordinates once using its GUI scale; the CEF surface has device scale 1, as the
original browser's default. The adapter also supplies the original double-click
count and scroll units. IME preedit composition, drag/drop, custom OS cursors,
downloads/dialog parenting, accelerated paint and additional native browser windows are
not implemented here. These are remaining integration boundaries.

The screen policy selects independent Shared and ClickGUI Chromium pages through
`PageTarget`. `EnsurePage` preloads a hidden ClickGUI document without selecting
it. `SelectPage` reuses its browser and cached CPU surface. `ClosePage` destroys
the selected browser asynchronously, and `RestartPages` recreates existing
browsers with their current URLs and the requested paint rate. Every page owns
its own client, document diagnostics and pixels; only the selected page publishes
to the game surface. Shutdown also waits for browsers retired by close/restart.
`page_status_current_thread()` exposes native browser IDs, actual host frame
rates and observed creation/close counts.

Prepare the isolated staging copy first; the verified upstream source is never
edited:

```powershell
python scripts/stage-horizon-build.py --gpu-ui
native/liquidbounce-webview/build-native.ps1 -Typecheck
native/liquidbounce-webview/build-native.ps1 -Smoke
```

The native build uses project-local CMake 3.31.6 and Ninja 1.13.2, both checked
against published SHA-256 values, the installed Visual Studio x64 C++ tools,
and `build/cef-runtime` for the pinned native CEF download. `cef-typecheck`
enables upstream's `dox` feature and deliberately skips native linking. Use
`cef-runtime` or `cef-smoke` without `cef-typecheck` to build runnable binaries.
CEF's libcef.dll/resources and `liquidbounce-cef-helper.exe` must accompany the
game executable. This initial helper build follows cef-rs's unsandboxed native
integration (`no_sandbox = 1`); it does not install a JVM or JCEF.

`build-native.ps1 -PrepareOnly` prepares the same Visual Studio, CMake/Ninja,
CEF and Cargo target environment without starting Cargo. The integrated game
build can call it in its PowerShell process before its single workspace build.

Register on the game thread before the first GPU dispatch:

```rust
let layer = Box::leak(Box::new(liquidbounce_webview::horizon::Layer::new([1920, 1080], 0)));
liquidbounce_webview::runtime::initialize_current_thread(
    config, layer.mailbox.clone(), layer.input.clone(),
)?;
layer.register()?;
// The actual native screen-open/close hooks control this visibility and input capture.
layer.set_visible(true);
```

The layer pumps Chromium on real frames; generated frames reuse the latest GUI.
On game shutdown call `runtime::shutdown_current_thread(Duration::from_secs(5))`
and inspect its result. `runtime::Config.url` is the server's authenticated theme
URL, `helper_executable` names the compiled helper, and `resources_directory`
contains icudtl.dat, the .pak files and locales/. Cache/log files belong in an
explicit project/game cache directory.

Verification: the CEF + Horizon adapter Rust API type check passed. Three tests
passed, including a real DX12/Vulkan offscreen draw that verifies half-transparent
BGRA colour, opaque/transparent pixels and HDR coverage bytes. The smoke binary
loads either an explicit URL or `fixtures/osr-smoke.html` and writes the actual
OnPaint image as a PNG. Its fixture is a pixel test, not the LiquidBounce UI.
Original-theme behavior in a running game requires the real game-backed REST/
WebSocket actor; the fixture test does not establish that integration.

`runtime::page_state_current_thread()` returns bounded diagnostics from real
Chromium load/title/console callbacks. The smoke runner waits for a completed
main document, idle load state and actual paint, then settles for two seconds.
It fails on main document network/HTTP errors, asserts the fixture's three RGBA
samples, and attempts `OnBeforeClose`/shutdown even after a capture error.
`-SettleSeconds` and `-Report` control the delay and JSON evidence path. The
report includes document status, JavaScript console messages, paint count,
pixel samples and the observed shutdown result. A successful HTTP load and
paint alone do not prove that game actions or module behavior work.

The 2026-11-26 native rerun passed the fixture's exact pixel assertions and
completed shutdown. `original-theme-preview-verification.json` also records
the unmodified production theme loading with HTTP 200, connecting its real
WebSocket and routing to ClickGUI without JavaScript error messages. That run
used `liquidbounce-interop/examples/theme-preview.rs`, an explicitly
presentation-only actor with an empty module registry. Its PNG shows the
original navigation and search field; it establishes browser/theme transport
initialization, not game integration or implemented modules.
`unreachable-smoke-verification.json` records an intentionally unsafe-port
navigation failing with exit code 1 while still completing browser close and
CEF shutdown.

```powershell
cargo test --manifest-path native/liquidbounce-webview/Cargo.toml --features gpu-tests --lib
native/liquidbounce-webview/build-native.ps1 -Smoke -Url '<Server.theme_url()>' -Output '<absolute PNG path>' -Report '<absolute JSON path>'
```

This integration was written against cef-rs's published APIs and OSR example
(`4e566e628a361f3ade6f9a211cb33ea3c7a9153f`, MIT/Apache-2.0). Original add-on source
used for comparison retains its LGPL-2.1-or-later headers in the local audit
cache. The LiquidBounce theme remains its unmodified GPL-3.0-or-later artifact.

The `--lifecycle` smoke mode uses offline red/blue HTML documents to check distinct
browser IDs, hidden preloading, cached page reuse, immediate CPU surface restoration,
background reload isolation, close/reopen IDs and recreation at an actual 37 FPS
queried from both CEF hosts. Its final shutdown starts with two live browsers and
an asynchronously closing retired browser. The JSON report retains every completed
check even if a later assertion fails. This fixture exercises native CEF lifecycle
and pixels; game integration still needs the game-backed actor. Run it after the
normal native build environment is prepared:

```powershell
native/liquidbounce-webview/build-native.ps1 -Smoke -Url '--lifecycle' -Output '<absolute PNG path>' -Report '<absolute JSON path>'
```

To run the fixture from an executable already produced by the coordinated build,
without invoking Cargo or staging again:

```powershell
native/liquidbounce-webview/run-lifecycle-smoke.ps1 -Executable '<absolute liquidbounce-cef-smoke.exe path>' -OutputDirectory '<absolute evidence directory>'
```
