// SPDX-License-Identifier: GPL-3.0-or-later
// LiquidBounce-derived core: Copyright (c) 2015-2026 CCBlueX.

//! Source-built trait adapter for Horizon 9f42064 / Minecraft 26.3.
//! This adapter observes real runtime events. No gameplay module is registered
//! until its game implementation exists, and it does not impersonate ClickGUI.

#[cfg(feature = "behavior-hooks")]
use crate::behaviors::{self, BehaviorModules, RawOptions as RuntimeOptions};
use crate::input;
#[cfg(feature = "behavior-hooks")]
use horizon_client::mod_api::{
    CameraBobEffectsEvent, CameraBobResultEvent, CameraFovOptionEvent, CameraFovReturnEvent,
    CameraSpinningResultEvent, ClientAbortBreakingEvent, ClientAirjumpPushQueriesEvent,
    ClientAttackQueriesEvent, ClientAttackRangeEvent, ClientCameraAspectEvent,
    ClientCameraPositionEvent, ClientCrystalViewEvent, ClientDeathScreenOpeningEvent,
    ClientDirectionalInputEvent, ClientInteractionRangeEvent, ClientInterpolationPolicyEvent,
    ClientKeepSprintAttackEvent, ClientKeepSprintMotionEvent, ClientKeepSprintSetEvent,
    ClientMiningConditionsEvent, ClientMovementQueriesEvent, ClientPickFluidEvent,
    ClientPlayerJumpEvent, ClientPlayerReducedDebugEvent, ClientPlayerTickFinalEvent,
    ClientPlayerTickPostEvent, ClientPortalAllowedEvent, ClientReachPolicyEvent,
    ClientRotationAimEvent, ClientRotationModelEvent, ClientRotationNetworkEvent,
    ClientSprintNetworkEvent, ClientSprintQueriesEvent, ClientTickPreEvent, ClientTimerFrameEvent,
    ClientTimerScaleEvent, ClientUseCooldownEvent, ClientVisualEnvironmentPolicyEvent,
    ClientWorldChangeEvent, LightmapGammaOptionEvent, MouseScopingConditionEvent,
    MouseScrollingEvent, PlayerFovReturnEvent,
};
#[cfg(feature = "behavior-hooks")]
use std::cell::RefCell;
#[cfg(not(feature = "behavior-hooks"))]
type RuntimeOptions = ();

use horizon_client::mod_api::{
    ClientDisconnectingEvent, ClientTickPostEvent, Hooks, Host, InputKeyEvent, ModsEvent,
    MouseButtonPostEvent, NativeClientState, NativeMod, NativeScreenKind, ScreenRenderPostEvent,
    SettingsClickEvent, SettingsEvent,
    winit::{event::ElementState, keyboard::PhysicalKey},
};
use liquidbounce_core::{
    InputContext, KeyAction, KeyboardInput, ModuleCallback, ModuleCallbacks, ModuleRegistry,
    ModuleSyncError, ModuleSyncReport, MouseInput, WorldState,
};
use std::{
    sync::{Mutex, MutexGuard},
    time::Instant,
};

pub const MOD_ID: &str = "liquidbounce_horizon";
pub const HORIZON_REVISION: &str = "9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0";

fn ui_label(label: &str) -> String {
    #[cfg(feature = "webview")]
    return liquidbounce_interop::localization::display_label(&horizon_client::lang::code(), label);
    #[cfg(not(feature = "webview"))]
    label.to_owned()
}

