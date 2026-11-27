from pathlib import Path
HERE=Path(__file__).resolve().parent
HOOKS='''    pub(super) fn native_client_clip_context(&self)->Option<String>{self.clip_context()}
    pub(super) fn native_client_clip_target(&self,direction:i32,length:i32,liquids:u32,stand:bool)->Option<[f64;3]>{self.clip_target(direction,length,liquids,stand)}
    pub(super) fn native_client_clip_snap(&mut self,position:[f64;3],reset:bool){self.clip_snap(position,reset);}
'''
def apply(edit,once):
    for name,path in [('client_clip',HERE/'clip-game.rs'),('client_clip_logic',HERE.parent/'liquidbounce-behaviors/src/clip.rs')]:
        content=path.read_text(encoding='utf8')
        edit('crates/horizon-client/src/game/'+name+'.rs',lambda _,v=content:v)
    edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod clip_ray;\n','mod clip_ray;\nmod client_clip;\n#[allow(dead_code)]\nmod client_clip_logic;\n'))
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,HOOKS+'\n'+anchor))
    return {'hook_names':[],'helper_names':['client_clip_context','client_clip_target','client_clip_snap']}
