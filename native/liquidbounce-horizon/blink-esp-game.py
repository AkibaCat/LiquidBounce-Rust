"""Clone pinned actual player render-state extraction; never mutates the live entity."""
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def source():
 original=(ROOT/'upstream/Horizon/crates/horizon-client/src/game/perspective.rs').read_text(encoding='utf8')
 start=original.index('    pub(super) fn player_model_geometry(');end=original.index('    /// `extractEntityInInventoryFollowsMouse`',start)
 t=original[start:end]
 t=t.replace('pub(super) fn player_model_geometry(','fn client_blink_model(').replace('        alpha: f64,','        model: &serde_json::Value,')
 t=t.replace('''        if !self.player_model_shown() {
            return;
        }
        let Some(m) = self.player_model.as_ref() else {
            return;
        };''','''        let Some(source)=self.player_model.as_ref() else{return};
        let Ok(position)=serde_json::from_value::<[f64;3]>(model["position"].clone())else{return};
        let Ok(rotation)=serde_json::from_value::<[f32;2]>(model["rotation"].clone())else{return};
        let light_percent=model["lightPercent"].as_i64().unwrap_or(60)as f32 * 0.01;
        let outline=model["outlineColor"].as_i64().unwrap_or(0)as u32;
        let original_position=source.position;
        let mut snapshot=source.clone();
        snapshot.position=Vec3::new(position[0],position[1],position[2]);
        snapshot.previous=snapshot.position;
        snapshot.yaw=rotation[0];snapshot.previous_yaw=rotation[0];
        snapshot.body_yaw=rotation[0];snapshot.previous_body_yaw=rotation[0];
        snapshot.head_yaw=rotation[0];snapshot.previous_head_yaw=rotation[0];
        snapshot.head_pitch=rotation[1];snapshot.previous_head_pitch=rotation[1];
        let m=&snapshot;''')
 t=t.replace('entity_light_block(m.position,','entity_light_block(original_position,')
 old='        let packed = (u32::from(light >> 4) * 16) | ((u32::from(light & 15) * 16) << 8);'
 assert t.count(old)==1
 t=t.replace(old,'''        let scale=|v:u8|((f32::from(v)*light_percent).round()as i32).clamp(0,15)as u32;
        let packed=(scale(light>>4)*16)|((scale(light&15)*16)<<8);''')
 t=t.replace('alpha as f32','0.').replace('camera, alpha,','camera, 0.,')
 # perspective::own_entity_id is private to its module; preserve its exact live-id expression.
 t=t.replace('self.own_entity_id()', 'self.remote.as_ref().map_or(0, |r| r.entity_id)')
 t=t.replace('''            return;
        }
        // A spectator''','''            self.client_blink_outline(&out[start..],outline);
            return;
        }
        // A spectator''')
 at=t.index('        // LivingEntity.isCurrentlyGlowing: the Glowing effect.')
 t=t[:at]+'''        if !body && outline>>24!=0 {
            let mut silhouette=Vec::new();
            horizon_client::mobs::render::render(m,0.,camera,&lookup,packed,self.res.mob_data(),&mut silhouette);
            self.client_blink_outline(&silhouette,outline);
        }
        self.client_blink_outline(&out[start..],outline);
    }
'''
 prefix='''// SPDX-License-Identifier: GPL-3.0-or-later
use super::{Game,native_mob_draw::NativePlayerRender};
use horizon_core::geometry::Vec3;
use horizon_client::mesher::Vertex;
// The original light probe is private to perspective; use its exact implementation below.
'''
 helper=original[original.index('fn entity_light_block('):]
 # This helper is an ordinary balanced Rust function, extracted verbatim.
 depth=0;beg=helper.index('{');finish=None
 for i in range(beg,len(helper)):
  if helper[i]=='{':depth+=1
  elif helper[i]=='}':
   depth-=1
   if depth==0:finish=i+1;break
 assert finish
 prefix+=helper[:finish]+'\nimpl Game<\'_>{\n'
 prefix+='''    fn client_blink_outline(&self,vertices:&[Vertex],argb:u32){
        if argb>>24==0{return;}
        let color=(argb&0xff00ff00)|((argb>>16)&255)|((argb&255)<<16);
        self.glowing_vertices.borrow_mut().extend(vertices.iter().map(|v|Vertex{color,normal:[0;4],..*v}));
    }
    pub(super) fn client_blink_frame(&self,camera:glam::DVec3,alpha:f64,out:&mut Vec<Vertex>){
        let mut output="null".to_owned();
        self.native_client_blink_esp(&alpha.to_string(),&mut output);
        let Ok(v)=serde_json::from_str::<serde_json::Value>(&output)else{return};
        horizon_client::gfx::blink_esp::begin_frame(&v["body"].to_string());
        horizon_client::gfx::blink_line::begin_frame(&v["line"].to_string());
        if !self.effective_camera_type().first_person() && !v["model"].is_null(){self.client_blink_model(camera,&v["model"],out);}
    }
'''
 return prefix+t+'}\n'

