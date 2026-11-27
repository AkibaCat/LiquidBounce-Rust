// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::ClientTrajectoriesEvent;
use serde_json::{Value,json};
impl LiquidBounceHooks{
    pub(crate) fn trajectories(&self,e:&mut ClientTrajectoriesEvent<'_>){
        let input=serde_json::from_str::<Value>(e.input).unwrap_or(Value::Null);
        let state=self.state();let mut b=state.behaviors.borrow_mut();
        let enabled=b.query_running(&state.registry,"Trajectories");
        if !enabled||input["phase"]=="reset"{b.trajectories.reset();*e.output="null".into();return;}
        if input["phase"]=="results"{b.trajectories.simulation_results=input["results"].as_array().cloned().unwrap_or_default();return;}
        let mut out=b.trajectories.options();let freeze=b.query_running(&state.registry,"Freeze");
        let rotation={
            let mut rotations=b.rotations.lock().expect("RotationManager");
            if b.query_running(&state.registry,"FreeCam"){Some(rotations.server_rotation(b.criticals.is_lagging(),freeze))}else{rotations.active_target(|id|b.rotation_provider_running(&state.registry,id)).map(|r|r.rotation).or(rotations.current)}
        };
        out["enabled"]=json!(true);out["freeze"]=json!(freeze);out["rotation"]=rotation.map(|r|json!([r.yaw,r.pitch])).unwrap_or(Value::Null);
        *e.output=out.to_string();
    }
}
