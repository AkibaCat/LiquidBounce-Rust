#!/usr/bin/env python3
HOOKS='''    pub(super) fn native_client_ping_spoof_context(&self)->Option<[bool;2]>{
        if !self.native_window()||self.lan_acting.is_some()||self.lan_acting_uuid.is_some(){return None;}
        let p=self.player.as_ref().filter(|p|p.native_client_queries_allowed())?;
        Some([!p.health.dead_or_dying(),self.has_singleplayer_server()])
    }
'''
def apply(edit,once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,HOOKS+'\n'+anchor))
    return {'hook_names':[],'helper_names':['client_ping_spoof_context']}
