#!/usr/bin/env python3
"""AutoAccount: accepted/delayed chat and actual title packet delivery."""
HOOKS='''    pub(super) fn native_client_auto_account_message(&mut self,source:u8,message:&str) {
        if self.native_window() && self.lan_acting.is_none() && self.lan_acting_uuid.is_none() {
            mod_api::dispatch_client_auto_account_message(self,mod_api::mods(),source,message);
        }
    }
    pub(super) fn native_client_auto_account_command(&mut self,message:&str,handled:&mut bool) {
        if self.native_window() && self.lan_acting.is_none() && self.lan_acting_uuid.is_none() {
            mod_api::dispatch_client_auto_account_command(self,mod_api::mods(),message,handled);
        }
    }
    pub(super) fn native_client_auto_account_connected(&self)->bool {
        self.native_window() && self.lan_acting.is_none() && self.lan_acting_uuid.is_none() && self.remote.is_some()
    }
    pub(super) fn native_client_auto_account_send(&mut self,command:&str) {
        if self.native_client_auto_account_connected(){self.send_command(command);}
    }
'''
def apply(edit,once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,HOOKS+'\n'+anchor))
    def remote(s):
        s=once(s,'    chat_type: i32,\n}', '    chat_type: i32,\n    auto_account_original: Option<String>,\n}')
        s=once(s,'            chat_type: chat_type.chat_type,\n        });','            chat_type: chat_type.chat_type,\n            auto_account_original: None,\n        });')
        s=once(s,'                    chat_type: p.chat_type.chat_type,\n                });\n            }\n            C::DisguisedChat', '                    chat_type: p.chat_type.chat_type,\n                    auto_account_original: None,\n                });\n            }\n            C::DisguisedChat')
        s=once(s,'                    sender: 0,\n                    chat_type: p.chat_type.chat_type,\n                });','                    sender: 0,\n                    chat_type: p.chat_type.chat_type,\n                    auto_account_original: Some(remote_text(&p.message).plain()),\n                });')
        s=once(s,'            chat_type,\n        } = chat;','            chat_type,\n            auto_account_original,\n        } = chat;')
        s=once(s,'        let error = matches!(trust, ChatTrust::Error);\n','        let error = matches!(trust, ChatTrust::Error);\n        if !error {self.native_client_auto_account_message(0,auto_account_original.as_deref().unwrap_or(&line.plain()));}\n')
        s=once(s,'            C::SystemChat(p) => {\n','            C::SystemChat(p) => {\n                self.native_client_auto_account_message(0,&remote_text(&p.content).plain());\n')
        s=once(s,'            C::SetTitleText(p) => self.titles.set_title(remote_text(&p.text)),','            C::SetTitleText(p) => {let text=remote_text(&p.text);self.native_client_auto_account_message(1,&text.plain());self.titles.set_title(text);},')
        s=once(s,'            C::SetSubtitleText(p) => self.titles.set_subtitle(remote_text(&p.text)),','            C::SetSubtitleText(p) => {let text=remote_text(&p.text);self.native_client_auto_account_message(2,&text.plain());self.titles.set_subtitle(text);},')
        return s
    edit('crates/horizon-client/src/game/remote.rs',remote)
    edit('crates/horizon-client/src/game/chat.rs',lambda s:once(s,'    pub(super) fn handle_chat_input(&mut self, message: &str) {','    pub(super) fn handle_chat_input(&mut self, message: &str) {\n        let mut handled=false;self.native_client_auto_account_command(message,&mut handled);if handled{return;}'))
    return {'hook_names':['client_auto_account_message','client_auto_account_command'],'helper_names':['client_auto_account_connected','client_auto_account_send']}
