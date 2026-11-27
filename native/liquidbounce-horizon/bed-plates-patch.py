"""Native BedPlates tracker, raw position/key events and common list submission."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
    edit('crates/horizon-client/src/game/bed_plates.rs',lambda _:(HERE/'bed-plates-game.rs').read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game/bed_plates_scan.rs',lambda _:(HERE.parent/'liquidbounce-behaviors/src/bed_plates_scan.rs').read_text(encoding='utf8'))
    def game(s):
        s=once(s,'mod item_tags;','mod item_tags;\nmod bed_plates;')
        s=once(s,'    client_item_tags:std::cell::RefCell<item_tags::State>,','    client_item_tags:std::cell::RefCell<item_tags::State>,\n    client_bed_plates:std::cell::RefCell<bed_plates::State>,')
        s=once(s,'            client_item_tags:Default::default(),','            client_item_tags:Default::default(),\n            client_bed_plates:Default::default(),')
        return once(s,'            self.client_item_tags_overlay(&mut batch);','            self.client_item_tags_overlay(&mut batch);\n            self.client_bed_plates_overlay(&mut batch);')
    edit('crates/horizon-client/src/game.rs',game)
    def hooks(s):
        a='    pub(super) fn native_client_item_tags('
        s=once(s,a,'    pub(super) fn native_client_bed_plates(&self,input:&str,output:&mut String){if self.native_window(){mod_api::dispatch_client_bed_plates(self,mod_api::mods(),input,output);}}\n\n'+a)
        s=once(s,'        self.client_item_tags_tick();','        self.client_item_tags_tick();\n        self.client_bed_plates_tick();')
        s=once(s,'        *self.client_item_tags.borrow_mut()=Default::default();','        self.client_bed_plates_world_change();\n        *self.client_item_tags.borrow_mut()=Default::default();')
        s=once(s,'        mod_api::dispatch_client_packet(self, mod_api::mods(), packet);','        if let horizon_protocol::packets::play::Clientbound::PlayerPosition(p)=packet{self.client_bed_plates_position(p.position);}\n        mod_api::dispatch_client_packet(self, mod_api::mods(), packet);')
        s=once(s,'        mod_api::dispatch_input_key(self, mod_api::mods(), event, modifiers);','        if let winit::keyboard::PhysicalKey::Code(key)=event.physical_key{self.client_bed_plates_key(key,event.state==winit::event::ElementState::Pressed);}\n        mod_api::dispatch_input_key(self, mod_api::mods(), event, modifiers);')
        return once(s,'        mod_api::dispatch_client_control_key(self, mod_api::mods(), key, pressed, repeat, modifiers);','        self.client_bed_plates_key(key,pressed);\n        mod_api::dispatch_client_control_key(self, mod_api::mods(), key, pressed, repeat, modifiers);')
    edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
    return {'hook_names':['client_bed_plates'],'helper_names':[]}
