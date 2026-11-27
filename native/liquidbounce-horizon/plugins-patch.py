#!/usr/bin/env python3
"""Plugins real command/suggestion sends and original system-message content."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
HOOKS='''    pub(super) fn native_client_plugins_game_message(&mut self,message:&str) {
        if self.native_client_slot_owner(){mod_api::dispatch_client_plugins_game_message(self,mod_api::mods(),message);}
    }
    pub(super) fn native_client_plugins_server_commands(&self,names:&[String]) {
        if self.native_client_slot_owner(){mod_api::dispatch_client_plugins_server_commands(self,mod_api::mods(),names);}
    }
'''
def apply(edit,once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda source:once(source,anchor,HOOKS+(HERE/'plugins-host.rs').read_text(encoding='utf8')+'\n'+anchor))
    # ChatReceiveEvent GAME_MESSAGE is emitted at handleSystemMessage HEAD;
    # player/disguised messages and client-added lines must not satisfy it.
    anchor='''            C::SystemChat(p) => {
'''
    edit('crates/horizon-client/src/game/remote.rs',lambda source:once(source,anchor,anchor+'                self.native_client_plugins_game_message(&remote_text(&p.content).plain());\n'))
    anchor='''            C::Commands(p) => {
'''
    edit('crates/horizon-client/src/game/remote.rs',lambda source:once(source,anchor,anchor+'''                let names:Vec<String>=p.nodes.get(p.root as usize).into_iter().flat_map(|root|root.children.iter()).filter_map(|&index|p.nodes.get(index as usize)).map(|node|node.name.clone()).collect();
                self.native_client_plugins_server_commands(&names);
'''))
    return {'hook_names':['client_plugins_game_message','client_plugins_server_commands'],'helper_names':['client_plugins_command','client_plugins_suggest','client_plugins_decode','client_plugins_chat']}
