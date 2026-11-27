# GhostHand integration handoff

Owned implementation and verification are complete. Root owns shared registration, production build and actual game checks. Full original/native client differential remains `not_run`; this module does not complete the 236-module objective.

## Original and owned files

Pinned originals: `ModuleGhostHand.kt`, `common/TweakedMethods.java`, `MixinBlockGetter.java` at ae0eb396.

Register **GhostHand**, category **Exploit**, no aliases. The only custom setting is **TargetedBlocks**, a complete registry-list set. Default contains chest, ender chest, trapped chest, undyed shulker box, and all sixteen dyed shulker boxes (20 total).

- Export behavior `native/liquidbounce-behaviors/src/ghost_hand.rs`; add `ghost_hand: GhostHand` and default initialization.
- Add `ModuleKind::GhostHand`, key `liquidbounce.module.ghostHand`.
- Add config child `engine/src/config/ghost_hand.rs`; export `GhostHandSetting` and `serialize_ghost_hand`, route `parse_ghost_hand`, add SettingChange/RuntimeRequest variants and actor apply with the full live block registry. `GhostHandSetting::apply(module, registry)` is self-contained.
- Add `ghost-hand-patch.py` to production transforms **after SmartEat** (it transforms SmartEat's actual Item.use ray as well). Add its one hook `client_ghost_hand`, no helper catalog names.
- There is no tick, coroutine, reset, world-change, disable, input or packet state to add.

Chinese module description and module key already exist in localization resources; Chinese agent supplied the missing TargetedBlocks help supplement while implementing BlockWalk.

## Runtime hook

The actual generated `ClientGhostHandEvent<'a>` uses a plain `bool` for `running` and `&'a mut Vec<String>` for `targeted_blocks`. Assign through the vector reference:

```rust
fn client_ghost_hand(&self, _host: &dyn Host, event: &mut ClientGhostHandEvent) {
    let state = self.state();
    let mut modules = state.behaviors.borrow_mut();
    event.running = modules.query_running(&state.registry, "GhostHand");
    if event.running {
        *event.targeted_blocks = modules.ghost_hand.sorted_targeted_blocks()
            .into_iter().map(str::to_owned).collect();
    }
}
```

Original `MixinBlockGetter` overwrites **every** `BlockGetter.clip`, so do not add a local-player, Playing-screen, native-window, remote-only or execution-role guard to this hook. The original module's running state remains the only module guard. The predicate queries state without causing any recursive clip.

## Clip contract and coverage

`ghost-hand-ray.rs` performs the exact two-pass algorithm:

1. If running (even with an empty target set), traverse the complete ray in original block order. Read each block state. Only selected blocks read the requested block shape and run `clipWithInteractionOverride`. This pass never queries fluids. The first selected **shape hit** wins through any earlier wall/fluid.
2. If that pass returns no hit, restart the ordinary block/fluid clip with its original modes, world provider, miss location, face and inside state.

Fluid selection cannot intercept a selected block from the first pass. A selected water/air/empty outline does not itself create a hit. Selection changes and mode-specific shapes take effect on each call. Provider/traversal errors propagate and do not become fabricated misses/fallbacks.

`ghost-hand-game.rs` binds the current registry names to actual `BlockStateId`s. The patch covers all actual runtime clip families found in the pinned client:

- Game outline `clip_ray_fluids`, using actual `clip_declared_outline` for the selected pass; all existing pick/debug/attack/native/LAN callers inherit it.
- SmartEat's actual Item.use outline ray.
- BoatItem's separate outline ray.
- Camera `VisualRays` clip.
- Explosion `ColliderRays` clip.

The latter four use the same generic exact wrapper and preserve their providers and clip contexts. `registry_fall_ray` is a standalone exported world API with no pinned game runtime caller; the generic algorithm itself has actual oracle coverage for FallDamageResetting.

## Verification

```powershell
python native/liquidbounce-horizon/collect-ghost-hand-facts.py
python native/liquidbounce-horizon/test-ghost-hand.py
```

**9/9 tests passed; Clippy `--all-targets -- -D warnings` passed.**

The collector compiles the untouched original `TweakedMethods.java` together with a minimal ModuleGhostHand settings shell and runs against the actual pinned MC 26.3 JAR. Its **4,608 cases** use actual block/fluid shapes: 18 states × four block modes × four fluid modes × four policies × four ray layouts. Rust matches original miss, position, face, inside and location (1e-12 bound) for all cases. Cases include both ray directions, corner traversal, equal endpoints, defaults/selected-state/empty/disabled sets, walls before targets, all Fluid modes, Collider/Outline/Visual/FallDamageResetting.

Additional tests cover complete 20-block defaults; registry identifier sorting/coercion, empty replacement, transactional errors; actual shared Tree/common serialization and parse helpers, both File/Interop formats and registry metadata; no fluid/nonselected-shape reads in the selected pass; fallback restart; provider/budget errors; actual native hook and actual generated Host/dispatch; actual Game target adapter and native legacy-outline entry.

The native fixture substitutes Game storage, registry-name storage and outline shape storage (actual MC boxes plus actual Horizon clipWithInteractionOverride). The generic algorithm and Game adapter are production source. It does not claim full module-registry/actor execution or full native typecheck.

`test-ghost-hand.py` also composes after the entire current production generator without writing shared patches/staging. **228 transformed Rust sources passed rustfmt syntax validation**. Evidence:

- `build/ghost-hand-tests/result.json`, `test.log`, `clippy.log`.
- `oracle.json`, `java-compile.log`, `java-oracle.log`; oracle records original Java source SHA256 and actual MC JAR SHA1.
- `composition.json`, `composition-syntax.log`, isolated `composition/` (syntax only).
- `api-source/`, actual generated build outputs and `native-game-ray.rs`.

No upstream or staging files were modified.
