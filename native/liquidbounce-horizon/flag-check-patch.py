"""FlagCheck reads actual client-player lifetime and attributes."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,(HERE/'flag-check-host.rs').read_text(encoding='utf8')+'\n'+anchor))
    def chat(s):
        s=once(s,'    signature: Option<Box<Signature>>,','    signature: Option<Box<Signature>>,\n    client_id: Option<String>,')
        s=once(s,'                signature: signature.map(Box::new),','                signature: signature.map(Box::new),\n                client_id: None,')
        # There are two original None initializers (ordinary + deletion).
        if s.count('signature: None,')!=2:raise RuntimeError('FlagCheck chat metadata constructors changed')
        s=s.replace('signature: None,','signature: None,\n                client_id: None,')
        return once(s,'    pub fn lines_per_page(&self, focused: bool) -> usize {','''    /// Client metadata replacement, preserving every unrelated chat entry.
    pub fn add_client_id(&mut self,atlas:&HudAtlas,id:&str,content:Text,ticks:u64,focused:bool){
        self.messages.retain(|m|m.client_id.as_deref()!=Some(id));
        self.rewrap(&atlas.font);
        self.push_message(&atlas.font,Message{added:ticks,time:(0,0),repeats:1,wrapped:0,content,tag:None,source:ChatSource::Client,signature:None,client_id:Some(id.into())},focused);
    }
    pub fn lines_per_page(&self, focused: bool) -> usize {''')
    edit('crates/horizon-client-render/src/chat.rs',chat)
    return {'hook_names':[],'helper_names':['client_flag_check_snapshot','client_flag_check_chat']}
