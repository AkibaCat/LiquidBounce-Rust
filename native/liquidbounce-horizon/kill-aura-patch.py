"""KillAura actual native targeting, item use and all-category attacks."""
from pathlib import Path
import runpy
HERE=Path(__file__).resolve().parent

def apply(edit,once):
 def source(_):
  reach=(HERE/'reach-host-methods.rs').read_text(encoding='utf8')
  start=reach.index('        let mut candidates = Vec::new();')
  end=reach.index('        super::client_reach_pick(candidates,',start)
  candidates='\nimpl Game<\'_>{\n    fn kill_aura_candidates(&self)->Vec<super::ClientReachCandidate>{\n        let root=self.riding.map(|r|self.client_reach_root_vehicle(r));\n'+reach[start:end]+'        self.kill_aura_extra_candidates(&mut candidates);\n        candidates.retain(|c|!self.mobs.iter().any(|m|m.id as i32==c.id && horizon_client::command::args::normalize(&m.kind.id)=="minecraft:ender_dragon"));\n        candidates\n    }\n}\n'
  fragments=['kill-aura-extra-candidates.rs','kill-aura-extra-attack.rs','kill-aura-fight-goal.rs','kill-aura-local-forecast.rs']
  return (HERE/'kill-aura-native.rs').read_text(encoding='utf8')+candidates+''.join((HERE/name).read_text(encoding='utf8').replace('//!','//')for name in fragments)
 edit('crates/horizon-client/src/game/kill_aura.rs',source)
 def game(s):
  s=once(s,'mod aimbot;','mod aimbot;\nmod kill_aura;')
  s=once(s,'    client_projectile_puncher_hide_implicit: bool,','    client_projectile_puncher_hide_implicit: bool,\n    client_kill_aura_keep_sprint:bool,\n    client_kill_aura_attack_hook:Option<u32>,\n    client_kill_aura_pick:Option<ClientReachPick>,\n    client_kill_aura_ignore_entities:bool,')
  return once(s,'            client_projectile_puncher_hide_implicit: false,','            client_projectile_puncher_hide_implicit: false,\n            client_kill_aura_keep_sprint:false,\n            client_kill_aura_attack_hook:None,\n            client_kill_aura_pick:None,\n            client_kill_aura_ignore_entities:false,')
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(s):
  anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  s=once(s,anchor,(HERE/'kill-aura-host.rs').read_text(encoding='utf8')+'\n'+anchor)
  s=once(s,'    pub(super) fn client_reach_target(&self) -> Option<super::ClientReachPick> {','    pub(super) fn client_reach_target(&self) -> Option<super::ClientReachPick> {\n        if let Some(hit)=self.client_kill_aura_pick{return Some(hit);}\n        if self.client_kill_aura_ignore_entities{return None;}')
  s=once(s,'    pub(super) fn client_weapon_before_attack(&mut self, target: u32, living: bool) -> bool {','    pub(super) fn client_weapon_before_attack(&mut self, target: u32, living: bool) -> bool {\n        if self.client_kill_aura_attack_hook==Some(target){return true;}')
  if 'pub(super) fn client_reach_root_vehicle('not in s:s=once(s,'    fn client_reach_root_vehicle(','    pub(super) fn client_reach_root_vehicle(')
  if 'pub(super) fn client_weapon_would_block_hit('not in s:s=once(s,'    fn client_weapon_would_block_hit(','    pub(super) fn client_weapon_would_block_hit(')
  return s if 'pub(super) fn client_crosshair_entity('in s else once(s,'fn client_crosshair_entity(','pub(super) fn client_crosshair_entity(')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 def hand(s):
  if 'pub(super) fn stack_attack_range('not in s:s=once(s,'    fn stack_attack_range(','    pub(super) fn stack_attack_range(')
  s=once(s,'    pub(super) fn held_range_allows(&self, distance: f64) -> bool {','    pub(super) fn held_range_allows(&self, distance: f64) -> bool {\n        if self.client_kill_aura_pick.is_some(){return true;}')
  return s+'''\nimpl Game<'_>{
    pub(super) fn client_kill_aura_start_destroy(&mut self,pos:[i32;3],face:u8){
        let mut destroying=std::mem::take(&mut self.clicks.destroying);
        let mut world=GameHands{game:self,error:None};
        destroying.start(&mut world,pos,face);
        if let Some(error)=world.error.take(){eprintln!("[LiquidBounce] KillAura mining: {error}");}
        self.clicks.destroying=destroying;
    }
}\n'''
 edit('crates/horizon-client/src/game/hand.rs',hand)
 def remote(s):
  for name in ['remote_player_attack','remote_use_entity','remote_use_item_on','remote_mining']:
   if 'pub(super) fn '+name+'('not in s:s=once(s,'    fn '+name+'(','    pub(super) fn '+name+'(')
  return s
 edit('crates/horizon-client/src/game/remote.rs',remote)
 edit('crates/horizon-client/src/game/enchant_effects.rs',lambda s:once(s,'    pub(super) fn stop_sprint_after_extra_knockback(&mut self) {','    pub(super) fn stop_sprint_after_extra_knockback(&mut self) {\n        if self.client_kill_aura_keep_sprint{return;}'))
 def renderer(s):
  s=once(s,'        if ids.is_empty() {\n            return;\n        }\n','')
  return once(s,'"phase":"frame","partial":partial,"entities":entities','"phase":"frame","playerPosition":self.player.as_ref().map(|p|{let old=self.previous_position;[old.x+(p.position.x-old.x)*partial,old.y+(p.position.y-old.y)*partial,old.z+(p.position.z-old.z)*partial]}),"partial":partial,"entities":entities')
 edit('crates/horizon-client/src/game/target_renderer.rs',renderer)
 runpy.run_path(str(HERE/'kill-aura-extra-patch.py'))['apply'](edit,once)
 return {'hook_names':['client_kill_aura_state'],'helper_names':['client_kill_aura_query']}
