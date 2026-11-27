"""AntiBot packet observations, real profile state and entity/armor snapshots."""
from pathlib import Path
HERE=Path(__file__).resolve().parent

def apply(edit,once):
 edit('crates/horizon-client/src/game/anti_bot_native.rs',lambda _:(HERE/'anti-bot-native.rs').read_text(encoding='utf8'))
 def game(s):
  s=once(s,'mod offhand_native;','mod offhand_native;\nmod anti_bot_native;')
  s=once(s,'    client_chest_stealer_hide_swing: bool,','    client_chest_stealer_hide_swing: bool,\n    client_anti_bot:anti_bot_native::Native,')
  return once(s,'            client_chest_stealer_hide_swing: false,','            client_chest_stealer_hide_swing: false,\n            client_anti_bot:Default::default(),')
 edit('crates/horizon-client/src/game.rs',game)
 def remote(s):
  a='    /// The profile name of another player the server sent, by entity id.\n'
  s=once(s,a,(HERE/'anti-bot-remote.rs').read_text(encoding='utf8')+'\n'+a)
  s=once(s,'            C::PlayerInfoUpdate(p) => {','            C::PlayerInfoUpdate(p) => {\n                self.client_anti_bot_keep_info(&p);')
  s=once(s,'            C::PlayerInfoRemove(p) => {','            C::PlayerInfoRemove(p) => {\n                for id in &p.uuids{self.client_anti_bot.profiles.remove(id);}')
  s=once(s,'            C::AddEntity(p) => {','            C::AddEntity(p) => {\n                self.client_anti_bot.entities.insert(p.id,self.client_anti_bot.profiles.get(&p.uuid).is_none_or(|p|p.0));\n                self.client_anti_bot.scales.remove(&p.id);')
  s=once(s,'            C::RemoveEntities(p) => {','            C::RemoveEntities(p) => {\n                for id in &p.ids{self.client_anti_bot.entities.remove(id);self.client_anti_bot.scales.remove(id);}')
  s=once(s,'            C::Login(p) => {','            C::Login(p) => {\n                self.client_anti_bot=Default::default();')
  return once(s,'            C::Respawn(p) => {','            C::Respawn(p) => {\n                self.client_anti_bot.entities.clear();self.client_anti_bot.scales.clear();')
 edit('crates/horizon-client/src/game/remote.rs',remote)
 edit('crates/horizon-client/src/game/remote_attributes.rs',lambda s:once(s,'            m.extra.model_scale = scale as f32;','            self.client_anti_bot.scales.insert(id,scale);\n            m.extra.model_scale = scale as f32;'))
 def hooks(s):
  a='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  s=once(s,a,(HERE/'anti-bot-host.rs').read_text(encoding='utf8')+'\n'+a)
  return once(s,'"within_border":within_border,','"within_border":within_border,"antiBot":self.client_anti_bot_query(entity),')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 def net(s):
  s=once(s,'    NotifierPacket(Box<play::Clientbound>),','    NotifierPacket(Box<play::Clientbound>),\n    AntiBotPacket(Box<play::Clientbound>,i64),')
  return once(s,'        if cancelled {continue;}','''        if matches!(&packet,play::Clientbound::PlayerInfoUpdate(_)|play::Clientbound::PlayerInfoRemove(_)|play::Clientbound::MoveEntityPos(_)|play::Clientbound::MoveEntityPosRot(_)|play::Clientbound::UpdateAttributes(_)|play::Clientbound::SetEntityData(_)|play::Clientbound::Animate(_)|play::Clientbound::SwingAnimation(_)|play::Clientbound::RemoveEntities(_)){
            let time=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0,|d|d.as_millis()as i64);
            let _=send.send(Event::AntiBotPacket(Box::new(packet.clone()),time));
        }
        if cancelled {continue;}''')
 edit('crates/horizon-client/src/net.rs',net)
 edit('crates/horizon-client/src/game/remote_poll.rs',lambda s:once(s,'                Event::NotifierPacket(packet)=>','                Event::AntiBotPacket(packet,time)=>self.client_anti_bot_packet(&packet,time),\n                Event::NotifierPacket(packet)=>'))
 return {'hook_names':['client_anti_bot'],'helper_names':['client_anti_bot_snapshot']}
