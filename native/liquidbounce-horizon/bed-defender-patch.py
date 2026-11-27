"""BedDefender snapshots, real keyboard input and original raw teleport observation."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/bed_defender.rs',lambda _:(HERE/'bed-defender-native.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod client_block_placer;','mod client_block_placer;\nmod bed_defender;'))
 def hooks(s):
  a='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  s=once(s,a,(HERE/'bed-defender-host.rs').read_text(encoding='utf8')+'\n'+a)
  s=once(s,'        mod_api::dispatch_input_key(self, mod_api::mods(), event, modifiers);','        if let winit::keyboard::PhysicalKey::Code(key)=event.physical_key{self.bed_defender_key(key,event.state==winit::event::ElementState::Pressed); }\n        mod_api::dispatch_input_key(self, mod_api::mods(), event, modifiers);')
  return once(s,'        mod_api::dispatch_client_control_key(self, mod_api::mods(), key, pressed, repeat, modifiers);','        self.bed_defender_key(key,pressed);\n        mod_api::dispatch_client_control_key(self, mod_api::mods(), key, pressed, repeat, modifiers);')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 def net(s):
  s=once(s,'    NotifierPacket(Box<play::Clientbound>),','    NotifierPacket(Box<play::Clientbound>),\n    BedDefenderPosition([f64;3]),')
  return once(s,'        if cancelled {continue;}','''        if let play::Clientbound::PlayerPosition(p)=&packet{let _=send.send(Event::BedDefenderPosition(p.position));}
        if cancelled {continue;}''')
 edit('crates/horizon-client/src/net.rs',net)
 edit('crates/horizon-client/src/game/remote_poll.rs',lambda s:once(s,'                Event::NotifierPacket(packet)=>','                Event::BedDefenderPosition(raw)=>self.bed_defender_position(raw),\n                Event::NotifierPacket(packet)=>'))
 return {'hook_names':['client_bed_defender'],'helper_names':['client_bed_defender_query']}
