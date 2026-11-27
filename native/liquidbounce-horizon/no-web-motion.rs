// SPDX-License-Identifier: GPL-3.0-or-later
    fn no_web_inside(&mut self,pos:horizon_core::position::BlockPos)->Result<()>{
        if self.player.client_high_jump_owner&&self.player.client_side&&self.player.native_client_queries_allowed(){
            let keys=crate::reverse_step_bridge::keys();
            let moving=keys[..4].iter().any(|v|*v);
            let response=crate::no_web_bridge::collision([pos.x,pos.y,pos.z],moving,self.player.ground.on_ground,self.player.health.tick_count);
            if let Ok(e)=serde_json::from_str::<serde_json::Value>(&response){
                if e["breakWorld"]==true{self.scene.set(pos.x,pos.y,pos.z,self.assets.registry.default_state("minecraft:air").expect("air state"))?;}
                if e["destroyPackets"]==true{if let Some(sender)=self.step_sender{use horizon_protocol::packets::play::{self,Serverbound as S};for action in [0,1,2]{sender(S::PlayerAction(play::PlayerAction{action,pos:[pos.x,pos.y,pos.z],direction:0,sequence:0}));}}}
                if e["jump"]==true{self.jump_from_ground()?;}
                if let Some(speed)=e["speed"].as_f64(){
                    let strength=e["blend"].as_f64().unwrap_or(1.);
                    let speed=if speed<0.{(self.player.velocity.x*self.player.velocity.x+self.player.velocity.z*self.player.velocity.z).sqrt()}else{speed};
                    let mut yaw=self.player.yaw;
                    if keys[1]{yaw+=180.;}
                    let factor=if keys[1]{-0.5}else if keys[0]{0.5}else{1.};
                    if keys[2]{yaw-=90.*factor;}if keys[3]{yaw+=90.*factor;}
                    let angle=(yaw as f64).to_radians();
                    self.player.velocity.x=self.player.velocity.x*(1.-strength)-horizon_core::jmath::sin(angle)*speed*strength;
                    self.player.velocity.z=self.player.velocity.z*(1.-strength)+horizon_core::jmath::cos(angle)*speed*strength;
                }
                if let Some(y)=e["y"].as_f64(){self.player.velocity.y=y;}
                if e["cancel"]==true{return Ok(());}
            }
        }
        self.web_inside();Ok(())
    }
