#!/usr/bin/env python3
"""MaceKill block-only height search and LocalPlayer.warp packet semantics."""
from pathlib import Path
import re
HERE=Path(__file__).resolve().parent

def apply(edit,once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda text:once(text,anchor,(HERE/'mace-kill-host.rs').read_text(encoding='utf8')+'\n'+anchor))
    edit('crates/horizon-client/src/game/mace_kill_height.rs',lambda _:(HERE.parent/'liquidbounce-behaviors/src/mace_kill_height.rs').read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game/mace_kill_vehicles.rs',lambda _:(HERE/'mace-kill-vehicles.rs').read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game.rs',lambda text:once(text,'mod native_hooks;\n','mod native_hooks;\nmod mace_kill_height;\nmod mace_kill_vehicles;\n'))
    anchor='    pub(super) fn tracked_entity(&self, id: i32) -> Option<(&str, [f64; 3])> {\n'
    helper='''    pub(super) fn liquidbounce_mace_kill_state(&mut self,id:i32,position:Option<[f64;3]>,sync_codec:bool)->Option<([f64;3],[f32;2],bool)>{
        let entity=self.entities.get_mut(&id)?;
        if let Some(position)=position{
            entity.position=position;
            if sync_codec{entity.codec_base=position;}
        }
        Some((entity.position,entity.liquidbounce_rotation,entity.liquidbounce_on_ground))
    }
'''
    def remote(text):
        text=once(text,'    vehicle: Option<u32>,\n','    vehicle: Option<u32>,\n    liquidbounce_rotation: [f32;2],\n    liquidbounce_on_ground: bool,\n')
        text=once(text,'                    codec_base: position,\n','                    codec_base: position,\n                    liquidbounce_rotation: [yaw,pitch],\n                    liquidbounce_on_ground: false,\n')
        text,count=re.subn(r'(?m)^([ \t]+)codec_base: (\[-?\d[^\n]+\]),$',lambda m:m[0]+'\n'+m[1]+'liquidbounce_rotation: [0.0;2],\n'+m[1]+'liquidbounce_on_ground: false,',text)
        if count!=3:raise ValueError(f'Expected three Tracked test initializers, got {count}')
        text=once(text,'                t.position = position;\n','''                t.position = position;
                t.liquidbounce_on_ground=on_ground;
                if let Some((yaw,pitch))=rotation{
                    if yaw.is_finite(){t.liquidbounce_rotation[0]=yaw;}
                    if pitch.is_finite(){t.liquidbounce_rotation[1]=horizon_client::packet_bridge::set_x_rot(pitch);}
                }
''')
        return once(text,anchor,helper+anchor)
    edit('crates/horizon-client/src/game/remote.rs',remote)
    anchor='    pub(super) fn position(&self) -> glam::DVec3 {\n'
    helper='''    pub(super) fn liquidbounce_mace_kill_position(&mut self,position:Option<[f64;3]>)->([f64;3],[f32;2]){
        if let Some(position)=position{self.position=position.into();}
        (self.position.to_array(),self.rotation)
    }
'''
    edit('crates/horizon-client/src/game/text_displays.rs',lambda text:once(text,anchor,helper+anchor))
    return {'hook_names':[],'helper_names':['client_mace_kill_context','client_mace_kill_height','client_mace_kill_warp']}
