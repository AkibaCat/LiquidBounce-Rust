#!/usr/bin/env python3
"""ItemScroller: AbstractContainerScreen mouseClicked HEAD and extractSlots AFTER."""
from pathlib import Path
HERE=Path(__file__).resolve().parent

def apply(edit,once):
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda text:once(text,
        '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',
        (HERE/'item-scroller-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
    # These are precisely BetterInventory's extractSlots AFTER locations. Its
    # original handler executes the preview before ItemScroller's move.
    for path in ['game.rs','game/stations.rs','game/trading.rs','game/beacon.rs','game/creative_screen.rs']:
        edit('crates/horizon-client/src/'+path,lambda text:once(text,
            '        self.client_better_inventory_end(batch);\n',
            '        self.client_better_inventory_end(batch);\n        self.client_item_scroller_render(batch);\n'))
    # mouseClicked HEAD precedes checkHotbarMouseClicked and applies to every
    # button, including a bound hotbar/pick button while left remains held.
    edit('crates/horizon-client/src/game.rs',lambda text:once(text,
        '''                    if matches!(self.screen, Screen::Inventory | Screen::Creative)
                        && let Some(key) = self
                            .settings
                            .keys
                            .container_mouse_key(button, self.creative)
''',
        '''                    if pressed && matches!(self.screen, Screen::Inventory | Screen::Creative)
                        && self.client_item_scroller_cancel_click() { return Ok(()); }
                    if matches!(self.screen, Screen::Inventory | Screen::Creative)
                        && let Some(key) = self
                            .settings
                            .keys
                            .container_mouse_key(button, self.creative)
'''))
    # Unlike skip_release(), the original assignment does not cancel a drag.
    edit('crates/horizon-client/src/container_screen.rs',lambda text:once(text,
        'impl Clicks {\n',
        'impl Clicks {\n    pub fn client_skip_next_release(&mut self) { self.skip_next_release = true; }\n'))
    edit('crates/horizon-client/src/game/remote.rs',lambda text:once(text,
        '    fn remote_synced(&mut self) {\n','    pub(super) fn remote_synced(&mut self) {\n'))
    return {'hook_names':['client_item_scroller_query','client_item_scroller_reset'],'helper_names':[]}
