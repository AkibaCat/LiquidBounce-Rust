// SPDX-License-Identifier: GPL-3.0-or-later
//! Actual game-module state, only compiled with the exact phase-hook patch.

use liquidbounce_behaviors::{
    FullBright, FullBrightMode, NightVisionIntent, NoFov, NoMissCooldown, Zoom, ZoomContext,
};
use liquidbounce_core::{
    BindAction, InputBind, InputKey, ModuleCallback, ModuleDefinition, ModuleId, ModuleRegistry,
};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy)]
pub struct RawOptions {
    pub fov: i32,
    pub gamma: f64,
    pub player_rotation: [f32; 2],
    pub actual_main_hand: i32,
    pub model_parts: u8,
}

pub fn current_time_millis() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_millis().min(i64::MAX as u128) as i64,
        Err(early) => -(early.duration().as_millis().min(i64::MAX as u128) as i64),
    }
}

pub struct BehaviorModules {
    pub no_fov_id: ModuleId,
    pub zoom_id: ModuleId,
    pub full_bright_id: ModuleId,
    pub no_bob_id: ModuleId,
    pub no_hurt_cam_id: ModuleId,
    pub query_ids: std::collections::BTreeMap<&'static str, ModuleId>,
    pub no_fov: NoFov,
    pub disabler: bool,
    pub server_crasher: bool,
    pub dupe: bool,
    pub teleport_reach: bool,
    pub phase: bool,
    pub notebot: bool,
    pub auto_config: bool,
    pub debug_recorder: bool,
    pub skin_changer: bool,
    pub zoom: Zoom,
    pub full_bright: FullBright,
    pub no_miss_cooldown: NoMissCooldown,
    pub no_swing: std::sync::Arc<liquidbounce_behaviors::no_swing::NoSwing>,
    pub more_carry: std::sync::Arc<liquidbounce_behaviors::more_carry::MoreCarry>,
    pub no_pitch_limit: std::sync::Arc<liquidbounce_behaviors::no_pitch_limit::NoPitchLimit>,
    pub no_interpolation: liquidbounce_behaviors::no_interpolation::NoInterpolation,
    pub no_slow_break: liquidbounce_behaviors::no_slow_break::NoSlowBreak,
    pub sprint: liquidbounce_behaviors::sprint::Sprint,
    pub directional_input: liquidbounce_behaviors::rotations::DirectionalInput,
    pub rotations:
        std::sync::Arc<std::sync::Mutex<liquidbounce_behaviors::rotations::RotationManager>>,
    pub anti_blind: liquidbounce_behaviors::visual_environment::AntiBlind,
    pub slots: std::sync::Arc<crate::slots::SlotService>,
    pub air_jump: liquidbounce_behaviors::air_jump::AirJump,
    pub no_push: liquidbounce_behaviors::no_push::NoPush,
    pub aspect: liquidbounce_behaviors::camera_control::Aspect,
    pub smooth_camera: liquidbounce_behaviors::camera_control::SmoothCamera,
    pub quick_perspective_swap: liquidbounce_behaviors::camera_control::QuickPerspectiveSwap,
    pub auto_respawn: liquidbounce_behaviors::auto_respawn::AutoRespawn,
    pub keep_sprint: liquidbounce_behaviors::keep_sprint::KeepSprint,
    pub crystal_view: liquidbounce_behaviors::crystal_view::CrystalView,
    pub fast_place: liquidbounce_behaviors::fast_place::FastPlace,
    pub auto_f5: liquidbounce_behaviors::auto_f5::AutoF5,
    pub reach: liquidbounce_behaviors::reach::Reach,
    pub no_pose: liquidbounce_behaviors::no_pose::NoPose,
    pub no_entity_interact: liquidbounce_behaviors::no_entity_interact::NoEntityInteract,
    pub no_block_interact: liquidbounce_behaviors::no_block_interact::NoBlockInteract,
    pub no_rotate_set: liquidbounce_behaviors::no_rotate_set::NoRotateSet,
    pub camera_clip: liquidbounce_behaviors::camera_clip::CameraClip,
    pub dank_bobbing: liquidbounce_behaviors::dank_bobbing::DankBobbing,
    pub multi_actions: liquidbounce_behaviors::multi_actions::MultiActions,
    pub twerk: liquidbounce_behaviors::twerk::Twerk,
    pub hitbox: liquidbounce_behaviors::hitbox::Hitbox,
    pub auto_weapon: liquidbounce_behaviors::auto_weapon::AutoWeapon,
    pub chams: liquidbounce_behaviors::chams::Chams,
    pub auto_tool: liquidbounce_behaviors::auto_tool::AutoTool,
    pub anti_afk: liquidbounce_behaviors::anti_afk::AntiAfk,
    pub trident_boost: liquidbounce_behaviors::trident_boost::TridentBoost,
    pub derp: liquidbounce_behaviors::derp::Derp,
    pub free_look: liquidbounce_behaviors::free_look::FreeLook,
    pub free_cam:crate::free_cam_runtime::FreeCamRuntime,
    pub hand_derp: std::sync::Arc<crate::hand_derp_packets::HandDerpService>,
    pub coroutine_ticker: crate::coroutine_ticker::CoroutineTicker,
    pub vehicle_boost: liquidbounce_behaviors::vehicle_boost::VehicleBoost,
    pub vehicle_control:liquidbounce_behaviors::vehicle_control::VehicleControl,
    pub block_bounce: liquidbounce_behaviors::block_bounce::BlockBounce,
    pub smart_eat: liquidbounce_behaviors::smart_eat::SmartEat,
    pub skin_derp: liquidbounce_behaviors::skin_derp::SkinDerp,
    pub entity_control: liquidbounce_behaviors::entity_control::EntityControl,
    pub snap_tap: liquidbounce_behaviors::snap_tap::SnapTap,
    pub block_walk: liquidbounce_behaviors::block_walk::BlockWalk,
    pub ghost_hand: liquidbounce_behaviors::ghost_hand::GhostHand,
    pub extended_firework: liquidbounce_behaviors::extended_firework::ExtendedFirework,
    pub strafe: liquidbounce_behaviors::strafe::Strafe,
    pub speed: crate::speed_runtime::SpeedRuntime,
    pub elytra_fly: bool,
    pub no_fall: bool,
    pub fast_break: std::sync::Arc<crate::fast_break_packets::FastBreakService>,
    pub timer: std::sync::Arc<crate::timer_runtime::TimerService>,
    pub better_inventory: liquidbounce_behaviors::better_inventory::BetterInventory,
    pub terrain_speed: liquidbounce_behaviors::terrain_speed::TerrainSpeed,
    pub auto_break: liquidbounce_behaviors::auto_break::AutoBreak,
    pub high_jump: liquidbounce_behaviors::high_jump::HighJump,
    pub no_slow:std::sync::Arc<crate::no_slow_runtime::NoSlowService>,
    pub no_web:crate::no_web_runtime::NoWebRuntime,
    pub liquid_walk:std::sync::Arc<crate::liquid_walk_runtime::LiquidWalkService>,
    pub step:std::sync::Arc<crate::step_runtime::StepService>,
    pub reverse_step: liquidbounce_behaviors::reverse_step::Settings,
    pub auto_walk: crate::auto_walk_runtime::AutoWalkSettings,
    pub parkour: crate::edge_assistance_runtime::ParkourSettings,
    pub safe_walk: crate::edge_assistance_runtime::SafeWalkSettings,
    pub anchor: liquidbounce_behaviors::anchor::Anchor,
    pub avoid_hazards: liquidbounce_behaviors::avoid_hazards::AvoidHazards,
    pub freeze: std::sync::Arc<crate::freeze_runtime::FreezeService>,
    pub reset_vl: liquidbounce_behaviors::reset_vl::ResetVl,
    pub item_scroller: liquidbounce_behaviors::item_scroller::ItemScroller,
    pub gui_closer: liquidbounce_behaviors::gui_closer::GuiCloser,
    pub text_field_protect: liquidbounce_behaviors::text_field_protect::TextFieldProtect,
    pub anti_hunger: std::sync::Arc<crate::anti_hunger_packets::AntiHungerService>,
    pub inventory: crate::inventory::InventoryActor,
    pub vomit: liquidbounce_behaviors::vomit::Vomit,
    pub elytra_swap: liquidbounce_behaviors::elytra_swap::ElytraSwap,
    pub inventory_packets: std::sync::Arc<crate::auto_tool_packets::InventoryPackets>,
    pub silent_hotbar: liquidbounce_behaviors::silent_hotbar_render::SilentHotbarRender,
    pub auto_fish: std::sync::Arc<crate::fishing_packets::AutoFishService>,
    pub press_times: liquidbounce_behaviors::fast_place::PressTimes,
    pub target_lock: liquidbounce_behaviors::target_lock::TargetLock,
    pub auto_disable: liquidbounce_behaviors::auto_disable::AutoDisable,
    pub auto_disable_packets: std::sync::Arc<crate::auto_disable_runtime::AutoDisablePackets>,
    pub potion_spoof: liquidbounce_behaviors::potion_spoof::PotionSpoof,
    pub no_clip: liquidbounce_behaviors::no_clip::NoClip,
    pub no_clip_packets: std::sync::Arc<crate::no_clip_packets::NoClipPackets>,
    pub true_sight: liquidbounce_behaviors::true_sight::TrueSight,
    pub plugins: std::sync::Arc<crate::plugins_runtime::PluginsService>,
    pub anti_cheat_detect:std::sync::Arc<crate::anti_cheat_detect_runtime::AntiCheatDetectService>,
    pub old_regen: liquidbounce_behaviors::old_regen::OldRegen,
    pub fast_use: liquidbounce_behaviors::fast_use::FastUse,
    pub fast_exp:crate::fast_exp_runtime::FastExpRuntime,
    pub auto_wind_charge:crate::auto_wind_charge_runtime::AutoWindChargeRuntime,
    pub auto_buff:crate::auto_buff_runtime::AutoBuffRuntime,
    pub auto_bow:crate::auto_bow_runtime::AutoBowRuntime,
    pub auto_pearl:crate::auto_pearl_runtime::AutoPearlRuntime,
    pub extinguish:crate::extinguish_runtime::ExtinguishRuntime,
    pub liquid_filler:crate::liquid_filler_runtime::LiquidFillerRuntime,
    pub hole_filler:crate::hole_filler_runtime::HoleFillerRuntime,
    pub surround:liquidbounce_behaviors::surround::Settings,
    pub block_in:liquidbounce_behaviors::block_in::Settings,
    pub bed_defender:crate::bed_defender_runtime::BedDefenderRuntime,
    pub block_placer_executor:crate::block_placer_runtime::Executor,
    pub easy_pearl:crate::easy_pearl_runtime::EasyPearlRuntime,
    pub elytra_target:crate::elytra_target_runtime::ElytraTargetRuntime,
    pub auto_rod:crate::auto_rod_runtime::AutoRodRuntime,
    pub auto_shoot:crate::auto_shoot_runtime::AutoShootRuntime,
    pub scaffold:crate::scaffold_runtime::ScaffoldRuntime,
    pub clutch:crate::scaffold_runtime::ClutchRuntime,
    pub aimbot:crate::aimbot_runtime::AimbotRuntime,
    pub target_strafe:crate::target_strafe_runtime::TargetStrafeRuntime,
    pub kill_aura:crate::kill_aura_runtime::KillAuraRuntime,
    pub tp_aura: bool,
    pub velocity: bool,
    pub packet_mine:crate::packet_mine_runtime::PacketMineRuntime,
    pub hud:crate::hud_runtime::HudRuntime,
    pub blink:std::sync::Arc<crate::blink_runtime::BlinkService>,
    pub fake_lag:std::sync::Arc<crate::fake_lag_runtime::FakeLagService>,
    pub sword_block:std::sync::Arc<crate::sword_block_runtime::SwordBlockService>,
    pub auto_dodge:crate::auto_dodge_runtime::AutoDodgeRuntime,
    pub air_place:liquidbounce_behaviors::air_place::Settings,
    pub flag_check:std::sync::Arc<crate::flag_check_runtime::FlagCheckService>,
    pub inventory_move:std::sync::Arc<crate::inventory_move_runtime::InventoryMoveService>,
    pub xray:crate::xray_runtime::XRayRuntime,
    pub anti_void:std::sync::Arc<crate::anti_void_runtime::AntiVoidService>,
    pub rescue_guards:std::sync::Arc<crate::anti_void_runtime::RescueGuards>,
    pub anti_bot:crate::anti_bot_runtime::AntiBotRuntime,
    pub notifier:crate::notifier_runtime::NotifierRuntime,
    pub anti_staff:std::sync::Arc<crate::anti_staff_runtime::AntiStaffService>,
    pub packet_logger:std::sync::Arc<crate::packet_logger_runtime::PacketLoggerService>,
    pub teams: liquidbounce_behaviors::teams::Teams,
    pub murder_mystery:liquidbounce_behaviors::murder_mystery::State,
    pub offhand:crate::offhand_runtime::OffhandRuntime,
    pub projectile_puncher:crate::projectile_puncher_runtime::ProjectilePuncherRuntime,
    pub spammer: crate::spammer_runtime::SpammerRuntime,
    pub auto_chat_game:crate::auto_chat_game_runtime::AutoChatGameRuntime,
    pub clip: liquidbounce_behaviors::clip::Clip,
    pub book_bot: crate::book_bot_runtime::BookBotRuntime,
    pub chest_cleaner:crate::chest_cleaner_runtime::ChestCleanerRuntime,
    pub auto_deposit:crate::auto_deposit_runtime::AutoDepositRuntime,
    pub replenish:crate::replenish_runtime::ReplenishRuntime,
    pub auto_crafter:liquidbounce_behaviors::auto_crafter::AutoCrafter,
    pub inventory_cleaner:crate::inventory_cleaner_runtime::InventoryCleanerRuntime,
    pub item_policy:crate::inventory_cleaner_runtime::ItemPolicy,
    pub auto_armor:crate::auto_armor_runtime::AutoArmorRuntime,
    pub auto_mob_heal:crate::auto_mob_heal_runtime::AutoMobHealRuntime,
    pub chest_stealer:crate::chest_stealer_runtime::ChestStealerRuntime,
    pub eagle:liquidbounce_behaviors::eagle::Settings,
    pub anti_exploit:liquidbounce_behaviors::anti_exploit::Settings,
    pub sneak: std::sync::Arc<crate::sneak_runtime::SneakService>,
    pub ping_spoof: std::sync::Arc<crate::ping_spoof_runtime::PingSpoofService>,
    pub spider: liquidbounce_behaviors::spider::Spider,
    pub middle_click_action: liquidbounce_behaviors::middle_click_action::MiddleClickAction,
    pub super_knockback: liquidbounce_behaviors::super_knockback::SuperKnockback,
    pub mace_kill: liquidbounce_behaviors::mace_kill::MaceKill,
    pub spear_kill: liquidbounce_behaviors::spear_kill::SpearKill,
    pub crosshair_clicker: liquidbounce_behaviors::crosshair_clicker::State,
    pub crosshair_break_observer: std::sync::Arc<crate::crosshair_clicker_runtime::BreakObserver>,
    pub criticals:std::sync::Arc<crate::criticals_runtime::CriticalsService>,
    pub combat_cooldown: std::sync::Arc<crate::super_knockback_runtime::CombatCooldown>,
    pub auto_account: liquidbounce_behaviors::auto_account::AutoAccount,
    pub report_helper:liquidbounce_behaviors::report_helper::State,
    pub auto_queue:liquidbounce_behaviors::auto_queue::State,
    pub macros:liquidbounce_behaviors::macros::State,
    pub client_commands:liquidbounce_behaviors::client_commands::Settings,
    pub mob_owners: liquidbounce_behaviors::mob_owners::MobOwners,
    pub combine_mobs: liquidbounce_behaviors::combine_mobs::CombineMobs,
    pub better_tab: liquidbounce_behaviors::better_tab::BetterTab,
    pub block_outline: liquidbounce_behaviors::block_outline::BlockOutline,
    pub damage_particles: liquidbounce_behaviors::damage_particles::DamageParticles,
    pub tracers: liquidbounce_behaviors::tracers::Tracers,
    pub esp:liquidbounce_behaviors::esp::Esp,
    pub radar:liquidbounce_behaviors::radar::Radar,
    pub item_esp:liquidbounce_behaviors::item_esp::ItemEsp,
    pub item_chams:liquidbounce_behaviors::item_chams::ItemChams,
    pub item_tags:liquidbounce_behaviors::item_tags::ItemTags,
    pub nametags:liquidbounce_behaviors::nametags::Nametags,
    pub better_chat:crate::better_chat_runtime::BetterChatRuntime,
    pub debug:crate::debug_runtime::DebugRuntime,
    pub click_gui:liquidbounce_behaviors::click_gui::ClickGui,
    pub bed_plates:liquidbounce_behaviors::bed_plates::BedPlates,
    pub new_chunks: liquidbounce_behaviors::new_chunks::NewChunks,
    pub stronghold_finder:liquidbounce_behaviors::stronghold_finder::StrongholdFinder,
    pub prophunt_esp: liquidbounce_behaviors::prophunt_esp::ProphuntEsp,
    pub void_esp: liquidbounce_behaviors::void_esp::VoidEsp,
    pub protection_zones: liquidbounce_behaviors::protection_zones::ProtectionZones,
    pub blink_esp: liquidbounce_behaviors::blink_esp::BlinkEsp,
    pub hole_esp: liquidbounce_behaviors::hole_esp::HoleEsp,
    pub particles: liquidbounce_behaviors::particles::Particles,
    pub trajectories: liquidbounce_behaviors::trajectories::Trajectories,
    pub custom_ambience: liquidbounce_behaviors::custom_ambience::CustomAmbience,
    pub tnt_timer: liquidbounce_behaviors::tnt_timer::TNTTimer,
    pub potion_fx: liquidbounce_behaviors::potion_fx::PotionFX,
    pub totem_effect: liquidbounce_behaviors::totem_effect::TotemEffect,
    pub hit_fx:liquidbounce_behaviors::hit_fx::HitFx,
    pub jump_effect: liquidbounce_behaviors::jump_effect::JumpEffect,
    pub name_protect:std::sync::Arc<crate::name_protect_runtime::NameProtectService>,
    pub inventory_tracker:crate::inventory_tracker_runtime::InventoryTrackerRuntime,
    pub logoff_spot:std::sync::Arc<crate::logoff_spot_runtime::LogoffSpotService>,
    pub storage_esp:liquidbounce_behaviors::storage_esp::StorageEsp,
    pub block_esp:liquidbounce_behaviors::block_esp::Settings,
    pub crosshair: liquidbounce_behaviors::crosshair::Crosshair,
    pub wings: liquidbounce_behaviors::wings::Wings,
    pub hats: liquidbounce_behaviors::hats::Hats,
    pub breadcrumbs: liquidbounce_behaviors::breadcrumbs::Breadcrumbs,
    pub auto_leave: liquidbounce_behaviors::auto_leave::AutoLeave,
    pub yggdrasil_force_slim: bool,
    pub damage: liquidbounce_behaviors::damage::Damage,
    pub kick_mode: liquidbounce_behaviors::auto_leave::KickMode,
    pub pending_one_shots: Vec<crate::one_shots_runtime::OneShotEffect>,
    pub pending_pitch_reset: bool,
    pub pending_fast_break_resets: usize,
    pub pending_slot_resets: Vec<u64>,
    pub pending_hand_sends: Vec<i32>,
    pub pending_model_parts: Vec<u8>,
    pub pending_night_vision: Vec<Option<i32>>,
    pub auto_shop: liquidbounce_behaviors::world_noop::WorldNoop,
    pub auto_build: liquidbounce_behaviors::world_noop::WorldNoop,
    pub auto_farm: liquidbounce_behaviors::world_noop::WorldNoop,
    pub crystal_aura: liquidbounce_behaviors::world_noop::WorldNoop,
    pub fucker: liquidbounce_behaviors::world_noop::WorldNoop,
    pub auto_trap: liquidbounce_behaviors::world_noop::WorldNoop,
    pub block_trap: liquidbounce_behaviors::world_noop::WorldNoop,
    pub nuker: liquidbounce_behaviors::world_noop::WorldNoop,
    pub tick_base: liquidbounce_behaviors::world_noop::WorldNoop,
    pub backtrack: liquidbounce_behaviors::world_noop::WorldNoop,
    pub fly: liquidbounce_behaviors::world_noop::WorldNoop,
    pub long_jump: liquidbounce_behaviors::world_noop::WorldNoop,
}

