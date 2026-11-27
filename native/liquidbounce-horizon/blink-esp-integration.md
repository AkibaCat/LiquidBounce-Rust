# BlinkManager global ESP

Pinned LiquidBounce ae0eb396 / Horizon 9f42064. This is a shared global manager group, not another registered gameplay module. AutoBreak and ItemScroller are untouched. Pristine upstream and staging are read only.

The complete global `BlinkManager` group contains `Esp` (Box/Model/Wireframe/Ring/None; default Wireframe), `Line` and `Render`. Each inactive ESP/Shader choice persists through File and Interop. Render defaults to Plain and all nine Shader styles start with Fill=1 (`rendering(solid=true)`). Actor GET, imports, File export, unknown-group preservation exclusions and per-value `valueChanged` events use the actual state.

Positions are `CriticalsService::positions()` from the shared BlinkManager queue, in outgoing movement-packet order. The first queued position anchors the ESP; the full sequence draws the line. The rotation is `behaviors.rotations.lock().actual_server`, never camera or theoretical rotation. No ESP entity enters the world. Root-owned ModuleBlink retains its separate optional dummy.

Host helper: `client_blink_visual(partial_ticks: f32) -> String`, returning a JSON snapshot of the actual Game player: effective first-person flag, pose network id, live width/height and interpolated real model SwimAnimation. The `client_blink_esp(input: &str, geometry: &mut String)` hook receives the frame partial tick as its input and emits native model settings plus separate ESP and line frames.

Box uses live pose dimensions inflated 0.05. Wireframe follows the six original cuboids, 1.9 root scale, actual server yaw/head pitch, crouch pivots and swimming rotations. Ring uses the pinned radial `gradient_circle.fsh` formula, transparent centre, and the separate `circle/rounded_rect.fsh` circle outline with CornerRadius=(1,1), StrokeWidth=2 physical pixels and derivative AA. It does not polygonize the circle. Radius<=0 hides both ring parts. First person hides Box/Model/Wireframe, while Ring/Line remain visible.

Model source generation copies the pinned native `player_model_geometry` extraction and real skin/equipment/layer calls. It clones the current Mob render state, uses partial=0, substitutes queue position and both old/current server rotations, scales sky/block coordinates independently with rounding/clamping, and adds optional true outline geometry. The snapshot never changes the live player's state. Its geometry uses the ordinary native entity/material renderer. Box/Wireframe/Ring have independent plain world rendering, and only Line consumes global Render's ShaderLayer. The two independent render instances derive from the frozen Breadcrumbs renderer and existing shader resources without editing that module.

Root integration (serial, before the next official native/engine check):

```powershell
python native/liquidbounce-horizon/integrate-blink-esp.py
python native/liquidbounce-horizon/integrate-blink-esp.py --apply
python native/liquidbounce-horizon/update-blink-esp-locales.py --directory native/liquidbounce-interop/resources/localization
```

The transactional integrator validates eleven anchors across seven shared files before writing. It keeps Crosshair wiring, adds no ModuleKind/registry row and never touches Blink's packet/dummy files. The independent native patch validates against a copy of the current stage.

Validation: `build/blink-esp-smoke-02/report.json`, five passing focused tests: modes/poses/first-person/line; all global settings and inactive choices; production runtime with actual-server-vs-theoretical distinction; one new radial WGSL validation plus native clone/skin/equipment/partial-zero source checks; all 86 description keys in each actual dictionary copy. Smoke01 is retained as historical evidence, superseded by Smoke02's source-correct two-pixel circle outline. No repeated shader/GPU matrix was run.

These focused fixtures do not establish actual in-game parity. Official native/engine aggregate compilation and actual game validation remain integration-owner work. New shader GPU execution is covered by that final full run, while this check validates its Naga module only.
