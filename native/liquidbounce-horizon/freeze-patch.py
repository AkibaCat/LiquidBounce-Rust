#!/usr/bin/env python3
"""Freeze frame holds, LocalPlayer.aiStep cancellation and nested silent sends."""
from pathlib import Path
import json
HERE=Path(__file__).resolve().parent

def apply(edit,once):
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'freeze-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
    edit('crates/horizon-client/src/game.rs',lambda s:once(s,'                    self.native_client_timer_scale(&mut timer_delta);','                    self.native_client_freeze_frame();\n                    self.native_client_timer_scale(&mut timer_delta);'))
    edit('crates/horizon-client/src/game/remote.rs',lambda s:once(s,'C::PlayerPosition(p) => {self.native_client_auto_disable_flag();','C::PlayerPosition(p) => {self.native_client_freeze_flag();self.native_client_auto_disable_flag();'))
    edit('crates/horizon-client/src/client_lib.rs',lambda s:once(s,'pub mod spider_bridge;\n','pub mod spider_bridge;\npub mod freeze_bridge;\n'))
    edit('crates/horizon-client/src/freeze_bridge.rs',lambda _:(HERE/'freeze-bridge.rs').read_text(encoding='utf8'))
    def entry(s):
        s=once(s,'    horizon_threads::configure_allocator();','    horizon_client::freeze_bridge::install(dispatch_freeze_movement);\n    horizon_threads::configure_allocator();')
        return s+'\nfn dispatch_freeze_movement(cancelled:&mut bool){crate::mod_api::dispatch_client_freeze_movement(crate::mod_api::mods(),cancelled);}\n'
    edit('crates/horizon-client/src/entry.rs',entry)
    edit('tools/native-hook-fragments/liquidbounce_freeze.json',lambda _:json.dumps({'hooks':[dict(name='client_freeze_movement',target='horizon_client_lib::freeze_bridge::native_client_freeze_movement',side='client',cancellable=False,mod_api={'contract':'listen'})],'helpers':[]},indent=2)+'\n')
    def motion(s):
        anchor='''    fn player_movement_tick_with_fluids_and_ai_tail(
        &mut self, direction: [f32; 2], jumping: bool,
        ai_tail: &mut dyn FnMut(&mut Self) -> Result<()>,
    ) -> Result<()> {'''
        return once(s,anchor,anchor+'\n        if self.player.client_high_jump_owner&&self.player.client_side&&self.player.native_client_queries_allowed()&&crate::freeze_bridge::cancel(){return Ok(());}\n')
    edit('crates/horizon-client/src/motion.rs',motion)
    def bridge(s):
        s=once(s,"    sender: Option<&'a dyn Fn(P)>,\n","    sender: Option<&'a dyn Fn(P)>,\n    silent_sender: Option<&'a dyn Fn(P)>,\n")
        s=once(s,'            sender: None,\n','            sender: None,\n            silent_sender: None,\n')
        s=once(s,'    pub fn cancel(&mut self) {\n','    pub fn send_silently(&self,packet:P)->bool{if let Some(sender)=self.silent_sender{sender(packet);true}else{false}}\n\n    pub fn cancel(&mut self) {\n')
        s=once(s,'    outgoing_impl(session, packet, silent, None, None)','    outgoing_impl(session, packet, silent, None, None, None)')
        s=once(s,'    outgoing_impl(session, packet, silent, Some(sender), None)','    outgoing_impl(session, packet, silent, Some(sender), None, None)')
        s=once(s,'    outgoing_impl(session,packet,silent,Some(sender),player)','    outgoing_impl(session,packet,silent,Some(sender),player,None)')
        anchor="fn outgoing_impl(session: u64, packet: &mut Serverbound, silent: bool, sender: Option<&dyn Fn(Serverbound)>,player:Option<&dyn crate::anti_hunger_bridge::ClientPacketPlayer>) -> bool {\n"
        replacement='''pub fn outgoing_with_senders(session:u64,packet:&mut Serverbound,silent:bool,sender:&dyn Fn(Serverbound),silent_sender:&dyn Fn(Serverbound),player:Option<&dyn crate::anti_hunger_bridge::ClientPacketPlayer>)->bool{
    outgoing_impl(session,packet,silent,Some(sender),player,Some(silent_sender))
}
fn outgoing_impl(session: u64, packet: &mut Serverbound, silent: bool, sender: Option<&dyn Fn(Serverbound)>,player:Option<&dyn crate::anti_hunger_bridge::ClientPacketPlayer>, silent_sender:Option<&dyn Fn(Serverbound)>) -> bool {
'''
        s=once(s,anchor,replacement)
        return once(s,'    event.sender = sender;\n','    event.sender = sender;\n    event.silent_sender = silent_sender;\n')
    edit('crates/horizon-client/src/packet_bridge.rs',bridge)
    edit('crates/horizon-client/src/net.rs',lambda s:once(s,'crate::packet_bridge::outgoing_with_client_player(self.serial, &mut packet, silent, &|nested| self.send_with_client_player_path(nested,false,player),player)','crate::packet_bridge::outgoing_with_senders(self.serial, &mut packet, silent, &|nested| self.send_with_client_player_path(nested,false,player), &|nested| self.send_with_client_player_path(nested,true,player),player)'))
    return {'hook_names':['client_freeze_frame','client_freeze_flag'],'helper_names':['client_freeze_context']}
