#!/usr/bin/env python3
"""Synchronous Step height/success and the actual remote movement sender."""
from pathlib import Path
import json,re,runpy
HERE=Path(__file__).resolve().parent
SOURCE=HERE.parents[1]/'upstream/Horizon'
method=runpy.run_path(str(HERE/'terrain-speed-order-patch.py'))['method']
def apply(edit,once,changes):
 for target,source in [
  ('crates/horizon-client/src/step_bridge.rs',HERE/'step-bridge.rs'),
  ('crates/horizon-client/src/game/step.rs',HERE/'step-native.rs')]:
  edit(target,lambda _,p=source:p.read_text(encoding='utf8'))
 edit('crates/horizon-client/src/client_lib.rs',lambda s:once(s,'pub mod high_jump_bridge;\n','pub mod high_jump_bridge;\npub mod step_bridge;\n'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod reverse_step;\n','mod reverse_step;\nmod step;\n'))
 def entry(s):
  s=once(s,'    horizon_threads::configure_allocator();','    horizon_client::step_bridge::install(dispatch_step_height,dispatch_step_success);\n    horizon_threads::configure_allocator();')
  return s+'\nfn dispatch_step_height(height:&mut f32){crate::mod_api::dispatch_client_step_height(crate::mod_api::mods(),height);}\nfn dispatch_step_success(adjusted_y:f64,player_y:f64,result:&mut String){crate::mod_api::dispatch_client_step_success(crate::mod_api::mods(),adjusted_y,player_y,result);}\n'
 edit('crates/horizon-client/src/entry.rs',entry)
 edit('tools/native-hook-fragments/liquidbounce_step.json',lambda _:json.dumps({'hooks':[dict(name='client_step_'+name,target='horizon_client_lib::step_bridge::native_client_step_'+name,side='client',cancellable=False,mod_api={'contract':'listen'})for name in ['height','success']],'helpers':[]},indent=2)+'\n')
 def hooks(s):
  anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  s=once(s,anchor,(HERE/'step-host.rs').read_text(encoding='utf8')+'\n'+anchor)
  anchor='        horizon_client::reverse_step_bridge::set_keys([*forward,*backward,*left,*right,*jump,*shift,*sprint]);\n'
  s=once(s,anchor,anchor+'        self.step_input(jump);\n')
  return once(s,'        super::reverse_step::clear();\n','        super::reverse_step::clear();\n        horizon_client::step_bridge::clear();\n')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 def motion(s):
  s=once(s,"pub struct MotionWorld<'p, 'w, 'r> {", "pub struct MotionWorld<'p, 'w, 'r> {\n    /// Borrowed remote sender: collision RETURN packets precede body movement.\n    pub step_sender:Option<&'p dyn Fn(horizon_protocol::packets::play::Serverbound)>,")
  s=once(s,'        entity_move::move_entity(self, MoverType::SelfMovement, movement)?;','        entity_move::move_entity(self, MoverType::SelfMovement, movement)?;\n        if self.player.client_high_jump_owner&&self.player.client_side&&self.player.native_client_queries_allowed(){crate::step_bridge::moved(self.player.ground.on_ground);}')
  s=once(s,'            p.auto_jump_enabled,','            p.auto_jump_enabled || (p.client_high_jump_owner&&p.client_side&&p.native_client_queries_allowed()&&crate::step_bridge::legit()),')
  old=method(s,'    fn collide(&mut self, movement: Vec3) -> Result<Vec3> {')
  new=once(old,'        let max_step = self.player.attribute("minecraft:step_height")? as f32;','''        let mut max_step = self.player.attribute("minecraft:step_height")? as f32;
        let actual=self.player.client_high_jump_owner&&self.player.client_side&&self.player.native_client_queries_allowed();
        if actual{crate::step_bridge::height(&mut max_step);}''')
  new=once(new,'        Ok(world_movement::resolve_world_movement(','        let adjusted=world_movement::resolve_world_movement(')
  new=once(new,'        )?)','''        )?;
        if actual{
            let response=crate::step_bridge::success(adjusted.y,self.player.position.y);
            if let Ok(value)=serde_json::from_str::<serde_json::Value>(&response){
                if value["awardJump"]==true{self.player.events.push(MotionEvent::Health(crate::health::HealthEvent::Statistic{name:"jump",amount:1}));}
                if let (Some(sender),Some(destinations))=(self.step_sender,value["destinations"].as_array()){
                    use horizon_protocol::packets::play::{self,Serverbound as S};
                    let flags=u8::from(self.player.ground.on_ground)|(u8::from(self.player.collision.horizontal)<<1);
                    for y in destinations.iter().filter_map(serde_json::Value::as_f64){
                        let (x,z)=(self.player.position.x,self.player.position.z);
                        let packet=if value["fullPacket"]==true{S::MovePlayerPosRot(play::MovePlayerPosRot{x,y,z,yaw:self.player.yaw,pitch:self.player.pitch,flags})}else{S::MovePlayerPos(play::MovePlayerPos{x,y,z,flags})};
                        sender(packet);
                    }
                }
            }
        }
        Ok(adjusted)''')
  return once(s,old,new)
 edit('crates/horizon-client/src/motion.rs',motion)
 # This runs last: include both pinned constructors and adapters added earlier.
 paths={p.relative_to(SOURCE).as_posix()for p in (SOURCE/'crates').rglob('*.rs')}
 paths.update(changes)
 pattern=re.compile(r'\bMotionWorld\s*\{')
 for path in sorted(paths):
  if not path.endswith('.rs'):continue
  current=changes[path][1]if path in changes else(SOURCE/path).read_text(encoding='utf8')
  if pattern.search(current):edit(path,lambda s:pattern.sub(lambda m:m.group(0)+'step_sender:None,',s))
 def remote(s):
  anchor='        player.client_movement_queries = client_movement_queries;'
  s=once(s,anchor,anchor) # validate the actual controlled-player branch
  anchor='        let mut elytra_ai_tail = |world: &mut MotionWorld'
  index=s.index(anchor)
  s=s[:index]+'        let step_sender=|packet|{if let Some(remote)=elytra_recast_remote{remote.session.send(packet);}};\n'+s[index:]
  close='            .controlled_tick_with_ai_tail(keys, sprint_window, &mut elytra_ai_tail)?;'
  end=s.index(close);start=s.rfind('MotionWorld {',0,end)
  body=s[start:end];body=once(body,'step_sender:None,','step_sender:Some(&step_sender),');s=s[:start]+body+s[end:]
  anchor='        let (mut yaw, mut pitch) = (self.yaw, self.pitch);\n'
  # The packet flag is scoped to sendPosition's current network tick.
  start=s.index('    fn remote_send_state(&mut self, input: u8) {')
  before=s[:start];after=s[start:]
  after=once(after,anchor,'        let on_ground=horizon_client::step_bridge::ground(on_ground);\n'+anchor)
  return before+after
 edit('crates/horizon-client/src/game/remote.rs',remote)
 # Late sendPosition hooks run after Step has rewritten the final sender.
 runpy.run_path(str(HERE/'anti-void-late-patch.py'))['apply'](edit,once)
 return {'hook_names':['client_step_input'],'helper_names':['client_step_effects']}
