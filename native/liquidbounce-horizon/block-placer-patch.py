"""Shared non-support BlockPlacer services and exact post-send movement phase."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 for target,source in [('client_liquid_filler.rs','liquid-filler-native.rs'),('client_block_placer.rs','block-placer-native.rs'),('block_placer_crystal.rs','block-placer-crystal-native.rs')]:
  edit('crates/horizon-client/src/game/'+target,lambda _,source=source:(HERE/source).read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game/liquid_filler_core.rs',lambda _:(HERE.parent/'liquidbounce-behaviors/src/liquid_filler.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game/liquid_filler_random.rs',lambda _:(HERE.parent/'liquidbounce-behaviors/src/random.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod client_extinguish;','mod client_extinguish;\nmod client_liquid_filler;\nmod liquid_filler_core;\nmod liquid_filler_random;\nmod client_block_placer;\nmod block_placer_crystal;'))
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,(HERE/'block-placer-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 def remote(s):
  # NoSlow wraps sendChanges. FIRST_PRIORITY PostRotationExecutor runs
  # immediately after the ordinary input/sprint/movement packets, before
  # NoSlow's POST listener or any end-of-player-tick listener.
  s=once(s,'        self.no_slow_vanilla_send_state(input);','        self.no_slow_vanilla_send_state(input);\n        self.native_client_block_placer_after_move();')
  return s
 edit('crates/horizon-client/src/game/remote.rs',remote)
 def net(s):
  s=once(s,'    NotifierPacket(Box<play::Clientbound>),','    NotifierPacket(Box<play::Clientbound>),\n    BlockPlacerRemoved(Vec<i32>),')
  return once(s,'        if cancelled {continue;}','''        // Ordinary PacketEvent observation, including cancelled RemoveEntities.
        if let play::Clientbound::RemoveEntities(p)=&packet{let _=send.send(Event::BlockPlacerRemoved(p.ids.clone()));}
        if cancelled {continue;}''')
 edit('crates/horizon-client/src/net.rs',net)
 edit('crates/horizon-client/src/game/remote_poll.rs',lambda s:once(s,'                Event::NotifierPacket(packet)=>','                Event::BlockPlacerRemoved(ids)=>{for id in ids{self.native_client_block_placer_removed(id);}},\n                Event::NotifierPacket(packet)=>'))
 def attack(s):
  anchor='    fn kill_aura_attack(&mut self, id: u32, keep: bool, critical: bool) -> bool {'
  s=once(s,anchor,anchor+'\n        self.kill_aura_attack_with_swing(id,keep,critical,0)\n    }\n    pub(super) fn kill_aura_attack_with_swing(&mut self,id:u32,keep:bool,critical:bool,swing:u8)->bool{')
  start=s.index('    pub(super) fn kill_aura_attack_with_swing(');end=s.index('    pub(super) fn kill_aura_query(',start)
  body=s[start:end]
  if 'self.client_projectile_puncher_swing(0);'in body:body=body.replace('self.client_projectile_puncher_swing(0);','self.client_projectile_puncher_swing(swing);')
  return s[:start]+body+s[end:]
 edit('crates/horizon-client/src/game/kill_aura.rs',attack)
 return {'hook_names':['client_block_placer_success','client_block_placer_after_move','client_block_placer_removed'],'helper_names':['client_block_placer_query','client_liquid_filler_query']}
