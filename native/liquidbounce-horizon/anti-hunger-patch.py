#!/usr/bin/env python3
"""AntiHunger's safe split-player synchronous ordinary packet send scope."""
from pathlib import Path
import runpy
HERE=Path(__file__).resolve().parent
method=runpy.run_path(str(HERE/'terrain-speed-order-patch.py'))['method']
HOOK_NAMES=[]
HELPER_NAMES=['client_anti_hunger_swimming']
def apply(edit,once):
    edit('crates/horizon-client/src/client_lib.rs',lambda t:once(t,'pub mod packet_bridge;\n','pub mod packet_bridge;\npub mod anti_hunger_bridge;\n'))
    edit('crates/horizon-client/src/anti_hunger_bridge.rs',lambda _:(HERE/'anti-hunger-bridge.rs').read_text(encoding='utf-8'))
    def bridge(t):
        t=once(t,'    sender: Option<&\'a dyn Fn(P)>,\n','    sender: Option<&\'a dyn Fn(P)>,\n    pub client_player:Option<&\'a dyn crate::anti_hunger_bridge::ClientPacketPlayer>,\n')
        t=once(t,'            sender: None,\n','            sender: None,\n            client_player:None,\n')
        t=once(t,'outgoing_impl(session, packet, silent, None)','outgoing_impl(session, packet, silent, None, None)')
        t=once(t,'outgoing_impl(session, packet, silent, Some(sender))','outgoing_impl(session, packet, silent, Some(sender), None)')
        anchor='fn outgoing_impl(session: u64, packet: &mut Serverbound, silent: bool, sender: Option<&dyn Fn(Serverbound)>) -> bool {\n'
        t=once(t,anchor,'''pub fn outgoing_with_client_player(session:u64,packet:&mut Serverbound,silent:bool,sender:&dyn Fn(Serverbound),player:Option<&dyn crate::anti_hunger_bridge::ClientPacketPlayer>)->bool{
    outgoing_impl(session,packet,silent,Some(sender),player)
}

fn outgoing_impl(session: u64, packet: &mut Serverbound, silent: bool, sender: Option<&dyn Fn(Serverbound)>,player:Option<&dyn crate::anti_hunger_bridge::ClientPacketPlayer>) -> bool {
''')
        t=once(t,'    event.sender = sender;\n','    event.sender = sender;\n    event.client_player=player;\n')
        return t
    edit('crates/horizon-client/src/packet_bridge.rs',bridge)
    def network(t):
        anchor='    fn send_with_path(&self, mut packet: play::Serverbound, silent: bool) {\n'
        t=once(t,anchor,'''    fn send_with_path(&self,packet:play::Serverbound,silent:bool){self.send_with_client_player_path(packet,silent,None);}
    pub fn send_with_client_player(&self,packet:play::Serverbound,player:&dyn crate::anti_hunger_bridge::ClientPacketPlayer){self.send_with_client_player_path(packet,false,Some(player));}
    fn send_with_client_player_path(&self,mut packet:play::Serverbound,silent:bool,player:Option<&dyn crate::anti_hunger_bridge::ClientPacketPlayer>){
''')
        return once(t,'crate::packet_bridge::outgoing_with_sender(self.serial, &mut packet, silent, &|nested| self.send(nested))','crate::packet_bridge::outgoing_with_client_player(self.serial, &mut packet, silent, &|nested| self.send_with_client_player_path(nested,false,player),player)')
    edit('crates/horizon-client/src/net.rs',network)
    def remote(t):
        original=method(t,'    pub(super) fn send(&self, packet: S) {')
        replacement=original.replace('fn send(&self','fn send(&mut self',1)
        replacement=once(replacement,'        if let Some(r) = &self.remote {','        if self.remote.is_some() {')
        replacement=once(replacement,'            r.session.send(packet);','            self.native_send_with_anti_hunger(packet);')
        t=once(t,original,replacement)
        t=once(t,'    fn send_action(&self, action: i32) {','    fn send_action(&mut self, action: i32) {')
        # Every Game-origin send, including the carried-slot predecessor, needs
        # the same player scope for reentrant ordinary listeners.
        carried=method(t,'    pub(super) fn remote_send_carried(&self) {')
        new=carried.replace('fn remote_send_carried(&self','fn remote_send_carried(&mut self',1)
        new=once(new,'            r.session.send(','            self.native_send_with_anti_hunger(')
        t=once(t,carried,new)
        return t+'\n'+(HERE/'anti-hunger-native.rs').read_text(encoding='utf-8')
    edit('crates/horizon-client/src/game/remote.rs',remote)
    edit('crates/horizon-client/src/game/spear.rs',lambda t:once(t,'                r.session.send(pkt);','                self.native_send_with_anti_hunger(pkt);'))
    # These send wrappers now borrow the same real player body synchronously.
    edit('crates/horizon-client/src/game/perf_link.rs',lambda t:once(t,'    fn send_perf(&self, request: &Request) {','    fn send_perf(&mut self, request: &Request) {'))
    edit('crates/horizon-client/src/game/recipe_book.rs',lambda t:once(t,'    fn send_book_settings(&self, book_type: BookType) {','    fn send_book_settings(&mut self, book_type: BookType) {'))
    edit('crates/horizon-client/src/game/remote_stations.rs',lambda t:once(t,'    pub(super) fn remote_station_button(&self, button: i32) {','    pub(super) fn remote_station_button(&mut self, button: i32) {'))
    def native(t):
        t=once(t,'    pub(super) fn native_reject_slot_packet(&self, packet: &ClientboundPlayPacket) -> bool {','    pub(super) fn native_reject_slot_packet(&mut self, packet: &ClientboundPlayPacket) -> bool {')
        t=once(t,'    pub(super) fn native_client_ensure_carried(&self) {','    pub(super) fn native_client_ensure_carried(&mut self) {')
        anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
        helper='''    pub(super) fn native_client_anti_hunger_swimming(&self)->Option<bool>{
        if !self.native_window()||self.lan_acting_uuid.is_some()||self.lan_acting.is_some(){return None;}
        self.player.as_ref().filter(|p|p.client_side&&p.native_client_queries_allowed()).map(|p|p.swimming)
    }
'''
        return once(t,anchor,helper+anchor)
    edit('crates/horizon-client/src/game/native_hooks.rs',native)
    return dict(hook_names=HOOK_NAMES,helper_names=HELPER_NAMES)
