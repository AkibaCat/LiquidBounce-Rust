# Chams port and verification

The source baseline is LiquidBounce commit
`ae0eb396c07f22383789b28d5912d714c6ca0a86`, specifically
`ModuleChams.kt` (SHA-256
`f311e85a01fb1fa677991e5f8079806c225ffbc616575e10c83c45fc0b9baca1`)
and the original Chams fragment shaders. This is a native Horizon adaptation,
with the original settings and target selection represented in Rust.

## Settings

The exposed tree contains `Mode: Normal | Image`. Image contains `File`,
`Mapping: Repeat | Stretch | Cover`, `Filtering: Nearest | Linear`, and a
two-component `Offset`. Repeat contains `TileWidth`, default 256, with an editor
range of 16 through 2048 px. File import retains values outside that editor
range, as the upstream serializer does. Missing offset axes deserialize to zero;
unrecognized object fields are ignored. Stable English names remain the storage
and REST identifiers when the display is Chinese.

The File value advertises `OPEN_FILE` and `png`, `jpg`, `jpeg` extensions to the
theme. The actual native file dialog, root-relative path resolution, and runtime
settings exchange are integrated by the main task. Image decoding uses the
existing Horizon decoder. A failed decode retains the most recent successful
image; before any successful decode the renderer uses Normal. The same selected
path is not polled continuously. Selecting another path and then selecting the
first path again retries its decode. Disabling releases GPU resources but retains
the decoded CPU image cache.

## Capture and composite

`chams-patch.py` applies the actual capture hooks to Horizon sources. Chams uses
the global Visual Targets check and the raw F5 camera perspective. CPU geometry
is used while Chams is active, and the ordinary GPU skinning path is restored
when it is disabled. Captured quads are removed from the ordinary entity pass;
the recorded entity spans and motion offsets are rebased after removal.

The current capture hooks cover the local and remote player path, mob path, and
armor-stand path. Third-person held items belong to their entity scope.
First-person hands have no entity scope. The Ender Dragon uses its separate
renderer and is outside this LivingEntityRenderer scope. Capture bypasses the
target's entity occlusion decision but retains the existing distance and frustum
selection. Unrelated block entities and objects are not assigned a target scope.

The capture pass uses the ordinary entity shader and atlas with an isolated
`Rgba16Float` color target and `Depth32Float` reverse-Z depth. Solid materials are
submitted before translucent models and translucent items. Translucent items use
the renderer's existing no-depth-write behavior. Within each material group the
Horizon submission order is retained. No claim is made that this ordering is an
exact reproduction of vanilla's model-pivot sorting for all intersecting
transparent geometry.

Normal composites the captured color. Image discards zero-alpha entity pixels
and keeps the entity color when `entityDepth > sceneDepth + 0.00001`; otherwise
it samples the image. Mapping preserves aspect ratio for Repeat and Cover,
centers Cover, adds Offset after the mapping offset, and follows the original
bottom-left image coordinate convention. Both nearest and linear sampling are
implemented. The floating-point path retains HDR highlights and the existing
scene alpha; image color is converted for the linear frame path. Composite runs
after world drawing and before world post-processing and tone mapping. A pending
capture prevents first-person hand rendering from clearing the scene depth.
Each prepare can composite once, while a reused CPU frame may be rendered again
for another output size.

## Improved transparency compatibility

The pinned upstream implementation does not call a separate OIT drawing path
inside Chams. `resetStorage` selects improved transparency, but `renderChams`
calls only `FeatureRenderDispatcher.renderAllFeatures`. With improved
transparency enabled, vanilla's `SubmitNodeCollection` aliases ten translucent
fields to the same phase. `renderAllFeatures` visits these fields individually
and passes a null OIT stage, producing ten classic translucent replays.

This behavior was executed against the official Minecraft 26.3 client JAR, SHA-1
`e877b6a07acd633fb3bb475002175cec036e7b87`. The standalone
`ChamsPhaseOracle.java` runs the real dispatcher and prepared frame with counting
FeatureRenderer and RenderPass proxies. Its recorded result is:

```json
{"passed":true,"classicSolid":1,"classicTranslucent":1,"improvedSolid":1,"improvedTranslucent":10,"oitStage":null}
```

The Rust capture therefore repeats translucent model and item phases ten times
when the original improved-transparency option is enabled, preserving this
upstream behavior. The call deliberately receives
`self.improved_transparency`, rather than Horizon's `oit_on` capability gate,
which also depends on shader packs and device features. Vanilla additionally
disables the option for `renderWireframeTerrain`; no corresponding Horizon
wireframe state was found in the current renderer. Shader-pack and warp
combinations have not been verified as visually equivalent.

Reproduce the official dispatch inspection with:

```powershell
python -X utf8 native/liquidbounce-horizon/inspect-chams-oit.py
```

The report, javap output, compiler log and executed oracle result are under
`build/chams-oit-inspection/`. The oracle allocates its dispatcher without a GPU;
it proves phase dispatch, not an in-game image.

## Verification and remaining scope

```powershell
python -X utf8 native/liquidbounce-horizon/test-chams-render.py
python -X utf8 native/liquidbounce-horizon/test-chams-config.py --out build/chams-config-03
```

The renderer check currently passes 11 tests and Clippy with `-D warnings`.
Four tests execute the production capture/composite code and full entity shader
on a real NVIDIA GeForce GTX 1060 3GB using Vulkan. They cover capture and scene
depth, PNG/JPEG decoding, failed-image retention, image mapping and orientation,
offsets and filtering, HDR and alpha, color encoding, classic transparency,
ten-phase replay, and translucent-item depth behavior. Other tests exercise
capture removal and offset rebasing, plus the real generated Host/events/dispatch
API and copied production hook bodies with a Game shell. Their counting listener
does not prove runtime target classification or actual game entity submissions.
The config snapshot in `build/chams-config-02/` passed 35 tests and Clippy; the
main task also runs the newer combined engine configuration tests.

The production capture maps Horizon's material IDs to the original supported
RenderType set. Eyes, named full-bright feature layers, and breeze wind have an
explicit unsupported marker. The additional body/armor emissive submissions
still need a precise RenderType audit; their full equivalence is not asserted.
Horizon merges some glint handling in its entity shader instead of issuing the
original Java FeatureRenderer's separate draws. Exact pixels for these merged
materials and intersecting transparent entities remain a verification boundary.
The main task owns the full executable build and actual in-game screenshots;
these isolated GPU tests are not reported as a completed in-game test.

Stable production source hashes at this handoff:

| File | SHA-256 |
| --- | --- |
| `chams-render.rs` | `f716704603d3b84448da54057ce15defbe7f9a56d1cbe1fd31f2270441dd5032` |
| `chams-patch.py` | `0ac121043a6628bae2580f672a7fa4f7a1eafcc996dfb9cd7021e93db7dfa7f7` |
| `chams.wgsl` | `1bc73c0b371b1ef434d992ead67e69c9e6d40a1ae649cf60b0fd4e839e403cac` |
