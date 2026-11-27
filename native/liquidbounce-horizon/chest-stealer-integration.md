# ChestStealer integration

Owned implementation is frozen for the root's next serial staging/build pass.

Apply shared wiring, then canonical localization:

```powershell
python native/liquidbounce-horizon/integrate-chest-stealer.py --check
python native/liquidbounce-horizon/integrate-chest-stealer.py
python native/liquidbounce-horizon/update-chest-stealer-locales.py
```

The integrator validates every original before any write and uses recoverable
`write_text` publication to avoid the previously observed Windows replacement
sharing violation. It edits nine shared files and leaves pinned upstream intact.

Implemented source behavior:

- Value, Distance, Index and Random selection, all mode children and exact
  distance factor draw order; default single-item categorization.
- InventoryCleaner cleanup plan reuse or all nonempty container items when the
  cleaner is disabled; category swap priority and pre-swap pickup prefix.
- Atomic quick swaps, preserving useful displaced hotbar stacks, legitimate
  offhand swap support, live slot remapping and post-swap replanning.
- Live capacity, whole drag/pickup chains, source merge-space accounting using
  item default max stack size, one throw/retry per candidate, OnFull and AutoClose.
- Shared InventoryManager ownership, screen identity, continuations,
  cancellation, requirements and world reset.
- Chest Aura registry defaults, range/wall geometry, blocked chest/cat checks,
  shared full rotation processors, normal UseItemOn result/prediction, local
  interaction animation, accepted manual-packet tracking, wait/retry/delay.
- SilentScreen real screen-event handling, container render and user slot-input
  suppression, retained real menus, pointer capture/raw mouse look, projected
  full contents/title at double-chest center, slot texture or rounded panel with
  fill/outline/gradient/glow, scale and offset.
- Full File/Interop settings, old inventory constraint imports, enum/registry
  semantics and inherited screen type/title/rotation trees.
- Simplified/traditional captions and 74 module description entries per locale,
  including inherited filters/constraints/rotations and English fallback.

Validation completed:

- `integrate-chest-stealer.py --check`: nine shared files validated.
- The single focused behavior smoke passed at
  `build/chest-stealer-smoke-01/chest-stealer-smoke.exe` (four selection modes,
  priority, swap chains, drag chain, capacity/merge budget, OnFull).
- Native patch generation passed into owned scratch
  `build/chest-stealer-smoke-01/hooks-preview.patch`; no production patch or
  staging was modified by this check.
- Locale updater is idempotent and has no replacement characters in scratch.
- Owned Rust files were formatted. The root owns formal native/engine compilation
  and actual game verification; these checks are not an in-game pass.

Shared cleaner changes included in this delivery: `Item.default_maximum`, actual
native `defaultMaximum` component facts and runtime parsing;
`Categorization::default_value`; all collision-context type paths fully qualified
in `inventory-cleaner-host.rs` for native_hooks's scope.
