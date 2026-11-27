"""Blink real local dummy entity identity and native controls."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 def game(s):
  s=once(s,'    player_model: Option<Mob>,','    player_model: Option<Mob>,\n    client_blink_dummy:Option<u32>,')
  return once(s,'            player_model: None,','            player_model: None,\n            client_blink_dummy:None,')
 def game_with_cleanup(s):
  s=game(s)
  return once(s,'    pub(super) fn clear_entities(&mut self) {','    pub(super) fn clear_entities(&mut self) {\n        self.native_client_blink_control("remove",&[]);')
 edit('crates/horizon-client/src/game.rs',game_with_cleanup)
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'blink-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
 def remote(s):
  s=once(s,'        let stick = self.input_move();','        self.native_client_blink_movement();\n        let stick = self.input_move();')
  s=s.replace('        self.mobs.clear();','        self.native_client_blink_control("remove",&[]);\n        self.mobs.clear();')
  return once(s,'    fn remote_use_entity(&mut self, hand: Hand) -> bool {','''    pub(super) fn client_blink_add_identity(&mut self,id:i32){
        if let Some(r)=self.remote.as_mut(){r.level_players.insert(id,r.profile);r.level_player_names.insert(id,r.profile_name.clone());}
    }
    pub(super) fn client_blink_remove_identity(&mut self,id:i32){
        if let Some(r)=self.remote.as_mut(){r.level_players.remove(&id);r.level_player_names.remove(&id);r.level_player_textures.remove(&id);}
    }
    fn remote_use_entity(&mut self, hand: Hand) -> bool {''')
 edit('crates/horizon-client/src/game/remote.rs',remote)
 def bridge(s):
  s=once(s,"    silent_sender: Option<&'a dyn Fn(P)>,","    silent_sender: Option<&'a dyn Fn(P)>,\n    incoming_replayer:Option<&'a dyn Fn(P)>,")
  s=once(s,'            silent_sender: None,','            silent_sender: None,\n            incoming_replayer:None,')
  s=once(s,'    pub fn cancel(&mut self) {','    pub fn replay_before(&self,packet:P)->bool{if let Some(replay)=self.incoming_replayer{replay(packet);true}else{false}}\n    pub fn cancel(&mut self) {')
  old='pub fn incoming(session: u64, packet: &mut Clientbound) -> bool {'
  s=once(s,old,'''pub fn incoming(session:u64,packet:&mut Clientbound)->bool{incoming_impl(session,packet,None)}
pub fn incoming_with_replayer(session:u64,packet:&mut Clientbound,replay:&dyn Fn(Clientbound))->bool{incoming_impl(session,packet,Some(replay))}
fn incoming_impl(session:u64,packet:&mut Clientbound,replay:Option<&dyn Fn(Clientbound)>)->bool{''')
  return once(s,'        interceptor.incoming(&mut event);','        event.incoming_replayer=replay;\n        interceptor.incoming(&mut event);')
 edit('crates/horizon-client/src/packet_bridge.rs',bridge)
 def net(s):
  s=once(s,'    Packet(Box<play::Clientbound>),','    Packet(Box<play::Clientbound>),\n    ReplayPacket(Box<play::Clientbound>),')
  return once(s,'crate::packet_bridge::incoming(crate::packet_bridge::reader_session(), &mut packet)','crate::packet_bridge::incoming_with_replayer(crate::packet_bridge::reader_session(), &mut packet,&|packet|{let _=send.send(Event::ReplayPacket(Box::new(packet)));})')
 edit('crates/horizon-client/src/net.rs',net)
 edit('crates/horizon-client/src/game/remote_poll.rs',lambda s:once(s,'                Event::Packet(packet) => match *packet {','''                Event::ReplayPacket(packet)=>{if let Some(session)=self.native_client_session(){if let Err(error)=self.native_replay_client_packet(session,*packet){log::warn!("Blink incoming replay: {error}");}}}
                Event::Packet(packet) => match *packet {'''))
 return {'hook_names':['client_blink_movement'],'helper_names':['client_blink_control','client_blink_dummy']}
