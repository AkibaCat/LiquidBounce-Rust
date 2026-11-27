    pub(crate) fn is_lagging(&self)->bool{!self.state.lock().expect("BlinkManager").blink.queue.is_empty()}
    pub(crate) fn positions(&self)->Vec<[f64;3]>{
        self.state.lock().expect("BlinkManager").blink.queue.iter().filter_map(|s|Self::packet_position(&s.packet)).collect()
    }
    fn packet_position(p:&Packet)->Option<[f64;3]>{match p{
        Packet::Outgoing(S::MovePlayerPos(p))=>Some([p.x,p.y,p.z]),
        Packet::Outgoing(S::MovePlayerPosRot(p))=>Some([p.x,p.y,p.z]),_=>None,
    }}
    pub(crate) fn flush_origins(&self,mask:u32){
        let mut s=self.state.lock().expect("BlinkManager");
        for outgoing in [false,true]{if mask&(if outgoing{2}else{1})!=0{let ready=Self::take_origin(&mut s,outgoing);s.ready.extend(ready);}}
    }
    fn operation_queue(s:&mut State)->liquidbounce_behaviors::blink::Queue<model::Queued<Packet>>{
        let mut q=liquidbounce_behaviors::blink::Queue::default();
        for item in s.blink.queue.drain(..){let position=Self::packet_position(&item.packet);let outgoing=matches!(item.packet,Packet::Outgoing(_));let movement=matches!(&item.packet,Packet::Outgoing(p)if movement(p));
            q.0.push_back(liquidbounce_behaviors::blink::Snapshot{outgoing,millis:item.millis,position,movement,packet:item});}
        q
    }
    pub(crate) fn flush_count(&self,count:i32){
        let mut s=self.state.lock().expect("BlinkManager");let mut q=Self::operation_queue(&mut s);
        s.ready.extend(q.flush_count(count).into_iter().map(|s|s.packet));s.blink.queue=q.0.into_iter().map(|s|s.packet).collect();
    }
    pub(crate) fn cancel_shared(&self)->Option<[f64;3]>{
        let mut s=self.state.lock().expect("BlinkManager");let mut q=Self::operation_queue(&mut s);let(at,ready)=q.cancel();s.ready.extend(ready.into_iter().map(|s|s.packet));at
    }
    pub(crate) fn flush_outgoing_event(&self,event:&mut PacketEvent<'_,S>){
        let mut s=self.state.lock().expect("BlinkManager");let mut keep=Vec::new();let mut ready=Vec::new();
        for p in s.ready.drain(..){if matches!(p.packet,Packet::Outgoing(_)){ready.push(p);}else{keep.push(p);}}
        s.ready=keep;drop(s);
        let mut pending=Vec::new();
        for p in ready {if p.session==event.context.session {if let Packet::Outgoing(packet)=&p.packet {if !event.send_silently(packet.clone()){pending.push(p);}}}}
        self.state.lock().expect("BlinkManager").ready.extend(pending);
    }
