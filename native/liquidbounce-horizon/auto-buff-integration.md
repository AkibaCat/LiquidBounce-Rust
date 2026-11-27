# AutoBuff integration handoff

Owned implementation is ready. The parent performs shared integration and official staging/build serially.

Run `python native/liquidbounce-horizon/integrate-auto-buff.py --check`, then invoke without `--check` to publish its ten shared files transactionally. Run `update-auto-buff-locales.py` for the production display locales. Normal `make-behavior-patch.py` picks up `auto-buff-patch.py` after integration.

The complete Soup/Head/Pot/Drink/Gapple policy, AutoSwap waits, shared cooldown, hold-use key, Soup None/Drop/Stack, Refill, two complete PlayerInventoryConstraints trees, all seven potion effects, and the complete noncombat Rotations group are included. Potion aim uses player movement replay for ground detection and up to four 300-tick trajectory passes with first-tick velocity correction, actual contextual COLLIDER block rays, pickable entity boxes, fluid drag, and Freeze-aware inherited velocity. Native use/drop/full position-rotation packets preserve the source's side effects. AutoWeapon also observes AutoBuff's actual silent-slot ownership at all three call sites.

Rotation provider is 17; smoothing processor base is 220; inventory owners are separate for Refill and bowl Stack. Disable cancels both inventory plans, invalidates coroutine waiters, and releases the silent slot. Head's chronometer survives disable. Runtime Host calls occur outside actor borrows.

Validation completed once:

- `rustfmt --edition 2024 --config skip_children=true` on the owned Rust files.
- `integrate-auto-buff.py --check`: ten shared files validated; no shared files changed.
- One focused `rustc` smoke: hotbar swap delay, bowl drop and cooldown, held-use termination, normal potion aim wait, enemy/effect exclusion, refill, and bowl-stack guards passed.
- `validate-auto-buff-patch.py`: native patch generation to `build/auto-buff-smoke-01/hooks-preview.patch`; both locale scratch files have 114 descriptions, valid UTF-8, and idempotent updates.

No Cargo/build/Clippy/oracle or live gameplay test was run by this agent. No production staging or shared source has been edited by this agent. Official compilation and in-game validation remain with the parent.
