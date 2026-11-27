#!/usr/bin/env python3
"""Transactionally wire complete Scaffold/Clutch; root applies with the batch."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 modules=['scaffold_block_selection','scaffold_clutch','scaffold_lifecycle','scaffold_movement_planner','scaffold_prediction','scaffold_aim','scaffold_techniques','scaffold_special_techniques','scaffold_features','clutch']
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod clicking;','pub mod clicking;\n'+'\n'.join('pub mod '+m+';'for m in modules))
 p.edit('lib.rs','mod auto_shoot_runtime;','mod auto_shoot_runtime;\n#[cfg(feature="behavior-hooks")]\nmod scaffold_runtime;\n#[cfg(feature="behavior-hooks")]\nmod scaffold_features_runtime;')
 p.edit('behaviors.rs','    pub auto_shoot:crate::auto_shoot_runtime::AutoShootRuntime,','    pub auto_shoot:crate::auto_shoot_runtime::AutoShootRuntime,\n    pub scaffold:crate::scaffold_runtime::ScaffoldRuntime,\n    pub clutch:crate::scaffold_runtime::ClutchRuntime,')
 p.edit('behaviors.rs','            "AutoShoot",','            "AutoShoot",\n            "Scaffold",\n            "Clutch",')
 p.edit('behaviors.rs','        let criticals=std::sync::Arc::new(crate::criticals_runtime::CriticalsService::new(', '        let scaffold=crate::scaffold_runtime::ScaffoldRuntime::default();\n        let criticals=std::sync::Arc::new(crate::criticals_runtime::CriticalsService::new(')
 p.edit('behaviors.rs','no_slow.clone(),anti_void.clone(),fake_lag.clone(),kill_aura_blocking.clone()));','no_slow.clone(),anti_void.clone(),fake_lag.clone(),kill_aura_blocking.clone(),scaffold.blink.clone()));')
 p.edit('behaviors.rs','            auto_shoot:Default::default(),','            auto_shoot:Default::default(),\n            scaffold,\n            clutch:Default::default(),')
 p.edit('behaviors.rs','    pub fn query_running(&self, registry: &ModuleRegistry, name: &str) -> bool {','    pub fn query_running(&self, registry: &ModuleRegistry, name: &str) -> bool {\n        if name=="Scaffold"&&self.query_ids.get("Clutch").is_some_and(|id|registry.running(*id))&&!self.clutch.path.is_null(){return true;}')
 p.edit('behaviors.rs','            crate::auto_shoot_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoShoot"),','            crate::auto_shoot_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoShoot"),\n            crate::scaffold_runtime::ROTATION_PROVIDER=>self.query_running(registry,"Scaffold"),')
 p.edit('behaviors.rs','        match callback {','''        match callback {
            ModuleCallback::OnEnabled(id) if self.query_ids.get("Scaffold")==Some(&id)=>{self.scaffold.enable_pending=true;if self.scaffold.flag("Technique.Normal.Eagle.Enabled"){self.scaffold.eagle.placed=0;}self.scaffold.head_cooldown=0;self.scaffold.move_ticks=0;},
            ModuleCallback::OnDisabled(id) if self.query_ids.get("Scaffold")==Some(&id)=>{if self.scaffold.flag("Strafe.Enabled")&&self.scaffold.flag("Strafe.Hypixel"){self.scaffold.strafe_disable_pending=true;}self.scaffold.reset();self.scaffold.blink.state.lock().expect("Scaffold Blink").running=false;self.pending_slot_resets.push(crate::scaffold_runtime::REQUESTER);},
            ModuleCallback::CancelSequences(id) if self.query_ids.get("Scaffold")==Some(&id)=>{self.scaffold.waiting=false;self.scaffold.tower.pending.clear();},
            ModuleCallback::OnDisabled(id) if self.query_ids.get("Clutch")==Some(&id)=>self.clutch.disabled(),''')
 p.edit('config.rs','pub use auto_shoot::{AutoShootSetting,serialize_auto_shoot};','pub use auto_shoot::{AutoShootSetting,serialize_auto_shoot};\n#[path="config/scaffold.rs"]\npub(crate) mod scaffold;\npub use scaffold::{ScaffoldSetting,ClutchSetting,serialize_scaffold,serialize_clutch};')
 p.edit('config.rs','    AutoShoot=>("AutoShoot","Combat","autoShoot"),','    AutoShoot=>("AutoShoot","Combat","autoShoot"),\n    Scaffold=>("Scaffold","World","scaffold"),\n    Clutch=>("Clutch","World","clutch"),')
 p.edit('config.rs','    AutoShoot(AutoShootSetting),','    AutoShoot(AutoShootSetting),\n    Scaffold(ScaffoldSetting),\n    Clutch(ClutchSetting),',2)
 p.edit('config.rs','            | ModuleKind::AutoShoot','            | ModuleKind::AutoShoot\n            | ModuleKind::Scaffold\n            | ModuleKind::Clutch')
 p.edit('config.rs','        ModuleKind::AutoShoot=>auto_shoot::parse_auto_shoot(&mut patch,&root),','        ModuleKind::AutoShoot=>auto_shoot::parse_auto_shoot(&mut patch,&root),\n        ModuleKind::Scaffold=>scaffold::parse_scaffold(&mut patch,&root),\n        ModuleKind::Clutch=>scaffold::parse_clutch(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::AutoShoot(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoShoot(value.clone())),','        SettingChange::AutoShoot(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoShoot(value.clone())),\n        SettingChange::Scaffold(value)=>return ApplyResult::Runtime(RuntimeRequest::Scaffold(value.clone())),\n        SettingChange::Clutch(value)=>return ApplyResult::Runtime(RuntimeRequest::Clutch(value.clone())),')
 p.edit('config/placement_esp.rs','fn placement_tree(','pub(super) fn placement_tree(')
 p.edit('config/placement_esp.rs','fn parse_placement(','pub(super) fn parse_placement(')
 p.edit('actor.rs','            ModuleKind::AutoShoot=>config::serialize_auto_shoot(&b.auto_shoot,&common,format,&describe),','            ModuleKind::AutoShoot=>config::serialize_auto_shoot(&b.auto_shoot,&common,format,&describe),\n            ModuleKind::Scaffold=>config::serialize_scaffold(&b.scaffold,&common,format,&describe),\n            ModuleKind::Clutch=>config::serialize_clutch(b.clutch.min_fall_distance,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::AutoShoot(setting))=>{','''                ApplyResult::Runtime(RuntimeRequest::Scaffold(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().scaffold,&self.state.block_registry){eprintln!("[LiquidBounce] Scaffold config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::Clutch(setting))=>{self.state.behaviors.borrow_mut().clutch.min_fall_distance=setting.0;}
                ApplyResult::Runtime(RuntimeRequest::AutoShoot(setting))=>{''')
 p.edit('actor.rs','            ModuleKind::AutoShoot=>Some(','            ModuleKind::Scaffold=>Some(match self.state.behaviors.borrow().scaffold.tag("Technique"){"Expand"=>"Expand","GodBridge"=>"GodBridge","Breezily"=>"Breezily",_=>"Normal"}),\n            ModuleKind::AutoShoot=>Some(')
 p.edit('coroutine_ticker.rs','    HighJump,','    HighJump,\n    Scaffold,\n    ScaffoldTower,')
 p.edit('runtime.rs','                WaitOwner::HighJump => self.high_jump_callback(host, in_game, task.callback),','                WaitOwner::HighJump => self.high_jump_callback(host, in_game, task.callback),\n                WaitOwner::Scaffold=>self.scaffold_wait(in_game),\n                WaitOwner::ScaffoldTower=>self.scaffold_tower_wait(host,in_game),')
 p.edit('runtime.rs','        self.auto_shoot_first_tick();','        self.auto_shoot_first_tick();\n        self.scaffold_first_tick();')
 p.edit('runtime.rs','        self.spider_tick(host);','        self.spider_tick(host);\n        self.scaffold_feature_tick(host);')
 p.edit('runtime.rs','        self.auto_shoot_prepare(host);','        self.auto_shoot_prepare(host);\n        self.scaffold_prepare(host);')
 p.edit('runtime.rs','        self.auto_shoot_tick(host);','        self.auto_shoot_tick(host);\n        self.scaffold_tick(host);')
 p.edit('runtime.rs','            let auto_shoot_running=modules.query_running(&state.registry,"AutoShoot");','            let auto_shoot_running=modules.query_running(&state.registry,"AutoShoot");\n            let scaffold_running=modules.query_running(&state.registry,"Scaffold");')
 p.edit('runtime.rs','                        crate::auto_shoot_runtime::ROTATION_PROVIDER=>auto_shoot_running,','                        crate::auto_shoot_runtime::ROTATION_PROVIDER=>auto_shoot_running,\n                        crate::scaffold_runtime::ROTATION_PROVIDER=>scaffold_running,')
 p.edit('runtime.rs','                        } else if let Some(mode)=crate::auto_shoot_runtime::AutoShootRuntime::processor_mode(processor){','                        } else if let Some(mode)=crate::scaffold_runtime::ScaffoldRuntime::processor_mode(processor){\n                            (&mut modules.scaffold.rotations,mode)\n                        } else if let Some(mode)=crate::auto_shoot_runtime::AutoShootRuntime::processor_mode(processor){')
 p.edit('runtime.rs','        self.inventory_move_input(host,event);','        self.inventory_move_input(host,event);\n        self.scaffold_input_early(host,event);')
 p.edit('runtime.rs','        self.no_slow_input(host,event);','        self.no_slow_input(host,event);\n        self.scaffold_input_model(host,event);\n        self.scaffold_input_safety(host,event);')
 p.edit('runtime.rs','        self.inventory_move_final(event);','        self.inventory_move_final(event);\n        self.scaffold_input_final(host,event);\n        let ground=self.state().behaviors.borrow().scaffold.facts["onGround"]==true;\n        event.sprint=self.scaffold_sprint(0,event.sprint,event.forward||event.backward||event.left||event.right,ground);')
 # The extra Sprint handlers run outside the actor borrow, retaining source order.
 p.edit('runtime.rs','        let criticals_stop_sprint=self.criticals_stop_sprint(host,1);','        let criticals_stop_sprint=self.criticals_stop_sprint(host,1);\n        let scaffold_active=self.scaffold_active();\n        let scaffold_omni=self.scaffold_omnidirectional();')
 p.edit('runtime.rs','            event.omnidirectional = modules.sprint.omnidirectional(running, false);','            event.omnidirectional = modules.sprint.omnidirectional(running, false)||scaffold_omni;\n            if scaffold_active{event.movement_override=modules.scaffold.sprint_value(1,event.movement_override,event.sideways!=0.||event.forward!=0.,event.on_ground); }')
 p.edit('runtime.rs','        let criticals_stop_sprint=self.criticals_stop_sprint(host,2);','        let criticals_stop_sprint=self.criticals_stop_sprint(host,2);\n        let ground=self.state().behaviors.borrow().scaffold.facts["onGround"]==true;\n        let moving=self.state().behaviors.borrow().directional_input.is_moving();\n        event.sprinting=self.scaffold_sprint(2,event.sprinting,moving,ground);')
 p.edit('runtime.rs','    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<\'_>) {','''    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {
        {let s=self.state();let mut b=s.behaviors.borrow_mut();b.scaffold.reset();b.clutch.guarding=false;b.clutch.path=serde_json::Value::Null;b.scaffold.blink.state.lock().expect("Scaffold Blink").running=false;}
        host.client_reset_silent_slot(crate::scaffold_runtime::REQUESTER);''')
 p.edit('runtime.rs','    fn client_air_place_configuration(','''    fn client_scaffold_success(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientScaffoldSuccessEvent){self.scaffold_success(host,event.placed,event.simulated);}
    #[cfg(feature="behavior-hooks")]
    fn client_scaffold_jump(&self,event:&mut horizon_client::mod_api::ClientScaffoldJumpEvent){self.scaffold_jump(event.y,event.motion);}
    #[cfg(feature="behavior-hooks")]
    fn client_scaffold_after_jump(&self,event:&mut horizon_client::mod_api::ClientScaffoldAfterJumpEvent){self.scaffold_after_jump(event.yaw,[event.forward,event.backward,event.left,event.right],event.x,event.y,event.z);}
    #[cfg(feature="behavior-hooks")]
    fn client_air_place_configuration(''')
 p.edit('crosshair_clicker_runtime.rs','self.hooks.auto_shoot_other_click(id);','self.hooks.auto_shoot_other_click(id);self.hooks.scaffold_other_click(id);')
 p.edit('crosshair_clicker_runtime.rs','        self.auto_shoot_player_press(use_key,open,pressed);','        self.auto_shoot_player_press(use_key,open,pressed);\n        self.scaffold_player_press(use_key,open,pressed);')
 p.edit('placement_esp_runtime.rs','        let void = m.query_running(&state.registry, "VoidESP");','        let void = m.query_running(&state.registry, "VoidESP");\n        let scaffold=m.query_running(&state.registry,"Scaffold");')
 p.edit('placement_esp_runtime.rs','                *event.output = geometry(draws).to_string();','                if m.scaffold.render.running(scaffold){draws.extend(m.scaffold.render.render(time));}\n                *event.output = geometry(draws).to_string();')
 p.edit('criticals_runtime.rs','    kill_aura:Arc<crate::kill_aura_runtime::KillAuraBlocking>,','    kill_aura:Arc<crate::kill_aura_runtime::KillAuraBlocking>,\n    scaffold:Arc<crate::scaffold_runtime::ScaffoldBlink>,',2)
 p.edit('criticals_runtime.rs','            kill_aura,','            kill_aura,\n            scaffold,')
 p.edit('criticals_runtime.rs','if fake_lag||self.blink_module.queues(outgoing)','if self.scaffold.queues(outgoing,use_item)||fake_lag||self.blink_module.queues(outgoing)')
 extensions=['falling-forecast-patch.py','scaffold-replay-patch.py','scaffold-selection-patch.py','scaffold-clutch-patch.py','scaffold-action-patch.py','scaffold-planner-patch.py','scaffold-prediction-patch.py','scaffold-techniques-patch.py','scaffold-click-inside-patch.py','clutch-prediction-patch.py','scaffold-features-patch.py','scaffold-special-techniques-patch.py']
 p.edit('../../make-behavior-patch.py',"'fightbot-navigation-patch.py', 'step-patch.py'","'fightbot-navigation-patch.py', "+', '.join(repr(x)for x in extensions)+", 'step-patch.py'")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('Scaffold/Clutch '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