impl BehaviorModules {
    pub fn register(registry: &mut ModuleRegistry) -> Self {
        let no_fov_id = registry
            .register(ModuleDefinition::new("NoFOV"))
            .expect("unique module");
        let mut zoom = ModuleDefinition::new("Zoom");
        zoom.bind = Some(InputBind::new(InputKey::Unbound, BindAction::Hold));
        let zoom_id = registry.register(zoom).expect("unique module");
        let full_bright_id = registry
            .register(ModuleDefinition::new("FullBright"))
            .expect("unique module");
        let no_bob_id = registry
            .register(ModuleDefinition::new("NoBob"))
            .expect("unique module");
        let no_hurt_cam_id = registry
            .register(ModuleDefinition::new("NoHurtCam"))
            .expect("unique module");
        let query_ids = [
            "AutoShop",
            "AutoBuild",
            "AutoFarm",
            "CrystalAura",
            "Fucker",
            "AutoTrap",
            "BlockTrap",
            "Nuker",
            "TickBase",
            "Backtrack",
            "Fly",
            "LongJump",
            "NoJumpDelay",
            "AntiBounce",
            "AntiLevitation",
            "LiquidPlace",
            "PortalMenu",
            "Disabler",
            "ServerCrasher",
            "Dupe",
            "TeleportReach",
            "Phase",
            "AntiReducedDebugInfo",
            "NoMissCooldown",
            "AbortBreaking",
            "NoSwing",
            "MoreCarry",
            "NoPitchLimit",
            "NoInterpolation",
            "NoSlowBreak",
            "NoSlotSet",
            "Sprint",
            "AntiBlind",
            "AirJump",
            "NoPush",
            "Aspect",
            "SmoothCamera",
            "QuickPerspectiveSwap",
            "AutoRespawn",
            "KeepSprint",
            "CrystalView",
            "FastPlace",
            "AutoF5",
            "AutoFish",
            "Reach",
            "NoPose",
            "NoEntityInteract",
            "NoBlockInteract",
            "NoRotateSet",
            "CameraClip",
            "DankBobbing",
            "MultiActions",
            "Twerk",
            "Notebot",
            "Hitbox",
            "AutoWeapon",
            "Chams",
            "AutoTool",
            "AntiAFK",
            "TridentBoost",
            "SleepWalker",
            "Derp",
            "HandDerp",
            "FreeLook",
            "FreeCam",
            "VehicleBoost",
            "VehicleControl",
            "BlockBounce",
            "SmartEat",
            "SkinDerp",
            "SkinChanger",
            "EntityControl",
            "SnapTap",
            "BlockWalk",
            "GhostHand",
            "ExtendedFirework",
            "Strafe",
            "Speed",
            "ElytraFly",
            "NoFall",
            "ElytraRecast",
            "FastBreak",
            "Timer",
            "BetterInventory",
            "TerrainSpeed",
            "AutoBreak",
            "HighJump",
            "NoSlow",
            "NoWeb",
            "LiquidWalk",
            "Step",
            "ReverseStep",
            "AutoWalk",
            "Parkour",
            "SafeWalk",
            "Anchor",
            "AvoidHazards",
            "Freeze",
            "ResetVL",
            "NameCollector",
            "Kick",
            "AutoLeave",
            "Damage",
            "YggdrasilSignatureFix",
            "OldRegen",
            "AutoDisable",
            "AutoAccount",
            "Breadcrumbs",
            "Hats",
            "Wings",
            "Crosshair",
            "StorageESP",
            "BlockESP",
            "LogoffSpot",
            "InventoryTracker",
            "NameProtect",
            "Particles",
            "Trajectories",
            "CustomAmbience",
            "TNTTimer",
            "PotionFX",
            "TotemEffect",
            "HitFX",
            "JumpEffect",
            "HoleESP",
            "ProtectionZones",
            "ProphuntESP",
            "VoidESP",
            "NewChunks",
            "StrongholdFinder",
            "Tracers",
            "ESP",
            "Radar",
            "ItemESP",
            "ItemChams",
            "ItemTags",
            "Nametags",
            "BetterChat",
            "Debug",
            "ClickGUI",
            "BedPlates",
            "DamageParticles",
            "BlockOutline",
            "BetterTab",
            "CombineMobs",
            "MobOwners",
            "ReportHelper",
            "AutoQueue",
            "Macros",
            "AutoConfig",
            "DebugRecorder",
            "MiddleClickAction",
            "SuperKnockback",
            "MaceKill",
            "SpearKill",
            "AutoClicker",
            "TriggerBot",
            "Criticals",
            "Spider",
            "FastUse",
            "FastExp",
            "AutoWindCharge",
            "AutoBuff",
            "AutoBow",
            "AutoPearl",
            "Extinguish",
            "LiquidFiller",
            "HoleFiller",
            "Surround",
            "BlockIn",
            "BedDefender",
            "EasyPearl",
            "AutoRod",
            "AutoShoot",
            "Scaffold",
            "Clutch",
            "FakeLag",
            "SwordBlock",
            "Aimbot",
            "TargetStrafe",
            "KillAura",
            "TpAura",
            "Velocity",
            "ElytraTarget",
            "PacketMine",
            "HUD",
            "Blink",
            "AutoDodge",
            "AirPlace",
            "FlagCheck",
            "PacketLogger",
            "AntiStaff",
            "Notifier",
            "AntiBot",
            "AntiVoid",
            "XRay",
            "InventoryMove",
            "Teams",
            "MurderMystery",
            "Offhand",
            "ProjectilePuncher",
            "Spammer",
            "AutoChatGame",
            "Clip",
            "BookBot",
            "ChestCleaner",
            "AutoDeposit",
            "Replenish",
            "AutoCrafter",
            "InventoryCleaner",
            "AutoArmor",
            "AutoMobHeal",
            "ChestStealer",
            "Eagle",
            "AntiExploit",
            "Sneak",
            "PingSpoof",
            "Plugins",
            "AntiCheatDetect",
            "TrueSight",
            "NoClip",
            "PotionSpoof",
            "ItemScroller",
            "GUICloser",
            "TextFieldProtect",
            "TargetLock",
            "AntiHunger",
            "Vomit",
            "ElytraSwap",
            "VehicleOneHit",
            "SilentHotbar",
        ]
        .into_iter()
        .map(|name| {
            (
                name,
                registry
                    .register({
                        let mut definition = ModuleDefinition::new(name);
                        if matches!(name, "QuickPerspectiveSwap" | "FreeLook" | "Anchor" | "FastExp") {
                            definition.bind =
                                Some(InputBind::new(InputKey::Unbound, BindAction::Hold));
                            definition.disable_on_quit = true;
                        }
                        if name=="ClickGUI"{definition.disable_activation=true;definition.bind=crate::input::configured_bind("key.keyboard.right.shift").map(|key|InputBind::new(key,BindAction::Toggle));}
                        if matches!(name, "NameCollector" | "Kick" | "Damage") {
                            definition.disable_activation = true;
                        }
                        if matches!(name, "Timer" | "ElytraSwap" | "Plugins" | "OldRegen" | "Spammer") {
                            definition.disable_on_quit = true;
                        }
                        if matches!(name,"Freeze"|"BookBot"|"FreeCam") { definition.disable_on_quit = true; }
                        definition
                    })
                    .expect("unique implemented module"),
            )
        })
        .collect();
        let coroutine_ticker = crate::coroutine_ticker::CoroutineTicker::default();
        let timer = std::sync::Arc::new(crate::timer_runtime::TimerService::default());
        let liquid_walk=std::sync::Arc::new(crate::liquid_walk_runtime::LiquidWalkService::new(timer.clone()));
        let step=std::sync::Arc::new(crate::step_runtime::StepService::new(timer.clone()));
        let freeze = std::sync::Arc::new(crate::freeze_runtime::FreezeService::new(timer.clone()));
        let combat_cooldown=std::sync::Arc::new(crate::super_knockback_runtime::CombatCooldown::new(timer.clone()));
        let fake_lag=std::sync::Arc::new(crate::fake_lag_runtime::FakeLagService::default());
        let blink=std::sync::Arc::new(crate::blink_runtime::BlinkService::default());
        let rescue_guards=std::sync::Arc::new(crate::anti_void_runtime::RescueGuards::default());
        let anti_void=std::sync::Arc::new(crate::anti_void_runtime::AntiVoidService::new(rescue_guards.clone()));
        let kill_aura_blocking=std::sync::Arc::new(crate::kill_aura_runtime::KillAuraBlocking::default());
        let no_slow=std::sync::Arc::new(crate::no_slow_runtime::NoSlowService::default());
        let scaffold=crate::scaffold_runtime::ScaffoldRuntime::default();
        let criticals=std::sync::Arc::new(crate::criticals_runtime::CriticalsService::new(timer.clone(),combat_cooldown.clone(),freeze.clone(),blink.clone(),no_slow.clone(),anti_void.clone(),fake_lag.clone(),kill_aura_blocking.clone(),scaffold.blink.clone()));
        Self {
            no_fov_id,
            zoom_id,
            full_bright_id,
            no_bob_id,
            no_hurt_cam_id,
            query_ids,
            no_fov: NoFov::default(),
            disabler: false,
            server_crasher: false,
            dupe: false,
            teleport_reach: false,
            phase: false,
            notebot: false,
            auto_config: false,
            debug_recorder: false,
            skin_changer: false,
            zoom: Zoom::default(),
            full_bright: FullBright::new(),
            no_miss_cooldown: NoMissCooldown::default(),
            no_swing: Default::default(),
            more_carry: Default::default(),
            no_pitch_limit: Default::default(),
            no_interpolation: Default::default(),
            no_slow_break: Default::default(),
            sprint: Default::default(),
            directional_input: Default::default(),
            rotations: Default::default(),
            anti_blind: Default::default(),
            slots: Default::default(),
            air_jump: Default::default(),
            no_push: Default::default(),
            aspect: Default::default(),
            smooth_camera: Default::default(),
            quick_perspective_swap: Default::default(),
            auto_respawn: Default::default(),
            keep_sprint: Default::default(),
            crystal_view: Default::default(),
            fast_place: Default::default(),
            auto_f5: Default::default(),
            reach: Default::default(),
            no_pose: Default::default(),
            no_entity_interact: Default::default(),
            no_block_interact: Default::default(),
            no_rotate_set: Default::default(),
            camera_clip: Default::default(),
            dank_bobbing: Default::default(),
            multi_actions: Default::default(),
            twerk: Default::default(),
            hitbox: Default::default(),
            auto_weapon: Default::default(),
            chams: Default::default(),
            auto_tool: Default::default(),
            anti_afk: Default::default(),
            trident_boost: Default::default(),
            derp: Default::default(),
            free_look: Default::default(),
            free_cam:Default::default(),
            hand_derp: Default::default(),
            coroutine_ticker: coroutine_ticker.clone(),
            vehicle_boost: Default::default(),
            vehicle_control:Default::default(),
            block_bounce: Default::default(),
            smart_eat: Default::default(),
            skin_derp: Default::default(),
            entity_control: Default::default(),
            snap_tap: Default::default(),
            block_walk: Default::default(),
            ghost_hand: Default::default(),
            extended_firework: Default::default(),
            strafe: Default::default(),
            speed: Default::default(),
            elytra_fly: false,
            no_fall: false,
            fast_break: Default::default(),
            timer: timer.clone(),
            better_inventory: Default::default(),
            terrain_speed: Default::default(),
            auto_break: liquidbounce_behaviors::auto_break::AutoBreak,
            high_jump: Default::default(),
            no_slow,
            no_web:Default::default(),
            liquid_walk,
            step,
            reverse_step: Default::default(),
            auto_walk: Default::default(),
            parkour: Default::default(),
            safe_walk: Default::default(),
            anchor: Default::default(),
            avoid_hazards: Default::default(),
            freeze,
            reset_vl: Default::default(),
            item_scroller: Default::default(),
            gui_closer: Default::default(),
            text_field_protect: Default::default(),
            anti_hunger: Default::default(),
            inventory: Default::default(),
            vomit: Default::default(),
            elytra_swap: Default::default(),
            inventory_packets: Default::default(),
            silent_hotbar: Default::default(),
            auto_fish: std::sync::Arc::new(crate::fishing_packets::AutoFishService::with_ticker(
                coroutine_ticker,
            )),
            press_times: Default::default(),
            target_lock: Default::default(),
            auto_disable: Default::default(),
            auto_disable_packets: Default::default(),
            potion_spoof: Default::default(),
            no_clip: Default::default(),
            no_clip_packets: Default::default(),
            true_sight: Default::default(),
            plugins: Default::default(),
            anti_cheat_detect:Default::default(),
            old_regen: Default::default(),
            fast_use: Default::default(),
            fast_exp:Default::default(),
            auto_wind_charge:Default::default(),
            auto_buff:Default::default(),
            auto_bow:Default::default(),
            auto_pearl:Default::default(),
            extinguish:Default::default(),
            liquid_filler:Default::default(),
            hole_filler:Default::default(),
            surround:Default::default(),
            block_in:Default::default(),
            bed_defender:Default::default(),
            block_placer_executor:Default::default(),
            easy_pearl:Default::default(),
            elytra_target:Default::default(),
            auto_rod:Default::default(),
            auto_shoot:Default::default(),
            scaffold,
            clutch:Default::default(),
            aimbot:Default::default(),
            target_strafe:Default::default(),
            kill_aura:crate::kill_aura_runtime::KillAuraRuntime::new(kill_aura_blocking),
            tp_aura: false,
            velocity: false,
            packet_mine:Default::default(),
            hud:Default::default(),
            blink,
            fake_lag,
            sword_block:Default::default(),
            auto_dodge:Default::default(),
            air_place:Default::default(),
            flag_check:Default::default(),
            packet_logger:Default::default(),
            anti_staff:Default::default(),
            notifier:Default::default(),
            anti_bot:Default::default(),
            anti_void,
            rescue_guards,
            xray:Default::default(),
            inventory_move:Default::default(),
            teams: Default::default(),
            murder_mystery:Default::default(),
            offhand:Default::default(),
            projectile_puncher:Default::default(),
            spammer: Default::default(),
            auto_chat_game:Default::default(),
            clip: Default::default(),
            book_bot: Default::default(),
            chest_cleaner:Default::default(),
            auto_deposit:Default::default(),
            replenish:Default::default(),
            auto_crafter:Default::default(),
            inventory_cleaner:Default::default(),
            item_policy:crate::inventory_cleaner_runtime::ItemPolicy {armor:liquidbounce_behaviors::auto_armor::AutoArmor::default().policy(),..Default::default()},
            auto_armor:Default::default(),
            auto_mob_heal:Default::default(),
            chest_stealer:Default::default(),
            eagle:Default::default(),
            anti_exploit:Default::default(),
            sneak: Default::default(),
            ping_spoof: Default::default(),
            spider: Default::default(),
            middle_click_action: Default::default(),
            super_knockback: Default::default(),
            mace_kill: Default::default(),
            spear_kill: Default::default(),
            crosshair_clicker: Default::default(),
            crosshair_break_observer: Default::default(),
            combat_cooldown,
            criticals,
            auto_account: Default::default(),
            report_helper:Default::default(),
            auto_queue:Default::default(),
            macros:Default::default(),
            client_commands:Default::default(),
            mob_owners: Default::default(),
            combine_mobs: Default::default(),
            better_tab: Default::default(),
            block_outline: Default::default(),
            damage_particles: Default::default(),
            tracers: Default::default(),
            esp:Default::default(),
            radar:Default::default(),
            item_esp:Default::default(),
            item_chams:Default::default(),
            item_tags:Default::default(),
            nametags:Default::default(),
            better_chat:Default::default(),
            debug:Default::default(),
            click_gui:Default::default(),
            bed_plates:Default::default(),
            new_chunks: Default::default(),
            stronghold_finder:Default::default(),
            prophunt_esp: Default::default(),
            void_esp: Default::default(),
            protection_zones: Default::default(),
            blink_esp: Default::default(),
            hole_esp: Default::default(),
            particles: Default::default(),
            trajectories:Default::default(),
            custom_ambience:Default::default(),
            tnt_timer:Default::default(),
            potion_fx:Default::default(),
            totem_effect:Default::default(),
            hit_fx:Default::default(),
            jump_effect:Default::default(),
            name_protect:Default::default(),
            inventory_tracker:Default::default(),
            logoff_spot: Default::default(),
            storage_esp:Default::default(),
            block_esp:Default::default(),
            crosshair: Default::default(),
            wings: Default::default(),
            hats: Default::default(),
            breadcrumbs: Default::default(),
            auto_leave: Default::default(),
            yggdrasil_force_slim: true,
            damage: Default::default(),
            kick_mode: Default::default(),
            pending_one_shots: Vec::new(),
            pending_pitch_reset: false,
            pending_fast_break_resets: 0,
            pending_slot_resets: Vec::new(),
            pending_hand_sends: Vec::new(),
            pending_model_parts: Vec::new(),
            pending_night_vision: Vec::new(),
            auto_shop: Default::default(),
            auto_build: Default::default(),
            auto_farm: Default::default(),
            crystal_aura: Default::default(),
            fucker: Default::default(),
            auto_trap: Default::default(),
            block_trap: Default::default(),
            nuker: Default::default(),
            tick_base: Default::default(),
            backtrack: Default::default(),
            fly: Default::default(),
            long_jump: Default::default(),
        }
    }

