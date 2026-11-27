#!/usr/bin/env python3
"""Complete native FreeCam consumers; appended after the two existing navigators."""
from pathlib import Path
import json
HERE=Path(__file__).resolve().parent

def apply(edit,once):
 for dst,src in [('free_cam.rs','free-cam-game.rs'),('free_cam_ray.rs','free-cam-ray.rs')]:
  edit('crates/horizon-client/src/game/'+dst,lambda _,src=src:(HERE/src).read_text(encoding='utf8'))
 edit('crates/horizon-client/src/free_cam_bridge.rs',lambda _:(HERE/'free-cam-bridge.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/client_lib.rs',lambda t:once(t,'pub mod freeze_bridge;','pub mod freeze_bridge;\npub mod free_cam_bridge;'))
 def entry(t):
  t=once(t,'    horizon_client::freeze_bridge::install(dispatch_freeze_movement);','    horizon_client::freeze_bridge::install(dispatch_freeze_movement);\n    horizon_client::free_cam_bridge::install(dispatch_free_cam_policy);')
  return t+'\nfn dispatch_free_cam_policy(kind:u8,x:f64,z:f64,flags:&mut u32){crate::mod_api::dispatch_client_free_cam_policy(crate::mod_api::mods(),kind,x,z,flags);}\n'
 edit('crates/horizon-client/src/entry.rs',entry)
 edit('tools/native-hook-fragments/liquidbounce_free_cam.json',lambda _:json.dumps({'hooks':[dict(name='client_free_cam_policy',target='horizon_client_lib::free_cam_bridge::native_client_free_cam_policy',side='client',cancellable=False,mod_api={'contract':'listen'})],'helpers':[]},indent=2)+'\n')
 def game(t):
  t=once(t,'mod auto_walk_runtime;','mod auto_walk_runtime;\nmod free_cam;\nmod free_cam_ray;')
  a='        let (mut yaw, mut pitch, mut managed) = (self.yaw, self.pitch, false);\n'
  i=t.index('    fn aim(&self)'); j=t.index(a,i)
  t=t[:j]+'        if let Some(aim)=self.client_free_cam_aim(){return Some(aim);}\n'+t[j:]
  return t
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(t):
  a='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  t=once(t,a,(HERE/'free-cam-host.rs').read_text(encoding='utf8')+'\n'+a)
  t=once(t,'        *self.client_stronghold_cache.borrow_mut()=Default::default();','        self.client_free_cam_event(serde_json::json!({"phase":"world"}));\n        *self.client_stronghold_cache.borrow_mut()=Default::default();')
  a='    pub(super) fn client_block_aim(&self) -> Option<([f64; 3], [f64; 3])> {\n'
  t=once(t,a,a+'        if self.camera_entity().is_none(){if let Some(aim)=self.client_free_cam_aim(){return Some(aim);}}\n')
  t=once(t,'player.bounds(), f64::from(range))','{let bounds=player.bounds();if self.client_free_cam_active()&&horizon_client::free_cam_bridge::flags()&2!=0 {let real=self.eye_position()?;let delta=Vec3::new(eye[0]-real[0],eye[1]-real[1],eye[2]-real[2]);bounds.move_by(delta)}else{bounds}}, f64::from(range))')
  a='        let navigation = self.native_client_auto_walk_input([*forward,*backward,*left,*right,*jump,*shift,*sprint]);'
  t=once(t,a,'        if self.client_free_cam_active(){[*forward,*backward,*left,*right,*jump,*shift]=[false;6];}\n'+a)
  a='        if let Some((keys,_)) = fight_navigation { [*forward,*backward,*left,*right,*jump,*shift,*sprint] = keys; }\n'
  t=once(t,a,a+'        let free_cam_navigation=self.native_client_free_cam_navigation_input([*forward,*backward,*left,*right,*jump,*shift,*sprint]);\n        if let Some((keys,_))=free_cam_navigation{[*forward,*backward,*left,*right,*jump,*shift,*sprint]=keys;}\n')
  a='        if let Some((_,Some(keys))) = fight_navigation { [*forward,*backward,*left,*right,*jump,*shift] = [keys[0],keys[1],keys[2],keys[3],keys[4],keys[5]]; }\n'
  t=once(t,a,a+'        if let Some((_,Some(keys)))=free_cam_navigation{[*forward,*backward,*left,*right,*jump,*shift]=[keys[0],keys[1],keys[2],keys[3],keys[4],keys[5]];}\n')
  t=once(t,'        self.liquid_walk_input(*shift,jump);','        self.liquid_walk_input(*shift,jump);\n        if self.client_free_cam_keep_sneaking(){*shift=true;}')
  a='        mod_api::dispatch_mouse_button_pre(\n'
  t=once(t,a,'        if button==2&&pressed{self.client_free_cam_middle();}\n'+a)
  return t
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 def owners(t):
  t=once(t,'    FightBot,\n','    FightBot,\n    FreeCam,\n')
  t=once(t,'            Self::FightBot => 1,','            Self::FightBot => 1,\n            Self::FreeCam => 2,')
  return once(t,'claims: [Option<Claim>; 2]','claims: [Option<Claim>; 3]')
 edit('crates/horizon-client/src/game/navigation_owners.rs',owners)
 def navigation(t):
  end=t.index('fn key_input(');tail=t[end:];t=t[:end]
  a=' static FIGHT_RETIRED:Cell<bool>=const{Cell::new(false)};'
  t=once(t,a,a+'''
 static FREE_CAM_RUNTIME:RefCell<Option<FreeCamNavigatorRuntime>>=const{RefCell::new(None)};
 static FREE_CAM_CONFIG:RefCell<Value>=const{RefCell::new(Value::Null)};
 static FREE_CAM_RESULT:RefCell<Option<String>>=const{RefCell::new(None)};
 static FREE_CAM_EVENTS:RefCell<Vec<Event>>=const{RefCell::new(Vec::new())};
 static FREE_CAM_RETIRED:Cell<bool>=const{Cell::new(false)};''')
  for old,new in [
   ('    FIGHT_EVENTS.with(|s| s.borrow_mut().push(event));','    FREE_CAM_EVENTS.with(|s|s.borrow_mut().push(event));'),
   ('    FIGHT_RETIRED.set(true);','    FREE_CAM_RETIRED.set(true);'),
   ('    FIGHT_EVENTS.with(|s| s.borrow_mut().clear());','    FREE_CAM_EVENTS.with(|s|s.borrow_mut().clear());'),
   ('    FIGHT_RESULT.with(|s| s.borrow_mut().take());','    FREE_CAM_RESULT.with(|s|s.borrow_mut().take());')]:
   t=once(t,old,old+'\n'+new)
  return t+tail+'\n'+(HERE/'free-cam-navigation-runtime.rs').read_text(encoding='utf8')
 edit('crates/horizon-client/src/game/auto_walk_runtime.rs',navigation)
 def camera(t):
  i=t.index('    pub fn align_with_entity_free_look('); j=t.index('    /// \u0060Camera.getMaxZoom',i)
  original=t[i:j]
  extended=original.replace('align_with_entity_free_look','align_with_entity_free_cam',1).replace('        mut clip: impl FnMut','        free_cam: Option<([f64;3],[f32;2])>,\n        mut clip: impl FnMut',1)
  extended=once(extended,'        self.set_rotation(entity.view_y_rot(partial), entity.view_x_rot(partial));','        let rotation=free_cam.map_or([entity.view_y_rot(partial),entity.view_x_rot(partial)],|(_,r)|r);\n        self.set_rotation(rotation[0],rotation[1]);')
  extended=once(extended,'        self.eye = self.position;','        if let Some((position,_))=free_cam{self.position=position;}\n        self.eye = self.position;')
  end=extended.rfind('    }')
  extended=extended[:end]+'        self.detached |= free_cam.is_some();\n'+extended[end:]
  wrapper='''    pub fn align_with_entity_free_look(
        &mut self,entity:&CameraEntity,partial:f32,camera_type:CameraType,
        adjust_distance:impl FnOnce(f32)->f32,adjust_position:impl FnOnce([f64;3])->[f64;3],
        free_look_rotation:Option<[f32;2]>,clip:impl FnMut([f64;3],[f64;3])->Option<[f64;3]>,
    ){
        self.align_with_entity_free_cam(entity,partial,camera_type,adjust_distance,adjust_position,free_look_rotation,None,clip);
    }
    #[allow(clippy::too_many_arguments)]
'''
  return t[:i]+wrapper+extended+t[j:]
 edit('crates/horizon-client-render/src/camera.rs',camera)
 def perspective(t):
  t=once(t,'        let mut view = self.view;','        let free_cam=self.client_free_cam_camera(partial);\n        let mut view = self.view;')
  t=once(t,'        view.align_with_entity_free_look(','        view.align_with_entity_free_cam(')
  t=once(t,'            free_look_rotation,\n','            free_look_rotation,\n            free_cam,\n')
  return once(t,'!self.effective_camera_type().first_person() || self.sleeping.is_some() || self.studio.flying()','!self.effective_camera_type().first_person() || self.sleeping.is_some() || self.studio.flying() || self.client_free_cam_active()')
 edit('crates/horizon-client/src/game/perspective.rs',perspective)
 def motion(t):
  a='        let before = self.player.position;\n'
  i=t.index('    pub fn move_self(');j=t.index(a,i)
  t=t[:j]+'        if self.player.client_high_jump_owner&&self.player.client_side&&self.player.native_client_queries_allowed(){crate::free_cam_bridge::moved(movement.x,movement.z);}\n'+t[j:]
  return once(t,'} else if self.player.may_fly && !was_jumping && jumping && !self.player.was_auto_jump {','} else if self.player.may_fly && !(self.player.client_high_jump_owner&&self.player.client_side&&self.player.native_client_queries_allowed()&&crate::free_cam_bridge::active()) && !was_jumping && jumping && !self.player.was_auto_jump {')
 edit('crates/horizon-client/src/motion.rs',motion)
 def sender(t):
  a='    pub fn update(\n'
  i=t.index(a,t.index('impl PositionSender'));j=t.index('        self.reminder += 1;',i)
  signature=t[i:j]
  wrapper=signature+'        self.update_free_cam(position,rotation,on_ground,collision,true)\n    }\n'
  extended=signature.replace('pub fn update(','pub fn update_free_cam(').replace('        collision: bool,\n','        collision: bool,\n        allow_rotation: bool,\n')
  t=t[:i]+wrapper+extended+t[j:]
  return once(t,'let rotated = rotation.0 - self.rotation.0 != 0.0 || rotation.1 - self.rotation.1 != 0.0;','let rotated = allow_rotation && (rotation.0 - self.rotation.0 != 0.0 || rotation.1 - self.rotation.1 != 0.0);')
 edit('crates/horizon-client/src/player_sync.rs',sender)
 def remote(t):
  t=once(t,'C::PlayerPosition(p) => {self.native_client_freeze_flag();','C::PlayerPosition(p) => {self.client_free_cam_event(serde_json::json!({"phase":"teleport"}));self.native_client_freeze_flag();')
  t=once(t,'            C::SetHealth(p) => {','            C::SetHealth(p) => {\n                self.client_free_cam_event(serde_json::json!({"phase":"damage","health":p.health,"previous":self.player.as_ref().map(|p|p.health.health())}));')
  a='    fn no_slow_vanilla_send_state(&mut self, input: u8) {\n'
  t=once(t,a,a+'        let free_cam_rotation=!self.client_free_cam_active()||horizon_client::free_cam_bridge::allow_rotation();\n')
  return once(t,'.update(pos, (yaw, pitch), on_ground, collision)','.update_free_cam(pos, (yaw, pitch), on_ground, collision,free_cam_rotation)')
 edit('crates/horizon-client/src/game/remote.rs',remote)
 edit('crates/horizon-client/src/game/crosshair.rs',lambda t:once(t,'    pub(super) fn client_crosshair_hide(&self) -> bool {\n','    pub(super) fn client_crosshair_hide(&self) -> bool {\n        if self.client_free_cam_hide_crosshair(){return true;}\n'))
 edit('crates/horizon-client/src/game/names.rs',lambda t:once(t,'        if !self.settings.extras.own_name_tag\n            || self.effective_camera_type().first_person()','        if (!self.settings.extras.own_name_tag&&!self.client_free_cam_active())\n            || (self.effective_camera_type().first_person()&&!self.client_free_cam_active())'))
 edit('crates/horizon-client/src/game/occlusion_culling.rs',lambda t:once(t,'            && !self.spectator()','            && !self.spectator()\n            && !self.client_free_cam_active()'))
 return {'hook_names':['client_free_cam'],'helper_names':['client_free_cam_context','client_free_cam_trace','client_free_cam_navigation_config','client_free_cam_navigation_result']}
