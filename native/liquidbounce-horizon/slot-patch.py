#!/usr/bin/env python3
"""SilentHotbar/NoSlotSet source hooks; never changes the upstream checkout."""
from pathlib import Path
import json

HERE = Path(__file__).resolve().parent

HELPERS = '''    /// Raw Inventory.selected, retained independently from the effective slot.
    pub(super) fn native_client_slot_owner(&self) -> bool {
        self.native_window() && self.lan_acting_uuid.is_none()
            && self.player.as_ref().is_some_and(|p| p.native_client_queries_allowed())
    }
    pub(super) fn native_client_real_selected_slot(&self) -> Option<i32> {
        self.native_client_slot_owner().then_some(self.selected as i32)
    }
    pub(super) fn client_hotbar_slot(&self) -> usize {
        if self.native_client_slot_owner() {
            horizon_client::slot_bridge::serverside_slot(Some(self.selected as u8)) as usize
        } else {
            self.selected
        }
    }
    pub(super) fn native_client_serverside_slot(&self) -> Option<i32> {
        self.native_client_real_selected_slot().map(|_| self.client_hotbar_slot() as i32)
    }
    pub(super) fn native_client_clientside_slot(&self) -> Option<i32> {
        self.native_client_real_selected_slot().map(|slot| horizon_client::slot_bridge::clientside_slot(Some(slot as u8)) as i32)
    }
    pub(super) fn native_client_select_slot_silently(&self, requester: u64, slot: i32, ticks: i32) -> Result<bool, String> {
        let real = self.native_client_real_selected_slot().ok_or("local player is unavailable")?;
        horizon_client::slot_bridge::select_slot(requester, slot, ticks, Some(real as u8))
    }
    pub(super) fn native_client_reset_silent_slot(&self, requester: u64) -> bool {
        self.native_window() && horizon_client::slot_bridge::reset_slot(requester)
    }
    pub(super) fn native_client_silent_slot_modified(&self) -> bool {
        self.native_window() && horizon_client::slot_bridge::modified()
    }
    pub(super) fn native_client_silent_slot_modified_by(&self, requester: u64) -> bool {
        self.native_window() && horizon_client::slot_bridge::modified_by(requester)
    }
    /// Runs on the game actor after the reader's cancellable original event.
    /// NoSlotSet answers every SetHeldSlot, including invalid slot indices.
    pub(super) fn native_reject_slot_packet(&self, packet: &ClientboundPlayPacket) -> bool {
        if !self.native_client_slot_owner()
            || !horizon_client::slot_bridge::no_slot_set_running()
            || !matches!(packet, horizon_protocol::packets::play::Clientbound::SetHeldSlot(_)) {
            return false;
        }
        let slot = self.client_hotbar_slot() as i16;
        self.send(horizon_protocol::packets::play::Serverbound::SetCarriedItem(
            horizon_protocol::packets::play::SetCarriedItem { slot }
        ));
        true
    }

'''


