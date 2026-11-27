#!/usr/bin/env python3
"""Complete FastBreak client game-mode state; compose after mining and slots."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
HELPERS='''    pub(super) fn client_fast_break_player(&self) -> bool {
        self.native_window() && self.lan_acting.is_none() && self.lan_acting_uuid.is_none()
            && self.player.as_ref().is_some_and(|p| p.native_client_queries_allowed())
    }
    pub(super) fn client_fast_break_mining_tool(&self) -> bool {
        let Some(stack) = self.inventory.get(self.client_hotbar_slot()).and_then(Option::as_ref) else { return false; };
        if stack.count == 0 { return false; }
        let tags = &self.items().item_tags;
        ["minecraft:axes", "minecraft:pickaxes", "minecraft:shovels", "minecraft:hoes"].into_iter()
            .any(|tag| tags.get(tag).is_some_and(|items| items.contains(&*stack.item)))
    }
    pub(super) fn client_fast_break_publish(&self) {
        // A swapped guest must not replace the real client's tag snapshot.
        if !self.native_window() || self.lan_acting.is_some() || self.lan_acting_uuid.is_some() { return; }
        let snapshot = self.player.as_ref().filter(|p| p.native_client_queries_allowed()).and(self.remote.as_ref())
            .filter(|remote| remote.joined && remote.reconfiguring.is_none())
            .map(|remote|horizon_client::fast_break_bridge::Snapshot {session:remote.session.serial(),mining_tool:self.client_fast_break_mining_tool()});
        horizon_client::fast_break_bridge::publish(snapshot);
    }
    pub(super) fn native_client_fast_break_update(&mut self, running: bool, damage: f32, only_tool: bool) {
        self.client_fast_break_publish();
        if !self.client_fast_break_player() { return; }
        let tool = self.client_fast_break_mining_tool();
        if !running || only_tool && !tool { return; }
        let behavior = horizon_client::fast_break::FastBreak { break_damage:damage, only_tool, ..Default::default() };
        if let Some(remote) = self.remote.as_mut() {
            let mut delay = remote.destroy_delay as i32;
            let mut progress = remote.destroying.map_or(0.0,|value|value.2);
            behavior.tick(running,tool,&mut delay,&mut progress);
            remote.destroy_delay = delay.max(0) as u32;
            if let Some(value) = remote.destroying.as_mut() { value.2 = progress; }
        } else {
            behavior.tick(running,tool,&mut self.clicks.destroying.delay,&mut self.clicks.destroying.progress);
        }
    }
    pub(super) fn native_client_fast_break_reset(&mut self) {
        if !self.native_window() || self.lan_acting.is_some() || self.lan_acting_uuid.is_some() { return; }
        let delay = horizon_client::fast_break::FastBreak::disabled_delay();
        if let Some(remote) = self.remote.as_mut() { remote.destroy_delay = delay as u32; }
        else { self.clicks.destroying.delay = delay; }
    }

'''
def apply(edit,once):
    for file,source in [('fast_break.rs',HERE.parent/'liquidbounce-behaviors/src/fast_break.rs'),('fast_break_bridge.rs',HERE/'fast-break-bridge.rs')]:
        edit('crates/horizon-client/src/'+file,lambda _,source=source:source.read_text(encoding='utf-8'))
    edit('crates/horizon-client/src/client_lib.rs',lambda text:once(text,'pub mod block_contact;\n','pub mod block_contact;\npub mod fast_break;\npub mod fast_break_bridge;\n'))
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda text:once(text,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',HELPERS+'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
    edit('crates/horizon-client/src/game/remote.rs',lambda text:once(text,'    pub(super) fn send(&self, packet: S) {\n','    pub(super) fn send(&self, packet: S) {\n        self.client_fast_break_publish();\n'))
    for field in ['destroying: Option<([i32; 3], u8, f32)>', 'destroy_delay: u32']:
        edit('crates/horizon-client/src/game/remote.rs',lambda text,field=field:once(text,'    '+field+',\n','    pub(super) '+field+',\n'))
    return {'hook_names':[],'helper_names':['client_fast_break_update','client_fast_break_reset']}
