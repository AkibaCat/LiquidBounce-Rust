from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 a='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,a,(HERE/'notifier-host.rs').read_text(encoding='utf8')+'\n'+a))
 edit('crates/horizon-client/src/game/remote.rs',lambda t:once(t,'    /// The profile name of another player the server sent, by entity id.\n',(HERE/'notifier-remote.rs').read_text(encoding='utf8')+'\n    /// The profile name of another player the server sent, by entity id.\n'))
 def net(t):
  t=once(t,'    MurderMysteryPacket(Box<play::Clientbound>),','    MurderMysteryPacket(Box<play::Clientbound>),\n    NotifierPacket(Box<play::Clientbound>),')
  return once(t,'        if cancelled {continue;}','''        if matches!(&packet,play::Clientbound::PlayerInfoUpdate(_)|play::Clientbound::PlayerInfoRemove(_)|play::Clientbound::EntityEvent(_)|play::Clientbound::RemoveEntities(_)){
            let _=send.send(Event::NotifierPacket(Box::new(packet.clone())));
        }
        if cancelled {continue;}''')
 edit('crates/horizon-client/src/net.rs',net)
 edit('crates/horizon-client/src/game/remote_poll.rs',lambda t:once(t,'                Event::MurderMysteryPacket(packet)=>','                Event::NotifierPacket(packet)=>self.client_notifier_packet(&packet),\n                Event::MurderMysteryPacket(packet)=>'))
 return {'hook_names':['client_notifier'],'helper_names':['client_notifier_snapshot']}
