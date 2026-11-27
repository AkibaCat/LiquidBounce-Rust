#!/usr/bin/env python3
"""ReportHelper real ScreenEvent identities and insertion-ordered world players."""
from pathlib import Path
HERE=Path(__file__).resolve().parent

def apply(edit,once):
    def game(s):
        s=once(s,'    client_inventory_screen_serial: std::cell::Cell<u64>,\n','''    client_inventory_screen_serial: std::cell::Cell<u64>,
    client_report_helper_next: std::cell::Cell<u64>,
    client_report_helper_pending: std::cell::Cell<Option<u64>>,
    client_report_helper_current: std::cell::Cell<Option<u64>>,
''')
        return once(s,'            client_inventory_screen_serial: std::cell::Cell::new(1),\n','''            client_inventory_screen_serial: std::cell::Cell::new(1),
            client_report_helper_next: std::cell::Cell::new(0),
            client_report_helper_pending: std::cell::Cell::new(None),
            client_report_helper_current: std::cell::Cell::new(None),
''')
    edit('crates/horizon-client/src/game.rs',game)
    def native(s):
        anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
        s=once(s,anchor,(HERE/'report-helper-host.rs').read_text(encoding='utf8')+'\n'+anchor)
        s=once(s,'            self.client_inventory_screen_serial.set(self.client_inventory_screen_serial.get().wrapping_add(1));','''            self.client_inventory_screen_serial.set(self.client_inventory_screen_serial.get().wrapping_add(1));
            self.client_report_helper_current.set(self.client_report_helper_pending.take());''')
        # Wrap the actual HEAD event. A cancelled prospective object must never
        # become the identity of a later accepted screen with the same enum/id.
        signature='    pub(super) fn client_gui_closer_accept_screen(&mut self, next: super::Screen) -> bool {'
        s=once(s,signature,'''    pub(super) fn client_gui_closer_accept_screen(&mut self, next: super::Screen) -> bool {
        let accepted=self.client_gui_closer_accept_screen_report_helper_inner(next);
        self.client_report_helper_screen_event(next,accepted);
        accepted
    }
    fn client_gui_closer_accept_screen_report_helper_inner(&mut self, next: super::Screen) -> bool {''')
        return s
    edit('crates/horizon-client/src/game/native_hooks.rs',native)
    def remote(s):
        s=once(s,'    level_players: HashMap<i32, u128>,\n','    level_players: HashMap<i32, u128>,\n    report_helper_player_order: Vec<i32>,\n')
        if s.count('            level_players: HashMap::new(),\n')!=2:raise ValueError('ReportHelper Remote constructor count')
        s=s.replace('            level_players: HashMap::new(),\n','            level_players: HashMap::new(),\n            report_helper_player_order: Vec::new(),\n')
        s=once(s,'                r.level_players.clear();','                r.level_players.clear();\n                r.report_helper_player_order.clear();')
        s=once(s,'                    if kind == "minecraft:player" {\n                        r.level_players.insert(p.id, p.uuid);','''                    r.report_helper_player_order.retain(|&id|id!=p.id);
                    if kind == "minecraft:player" {
                        r.report_helper_player_order.push(p.id);
                        r.level_players.insert(p.id, p.uuid);''')
        s=once(s,'                        if let Some(uuid) = r.level_players.remove(&id) {','                        r.report_helper_player_order.retain(|&player|player!=id);\n                        if let Some(uuid) = r.level_players.remove(&id) {')
        s=once(s,'    fn remote_close_local(&mut self) {','    pub(super) fn remote_close_local(&mut self) {')
        anchor='    /// The profile name of another player the server sent, by entity id.\n'
        return once(s,anchor,'''    pub(super) fn client_report_helper_world_players(&self)->Vec<serde_json::Value>{
        let Some(remote)=self.remote.as_ref()else{return vec![];};
        remote.report_helper_player_order.iter().filter_map(|&id|{
            // A profile retained by a level entity survives a tab-list removal.
            let name=self.remote_player_name(id as u32)?;
            Some(serde_json::json!({"id":id,"name":name,"local":id==remote.entity_id}))
        }).collect()
    }
'''+anchor)
    edit('crates/horizon-client/src/game/remote.rs',remote)
    return {'hook_names':['client_report_helper_screen'],'helper_names':['client_report_helper_session','client_report_helper_players','client_report_helper_current','client_report_helper_screen_snapshot','client_report_helper_close']}