    pub fn ids(&self) -> Vec<ModuleId> {
        let mut ids = vec![
            self.full_bright_id,
            self.no_fov_id,
            self.zoom_id,
            self.no_bob_id,
            self.no_hurt_cam_id,
        ];
        ids.extend(self.query_ids.values().copied());
        ids
    }

    pub fn query_running(&self, registry: &ModuleRegistry, name: &str) -> bool {
        if name=="ElytraTarget"&&!self.elytra_target.behavior.gliding{return false;}
        if name=="Scaffold"&&self.query_ids.get("Clutch").is_some_and(|id|registry.running(*id))&&!self.clutch.path.is_null(){return true;}
        self.query_ids
            .get(name)
            .is_some_and(|id| registry.running(*id))
    }

    pub fn rotation_provider_running(&self, registry: &ModuleRegistry, provider: u64) -> bool {
        match provider {
            1 => self.query_running(registry, "Sprint"),
            2 => self.query_running(registry, "NoRotateSet"),
            3 => self.query_running(registry, "AntiAFK"),
            crate::auto_deposit_runtime::ROTATION_PROVIDER => self.query_running(registry,"AutoDeposit"),
            crate::chest_stealer_runtime::ROTATION_PROVIDER => self.query_running(registry,"ChestStealer"),
            crate::projectile_puncher_runtime::ROTATION_PROVIDER=>self.query_running(registry,"ProjectilePuncher"),
            crate::free_cam_runtime::ROTATION_PROVIDER=>self.free_cam.behavior.active(self.query_running(registry,"FreeCam")),
            crate::fast_exp_runtime::ROTATION_PROVIDER=>self.query_running(registry,"FastExp"),
            crate::auto_wind_charge_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoWindCharge"),
            crate::auto_buff_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoBuff"),
            crate::auto_bow_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoBow"),
            crate::auto_pearl_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoPearl"),
            crate::extinguish_runtime::ROTATION_PROVIDER=>self.query_running(registry,"Extinguish"),
            crate::block_placer_runtime::PLACEMENT_PROVIDER|crate::block_placer_runtime::CRYSTAL_PROVIDER=>self.query_running(registry,"LiquidFiller"),
            crate::hole_filler_runtime::PLACEMENT_PROVIDER|crate::hole_filler_runtime::CRYSTAL_PROVIDER=>self.query_running(registry,"HoleFiller"),
            crate::bed_defender_runtime::PLACEMENT_PROVIDER|crate::bed_defender_runtime::CRYSTAL_PROVIDER=>self.query_running(registry,"BedDefender"),
            crate::easy_pearl_runtime::ROTATION_PROVIDER=>self.query_running(registry,"EasyPearl"),
            crate::elytra_target_runtime::ROTATION_PROVIDER=>self.query_running(registry,"ElytraTarget"),
            crate::auto_rod_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoRod"),
            crate::kill_aura_runtime::ROTATION_PROVIDER=>self.query_running(registry,"KillAura"),
            crate::auto_shoot_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoShoot"),
            crate::scaffold_runtime::ROTATION_PROVIDER=>self.query_running(registry,"Scaffold"),
            crate::no_web_runtime::ROTATION_PROVIDER => self.query_running(registry,"NoWeb"),
            crate::auto_walk_runtime::ROTATION_PROVIDER => self.query_running(registry, "AutoWalk"),
            crate::edge_assistance_runtime::ROTATION_PROVIDER => self.query_running(registry, "Parkour"),
            _ => false,
        }
    }

    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {
        self.auto_disable_packets.set_running(self.query_running(registry,"AutoDisable"));
        self.inventory_move.publish(self.query_running(registry,"InventoryMove"),self.inventory.manager.inventory_open_server_side);
        self.anti_staff.configure(self.query_ids.get("AntiStaff").is_some_and(|id|registry.enabled_without_world(*id)),self.query_running(registry,"AntiStaff"));
        self.packet_logger.configure(self.query_ids.get("PacketLogger").is_some_and(|id|registry.enabled_without_world(*id)));
        self.logoff_spot.set_running(self.query_running(registry,"LogoffSpot"));
        self.flag_check.configure(self.query_running(registry,"FlagCheck"));
        self.blink.set_running(self.query_running(registry,"Blink"));
        self.fake_lag.set_running(self.query_running(registry,"FakeLag"));
        self.sword_block.set_running(self.query_running(registry,"SwordBlock"));
        self.name_protect.set_running(self.query_running(registry,"NameProtect"));
        self.anti_void.set_context(self.query_running(registry,"AntiVoid"),self.query_running(registry,"Fly"),self.query_running(registry,"Clutch"),self.query_running(registry,"Scaffold"));
        self.timer.set_provider_running(crate::auto_dodge_runtime::TIMER_PROVIDER,self.query_running(registry,"AutoDodge")&&!self.query_running(registry,"Blink")&&!self.murder_mystery.disallows_arrow_dodge(self.query_running(registry,"MurderMystery")));
        self.no_slow.set_running(self.query_running(registry,"NoSlow"));
        let a=&self.anti_exploit;
        horizon_client::anti_exploit::configure(self.query_running(registry,"AntiExploit"),a.limits,a.cancel_demo,a.ignore_protocol,a.max_obfuscated,a.notify,a.scoreboard_fix);
        self.criticals.set_running(self.query_running(registry,"Criticals"));
        self.crosshair_break_observer.set_running(self.query_running(registry,"AutoClicker"));
        self.liquid_walk.set_running(self.query_running(registry,"LiquidWalk"));
        self.sneak.set_running(self.query_running(registry,"Sneak"));
        self.step.set_running(self.query_running(registry,"Step"),self.query_running(registry,"Speed"));
        self.freeze.set_running(self.query_running(registry, "Freeze"));
        self.ping_spoof.set_running(self.query_running(registry,"PingSpoof"));
        self.timer.set_provider_running(crate::fast_use_runtime::TIMER_PROVIDER, self.query_running(registry, "FastUse"));
        self.timer.set_provider_running(
            crate::old_regen_runtime::TIMER_PROVIDER,
            self.query_running(registry, "OldRegen"),
        );
        self.no_clip_packets
            .set_running(self.query_running(registry, "NoClip"));
        self.plugins
            .set_running(self.query_running(registry, "Plugins"));
        self.auto_disable_packets
            .set_running(self.query_running(registry, "AutoDisable"));
        horizon_client::skins::liquidbounce_yggdrasil_policy(
            self.query_running(registry, "YggdrasilSignatureFix"),
            self.yggdrasil_force_slim,
        );
        self.anti_hunger
            .set_running(self.query_running(registry, "AntiHunger"));
        self.no_swing
            .set_running(self.query_running(registry, "NoSwing"));
        self.more_carry
            .set_running(self.query_running(registry, "MoreCarry"));
        self.no_pitch_limit
            .set_running(self.query_running(registry, "NoPitchLimit"));
        self.slots
            .no_slot_set
            .set_running(self.query_running(registry, "NoSlotSet"));
        self.auto_fish
            .set_running(self.query_running(registry, "AutoFish"));
        self.hand_derp
            .set_running(self.query_running(registry, "HandDerp"));
        self.fast_break
            .set_running(self.query_running(registry, "FastBreak"));
        self.timer
            .set_running(self.query_running(registry, "Timer"));
        self.timer.set_provider_running(
            crate::reset_vl_runtime::RESET_VL_TIMER_PROVIDER,
            self.query_running(registry, "ResetVL"),
        );
    }

