# InventoryCleaner shared foundation

Owned files implement the pinned ItemCategorization, armor comparator/evaluation,
ItemPacker, CleanupPlanGenerator and merge simulation, followed by the actual
InventoryCleaner inventory plan. All settings and the InventoryManager alias are
present. Greedy is stored and published but unused, exactly as the pinned source's
TODO. Maximum settings are lower desired bounds with an INT_MAX upper bound,
not hard caps: accepting a whole stack can exceed the desired count.

Classification includes generic and specialized weapons, knockback compared with
the best real main weapon, bows/crossbows/arrows, tools and sharp/god axes,
armor with both damage models and two kit passes, future diamond-kit armor,
food functions, bucket fluid groups, useful/junk/mixed potions, throwable items,
named server items and scaffold-eligible blocks. Comparators preserve source
chain ordering, stable sorting, float-weight order, durability and identity ties.
PREFER_FEWER is the source's natural count comparator; descending selection
therefore visits large stacks first despite that comparator's name.

Native facts read real stack identity, effective components, count-normalized
canonical patch, item class/tag tables, attribute modifier order, item component
food/tool/potion/equipment data, block collision support, actual block movement
coefficients, full collision and conductor flags. The fixed protocol is 26.3 and
therefore offhand swaps are supported. Missing max_stack_size defaults to 1.

One continuation uses the existing shared InventoryManager, resolving each swap's
current source through slot_of. Any actual swaps finish the plan after all swaps;
the next plan reassesses displaced items. With no swaps, the captured arrow
quick-moves, atomic merge chains and low-priority throws run in source order,
with live empty checks and the manager's existing menu/lock guards.

The shared ItemPolicy is the required future integration point for AutoArmor's
active comparator mode, SaveArmor threshold, projectile option and curse filter;
Scaffold's Disallowed/Unfavorable block selection lists; and Offhand.isOperating.
Until those modules are implemented it contains their actual default policies.
It does not register an AutoArmor, Scaffold or Offhand placeholder module.

Apply integrate-inventory-cleaner.py through the root's serial integration, and
update-inventory-cleaner-locales.py through the canonical locale publisher.
Simplified/traditional names, all settings and slot choices use the existing
game-language display mechanism, preserving stable English configuration tags.

One short Rust smoke passed after fixing a local closure lifetime. It covers
classification/retention, custom names, quantity lower bounds, allocation order,
armor and curse policies, server locks and merge planning. Read-only integration
anchors pass for seven shared files. Official native and engine compilation is
owned by the root agent; no full-game result is claimed by this short smoke.
