"""Reusable original block placement target/context/verification kernel."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
    for target,source in [
        ('crates/horizon-client/src/game/placement_core.rs','block-placement-kernel.rs'),
        ('crates/horizon-client/src/game/block_placement.rs','block-placement-native.rs')]:
        edit(target,lambda _,source=source:(HERE/source).read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod block_items;','mod block_items;\nmod placement_core;\npub(super) mod block_placement;'))
    edit('crates/horizon-world/src/level/place.rs',lambda s:s+'\n'+(HERE/'block-placement-context.rs').read_text(encoding='utf8'))
    return {'hook_names':[],'helper_names':[]}