def apply(edit, replace_once) -> dict[str, str]:
    def network(text):
        return replace_once(text, '''        let mut writer = self.writer.lock().unwrap();
        if self.configuring.load(Ordering::Relaxed) {
            return;
        }
        if let Some(w) = writer.as_mut()
            && w.send(&packet.encode()).is_err()
        {
            w.shutdown();
        }
''', '''        let sent = {
            let mut writer = self.writer.lock().unwrap();
            if self.configuring.load(Ordering::Relaxed) {
                return;
            }
            if let Some(w) = writer.as_mut() {
                if w.send(&packet.encode()).is_err() {
                    w.shutdown();
                    false
                } else {
                    true
                }
            } else {
                false
            }
        };
        // The observation receives the final, actually written packet and can
        // re-enter Session.send without retaining the writer lock.
        if sent {
            crate::slot_bridge::outgoing_sent(self.serial, &packet, silent);
        }
''')
    edit("crates/horizon-client/src/net.rs", network)
    edit("crates/horizon-client/src/client_lib.rs", lambda text: replace_once(text, "pub mod packet_bridge;\n", "pub mod packet_bridge;\npub mod slot_bridge;\n"))

    def native(text):
        text = replace_once(text, "    /// Player FOV scale, after vanilla effects and before zoom and easing.\n", HELPERS + "    /// Player FOV scale, after vanilla effects and before zoom and easing.\n")
        text = replace_once(text, "    pub(super) fn native_client_tick_pre(&mut self) {\n", "    pub(super) fn native_client_tick_pre(&mut self) {\n        if self.native_window() { horizon_client::slot_bridge::tick(); }\n")
        for signature in [
            "    pub(super) fn native_client_disconnecting(&self) {\n",
            "    pub(super) fn native_client_level_load(&mut self, dimension: &str) {\n",
            "    pub(super) fn native_client_level_unload(&mut self, dimension: &str) {\n",
        ]:
            text = replace_once(text, signature, signature + "        if self.native_window() { horizon_client::slot_bridge::world_change(); }\n")
        text = replace_once(text, "[self.stack_in(self.selected).cloned(), self.offhand.clone()]", "[self.stack_in(self.client_hotbar_slot()).cloned(), self.offhand.clone()]")
        return replace_once(text, "self.native_slot_tick(i as i32, i == self.selected);", "self.native_slot_tick(i as i32, i == self.client_hotbar_slot());")
    edit("crates/horizon-client/src/game/native_hooks.rs", native)

    def remote(text):
        text = replace_once(text, '''    pub(super) fn handle_packet(&mut self, packet: C) -> Result<()> {
        self.native_client_packet(&packet);
''', '''    pub(super) fn handle_packet(&mut self, packet: C) -> Result<()> {
        if self.native_reject_slot_packet(&packet) { return Ok(()); }
        self.native_client_packet(&packet);
''')
        text = replace_once(text, '''                    self.selected = p.slot as usize;
''', '''                    self.selected = p.slot as usize;
                    if self.native_window() { horizon_client::slot_bridge::server_slot_applied(p.slot as u8); }
''')
        text = replace_once(text, '''    fn remote_send_carried(&self) {
        if let Some(r) = &self.remote
            && r.carried.get() != Some(self.selected)
        {
            r.carried.set(Some(self.selected));
            r.session.send(S::SetCarriedItem(play::SetCarriedItem {
                slot: self.selected as i16,
''', '''    fn remote_send_carried(&self) {
        let selected = self.client_hotbar_slot();
        if let Some(r) = &self.remote
            && r.carried.get() != Some(selected)
        {
            r.carried.set(Some(selected));
            r.session.send(S::SetCarriedItem(play::SetCarriedItem {
                slot: selected as i16,
''')
        text = replace_once(text, "let selected = self.selected;", "let selected = self.client_hotbar_slot();")
        text = replace_once(text, '''        let dropped =
            horizon_client::items::take_held_drop(&mut self.inventory[self.selected], all, false);''', '''        let selected = self.client_hotbar_slot();
        let dropped =
            horizon_client::items::take_held_drop(&mut self.inventory[selected], all, false);''')
        text = text.replace("self.stack_in(self.selected)", "self.stack_in(self.client_hotbar_slot())")
        return text.replace("slot == self.selected", "slot == self.client_hotbar_slot()")
    edit("crates/horizon-client/src/game/remote.rs", remote)

    # These are local main-hand reads, corresponding to Inventory.getSelectedItem
    # and LivingEntity.getMainHandItem. Input, menus and raw slot setters retain
    # Inventory.selected; server/guest Games use the original field in the helper.
    readonly = [
        "block_breaking.rs", "combat.rs", "crossbow.rs", "dynamic_lights.rs",
        "fishing.rs", "fireworks.rs", "item_draw.rs", "mace.rs", "maps.rs",
        "pad/hardware.rs", "pad/guide.rs", "perspective.rs",
        "remote_item_use.rs", "remote_mob_use.rs", "render_tick.rs", "shader_packs.rs",
        "storage.rs", "titles.rs", "use_hand.rs", "using_item.rs",
        "native_host_bridge.rs",
    ]
    for path in readonly:
        edit("crates/horizon-client/src/game/" + path, lambda text: text.replace("self.selected", "self.client_hotbar_slot()"))

    def player_equipment(text):
        text = replace_once(text, "    pub(super) fn apply_equipment(&mut self) {\n", "    pub(super) fn apply_equipment(&mut self) {\n        let selected = self.client_hotbar_slot();\n")
        return replace_once(text, "self.inventory[self.selected].as_ref()", "self.inventory[selected].as_ref()")
    edit("crates/horizon-client/src/game/player_equip.rs", player_equipment)

    def station_control(text):
        return replace_once(text, "                let held = self.inventory[self.selected]\n", "                let selected = self.client_hotbar_slot();\n                let held = self.inventory[selected]\n")
    edit("crates/horizon-client/src/game/station_control.rs", station_control)

    def held(text):
        text = replace_once(text, "Hand::Main => self.selected,", "Hand::Main => self.client_hotbar_slot(),")
        return replace_once(text, "self.inventory[self.selected].as_ref()", "self.inventory[self.client_hotbar_slot()].as_ref()")
    edit("crates/horizon-client/src/game/held_items.rs", held)

    def hands(text):
        text = text.replace("slot == self.selected", "slot == self.client_hotbar_slot()")
        text = text.replace("self.stack_in(self.selected)", "self.stack_in(self.client_hotbar_slot())")
        text = replace_once(text, "let main = self.inventory[self.selected].clone();", "let main = self.inventory[self.client_hotbar_slot()].clone();")
        text = replace_once(text, "let next = [(self.selected, main),", "let next = [(self.client_hotbar_slot(), main),")
        return replace_once(text, "[held(self.selected), held(super::shield::OFFHAND_SLOT)]", "[held(self.client_hotbar_slot()), held(super::shield::OFFHAND_SLOT)]")
    edit("crates/horizon-client/src/game/hand.rs", hands)

    def equipment(text):
        text = replace_once(text, "PlayerSlot::Main => &self.inventory[self.selected],", "PlayerSlot::Main => &self.inventory[self.client_hotbar_slot()],")
        text = replace_once(text, "PlayerSlot::Main => &mut self.inventory[self.selected],", "PlayerSlot::Main => { let slot = self.client_hotbar_slot(); &mut self.inventory[slot] },")
        return replace_once(text, "(&self.inventory[self.selected], \"mainhand\")", "(&self.inventory[self.client_hotbar_slot()], \"mainhand\")")
    edit("crates/horizon-client/src/game/enchant_effects.rs", equipment)

    # Inventory.removeFromSelected/setSelectedItem and LivingEntity's main-hand
    # SlotAccess must use the same slot as the getter. Cache before field borrows.
    def totem(text):
        text = replace_once(text, "        let cell = if main {\n", "        let selected = self.client_hotbar_slot();\n        let cell = if main {\n")
        return replace_once(text, "&mut self.inventory[self.selected]", "&mut self.inventory[selected]")
    edit("crates/horizon-client/src/game/totem.rs", totem)

    def item_entities(text):
        text = replace_once(text, "        let spectator = self.spectator();\n        let Some(thrown)", "        let spectator = self.spectator();\n        let selected = self.client_hotbar_slot();\n        let Some(thrown)")
        return replace_once(text, "&mut self.inventory[self.selected]", "&mut self.inventory[selected]")
    edit("crates/horizon-client/src/game/item_entities.rs", item_entities)

    def mounts(text):
        for signature in ["    fn interact_steerable(&mut self, index: usize, item: &str) -> bool {\n", "    pub(super) fn use_steering_stick(&mut self, item: &str) -> bool {\n"]:
            text = replace_once(text, signature, signature + "        let selected = self.client_hotbar_slot();\n")
        return text.replace("self.inventory[self.selected]", "self.inventory[selected]")
    edit("crates/horizon-client/src/game/mounts.rs", mounts)

    def nautili(text):
        text = replace_once(text, "        if bucket && !self.creative {\n", "        if bucket && !self.creative {\n            let selected = self.client_hotbar_slot();\n")
        text = replace_once(text, "            // tame adult.\n            let m = &mut self.mobs[index];", "            // tame adult.\n            let selected = self.client_hotbar_slot();\n            let m = &mut self.mobs[index];")
        return text.replace("self.inventory[self.selected]", "self.inventory[selected]")
    edit("crates/horizon-client/src/game/nautili.rs", nautili)

    def shelf(text):
        text = replace_once(text, "            let held = self.inventory[self.selected].clone();", "            let selected = self.client_hotbar_slot();\n            let held = self.inventory[selected].clone();")
        return text.replace("self.inventory[self.selected]", "self.inventory[selected]")
    edit("crates/horizon-client/src/game/shelf.rs", shelf)

    def command_items(text):
        for signature in ["    pub(in crate::game) fn slot_get(&mut self, s: &Slot) -> Option<ItemStack> {\n", "    pub(in crate::game) fn slot_set(&mut self, s: &Slot, stack: Option<ItemStack>) -> bool {\n"]:
            text = replace_once(text, signature, signature + "        let selected = self.client_hotbar_slot();\n")
        return text.replace("self.inventory.get(self.selected)", "self.inventory.get(selected)").replace("self.inventory.get_mut(self.selected)", "self.inventory.get_mut(selected)")
    edit("crates/horizon-client/src/game/commands/items.rs", command_items)

    def command_effects(text):
        text = replace_once(text, "                    let Some(stack) = self.inventory[self.selected].clone()", "                    let selected = self.client_hotbar_slot();\n                    let Some(stack) = self.inventory[selected].clone()")
        return replace_once(text, "self.inventory[self.selected].as_mut()", "self.inventory[selected].as_mut()")
    edit("crates/horizon-client/src/game/commands/effects.rs", command_effects)
    edit("crates/horizon-client/src/game/commands/terrain_cmds.rs", lambda text: text.replace("super::items::Slot::Inv(g.selected)", "super::items::Slot::MainHand").replace("super::items::Slot::Inv(self.selected)", "super::items::Slot::MainHand"))

    edit("crates/horizon-client/src/game/native_living_refs.rs", lambda text: replace_once(text, "                self.selected,", "                self.client_hotbar_slot(),"))
    edit("crates/horizon-client/src/game/mob_blocks.rs", lambda text: replace_once(text, "                    self.selected\n", "                    self.client_hotbar_slot()\n"))
    edit("crates/horizon-client/src/game.rs", lambda text: replace_once(text, "self.inventory.get(self.selected).and_then(Option::as_ref)", "self.inventory.get(self.client_hotbar_slot()).and_then(Option::as_ref)"))

    def native_server(text):
        text = replace_once(text, "self.inventory.get(self.selected)?.as_ref()?", "self.inventory.get(self.client_hotbar_slot())?.as_ref()?")
        return replace_once(text, "        let selected = self.selected;\n        take_from(", "        let selected = self.client_hotbar_slot();\n        take_from(")
    edit("crates/horizon-client/src/game/native_server.rs", native_server)

    def play_keys(text):
        text = replace_once(text, "                std::mem::swap(&mut self.offhand, &mut self.inventory[self.selected]);", "                let selected = self.client_hotbar_slot();\n                std::mem::swap(&mut self.offhand, &mut self.inventory[selected]);")
        text = replace_once(text, "                self.hand_stack_replaced(self.selected);", "                self.hand_stack_replaced(selected);")
        return replace_once(text, "(self.offhand.clone(), self.inventory[self.selected].clone())", "(self.offhand.clone(), self.inventory[selected].clone())")
    edit("crates/horizon-client/src/game/play_keys.rs", play_keys)

    helpers = ["client_real_selected_slot", "client_serverside_slot", "client_clientside_slot", "client_select_slot_silently", "client_reset_silent_slot", "client_silent_slot_modified", "client_silent_slot_modified_by"]
    fragment = {
        "helpers": [{"name": name, "target": f"horizon_client::game::native_hooks::Game<'_>::native_{name}"} for name in helpers],
        "hooks": [],
        "rust_summary": "Local actor SilentHotbar real/effective slot lifecycle, cancellable selection service, and ordinary NoSlotSet reply.",
        "doc_summary": "Separate actual selected field and accepted wire observations; replay bypasses NoSlotSet; writer and service locks released before ordinary send.",
    }
    return {
        "crates/horizon-client/src/slot_bridge.rs": (HERE / "slot-bridge.rs").read_text(encoding="utf-8"),
        "tools/native-hook-fragments/liquidbounce_slots.json": json.dumps(fragment, indent=2) + "\n",
    }
