# StorageESP

Complete Render module StorageESP, alias ChestESP, tag=null, +1 module, pinned LiquidBounce ae0eb396 / Horizon 9f42064 / Horizon-Mixins 5a64af93. Owned files only; no upstream, stage, AutoBreak or ItemScroller writes.

All original settings are present in source order: Box / Glow (default Glow), Box.Outline=true; Glow.Style Vanilla plus all nine frozen post styles (default Smooth), including all inactive style parameters and original distinct fills [0.12,0,0.08,0.1,0.08,0.18,0,0.08,0.1]. Thirteen toggleable categories retain source colors/Enabled=true/Tracers=false. RequiresChestStealer=false, MergeAdjacent=false and all four DistanceFade values/ranges/callback constraints are included. Full File/Interop roundtrip is covered. A complete coherent DistanceFade subtree imports atomically so sequential bounds callbacks cannot corrupt a saved valid range; partial field updates still use source callbacks.

Native snapshot enumerates the actual complete block-entity map, validates actual state names, and uses Game.outline_boxes for real outline shapes/context/offsets. Animated block-entity models use the actual native BlockEntity Scene renderer with original partial tick, chest lids, world, atlas and animation state. Box shapes use fixed 50 fill/100 outline alpha; the source distance shader's smoothstep fade is applied per vertex. MergeAdjacent groups only same-category six-neighbour blocks and forms voxel unions with internal faces/edges removed. There is no bounds substitution for Glow.

Chest/hopper minecarts, chest boats/rafts and actual chested donkey/mule/llama/trader llama models use their native dimensions, interpolated positions and submitted vertex slices. Native GPU skinning is disabled while this independent capture is active so all model/equipment vertices exist on CPU. Horizon occlusion is bypassed only for selected outlines; ordinary game distance/frustum tests remain. Selected invisible storage silhouettes are admitted by both native invisibility guards and then excluded from the ordinary entity pass unless TrueSight requests it. Capture composes with frozen Chams and TrueSight without mutating their files.

Native model vertices retain UVs and use the original atlas-alpha-zero discard; GPU readback verifies opaque and transparent native model silhouettes. Models only use strict distance < FarEnd², matching source; static shape meshes use vertex distance fade. Block tracers target centers, entity tracers target interpolated feet then actual height, with source one-pixel lines/ARGB and camera-bob cancellation when any category tracer setting is enabled. Actual ChestStealer running state gates RequiresChestStealer; an owner getter reads the real interacted set to exclude opened blocks.

The independent renderer derives from frozen Breadcrumbs ShaderLayer without changing it or its shaders. Default Smooth and Vanilla Sobel were exercised in one focused GPU usability test; the previously frozen nine-style matrix was not repeated. Plain entity boxes/tracers use an independent frozen-renderer instance. Real native model slices, rather than proxy cuboids, feed the no-depth RGBA8 mask before original post-processing.

Native actual block-map snapshots are rebuilt every frame. This produces current shapes while avoiding a second mutable scanner/cache; it trades source scanner caching for additional CPU work in storage-dense worlds. Actual native compile/performance/game parity remain root aggregate verification tasks.

Root serial integration:

```powershell
python native/liquidbounce-horizon/integrate-storage-esp.py
python native/liquidbounce-horizon/integrate-storage-esp.py --apply
python native/liquidbounce-horizon/update-storage-esp-locales.py --directory native/liquidbounce-interop/resources/localization
```

Integrator: 18 edits / 8 shared files. Native patch: 12 files, hook client_storage_esp(&self,input:&str,geometry:&mut String), generated ClientStorageEspEvent. Each copied Chinese dictionary has 116 nested descriptions. Passing production config/behavior/runtime/renderer and focused GPU evidence: build/storage-esp-smoke-02/test.log (one test). The report's native patch item initially failed; only that item was rerun through smoke03-05. Final build/storage-esp-smoke-05/report.json passed=true, compileExit=0, patchError=null and references retained smoke02 compile evidence. Smoke01-05 are historical and preserved. No official build or actual game verification was run here.
