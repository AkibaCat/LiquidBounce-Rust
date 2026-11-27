// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{runtime::LiquidBounceHooks,coroutine_ticker::WaitOwner};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::spider::{Mode,Step,Velocity};
impl LiquidBounceHooks{
    fn spider_step(&self,host:&mut dyn Host,step:Step){
        if let Some(v)=step.velocity{match v{Velocity::Vertical(y)=>host.client_spider_velocity([0.0,y,0.0],false),Velocity::All(v)=>host.client_spider_velocity(v,true)}}
        if let Some(id)=step.wait{self.state().behaviors.borrow().coroutine_ticker.register(WaitOwner::Spider,id);}
    }
    pub(crate) fn spider_tick(&self,host:&mut dyn Host){
        let context=host.client_spider_context();
        let step={let state=self.state();let mut modules=state.behaviors.borrow_mut();let running=modules.query_running(&state.registry,"Spider");context.map_or(Step::default(),|c|modules.spider.game_tick(running,c[0],c[1]))};
        host.client_spider_player_sync();self.spider_step(host,step);
    }
    pub(crate) fn spider_callback(&self,host:&mut dyn Host,in_game:bool,id:u64)->bool{
        let (keep,step)={let state=self.state();let mut modules=state.behaviors.borrow_mut();let running=in_game&&modules.query_running(&state.registry,"Spider");modules.spider.callback(id,running)};
        self.spider_step(host,step);keep
    }
    pub(crate) fn spider_jump(&self,horizontal_collision:bool,motion:&mut f32){let state=self.state();let modules=state.behaviors.borrow();modules.spider.jump(modules.query_running(&state.registry,"Spider"),horizontal_collision,motion);}
    pub(crate) fn spider_shape_active(&self)->bool{let state=self.state();let modules=state.behaviors.borrow();modules.query_running(&state.registry,"Spider")&&modules.spider.settings.mode==Mode::Polar}
}
