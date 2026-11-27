    pub(super) fn native_client_auto_chat_game_schedule(&self,deadline:Option<i64>){
        self.client_auto_chat_game_wake_at.set(deadline);
    }
    pub(super) fn native_client_auto_chat_game_wake(&mut self){
        if self.client_auto_tool_player(){mod_api::dispatch_client_auto_chat_game_wake(self,mod_api::mods());}
    }
    pub(super) fn client_auto_chat_game_now()->i64{
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH){Ok(t)=>t.as_millis().min(i64::MAX as u128)as i64,Err(t)=>-(t.duration().as_millis().min(i64::MAX as u128)as i64)}
    }
    pub(super) fn client_auto_chat_game_poll_wake(&mut self){
        if self.client_auto_chat_game_wake_at.get().is_some_and(|at|at<=Self::client_auto_chat_game_now()){
            self.client_auto_chat_game_wake_at.set(None);self.native_client_auto_chat_game_wake();
        }
    }
    pub(super) fn client_auto_chat_game_control_flow(&self,flow:winit::event_loop::ControlFlow)->winit::event_loop::ControlFlow{
        use winit::event_loop::ControlFlow;
        let Some(at)=self.client_auto_chat_game_wake_at.get()else{return flow;};
        let until=std::time::Instant::now()+std::time::Duration::from_millis(at.saturating_sub(Self::client_auto_chat_game_now()).max(0)as u64);
        match flow{ControlFlow::Poll=>flow,ControlFlow::Wait=>ControlFlow::WaitUntil(until),ControlFlow::WaitUntil(old)=>ControlFlow::WaitUntil(old.min(until))}
    }
