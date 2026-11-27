#!/usr/bin/env python3
"""Committed local hook snapshots and an actual item-only interaction helper."""
from pathlib import Path
import json

HERE = Path(__file__).resolve().parent

HELPERS = '''    pub(super) fn native_client_fishing_owner(&self) -> bool {
        self.native_window() && self.lan_acting_uuid.is_none()
            && self.player.as_ref().is_some_and(|p| p.native_client_queries_allowed())
    }
    /// BuiltInRegistries.SOUND_EVENT is available before entering a world.
    pub(super) fn native_client_sound_event_names(&self) -> Vec<String> {
        if !self.native_window() { return Vec::new(); }
        super::remote::builtin_registries(&self.options.resources)
            .and_then(|builtin| builtin.get("minecraft:sound_event").map(|ids| ids.names.clone()))
            .unwrap_or_default()
    }
    pub(super) fn native_client_fishing_rod_hand(&self) -> Option<i32> {
        if !self.native_client_fishing_owner() { return None; }
        if self.stack_in(self.client_hotbar_slot()).is_some_and(|s| &*s.item == "minecraft:fishing_rod") { Some(0) }
        else if self.offhand.as_ref().is_some_and(|s| &*s.item == "minecraft:fishing_rod") { Some(1) }
        else { None }
    }
    pub(super) fn native_client_fishing_hook(&self) -> Option<[f64; 3]> {
        if !self.native_client_fishing_owner() { return None; }
        self.hook.as_ref().map(|h| [h.position.x, h.position.y, h.position.z])
    }
    /// Publish only after a hook operation commits. Guest/fake-player swaps
    /// must leave the real client's previous snapshot intact.
    pub(super) fn native_publish_fishing_state(&self) {
        if !self.native_window() || self.lan_acting_uuid.is_some()
            || self.player.as_ref().is_some_and(|p| !p.native_client_queries_allowed()) { return; }
        if !self.native_client_fishing_owner() {
            horizon_client::fish_bridge::clear();
            return;
        }
        let remote = self.remote.as_ref().filter(|r| r.joined && r.reconfiguring.is_none());
        horizon_client::fish_bridge::publish(horizon_client::fish_bridge::FishingSnapshot {
            session: remote.map_or(0, |r| r.session.serial()),
            hook: self.native_client_fishing_hook(),
            builtin: remote.map(|r| r.builtin.clone()),
        });
    }
    /// InteractionUtils.useItem: a single hand's item, with no entity/block
    /// interaction or key cooldown. The managed rotation changes only the wire
    /// packet and temporarily aligns Item.use prediction with it, as the pinned
    /// NetworkUtils.useItem overload does. Restore current angles afterwards.
    pub(super) fn native_client_use_item(&mut self, hand: i32, yaw: f32, pitch: f32) -> bool {
        if !self.native_client_fishing_owner() || self.spectator() { return false; }
        let hand = match hand { 0 => Hand::Main, 1 => Hand::Off, _ => return false };
        let used = if self.remote.is_some() {
            self.remote_use_item_rotated(hand, yaw, pitch)
        } else {
            self.native_with_item_rotation(yaw, pitch, |g| {
                g.with_hand(hand, |g| g.use_hand(None).unwrap_or(false) && !g.use_failed)
            })
        };
        if used { self.heights.item_used(hand); }
        used
    }
    pub(super) fn native_with_item_rotation<T>(&mut self, yaw: f32, pitch: f32, use_item: impl FnOnce(&mut Self) -> T) -> T {
        let previous = (self.yaw, self.pitch);
        let player_previous = self.player.as_ref().map(|p| (p.yaw, p.pitch));
        if yaw.is_finite() { self.yaw = yaw; }
        if pitch.is_finite() { self.pitch = horizon_client::packet_bridge::set_x_rot(pitch); }
        if let Some(player) = self.player.as_mut() {
            if yaw.is_finite() { player.yaw = yaw; }
            if pitch.is_finite() { player.pitch = horizon_client::packet_bridge::set_x_rot(pitch); }
        }
        let result = use_item(self);
        self.yaw = previous.0;
        self.pitch = horizon_client::packet_bridge::set_x_rot(previous.1);
        if let (Some(player), Some((yaw, pitch))) = (self.player.as_mut(), player_previous) {
            player.yaw = yaw;
            player.pitch = horizon_client::packet_bridge::set_x_rot(pitch);
        }
        result
    }

'''


def wrap(text, signature, call, replace_once):
    """Retain all early returns inside the original body; publish on commit."""
    start = text.index(signature)
    end = text.index('{', start) + 1
    depth = 1
    while depth:
        depth += (text[end] == '{') - (text[end] == '}')
        end += 1
    original = text[start:end]
    name = signature.split('fn ', 1)[1].split('(', 1)[0]
    renamed = original.replace('fn ' + name + '(', 'fn ' + name + '_committed(', 1)
    if '->' in signature:
        wrapper = signature + '\n        let result = self.' + name + '_committed(' + call + ');\n        self.native_publish_fishing_state();\n        result\n    }'
    else:
        wrapper = signature + '\n        self.' + name + '_committed(' + call + ');\n        self.native_publish_fishing_state();\n    }'
    return replace_once(text, original, wrapper + '\n\n' + renamed)


