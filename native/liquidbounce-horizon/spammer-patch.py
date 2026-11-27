#!/usr/bin/env python3
"""Actual chat/command send and the remote tab-list player names."""
HOOKS='''    pub(super) fn native_client_spammer_players(&self)->Vec<String>{
        let own=self.own_profile_name();
        self.remote_online_names().unwrap_or_default().into_iter().filter(|n|*n!=own).collect()
    }
    pub(super) fn native_client_spammer_send(&mut self,message:&str){
        if self.native_window() && self.remote.is_some() && self.lan_acting.is_none() && self.lan_acting_uuid.is_none(){
            if let Some(command)=message.strip_prefix('/') {self.send_command(command);}else{self.send_chat(message);}
        }
    }
'''
def apply(edit,once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,HOOKS+'\n'+anchor))
    return {'hook_names':[],'helper_names':['client_spammer_players','client_spammer_send']}
