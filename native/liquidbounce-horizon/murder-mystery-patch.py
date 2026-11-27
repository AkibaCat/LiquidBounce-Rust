"""Genuine equipment/sound PacketEvents, maps, profiles, team prefixes and bow boxes."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
    def native(s):
        anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
        s=once(s,anchor,(HERE/'murder-mystery-host.rs').read_text(encoding='utf8')+'\n'+anchor)
        s=once(s,'        mod_api::dispatch_client_tick_pre(self, mod_api::mods());','        self.client_murder_mystery_tick();\n        mod_api::dispatch_client_tick_pre(self, mod_api::mods());')
        return once(s,'"within_border":within_border,','"within_border":within_border,"murderMysteryUuid":self.client_murder_mystery_uuid(entity),')
    edit('crates/horizon-client/src/game/native_hooks.rs',native)
    edit('crates/horizon-client/src/game/remote.rs',lambda s:once(s,'    /// The profile name of another player the server sent, by entity id.\n',(HERE/'murder-mystery-remote.rs').read_text(encoding='utf8')+'\n    /// The profile name of another player the server sent, by entity id.\n'))
    # Preserve the actor's packet order even for a cancelled ordinary PacketEvent.
    # Observe final mutations, filter the four relevant packet classes, and never
    # resubmit a packet to the network or to an incoming-replay handler.
    def net(s):
        s=once(s,'    ReplayPacket(Box<play::Clientbound>),','    ReplayPacket(Box<play::Clientbound>),\n    MurderMysteryPacket(Box<play::Clientbound>),')
        old='        if crate::packet_bridge::incoming_with_replayer(crate::packet_bridge::reader_session(), &mut packet,&|packet|{let _=send.send(Event::ReplayPacket(Box::new(packet)));}) {\n            continue;\n        }'
        new='''        let cancelled=crate::packet_bridge::incoming_with_replayer(crate::packet_bridge::reader_session(), &mut packet,&|packet|{let _=send.send(Event::ReplayPacket(Box::new(packet)));});
        if matches!(&packet,play::Clientbound::SetEquipment(_)|play::Clientbound::Sound(_)|play::Clientbound::Login(_)|play::Clientbound::Respawn(_)) {
            let _=send.send(Event::MurderMysteryPacket(Box::new(packet.clone())));
        }
        if cancelled {continue;}'''
        return once(s,old,new)
    edit('crates/horizon-client/src/net.rs',net)
    edit('crates/horizon-client/src/game/remote_poll.rs',lambda s:once(s,'                Event::ReplayPacket(packet)=>','                Event::MurderMysteryPacket(packet)=>self.client_murder_mystery_packet(&packet),\n                Event::ReplayPacket(packet)=>'))
    edit('crates/horizon-client/src/game/world_primitives.rs',lambda s:once(s,'        let mut primitives = WorldPrimitives::default();','        let mut primitives = WorldPrimitives::default();\n        self.client_murder_mystery_frame(alpha,&mut primitives);'))
    return {'hook_names':['client_murder_mystery'],'helper_names':[]}
