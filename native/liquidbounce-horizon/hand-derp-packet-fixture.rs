use crate::{hand_derp_bridge, hand_derp_packets::HandDerpService, packets::*};
use horizon_protocol::{connection::{Connection,ConnWriter,Pipe}, packets::{configuration::ClientInformation,play,Empty}, entity_data::Value};
use std::sync::{Arc,Mutex,atomic::{AtomicBool,Ordering}};
struct Session { writer:Arc<Mutex<Option<ConnWriter>>>,configuring:Arc<AtomicBool>,serial:u64,debug:bool }
impl Session { /* SESSION_METHODS */ }
#[test]
fn nested_hand_information_is_ordinary_before_punch_and_after_prior_cancel() {
    use liquidbounce_behaviors::hand_derp::{MainHand,Mode};
    struct Observer(Arc<Mutex<Vec<&'static str>>>);
    impl PacketListener for Observer {
        fn outgoing(&self,event:&mut PacketEvent<'_,play::Serverbound>) {
            let tag=match event.packet { play::Serverbound::Punch(_)=>"punch",play::Serverbound::ClientInformation(_)=>"info",_=>"other" };
            self.0.lock().unwrap().push(tag);
        }
    }
    struct CancelPunch;
    impl PacketListener for CancelPunch { fn outgoing(&self,event:&mut PacketEvent<'_,play::Serverbound>) { if matches!(event.packet,play::Serverbound::Punch(_)) { event.cancel(); } } }
    let mut actual=ClientInformation::new(37);actual.language="zh_tw".into();actual.chat_visibility=2;actual.chat_colors=false;actual.model_customisation=0x45;actual.main_hand=0;actual.text_filtering=true;actual.allows_listing=false;actual.particle_status=2;
    hand_derp_bridge::publish(actual.clone());
    let service=Arc::new(HandDerpService::default());service.initialize();service.set_running(true);
    service.behavior.lock().unwrap().set_mode(Mode::Swing);
    let seen=Arc::new(Mutex::new(Vec::new()));let pipeline=Arc::new(PacketPipeline::default());
    pipeline.add_listener(1,service.clone());pipeline.add_listener(0,Arc::new(Observer(seen.clone())));
    let previous=crate::packet_bridge::set_interceptor(Some(pipeline.clone()));
    let (client,server)=Pipe::pair();let (_,writer)=Connection::local(client).split();let mut server=Connection::local(server);
    let session=Session { writer:Arc::new(Mutex::new(Some(writer))),configuring:Arc::new(AtomicBool::new(false)),serial:19,debug:false };
    let next=|server:&mut Connection| { let (id,bytes)=server.recv().unwrap();play::Serverbound::decode(id,&bytes).unwrap() };
    session.send(play::Serverbound::Punch(Empty));
    let mut expected=actual.clone();expected.main_hand=1;
    assert_eq!(next(&mut server),play::Serverbound::ClientInformation(expected));assert_eq!(next(&mut server),play::Serverbound::Punch(Empty));
    assert_eq!(*seen.lock().unwrap(),["info","punch"]);
    assert_eq!(service.accepted_packets()["clientInformation"],1);
    assert_eq!(service.accepted_packets()["punches"],1);
    assert_eq!(service.accepted_packets()["lastClientInformation"]["mainHand"],1);
    assert_eq!(service.accepted_packets()["lastClientInformation"]["modelCustomisation"],0x45);
    let cancel_id=pipeline.add_listener(2,Arc::new(CancelPunch));
    session.send(play::Serverbound::Punch(Empty));session.send(play::Serverbound::ClientTickEnd(Empty));
    assert_eq!(next(&mut server),play::Serverbound::ClientInformation(actual.clone()));assert_eq!(next(&mut server),play::Serverbound::ClientTickEnd(Empty));
    assert_eq!(service.behavior.lock().unwrap().current(),Some(MainHand::Left));
    assert_eq!(service.accepted_packets()["clientInformation"],2);
    assert_eq!(service.accepted_packets()["punches"],1);
    pipeline.remove_listener(cancel_id);pipeline.no_swing.set_running(true);
    session.send(play::Serverbound::Punch(Empty));session.send(play::Serverbound::ClientTickEnd(Empty));
    assert_eq!(next(&mut server),play::Serverbound::ClientTickEnd(Empty));
    assert_eq!(service.behavior.lock().unwrap().current(),Some(MainHand::Left));
    session.send_silently(play::Serverbound::Punch(Empty));assert_eq!(next(&mut server),play::Serverbound::Punch(Empty));
    assert_eq!(service.accepted_packets()["clientInformation"],2);
    assert_eq!(service.accepted_packets()["punches"],1);
    assert_eq!(service.behavior.lock().unwrap().current(),Some(MainHand::Left));
    // The real option remains the snapshot's left arm; only outgoing packets changed.
    assert_eq!(hand_derp_bridge::snapshot().unwrap().information,actual);
    actual.main_hand=1;actual.language="fr_fr".into();hand_derp_bridge::publish(actual.clone());
    assert_eq!(hand_derp_bridge::snapshot().unwrap().original_hand,0);
    service.behavior.lock().unwrap().settings.silent=true;
    for (entity,ids,cancelled) in [(999,vec![17,0],false),(123,vec![1,17],true),(-8,vec![1],true)] {
        let mut packet=play::Clientbound::SetEntityData(play::SetEntityData { id:entity,values:ids.into_iter().map(|id|(id,0,Value::Byte(0))).collect() });
        assert_eq!(crate::packet_bridge::incoming(19,&mut packet),cancelled);
    }
    service.set_running(false);
    let mut packet=play::Clientbound::SetEntityData(play::SetEntityData { id:19,values:vec![(1,0,Value::Byte(0))] });
    assert!(!crate::packet_bridge::incoming(19,&mut packet));
    crate::packet_bridge::set_interceptor(previous);
}
