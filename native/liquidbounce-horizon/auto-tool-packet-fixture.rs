#[path="__PACKET_PATH__"]pub mod inventory_packets;
#[cfg(test)]mod packet_tests {
    use super::inventory_packets::{InventoryPackets,Observation};
    use super::packets::{Delivery,PacketContext,PacketListener,TransferOrigin};
    use horizon_protocol::packets::play::*;
    fn context(session:u64,delivery:Delivery)->PacketContext{PacketContext{session,delivery,origin:TransferOrigin::Outgoing}}
    fn click(container:i32)->Serverbound{Serverbound::ContainerClick(ContainerClick{container,state:0,slot:9,button:0,input:2,changed:vec![],carried:None})}
    #[test]fn accepted_original_packet_observations_preserve_order_and_filter_sessions(){
        let queue=InventoryPackets::default();let c=context(7,Delivery::Original);
        queue.outgoing_accepted(context(6,Delivery::Original),&Serverbound::ContainerClose(ContainerClose{container:0}));
        queue.outgoing_accepted(c,&Serverbound::UseItemOn(UseItemOn{hand:0,pos:[1,2,3],face:1,cursor:[0.5;3],inside:false,world_border:false,sequence:1}));
        queue.outgoing_accepted(c,&Serverbound::Interact(Interact{id:9,hand:0,location:[0.;3],secondary:false}));
        queue.outgoing_accepted(c,&click(1));queue.outgoing_accepted(c,&click(0));
        queue.outgoing_accepted(c,&Serverbound::ContainerClose(ContainerClose{container:1}));
        assert_eq!(queue.drain(7),vec![Observation::UseOn([1,2,3]),Observation::MayOpen,Observation::Click,Observation::Close]);
        assert!(queue.drain(6).is_empty());assert!(queue.drain(7).is_empty());
    }
    #[test]fn silent_delivery_and_world_change_do_not_leave_observations(){
        let queue=InventoryPackets::default();queue.outgoing_accepted(context(7,Delivery::Silent),&click(0));
        assert!(queue.drain(7).is_empty());queue.outgoing_accepted(context(7,Delivery::Original),&click(0));
        queue.world_change();assert!(queue.drain(7).is_empty());
    }
}
