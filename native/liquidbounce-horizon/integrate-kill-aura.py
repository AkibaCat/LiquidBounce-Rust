"""Transactional KillAura runtime/config/shared Blink wiring. --check is read-only."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']

def prepare():
 p=Plan()
 p.edit('lib.rs','mod aimbot_runtime;','mod aimbot_runtime;\n#[cfg(feature="behavior-hooks")]\nmod kill_aura_runtime;')
 p.edit('runtime.rs','    _packets: std::sync::Arc<crate::packets::PacketPipeline>,','    _packets: std::sync::Arc<crate::packets::PacketPipeline>,\n    #[cfg(feature="behavior-hooks")]\n    pub(crate) kill_aura_blocking:std::sync::Arc<crate::kill_aura_runtime::KillAuraBlocking>,')
 p.edit('runtime.rs','        let state = State::default();','        let state = State::default();\n        #[cfg(feature="behavior-hooks")]\n        let kill_aura_blocking=state.behaviors.borrow().kill_aura.blocking.clone();')
 p.edit('runtime.rs','            _packets: packets,','            _packets: packets,\n            #[cfg(feature="behavior-hooks")]\n            kill_aura_blocking,')
 p.edit('behaviors.rs','    pub aimbot:crate::aimbot_runtime::AimbotRuntime,','    pub aimbot:crate::aimbot_runtime::AimbotRuntime,\n    pub kill_aura:crate::kill_aura_runtime::KillAuraRuntime,')
 p.edit('behaviors.rs','            "Aimbot",','            "Aimbot",\n            "KillAura",')
 p.edit('behaviors.rs','        let no_slow=','        let kill_aura_blocking=std::sync::Arc::new(crate::kill_aura_runtime::KillAuraBlocking::default());\n        let no_slow=')
 p.edit('behaviors.rs','anti_void.clone(),fake_lag.clone()));','anti_void.clone(),fake_lag.clone(),kill_aura_blocking.clone()));')
 p.edit('behaviors.rs','            aimbot:Default::default(),','            aimbot:Default::default(),\n            kill_aura:crate::kill_aura_runtime::KillAuraRuntime::new(kill_aura_blocking),')
 p.edit('behaviors.rs','            crate::auto_rod_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoRod"),','            crate::auto_rod_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoRod"),\n            crate::kill_aura_runtime::ROTATION_PROVIDER=>self.query_running(registry,"KillAura"),')
 anchor='            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("Aimbot")==Some(&id)=>self.aimbot.clear(),'
 p.edit('behaviors.rs',anchor,anchor+'\n            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("KillAura")==Some(&id)=>{self.kill_aura.clear();self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::KillAuraDisabled);},')
 p.edit('config.rs','pub use aimbot::{AimbotSetting,serialize_aimbot};','pub use aimbot::{AimbotSetting,serialize_aimbot};\n#[path="config/kill_aura.rs"]\nmod kill_aura;\npub use kill_aura::{KillAuraSetting,serialize_kill_aura};')
 p.edit('config.rs','    Aimbot=>("Aimbot","Combat","aimbot"),','    Aimbot=>("Aimbot","Combat","aimbot"),\n    KillAura=>("KillAura","Combat","killAura"),')
 p.edit('config.rs','    Aimbot(AimbotSetting),','    Aimbot(AimbotSetting),\n    KillAura(KillAuraSetting),',2)
 p.edit('config.rs','            | ModuleKind::Aimbot','            | ModuleKind::Aimbot\n            | ModuleKind::KillAura')
 p.edit('config.rs','        ModuleKind::Aimbot=>aimbot::parse_aimbot(&mut patch,&root),','        ModuleKind::Aimbot=>aimbot::parse_aimbot(&mut patch,&root),\n        ModuleKind::KillAura=>kill_aura::parse_kill_aura(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::Aimbot(value)=>return ApplyResult::Runtime(RuntimeRequest::Aimbot(value.clone())),','        SettingChange::Aimbot(value)=>return ApplyResult::Runtime(RuntimeRequest::Aimbot(value.clone())),\n        SettingChange::KillAura(value)=>return ApplyResult::Runtime(RuntimeRequest::KillAura(value.clone())),')
 p.edit('actor.rs','            ModuleKind::Aimbot=>config::serialize_aimbot(&b.aimbot,&common,format,&describe),','            ModuleKind::Aimbot=>config::serialize_aimbot(&b.aimbot,&common,format,&describe),\n            ModuleKind::KillAura=>config::serialize_kill_aura(&b.kill_aura,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::Aimbot(setting))=>{','                ApplyResult::Runtime(RuntimeRequest::KillAura(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().kill_aura){eprintln!("[LiquidBounce] KillAura config leaf skipped: {e}");}}\n                ApplyResult::Runtime(RuntimeRequest::Aimbot(setting))=>{')
 p.edit('inventory_runtime.rs','        modules.aimbot.clear();','        modules.aimbot.clear();\n        modules.kill_aura.clear();\n        modules.kill_aura.release_on_disable=false;')
 p.edit('one_shots_runtime.rs','    AutoRodDisabled(bool),','    AutoRodDisabled(bool),\n    KillAuraDisabled,')
 p.edit('one_shots_runtime.rs','            OneShotEffect::AutoRodDisabled(release)=>self.auto_rod_disabled(host,release),','            OneShotEffect::AutoRodDisabled(release)=>self.auto_rod_disabled(host,release),\n            OneShotEffect::KillAuraDisabled=>self.kill_aura_disabled(host),')
 p.edit('auto_pearl_runtime.rs','                b.auto_pearl.aura_target,','                if b.query_running(&s.registry,"KillAura"){b.kill_aura.behavior.target}else{None},')
 p.edit('runtime.rs','        self.projectile_puncher_first_tick();','        self.projectile_puncher_first_tick();\n        self.kill_aura_first_tick();')
 p.edit('runtime.rs','        self.projectile_puncher_prepare(host);','        self.projectile_puncher_prepare(host);\n        self.kill_aura_prepare(host);')
 p.edit('runtime.rs','            let projectile_puncher_running=modules.query_running(&state.registry,"ProjectilePuncher");','            let projectile_puncher_running=modules.query_running(&state.registry,"ProjectilePuncher");\n            let kill_aura_running=modules.query_running(&state.registry,"KillAura");')
 p.edit('runtime.rs','                        crate::projectile_puncher_runtime::ROTATION_PROVIDER=>projectile_puncher_running,','                        crate::projectile_puncher_runtime::ROTATION_PROVIDER=>projectile_puncher_running,\n                        crate::kill_aura_runtime::ROTATION_PROVIDER=>kill_aura_running,')
 p.edit('runtime.rs','                        if processor == 0 {','''                        if let Some(id)=processor.checked_sub(crate::kill_aura_runtime::PROCESSOR_BASE).filter(|id|liquidbounce_behaviors::combat_rotations::Settings::decode(*id).is_some()){
                            modules.kill_aura.ai_notice_pending=liquidbounce_behaviors::combat_rotations::Settings::decode(id).is_some_and(|v|v.0==liquidbounce_behaviors::combat_rotations::SmoothMode::Ai);
                            let entity=modules.kill_aura.rotation_entity;
                            return modules.kill_aura.rotations.kill_aura_process(id,kill_aura_running,from,to,last,Rotation::new(last_yaw,last_pitch),projectile_previous_target,projectile_server,entity,&mut liquidbounce_behaviors::random::random_unit_f32).map_err(liquidbounce_behaviors::rotations::RotationError::Processor);
                        }
                        if processor == 0 {''')
 p.edit('runtime.rs','        self.projectile_puncher_ai_notification(host);','        self.kill_aura_ai_notification(host);\n        self.kill_aura_tick(host);\n        self.projectile_puncher_ai_notification(host);')
 p.edit('runtime.rs','        self.anti_bot_attack(event.target as i32);','        self.anti_bot_attack(event.target as i32);\n        self.kill_aura_attack_event();')
 p.edit('runtime.rs','let crosshair_stop_sprint=self.crosshair_stop_sprint(host);','let crosshair_stop_sprint=self.crosshair_stop_sprint(host)||self.kill_aura_stop_sprint(host);',2)
 p.edit('runtime.rs','{self.crosshair_attack_key(&mut event.pressed);}','{self.crosshair_attack_key(&mut event.pressed);self.kill_aura_attack_key(&mut event.pressed);}')
 p.edit('runtime.rs','    fn client_crosshair_clicker_attack_key(','    fn client_kill_aura_state(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientKillAuraStateEvent){self.kill_aura_block_state(&mut event.enforced,&mut event.pose);}\n    #[cfg(feature="behavior-hooks")]\n    fn client_crosshair_clicker_attack_key(')
 p.edit('crosshair_clicker_runtime.rs','        self.projectile_puncher_player_press(attack,open,pressed);','        self.projectile_puncher_player_press(attack,open,pressed);\n        self.kill_aura_player_press(attack,open,pressed);')
 p.edit('crosshair_clicker_runtime.rs','self.hooks.projectile_puncher_other_click(id);','self.hooks.projectile_puncher_other_click(id);self.hooks.kill_aura_other_click(id);')
 # Same-key clickers share their success plans. AutoShoot owns Use and is separate.
 p.edit('projectile_puncher_runtime.rs','            clicks = clicks.wrapping_add(1);','            if b.query_running(&state.registry,"KillAura"){b.kill_aura.clicker.plan.pressed_by(&leader,now);}\n            clicks = clicks.wrapping_add(1);')
 p.edit('target_renderer_runtime.rs','        // TARGET_RENDERER_OWNERS_END','        owner!("KillAura",kill_aura,true);\n        // TARGET_RENDERER_OWNERS_END\n        drop(m);drop(state);\n        if !targets_only{self.kill_aura_render(&input,&mut frame);}')
 c='criticals_runtime.rs'
 p.edit(c,'    fake_lag:Arc<crate::fake_lag_runtime::FakeLagService>,','    fake_lag:Arc<crate::fake_lag_runtime::FakeLagService>,\n    kill_aura:Arc<crate::kill_aura_runtime::KillAuraBlocking>,',2)
 p.edit(c,'            fake_lag,','            fake_lag,\n            kill_aura,')
 p.edit(c,'fake_packet:liquidbounce_behaviors::fake_lag::PacketKind) -> BlinkAction','fake_packet:liquidbounce_behaviors::fake_lag::PacketKind, use_item:bool) -> BlinkAction')
 p.edit(c,'if fake_lag||self.blink_module.queues(outgoing)||self.freeze.blink_queue(outgoing)||self.anti_void.queues(outgoing)','if fake_lag||self.blink_module.queues(outgoing)||self.freeze.blink_queue(outgoing)||self.anti_void.queues(outgoing)||self.kill_aura.queues(outgoing,use_item)')
 p.edit(c,'self.action(&mut s, outgoing, false,None,false,Default::default())','self.action(&mut s, outgoing, false,None,false,Default::default(),false)')
 p.edit(c,'crate::fake_lag_runtime::packet(event.packet))','crate::fake_lag_runtime::packet(event.packet),matches!(event.packet,S::UseItem(_)))')
 p.edit(c,'self.0.action(&mut s, false, false,None,false,Default::default())','self.0.action(&mut s, false, false,None,false,Default::default(),false)')
 anchor='impl PacketListener for BlinkListener {\n    fn outgoing(&self, event: &mut PacketEvent<\'_, S>) {'
 p.edit(c,anchor,anchor+'\n        self.0.kill_aura.outgoing(event.packet);')
 p.edit('../../make-behavior-patch.py',"'step-patch.py'):","'kill-aura-patch.py', 'kill-aura-block-input-patch.py', 'fightbot-navigation-patch.py', 'step-patch.py'):")
 return p

if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('KillAura '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
