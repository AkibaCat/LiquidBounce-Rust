#!/usr/bin/env python3
"""Game-thread drain before accepted flags; actual chat command consumption."""
HOOKS='''    pub(super) fn native_client_auto_disable_flag(&mut self) {
        if self.native_client_slot_owner(){mod_api::dispatch_client_auto_disable_flag(self,mod_api::mods());}
    }
    pub(super) fn native_client_auto_disable_death(&mut self) {
        if self.native_client_slot_owner(){mod_api::dispatch_client_auto_disable_death(self,mod_api::mods());}
    }
    pub(super) fn native_client_auto_disable_lifecycle(&mut self,reason:u8) {
        if self.native_client_slot_owner(){mod_api::dispatch_client_auto_disable_lifecycle(self,mod_api::mods(),reason);}
    }
    pub(super) fn native_client_auto_disable_command(&mut self,message:&str,handled:&mut bool) {
        if self.native_client_slot_owner(){mod_api::dispatch_client_auto_disable_command(self,mod_api::mods(),message,handled);}
    }
'''
def apply(edit,once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda source:once(source,anchor,HOOKS+'\n'+anchor))
    edit('crates/horizon-client/src/game/remote.rs',lambda source:once(source,'            C::PlayerPosition(p) => self.remote_teleport(&p)?,','            C::PlayerPosition(p) => {self.native_client_auto_disable_flag();self.remote_teleport(&p)?;},'))
    anchor='                let me = self.remote.as_ref().is_some_and(|r| r.entity_id == p.id);\n'
    edit('crates/horizon-client/src/game/remote.rs',lambda source:once(source,anchor,anchor+'                if me {self.native_client_auto_disable_death();}\n'))
    edit('crates/horizon-client/src/game/chat.rs',lambda source:once(source,'    pub(super) fn handle_chat_input(&mut self, message: &str) {','    pub(super) fn handle_chat_input(&mut self, message: &str) {\n        let mut handled=false;self.native_client_auto_disable_command(message,&mut handled);if handled{return;}'))
    # This extension runs after timer-patch, at its exact level replacement
    # points. A dedicated mutable event permits immediate onDisabled cleanup.
    edit('crates/horizon-client/src/game/remote.rs',lambda source:once(source,'            self.native_client_world_change(Some(&spawn.dimension));','            self.native_client_auto_disable_lifecycle(2);\n            self.native_client_world_change(Some(&spawn.dimension));'))
    edit('crates/horizon-client/src/game/remote.rs',lambda source:once(source,'        self.native_client_world_change(None);','        self.native_client_auto_disable_lifecycle(2);\n        self.native_client_world_change(None);'))
    edit('crates/horizon-client/src/game.rs',lambda source:once(source,'        self.native_client_world_change(None);','        self.native_client_auto_disable_lifecycle(2);\n        self.native_client_world_change(None);'))
    edit('crates/horizon-client/src/game/remote.rs',lambda source:once(source,'            self.native_client_disconnecting();','            self.native_client_auto_disable_lifecycle(3);\n            self.native_client_disconnecting();'))
    return {'hook_names':['client_auto_disable_flag','client_auto_disable_death','client_auto_disable_lifecycle','client_auto_disable_command'],'helper_names':[]}
