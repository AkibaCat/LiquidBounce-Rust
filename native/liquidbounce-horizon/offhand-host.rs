    pub(super) fn native_client_offhand_context(&mut self,policy:&str)->Option<String>{super::offhand_native::snapshot(self,policy)}
    /// Smart sends ordinary packets in the source order, without changing local selection.
    pub(super) fn native_client_offhand_smart(&mut self,slot:i32,selected:i32){
        use horizon_protocol::packets::play::{self,Serverbound as S};
        if !self.client_auto_tool_player()||!(0..9).contains(&slot){return;}
        if slot!=selected{self.send(S::SetCarriedItem(play::SetCarriedItem{slot:slot as i16}));}
        self.send(S::PlayerAction(play::PlayerAction{action:7,pos:[0;3],direction:0,sequence:0}));
        if slot!=selected{self.send(S::SetCarriedItem(play::SetCarriedItem{slot:selected as i16}));}
    }
