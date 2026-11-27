#!/usr/bin/env python3
"""BetterInventory's real Slot HEAD, itemCooldown TAIL and extractSlots AFTER."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
    # horizon-client forbids unsafe code. Expose the Win32 locale query through
    # the existing platform crate rather than weakening its safety boundary.
    def platform(text):
        if text:raise ValueError('format_locale.rs already exists')
        return (HERE/'better-inventory-locale-platform.rs').read_text(encoding='utf-8')
    edit('crates/horizon-system/src/format_locale.rs',platform)
    edit('crates/horizon-system/src/lib.rs',lambda text:once(text,'pub mod scrub;\n','pub mod scrub;\nmod format_locale;\npub use format_locale::initial_format_locale;\n'))
    helper=(HERE/'better-inventory-host.rs').read_text(encoding='utf-8')
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda text:once(text,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',helper+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
    def game(text):
        text=once(text,'    inventory: Vec<Option<ItemStack>>,\n','    inventory: Vec<Option<ItemStack>>,\n    client_better_inventory_slots: std::cell::RefCell<Vec<Option<usize>>>,\n    client_better_inventory_origin: std::cell::Cell<(f32,f32)>,\n')
        text=once(text,'            selected: 0,\n','            selected: 0,\n            client_better_inventory_slots: std::cell::RefCell::new(Vec::new()),\n            client_better_inventory_origin: std::cell::Cell::new((0.0,0.0)),\n')
        text=once(text,'        self.draw_hovered_slot_highlight(batch, false, ox, oy);\n','        self.client_better_inventory_begin(ox,oy);\n        self.draw_hovered_slot_highlight(batch, false, ox, oy);\n')
        return once(text,'        self.draw_hovered_slot_highlight(batch, true, ox, oy);\n','        self.draw_hovered_slot_highlight(batch, true, ox, oy);\n        self.client_better_inventory_end(batch);\n')
    edit('crates/horizon-client/src/game.rs',game)
    def slots(text):
        text=once(text,'    ) {\n        let carried;','    ) {\n        self.client_better_inventory_slot(batch,slot,x,y);\n        let carried;')
        text=once(text,'        let decorations = hud_overlays::ItemDecorations {','        let progress = self.cooldowns.percent(&cooldown, partial);\n        let decorations = hud_overlays::ItemDecorations {')
        text=once(text,'            cooldown: self.cooldowns.percent(&cooldown, partial),','            cooldown: progress,')
        old='''        draws.extend(hud_overlays::item_decorations(
            0,
            0,
            &decorations,
            &self.gui.atlas.font,
        ));
        draws.push(hud_layout::GuiDraw::Pop);
        hud_layout::paint(batch, &self.gui.atlas, &draws);
'''
        new='''        // itemCooldown TAIL runs before itemCount in vanilla itemDecorations.
        let before = hud_overlays::ItemDecorations { count:1,count_text:None,bar:decorations.bar,cooldown:progress };
        draws.extend(hud_overlays::item_decorations(0,0,&before,&self.gui.atlas.font));
        draws.push(hud_layout::GuiDraw::Pop);
        hud_layout::paint(batch, &self.gui.atlas, &draws);
        self.client_better_inventory_cooldown(batch,stack,x,y,progress,&cooldown);
        let after = hud_overlays::ItemDecorations { bar:None,cooldown:0.0,..decorations };
        let mut counts=vec![hud_layout::GuiDraw::Push(vec![hud_layout::PoseOp::Translate(x,y)])];
        counts.extend(hud_overlays::item_decorations(0,0,&after,&self.gui.atlas.font));
        counts.push(hud_layout::GuiDraw::Pop);
        hud_layout::paint(batch,&self.gui.atlas,&counts);
'''
        return once(text,old,new)
    edit('crates/horizon-client/src/game/inventory_slots.rs',slots)
    def regular(text):
        text=once(text,'        self.draw_hovered_slot_highlight(batch, false, ox, oy);\n','        self.client_better_inventory_begin(ox,oy);\n        self.draw_hovered_slot_highlight(batch, false, ox, oy);\n')
        return once(text,'        self.draw_hovered_slot_highlight(batch, true, ox, oy);\n','        self.draw_hovered_slot_highlight(batch, true, ox, oy);\n        self.client_better_inventory_end(batch);\n')
    for name in ['stations','trading','beacon']:edit(f'crates/horizon-client/src/game/{name}.rs',regular)
    def beacon(text):
        return once(text,'''            if let Some(stack) = self.slot_mut(i).clone() {
                self.draw_item(batch, &stack, ox + sx, oy + sy);
            }
''','''            let stack=self.slot_mut(i).clone();
            self.draw_slot_item(batch,None,i,&stack,ox+sx,oy+sy);
''')
    edit('crates/horizon-client/src/game/beacon.rs',beacon)
    def creative(text):
        text=once(text,'        if tab.inventory {\n            self.draw_player_preview(','        self.client_better_inventory_begin(ox,oy);\n        if tab.inventory {\n            self.draw_player_preview(')
        text=once(text,'            let (tx, ty) = Creative::TRASH_POSITION;\n','            let (tx, ty) = Creative::TRASH_POSITION;\n            self.client_better_inventory_slot(batch,1023,ox+tx,oy+ty);\n')
        text=once(text,'                    let Some(stack) = items.get((first + r) * 9 + c).filter(|s| s.count > 0) else {\n','                    let (hx,hy)=(ox+9.0+c as f32*18.0,oy+18.0+r as f32*18.0);\n                    self.client_better_inventory_slot(batch,Creative::DISPLAY+r*9+c,hx,hy);\n                    let Some(stack) = items.get((first + r) * 9 + c).filter(|s| s.count > 0) else {\n')
        return once(text,'        self.draw_carried(batch);\n','        self.client_better_inventory_end(batch);\n        self.draw_carried(batch);\n')
    edit('crates/horizon-client/src/game/creative_screen.rs',creative)
    edit('crates/horizon-items/src/cooldowns.rs',lambda text:once(text,'    /// `isOnCooldown`: whether the group still has time to run.\n','    /// Read-only ItemCooldownsAccessor view, used by the actual item render hook.\n    pub fn render_entry(&self, group:&str)->Option<(i32,i32)> {\n        self.entries.get(group).map(|(_,end)|(*end,self.tick_count))\n    }\n    /// `isOnCooldown`: whether the group still has time to run.\n'))
    return {'hook_names':['client_better_inventory_render'],'helper_names':[]}