fn ui_module_name(name: &str) -> String {
    #[cfg(feature = "webview")]
    return liquidbounce_interop::localization::display_module_name(
        &horizon_client::lang::code(),
        name,
    );
    #[cfg(not(feature = "webview"))]
    name.to_owned()
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ObservedState {
    pub world: WorldState,
    pub ticks: u64,
    pub keyboard_events: u64,
    pub mouse_events: u64,
    pub disconnects: u64,
}

#[derive(Default)]
pub(crate) struct RuntimeEffects {
    #[cfg(feature = "behavior-hooks")]
    one_shots: Vec<crate::one_shots_runtime::OneShotEffect>,
    fast_break_resets: usize,
    night_vision: Vec<Option<i32>>,
    reset_pitch: bool,
    reset_slots: Vec<u64>,
    hand_sends: Vec<i32>,
    model_parts: Vec<u8>,
}

pub(crate) struct State {
    pub(crate) registry: ModuleRegistry,
    pub(crate) observed: ObservedState,
    show_status_when_playing: bool,
    #[cfg(feature = "behavior-hooks")]
    pub(crate) behaviors: RefCell<BehaviorModules>,
    #[cfg(feature = "behavior-hooks")]
    pub(crate) global_targets: liquidbounce_behaviors::targeting::GlobalTargets,
    #[cfg(feature = "behavior-hooks")]
    pub(crate) target_tags: liquidbounce_behaviors::targeting::TagCache,
    #[cfg(feature = "behavior-hooks")]
    pub(crate) friends: liquidbounce_behaviors::targeting::FriendBook,
    #[cfg(feature = "behavior-hooks")]
    pub(crate) combat: liquidbounce_behaviors::combat_window::CombatWindow,
    #[cfg(feature = "behavior-hooks")]
    pub(crate) config_root: std::path::PathBuf,
    #[cfg(feature = "behavior-hooks")]
    pub(crate) block_registry: Vec<String>,
    #[cfg(feature = "webview")]
    pub(crate) global_preserved: serde_json::Value,
    #[cfg(feature = "webview")]
    pub(crate) hidden: std::collections::BTreeMap<String, bool>,
    #[cfg(feature = "webview")]
    pub(crate) events: RefCell<Vec<(String, serde_json::Value)>>,
    #[cfg(feature = "webview")]
    pub(crate) probes: serde_json::Value,
    #[cfg(feature = "webview")]
    pub(crate) sound_registry: Vec<String>,
    #[cfg(feature = "webview")]
    pub(crate) entity_type_registry: Vec<String>,
    #[cfg(feature = "webview")]
    pub(crate) item_registry: Vec<String>,
}

impl Default for State {
    fn default() -> Self {
        #[allow(unused_mut)]
        let mut registry = ModuleRegistry::new();
        #[cfg(feature = "behavior-hooks")]
        let behaviors = RefCell::new(BehaviorModules::register(&mut registry));
        Self {
            registry,
            observed: ObservedState::default(),
            show_status_when_playing: false,
            #[cfg(feature = "behavior-hooks")]
            behaviors,
            #[cfg(feature = "behavior-hooks")]
            global_targets: Default::default(),
            #[cfg(feature = "behavior-hooks")]
            target_tags: Default::default(),
            #[cfg(feature = "behavior-hooks")]
            friends: Default::default(),
            #[cfg(feature = "behavior-hooks")]
            combat: Default::default(),
            #[cfg(feature = "behavior-hooks")]
            config_root: Default::default(),
            #[cfg(feature = "behavior-hooks")]
            block_registry: Vec::new(),
            #[cfg(feature = "webview")]
            global_preserved: serde_json::json!({"name":"Settings","value":[]}),
            #[cfg(feature = "webview")]
            hidden: Default::default(),
            #[cfg(feature = "webview")]
            events: Default::default(),
            #[cfg(feature = "webview")]
            probes: serde_json::json!({}),
            #[cfg(feature = "webview")]
            sound_registry: Vec::new(),
            #[cfg(feature = "webview")]
            entity_type_registry: Vec::new(),
            #[cfg(feature = "webview")]
            item_registry: Vec::new(),
        }
    }
}

impl State {
    #[cfg(feature = "behavior-hooks")]
    pub(crate) fn target_info(
        &mut self,
        id: i32,
        snapshot: &crate::targeting::Snapshot,
    ) -> liquidbounce_behaviors::targeting::TargetInfo {
        self.target_tags.get(id, || {
            let mut info = self
                .friends
                .tag(snapshot.entity.kind, snapshot.player_name.as_deref());
            let player = snapshot
                .abstract_client_player
                .then(|| {
                    snapshot.player_name.as_ref().map(|name| {
                        liquidbounce_behaviors::target_lock::Player {
                            id,
                            name: name.clone(),
                        }
                    })
                })
                .flatten();
            let mut modules = self.behaviors.borrow_mut();
            crate::target_lock_runtime::apply_tag(
                &modules.target_lock,
                modules.query_running(&self.registry, "TargetLock"),
                player.as_ref(),
                &mut info,
            );
            crate::teams_runtime::apply_tag(&modules.teams,modules.query_running(&self.registry,"Teams"),snapshot.entity.kind.living,snapshot.teams.as_ref(),&mut info);
            let murder_running=modules.query_running(&self.registry,"MurderMystery");
            crate::murder_mystery_runtime::apply_tag(&mut modules.murder_mystery,murder_running,snapshot.abstract_client_player,snapshot.player_name.as_deref(),snapshot.murder_mystery_uuid.as_deref(),&mut info);
            if let Some(q)=&snapshot.anti_bot{if modules.anti_bot.state.is_bot(modules.query_running(&self.registry,"AntiBot"),&q.player,&q.online,q.max_health){info.ignore();}}
            info
        })
    }

    #[cfg(feature = "webview")]
    pub(crate) fn camera_distance_changed(&self) {
        Self::publish_camera_distance_changed(&self.behaviors, &self.events);
    }

    #[cfg(feature = "webview")]
    fn publish_camera_distance_changed(
        behaviors: &RefCell<BehaviorModules>,
        events: &RefCell<Vec<(String, serde_json::Value)>>,
    ) {
        let language = horizon_client::lang::code();
        let describe = |key: &str| liquidbounce_interop::localization::describe(&language, key);
        let common = crate::config::CommonSettings {
            enabled: false,
            hidden: false,
            key_bind: serde_json::Value::Null,
        };
        if let Ok(tree) = crate::config::serialize_camera_clip(
            &behaviors.borrow().camera_clip,
            &common,
            crate::config::JsonFormat::Interop,
            &describe,
        ) {
            if let Some(value) = tree["value"].as_array().and_then(|values| {
                values
                    .iter()
                    .find(|value| value["name"] == "CameraDistance")
            }) {
                events
                    .borrow_mut()
                    .push(("valueChanged".into(), serde_json::json!({"value": value})));
            }
        }
    }

    #[cfg(feature = "webview")]
    pub(crate) fn probe(
        &mut self,
        name: &str,
        input: serde_json::Value,
        output: serde_json::Value,
    ) {
        let count = self.probes[name]["count"]
            .as_u64()
            .unwrap_or(0)
            .saturating_add(1);
        self.probes[name] = serde_json::json!({"count":count,"input":input,"output":output});
    }
    pub(crate) fn with_registry<R>(
        &mut self,
        options: RuntimeOptions,
        operation: impl FnOnce(&mut ModuleRegistry, &dyn ModuleCallbacks) -> R,
    ) -> R {
        #[cfg(feature = "behavior-hooks")]
        {
            let callbacks = |registry: &mut ModuleRegistry, callback: ModuleCallback| {
                self.behaviors.borrow().sync_packet_modules(registry);
                #[cfg(feature = "webview")]
                if let ModuleCallback::PublishEnabled { module, enabled } = callback {
                    let name = registry
                        .module(module)
                        .expect("registered callback")
                        .definition()
                        .name
                        .clone();
                    self.events.borrow_mut().push(("moduleToggle".into(), serde_json::json!({"moduleName":name,"enabled":enabled,"hidden":self.hidden.get(&name).copied().unwrap_or(false)})));
                }
                #[cfg(feature = "webview")]
                let camera_distance = self.behaviors.borrow().camera_clip.camera_distance;
                self.behaviors
                    .borrow_mut()
                    .callback(registry, callback, options);
                #[cfg(feature = "webview")]
                if camera_distance != self.behaviors.borrow().camera_clip.camera_distance {
                    Self::publish_camera_distance_changed(&self.behaviors, &self.events);
                }
                Ok(())
            };
            let result = operation(&mut self.registry, &callbacks);
            self.behaviors.borrow().sync_packet_modules(&self.registry);
            result
        }
        #[cfg(not(feature = "behavior-hooks"))]
        {
            let _ = options;
            let callbacks = |_registry: &mut ModuleRegistry, _callback: ModuleCallback| {
                Err("No actual game implementation is attached".to_owned())
            };
            operation(&mut self.registry, &callbacks)
        }
    }
    pub(crate) fn take_effects(&mut self) -> RuntimeEffects {
        #[cfg(feature = "behavior-hooks")]
        {
            let mut modules = self.behaviors.borrow_mut();
            RuntimeEffects {
                one_shots: std::mem::take(&mut modules.pending_one_shots),
                fast_break_resets: std::mem::take(&mut modules.pending_fast_break_resets),
                night_vision: std::mem::take(&mut modules.pending_night_vision),
                reset_pitch: std::mem::take(&mut modules.pending_pitch_reset),
                reset_slots: std::mem::take(&mut modules.pending_slot_resets),
                hand_sends: std::mem::take(&mut modules.pending_hand_sends),
                model_parts: std::mem::take(&mut modules.pending_model_parts),
            }
        }
        #[cfg(not(feature = "behavior-hooks"))]
        {
            RuntimeEffects::default()
        }
    }
}

fn log_report(report: &ModuleSyncReport) {
    for failure in &report.failures {
        eprintln!("[LiquidBounce] module callback failure: {failure:?}");
    }
}
fn log_reports(result: Result<Vec<ModuleSyncReport>, ModuleSyncError>) {
    match result {
        Ok(reports) => {
            for report in reports {
                log_report(&report);
            }
        }
        Err(error) => eprintln!("[LiquidBounce] module operation failed: {error:?}"),
    }
}
pub(crate) fn raw_options(host: &dyn Host) -> RuntimeOptions {
    #[cfg(feature = "behavior-hooks")]
    {
        RuntimeOptions {
            fov: host.client_option_fov(),
            gamma: host.client_option_gamma(),
            player_rotation: host.client_player_rotation(),
            actual_main_hand: host.client_hand_derp_hand(),
            model_parts: host.client_skin_derp_parts() as u8,
        }
    }
    #[cfg(not(feature = "behavior-hooks"))]
    {
        let _ = host;
    }
}
pub(crate) fn apply_effects(
    hooks: &LiquidBounceHooks,
    host: &mut dyn Host,
    intents: RuntimeEffects,
) {
    #[cfg(feature = "behavior-hooks")]
    {
        for effect in intents.one_shots {
            hooks.apply_one_shot(host, effect);
        }
        for duration in intents.night_vision {
            host.client_night_vision(duration);
        }
        if intents.reset_pitch {
            host.client_reset_pitch();
        }
        for _ in 0..intents.fast_break_resets {
            host.client_fast_break_reset();
        }
        for hand in intents.hand_sends {
            host.client_hand_derp_send(hand);
        }
        for parts in intents.model_parts {
            host.client_skin_derp_set_parts(i32::from(parts));
        }
        for requester in intents.reset_slots {
            host.client_reset_silent_slot(requester);
        }
    }
    #[cfg(not(feature = "behavior-hooks"))]
    {
        let _ = host;
        let _ = hooks;
        assert!(
            intents.night_vision.is_empty()
                && intents.fast_break_resets == 0
                && !intents.reset_pitch
                && intents.reset_slots.is_empty()
                && intents.hand_sends.is_empty()
                && intents.model_parts.is_empty()
        );
    }
}

fn debug_modifier_down(host: &dyn Host) -> bool {
    #[cfg(feature = "behavior-hooks")]
    {
        host.client_debug_modifier_down()
    }
    #[cfg(not(feature = "behavior-hooks"))]
    {
        // The unextended adapter has no modules. Original Host exposes raw
        // physical keys but does not expose the configurable debug mapping.
        host.is_key_down(horizon_client::mod_api::winit::keyboard::KeyCode::F3)
    }
}

pub struct LiquidBounceHooks {
    state: Mutex<State>,
    #[cfg(feature = "behavior-hooks")]
    _packets: std::sync::Arc<crate::packets::PacketPipeline>,
    #[cfg(feature="behavior-hooks")]
    pub(crate) sword_block:std::sync::Arc<crate::sword_block_runtime::SwordBlockService>,
    #[cfg(feature="behavior-hooks")]
    pub(crate) kill_aura_blocking:std::sync::Arc<crate::kill_aura_runtime::KillAuraBlocking>,
    pub(crate) started: Instant,
    #[cfg(feature = "webview")]
    pub(crate) browser: Option<Mutex<crate::browser::Browser>>,
}

impl Default for LiquidBounceHooks {
    fn default() -> Self {
        let state = State::default();
        #[cfg(feature="behavior-hooks")]
        horizon_client::text::install_name_protection(state.behaviors.borrow().name_protect.clone());
        #[cfg(feature="behavior-hooks")]
        let sword_block=state.behaviors.borrow().sword_block.clone();
        #[cfg(feature="behavior-hooks")]
        let kill_aura_blocking=state.behaviors.borrow().kill_aura.blocking.clone();
        #[cfg(feature = "behavior-hooks")]
        let packets = {
            let modules = state.behaviors.borrow();
            let pipeline = std::sync::Arc::new(crate::packets::PacketPipeline::with_no_swing(
                modules.no_swing.clone(),
            ));
            pipeline.add_listener(i32::MAX, modules.plugins.clone());
            pipeline.add_listener(1000,modules.inventory_move.clone());
            pipeline.add_listener(0,modules.anti_cheat_detect.clone());
            pipeline.add_listener(0, modules.anti_hunger.clone());
            pipeline.add_listener(0,modules.sneak.clone());
            pipeline.add_listener(0,modules.no_slow.clone());
            pipeline.add_listener(-10,std::sync::Arc::new(crate::blink_runtime::AmbushListener(modules.blink.clone(),modules.criticals.clone())));
            pipeline.add_listener(-1000,std::sync::Arc::new(crate::no_slow_runtime::NoSlowFinal(modules.no_slow.clone())));
            pipeline.add_listener(0, modules.auto_disable_packets.clone());
            pipeline.add_listener(0, modules.flag_check.clone());
            pipeline.add_listener(0,modules.anti_staff.clone());
            pipeline.add_listener(-1000,modules.packet_logger.clone());
            pipeline.add_listener(0,modules.logoff_spot.clone());
            pipeline.add_listener(0, modules.no_clip_packets.clone());
            pipeline.add_listener(
                0,
                std::sync::Arc::new(crate::packet_modules::MoreCarryListener(
                    modules.more_carry.clone(),
                )),
            );
            pipeline.add_listener(
                0,
                std::sync::Arc::new(crate::packet_modules::NoPitchLimitListener(
                    modules.no_pitch_limit.clone(),
                )),
            );
            pipeline.add_listener(1, modules.hand_derp.clone());
            pipeline.add_listener(0, modules.auto_fish.clone());
            pipeline.add_listener(0, modules.fast_break.clone());
            pipeline.add_listener(0, modules.timer.clone());
            pipeline.add_listener(0,modules.liquid_walk.clone());
            pipeline.add_listener(-910,modules.step.clone());
            pipeline.add_listener(0, modules.freeze.clone());
            pipeline.add_listener(0, modules.combat_cooldown.clone());
            pipeline.add_listener(0, modules.crosshair_break_observer.clone());
            pipeline.add_listener(0,std::sync::Arc::new(crate::criticals_runtime::NoGroundListener(modules.criticals.clone())));
            pipeline.add_listener(-500,std::sync::Arc::new(crate::criticals_runtime::BlinkListener(modules.criticals.clone())));
            pipeline.add_listener(-1000,std::sync::Arc::new(crate::criticals_runtime::RewoundListener(modules.criticals.clone())));
            pipeline.add_listener(-1000, modules.inventory_packets.clone());
            pipeline.add_listener(i32::MIN+1, modules.ping_spoof.clone());
            let rotations = std::sync::Arc::new(crate::rotation_packets::RotationListener(
                modules.rotations.clone(),
            ));
            pipeline.add_listener(i32::MIN, rotations.clone());
            pipeline.set_silent_sinks(crate::packets::SilentSinks {
                rotation: Some(rotations),
                packet_logger:Some(modules.packet_logger.clone()),
                ..Default::default()
            });
            modules.slots.install();
            pipeline.install();
            pipeline
        };
        Self {
            state: Mutex::new(state),
            #[cfg(feature = "behavior-hooks")]
            _packets: packets,
            #[cfg(feature="behavior-hooks")]
            sword_block,
            #[cfg(feature="behavior-hooks")]
            kill_aura_blocking,
            started: Instant::now(),
            #[cfg(feature = "webview")]
            browser: None,
        }
    }
}

impl LiquidBounceHooks {
    #[cfg(feature = "behavior-hooks")]
    fn auto_fish_callback(&self, host: &mut dyn Host, client_present: bool, callback: u64) -> bool {
        use crate::fishing_packets::FishHand;
        let hand = match host.client_fishing_rod_hand() {
            Some(0) => Some(FishHand::Main),
            Some(1) => Some(FishHand::Off),
            _ => None,
        };
        let hook = host.client_fishing_hook();
        let hook_present = hook.is_some();
        #[cfg(feature = "webview")]
        self.state().probe("autoFishObserved",
            serde_json::json!({"rodHand":hand.map(|h| match h { FishHand::Main=>0, FishHand::Off=>1 })}),
            serde_json::json!({"hook":hook}));
        let mut random = liquidbounce_behaviors::random::random_i32_inclusive;
        let next = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            let running = client_present && modules.query_running(&state.registry, "AutoFish");
            modules.auto_fish.set_running(running);
            let mut fish = modules
                .auto_fish
                .behavior
                .lock()
                .expect("AutoFish poisoned");
            let result = fish.advance_callback(callback, running, hand, hook_present, &mut random);
            modules.auto_fish.register_waits(&mut fish);
            result
        };
        let (keep, mut next) = match next {
            Ok(result) => result,
            Err(error) => {
                eprintln!("[LiquidBounce] AutoFish: {error}");
                (false, None)
            }
        };
        while let Some(hand) = next {
            // RotationManager.currentRotation is read at each real use. The
            // ordinary native path can invoke PacketEvent and registry work.
            let [yaw, pitch] = host.client_player_rotation();
            let rotation = {
                let state = self.state();
                let modules = state.behaviors.borrow();
                let rotations = modules
                    .rotations
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                rotations.current
            };
            let (yaw, pitch) =
                rotation.map_or((yaw, pitch), |rotation| (rotation.yaw, rotation.pitch));
            host.client_use_item(
                match hand {
                    FishHand::Main => 0,
                    FishHand::Off => 1,
                },
                yaw,
                pitch,
            );
            #[cfg(feature = "webview")]
            {
                let hook = host.client_fishing_hook();
                self.state().probe("autoFishUseItem",
                    serde_json::json!({"hand":match hand { FishHand::Main=>0, FishHand::Off=>1 },"yaw":yaw,"pitch":pitch}),
                    serde_json::json!({"hook":hook}));
            }
            let result = {
                let state = self.state();
                let modules = state.behaviors.borrow();
                let mut fish = modules
                    .auto_fish
                    .behavior
                    .lock()
                    .expect("AutoFish poisoned");
                let result = fish.after_use(&mut random);
                modules.auto_fish.register_waits(&mut fish);
                result
            };
            next = match result {
                Ok(next) => next,
                Err(error) => {
                    eprintln!("[LiquidBounce] AutoFish: {error}");
                    None
                }
            };
        }
        keep
    }

    #[cfg(feature = "behavior-hooks")]
    fn auto_fish_tick_handler(&self, host: &dyn Host) {
        use crate::fishing_packets::FishHand;
        let hand = match host.client_fishing_rod_hand() {
            Some(0) => Some(FishHand::Main),
            Some(1) => Some(FishHand::Off),
            _ => None,
        };
        let state = self.state();
        let modules = state.behaviors.borrow();
        let running = modules.query_running(&state.registry, "AutoFish");
        let mut fish = modules
            .auto_fish
            .behavior
            .lock()
            .expect("AutoFish poisoned");
        fish.start_tick_handler(running, hand);
        modules.auto_fish.register_waits(&mut fish);
    }

    #[cfg(feature = "behavior-hooks")]
    fn coroutine_callbacks(&self, host: &mut dyn Host, in_game: bool) {
        use crate::coroutine_ticker::WaitOwner;
        let ticker = self.state().behaviors.borrow().coroutine_ticker.clone();
        let Some(mut batch) = ticker.begin_tick() else {
            return;
        };
        while let Some(task) = batch.next_task() {
            // Earlier continuations can synchronously change the world. Each
            // callback therefore reads its current guard at its FIFO position.
            let in_game = in_game
                && host.client_state().is_some_and(|client| {
                    client.world.is_some() && client.logged_in && client.eye_position.is_some()
                });
            let keep = match task.owner {
                WaitOwner::Derp => self.derp_callback(host, in_game, task.callback),
                WaitOwner::AntiAfk => self.anti_afk_callback(host, in_game, task.callback),
                WaitOwner::AutoFish => self.auto_fish_callback(host, in_game, task.callback),
                WaitOwner::SmartEat => self.smart_eat_callback(host, in_game, task.callback),
                WaitOwner::Timer => self.timer_callback(host, in_game, task.callback),
                WaitOwner::NoWeb => self.no_web_callback(host,in_game,task.callback),
                WaitOwner::VehicleControl=>self.vehicle_control_callback(host,in_game,task.callback),
                WaitOwner::FastExp=>self.fast_exp_callback(host,in_game,task.callback),
                WaitOwner::AutoBuff=>self.auto_buff_callback(host,in_game,task.callback),
                WaitOwner::AutoPearl=>self.auto_pearl_callback(host,in_game,task.callback),
                WaitOwner::AutoRod=>self.auto_rod_callback(host,in_game,task.callback),
                WaitOwner::NoSlow => self.no_slow_callback(host,in_game,task.callback),
                WaitOwner::AutoDeposit=>self.auto_deposit_callback(host,in_game,task.callback),
                WaitOwner::Replenish=>self.replenish_callback(host,in_game,task.callback),
                WaitOwner::AutoCrafter=>self.auto_crafter_callback(host,in_game,task.callback),
                WaitOwner::AutoArmor=>self.auto_armor_callback(host,in_game,task.callback),
                WaitOwner::AutoMobHeal=>self.auto_mob_heal_callback(in_game,task.callback),
                WaitOwner::HighJump => self.high_jump_callback(host, in_game, task.callback),
                WaitOwner::Scaffold=>self.scaffold_wait(in_game),
                WaitOwner::ScaffoldTower=>self.scaffold_tower_wait(host,in_game),
                WaitOwner::Step => self.step_callback(host,in_game,task.callback),
                WaitOwner::ResetVl => self.reset_vl_callback(host, in_game, task.callback),
                WaitOwner::AutoAccount => self.auto_account_callback(host, task.callback),
                WaitOwner::ReportHelperReport=>self.report_helper_report_callback(host,in_game,task.callback),
                WaitOwner::ReportHelperConfirm=>self.report_helper_confirm_callback(host,in_game,task.callback),
                WaitOwner::AutoQueue=>self.auto_queue_callback(host,in_game,task.callback),
                WaitOwner::FastUse => self.fast_use_callback(host, in_game, task.callback),
                WaitOwner::Clip => self.clip_callback(host, in_game, task.callback),
                WaitOwner::AutoLeave => self.auto_leave_callback(host, in_game, task.callback),
                WaitOwner::Spider => self.spider_callback(host, in_game, task.callback),
                WaitOwner::SuperKnockback => self.super_knockback_callback(host, in_game, task.callback),
                WaitOwner::AntiBotMatrix=>self.anti_bot_callback(host,in_game,task.callback),
                WaitOwner::TriggerBot => self.crosshair_callback(host,in_game,liquidbounce_behaviors::crosshair_clicker::Id::Trigger,task.callback),
                WaitOwner::AutoClickerAttack => self.crosshair_callback(host,in_game,liquidbounce_behaviors::crosshair_clicker::Id::Attack,task.callback),
                WaitOwner::AutoClickerUse => self.crosshair_callback(host,in_game,liquidbounce_behaviors::crosshair_clicker::Id::Use,task.callback),
                WaitOwner::SkinDerp => self.skin_derp_callback(host, in_game, task.callback),
                WaitOwner::InventoryManager => {
                    self.inventory_callback(host, in_game, task.callback)
                }
                WaitOwner::HandDerp => {
                    let (keep, hand) = {
                        let state = self.state();
                        let modules = state.behaviors.borrow();
                        modules.hand_derp.initialize();
                        let running = in_game && modules.query_running(&state.registry, "HandDerp");
                        modules
                            .hand_derp
                            .behavior
                            .lock()
                            .expect("HandDerp poisoned")
                            .advance_callback(task.ticket, running)
                    };
                    if let Some(hand) = hand {
                        host.client_hand_derp_send(hand.id());
                    }
                    keep
                }
                WaitOwner::AutoRespawn => {
                    let (keep, respawn) = {
                        let state = self.state();
                        let mut modules = state.behaviors.borrow_mut();
                        let running =
                            in_game && modules.query_running(&state.registry, "AutoRespawn");
                        modules
                            .auto_respawn
                            .advance_callback(task.callback, running)
                    };
                    if respawn && host.client_is_death_screen() {
                        host.client_respawn();
                    }
                    keep
                }
            };
            batch.finish_current(keep);
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn reset_rotations(&self) {
        horizon_client::inventory_move::clear();
        self.reset_inventory();
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        modules
            .rotations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .reset_world();
        modules.directional_input = Default::default();
    }
    pub(crate) fn state(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn observed_state(&self) -> ObservedState {
        self.state().observed
    }

    pub(crate) fn observe_world(
        state: &mut State,
        client: &NativeClientState,
        options: RuntimeOptions,
    ) {
        let world = WorldState {
            world_present: client.world.is_some(),
            player_present: client.logged_in && client.eye_position.is_some(),
        };
        #[cfg(feature = "behavior-hooks")]
        if state.observed.world.in_game() && !world.in_game() {
            let mut modules = state.behaviors.borrow_mut();
            modules
                .rotations
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .reset_world();
            modules.click_gui.disconnected();
            modules.directional_input = Default::default();
            modules.middle_click_action.world_change();
            modules.anti_bot.state.reset();modules.notifier.bot_entities.clear();
            modules.combat_cooldown.world_change();
            modules.report_helper.disabled();
            modules.criticals.disconnect();
            modules.inventory.world_change();
            modules.fast_use.cancel();
            modules.ping_spoof.world_change();
            modules.clip.cancel();
            modules.auto_tool.swaps.cancel_plan();
            modules.inventory_packets.world_change();
        }
        state.observed.world = world;
        for report in state.with_registry(options, |registry, callbacks| {
            registry.update_world_sync(world, callbacks)
        }) {
            log_report(&report);
        }
    }

    fn feed_keyboard(
        &self,
        host: &mut dyn Host,
        code: horizon_client::mod_api::KeyCode,
        action: KeyAction,
        modifiers: u32,
    ) {
        let Some(client) = host.client_state() else {
            return;
        };
        let Some(scan) = input::keyboard_scan_code(code) else {
            return;
        };
        let mut keyboard = KeyboardInput::new(scan, action, input::modifiers(modifiers));
        if action == KeyAction::Release {
            keyboard.released_modifier = input::physical_modifier(code);
        }
        let context = InputContext {
            screen_open: !matches!(client.screen, NativeScreenKind::Playing),
            debug_modifier_down: debug_modifier_down(host),
        };
        let options = raw_options(host);
        #[cfg(feature = "behavior-hooks")]
        let snap_tap_mask = host.client_snap_tap_key_mask(code);
        let effects = {
            let mut state = self.state();
            Self::observe_world(&mut state, &client, options);
            #[cfg(feature = "behavior-hooks")]
            {
                use liquidbounce_behaviors::snap_tap::Action;
                let mut modules = state.behaviors.borrow_mut();
                let running = modules.query_running(&state.registry, "SnapTap");
                modules.snap_tap.keyboard(
                    running,
                    snap_tap_mask,
                    match action {
                        KeyAction::Press => Action::Press,
                        KeyAction::Release => Action::Release,
                        KeyAction::Repeat => Action::Repeat,
                    },
                    crate::snap_tap::nano_time,
                );
            }
            state.observed.keyboard_events = state.observed.keyboard_events.saturating_add(1);
            log_reports(state.with_registry(options, |registry, callbacks| {
                registry.handle_keyboard_sync(keyboard, context, callbacks)
            }));
            #[cfg(feature = "behavior-hooks")]
            if action == KeyAction::Release {
                let changed = {
                    let mut modules = state.behaviors.borrow_mut();
                    let running = modules.query_running(&state.registry, "CameraClip");
                    modules.camera_clip.keyboard_release(running, scan)
                };
                #[cfg(feature = "webview")]
                if changed {
                    state.camera_distance_changed();
                }
            }
            state.take_effects()
        };
        apply_effects(self, host, effects);
        #[cfg(feature="behavior-hooks")]
        self.macros_key(host,code);
        #[cfg(feature="behavior-hooks")]
        if action==KeyAction::Press{self.offhand_key(host,code);}
    }
}

/// The old launcher's required entry point. This is a linked Rust library,
/// neither a signed SDK dylib nor a prebuilt game executable.
pub fn hooks() -> Box<dyn Hooks> {
    Box::new(LiquidBounceHooks::default())
}

impl Hooks for LiquidBounceHooks {
    #[cfg(feature = "behavior-hooks")]
    fn render(&self) -> Option<&dyn horizon_client::mod_api::RenderHooks> {
        Some(self)
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_block_interact(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientBlockInteractEvent,
    ) {
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        if modules.query_running(&state.registry, "NoBlockInteract") && event.should_sneak {
            modules.no_block_interact.start_sneaking();
            event.canceled = true;
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_entity_pick(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientEntityPickEvent<'_>,
    ) {
        #[allow(unused_mut)]
        let mut state = self.state();
        let original = event.allowed;
        {
            let modules = state.behaviors.borrow();
            event.allowed &= modules.no_entity_interact.test(
                modules.query_running(&state.registry, "NoEntityInteract"),
                event.main_hand_item,
            );
        }
        #[cfg(feature = "webview")]
        state.probe(
            "clientEntityPick",
            serde_json::json!({"mainHandItem":event.main_hand_item,"allowed":original}),
            serde_json::json!(event.allowed),
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_no_pose_policy(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientNoPosePolicyEvent,
    ) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        (event.no_swim, event.sneak_height) = modules
            .no_pose
            .policy(modules.query_running(&state.registry, "NoPose"));
    }

    #[cfg(feature="webview")]
    fn framerate_limit(&self,host:&dyn Host,event:&mut horizon_client::mod_api::FramerateLimitEvent){self.click_gui_limit(host,event);}

    #[cfg(feature = "behavior-hooks")]
    fn client_level_load(
        &self,
        host: &mut dyn Host,
        _event: &mut horizon_client::mod_api::ClientLevelLoadEvent<'_>,
    ) {
        self.state().behaviors.borrow_mut().click_gui.level_loaded(self.started.elapsed().as_millis().min(u64::MAX as u128)as u64);
        if host.client_state().is_some() {
            self.reset_rotations();
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_level_unload(
        &self,
        host: &mut dyn Host,
        _event: &mut horizon_client::mod_api::ClientLevelUnloadEvent<'_>,
    ) {
        if host.client_state().is_some() {
            self.reset_rotations();
        }
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_airjump_push_queries(
        &self,
        _host: &dyn Host,
        event: &mut ClientAirjumpPushQueriesEvent,
    ) {
        use liquidbounce_behaviors::{air_jump::AirJumpMode, no_push::PushBy};
        let state = self.state();
        let modules = state.behaviors.borrow();
        let jumping = modules.query_running(&state.registry, "AirJump");
        let pushing = modules.query_running(&state.registry, "NoPush");
        event.air_jump = modules.air_jump.allow_jump(jumping);
        event.double_jump_mode = jumping && modules.air_jump.mode == AirJumpMode::DoubleJump;
        event.ghost_block =
            jumping && modules.air_jump.mode == AirJumpMode::GhostBlock && event.physical_jump;
        event.allow_entities = modules.no_push.can_push(pushing, PushBy::Entities);
        event.allow_blocks = modules.no_push.can_push(pushing, PushBy::Blocks);
        event.allow_fishing = modules.no_push.can_push(pushing, PushBy::FishingRod);
        event.allow_liquids = modules.no_push.can_push(pushing, PushBy::Liquids);
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_player_jump(&self, host: &dyn Host, event: &mut ClientPlayerJumpEvent) {
        self.speed_jump(host, event);
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        let running = modules.query_running(&state.registry, "AirJump");
        modules.air_jump.player_jump(running, event.on_ground);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_mining_conditions(&self, _host: &dyn Host, event: &mut ClientMiningConditionsEvent) {
        let mut state = self.state();
        let before = serde_json::json!({"fatigue":event.fatigue,"underwater":event.water,"onGround":event.on_ground});
        {
            let modules = state.behaviors.borrow();
            modules.no_slow_break.apply(
                modules.query_running(&state.registry, "NoSlowBreak"),
                &mut event.fatigue,
                &mut event.water,
                &mut event.on_ground,
            );
        }
        #[cfg(feature = "webview")]
        state.probe("clientMiningConditions", before, serde_json::json!({"fatigue":event.fatigue,"underwater":event.water,"onGround":event.on_ground}));
        #[cfg(not(feature = "webview"))]
        let _ = before;
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_visual_environment_policy(
        &self,
        _host: &dyn Host,
        event: &mut ClientVisualEnvironmentPolicyEvent,
    ) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        (
            event.render_mask,
            event.fire_opacity,
            event.hide_particles,
            event.skip_terrain_loading,
        ) = modules
            .anti_blind
            .policy(modules.query_running(&state.registry, "AntiBlind"));
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_crystal_view(&self, _host: &dyn Host, event: &mut ClientCrystalViewEvent) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        let [size, y, spin, bounce] = modules
            .crystal_view
            .parameters(modules.query_running(&state.registry, "CrystalView"));
        event.size = size;
        event.y_translate = y;
        event.spin_speed = spin;
        event.bounce = bounce;
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_use_cooldown(&self, _host: &dyn Host, event: &mut ClientUseCooldownEvent) {
        use liquidbounce_behaviors::fast_place::ItemClasses;
        let state = self.state();
        let modules = state.behaviors.borrow();
        let classify = |kind: u8| ItemClasses {
            projectile: kind & 1 != 0,
            block: kind & 2 != 0,
        };
        let since_press = modules
            .press_times
            .since_use_press(event.mouse_button, behaviors::current_time_millis());
        let result = modules.fast_place.use_cooldown(
            modules.query_running(&state.registry, "FastPlace"),
            [classify(event.main_kind), classify(event.off_kind)],
            since_press,
            event.cooldown,
            |from, to| {
                if from <= to {
                    Ok(liquidbounce_behaviors::random::random_i32_inclusive(
                        from, to,
                    ))
                } else {
                    Err("empty cooldown range")
                }
            },
        );
        match result {
            Ok(value) => event.cooldown = value,
            Err(error) => eprintln!("[LiquidBounce] FastPlace: {error}"),
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_death_screen_opening(
        &self,
        host: &mut dyn Host,
        _event: &mut ClientDeathScreenOpeningEvent,
    ) {
        let client = host.client_state();
        let options = raw_options(host);
        let respawn = {
            let mut state = self.state();
            if let Some(client) = client.as_ref() {
                Self::observe_world(&mut state, client, options);
            }
            let mut modules = state.behaviors.borrow_mut();
            let running = modules.query_running(&state.registry, "AutoRespawn") && client.is_some();
            let respawn = modules.auto_respawn.screen_event(running, true);
            for callback in modules.auto_respawn.take_wait_registrations() {
                modules
                    .coroutine_ticker
                    .register(crate::coroutine_ticker::WaitOwner::AutoRespawn, callback);
            }
            respawn
        };
        // No registry or behavior borrow may survive the ordinary respawn path.
        if respawn {
            host.client_respawn();
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_keep_sprint_attack(&self, _host: &dyn Host, event: &mut ClientKeepSprintAttackEvent) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        event.sprinting = modules.keep_sprint.attack_sprinting(
            modules.query_running(&state.registry, "KeepSprint"),
            event.sprinting,
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_keep_sprint_motion(&self, _host: &dyn Host, event: &mut ClientKeepSprintMotionEvent) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        if modules.query_running(&state.registry, "KeepSprint") {
            event.multiplier = modules.keep_sprint.motion_multiplier(
                event.hurt_time,
                liquidbounce_behaviors::random::random_unit_f32,
            );
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_keep_sprint_set(&self, _host: &dyn Host, event: &mut ClientKeepSprintSetEvent) {
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        let running = modules.query_running(&state.registry, "KeepSprint");
        event.apply = modules.keep_sprint.set_sprinting(running, event.requested);
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_better_inventory_render(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientBetterInventoryRenderEvent<'_>,
    ) {
        if self.chest_stealer_render(_host,event.query,event.draw){return;}
        self.better_inventory_render(event.query, event.draw);
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_step_height(&self,event:&mut horizon_client::mod_api::ClientStepHeightEvent){self.state().behaviors.borrow().step.height(&mut event.height);}
    #[cfg(feature = "behavior-hooks")]
    fn client_step_success(&self,event:&mut horizon_client::mod_api::ClientStepSuccessEvent){*event.result=self.state().behaviors.borrow().step.success(event.adjusted_y,event.player_y);}
    #[cfg(feature = "behavior-hooks")]
    fn client_step_input(&self,_host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientStepInputEvent){self.step_input([event.can_one,event.can_one_quarter,event.can_one_half],&mut event.jump,&mut event.effects);}
    #[cfg(feature = "behavior-hooks")]
    fn client_no_web_collision(&self,event:&mut horizon_client::mod_api::ClientNoWebCollisionEvent){self.no_web_collision(event.pos,event.moving,event.ground,event.tick,event.result);}
    #[cfg(feature = "behavior-hooks")]
    fn client_no_slow_network(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientNoSlowNetworkEvent){self.no_slow_network(host,event.pre);}
    #[cfg(feature = "behavior-hooks")]
    fn client_high_jump(&self, event: &mut horizon_client::mod_api::ClientHighJumpEvent) {
        #[cfg(feature = "webview")]
        let original = event.motion;
        self.state().behaviors.borrow().criticals.jump(&mut event.motion);
        self.high_jump_jump(&mut event.motion);
        #[cfg(feature = "webview")]
        self.state().probe(
            "clientHighJump",
            serde_json::json!(original),
            serde_json::json!(event.motion),
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_item_scroller_query(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientItemScrollerQueryEvent,
    ) {
        self.item_scroller_query(event.shift, event.left_pressed, &mut event.allowed);
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_gui_closer_screen(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientGuiCloserScreenEvent<'_>,
    ) {
        self.gui_closer_screen(event.title, &mut event.cancel, &mut event.print_title);
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_item_scroller_reset(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientItemScrollerResetEvent,
    ) {
        self.item_scroller_reset(event.now);
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_auto_break_key(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientAutoBreakKeyEvent,
    ) {
        self.auto_break_key(
            event.attack,
            event.miss_time,
            event.block_hit,
            event.block_state_available,
            event.air,
            event.destroying,
            &mut event.pressed,
            &mut event.click,
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_terrain_speed(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientTerrainSpeedEvent,
    ) {
        use liquidbounce_behaviors::terrain_speed::ClimbMode;
        #[allow(unused_mut)]
        let mut state = self.state();
        let modules = state.behaviors.borrow();
        let value = modules.terrain_speed;
        event.running = modules.query_running(&state.registry, "TerrainSpeed");
        event.flags = u32::from(value.fast_climb.enabled)
            | (u32::from(value.ice_speed.enabled) << 1)
            | (u32::from(value.ice_speed.motion_enabled) << 2)
            | (u32::from(value.water_speed.enabled) << 3)
            | (u32::from(value.water_speed.auto_swim) << 4)
            | (u32::from(value.water_speed.sprint_enabled) << 5);
        event.mode = u8::from(value.fast_climb.mode == ClimbMode::Clip);
        event.climb_motion = value.fast_climb.motion;
        event.slipperiness = value.ice_speed.slipperiness;
        event.ice_motion = value.ice_speed.horizontal_motion;
        event.base_horizontal = value.water_speed.base_horizontal;
        event.base_vertical = value.water_speed.base_vertical;
        event.sprint_horizontal = value.water_speed.sprint_horizontal;
        event.sprint_vertical = value.water_speed.sprint_vertical;
        let input = modules.directional_input;
        event.input = u8::from(input.forwards)
            | (u8::from(input.backwards) << 1)
            | (u8::from(input.left) << 2)
            | (u8::from(input.right) << 3);
        #[cfg(feature = "webview")]
        {
            let output = serde_json::json!({"running":event.running,"flags":event.flags,"mode":event.mode,
                "input":event.input,"climbMotion":event.climb_motion,"slipperiness":event.slipperiness,
                "iceMotion":event.ice_motion,"baseHorizontal":event.base_horizontal,"baseVertical":event.base_vertical,
                "sprintHorizontal":event.sprint_horizontal,"sprintVertical":event.sprint_vertical});
            drop(modules);
            state.probe("clientTerrainSpeed", serde_json::Value::Null, output);
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_auto_disable_flag(
        &self,
        host: &mut dyn Host,
        _event: &mut horizon_client::mod_api::ClientAutoDisableFlagEvent,
    ) {
        self.auto_disable_drain(host);
        self.no_clip_drain(host);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_auto_disable_death(
        &self,
        host: &mut dyn Host,
        _event: &mut horizon_client::mod_api::ClientAutoDisableDeathEvent,
    ) {
        self.auto_disable_drain(host);
        self.auto_disable_event(host, liquidbounce_behaviors::auto_disable::Reason::Death);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_auto_disable_lifecycle(
        &self,
        host: &mut dyn Host,
        event: &mut horizon_client::mod_api::ClientAutoDisableLifecycleEvent,
    ) {
        self.auto_disable_drain(host);
        let reason = if event.reason == 2 {
            liquidbounce_behaviors::auto_disable::Reason::WorldChange
        } else {
            liquidbounce_behaviors::auto_disable::Reason::Disconnect
        };
        self.auto_disable_event(host, reason);
        if event.reason == 3 {
            self.state().behaviors.borrow().plugins.disconnected();
        }
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_auto_disable_command(
        &self,
        host: &mut dyn Host,
        event: &mut horizon_client::mod_api::ClientAutoDisableCommandEvent<'_>,
    ) {
        event.handled |= self.auto_disable_command(host, event.message);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_true_sight_entity(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientTrueSightEntityEvent,
    ) {
        self.true_sight_entity(host, event);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_true_sight_markers(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientTrueSightMarkersEvent,
    ) {
        self.true_sight_markers(host, event);
    }
    #[cfg(feature = "behavior-hooks")]
    fn interaction_key(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::InteractionKeyEvent<'_>) -> horizon_client::mod_api::Flow {
        if event.button==2 && self.middle_click_action_cancel_pick(host) {
            horizon_client::mod_api::Flow::Cancel
        } else {
            horizon_client::mod_api::Flow::Continue
        }
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_auto_account_message(
        &self,
        host: &mut dyn Host,
        event: &mut horizon_client::mod_api::ClientAutoAccountMessageEvent<'_>,
    ) {
        self.auto_account_message(event.source, event.message);
        if event.source==0{self.report_helper_message(host,event.message);}
        if event.source==1||event.source==2{self.auto_queue_message(host,event.source,0,event.message);}
    }
    #[cfg(feature="behavior-hooks")]
    fn client_auto_queue_message(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientAutoQueueMessageEvent<'_>){self.auto_queue_message(host,0,event.kind,event.message);}
    #[cfg(feature="behavior-hooks")]
    fn client_macros_wake(&self,host:&mut dyn Host,_event:&mut horizon_client::mod_api::ClientMacrosWakeEvent){self.macros_wake(host);}
    #[cfg(feature="behavior-hooks")]
    fn client_auto_chat_game_wake(&self,host:&mut dyn Host,_event:&mut horizon_client::mod_api::ClientAutoChatGameWakeEvent){self.auto_chat_game_poll(host,false);}
    #[cfg(feature="behavior-hooks")]
    fn client_commands_chat(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientCommandsChatEvent<'_>){event.handled|=self.client_command_message(host,event.message);}
    #[cfg(feature="behavior-hooks")]
    fn client_auto_queue_wake(&self,host:&mut dyn Host,_event:&mut horizon_client::mod_api::ClientAutoQueueWakeEvent){self.auto_queue_wake(host);}
    #[cfg(feature="behavior-hooks")]
    fn client_report_helper_screen(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientReportHelperScreenEvent){self.report_helper_screen(host,event.token);}
    #[cfg(feature = "behavior-hooks")]
    fn client_auto_account_command(
        &self,
        host: &mut dyn Host,
        event: &mut horizon_client::mod_api::ClientAutoAccountCommandEvent<'_>,
    ) {
        event.handled |= self.auto_account_command(host, event.message);
    }


    #[cfg(feature = "behavior-hooks")]
    fn client_plugins_game_message(
        &self,
        host: &mut dyn Host,
        event: &mut horizon_client::mod_api::ClientPluginsGameMessageEvent<'_>,
    ) {
        self.auto_chat_game_message(event.message);
        self.auto_chat_game_schedule(host);
        self.plugins_game_message(host, event.message);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_plugins_server_commands(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientPluginsServerCommandsEvent<'_>,
    ) {
        self.plugins_server_commands(event.names);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_blink_movement(&self,host:&mut dyn Host,_event:&mut horizon_client::mod_api::ClientBlinkMovementEvent){self.blink_movement(host);}
    #[cfg(feature = "behavior-hooks")]
    fn client_freeze_movement(&self,event:&mut horizon_client::mod_api::ClientFreezeMovementEvent){event.cancelled|=self.state().behaviors.borrow().freeze.cancel_movement();}
    #[cfg(feature="behavior-hooks")]
    fn client_sword_block_state(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientSwordBlockStateEvent){event.flags=self.sword_block.flags();}
    #[cfg(feature="behavior-hooks")]
    fn client_fake_lag_capture(&self,host:&dyn Host,_event:&mut horizon_client::mod_api::ClientFakeLagCaptureEvent){self.fake_lag_refresh(host,false);}
    #[cfg(feature="behavior-hooks")]
    fn client_criticals_receive_probe(&self,host:&mut dyn Host,_event:&mut horizon_client::mod_api::ClientCriticalsReceiveProbeEvent){self.criticals_receive(host);}
    #[cfg(feature = "behavior-hooks")]
    fn client_freeze_frame(&self,host:&mut dyn Host,_event:&mut horizon_client::mod_api::ClientFreezeFrameEvent){self.xray_frame(host);self.anti_staff_frame(host);self.blink_pending(host);self.no_slow_frame(host);self.freeze_frame(host);self.fake_lag_refresh(host,false);self.criticals_frame(host);}
    #[cfg(feature = "behavior-hooks")]
    fn client_freeze_flag(&self,host:&mut dyn Host,_event:&mut horizon_client::mod_api::ClientFreezeFlagEvent){self.freeze_frame(host);}
    #[cfg(feature = "behavior-hooks")]
    fn client_spider_jump(&self, event: &mut horizon_client::mod_api::ClientSpiderJumpEvent) {
        self.spider_jump(event.horizontal_collision, &mut event.motion);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_spider_shape_active(
        &self,
        event: &mut horizon_client::mod_api::ClientSpiderShapeActiveEvent,
    ) {
        event.active |= self.spider_shape_active();
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_mob_owners(&self, host: &dyn Host, event: &mut horizon_client::mod_api::ClientMobOwnersEvent) {
        self.mob_owners(host, event);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_combine_mobs_frame(&self, host: &dyn Host, event: &mut horizon_client::mod_api::ClientCombineMobsFrameEvent) {
        self.combine_mobs_frame(host, event);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_combine_mobs(&self, host: &dyn Host, event: &mut horizon_client::mod_api::ClientCombineMobsEvent<'_>) {
        self.combine_mobs(host, event);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_better_tab(&self, host: &dyn Host, event: &mut horizon_client::mod_api::ClientBetterTabEvent<'_>) {
        self.better_tab(host, event);
        self.anti_staff_tab(event);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_block_outline(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientBlockOutlineEvent<'_>){
        self.block_outline(host,event);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_breadcrumbs(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientBreadcrumbsEvent<'_>){
        self.breadcrumbs(host,event);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_damage_particles(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientDamageParticlesEvent<'_>){self.damage_particles(host,event);}
    #[cfg(feature = "behavior-hooks")]
    fn client_tracers(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientTracersEvent<'_>){self.tracers(host,event);}
    #[cfg(feature = "behavior-hooks")]
    fn client_new_chunks(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientNewChunksEvent<'_>){self.new_chunks(host,event);}
    #[cfg(feature = "behavior-hooks")]
    fn client_placement_esp(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientPlacementEspEvent<'_>){self.placement_esp(host,event);}
    #[cfg(feature = "behavior-hooks")]
    fn client_protection_zones(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientProtectionZonesEvent<'_>){self.protection_zones(host,event);}
    #[cfg(feature = "behavior-hooks")]
    fn client_hats(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientHatsEvent<'_>){self.hats(host,event);}
    #[cfg(feature = "behavior-hooks")]
    fn client_wings(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientWingsEvent<'_>){self.wings(host,event);}
    #[cfg(feature = "behavior-hooks")]
    fn client_crosshair(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientCrosshairEvent<'_>){self.crosshair(host,event);}
    #[cfg(feature="behavior-hooks")]
    fn client_murder_mystery(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientMurderMysteryEvent<'_>){self.murder_mystery(host,event);}
    #[cfg(feature="behavior-hooks")]
    fn client_blink_esp(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientBlinkEspEvent<'_>){self.blink_esp(host,event);}
    #[cfg(feature="behavior-hooks")]
    fn client_hole_esp(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientHoleEspEvent<'_>){self.hole_esp(host,event);}
    #[cfg(feature="behavior-hooks")]
    fn client_particles(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientParticlesEvent<'_>){self.particles(host,event);}
    #[cfg(feature="behavior-hooks")]
    fn client_logoff_spot(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientLogoffSpotEvent<'_>){self.logoff_spot(host,event);}
    #[cfg(feature="behavior-hooks")]
    fn client_inventory_tracker(&self,_host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientInventoryTrackerEvent<'_>){self.inventory_tracker_event(event.input,event.output);}
    #[cfg(feature="behavior-hooks")]
    fn client_notifier(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientNotifierEvent<'_>){self.notifier_event(host,event);}
    #[cfg(feature="behavior-hooks")]
    fn client_anti_bot(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientAntiBotEvent<'_>){self.anti_bot_packet(host,event);}
    #[cfg(feature="behavior-hooks")]
    fn client_block_placer_success(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientBlockPlacerSuccessEvent){self.block_placer_success(event.requester,event.position,[0.,0.,0.,1.,1.,1.]);}
    #[cfg(feature="behavior-hooks")]
    fn client_block_placer_after_move(&self,host:&mut dyn Host,_event:&mut horizon_client::mod_api::ClientBlockPlacerAfterMoveEvent){self.block_placer_after_move(host);}
    #[cfg(feature="behavior-hooks")]
    fn client_block_placer_removed(&self,_host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientBlockPlacerRemovedEvent){self.block_placer_removed(event.id);}
    #[cfg(feature="behavior-hooks")]
    fn client_bed_defender(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientBedDefenderEvent<'_>){self.bed_defender_event(event);}
    #[cfg(feature="behavior-hooks")]
    fn client_hole_filler(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientHoleFillerEvent<'_>){self.hole_filler_event(event);}
    #[cfg(feature="behavior-hooks")]
    fn client_extinguish_success(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientExtinguishSuccessEvent){self.extinguish_success(event.position);}
    #[cfg(feature="behavior-hooks")]
    fn client_anti_void_shape(&self,event:&mut horizon_client::mod_api::ClientAntiVoidShapeEvent){self.anti_void_shape(&mut event.active,&mut event.floor);}
    #[cfg(feature="behavior-hooks")]
    fn client_anti_void_network(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientAntiVoidNetworkEvent){self.anti_void_network(host,event.pre,&mut event.y);}
    #[cfg(feature="behavior-hooks")]
    fn client_storage_esp(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientStorageEspEvent<'_>){self.storage_esp(host,event);}
    #[cfg(feature="behavior-hooks")]
    fn client_block_esp(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientBlockEspEvent<'_>){self.block_esp(event);}
    #[cfg(feature="behavior-hooks")]
    fn client_target_renderer(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientTargetRendererEvent<'_>){self.target_renderer(host,event);}
    #[cfg(feature="behavior-hooks")]
    fn client_aimbot_render(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientAimbotRenderEvent){self.aimbot_render(host,event);}
    #[cfg(feature="behavior-hooks")]
    fn client_easy_pearl(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientEasyPearlEvent<'_>){self.easy_pearl(host,event);}
    #[cfg(feature="behavior-hooks")]
    fn client_jump_effect_jump(&self,event:&mut horizon_client::mod_api::ClientJumpEffectJumpEvent){self.jump_effect_jump(event.position);}
    #[cfg(feature="behavior-hooks")]
    fn client_jump_effect(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientJumpEffectEvent<'_>){self.jump_effect(event);}
    #[cfg(feature="behavior-hooks")]
    fn client_hit_fx(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientHitFxEvent<'_>){self.hit_fx(host,event);}
    #[cfg(feature="behavior-hooks")]
    fn client_totem_effect(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientTotemEffectEvent<'_>){self.totem_effect(host,event);}
    #[cfg(feature="behavior-hooks")]
    fn client_stronghold_finder(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientStrongholdFinderEvent<'_>){self.stronghold_finder(event);}
    #[cfg(feature="behavior-hooks")]
    fn client_potion_fx(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientPotionFxEvent<'_>){self.potion_fx(host,event);}
    #[cfg(feature="behavior-hooks")]
    fn client_tnt_timer(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientTntTimerEvent<'_>){self.tnt_timer(host,event);}
    #[cfg(feature="behavior-hooks")]
    fn client_radar(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientRadarEvent<'_>){self.radar(event);}
    #[cfg(feature="behavior-hooks")]
    fn client_custom_ambience(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientCustomAmbienceEvent<'_>){self.custom_ambience(host,event);}
    #[cfg(feature="behavior-hooks")]
    fn client_bed_plates(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientBedPlatesEvent<'_>){self.bed_plates(event);}
    #[cfg(feature="behavior-hooks")]
    fn client_debug(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientDebugEvent<'_>){self.debug(event);}
    #[cfg(feature="behavior-hooks")]
    fn client_nametags(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientNametagsEvent<'_>){self.nametags(event);}
    #[cfg(feature="behavior-hooks")]
    fn client_better_chat(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientBetterChatEvent<'_>){self.better_chat(event);}
    #[cfg(feature="behavior-hooks")]
    fn client_item_tags(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientItemTagsEvent<'_>){self.item_tags(event);}
    #[cfg(feature="behavior-hooks")]
    fn client_esp(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientEspEvent<'_>){self.esp(host,event);}
    #[cfg(feature="behavior-hooks")]
    fn client_item_esp(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientItemEspEvent<'_>){self.item_esp(event);}
    #[cfg(feature="behavior-hooks")]
    fn client_item_chams(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientItemChamsEvent<'_>){self.item_chams(event);}
    #[cfg(feature="behavior-hooks")]
    fn client_trajectories(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientTrajectoriesEvent<'_>){self.trajectories(event);}
    #[cfg(feature="behavior-hooks")]
    fn client_free_cam(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientFreeCamEvent<'_>){self.free_cam(event.input,event.output);}
    #[cfg(feature="behavior-hooks")]
    fn client_free_cam_policy(&self,event:&mut horizon_client::mod_api::ClientFreeCamPolicyEvent){self.free_cam_policy(event.kind,event.x,event.z,&mut event.flags);}


    #[cfg(feature = "behavior-hooks")]
    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {
        {let s=self.state();let mut b=s.behaviors.borrow_mut();b.scaffold.reset();b.clutch.guarding=false;b.clutch.path=serde_json::Value::Null;b.scaffold.blink.state.lock().expect("Scaffold Blink").running=false;}
        host.client_reset_silent_slot(crate::scaffold_runtime::REQUESTER);
        if event.dimension.is_none(){self.state().behaviors.borrow().name_protect.set_running(false);}
        self.state().behaviors.borrow_mut().inventory_tracker.model.reset();
        self.state().behaviors.borrow_mut().auto_crafter.cancel();
        self.state().behaviors.borrow().logoff_spot.clear();
        self.state().behaviors.borrow_mut().notifier.model.world_change();
        self.anti_bot_reset();
        self.state().behaviors.borrow_mut().particles.clear();
        self.auto_queue_world_change();
        self.hole_filler_world_change();
        self.bed_defender_world_change();
        {let s=self.state();let mut b=s.behaviors.borrow_mut();b.liquid_filler.placer.reset_world();b.block_placer_executor.clear();}
        self.state().behaviors.borrow_mut().auto_mob_heal.clear();
        self.state().behaviors.borrow_mut().extinguish.behavior.world_changed();
        self.murder_mystery_reset();
        self.state().behaviors.borrow().no_slow.world_change();
        self.state().behaviors.borrow_mut().no_web.reset();
        host.client_reset_silent_slot(crate::no_web_runtime::REQUESTER);
        if event.dimension.is_none(){self.state().behaviors.borrow().anti_cheat_detect.disconnect();}
        { let state=self.state();let mut m=state.behaviors.borrow_mut();m.prophunt_esp.render.clear();m.void_esp.world_change(); }
        self.state().behaviors.borrow_mut().new_chunks.clear();
        self.state().behaviors.borrow_mut().stronghold_finder.world_change();
        if event.dimension.is_none(){self.state().behaviors.borrow().liquid_walk.clear();}
        if event.dimension.is_none(){self.state().behaviors.borrow().step.clear();}
        self.state().behaviors.borrow_mut().tracers.clear();
        self.state().behaviors.borrow_mut().esp.clear();
        self.state().behaviors.borrow_mut().radar.clear();
        self.state().behaviors.borrow_mut().item_esp.clear();
        self.state().behaviors.borrow_mut().item_tags.clear();
        self.state().behaviors.borrow_mut().damage_particles.clear();
        if event.dimension.is_none(){self.state().behaviors.borrow().freeze.clear();}
        {
            self.state().behaviors.borrow_mut().breadcrumbs.clear();
        }
        {
            let now=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0,|v|v.as_millis() as i64);
            self.state().behaviors.borrow_mut().block_outline.world_change(now);
        }
        self.state()
            .behaviors
            .borrow_mut()
            .middle_click_action
            .world_change();
        if host.client_state().is_some() {
            let timer = self.state().behaviors.borrow().timer.clone();
            timer.world_change();
            self.state().behaviors.borrow().combat_cooldown.world_change();
            #[cfg(feature = "webview")]
            {
                let output = {
                    let state = timer.state.lock().expect("Timer poisoned");
                    serde_json::json!({"balance":state.timer.balance,"burstTicks":state.timer.burst_ticks,
                        "heldTicks":state.timer.held_ticks})
                };
                self.state().probe(
                    "clientWorldChange",
                    serde_json::json!(event.dimension),
                    output,
                );
            }
            #[cfg(not(feature = "webview"))]
            let _ = event;
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_timer_scale(&self, _host: &dyn Host, event: &mut ClientTimerScaleEvent) {
        let original = event.delta;
        #[allow(unused_mut)]
        let mut state = self.state();
        event.delta = state.behaviors.borrow().timer.scale(original);
        #[cfg(feature = "webview")]
        {
            let output = {
                let modules = state.behaviors.borrow();
                let timer = modules.timer.state.lock().expect("Timer poisoned");
                serde_json::json!({"delta":event.delta,"speed":timer.timer.frame_speed,"mode":timer.settings.mode.tag(),"running":modules.timer.running()})
            };
            state.probe("clientTimerScale", serde_json::json!(original), output);
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_timer_frame(&self, _host: &dyn Host, event: &mut ClientTimerFrameEvent) {
        #[allow(unused_mut)]
        let mut state = self.state();
        let (end, held) = state
            .behaviors
            .borrow()
            .timer
            .frame(event.ticks, event.index);
        event.end = end;
        event.held = held;
        #[cfg(feature = "webview")]
        {
            let output = {
                let modules = state.behaviors.borrow();
                let timer = modules.timer.state.lock().expect("Timer poisoned");
                serde_json::json!({"end":end,"held":held,"balance":timer.timer.balance,"burstTicks":timer.timer.burst_ticks,"realTicks":timer.timer.real_ticks})
            };
            state.probe(
                "clientTimerFrame",
                serde_json::json!({"ticks":event.ticks,"index":event.index}),
                output,
            );
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_player_tick_final(&self, _host: &dyn Host, event: &mut ClientPlayerTickFinalEvent) {
        #[allow(unused_mut)]
        let mut state = self.state();
        state
            .behaviors
            .borrow()
            .timer
            .player_tick_final(event.cancelled);
        #[cfg(feature = "webview")]
        {
            let output = {
                let modules = state.behaviors.borrow();
                let timer = modules.timer.state.lock().expect("Timer poisoned");
                serde_json::json!({"balance":timer.timer.balance,"speed":timer.timer.frame_speed,"bursting":timer.timer.bursting})
            };
            state.probe(
                "clientTimerPlayerTickFinal",
                serde_json::json!({"cancelled":event.cancelled}),
                output,
            );
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_tick_pre(&self, host: &mut dyn Host, _event: &mut ClientTickPreEvent) {
        {
            let state = self.state();
            crate::world_noop_runtime::tick_world_noops(&mut state.behaviors.borrow_mut(), &state.registry);
        }
        self.auto_disable_drain(host);
        self.block_placer_first_tick(host); // PostRotationExecutor FIRST_PRIORITY
        self.free_cam_prepare(host);
        self.custom_ambience_tick(); // ExpiringList FIRST_PRIORITY
        self.potion_fx_tick(); // ExpiringList FIRST_PRIORITY
        self.totem_effect_tick(); // ExpiringList FIRST_PRIORITY
        self.jump_effect_tick(); // ExpiringList FIRST_PRIORITY
        self.flag_check_tick(host);
        self.packet_logger_tick(host);
        self.anti_void_tick(host);
        self.anti_bot_tick(host);
        self.notifier_tick(host);
        self.no_slow_tick(host);
        self.auto_disable_drain(host);
        self.no_clip_drain(host);
        self.plugins_tick(host);
        self.anti_cheat_detect_tick(host);
        self.anti_exploit_notifications();
        self.spammer_tick(host);
        self.auto_crafter_tick(host);
        self.auto_chat_game_tick(host);
        self.ping_spoof_tick(host);
        self.clip_tick(host);
        use liquidbounce_behaviors::rotations::{Rotation, modern_mouse_gcd};
        self.state().behaviors.borrow().timer.first_game_tick();
        self.crosshair_first_tick(host); // FIRST_PRIORITY GameTick
        self.projectile_puncher_first_tick();
        self.kill_aura_first_tick();
        self.auto_buff_first_tick();
        self.auto_rod_first_tick(host);
        self.auto_shoot_first_tick();
        self.scaffold_first_tick();
        {
            let mut state = self.state();
            state.target_tags.begin_tick();
            state.combat.tick();
        }
        let client = host.client_state();
        let options = raw_options(host);
        if let Some(client) = client.as_ref() {
            let mut state = self.state();
            Self::observe_world(&mut state, client, options);
        }
        self.name_protect_tick(host);
        self.criticals_tick(host);
        self.middle_click_action_tick(host);
        self.auto_queue_tick(host);
        self.blink_tick(host);
        self.fake_lag_tick(host);
        self.timer_tick(host);
        self.auto_leave_tick(host);
        self.old_regen_tick(host);
        self.fast_use_tick(host);
        self.reset_vl_tick(host);
        self.target_lock_tick(host);
        self.high_jump_tick(host);
        self.liquid_walk_tick(host);
        self.step_tick(host);
        self.reverse_step_tick(host);
        self.no_clip_tick(host);
        self.spider_tick(host);
        self.scaffold_feature_tick(host);
        let Some(client) = client else {
            return;
        };
        #[cfg(feature = "webview")]
        {
            let hand = host.client_fishing_rod_hand();
            let hook = host.client_fishing_hook();
            self.state().probe(
                "autoFishObserved",
                serde_json::json!({"rodHand":hand}),
                serde_json::json!({"hook":hook}),
            );
        }
        self.auto_tool_tick(host);
        self.fast_break_tick(host);
        self.smart_eat_tick(host);
        self.crosshair_tick(host);
        let [yaw, pitch] = host.client_player_rotation();
        let sensitivity = host.client_mouse_sensitivity();
        let inventory_open = host.client_inventory_open();
        let [last_yaw, last_pitch] = host.client_player_last_rotation();
        let [
            on_ground,
            in_liquid,
            physical_jump,
            physical_shift,
            velocity_y,
        ] = host.client_motion_state();
        self.auto_pearl_prepare(host);
        self.extinguish_prepare(host);
        self.easy_pearl_prepare(host);
        self.auto_deposit_prepare(host);
        self.chest_aura_prepare(host);
        self.no_web_prepare(host);
        self.inventory_tracker_flush(host);
        self.projectile_puncher_prepare(host);
        self.elytra_target_prepare(host);
        self.kill_aura_prepare(host);
        self.aimbot_prepare(host);
        self.auto_rod_prepare(host);
        self.auto_shoot_prepare(host);
        self.scaffold_prepare(host);
        self.liquid_filler_prepare(host); // BlockPlacer RotationUpdate priority -20
        self.hole_filler_prepare(host);
        self.bed_defender_prepare(host);
        let reset = {
            let mut state = self.state();
            Self::observe_world(&mut state, &client, options);
            if !state.observed.world.in_game() {
                return;
            }
            let mut modules = state.behaviors.borrow_mut();
            let running = modules.query_running(&state.registry, "AirJump");
            modules.air_jump.game_tick(running, on_ground != 0.0);
            let running = modules.query_running(&state.registry, "Sprint");
            let no_rotate_running = modules.query_running(&state.registry, "NoRotateSet");
            let anti_afk_running = modules.query_running(&state.registry, "AntiAFK");
            let derp_running = modules.query_running(&state.registry, "Derp");
            let auto_walk_running = modules.query_running(&state.registry, "AutoWalk");
            let parkour_running = modules.query_running(&state.registry, "Parkour");
            let no_web_running = modules.query_running(&state.registry,"NoWeb");
            let fast_exp_running=modules.query_running(&state.registry,"FastExp");
            let auto_wind_charge_running=modules.query_running(&state.registry,"AutoWindCharge");
            let projectile_puncher_running=modules.query_running(&state.registry,"ProjectilePuncher");
            let free_cam_running=modules.free_cam.behavior.active(modules.query_running(&state.registry,"FreeCam"));
            let kill_aura_running=modules.query_running(&state.registry,"KillAura");
            let auto_buff_running=modules.query_running(&state.registry,"AutoBuff");
            let auto_bow_running=modules.query_running(&state.registry,"AutoBow");
            let auto_pearl_running=modules.query_running(&state.registry,"AutoPearl");
            let extinguish_running=modules.query_running(&state.registry,"Extinguish");
            let liquid_filler_running=modules.query_running(&state.registry,"LiquidFiller");
            let hole_filler_running=modules.query_running(&state.registry,"HoleFiller");
            let bed_defender_running=modules.query_running(&state.registry,"BedDefender");
            let easy_pearl_running=modules.query_running(&state.registry,"EasyPearl");
            let elytra_target_running=modules.query_running(&state.registry,"ElytraTarget");
            let auto_rod_running=modules.query_running(&state.registry,"AutoRod");
            let auto_shoot_running=modules.query_running(&state.registry,"AutoShoot");
            let scaffold_running=modules.query_running(&state.registry,"Scaffold");
            let auto_deposit_running=modules.query_running(&state.registry,"AutoDeposit");
            let chest_stealer_running=modules.query_running(&state.registry,"ChestStealer");
            let rotation_manager = modules.rotations.clone();
            let mut rotations = rotation_manager
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let projectile_previous_target=rotations.previous_rotation_target();
            let projectile_server=rotations.server_rotation(modules.criticals.is_lagging(),modules.query_running(&state.registry,"Freeze"));
            // FIRST_PRIORITY GameTick consumes the previous tick's request.
            let reset = rotations
                .update_with(
                    Rotation::new(yaw, pitch),
                    modern_mouse_gcd(sensitivity),
                    true,
                    inventory_open,
                    |provider| match provider {
                        1 => running,
                        2 => no_rotate_running,
                        3 => anti_afk_running,
                        4 => derp_running,
                        crate::auto_deposit_runtime::ROTATION_PROVIDER=>auto_deposit_running,
                        crate::chest_stealer_runtime::ROTATION_PROVIDER=>chest_stealer_running,
                        crate::fast_exp_runtime::ROTATION_PROVIDER=>fast_exp_running,
                        crate::auto_wind_charge_runtime::ROTATION_PROVIDER=>auto_wind_charge_running,
                        crate::projectile_puncher_runtime::ROTATION_PROVIDER=>projectile_puncher_running,
                        crate::free_cam_runtime::ROTATION_PROVIDER=>free_cam_running,
                        crate::kill_aura_runtime::ROTATION_PROVIDER=>kill_aura_running,
                        crate::auto_buff_runtime::ROTATION_PROVIDER=>auto_buff_running,
                        crate::auto_bow_runtime::ROTATION_PROVIDER=>auto_bow_running,
                        crate::auto_pearl_runtime::ROTATION_PROVIDER=>auto_pearl_running,
                        crate::extinguish_runtime::ROTATION_PROVIDER=>extinguish_running,
                        crate::block_placer_runtime::PLACEMENT_PROVIDER|crate::block_placer_runtime::CRYSTAL_PROVIDER=>liquid_filler_running,
                        crate::hole_filler_runtime::PLACEMENT_PROVIDER|crate::hole_filler_runtime::CRYSTAL_PROVIDER=>hole_filler_running,
                        crate::bed_defender_runtime::PLACEMENT_PROVIDER|crate::bed_defender_runtime::CRYSTAL_PROVIDER=>bed_defender_running,
                        crate::easy_pearl_runtime::ROTATION_PROVIDER=>easy_pearl_running,
                        crate::elytra_target_runtime::ROTATION_PROVIDER=>elytra_target_running,
                        crate::auto_rod_runtime::ROTATION_PROVIDER=>auto_rod_running,
                        crate::auto_shoot_runtime::ROTATION_PROVIDER=>auto_shoot_running,
                        crate::scaffold_runtime::ROTATION_PROVIDER=>scaffold_running,
                        crate::no_web_runtime::ROTATION_PROVIDER => no_web_running,
                        crate::auto_walk_runtime::ROTATION_PROVIDER => auto_walk_running,
                        crate::edge_assistance_runtime::ROTATION_PROVIDER => parkour_running,
                        _ => false,
                    },
                    |processor, target, from, to, last| {
                        if let Some(id)=processor.checked_sub(crate::free_cam_runtime::PROCESSOR_BASE).filter(|id|liquidbounce_behaviors::combat_rotations::Settings::decode(*id).is_some()){
                            return modules.free_cam.rotations.process(id,free_cam_running,from,to,last,Rotation::new(last_yaw,last_pitch),projectile_previous_target,projectile_server,&mut liquidbounce_behaviors::random::random_unit_f32).map_err(liquidbounce_behaviors::rotations::RotationError::Processor);
                        }
                        if let Some(id)=processor.checked_sub(crate::projectile_puncher_runtime::PROCESSOR_BASE).filter(|id|liquidbounce_behaviors::combat_rotations::Settings::decode(*id).is_some()){
                            modules.projectile_puncher.ai_notice_pending=liquidbounce_behaviors::combat_rotations::Settings::decode(id).is_some_and(|v|v.0==liquidbounce_behaviors::combat_rotations::SmoothMode::Ai);
                            return modules.projectile_puncher.rotations.process(id,projectile_puncher_running,from,to,last,Rotation::new(last_yaw,last_pitch),projectile_previous_target,projectile_server,&mut liquidbounce_behaviors::random::random_unit_f32).map_err(liquidbounce_behaviors::rotations::RotationError::Processor);
                        }
                        if let Some(id)=processor.checked_sub(crate::kill_aura_runtime::PROCESSOR_BASE).filter(|id|liquidbounce_behaviors::combat_rotations::Settings::decode(*id).is_some()){
                            modules.kill_aura.ai_notice_pending=liquidbounce_behaviors::combat_rotations::Settings::decode(id).is_some_and(|v|v.0==liquidbounce_behaviors::combat_rotations::SmoothMode::Ai);
                            let entity=modules.kill_aura.rotation_entity;
                            return modules.kill_aura.rotations.kill_aura_process(id,kill_aura_running,from,to,last,Rotation::new(last_yaw,last_pitch),projectile_previous_target,projectile_server,entity,&mut liquidbounce_behaviors::random::random_unit_f32).map_err(liquidbounce_behaviors::rotations::RotationError::Processor);
                        }
                        if processor==crate::elytra_target_runtime::PROCESSOR{return Ok(modules.elytra_target.behavior.process(from,to,crate::behaviors::current_time_millis()));}
                        if processor == 0 {
                            return Ok(from.towards_linear(
                                to,
                                target.horizontal_turn_speed,
                                target.vertical_turn_speed,
                            ));
                        }
                        if processor == liquidbounce_behaviors::derp::ROTATION_PROCESSOR {
                            return liquidbounce_behaviors::derp::Derp::process_rotation(
                                from,
                                to,
                                last,
                                Rotation::new(last_yaw, last_pitch),
                                &mut liquidbounce_behaviors::random::random_unit_f32,
                            )
                            .map_err(liquidbounce_behaviors::rotations::RotationError::Processor);
                        }
                        let (settings, mode) = if let Some(mode) =
                            liquidbounce_behaviors::no_rotate_set::SmoothMode::from_processor_id(
                                processor,
                            ) {
                            (&mut modules.no_rotate_set.rotations, mode)
                        } else if let Some(mode) =
                            liquidbounce_behaviors::anti_afk::RotationTarget::processor_mode(
                                processor,
                            )
                        {
                            (&mut modules.anti_afk.settings.rotations, mode)
                        } else if let Some(mode)=crate::chest_stealer_runtime::ChestStealerRuntime::processor_mode(processor){
                            (&mut modules.chest_stealer.rotations,mode)
                        } else if let Some(mode)=crate::auto_deposit_runtime::AutoDepositRuntime::processor_mode(processor){
                            (&mut modules.auto_deposit.rotations,mode)
                        } else if let Some(mode)=crate::auto_wind_charge_runtime::AutoWindChargeRuntime::processor_mode(processor){
                            (&mut modules.auto_wind_charge.rotations,mode)
                        } else if let Some(mode)=crate::scaffold_runtime::ScaffoldRuntime::processor_mode(processor){
                            (&mut modules.scaffold.rotations,mode)
                        } else if let Some(mode)=crate::auto_shoot_runtime::AutoShootRuntime::processor_mode(processor){
                            (&mut modules.auto_shoot.rotations,mode)
                        } else if let Some(mode)=crate::auto_rod_runtime::AutoRodRuntime::processor_mode(processor){
                            (&mut modules.auto_rod.rotations,mode)
                        } else if let Some(mode)=crate::auto_bow_runtime::AutoBowRuntime::processor_mode(processor){
                            (&mut modules.auto_bow.rotations,mode)
                        } else if let Some(mode)=crate::easy_pearl_runtime::EasyPearlRuntime::processor_mode(processor){
                            (&mut modules.easy_pearl.rotations,mode)
                        } else if let Some((crystal,mode))=crate::bed_defender_runtime::BedDefenderRuntime::processor_mode(processor){
                            (if crystal{&mut modules.bed_defender.placer.crystal_rotations}else{&mut modules.bed_defender.placer.rotations},mode)
                        } else if let Some((crystal,mode))=crate::hole_filler_runtime::HoleFillerRuntime::processor_mode(processor){
                            (if crystal{&mut modules.hole_filler.placer.crystal_rotations}else{&mut modules.hole_filler.placer.rotations},mode)
                        } else if let Some((crystal,mode))=crate::block_placer_runtime::BlockPlacerRuntime::processor_mode(processor){
                            (if crystal{&mut modules.liquid_filler.placer.crystal_rotations}else{&mut modules.liquid_filler.placer.rotations},mode)
                        } else if let Some(mode)=crate::extinguish_runtime::ExtinguishRuntime::processor_mode(processor){
                            (&mut modules.extinguish.rotations,mode)
                        } else if let Some(mode)=crate::auto_pearl_runtime::AutoPearlRuntime::processor_mode(processor){
                            (&mut modules.auto_pearl.rotations,mode)
                        } else if let Some(mode)=crate::auto_buff_runtime::AutoBuffRuntime::processor_mode(processor){
                            (&mut modules.auto_buff.rotations,mode)
                        } else if let Some(mode)=crate::fast_exp_runtime::FastExpRuntime::processor_mode(processor){
                            (&mut modules.fast_exp.rotations,mode)
                        } else if let Some(mode) = crate::no_web_runtime::NoWebRuntime::processor_mode(processor) {
                            (&mut modules.no_web.rotations,mode)
                        } else if let Some(mode) = crate::auto_walk_runtime::AutoWalkSettings::processor_mode(processor) {
                            (&mut modules.auto_walk.rotations, mode)
                        } else if let Some(mode) = crate::edge_assistance_runtime::ParkourSettings::processor_mode(processor) {
                            (&mut modules.parkour.rotations,mode)
                        } else {
                            return Err(
                                liquidbounce_behaviors::rotations::RotationError::Processor(
                                    "unknown rotation processor",
                                ),
                            );
                        };
                        settings
                            .process(
                                mode,
                                from,
                                to,
                                last,
                                Rotation::new(last_yaw, last_pitch),
                                &mut liquidbounce_behaviors::random::random_unit_f32,
                            )
                            .map_err(liquidbounce_behaviors::rotations::RotationError::Processor)
                    },
                )
                .unwrap_or_else(|error| {
                    eprintln!("[LiquidBounce] rotation update: {error:?}");
                    liquidbounce_behaviors::rotations::RotationUpdate {
                        reset_player_yaw: None,
                    }
                });
            reset.reset_player_yaw
        };
        if let Some(yaw) = reset {
            host.client_rotation_reset_yaw(yaw);
        }
        self.block_placer_reached(host);
        self.hole_filler_reached(host);
        self.bed_defender_reached(host);
        self.block_placer_crystal_tick(host);
        self.hole_filler_crystal_tick(host);
        self.bed_defender_crystal_tick(host);
        self.kill_aura_ai_notification(host);
        self.kill_aura_tick(host);
        self.projectile_puncher_ai_notification(host);
        self.projectile_puncher_tick(host);
        self.no_web_tick(host);
        self.auto_deposit_punch_tick(host);
        self.chest_aura_tick(host);
        let sinking_velocity = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            modules.no_push.sinking_velocity(
                modules.query_running(&state.registry, "NoPush"),
                velocity_y,
                in_liquid != 0.0,
                physical_jump != 0.0,
                physical_shift != 0.0,
            )
        };
        if sinking_velocity.to_bits() != velocity_y.to_bits() {
            host.client_motion_velocity_y(sinking_velocity);
        }
        // A reset synchronizes yRot/yBob/yBobO before the normal GameTick
        // handler reads player rotation and submits the next tick's plan.
        let sleep_walker_running = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            modules.query_running(&state.registry, "SleepWalker")
        };
        if sleep_walker_running {
            let cleared = host.client_sleep_walker_tick();
            #[cfg(feature = "webview")]
            if cleared {
                self.state().probe(
                    "clientSleepWalker",
                    serde_json::json!(true),
                    serde_json::json!(true),
                );
            }
            #[cfg(not(feature = "webview"))]
            let _ = cleared;
        }
        self.derp_tick(host);
        self.skin_derp_tick(host);
        let hand = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            modules.hand_derp.initialize();
            let running = modules.query_running(&state.registry, "HandDerp");
            modules
                .hand_derp
                .behavior
                .lock()
                .expect("HandDerp poisoned")
                .start_delay(running, || {
                    modules
                        .coroutine_ticker
                        .register(crate::coroutine_ticker::WaitOwner::HandDerp, 0)
                })
        };
        if let Some(hand) = hand {
            host.client_hand_derp_send(hand.id());
        }
        let [yaw, pitch] = host.client_player_rotation();
        {
            let state = self.state();
            let modules = state.behaviors.borrow();
            let running = modules.query_running(&state.registry, "Sprint");
            if let Some(target) = modules.sprint.rotation_target(
                running,
                Rotation::new(yaw, pitch),
                modules.directional_input,
            ) {
                modules
                    .rotations
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .request(1, -20, target, true);
            }
        }
        if let Some([yaw, passenger]) = host.client_vehicle_boost_state() {
            #[cfg(feature = "webview")]
            self.state().probe(
                "clientVehicleBoostState",
                serde_json::json!({}),
                serde_json::json!({"yaw":yaw,"passenger":passenger != 0.0}),
            );
            let angle = yaw * 0.017453292519943295_f64;
            let sin_cos = host.client_java_sin_cos(angle);
            let boost = {
                let state = self.state();
                let mut modules = state.behaviors.borrow_mut();
                let running = modules.query_running(&state.registry, "VehicleBoost");
                modules
                    .vehicle_boost
                    .game_tick(running, passenger != 0.0, yaw as f32, |_| sin_cos)
            };
            if let Some(velocity) = boost {
                let applied = host.client_vehicle_boost_velocity(velocity);
                #[cfg(feature = "webview")]
                self.state().probe(
                    "clientVehicleBoost",
                    serde_json::json!({"yaw":yaw,"passenger":passenger != 0.0}),
                    serde_json::json!({"velocity":velocity,"applied":applied}),
                );
                #[cfg(not(feature = "webview"))]
                let _ = applied;
            }
        }
        self.anti_afk_tick(host);
        self.auto_fish_tick_handler(host);
        let retry_block_use = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            modules
                .no_block_interact
                .should_retry(modules.query_running(&state.registry, "NoBlockInteract"))
        };
        if retry_block_use {
            // This invokes real useItemOn and may re-enter client_block_interact.
            // The source clears both flags only after the nested call returns.
            host.client_start_use_item();
            self.state()
                .behaviors
                .borrow_mut()
                .no_block_interact
                .finish_retry();
        }
        self.auto_walk_prepare(host);
        self.free_cam_navigation(host);
        self.edge_assistance_prepare(host);
        self.fast_exp_tick(host);
        self.auto_wind_charge_tick(host);
        self.auto_bow_tick(host);
        self.elytra_target_tick(host);
        self.inventory_move_tick(host);
        self.auto_pearl_tick(host);
        self.extinguish_tick(host);
        self.easy_pearl_tick(host);
        self.auto_bow_tick(host);
        self.auto_buff_tick(host);
        self.auto_rod_tick(host);
        self.auto_mob_heal_tick(host);
        self.auto_shoot_tick(host);
        self.scaffold_tick(host);
        self.auto_wind_charge_tick(host);
        self.vehicle_control_tick(host);
        self.spear_kill_tick(host); // FINAL_DECISION (-500)
        self.movement_guard_prepare(host);
    }

    #[cfg(feature = "behavior-hooks")]
    fn world_primitives(&self, host: &dyn Host, event: &mut horizon_client::mod_api::WorldPrimitivesEvent<'_>) {
        self.auto_walk_render(host, event.partial, event.primitives);
        self.free_cam_render(host,event.partial,event.primitives);
        self.edge_assistance_render(host,event.primitives);
        self.spear_kill_render(host,event);
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_directional_input(&self, host: &mut dyn Host, event: &mut ClientDirectionalInputEvent) {
        self.speed_input(host, event);
        self.inventory_move_input(host,event);
        self.scaffold_input_early(host,event);
        self.criticals_input(host,&mut event.jump);
        let auto_dodge_jump=self.auto_dodge_input(host,event);
        self.sneak_input(host,event);
        self.no_slow_input(host,event);
        self.target_strafe_input(host,event);
        self.scaffold_input_model(host,event);
        self.scaffold_input_safety(host,event);
        use liquidbounce_behaviors::{rotations::DirectionalInput, sprint::Source};
        // Native navigation has just consumed the current physical input. Its
        // host calls finish before taking actor state or registry callbacks.
        let parkour_sprint = self.edge_assistance_result(host);
        let free_cam_sprint=self.free_cam_navigation_result(host);
        if self.auto_walk_result(host) {
            let options = raw_options(host);
            let mut state = self.state();
            let id = state.behaviors.borrow().query_ids.get("AutoWalk").copied();
            if let Some(id) = id {
                log_reports(state.with_registry(options, |registry, callbacks| {
                    registry.set_enabled_sync(id, false, callbacks)
                }).map(|report| vec![report]));
            }
        }
        let auto_walk_sprint = self.auto_walk_wants_sprint(host);
        let crosshair_stop_sprint=self.crosshair_stop_sprint(host)||self.kill_aura_stop_sprint(host);
        let criticals_stop_sprint=self.criticals_stop_sprint(host,0);
        let [yaw, _] = host.client_player_rotation();
        let player_tick_count = host.client_player_input_tick_count();
        // Reading use input dispatches client_use_key_pressed and may reenter
        // this mod. Sample it before taking the actor state mutex.
        let fast_use_key = host.client_fast_use_key();
        let swimming = host.client_anti_hunger_swimming().unwrap_or(false);
        #[allow(unused_mut)]
        let mut state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        if modules.fast_use.stop_input(modules.query_running(&state.registry,"FastUse"),fast_use_key) {
            event.forward=false; event.backward=false; event.left=false; event.right=false;
        }
        let knockback_running = modules.query_running(&state.registry, "SuperKnockback");
        if modules.super_knockback.block_movement(knockback_running) {
            event.forward = false;
            event.backward = false;
            event.left = false;
            event.right = false;
        }
        #[cfg(feature = "webview")]
        let twerk_input = serde_json::json!({"tickCount":player_tick_count,"shift":event.shift,
            "delay":modules.twerk.delay,"running":modules.query_running(&state.registry, "Twerk")});
        if let Some(tick_count) = player_tick_count {
            let running = modules.query_running(&state.registry, "Twerk");
            if let Err(error) = modules
                .twerk
                .movement_input(running, tick_count, &mut event.shift)
            {
                eprintln!("[LiquidBounce] Twerk input: {error:?}");
            }
        }
        #[cfg(feature = "webview")]
        let twerk_shift = event.shift;
        let mut snap_input = DirectionalInput {
            forwards: event.forward,
            backwards: event.backward,
            left: event.left,
            right: event.right,
        };
        let snap_running = modules.query_running(&state.registry, "SnapTap");
        modules
            .snap_tap
            .movement_input(snap_running, &mut snap_input);
        event.forward = snap_input.forwards;
        event.backward = snap_input.backwards;
        event.left = snap_input.left;
        event.right = snap_input.right;
        let anti_afk_running = modules.query_running(&state.registry, "AntiAFK");
        let mut direction = liquidbounce_behaviors::anti_afk::Direction {
            forwards: event.forward,
            backwards: event.backward,
            left: event.left,
            right: event.right,
        };
        modules
            .anti_afk
            .movement_input(anti_afk_running, &mut direction, &mut event.jump);
        event.forward = direction.forwards;
        event.backward = direction.backwards;
        event.left = direction.left;
        event.right = direction.right;
        let auto_wind_charge_running=modules.query_running(&state.registry,"AutoWindCharge");
        modules.auto_wind_charge.behavior.movement_input(auto_wind_charge_running,&mut event.jump);
        let no_block_running = modules.query_running(&state.registry, "NoBlockInteract");
        modules
            .no_block_interact
            .movement_input(no_block_running, &mut event.shift);
        if modules.query_running(&state.registry,"LiquidFiller")&&modules.liquid_filler.placer.queue.movement(){event.shift=true;}
        if modules.query_running(&state.registry,"BedDefender")&&modules.bed_defender.placer.queue.movement(){event.shift=true;}
        if modules.query_running(&state.registry,"HoleFiller")&&modules.hole_filler.placer.queue.movement(){event.shift=true;}
        event.shift=horizon_client::inventory_move::sneak(event.shift);
        let running = modules.query_running(&state.registry, "Sprint");
        let input = DirectionalInput {
            forwards: event.forward,
            backwards: event.backward,
            left: event.left,
            right: event.right,
        };
        modules.directional_input = input;
        let transformed = {
            let mut rotations = modules
                .rotations
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let correction = rotations
                .active_target(|provider| {
                    modules.rotation_provider_running(&state.registry, provider)
                })
                .map(|target| target.movement_correction);
            input.transform(Some(yaw), rotations.current, correction, |radians| {
                let [sin, cos] = host.client_vanilla_sin_cos(f64::from(radians));
                (sin, cos)
            })
        };
        event.forward = transformed.forwards;
        event.backward = transformed.backwards;
        event.left = transformed.left;
        event.right = transformed.right;
        if parkour_sprint && transformed.is_moving() { event.sprint = true; }
        if free_cam_sprint&&transformed.is_moving(){event.sprint=true;}
        if auto_walk_sprint && transformed.is_moving() && modules.query_running(&state.registry, "AutoWalk") {
            event.sprint = true;
        }
        event.sprint =
            modules
                .sprint
                .event(running, Source::Input, transformed, event.sprint, None);
        event.sprint=horizon_client::inventory_move::sprint(0,transformed.is_moving(),event.sprint);
        #[cfg(feature = "webview")]
        let anti_hunger_input = event.sprint;
        event.sprint = modules.anti_hunger.sprint(
            swimming,
            liquidbounce_behaviors::anti_hunger::Source::Input,
            event.sprint,
        );
        #[cfg(feature = "webview")]
        let anti_hunger_probe = serde_json::json!({"running":modules.anti_hunger.running(),
            "swimming":swimming,"sprint":anti_hunger_input});
        if criticals_stop_sprint {event.sprint=false;}
        if crosshair_stop_sprint {event.sprint=false;}
        if modules.super_knockback.stop_sprint(knockback_running) {
            event.sprint = false;
        }
        drop(modules);
        #[cfg(feature = "webview")]
        state.probe(
            "clientAntiHungerInput",
            anti_hunger_probe,
            serde_json::json!(event.sprint),
        );
        #[cfg(feature = "webview")]
        state.probe(
            "clientSnapTapInput",
            serde_json::json!({"running":snap_running}),
            serde_json::json!({"forward":event.forward,"backward":event.backward,"left":event.left,"right":event.right}),
        );
        #[cfg(feature = "webview")]
        state.probe(
            "clientTwerkInput",
            twerk_input,
            serde_json::json!({"shift":twerk_shift,"finalShift":event.shift}),
        );
        drop(state);
        self.vehicle_control_input(host,event);
        if auto_dodge_jump{event.jump=true;}
        self.state().behaviors.borrow_mut().auto_wind_charge.record_input(event);
        self.inventory_move_final(event);
        self.scaffold_input_final(host,event);
        let ground=self.state().behaviors.borrow().scaffold.facts["onGround"]==true;
        event.sprint=self.scaffold_sprint(0,event.sprint,event.forward||event.backward||event.left||event.right,ground);
        self.state().behaviors.borrow_mut().auto_buff.record_input(event);
        self.state().behaviors.borrow_mut().auto_wind_charge.record_input(event);
        self.state().behaviors.borrow().anti_void.record_input(event);
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_sprint_queries(&self, host: &dyn Host, event: &mut ClientSprintQueriesEvent) {
        self.state().behaviors.borrow().sneak.moving.store(event.sideways!=0.0||event.forward!=0.0,std::sync::atomic::Ordering::Release);
        let auto_walk_sprint = self.auto_walk_wants_sprint(host);
        let crosshair_stop_sprint=self.crosshair_stop_sprint(host)||self.kill_aura_stop_sprint(host);
        let criticals_stop_sprint=self.criticals_stop_sprint(host,1);
        let scaffold_active=self.scaffold_active();
        let scaffold_omni=self.scaffold_omnidirectional();
        use liquidbounce_behaviors::{
            rotations::{MovementCorrection, movement_yaw},
            sprint::Context,
        };
        let mut state = self.state();
        {
            let modules = state.behaviors.borrow();
            let running = modules.query_running(&state.registry, "Sprint");
            let mut rotations = modules
                .rotations
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let correction = rotations
                .active_target(|provider| {
                    modules.rotation_provider_running(&state.registry, provider)
                })
                .map(|target| target.movement_correction);
            let context = Context {
                on_ground: event.on_ground,
                slow_due_to_item: event.slow_item,
                shift_key_down: event.shift,
                player_yaw: event.player_yaw,
                sideways: event.sideways,
                forward: event.forward,
                current_rotation: rotations.current,
                movement_correction: correction,
            };
            event.movement_override = modules
                .sprint
                .movement_override(
                    running,
                    modules.directional_input,
                    context,
                    false,
                    |radians| {
                        let [sin, cos] = host.client_vanilla_sin_cos(radians);
                        (sin, cos)
                    },
                )
                .or_else(|| (auto_walk_sprint && modules.directional_input.is_moving()
                    && modules.query_running(&state.registry, "AutoWalk")).then_some(true))
                .map_or(-1, i8::from);
            event.movement_override=horizon_client::inventory_move::movement_sprint(modules.directional_input.is_moving(),event.movement_override);
            event.omnidirectional = modules.sprint.omnidirectional(running, false)||scaffold_omni;
            if scaffold_active{event.movement_override=modules.scaffold.sprint_value(1,event.movement_override,event.sideways!=0.||event.forward!=0.,event.on_ground); }
            event.ignore_blindness = modules.sprint.ignore_blindness(running);
            event.ignore_hunger = modules.sprint.ignore_hunger(running);
            event.ignore_collision = modules.sprint.ignore_collision(running);
            if running
                && modules.sprint.mode
                    == liquidbounce_behaviors::sprint::SprintMode::Omnidirectional
            {
                *event.jump_yaw = Some(modules.sprint.jump_yaw(
                    running,
                    event.player_yaw,
                    event.player_yaw,
                    modules.directional_input,
                ));
            }
            if rotations.current.is_some()
                && correction.is_some_and(|value| value != MovementCorrection::Off)
            {
                let yaw = movement_yaw(event.player_yaw, rotations.current, correction);
                *event.movement_yaw = Some(yaw);
                *event.managed_jump_yaw = Some(yaw);
            }
            if criticals_stop_sprint {event.movement_override=0;}
            if crosshair_stop_sprint {event.movement_override=0;}
            let knockback_running = modules.query_running(&state.registry, "SuperKnockback");
            if modules.super_knockback.stop_sprint(knockback_running) {
                event.movement_override = 0;
            }
        }
        #[cfg(feature = "webview")]
        state.probe("clientSprintQueries", serde_json::json!({"sideways":event.sideways,"forward":event.forward,"onGround":event.on_ground,"shift":event.shift,"slowItem":event.slow_item}), serde_json::json!({"movementOverride":event.movement_override,"omnidirectional":event.omnidirectional,"ignoreBlindness":event.ignore_blindness,"ignoreHunger":event.ignore_hunger,"ignoreCollision":event.ignore_collision,"jumpYaw":event.jump_yaw,"movementYaw":event.movement_yaw,"managedJumpYaw":event.managed_jump_yaw}));
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_sprint_network(&self, host: &dyn Host, event: &mut ClientSprintNetworkEvent) {
        use liquidbounce_behaviors::sprint::Source;
        let criticals_stop_sprint=self.criticals_stop_sprint(host,2);
        let ground=self.state().behaviors.borrow().scaffold.facts["onGround"]==true;
        let moving=self.state().behaviors.borrow().directional_input.is_moving();
        event.sprinting=self.scaffold_sprint(2,event.sprinting,moving,ground);
        #[allow(unused_mut)]
        let mut state = self.state();
        let modules = state.behaviors.borrow();
        event.sprinting = modules.sprint.event(
            modules.query_running(&state.registry, "Sprint"),
            Source::Network,
            modules.directional_input,
            event.sprinting,
            None,
        );
        event.sprinting=horizon_client::inventory_move::sprint(2,modules.directional_input.is_moving(),event.sprinting);
        if criticals_stop_sprint{event.sprinting=false;}
        #[cfg(feature = "webview")]
        let anti_hunger_input = event.sprinting;
        let swimming = host.client_anti_hunger_swimming().unwrap_or(false);
        event.sprinting = modules.anti_hunger.sprint(
            swimming,
            liquidbounce_behaviors::anti_hunger::Source::Network,
            event.sprinting,
        );
        #[cfg(feature = "webview")]
        let input = serde_json::json!({"running":modules.anti_hunger.running(),
            "swimming":swimming,"sprint":anti_hunger_input});
        drop(modules);
        #[cfg(feature = "webview")]
        state.probe(
            "clientAntiHungerNetwork",
            input,
            serde_json::json!(event.sprinting),
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_interaction_range(
        &self,
        _host: &dyn Host,
        event: &mut ClientInteractionRangeEvent<'_>,
    ) {
        #[allow(unused_mut)]
        let mut state = self.state();
        let original = event.range;
        {
            let modules = state.behaviors.borrow();
            if modules.query_running(&state.registry, "Reach") {
                event.range = modules.reach.query_range(event.attribute, event.original);
            }
        }
        #[cfg(feature = "webview")]
        state.probe(
            if event.attribute == "minecraft:entity_interaction_range" { "clientEntityReach" } else { "clientBlockReach" },
            serde_json::json!({"attribute":event.attribute,"original":event.original,"range":original}),
            serde_json::json!(event.range),
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_attack_range(&self, _host: &dyn Host, event: &mut ClientAttackRangeEvent) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        if modules.query_running(&state.registry, "Reach") {
            modules.reach.adjust_attack_range(
                &mut event.min_reach,
                &mut event.max_reach,
                &mut event.min_creative_reach,
                &mut event.max_creative_reach,
            );
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_reach_policy(&self, _host: &dyn Host, event: &mut ClientReachPolicyEvent) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        if modules.query_running(&state.registry, "Reach") {
            event.through_walls = modules.reach.through_walls_range;
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_no_rotate_capture(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientNoRotateCaptureEvent,
    ) {
        let state = self.state();
        event.running = state
            .behaviors
            .borrow()
            .query_running(&state.registry, "NoRotateSet");
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_trident_water(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientTridentWaterEvent,
    ) {
        let mut state = self.state();
        {
            let modules = state.behaviors.borrow();
            event.allowed = modules.trident_boost.water_requirement(
                modules.query_running(&state.registry, "TridentBoost"),
                event.original,
            );
        }
        #[cfg(feature = "webview")]
        state.probe(
            "clientTridentWater",
            serde_json::json!(event.original),
            serde_json::json!(event.allowed),
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_trident_impulse(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientTridentImpulseEvent,
    ) {
        let mut state = self.state();
        {
            let modules = state.behaviors.borrow();
            *event.impulse = modules.trident_boost.impulse(
                modules.query_running(&state.registry, "TridentBoost"),
                event.original,
            );
        }
        #[cfg(feature = "webview")]
        state.probe(
            "clientTridentImpulse",
            serde_json::json!(event.original),
            serde_json::json!(event.impulse),
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_entity_margin(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientEntityMarginEvent,
    ) {
        let snapshot = crate::targeting::snapshot(&host.client_targeting_entity(event.entity));
        #[allow(unused_mut)]
        let mut state = self.state();
        let valid = snapshot.as_ref().is_some_and(|snapshot| {
            let info = state.target_info(event.entity, snapshot);
            state
                .global_targets
                .combat
                .should_be_attacked(&snapshot.entity, info)
        });
        let before = event.margin;
        {
            let modules = state.behaviors.borrow();
            if modules.query_running(&state.registry, "Hitbox") {
                event.margin = modules.hitbox.pick_margin(event.margin, valid);
            }
        }
        #[cfg(feature = "webview")]
        {
            let input = serde_json::json!({"entity":event.entity,"margin":before,"target":valid});
            state.probe(
                &format!("clientEntityMargin:{}", event.entity),
                input.clone(),
                serde_json::json!(event.margin),
            );
            state.probe("clientEntityMargin", input, serde_json::json!(event.margin));
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_debug_hitbox_margin(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientDebugHitboxMarginEvent,
    ) {
        let snapshot = crate::targeting::snapshot(&host.client_targeting_entity(event.entity));
        let mut state = self.state();
        let valid = snapshot.as_ref().is_some_and(|snapshot| {
            let info = state.target_info(event.entity, snapshot);
            state
                .global_targets
                .combat
                .should_be_attacked(&snapshot.entity, info)
        });
        {
            let modules = state.behaviors.borrow();
            if modules.query_running(&state.registry, "Hitbox") {
                event.margin = modules.hitbox.debug_margin(valid);
            }
        }
        #[cfg(feature = "webview")]
        state.probe(
            "clientDebugHitboxMargin",
            serde_json::json!({"entity":event.entity,"target":valid}),
            serde_json::json!(event.margin),
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_hitbox_component_margin(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientHitboxComponentMarginEvent,
    ) {
        #[allow(unused_mut)]
        let mut state = self.state();
        let before = event.margin;
        {
            let modules = state.behaviors.borrow();
            if modules.query_running(&state.registry, "Hitbox") {
                event.margin = modules.hitbox.component_margin(event.margin);
            }
        }
        #[cfg(feature = "webview")]
        state.probe(
            "clientHitboxComponentMargin",
            serde_json::json!(before),
            serde_json::json!(event.margin),
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_attack_entity(
        &self,
        host: &mut dyn Host,
        event: &mut horizon_client::mod_api::ClientAttackEntityEvent,
    ) {
        self.anti_bot_attack(event.target as i32);
        self.kill_aura_attack_event();
        let auto_buff_owns_slot=host.client_silent_slot_modified_by(crate::auto_buff_runtime::REQUESTER);
        self.particles_attack(host,event.target as i32);
        self.hit_fx_attack(host,event.target as i32,event.living);
        self.criticals_attack_sprint(host);
        use liquidbounce_behaviors::auto_weapon::{REQUESTER, Snapshot, Target};
        // Snapshot Host state before locking: attribute/item reads can dispatch
        // other native hooks. Selecting and syncing a slot can dispatch packets.
        let weapon = host
            .client_weapon_context()
            .as_ref()
            .and_then(Snapshot::parse);
        let target = crate::targeting::snapshot(&host.client_targeting_entity(event.target as i32));
        let vehicle = host.client_vehicle_one_hit_context(event.target);
        let (switch, vehicle_actions) = {
            let mut state = self.state();
            let eligible = target.as_ref().is_some_and(|target| {
                let info = state.target_info(event.target as i32, target);
                state
                    .global_targets
                    .combat
                    .should_be_attacked(&target.entity, info)
            });
            state.combat.attack(event.living, eligible);
            // FriendManager tests actual membership rather than cached tags.
            if state.friends.cancel_attack
                && target.as_ref().is_some_and(|target| {
                    state
                        .friends
                        .tag(target.entity.kind, target.player_name.as_deref())
                        .is_friend
                })
            {
                event.cancel = true;
            }
            let modules = state.behaviors.borrow();
            let running = modules.query_running(&state.registry, "AutoWeapon");
            let switch = weapon.as_ref().and_then(|weapon| {
                modules.auto_weapon.on_attack(
                    running,
                    &weapon.context(auto_buff_owns_slot, state.registry.enabled_without_world(modules.query_ids["MaceKill"])),
                    Target {
                        living: event.living,
                        would_block_hit: event.would_block_hit,
                    },
                )
            });
            let vehicle_actions = vehicle
                .as_ref()
                .and_then(|context| {
                    Some(liquidbounce_behaviors::vehicle_one_hit::actions(
                        modules.query_running(&state.registry, "VehicleOneHit"),
                        context.get("kind")?.as_str()?,
                        event.target as i32,
                        context.get("legacy")?.as_bool()?,
                    ))
                })
                .unwrap_or_default();
            (switch, vehicle_actions)
        };
        if let Some(switch) = switch {
            if let Err(error) =
                host.client_select_slot_silently(REQUESTER, switch.slot, switch.ticks)
            {
                eprintln!("[LiquidBounce] AutoWeapon slot selection: {error}");
            }
            // Original handler syncs even when SelectSlotEvent was cancelled.
            if switch.sync_carried {
                host.client_ensure_carried();
            }
        }
        self.criticals_attack(host,event.target,event.living);
        self.super_knockback_attack(host,event.target);
        self.mace_kill_attack(host);
        for action in &vehicle_actions {
            match *action {
                liquidbounce_behaviors::vehicle_one_hit::Action::Attack(target) => {
                    host.client_vehicle_one_hit_attack(target)
                }
                liquidbounce_behaviors::vehicle_one_hit::Action::Swing => {
                    host.client_vehicle_one_hit_swing()
                }
            }
        }
        self.target_lock_attack(host, event.target as i32);
        #[cfg(feature = "webview")]
        self.state().probe("clientAttackEntity", serde_json::json!({"target":event.target,"living":event.living,"wouldBlockHit":event.would_block_hit}),
            serde_json::json!({"cancel":event.cancel,"slot":switch.map(|switch|switch.slot),"vehicleActions":vehicle_actions.len()}));
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_attack_speed(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientAttackSpeedEvent,
    ) {
        let auto_buff_owns_slot=host.client_silent_slot_modified_by(crate::auto_buff_runtime::REQUESTER);
        use liquidbounce_behaviors::auto_weapon::Snapshot;
        let snapshot = host
            .client_weapon_context()
            .as_ref()
            .and_then(Snapshot::parse);
        #[allow(unused_mut)]
        let mut state = self.state();
        {
            let modules = state.behaviors.borrow();
            if let Some(snapshot) = snapshot {
                event.speed = modules.auto_weapon.attack_speed(
                    modules.query_running(&state.registry, "AutoWeapon"),
                    modules.query_running(&state.registry, "Reach"),
                    &snapshot.context(auto_buff_owns_slot, state.registry.enabled_without_world(modules.query_ids["MaceKill"])),
                    event.original,
                );
            }
        }
        #[cfg(feature = "webview")]
        state.probe(
            "clientAttackSpeed",
            serde_json::json!(event.original),
            serde_json::json!(event.speed),
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_chams_settings(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientChamsSettingsEvent,
    ) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        *event.settings = if modules.query_running(&state.registry, "Chams") {
            modules.chams.render_settings(&state.config_root)
        } else {
            "null".into()
        };
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_chams_entity(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientChamsEntityEvent,
    ) {
        let snapshot = crate::targeting::snapshot(&host.client_targeting_entity(event.entity));
        let mut state = self.state();
        let running = state
            .behaviors
            .borrow()
            .query_running(&state.registry, "Chams");
        let free_look = state
            .registry
            .module(state.behaviors.borrow().query_ids["FreeLook"])
            .is_some_and(|module| module.enabled());
        event.target = running
            && snapshot.as_ref().is_some_and(|snapshot| {
                let info = state.target_info(event.entity, snapshot);
                state.global_targets.visual.should_show(
                    &snapshot.entity,
                    info,
                    liquidbounce_behaviors::targeting::VisualCamera {
                        raw_first_person: event.raw_perspective == 0,
                        free_cam: false,
                        free_look,
                    },
                )
            });
        #[cfg(feature = "webview")]
        {
            let input =
                serde_json::json!({"entity":event.entity,"rawPerspective":event.raw_perspective});
            state.probe(
                &format!("clientChamsEntity:{}", event.entity),
                input.clone(),
                serde_json::json!(event.target),
            );
            state.probe("clientChamsEntity", input, serde_json::json!(event.target));
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_block_breaking_progress(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientBlockBreakingProgressEvent,
    ) {
        self.auto_tool_progress(host, event.pos);
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_inventory_wake(
        &self,
        host: &mut dyn Host,
        _event: &mut horizon_client::mod_api::ClientInventoryWakeEvent,
    ) {
        self.inventory_frame(host);
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_auto_tool_frame(
        &self,
        host: &mut dyn Host,
        _event: &mut horizon_client::mod_api::ClientAutoToolFrameEvent,
    ) {
        self.auto_tool_frame(host);
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_inventory_observation(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientInventoryObservationEvent<'_>,
    ) {
        self.auto_tool_observe(host, event.payload);
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_silent_hotbar_render(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientSilentHotbarRenderEvent,
    ) {
        let slot = host
            .client_clientside_slot()
            .and_then(|slot| u8::try_from(slot).ok())
            .filter(|slot| *slot < 9);
        let modified = host.client_silent_slot_modified();
        #[allow(unused_mut)]
        let mut state = self.state();
        {
            let modules = state.behaviors.borrow();
            if let Some(slot) = slot {
                modules.silent_hotbar.render(
                    modules.query_running(&state.registry, "SilentHotbar"),
                    modified,
                    slot,
                    &mut event.visible_slot,
                    &mut event.swap_scale,
                );
            }
        }
        #[cfg(feature = "webview")]
        state.probe(
            "clientSilentHotbarRender",
            serde_json::json!({"modified":modified,"clientSlot":slot}),
            serde_json::json!({"visibleSlot":event.visible_slot,"swapScale":event.swap_scale}),
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_multi_actions(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientMultiActionsEvent,
    ) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        let enabled = modules
            .query_ids
            .get("MultiActions")
            .and_then(|id| state.registry.module(*id))
            .is_some_and(|module| module.enabled());
        (
            event.place_while_breaking,
            event.attack_while_using,
            event.break_while_using,
        ) = modules.multi_actions.policy(enabled);
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_server_rotation(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientServerRotationEvent,
    ) {
        use liquidbounce_behaviors::rotations::Rotation;
        #[allow(unused_mut)]
        let mut state = self.state();
        {
            let modules = state.behaviors.borrow();
            let running = modules.query_running(&state.registry, "NoRotateSet");
            if let Some(correction) = modules.no_rotate_set.after_correction(
                event.preaccepted || running,
                false,
                Rotation::new(event.previous_yaw, event.previous_pitch),
                Rotation::new(event.corrected_yaw, event.corrected_pitch),
                true,
            ) {
                event.yaw = correction.restored.yaw;
                event.pitch = correction.restored.pitch;
                event.restore = true;
                if let Some(target) = correction.target {
                    modules
                        .rotations
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .request_processed(
                            2,
                            -20,
                            target.target,
                            target.processor.processor_id(),
                            true,
                        );
                }
            }
        }
        #[cfg(feature = "webview")]
        state.probe("clientServerRotation", serde_json::json!({"previous":[event.previous_yaw,event.previous_pitch],"corrected":[event.corrected_yaw,event.corrected_pitch],"preaccepted":event.preaccepted}),
            serde_json::json!({"yaw":event.yaw,"pitch":event.pitch,"restore":event.restore}));
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_smart_eat_wait(
        &self,
        host: &mut dyn Host,
        _event: &mut horizon_client::mod_api::ClientSmartEatWaitEvent,
    ) {
        if host.client_coroutine_outermost() {
            let has_client = host.client_state().is_some();
            if has_client {
                self.state().behaviors.borrow().combat_cooldown.drain();
            }
            self.coroutine_callbacks(host, has_client);
        }
    }

    #[cfg(feature="behavior-hooks")]
    fn client_kill_aura_state(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientKillAuraStateEvent){self.kill_aura_block_state(&mut event.enforced,&mut event.pose);}
    #[cfg(feature="behavior-hooks")]
    fn client_crosshair_clicker_attack_key(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientCrosshairClickerAttackKeyEvent){self.crosshair_attack_key(&mut event.pressed);self.kill_aura_attack_key(&mut event.pressed);}

    #[cfg(feature = "behavior-hooks")]
    fn client_use_key_pressed(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientUseKeyPressedEvent,
    ) {
        self.auto_bow_use_key(&mut event.pressed);
        self.auto_bow_use_key(&mut event.pressed);
        self.smart_eat_use_key(&mut event.pressed);
        self.auto_buff_use_key(&mut event.pressed);
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_item_interacted(
        &self,
        host: &mut dyn Host,
        event: &mut horizon_client::mod_api::ClientItemInteractedEvent,
    ) {
        self.smart_eat_interacted(host, event.pass);
    }

    #[cfg(feature = "behavior-hooks")]
    fn gui_layer_pre(
        &self,
        host: &mut dyn Host,
        event: &mut horizon_client::mod_api::GuiLayerPreEvent<'_>,
    ) -> horizon_client::mod_api::Flow {
        if event.layer == "minecraft:hotbar" {
            self.smart_eat_render(host, event.batch);
            self.clip_render(host,event.batch);
        }
        horizon_client::mod_api::Flow::Continue
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_elytra_recast_running(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientElytraRecastRunningEvent,
    ) {
        let state = self.state();
        event.running = state
            .behaviors
            .borrow_mut()
            .query_running(&state.registry, "ElytraRecast");
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_strafe(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientStrafeEvent,
    ) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        event.air = modules.strafe.strength_in_air;
        event.ground = modules.strafe.strength_on_ground;
        event.strict = modules.strafe.strict_movement;
        let input = modules.directional_input;
        event.input = u8::from(input.forwards)
            | (u8::from(input.backwards) << 1)
            | (u8::from(input.left) << 2)
            | (u8::from(input.right) << 3);
        event.apply = modules.query_running(&state.registry, "Strafe");
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_firework_boost(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientFireworkBoostEvent<'_>,
    ) {
        use liquidbounce_behaviors::rotations::MovementCorrection;
        #[allow(unused_mut)]
        let mut state = self.state();
        let modules = state.behaviors.borrow();
        let managed = {
            let mut rotations = modules
                .rotations
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let target = rotations.active_target(|provider| {
                modules.rotation_provider_running(&state.registry, provider)
            });
            rotations.current.filter(|_| {
                target.is_some_and(|target| target.movement_correction != MovementCorrection::Off)
            })
        };
        if let Some(rotation) = managed {
            *event.look = liquidbounce_behaviors::extended_firework::direction_from_rotation(
                rotation.pitch,
                rotation.yaw,
                |angle| host.client_vanilla_sin_cos(angle),
            );
        }
        let running = modules.query_running(&state.registry, "ExtendedFirework");
        if running {
            *event.multiplier = modules.extended_firework.multiplier();
            event.apply = true;
        }
        #[cfg(feature = "webview")]
        let result = serde_json::json!({"look":event.look,"multiplier":event.multiplier,"apply":event.apply,"running":running});
        drop(modules);
        #[cfg(feature = "webview")]
        state.probe(
            "clientFireworkBoost",
            serde_json::json!(event.movement),
            result,
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_ghost_hand(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientGhostHandEvent,
    ) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        event.running = modules.query_running(&state.registry, "GhostHand");
        if event.running {
            *event.targeted_blocks = modules
                .ghost_hand
                .sorted_targeted_blocks()
                .into_iter()
                .map(str::to_owned)
                .collect();
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_block_walk(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientBlockWalkEvent<'_>,
    ) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        *event.blocks = serde_json::to_string(&modules.block_walk.blocks)
            .expect("BlockWalk identifiers serialize");
        event.apply = modules.query_running(&state.registry, "BlockWalk");
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_entity_control(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientEntityControlEvent,
    ) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        (event.saddled, event.jump_strength) = modules
            .entity_control
            .policy(modules.query_running(&state.registry, "EntityControl"));
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_block_bounce(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientBlockBounceEvent,
    ) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        if modules.query_running(&state.registry, "BlockBounce") {
            event.motion = modules.block_bounce.motion;
            event.apply = true;
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_mouse_rotation(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientMouseRotationEvent,
    ) {
        let inventory_open = host.client_inventory_open();
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        let free_cam_running=modules.query_running(&state.registry,"FreeCam");
        event.canceled|=modules.free_cam.behavior.mouse(free_cam_running,event.cursor_x,event.cursor_y);
        let running = modules.query_running(&state.registry, "FreeLook");
        event.canceled |= modules
            .free_look
            .mouse_rotation(running, event.cursor_x, event.cursor_y);
        modules.auto_rod.input.moved(event.cursor_x,event.cursor_y,behaviors::current_time_millis());
        if modules.query_running(&state.registry,"Aimbot"){modules.aimbot.behavior.mouse(event.cursor_x,event.cursor_y);}
        modules
            .rotations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .mouse_rotation(
                event.cursor_x,
                event.cursor_y,
                true,
                inventory_open,
                |provider| modules.rotation_provider_running(&state.registry, provider),
            );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_free_look_camera(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientFreeLookCameraEvent<'_>,
    ) {
        let mut state = self.state();
        let rotation = {
            let modules = state.behaviors.borrow();
            modules
                .free_look
                .camera_rotation(modules.query_running(&state.registry, "FreeLook"))
        };
        if let Some(rotation) = rotation {
            *event.rotation = rotation;
            event.apply = true;
        }
        #[cfg(feature = "webview")]
        state.probe(
            "clientFreeLookCamera",
            serde_json::Value::Null,
            serde_json::json!({"apply":event.apply,"rotation":event.rotation}),
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_change_look(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientChangeLookEvent,
    ) {
        use liquidbounce_behaviors::rotations::modern_mouse_gcd;
        let sensitivity = host.client_mouse_sensitivity();
        let inventory_open = host.client_inventory_open();
        let state = self.state();
        let modules = state.behaviors.borrow();
        match modules
            .rotations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .change_look_rotation(
                event.partial,
                modern_mouse_gcd(sensitivity),
                true,
                inventory_open,
                |provider| modules.rotation_provider_running(&state.registry, provider),
            ) {
            Ok(Some(rotation)) => {
                event.yaw = rotation.yaw;
                event.pitch = rotation.pitch;
                event.apply = true;
            }
            Ok(None) => {}
            Err(error) => eprintln!("[LiquidBounce] change look: {error:?}"),
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_rotation_aim(&self, _host: &dyn Host, event: &mut ClientRotationAimEvent) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        if let Some(rotation) = modules
            .rotations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .current
        {
            event.yaw = rotation.yaw;
            event.pitch = rotation.pitch;
            event.managed = true;
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_rotation_network(&self, _host: &dyn Host, event: &mut ClientRotationNetworkEvent) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        let rotations = modules
            .rotations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(rotation) = rotations.current {
            event.yaw = rotation.yaw;
            event.pitch = rotation.pitch;
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_rotation_model(&self, _host: &dyn Host, event: &mut ClientRotationModelEvent) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        let rotations = modules
            .rotations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        event.body = rotations.model_yaw(event.partial, event.body, true);
        event.head = rotations.model_yaw(event.partial, event.head, true);
        event.pitch = rotations.model_pitch(event.partial, event.pitch, true);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_interpolation_policy(
        &self,
        _host: &dyn Host,
        event: &mut ClientInterpolationPolicyEvent,
    ) {
        let mut state = self.state();
        {
            let modules = state.behaviors.borrow();
            if modules.query_running(&state.registry, "NoInterpolation") {
                event.enabled = true;
                event.reduce_by = modules.no_interpolation.reduce_by;
            }
        }
        #[cfg(feature = "webview")]
        state.probe(
            "clientInterpolationPolicy",
            serde_json::json!(null),
            serde_json::json!({"enabled":event.enabled,"reduceBy":event.reduce_by}),
        );
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_movement_queries(&self, _host: &dyn Host, event: &mut ClientMovementQueriesEvent) {
        let mut state = self.state();
        let before = serde_json::json!({"resetJumpDelay":event.reset_jump_delay,"suppressBounce":event.suppress_bounce,"ignoreLevitation":event.ignore_levitation,"ignoreSlowFalling":event.ignore_slow_falling});
        {
            let b = state.behaviors.borrow();
            let running = |name| b.query_running(&state.registry, name);
            event.reset_jump_delay |= liquidbounce_behaviors::movement_query::clear_jump_delay(
                running("NoJumpDelay"),
                b.air_jump.allow_jump(running("AirJump")),
                false,
            );
            event.suppress_bounce |= running("AntiBounce");
            event.ignore_levitation |= running("AntiLevitation");
            event.ignore_slow_falling |= running("AntiLevitation");
        }
        #[cfg(feature = "webview")]
        state.probe("clientMovementQueries", before, serde_json::json!({"resetJumpDelay":event.reset_jump_delay,"suppressBounce":event.suppress_bounce,"ignoreLevitation":event.ignore_levitation,"ignoreSlowFalling":event.ignore_slow_falling}));
        #[cfg(not(feature = "webview"))]
        let _ = before;
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_pick_fluid(&self, _host: &dyn Host, event: &mut ClientPickFluidEvent) {
        let mut state = self.state();
        let original = event.any;
        event.any = liquidbounce_behaviors::interaction_query::pick_any_fluid(
            original,
            state
                .behaviors
                .borrow()
                .query_running(&state.registry, "LiquidPlace"),
        );
        #[cfg(feature = "webview")]
        state.probe(
            "clientPickFluid",
            serde_json::json!(original),
            serde_json::json!(event.any),
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_portal_allowed(&self, _host: &dyn Host, event: &mut ClientPortalAllowedEvent) {
        let mut state = self.state();
        let original = event.allowed;
        event.allowed = liquidbounce_behaviors::interaction_query::portal_allowed(
            original,
            state
                .behaviors
                .borrow()
                .query_running(&state.registry, "PortalMenu"),
        );
        #[cfg(feature = "webview")]
        state.probe(
            "clientPortalAllowed",
            serde_json::json!(original),
            serde_json::json!(event.allowed),
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_player_reduced_debug(
        &self,
        _host: &dyn Host,
        event: &mut ClientPlayerReducedDebugEvent,
    ) {
        let mut state = self.state();
        let original = event.reduced;
        event.reduced = liquidbounce_behaviors::interaction_query::player_reduced_debug(
            original,
            state
                .behaviors
                .borrow()
                .query_running(&state.registry, "AntiReducedDebugInfo"),
        );
        #[cfg(feature = "webview")]
        state.probe(
            "clientPlayerReducedDebug",
            serde_json::json!(original),
            serde_json::json!(event.reduced),
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_attack_queries(&self, _host: &dyn Host, event: &mut ClientAttackQueriesEvent) {
        let mut state = self.state();
        let before = serde_json::json!({"removeCooldown":event.remove_cooldown,"cancelOnMiss":event.cancel_on_miss});
        {
            let b = state.behaviors.borrow();
            let (remove, cancel) = b
                .no_miss_cooldown
                .queries(b.query_running(&state.registry, "NoMissCooldown"));
            event.remove_cooldown |= remove;
            event.cancel_on_miss |= cancel;
        }
        #[cfg(feature = "webview")]
        state.probe("clientAttackQueries", before, serde_json::json!({"removeCooldown":event.remove_cooldown,"cancelOnMiss":event.cancel_on_miss}));
        #[cfg(not(feature = "webview"))]
        let _ = before;
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_abort_breaking(&self, _host: &dyn Host, event: &mut ClientAbortBreakingEvent) {
        let mut state = self.state();
        let original = event.canceled;
        {
            let mut modules = state.behaviors.borrow_mut();
            let running = modules.query_running(&state.registry, "AutoTool");
            modules.auto_tool.cancel_breaking(running);
        }
        event.canceled |= state
            .behaviors
            .borrow()
            .query_running(&state.registry, "AbortBreaking");
        #[cfg(feature = "webview")]
        state.probe(
            "clientAbortBreaking",
            serde_json::json!(original),
            serde_json::json!(event.canceled),
        );
    }

    #[cfg(feature = "webview")]
    fn screen_mouse_button_pressed_pre(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ScreenMouseButtonPressedPreEvent<'_>,
    ) -> horizon_client::mod_api::Flow {
        self.browser_button(host, event.button, event.modifiers, false)
    }
    #[cfg(feature = "webview")]
    fn screen_mouse_button_released_pre(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ScreenMouseButtonReleasedPreEvent<'_>,
    ) -> horizon_client::mod_api::Flow {
        self.browser_button(host, event.button, event.modifiers, true)
    }
    #[cfg(feature = "webview")]
    fn screen_mouse_scrolled_pre(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ScreenMouseScrolledPreEvent<'_>,
    ) -> horizon_client::mod_api::Flow {
        self.browser_wheel(host, event.scroll)
    }
    #[cfg(feature = "webview")]
    fn screen_char_typed_pre(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ScreenCharTypedPreEvent<'_>,
    ) -> horizon_client::mod_api::Flow {
        self.browser_char(host, event.c, event.modifiers)
    }
    #[cfg(feature = "webview")]
    fn screen_key_pressed_pre(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ScreenKeyPressedPreEvent<'_>,
    ) -> horizon_client::mod_api::Flow {
        self.browser_screen_key(host, event.key, event.repeat, event.modifiers, false)
    }
    #[cfg(feature = "webview")]
    fn screen_key_released_pre(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ScreenKeyReleasedPreEvent<'_>,
    ) -> horizon_client::mod_api::Flow {
        self.browser_screen_key(host, event.key, false, event.modifiers, true)
    }
    fn mods(&self, _host: &mut dyn Host, event: &mut ModsEvent<'_>) {
        if event.mods.iter().any(|item| item.id == MOD_ID) {
            return;
        }
        event.mods.push(NativeMod {
            id: MOD_ID.into(),
            title: ui_label("LiquidBounce Rust port 0.1.0"),
            description: format!(
                "{}: {} / 236; {}.",
                ui_label("Native gameplay modules"),
                self.state().registry.modules().count(),
                ui_label(if cfg!(feature = "webview") {
                    "Chromium ClickGUI"
                } else {
                    "Source extensions required"
                })
            ),
            keys: self
                .state()
                .registry
                .modules()
                .map(|module| {
                    let name = &module.definition().name;
                    [
                        name.clone(),
                        ui_module_name(name),
                        "key.keyboard.unknown".into(),
                    ]
                })
                .collect(),
        });
    }

    fn client_tick_post(&self, host: &mut dyn Host, _event: &mut ClientTickPostEvent) {
        let Some(client) = host.client_state() else {
            return;
        };
        let options = raw_options(host);
        #[cfg(feature = "behavior-hooks")]
        self.speed_tick(host);
        #[cfg(feature = "webview")]
        self.browser_tick(host, &client, options);
        #[cfg(all(feature = "behavior-hooks", not(feature = "webview")))]
        let bindings: Vec<_> = self
            .state()
            .registry
            .modules()
            .map(|module| {
                let name = &module.definition().name;
                let key =
                    input::configured_bind(&host.mod_key(MOD_ID, name, "key.keyboard.unknown"));
                let action =
                    if matches!(name.as_str(), "Zoom" | "QuickPerspectiveSwap" | "FreeLook") {
                        liquidbounce_core::BindAction::Hold
                    } else {
                        liquidbounce_core::BindAction::Toggle
                    };
                (module.id(), key, action)
            })
            .collect();
        let (open_status, effects) = {
            let mut state = self.state();
            Self::observe_world(&mut state, &client, options);
            #[cfg(all(feature = "behavior-hooks", not(feature = "webview")))]
            {
                for (id, key, action) in bindings {
                    state
                        .registry
                        .set_bind(
                            id,
                            key.map(|key| liquidbounce_core::InputBind::new(key, action)),
                        )
                        .expect("registered module");
                }
            }
            state.observed.ticks = state.observed.ticks.saturating_add(1);
            let show = state.show_status_when_playing
                && state.observed.world.in_game()
                && matches!(client.screen, NativeScreenKind::Playing);
            (show, state.take_effects())
        };
        apply_effects(self, host, effects);
        // Host calls happen outside the state lock: native hooks may reenter.
        if open_status && host.mod_screen_open(MOD_ID) {
            self.state().show_status_when_playing = false;
        }
    }

    fn input_key(&self, host: &mut dyn Host, event: &mut InputKeyEvent<'_>) {
        #[cfg(feature = "webview")]
        self.browser_key(host, event);
        let PhysicalKey::Code(code) = event.event.physical_key else {
            return;
        };
        let action = match (event.event.state, event.event.repeat) {
            (ElementState::Released, _) => KeyAction::Release,
            (ElementState::Pressed, true) => KeyAction::Repeat,
            (ElementState::Pressed, false) => KeyAction::Press,
        };
        #[cfg(feature="behavior-hooks")]
        if let Some(key)=horizon_client::keybinds::key_name(code){self.crosshair_player_press(host,key,action==KeyAction::Press);}
        self.feed_keyboard(host, code, action, event.modifiers);
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_control_key(
        &self,
        host: &mut dyn Host,
        event: &mut horizon_client::mod_api::ClientControlKeyEvent,
    ) {
        #[cfg(feature = "webview")]
        self.browser_physical_key(
            host,
            event.key,
            if event.pressed {
                ElementState::Pressed
            } else {
                ElementState::Released
            },
            event.repeat,
            event.modifiers,
        );
        self.feed_keyboard(
            host,
            event.key,
            if !event.pressed {
                KeyAction::Release
            } else if event.repeat {
                KeyAction::Repeat
            } else {
                KeyAction::Press
            },
            event.modifiers,
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn mouse_button_pre(
        &self,
        host: &mut dyn Host,
        event: &mut horizon_client::mod_api::MouseButtonPreEvent,
    ) -> horizon_client::mod_api::Flow {
        self.crosshair_mouse(event.button,event.pressed);
        if let Some(button)=input::mouse_button(event.button){self.state().behaviors.borrow_mut().auto_rod.input.mouse_event(button,event.pressed,behaviors::current_time_millis());}
        if let Some(button)=input::mouse_button(event.button) && let Some((_,key))=horizon_client::key_names::MOUSE.iter().find(|(b,_)|*b==button){self.crosshair_player_press(host,key,event.pressed);}
        self.state().behaviors.borrow_mut().press_times.mouse_event(
            event.button,
            event.pressed,
            behaviors::current_time_millis(),
        );
        horizon_client::mod_api::Flow::Continue
    }

    fn mouse_button_post(&self, host: &mut dyn Host, event: &mut MouseButtonPostEvent) {
        let Some(client) = host.client_state() else {
            return;
        };
        let Some(button) = input::mouse_button(event.button) else {
            return;
        };
        let action = if event.pressed {
            KeyAction::Press
        } else {
            KeyAction::Release
        };
        let context = InputContext {
            screen_open: !matches!(client.screen, NativeScreenKind::Playing),
            debug_modifier_down: debug_modifier_down(host),
        };
        let now_ms = self.started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
        let options = raw_options(host);
        let effects = {
            let mut state = self.state();
            Self::observe_world(&mut state, &client, options);
            state.observed.mouse_events = state.observed.mouse_events.saturating_add(1);
            log_reports(state.with_registry(options, |registry, callbacks| {
                registry.handle_mouse_sync(
                    MouseInput::new(button, action, input::modifiers(event.modifiers)),
                    context,
                    now_ms,
                    callbacks,
                )
            }));
            state.take_effects()
        };
        apply_effects(self, host, effects);
    }

    fn client_disconnecting(&self, host: &dyn Host, _event: &mut ClientDisconnectingEvent) {
        #[cfg(feature="behavior-hooks")]
        self.state().behaviors.borrow().ping_spoof.world_change();
        // The hook also runs on server threads; only the window has client state.
        let Some(client) = host.client_state() else {
            return;
        };
        let options = raw_options(host);
        let mut state = self.state();
        Self::observe_world(&mut state, &client, options);
        for report in state.with_registry(options, |registry, callbacks| {
            registry.disconnect_sync(callbacks)
        }) {
            log_reports(report.map(|report| vec![report]));
        }
        state.observed.disconnects = state.observed.disconnects.saturating_add(1);
        #[cfg(feature = "behavior-hooks")]
        {
            let mut modules = state.behaviors.borrow_mut();
            modules
                .rotations
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .reset_world();
            modules.directional_input = Default::default();
            modules.middle_click_action.world_change();
            modules.anti_bot.state.reset();modules.notifier.bot_entities.clear();
            modules.combat_cooldown.disconnect();
            modules.auto_queue.cancel();
            modules.macros.cancel();
            modules.report_helper.disabled();
            modules.inventory.world_change();
            modules.fast_use.cancel();
            modules.ping_spoof.world_change();
            modules.clip.cancel();
            modules.auto_tool.swaps.cancel_plan();
            modules.inventory_packets.world_change();
        }
    }

    fn settings(&self, _host: &mut dyn Host, event: &mut SettingsEvent<'_>) {
        if event.mod_id != MOD_ID {
            return;
        }
        event.buttons.push(ui_label(if cfg!(feature = "webview") {
            "Open ClickGUI after closing menus"
        } else {
            "Show integration status after closing menus"
        }));
        #[cfg(feature = "behavior-hooks")]
        {
            let state = self.state();
            let modules = state.behaviors.borrow();
            for id in modules.ids() {
                let module = state.registry.module(id).expect("registered module");
                event.buttons.push(format!(
                    "{}: {}",
                    ui_module_name(&module.definition().name),
                    ui_label(if module.enabled() { "ON" } else { "OFF" })
                ));
            }
            event.buttons.push(format!(
                "{}: {}",
                ui_label("FullBright mode"),
                ui_label(modules.full_bright.mode().tag())
            ));
        }
    }

    fn settings_click(&self, host: &mut dyn Host, event: &mut SettingsClickEvent<'_>) {
        if event.mod_id != MOD_ID {
            return;
        }
        if event.button == 0 {
            self.state().show_status_when_playing = true;
            return;
        }
        #[cfg(feature = "behavior-hooks")]
        {
            let options = raw_options(host);
            let client = host.client_state();
            let effects = {
                let mut state = self.state();
                if let Some(client) = client {
                    Self::observe_world(&mut state, &client, options);
                }
                let ids = {
                    let modules = state.behaviors.borrow();
                    modules.ids()
                };
                if let Some(id) = event
                    .button
                    .checked_sub(1)
                    .and_then(|index| ids.get(index))
                    .copied()
                {
                    let enabled = !state
                        .registry
                        .module(id)
                        .expect("registered module")
                        .enabled();
                    log_reports(
                        state
                            .with_registry(options, |registry, callbacks| {
                                registry.set_enabled_sync(id, enabled, callbacks)
                            })
                            .map(|report| vec![report]),
                    );
                } else if event.button == ids.len() + 1 {
                    let intents = state
                        .behaviors
                        .borrow_mut()
                        .toggle_full_bright_mode(&state.registry, options);
                    for intent in intents {
                        state.behaviors.borrow_mut().push_night_vision(intent);
                    }
                }
                state.take_effects()
            };
            apply_effects(self, host, effects);
        }
        #[cfg(not(feature = "behavior-hooks"))]
        let _ = host;
    }

    #[cfg(feature = "behavior-hooks")]
    fn camera_bob_effects(&self, _host: &dyn Host, event: &mut CameraBobEffectsEvent) {
        #[allow(unused_mut)]
        let mut state = self.state();
        let original = (event.hurt, event.view);
        let filtered =
            state
                .behaviors
                .borrow()
                .camera_bob_effects(&state.registry, original.0, original.1);
        event.hurt = filtered.0;
        event.view = filtered.1;
        #[cfg(feature = "webview")]
        state.probe(
            "cameraBobEffects",
            serde_json::json!({"hurt":original.0,"view":original.1}),
            serde_json::json!({"hurt":filtered.0,"view":filtered.1}),
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn camera_bob_result(&self, _host: &dyn Host, event: &mut CameraBobResultEvent) {
        #[cfg(feature = "webview")]
        self.state().probe("cameraBobResult",
            serde_json::json!({"partial":event.partial,"hurtTime":event.hurt_time,"hurtDuration":event.hurt_duration,"hurtDir":event.hurt_dir,"deathTime":event.death_time,"walk":event.walk,"bob":event.bob,"damageTiltStrength":event.damage_tilt,"original":event.original,"shake":event.shake}),
            serde_json::json!({"bob":event.applied,"combined":event.combined}));
        #[cfg(not(feature = "webview"))]
        let _ = event;
    }

    #[cfg(feature = "behavior-hooks")]
    fn camera_spinning_result(&self, _host: &dyn Host, event: &mut CameraSpinningResultEvent) {
        #[cfg(feature = "webview")]
        self.state().probe("cameraSpinningResult",
            serde_json::json!({"portal":event.portal,"nausea":event.nausea,"scale":event.scale,"intensity":event.intensity,"angle":event.angle}),
            serde_json::json!({"matrix":event.matrix}));
        #[cfg(not(feature = "webview"))]
        let _ = event;
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_camera_aspect(&self, _host: &dyn Host, event: &mut ClientCameraAspectEvent) {
        #[allow(unused_mut)]
        let mut state = self.state();
        #[cfg(feature = "webview")]
        let original = event.ratio_percentage;
        {
            let modules = state.behaviors.borrow();
            event.ratio_percentage = modules
                .aspect
                .percentage(modules.query_running(&state.registry, "Aspect"));
        }
        #[cfg(feature = "webview")]
        state.probe(
            "clientCameraAspect",
            serde_json::json!(original),
            serde_json::json!(event.ratio_percentage),
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_camera_position(&self, _host: &dyn Host, event: &mut ClientCameraPositionEvent<'_>) {
        use liquidbounce_behaviors::camera_control::Perspective;
        #[allow(unused_mut)]
        let mut state = self.state();
        #[cfg(feature = "webview")]
        let original = *event.position;
        {
            let mut modules = state.behaviors.borrow_mut();
            let running = modules.query_running(&state.registry, "SmoothCamera");
            let free_look = modules.query_running(&state.registry, "FreeLook");
            if let Some(perspective) = Perspective::from_code(event.raw_perspective) {
                *event.position = modules.smooth_camera.camera_update(
                    running,
                    free_look,
                    perspective,
                    *event.position,
                );
            }
        }
        #[cfg(feature = "webview")]
        state.probe(
            "clientCameraPosition",
            serde_json::json!({"rawPerspective":event.raw_perspective,"position":original}),
            serde_json::json!(event.position),
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_camera_tick(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientCameraTickEvent,
    ) {
        use liquidbounce_behaviors::camera_control::Perspective;
        let screen = host.auto_f5_screen();
        #[allow(unused_mut)]
        let mut state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        let clip_running = modules.query_running(&state.registry, "CameraClip");
        modules.camera_clip.perspective(
            clip_running,
            event.last_perspective,
            event.perspective,
            &mut event.last_distance,
            &mut event.distance,
            &mut event.no_clip,
        );
        event.perspective = modules.auto_f5.perspective(
            modules.query_running(&state.registry, "AutoF5"),
            event.perspective,
            screen
                .as_ref()
                .map(|(menu, title, tags)| (menu.as_deref(), title.as_str(), tags.as_slice())),
        );
        event.perspective = modules.free_look.perspective(
            modules.query_running(&state.registry, "FreeLook"),
            event.perspective,
        );
        if modules.free_cam.behavior.active(modules.query_running(&state.registry,"FreeCam")){event.perspective=0;}
        // Source listeners use the same event in ModuleManager order.
        if let Some(original) = Perspective::from_code(event.perspective) {
            event.perspective = modules.quick_perspective_swap.perspective(
                modules.query_running(&state.registry, "QuickPerspectiveSwap"),
                modules.query_running(&state.registry, "FreeLook"),
                event.managed_camera || modules.query_running(&state.registry, "FreeCam"),
                original,
            ) as u8;
        }
        let changed = modules
            .camera_clip
            .final_perspective(clip_running, event.perspective);
        drop(modules);
        #[cfg(feature = "webview")]
        if changed {
            state.camera_distance_changed();
        }
        #[cfg(feature = "webview")]
        {
            state.probe("clientCameraPerspective", serde_json::json!({"rawPerspective":event.raw_perspective,"managedCamera":event.managed_camera,"screen":screen}), serde_json::json!(event.perspective));
            state.probe("clientCameraTick", serde_json::json!({"rawPerspective":event.raw_perspective,"lastPerspective":event.last_perspective}),
                serde_json::json!({"perspective":event.perspective,"lastDistance":event.last_distance,"distance":event.distance,"noClip":event.no_clip}));
        }
    }

    #[cfg(feature="behavior-hooks")]
    fn client_scaffold_success(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientScaffoldSuccessEvent){self.scaffold_success(host,event.placed,event.simulated);}
    #[cfg(feature="behavior-hooks")]
    fn client_scaffold_jump(&self,event:&mut horizon_client::mod_api::ClientScaffoldJumpEvent){self.scaffold_jump(event.y,event.motion);}
    #[cfg(feature="behavior-hooks")]
    fn client_scaffold_after_jump(&self,event:&mut horizon_client::mod_api::ClientScaffoldAfterJumpEvent){self.scaffold_after_jump(event.yaw,[event.forward,event.backward,event.left,event.right],&mut event.x,&mut event.y,&mut event.z);}
    #[cfg(feature="behavior-hooks")]
    fn client_air_place_configuration(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientAirPlaceConfigurationEvent<'_>){self.air_place_configuration(event.configuration);}
    #[cfg(feature = "behavior-hooks")]
    fn client_mouse_scroll_raw(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientMouseScrollRawEvent,
    ) {
        self.air_place_scroll(host,event.vertical);
        let modifier_scan = self.state().behaviors.borrow().camera_clip.modifier.value;
        let modifier_down = host.client_keyboard_scan_down(modifier_scan);
        let raw_perspective = host.raw_camera_perspective();
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        let running = modules.query_running(&state.registry, "CameraClip");
        let free_look = modules.query_running(&state.registry, "FreeLook");
        modules.camera_clip.scroll(
            running,
            modifier_down,
            raw_perspective,
            free_look,
            event.vertical,
        );
    }

    #[cfg(feature = "behavior-hooks")]
    fn client_dank_bobbing(
        &self,
        _host: &dyn Host,
        event: &mut horizon_client::mod_api::ClientDankBobbingEvent,
    ) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        if let Some(motion) = modules
            .dank_bobbing
            .motion(modules.query_running(&state.registry, "DankBobbing"))
        {
            event.enabled = true;
            event.motion = motion;
        }
    }

    #[cfg(feature = "behavior-hooks")]
    fn camera_fov_option(&self, _host: &dyn Host, event: &mut CameraFovOptionEvent) {
        let original = event.option;
        #[allow(unused_mut)]
        let mut state = self.state();
        event.option = state
            .behaviors
            .borrow_mut()
            .camera_option(&state.registry, event.original);
        #[cfg(feature = "webview")]
        state.probe(
            "cameraFovOption",
            serde_json::json!(original),
            serde_json::json!(event.option),
        );
    }
    #[cfg(feature = "behavior-hooks")]
    fn camera_fov_return(&self, _host: &dyn Host, event: &mut CameraFovReturnEvent) {
        let original = event.fov;
        #[allow(unused_mut)]
        let mut state = self.state();
        event.fov = state
            .behaviors
            .borrow()
            .camera_return(&state.registry, event.fov);
        #[cfg(feature = "webview")]
        state.probe(
            "cameraFovReturn",
            serde_json::json!(original),
            serde_json::json!(event.fov),
        );
    }
    #[cfg(feature = "behavior-hooks")]
    fn player_fov_return(&self, _host: &dyn Host, event: &mut PlayerFovReturnEvent) {
        let original = event.modifier;
        #[allow(unused_mut)]
        let mut state = self.state();
        let modules = state.behaviors.borrow();
        if state.registry.running(modules.no_fov_id) {
            event.modifier = modules
                .no_fov
                .get_fov_multiplier(event.modifier)
                .expect("LiquidBounce rejects an empty NoFOV custom range");
        }
        drop(modules);
        #[cfg(feature = "webview")]
        state.probe(
            "playerFovReturn",
            serde_json::json!(original),
            serde_json::json!(event.modifier),
        );
    }
    #[cfg(feature = "behavior-hooks")]
    fn lightmap_gamma_option(&self, _host: &dyn Host, event: &mut LightmapGammaOptionEvent) {
        let original = event.gamma;
        #[allow(unused_mut)]
        let mut state = self.state();
        let modules = state.behaviors.borrow();
        if let Some(gamma) = modules
            .full_bright
            .brightness_override(state.registry.running(modules.full_bright_id))
        {
            event.gamma = gamma;
        }else if modules.query_running(&state.registry,"XRay") && modules.xray.settings.full_bright {event.gamma=f32::MAX;}
        drop(modules);
        #[cfg(feature = "webview")]
        state.probe(
            "lightmapGammaOption",
            serde_json::json!(original),
            serde_json::json!(event.gamma),
        );
    }
    #[cfg(feature = "behavior-hooks")]
    fn mouse_scoping_condition(&self, _host: &dyn Host, event: &mut MouseScopingConditionEvent) {
        let state = self.state();
        if state.registry.running(state.behaviors.borrow().zoom_id) {
            event.scoping = true;
        }
    }
    #[cfg(feature = "behavior-hooks")]
    fn mouse_scrolling(
        &self,
        host: &mut dyn Host,
        event: &mut MouseScrollingEvent,
    ) -> horizon_client::mod_api::Flow {
        let air_place_cancel=self.air_place_cancel_scroll(host);
        let modifier_scan = self.state().behaviors.borrow().camera_clip.modifier.value;
        let modifier_down = host.client_keyboard_scan_down(modifier_scan);
        let raw_perspective = host.raw_camera_perspective();
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        let running = state.registry.running(modules.zoom_id);
        let steps = if event.accumulated[1] == 0 {
            -event.accumulated[0]
        } else {
            event.accumulated[1]
        };
        let clip_cancel = modules.camera_clip.can_scroll(
            modules.query_running(&state.registry, "CameraClip"),
            modifier_down,
            raw_perspective,
            modules.query_running(&state.registry, "FreeLook"),
        );
        let cancel = modules
            .zoom
            .on_hotbar_scroll(steps, running, &mut behaviors::current_time_millis)
            .cancel_hotbar_scroll;
        if cancel || clip_cancel || air_place_cancel {
            horizon_client::mod_api::Flow::Cancel
        } else {
            horizon_client::mod_api::Flow::Continue
        }
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_player_tick_post(&self, host: &mut dyn Host, _event: &mut ClientPlayerTickPostEvent) {
        self.potion_spoof_tick(host);
        self.packet_mine_tick(host);
        let Some(client) = host.client_state() else {
            return;
        };
        let options = raw_options(host);
        let [sprinting, _hurt_time] = host.client_combat_state();
        let effects = {
            let mut state = self.state();
            Self::observe_world(&mut state, &client, options);
            {
                let mut modules = state.behaviors.borrow_mut();
                let running = modules.query_running(&state.registry, "KeepSprint");
                modules.keep_sprint.post_tick(running, sprinting != 0);
            }
            #[cfg(feature = "webview")]
            state.probe(
                "clientPlayerTickPost",
                serde_json::Value::Null,
                serde_json::Value::Null,
            );
            let running = state
                .registry
                .running(state.behaviors.borrow().full_bright_id);
            let intent = state
                .behaviors
                .borrow_mut()
                .full_bright
                .on_player_post_tick(running);
            if let Some(intent) = intent {
                state.behaviors.borrow_mut().push_night_vision(intent);
            }
            state.take_effects()
        };
        apply_effects(self, host, effects);
    }

    fn screen_render_post(&self, host: &dyn Host, event: &mut ScreenRenderPostEvent<'_>) {
        #[cfg(feature = "webview")]
        if self.browser_screen_render(host, event) {
            return;
        }
        if event.screen.id != "mod" || host.mod_screen() != Some(MOD_ID) {
            return;
        }
        let observed = self.observed_state();
        let [x, y, width, height] = event.screen.rect;
        let left = x + 12.0;
        let top = y + 12.0;
        host.fill(
            event.batch,
            [
                left - 4.0,
                top - 4.0,
                (width - 16.0).max(0.0),
                (height - 16.0).max(0.0),
            ],
            0xdd101820,
        );
        let lines = [
            ui_label("LiquidBounce Rust port: integration status"),
            "Horizon 9f42064 / Minecraft 26.3".to_owned(),
            format!(
                "{}: {} / 236",
                ui_label("Native game implementations"),
                self.state().registry.modules().count()
            ),
            ui_label("Original CEF/Svelte ClickGUI: not connected"),
            format!(
                "{}: {}  {}: {}",
                ui_label("Observed world"),
                observed.world.world_present,
                ui_label("player"),
                observed.world.player_present
            ),
            format!(
                "{}: {}  {}: {}  {}: {}",
                ui_label("Ticks"),
                observed.ticks,
                ui_label("keys"),
                observed.keyboard_events,
                ui_label("mouse"),
                observed.mouse_events
            ),
            format!("{}: {}", ui_label("Disconnects"), observed.disconnects),
            ui_label("Escape closes this integration status screen."),
        ];
        for (index, line) in lines.iter().enumerate() {
            let at_y = top + index as f32 * 14.0;
            if at_y + 10.0 > y + height - 8.0 {
                break;
            }
            host.text(
                event.batch,
                line,
                [left, at_y],
                if index == 0 { 0x55aaff } else { 0xe6e6e6 },
                false,
            );
        }
    }
}

#[cfg(all(test, feature = "behavior-hooks"))]
#[path = "runtime_reentry_tests.rs"]
mod reentry_tests;