def apply(edit, replace_once):
    edit('crates/horizon-client/src/client_lib.rs', lambda text: replace_once(text,
        'pub mod packet_bridge;\n', 'pub mod packet_bridge;\npub mod fish_bridge;\n'))

    def native(text):
        text = replace_once(text, '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n', HELPERS + '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n')
        text = replace_once(text, '    pub(super) fn native_client_tick_pre(&mut self) {\n', '    pub(super) fn native_client_tick_pre(&mut self) {\n        self.native_publish_fishing_state();\n')
        for signature in ['    pub(super) fn native_client_disconnecting(&self) {\n', '    pub(super) fn native_client_level_load(&mut self, dimension: &str) {\n', '    pub(super) fn native_client_level_unload(&mut self, dimension: &str) {\n']:
            text = replace_once(text, signature, signature + '        if self.native_window() { horizon_client::fish_bridge::clear(); }\n')
        return text
    edit('crates/horizon-client/src/game/native_hooks.rs', native)

    def fishing(text):
        for signature, args in [
            ('    pub(super) fn use_fishing_rod(&mut self, stack: &ItemStack) -> bool {', 'stack'),
            ('    pub(super) fn tick_fishing(&mut self) {', ''),
            ('    pub(super) fn spawn_remote_hook(&mut self, id: u32, at: Vec3, velocity: Vec3, owner: i32) {', 'id, at, velocity, owner'),
            ('    pub(super) fn tick_remote_hooks(&mut self) {', ''),
            ('    pub(super) fn remove_server_hook(&mut self, id: u32) -> bool {', 'id'),
            ('    pub(super) fn remove_remote_hook(&mut self, id: u32) {', 'id'),
            ('    pub(super) fn clear_remote_hooks(&mut self) {', ''),
        ]:
            text = wrap(text, signature, args, replace_once)
        return text
    edit('crates/horizon-client/src/game/fishing.rs', fishing)

    def remote(text):
        signature = '    fn remote_use_item(&mut self, hand: Hand) -> bool {\n'
        start = text.index(signature)
        end = text.index('    /// Whether `held`', start)
        original = text[start:end]
        body = replace_once(original, signature, '    pub(super) fn remote_use_item_rotated(&mut self, hand: Hand, yaw: f32, pitch: f32) -> bool {\n')
        body = replace_once(body, '''        let Some(held) = self.with_hand(hand, |g| g.held().cloned()) else {
            return false;
        };''', '''        let held = self.with_hand(hand, |g| g.held().cloned())
            .unwrap_or_else(|| ItemStack::new("minecraft:air", 0));''')
        body = replace_once(body, '        self.send(S::UseItem(play::UseItem {\n', '        let packet = S::UseItem(play::UseItem {\n')
        body = replace_once(body, '            yaw: self.yaw,\n            pitch: self.pitch,\n        }));\n', '            yaw,\n            pitch,\n        });\n        let result = self.native_with_item_rotation(yaw, pitch, |game| {\n            game.remote_predict_item(hand, &held, seq)\n        });\n        self.send(packet);\n        result\n    }\n\n    fn remote_predict_item(&mut self, hand: Hand, held: &ItemStack, seq: i32) -> bool {\n')
        # The copied prediction already holds a reference to the same stack;
        # preserve the original source's typed operations without && references.
        split = body.index('    fn remote_predict_item(')
        body = body[:split] + body[split:].replace('&held)', 'held)').replace('&held,', 'held,')
        return text[:end] + body + text[end:]
    edit('crates/horizon-client/src/game/remote.rs', remote)
    helpers = ['client_fishing_rod_hand', 'client_fishing_hook', 'client_use_item', 'client_sound_event_names']
    fragment = {
        'hooks': [],
        'helpers': [{'name':name,'target':f"horizon_client::game::native_hooks::Game<'_>::native_{name}"} for name in helpers],
        'rust_summary':'AutoFish actual rod hand and hook, item-only interaction and committed reader-safe fishing snapshots.',
        'doc_summary':'Hook snapshots commit after real creation/movement/removal and reject guest/server/replay state. Sound PacketEvent reads the active session and original sound registry holder.',
    }
    return {
        'crates/horizon-client/src/fish_bridge.rs': (HERE / 'fish-bridge.rs').read_text(encoding='utf-8'),
        'tools/native-hook-fragments/liquidbounce_fishing.json': json.dumps(fragment, indent=2) + '\n',
    }