    /// The two original HEAD cancellations are independent; do not touch the
    /// avatar history, View Bobbing option, other native shake or nausea.
    pub fn camera_bob_effects(
        &self,
        registry: &ModuleRegistry,
        hurt: bool,
        view: bool,
    ) -> (bool, bool) {
        (
            hurt && !registry.running(self.no_hurt_cam_id),
            view && !registry.running(self.no_bob_id) && !(self.query_running(registry,"StorageESP") && self.storage_esp.active(self.query_running(registry,"ChestStealer")) && self.storage_esp.show_tracers()),
        )
    }

    pub fn default_fov(&self, registry: &ModuleRegistry, option: i32) -> i32 {
        if registry.running(self.no_fov_id) {
            self.no_fov.get_fov(option)
        } else {
            option
        }
    }

    pub fn callback(
        &mut self,
        registry: &mut ModuleRegistry,
        callback: ModuleCallback,
        options: RawOptions,
    ) {
        match callback {
            ModuleCallback::OnDisabled(id) if self.query_ids.get("HoleFiller")==Some(&id)=>{self.hole_filler.placer.disable(current_time_millis());},
            ModuleCallback::OnDisabled(id) | ModuleCallback::CancelSequences(id)
                if ["Disabler","ServerCrasher","Dupe","TeleportReach","Phase","Notebot","AutoConfig","DebugRecorder","SkinChanger"]
                    .iter().any(|name| self.query_ids.get(name) == Some(&id)) => {}
            ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoAccount") == Some(&id) => { self.auto_account.cancel(); }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoAccount") == Some(&id) => { self.auto_account.cancel(); }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("PacketMine") == Some(&id) => {
                self.packet_mine.reset();
            }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("BedDefender")==Some(&id)=>{self.bed_defender.placer.disable(current_time_millis());self.bed_defender.self_bed.disable();},
            ModuleCallback::OnDisabled(id) if self.query_ids.get("LiquidFiller")==Some(&id)=>{self.liquid_filler.placer.disable(current_time_millis());},
            ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoMobHeal")==Some(&id)=>{self.auto_mob_heal.clear();self.pending_slot_resets.push(crate::auto_mob_heal_runtime::REQUESTER);},
            ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoMobHeal")==Some(&id)=>self.auto_mob_heal.clear(),
            ModuleCallback::OnEnabled(id) if self.query_ids.get("Scaffold")==Some(&id)=>{self.scaffold.enable_pending=true;if self.scaffold.flag("Technique.Normal.Eagle.Enabled"){self.scaffold.eagle.placed=0;}self.scaffold.head_cooldown=0;self.scaffold.move_ticks=0;},
            ModuleCallback::OnDisabled(id) if self.query_ids.get("Scaffold")==Some(&id)=>{if self.scaffold.flag("Strafe.Enabled")&&self.scaffold.flag("Strafe.Hypixel"){self.scaffold.strafe_disable_pending=true;}self.scaffold.reset();self.scaffold.blink.state.lock().expect("Scaffold Blink").running=false;self.pending_slot_resets.push(crate::scaffold_runtime::REQUESTER);},
            ModuleCallback::CancelSequences(id) if self.query_ids.get("Scaffold")==Some(&id)=>{self.scaffold.waiting=false;self.scaffold.tower.pending.clear();},
            ModuleCallback::OnDisabled(id) if self.query_ids.get("Clutch")==Some(&id)=>self.clutch.disabled(),
            ModuleCallback::OnEnabled(id) if self.query_ids.get("AutoChatGame")==Some(&id)=>{self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::AutoChatGameEnabled);}
            ModuleCallback::OnEnabled(id) if self.query_ids.get("ClickGUI")==Some(&id)=>self.click_gui.enabled(registry.world().in_game()),
            ModuleCallback::OnEnabled(id) if self.query_ids.get("Debug")==Some(&id)=>self.debug.enabled(),
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("Debug")==Some(&id)=>self.debug.clear(),
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("Nametags")==Some(&id)=>self.nametags.clear(),
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("BetterChat")==Some(&id)=>self.better_chat.cancel(),
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("BedPlates")==Some(&id)=>self.bed_plates.disable(),
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("ItemTags")==Some(&id)=>self.item_tags.clear(),
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoChatGame")==Some(&id)=>self.auto_chat_game.cancel(),
            ModuleCallback::OnEnabled(id) if self.query_ids.get("AntiVoid")==Some(&id)=>{self.anti_void.state.lock().expect("AntiVoid").enabled();}
            ModuleCallback::OnDisabled(id) if self.query_ids.get("AntiBot")==Some(&id)=>{self.anti_bot.state.reset();self.notifier.bot_entities.clear();}
            ModuleCallback::CancelSequences(id) if self.query_ids.get("AntiBot")==Some(&id)=>{self.anti_bot.state.matrix.continuation=None;}
            ModuleCallback::CancelSequences(id) if self.query_ids.get("FastUse") == Some(&id) => { self.fast_use.cancel(); }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("Breadcrumbs") == Some(&id) => { self.breadcrumbs.clear(); }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("DamageParticles") == Some(&id) => { self.damage_particles.clear(); }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("Particles") == Some(&id) => { self.particles.clear(); }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("CustomAmbience")==Some(&id)=>self.custom_ambience.clear(),
            ModuleCallback::OnDisabled(id) if self.query_ids.get("PotionFX")==Some(&id)=>self.potion_fx.clear(),
            ModuleCallback::OnDisabled(id) if self.query_ids.get("TotemEffect")==Some(&id)=>self.totem_effect.clear(),
            ModuleCallback::OnDisabled(id) if self.query_ids.get("JumpEffect")==Some(&id)=>self.jump_effect.clear(),
            ModuleCallback::OnDisabled(id) if self.query_ids.get("Tracers") == Some(&id) => { self.tracers.clear(); }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("ESP")==Some(&id)=>self.esp.clear(),
            ModuleCallback::OnDisabled(id) if self.query_ids.get("Radar")==Some(&id)=>self.radar.clear(),
            ModuleCallback::OnDisabled(id) if self.query_ids.get("ItemESP")==Some(&id)=>self.item_esp.clear(),
            ModuleCallback::OnDisabled(id) if self.query_ids.get("ItemChams")==Some(&id)=>self.item_chams.clear(),
            ModuleCallback::OnDisabled(id) if self.query_ids.get("NewChunks") == Some(&id) => { self.new_chunks.clear(); }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("StrongholdFinder")==Some(&id)=>self.stronghold_finder.clear(),
            ModuleCallback::OnDisabled(id) if self.query_ids.get("ProphuntESP") == Some(&id) => { self.prophunt_esp.render.clear_silently(current_time_millis()); }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("VoidESP") == Some(&id) => { self.void_esp.disabled(current_time_millis()); }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("CombineMobs") == Some(&id) => {
                self.combine_mobs.disabled();
            }
            ModuleCallback::CancelSequences(id)
                if self.query_ids.get("MiddleClickAction") == Some(&id) =>
            {
                self.middle_click_action.pearl_disable();
                self.pending_slot_resets
                    .push(crate::middle_click_action_runtime::PEARL_REQUESTER);
            }
            ModuleCallback::OnEnabled(id) if self.query_ids.get("AutoQueue")==Some(&id)=>{self.auto_queue.enabled();if self.auto_queue.settings.preset==1{self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::AutoQueueEnabled);}}
            ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoQueue")==Some(&id)=>{self.auto_queue.disabled();if self.auto_queue.reset_gomme{self.pending_slot_resets.push(crate::auto_queue_runtime::GOMME_REQUESTER);self.auto_queue.reset_gomme=false;}}
            ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoQueue")==Some(&id)=>{self.auto_queue.cancel();}
            ModuleCallback::OnEnabled(id) if self.query_ids.get("Offhand")==Some(&id)=>{let ca=self.query_running(registry,"CrystalAura");self.offhand.enable(ca);}
            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("Offhand")==Some(&id)=>{self.offhand.cancel(&mut self.inventory);self.item_policy.offhand_operating=false;}
            ModuleCallback::OnDisabled(id) if self.query_ids.get("MurderMystery")==Some(&id)=>{self.murder_mystery.reset();self.auto_dodge.murder_mystery_disallows=false;}
            ModuleCallback::OnEnabled(id) if self.query_ids.get("MurderMystery")==Some(&id)=>{self.auto_dodge.murder_mystery_disallows=self.murder_mystery.disallows_arrow_dodge(true);}
            ModuleCallback::OnDisabled(id) if self.query_ids.get("Macros")==Some(&id)=>{self.pending_slot_resets.push(crate::macros_runtime::REQUESTER);}
            ModuleCallback::CancelSequences(id) if self.query_ids.get("Macros")==Some(&id)=>{self.macros.cancel();}
            ModuleCallback::OnDisabled(id) if self.query_ids.get("ReportHelper")==Some(&id)=>{self.report_helper.disabled();}
            ModuleCallback::CancelSequences(id) if self.query_ids.get("ReportHelper")==Some(&id)=>{self.report_helper.cancel();}
            ModuleCallback::CancelSequences(id)
                if self.query_ids.get("AutoAccount") == Some(&id) =>
            {
                self.auto_account.cancel();
            }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoAccount") == Some(&id) => {
                self.auto_account.cancel();
            }
            ModuleCallback::CancelSequences(id) if self.query_ids.get("SuperKnockback") == Some(&id) => {
                self.super_knockback.cancel();
            }
            ModuleCallback::CancelSequences(id) if self.query_ids.get("Spider") == Some(&id) => {
                self.spider.cancel()
            }
            ModuleCallback::OnEnabled(id) if self.query_ids.get("OldRegen") == Some(&id) => {
                self.pending_one_shots
                    .push(crate::one_shots_runtime::OneShotEffect::OldRegenEnabled);
            }
            ModuleCallback::EnabledEffect(id) if self.query_ids.get("Plugins") == Some(&id) => {
                if registry.module(id).is_some_and(|module| module.enabled()) {
                    self.pending_one_shots
                        .push(crate::one_shots_runtime::OneShotEffect::PluginsStart);
                }
            }
            ModuleCallback::CancelSequences(id) if self.query_ids.get("Plugins") == Some(&id) => {
                self.plugins.cancel()
            }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("NoClip") == Some(&id) => {
                self.pending_one_shots
                    .push(crate::one_shots_runtime::OneShotEffect::NoClipDisabled);
            }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("PotionSpoof") == Some(&id) => {
                self.pending_one_shots.push(
                    crate::one_shots_runtime::OneShotEffect::PotionSpoofDisabled(
                        self.potion_spoof.clone(),
                    ),
                );
            }
            ModuleCallback::OnEnabled(id) if self.query_ids.get("Sneak") == Some(&id) => {if self.sneak.settings().protocol_warning(){self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::SneakSwitchWarn);}}
            ModuleCallback::OnEnabled(id) if self.query_ids.get("Blink")==Some(&id)=>{self.blink.enabled();self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::BlinkEnabled);}
            ModuleCallback::OnDisabled(id) if self.query_ids.get("Blink")==Some(&id)=>{self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::BlinkDisabled);}
            ModuleCallback::OnDisabled(id) if self.query_ids.get("LogoffSpot")==Some(&id)=>{self.logoff_spot.clear();self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::LogoffSpotDisabled);}
            ModuleCallback::OnDisabled(id) if self.query_ids.get("InventoryTracker")==Some(&id)=>{self.inventory_tracker.reset();self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::InventoryTrackerReset);}
            ModuleCallback::OnEnabled(id) if self.query_ids.get("VehicleControl")==Some(&id)=>{self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::VehicleControlEnabled);}
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("VehicleControl")==Some(&id)=>{self.vehicle_control.cancel();}
            ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoWindCharge")==Some(&id)=>self.auto_wind_charge.clear(),
            ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoWindCharge")==Some(&id)=>self.auto_wind_charge.clear(),
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("Aimbot")==Some(&id)=>self.aimbot.clear(),
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("KillAura")==Some(&id)=>{self.kill_aura.clear();self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::KillAuraDisabled);},
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("TpAura")==Some(&id)=>self.tp_aura=false,
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("Velocity")==Some(&id)=>self.velocity=false,
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("ElytraFly")==Some(&id)=>self.elytra_fly=false,
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("NoFall")==Some(&id)=>self.no_fall=false,
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoRod")==Some(&id)=>{let release=self.auto_rod.clear();if release{self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::AutoRodDisabled(true));}self.pending_slot_resets.push(crate::auto_rod_runtime::REQUESTER);},
            ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoShoot")==Some(&id)=>{self.auto_shoot.behavior.disabled();self.pending_slot_resets.push(crate::auto_shoot_runtime::REQUESTER);},
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("ElytraTarget")==Some(&id)=>{self.elytra_target.behavior.disabled();self.pending_slot_resets.push(crate::elytra_target_runtime::REQUESTER);},
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoBow")==Some(&id)=>self.auto_bow.clear(),
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("EasyPearl")==Some(&id)=>self.easy_pearl.behavior.disabled(),
            ModuleCallback::OnEnabled(id) if self.query_ids.get("Extinguish")==Some(&id)=>self.extinguish.behavior.enabled(),
            ModuleCallback::OnDisabled(id) if self.query_ids.get("Extinguish")==Some(&id)=>self.pending_slot_resets.push(crate::extinguish_runtime::REQUESTER),
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoPearl")==Some(&id)=>self.auto_pearl.clear(),
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoBuff")==Some(&id)=>{
                for owner in [crate::auto_buff_runtime::REFILL_OWNER,crate::auto_buff_runtime::STACK_OWNER]{self.inventory.manager.cancel_pending(owner);}
                for plan in [self.auto_buff.refill_pending,self.auto_buff.stack_pending].into_iter().flatten(){self.inventory.manager.end_plan(plan);}
                self.pending_slot_resets.push(crate::auto_buff_runtime::REQUESTER);self.auto_buff.clear();
            }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("ProjectilePuncher")==Some(&id)=>self.projectile_puncher.target=None,
            ModuleCallback::OnEnabled(id) if self.query_ids.get("FreeCam")==Some(&id)=>self.free_cam.behavior.enabled(),
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("FreeCam")==Some(&id)=>self.free_cam.clear(),
            ModuleCallback::OnDisabled(id) if self.query_ids.get("FastExp")==Some(&id)=>self.fast_exp.behavior.disabled(),
            ModuleCallback::CancelSequences(id) if self.query_ids.get("FastExp")==Some(&id)=>self.fast_exp.behavior.cancel(),
            ModuleCallback::OnEnabled(id) if self.query_ids.get("AntiCheatDetect") == Some(&id) => {
                self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::AntiCheatDetectEnabled);
            }
            ModuleCallback::OnEnabled(id) if self.query_ids.get("BookBot") == Some(&id) => {
                self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::BookBotEnabled);
            }
            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("ChestStealer")==Some(&id)=>{self.inventory.manager.cancel_pending(crate::chest_stealer_runtime::OWNER);if let Some(id)=self.chest_stealer.pending{self.inventory.manager.end_plan(id);}self.chest_stealer.clear();}
            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoArmor")==Some(&id)=>{self.inventory.manager.cancel_pending(crate::auto_armor_runtime::OWNER);if let Some(id)=self.auto_armor.pending{self.inventory.manager.end_plan(id);}self.auto_armor.clear();}
            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("Trajectories")==Some(&id)=>{self.trajectories.reset();}
            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("InventoryCleaner")==Some(&id)=>{self.inventory.manager.cancel_pending(crate::inventory_cleaner_runtime::OWNER);if let Some(id)=self.inventory_cleaner.pending{self.inventory.manager.end_plan(id);}self.inventory_cleaner.clear();}
            ModuleCallback::OnEnabled(id) if self.query_ids.get("Replenish")==Some(&id)=>{self.replenish.settings.clear_tracked();}
            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("Replenish")==Some(&id)=>{self.replenish.cancel(&mut self.inventory);}
            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoCrafter")==Some(&id)=>self.auto_crafter.cancel(),
            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoDeposit")==Some(&id)=>{
                self.inventory.manager.cancel_pending(crate::auto_deposit_runtime::OWNER);self.auto_deposit.clear();self.pending_slot_resets.push(crate::auto_deposit_runtime::REQUESTER);
            }
            ModuleCallback::CancelSequences(id) if self.query_ids.get("ChestCleaner") == Some(&id) => {
                self.inventory.manager.cancel_pending(crate::chest_cleaner_runtime::OWNER);
            }
            ModuleCallback::CancelSequences(id) if self.query_ids.get("BookBot") == Some(&id) => {
                self.inventory.manager.cancel_pending(crate::book_bot_runtime::OWNER);
            }
            ModuleCallback::OnEnabled(id) if self.query_ids.get("Clip") == Some(&id) => {
                self.clip.cancel();
                if self.clip.mode==1 {self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::Clip(self.clip));}
            }
            ModuleCallback::CancelSequences(id) | ModuleCallback::OnDisabled(id) if self.query_ids.get("Clip") == Some(&id) => {self.clip.cancel();}
            ModuleCallback::OnEnabled(id) if self.query_ids.get("Damage") == Some(&id) => {
                self.pending_one_shots
                    .push(crate::one_shots_runtime::OneShotEffect::Damage(self.damage));
            }
            ModuleCallback::OnEnabled(id) if self.query_ids.get("NameCollector") == Some(&id) => {
                self.pending_one_shots
                    .push(crate::one_shots_runtime::OneShotEffect::CaptureNames);
            }
            ModuleCallback::OnEnabled(id) if self.query_ids.get("Kick") == Some(&id) => {
                self.pending_one_shots
                    .push(crate::one_shots_runtime::OneShotEffect::Kick(
                        self.kick_mode,
                    ));
            }
            ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoLeave") == Some(&id) => {
                self.auto_leave.cancel_sequences();
            }
            ModuleCallback::CancelSequences(id) if self.query_ids.get("ResetVL") == Some(&id) => {
                self.reset_vl.cancel_sequences();
            }
            ModuleCallback::CancelSequences(id) if self.query_ids.get("Timer") == Some(&id) => {
                self.timer
                    .state
                    .lock()
                    .expect("Timer poisoned")
                    .module
                    .cancel_sequences();
            }
            ModuleCallback::CancelSequences(id) if self.query_ids.get("HandDerp") == Some(&id) => {
                self.hand_derp
                    .behavior
                    .lock()
                    .expect("HandDerp poisoned")
                    .cancel_sequences();
            }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("HandDerp") == Some(&id) => {
                self.hand_derp.initialize();
                if let Some(hand) = self
                    .hand_derp
                    .behavior
                    .lock()
                    .expect("HandDerp poisoned")
                    .on_disabled(liquidbounce_behaviors::hand_derp::MainHand::from_left(
                        options.actual_main_hand == 0,
                    ))
                {
                    self.pending_hand_sends.push(hand.id());
                }
            }
            ModuleCallback::OnEnabled(id) if self.query_ids.get("FreeLook") == Some(&id) => {
                self.free_look.on_enabled(options.player_rotation);
            }
            ModuleCallback::OnEnabled(id) if self.query_ids.get("SkinDerp") == Some(&id) => {
                self.skin_derp.on_enabled(options.model_parts);
            }
            ModuleCallback::CancelSequences(id) if self.query_ids.get("SkinDerp") == Some(&id) => {
                self.skin_derp.cancel_sequences();
            }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("SkinDerp") == Some(&id) => {
                self.pending_model_parts.push(self.skin_derp.on_disabled());
            }
            ModuleCallback::CancelSequences(id) if self.query_ids.get("SmartEat") == Some(&id) => {
                self.smart_eat.cancel_auto_eat();
            }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("SmartEat") == Some(&id) => {
                use liquidbounce_behaviors::smart_eat::{
                    AUTO_EAT_REQUESTER, SILENT_OFFHAND_REQUESTER,
                };
                if self.smart_eat.silent_offhand {
                    self.pending_slot_resets.push(SILENT_OFFHAND_REQUESTER);
                }
                if self.smart_eat.auto_eat {
                    self.pending_slot_resets.push(AUTO_EAT_REQUESTER);
                }
            }
            ModuleCallback::CancelSequences(id) | ModuleCallback::OnDisabled(id)
                if self.query_ids.get("Derp") == Some(&id) =>
            {
                self.derp.cancel_sequences();
            }
            ModuleCallback::CancelSequences(id) | ModuleCallback::OnDisabled(id)
                if self.query_ids.get("AntiAFK") == Some(&id) =>
            {
                self.anti_afk.cancel_sequences();
            }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoTool") == Some(&id) => {
                self.auto_tool.disable();
                self.pending_slot_resets
                    .push(crate::auto_tool_runtime::REQUESTER);
            }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("Vomit") == Some(&id) => {
                self.inventory.cancel_vomit();
            }
            ModuleCallback::OnEnabled(id) if self.query_ids.get("ElytraSwap") == Some(&id) => {
                self.elytra_swap.on_enabled();
            }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("ElytraSwap") == Some(&id) => {
                self.inventory.cancel_elytra_swap();
            }
            ModuleCallback::CancelSequences(id) | ModuleCallback::OnDisabled(id) if self.query_ids.get("Spammer") == Some(&id) => { self.spammer.cancel(); }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoWalk") == Some(&id) => {
                self.auto_walk.own_target = None;
            }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("Step") == Some(&id) => { self.step.disabled(); }
            ModuleCallback::OnEnabled(id) if self.query_ids.get("NoWeb")==Some(&id)=>{let id=self.no_web.start();self.coroutine_ticker.register(crate::coroutine_ticker::WaitOwner::NoWeb,id);}
            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("NoWeb")==Some(&id)=>{self.no_web.cancel();self.pending_slot_resets.push(crate::no_web_runtime::REQUESTER);}
            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("NoSlow")==Some(&id)=>{self.no_slow.cancel();}
            ModuleCallback::OnDisabled(id) if self.query_ids.get("HighJump") == Some(&id) => {
                self.high_jump.disabled();
            }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoWeapon") == Some(&id) => {
                self.pending_slot_resets
                    .push(liquidbounce_behaviors::auto_weapon::REQUESTER);
            }
            ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoFish") == Some(&id) => {
                self.auto_fish
                    .behavior
                    .lock()
                    .expect("AutoFish poisoned")
                    .cancel_sequences();
            }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoFish") == Some(&id) => {
                self.auto_fish
                    .behavior
                    .lock()
                    .expect("AutoFish poisoned")
                    .on_disabled();
            }
            ModuleCallback::CancelSequences(id)
                if self.query_ids.get("AutoRespawn") == Some(&id) =>
            {
                self.auto_respawn.on_disabled();
            }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("SmoothCamera") == Some(&id) => {
                self.smooth_camera.on_disabled();
            }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("NoPitchLimit") == Some(&id) => {
                self.pending_pitch_reset = true;
            }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("FastBreak") == Some(&id) => {
                self.pending_fast_break_resets += 1;
            }
            ModuleCallback::OnEnabled(id) if id == self.zoom_id => {
                let fov = self.default_fov(registry, options.fov);
                self.zoom.on_enabled(fov, &mut current_time_millis);
            }
            ModuleCallback::OnDisabled(id) if id == self.zoom_id => {
                let no_fov = self.no_fov;
                let no_fov_running = registry.running(self.no_fov_id);
                self.zoom.on_disabled_with_default_fov(
                    || {
                        if no_fov_running {
                            no_fov.get_fov(options.fov)
                        } else {
                            options.fov
                        }
                    },
                    &mut current_time_millis,
                );
            }
            ModuleCallback::UpdateChildren {
                module,
                enabled: true,
            } if self.query_ids.get("CameraClip") == Some(&module) => {
                self.camera_clip.parent_enabled();
            }
            ModuleCallback::UpdateChildren {
                module,
                enabled: true,
            } if module == self.full_bright_id => self.full_bright.on_enabled(options.gamma),
            ModuleCallback::UpdateChildren {
                module,
                enabled: false,
            } if module == self.full_bright_id => {
                if let Some(intent) = self.full_bright.on_disabled() {
                    self.push_night_vision(intent);
                }
            }
            // These modules own no coroutine sequences. Core notifications,
            // value events and persistence are handled by the integrating UI.
            _ => {}
        }
    }

    pub fn push_night_vision(&mut self, intent: NightVisionIntent) {
        self.pending_night_vision.push(match intent {
            NightVisionIntent::Apply { duration_ticks, .. } => Some(duration_ticks),
            NightVisionIntent::Remove => None,
        });
    }

    /// Original MixinCamera: Zoom first, then NoFOV only when its returned
    /// integer still equals the original option. Sample the real clock per call.
    pub fn camera_option(&mut self, registry: &ModuleRegistry, original: i32) -> i32 {
        if registry.running(self.zoom_id) {
            return self.zoom.get_fov(true, 0, &mut current_time_millis);
        }
        let returned = self.zoom.get_fov(false, original, &mut current_time_millis);
        if registry.running(self.no_fov_id) && returned == original {
            self.no_fov.get_fov(returned)
        } else {
            returned
        }
    }

    pub fn camera_return(&self, registry: &ModuleRegistry, vanilla: f32) -> f32 {
        self.zoom.apply_fractional_fov(
            vanilla,
            ZoomContext {
                in_game: registry.world().in_game(),
                running: registry.running(self.zoom_id),
            },
            &mut current_time_millis,
        )
    }

    pub fn toggle_full_bright_mode(
        &mut self,
        registry: &ModuleRegistry,
        options: RawOptions,
    ) -> Vec<NightVisionIntent> {
        let next = match self.full_bright.mode() {
            FullBrightMode::Gamma => FullBrightMode::NightVision,
            FullBrightMode::NightVision => FullBrightMode::Gamma,
        };
        let mut effects = Vec::new();
        self.full_bright.switch_mode(
            next,
            registry.running(self.full_bright_id),
            options.gamma,
            |effect| effects.push(effect),
        );
        effects
    }
}
