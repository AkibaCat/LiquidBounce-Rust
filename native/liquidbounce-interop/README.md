# Original theme transport

Real Rust localhost HTTP/WebSocket service for LiquidBounce's unchanged built
theme. Source behavior comes from the pinned upstream ClientInteropServer,
AuthPlugin and WebSocketSessionManager, under GPL-3.0-or-later.

Implemented: secure 16-character auth codes, `lb_code` bootstrap with HttpOnly
`lb_auth` cookie, original `/resource/liquidbounce` theme URL, root server-info
response and cookie-authenticated WebSocket, 15-second protocol ping, original
`{"name": ..., "event": ...}` event envelopes, and `/api/v1/*` request forwarding.
Config request bytes, HTTP method and decoded query arguments reach the real
game backend without invented module data.

`main_thread::QueuedBackend::channel(capacity, timeout)` sends actual HTTP
requests to an `Inbox` owned by the game thread. Drain `try_next()`, execute the
request, then `Pending::reply(response)`. Expired queued mutations are discarded;
queue overflow, an unresponsive game and shutdown return 503.

`protocol::module_request(&mut impl ModuleAccess, &ApiRequest)` handles the
original module list/detail/settings/category/toggle/panic routes. The actual
adapter supplies module JSON and setting trees, executes lifecycle callbacks,
persists state and broadcasts events from those real mutations. POST toggles
the current state and ignores the frontend's `enabled` field; PUT enables and
DELETE disables, with 403 on redundant requests. REST lookup is case-insensitive
on the canonical module name, just like upstream; descriptive aliases do not
resolve modules. Only modules actually registered by the adapter are listed.
Category icons are null until the real resource route is connected. Backend
mutation failures return 500 instead of silently reporting success.

`ui::UiState::load(theme_root, isolated_state_root)` reads original metadata
colors and stores actual presentation state in `web-ui.json`. Call `request`
before `module_request`; it supplies theme, localStorage/all, virtual-screen
acknowledgement, text-entry/Escape capture and the implemented ClickGUI/HUD
presentation settings. Call `set_screen` after the actual native screen changes.
Forward `take_events()` through `Server::broadcast` after settings mutations.
This exposes Scale/Snapping/SearchBarAutoFocus and SpaceSeperatedNames without
pretending that all ClickGUI, HUD components or global features are implemented.
Window/player/session/input data still comes from the actual Horizon backend.

The original server also supports external-theme CORS, compression, local and
marketplace directories, configurable bind retries and request diagnostics.
Those are pending here. This version requires an alphanumeric authentication
code; upstream environment overrides have broader input. A WebSocket that
overflows 1024 pending events closes instead of silently losing state; upstream
launches sends independently without that fixed application queue. These are
explicit transport boundaries, not a claim of full parity.

Seven tests pass: auth/cookie behavior, exact config forwarding, a real TCP
WebSocket connection, game-thread replies/expiry, real TCP toggle/config routes,
and metadata/UI persistence/stale-screen acknowledgement. These protocol tests
use a deliberately small test registry, not a claim of gameplay validation.
Formatting and strict Clippy pass.

```powershell
cargo +stable test --manifest-path native/liquidbounce-interop/Cargo.toml
cargo +stable clippy --manifest-path native/liquidbounce-interop/Cargo.toml --all-targets -- -D warnings
```

For a presentation-only CEF regression run, without any registered game modules:

```powershell
cargo +stable run --manifest-path native/liquidbounce-interop/Cargo.toml --example theme-preview -- upstream/LiquidBounce/src-theme/dist build/theme-preview-state 1280 720 300
```

The first output JSON contains `theme_url` for the real built theme. Width and
height describe the configured preview viewport; `/info` reports `inGame:false`.
The process remains available for 300 seconds, stores its own UI layout and
exposes an empty gameplay registry. This checks original HTML/JS loading in CEF;
it cannot establish that a game module, player, native HUD or game input works.
