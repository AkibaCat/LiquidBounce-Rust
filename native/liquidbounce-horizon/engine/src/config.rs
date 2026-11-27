// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce ae0eb396c07f22383789b28d5912d714c6ca0a86,
// Copyright (c) 2015-2026 CCBlueX. Source: ConfigSystem.kt, Value.kt,
// ValueGroupSerializer.kt, ModeValueGroupSerializer.kt, RangeAdapter.kt,
// ChoiceListValue.kt and REST v1/client/ModuleFunctions.kt.

//! Original settings trees for the implemented native behavior modules.
//! Parse first, then execute `SettingsPatch.changes` sequentially on the client
//! actor. `Runtime` results require actual registry/Host work at that position;
//! notably FullBright mode changes must execute before later Gamma settings.
//! This module does not invoke Host, register modules, or fabricate game state.

use liquidbounce_behaviors::{
    Easing, FullBright, FullBrightMode, NoFov, NoFovMode, NoMissCooldown, Zoom,
};
use serde_json::{Map, Number, Value, json};
use std::{collections::BTreeMap, fmt};

#[path = "config/reach.rs"]
mod reach;
pub use reach::{ReachSetting, serialize_reach};
#[path = "config/no_pose.rs"]
mod no_pose;
pub use no_pose::{NoPoseSetting, serialize_no_pose};
#[path = "config/no_interact.rs"]
mod no_interact;
pub use no_interact::{NoEntitySetting, serialize_no_entity_interact};
#[path = "config/no_rotate_set.rs"]
mod no_rotate_set;
pub use no_rotate_set::{NoRotateSetSetting, serialize_no_rotate_set};
#[path = "config/camera_clip.rs"]
mod camera_clip;
pub use camera_clip::{CameraClipSetting, serialize_camera_clip};
#[path = "config/dank_bobbing.rs"]
mod dank_bobbing;
pub use dank_bobbing::{DankBobbingSetting, serialize_dank_bobbing};
#[path = "config/multi_actions.rs"]
mod multi_actions;
pub use multi_actions::{MultiActionsSetting, serialize_multi_actions};
#[path = "config/twerk.rs"]
mod twerk;
pub use twerk::{TwerkSetting, serialize_twerk};
#[path = "config/targets.rs"]
mod targets;
pub use targets::{
    GlobalTargetsPatch, TargetSetting, parse_global_targets, serialize_global_targets,
};
#[path = "config/hitbox.rs"]
mod hitbox;
pub use hitbox::{HitboxSetting, serialize_hitbox};
#[path = "config/auto_disable.rs"]
mod auto_disable;
pub use auto_disable::{AutoDisableSetting, serialize_auto_disable};
#[path = "config/potion_spoof.rs"]
mod potion_spoof;
pub use potion_spoof::{PotionSpoofSetting, serialize_potion_spoof};
#[path = "config/no_clip.rs"]
mod no_clip;
pub use no_clip::{NoClipSetting, serialize_no_clip};
#[path = "config/true_sight.rs"]
mod true_sight;
pub use true_sight::{TrueSightSetting, serialize_true_sight};
#[path = "config/plugins.rs"]
mod plugins;
pub use plugins::serialize_plugins;
#[path = "config/old_regen.rs"]
mod old_regen;
pub use old_regen::{OldRegenSetting, serialize_old_regen};
#[path = "config/fast_use.rs"]
mod fast_use;
pub use fast_use::{FastUseSetting, serialize_fast_use};
#[path="config/fast_exp.rs"]
mod fast_exp;
pub use fast_exp::{FastExpSetting,serialize_fast_exp};
#[path="config/auto_wind_charge.rs"]
mod auto_wind_charge;
pub use auto_wind_charge::{AutoWindChargeSetting,serialize_auto_wind_charge};
#[path="config/auto_buff.rs"]
mod auto_buff;
pub use auto_buff::{AutoBuffSetting,serialize_auto_buff};
#[path="config/auto_bow.rs"]
mod auto_bow;
pub use auto_bow::{AutoBowSetting,serialize_auto_bow};
#[path="config/auto_pearl.rs"]
mod auto_pearl;
pub use auto_pearl::{AutoPearlSetting,serialize_auto_pearl};
#[path="config/extinguish.rs"]
mod extinguish;
pub use extinguish::{ExtinguishSetting,serialize_extinguish};
#[path="config/liquid_filler.rs"]
mod liquid_filler;
pub use liquid_filler::{LiquidFillerSetting,serialize_liquid_filler};
#[path="config/hole_filler.rs"]
mod hole_filler;
pub use hole_filler::{HoleFillerSetting,serialize_hole_filler};
#[path="config/surround.rs"]
mod surround;
pub use surround::{SurroundSetting,serialize_surround};
#[path="config/block_in.rs"]
mod block_in;
pub use block_in::{BlockInSetting,serialize_block_in};
#[path="config/bed_defender.rs"]
mod bed_defender;
pub use bed_defender::{BedDefenderSetting,serialize_bed_defender};
#[path="config/easy_pearl.rs"]
mod easy_pearl;
pub use easy_pearl::{EasyPearlSetting,serialize_easy_pearl};
#[path="config/elytra_target.rs"]
mod elytra_target;
pub use elytra_target::{ElytraTargetSetting,serialize_elytra_target};
#[path="config/auto_rod.rs"]
mod auto_rod;
pub use auto_rod::{AutoRodSetting,serialize_auto_rod};
#[path="config/auto_shoot.rs"]
mod auto_shoot;
pub use auto_shoot::{AutoShootSetting,serialize_auto_shoot};
#[path="config/scaffold.rs"]
pub(crate) mod scaffold;
pub use scaffold::{ScaffoldSetting,ClutchSetting,serialize_scaffold,serialize_clutch};
#[path="config/fake_lag.rs"]
mod fake_lag;
pub use fake_lag::{FakeLagSetting,serialize_fake_lag};
#[path="config/sword_block.rs"]
mod sword_block;
pub use sword_block::{SwordBlockSetting,serialize_sword_block};
#[path="config/aimbot.rs"]
mod aimbot;
pub use aimbot::{AimbotSetting,serialize_aimbot};
#[path="config/target_strafe.rs"]
mod target_strafe;
pub use target_strafe::{TargetStrafeSetting,serialize_target_strafe};
#[path="config/kill_aura.rs"]
mod kill_aura;
pub use kill_aura::{KillAuraSetting,serialize_kill_aura};
#[path="config/packet_mine.rs"]
mod packet_mine;
pub use packet_mine::{PacketMineSetting,serialize_packet_mine};
#[path="config/hud.rs"]
mod hud;
pub use hud::{HudSetting,serialize_hud};
#[path="config/blink.rs"]
mod blink;
pub use blink::{BlinkSetting,serialize_blink};
#[path="config/auto_dodge.rs"]
mod auto_dodge;
pub use auto_dodge::{AutoDodgeSetting,serialize_auto_dodge};
#[path="config/air_place.rs"]
mod air_place;
pub use air_place::{AirPlaceSetting,serialize_air_place};
#[path="config/flag_check.rs"]
mod flag_check;
pub use flag_check::{FlagCheckSetting,serialize_flag_check};
#[path="config/packet_logger.rs"]
mod packet_logger;
pub use packet_logger::{PacketLoggerSetting,serialize_packet_logger};
#[path="config/anti_staff.rs"]
mod anti_staff;
pub use anti_staff::{AntiStaffSetting,serialize_anti_staff};
#[path="config/notifier.rs"]
mod notifier;
pub use notifier::{NotifierSetting,serialize_notifier};
#[path="config/anti_bot.rs"]
mod anti_bot;
pub use anti_bot::{AntiBotSetting,serialize_anti_bot};
#[path="config/anti_void.rs"]
mod anti_void;
pub use anti_void::{AntiVoidSetting,serialize_anti_void};
#[path="config/xray.rs"]
mod xray;
pub use xray::{XRaySetting,serialize_xray};
#[path="config/inventory_move.rs"]
mod inventory_move;
pub use inventory_move::{InventoryMoveSetting,serialize_inventory_move};
#[path = "config/teams.rs"]
mod teams;
pub use teams::{TeamsSetting,serialize_teams};
#[path="config/murder_mystery.rs"]
mod murder_mystery;
pub use murder_mystery::{MurderMysterySetting,serialize_murder_mystery};
#[path="config/offhand.rs"]
mod offhand;
pub use offhand::{OffhandSetting,serialize_offhand};
#[path="config/combat_rotations.rs"]
mod combat_rotations;
#[path="config/projectile_puncher.rs"]
mod projectile_puncher;
pub use projectile_puncher::{ProjectilePuncherSetting,serialize_projectile_puncher};
#[path = "config/spammer.rs"]
mod spammer;
pub use spammer::{SpammerSetting,serialize_spammer};
#[path="config/auto_chat_game.rs"]
mod auto_chat_game;
pub use auto_chat_game::{AutoChatGameSetting,serialize_auto_chat_game};
#[path = "config/ping_spoof.rs"]
mod ping_spoof;
pub use ping_spoof::{PingSpoofSetting,serialize_ping_spoof};
#[path = "config/clip.rs"]
mod clip;
pub use clip::{ClipSetting,serialize_clip};
#[path = "config/book_bot.rs"]
mod book_bot;
pub use book_bot::{BookBotSetting,serialize_book_bot};
#[path="config/chest_cleaner.rs"]
mod chest_cleaner;
pub use chest_cleaner::{ChestCleanerSetting,serialize_chest_cleaner};
#[path="config/auto_deposit.rs"]
mod auto_deposit;
pub use auto_deposit::{AutoDepositSetting,serialize_auto_deposit};
#[path="config/replenish.rs"]
mod replenish;
pub use replenish::{ReplenishSetting,serialize_replenish};
#[path="config/auto_crafter.rs"]
mod auto_crafter;
pub use auto_crafter::{AutoCrafterSetting,serialize_auto_crafter};
#[path="config/inventory_cleaner.rs"]
mod inventory_cleaner;
pub use inventory_cleaner::{InventoryCleanerSetting,serialize_inventory_cleaner};
#[path="config/auto_armor.rs"]
mod auto_armor;
pub use auto_armor::{AutoArmorSetting,serialize_auto_armor};
#[path="config/auto_mob_heal.rs"]
mod auto_mob_heal;
pub use auto_mob_heal::{AutoMobHealSetting,serialize_auto_mob_heal};
#[path="config/chest_stealer.rs"]
mod chest_stealer;
pub use chest_stealer::{ChestStealerSetting,serialize_chest_stealer};
#[path="config/eagle.rs"]
mod eagle;
pub use eagle::{EagleSetting,serialize_eagle};
#[path="config/anti_exploit.rs"]
mod anti_exploit;
pub use anti_exploit::{AntiExploitSetting,serialize_anti_exploit};
#[path="config/sneak.rs"]
mod sneak;
pub use sneak::{SneakSetting,serialize_sneak};
#[path = "config/spider.rs"]
mod spider;
pub use spider::{SpiderSetting, serialize_spider};
#[path = "config/auto_account.rs"]
mod auto_account;
pub use auto_account::{AutoAccountSetting, serialize_auto_account};
#[path="config/report_helper.rs"]
mod report_helper;
pub use report_helper::{ReportHelperSetting,serialize_report_helper};
#[path="config/auto_queue.rs"]
mod auto_queue;
pub use auto_queue::{AutoQueueSetting,serialize_auto_queue};
#[path="config/macros.rs"]
mod macros;
pub use macros::{MacrosSetting,serialize_macros};
#[path="config/client_commands.rs"]
mod client_commands;
pub use client_commands::{serialize_client_commands,apply_client_commands};
#[path = "config/middle_click_action.rs"]
mod middle_click_action;
pub use middle_click_action::{MiddleClickActionSetting, serialize_middle_click_action};
#[path = "config/super_knockback.rs"]
mod super_knockback;
pub use super_knockback::{SuperKnockbackSetting, serialize_super_knockback};
#[path = "config/mace_kill.rs"]
mod mace_kill;
pub use mace_kill::{MaceKillSetting, serialize_mace_kill};
#[path = "config/spear_kill.rs"]
mod spear_kill;
pub use spear_kill::{SpearKillSetting, serialize_spear_kill};
#[path="config/crosshair_clicker.rs"]
mod crosshair_clicker;
pub use crosshair_clicker::{CrosshairClickerSetting,serialize_trigger_bot,serialize_auto_clicker};
#[path="config/criticals.rs"]
mod criticals;
pub use criticals::{CriticalsSetting,serialize_criticals};
#[path = "config/mob_owners.rs"]
mod mob_owners;
pub use mob_owners::{MobOwnersSetting, serialize_mob_owners};
#[path = "config/combine_mobs.rs"]
mod combine_mobs;
pub use combine_mobs::{CombineMobsSetting, serialize_combine_mobs};
#[path = "config/better_tab.rs"]
mod better_tab;
pub use better_tab::{BetterTabSetting, serialize_better_tab};
#[path = "config/block_outline.rs"]
mod block_outline;
pub use block_outline::{BlockOutlineSetting,serialize_block_outline};
#[path = "config/damage_particles.rs"]
mod damage_particles;
pub use damage_particles::{DamageParticlesSetting,serialize_damage_particles};
#[path = "config/tracers.rs"]
mod tracers;
pub use tracers::{TracersSetting,serialize_tracers};
#[path="config/esp.rs"]
mod esp;
pub use esp::{EspSetting,serialize_esp};
#[path="config/radar.rs"]
mod radar;
pub use radar::{RadarSetting,serialize_radar};
#[path="config/item_esp.rs"]
mod item_esp;
pub use item_esp::{ItemEspSetting,serialize_item_esp};
#[path="config/item_chams.rs"]
mod item_chams;
pub use item_chams::{ItemChamsSetting,serialize_item_chams};
#[path="config/item_tags.rs"]
mod item_tags;
pub use item_tags::{ItemTagsSetting,serialize_item_tags};
#[path="config/nametags.rs"]
mod nametags;
pub use nametags::{NametagsSetting,serialize_nametags};
#[path="config/better_chat.rs"]
mod better_chat;
pub use better_chat::{BetterChatSetting,serialize_better_chat,serialize_global_translate,apply_global_translate,global_translate_changes};
#[path="config/debug.rs"]
mod debug;
pub use debug::{DebugSetting,serialize_debug};
#[path="config/click_gui.rs"]
mod click_gui;
pub use click_gui::{ClickGuiSetting,serialize_click_gui};
#[path="config/bed_plates.rs"]
mod bed_plates;
pub use bed_plates::{BedPlatesSetting,serialize_bed_plates};
#[path = "config/new_chunks.rs"]
mod new_chunks;
pub use new_chunks::{NewChunksSetting,serialize_new_chunks};
#[path="config/stronghold_finder.rs"]
mod stronghold_finder;
pub use stronghold_finder::{StrongholdFinderSetting,serialize_stronghold_finder};
#[path = "config/placement_esp.rs"]
mod placement_esp;
pub use placement_esp::{ProphuntEspSetting,VoidEspSetting,serialize_prophunt_esp,serialize_void_esp};
#[path = "config/protection_zones.rs"]
mod protection_zones;
pub use protection_zones::{ProtectionZonesSetting,serialize_protection_zones};
#[path = "config/hole_esp.rs"]
mod hole_esp;
pub use hole_esp::{HoleEspSetting,serialize_hole_esp};
#[path = "config/particles.rs"]
mod particles;
pub use particles::{ParticlesSetting,serialize_particles};
#[path="config/trajectories.rs"]
mod trajectories;
pub use trajectories::{TrajectoriesSetting,serialize_trajectories};
#[path="config/custom_ambience.rs"]
mod custom_ambience;
pub use custom_ambience::{CustomAmbienceSetting,serialize_custom_ambience};
#[path="config/tnt_timer.rs"]
mod tnt_timer;
pub use tnt_timer::{TNTTimerSetting,serialize_tnt_timer};
#[path="config/potion_fx.rs"]
mod potion_fx;
pub use potion_fx::{PotionFXSetting,serialize_potion_fx};
#[path="config/totem_effect.rs"]
mod totem_effect;
pub use totem_effect::{TotemEffectSetting,serialize_totem_effect};
#[path="config/hit_fx.rs"]mod hit_fx;
pub use hit_fx::{HitFxSetting,serialize_hit_fx};
#[path="config/jump_effect.rs"]
mod jump_effect;
pub use jump_effect::{JumpEffectSetting,serialize_jump_effect};
#[path = "config/logoff_spot.rs"]
mod logoff_spot;
pub use logoff_spot::{LogoffSpotSetting,serialize_logoff_spot};
#[path="config/inventory_tracker.rs"]
mod inventory_tracker;
pub use inventory_tracker::{InventoryTrackerSetting,serialize_inventory_tracker};
#[path="config/name_protect.rs"]
mod name_protect;
pub use name_protect::{NameProtectSetting,serialize_name_protect};
#[path="config/storage_esp.rs"]
mod storage_esp;
pub use storage_esp::{StorageEspSetting,serialize_storage_esp};
#[path="config/block_esp.rs"]
mod block_esp;
pub use block_esp::{BlockEspSetting,serialize_block_esp};
#[path = "config/crosshair.rs"]
mod crosshair;
pub use crosshair::{CrosshairSetting,serialize_crosshair};
#[path = "config/wings.rs"]
mod wings;
pub use wings::{WingsSetting,serialize_wings};
#[path="config/target_renderer.rs"]
pub(crate) mod target_renderer;
#[path = "config/hats.rs"]
mod hats;
pub use hats::{HatsSetting,serialize_hats};
#[path = "config/breadcrumbs.rs"]
mod breadcrumbs;
pub use breadcrumbs::{BreadcrumbsSetting,serialize_breadcrumbs};
#[path = "config/auto_weapon.rs"]
mod auto_weapon;
pub use auto_weapon::{AutoWeaponSetting, serialize_auto_weapon};
#[path = "config/chams.rs"]
mod chams;
pub use chams::{ChamsSetting, serialize_chams};
#[path = "config/auto_tool.rs"]
mod auto_tool;
pub use auto_tool::{AutoToolSetting, serialize_auto_tool};
#[path = "config/anti_afk.rs"]
mod anti_afk;
pub use anti_afk::{AntiAfkSetting, serialize_anti_afk};
#[path = "config/trident_boost.rs"]
mod trident_boost;
pub use trident_boost::{TridentBoostSetting, serialize_trident_boost};
#[path = "config/sleep_walker.rs"]
mod sleep_walker;
pub use sleep_walker::serialize_sleep_walker;
#[path = "config/derp.rs"]
mod derp;
pub use derp::{DerpSetting, serialize_derp};
#[path = "config/free_look.rs"]
mod free_look;
pub use free_look::{FreeLookSetting, serialize_free_look};
#[path="config/free_cam.rs"]
mod free_cam;
pub use free_cam::{FreeCamSetting,serialize_free_cam};
#[path = "config/hand_derp.rs"]
mod hand_derp;
pub use hand_derp::{HandDerpSetting, serialize_hand_derp};
#[path = "config/vehicle_boost.rs"]
mod vehicle_boost;
pub use vehicle_boost::{VehicleBoostSetting, serialize_vehicle_boost};
#[path="config/vehicle_control.rs"]
mod vehicle_control;
pub use vehicle_control::{VehicleControlSetting,serialize_vehicle_control};
#[path = "config/block_bounce.rs"]
mod block_bounce;
pub use block_bounce::{BlockBounceSetting, serialize_block_bounce};
#[path = "config/smart_eat.rs"]
mod smart_eat;
pub use smart_eat::{SmartEatSetting, serialize_smart_eat};
#[path = "config/skin_derp.rs"]
mod skin_derp;
pub use skin_derp::{SkinDerpSetting, serialize_skin_derp};
#[path = "config/entity_control.rs"]
mod entity_control;
pub use entity_control::{EntityControlSetting, serialize_entity_control};
#[path = "config/snap_tap.rs"]
mod snap_tap;
pub use snap_tap::serialize_snap_tap;
#[path = "config/block_walk.rs"]
mod block_walk;
pub use block_walk::{BlockWalkSetting, serialize_block_walk};
#[path = "config/ghost_hand.rs"]
mod ghost_hand;
pub use ghost_hand::{GhostHandSetting, serialize_ghost_hand};
#[path = "config/extended_firework.rs"]
mod extended_firework;
pub use extended_firework::{ExtendedFireworkSetting, serialize_extended_firework};
#[path = "config/strafe.rs"]
mod strafe;
pub use strafe::{StrafeSetting, serialize_strafe};
#[path = "config/speed.rs"]
mod speed;
pub use speed::{SpeedSetting, serialize_speed};
#[path = "config/elytra_recast.rs"]
mod elytra_recast;
pub use elytra_recast::serialize_elytra_recast;
#[path = "config/fast_break.rs"]
mod fast_break;
pub use fast_break::{FastBreakSetting, serialize_fast_break};
#[path = "config/timer.rs"]
mod timer;
pub use timer::{TimerSetting, serialize_timer};
#[path = "config/better_inventory.rs"]
mod better_inventory;
pub use better_inventory::{BetterInventorySetting, serialize_better_inventory};
#[path = "config/terrain_speed.rs"]
mod terrain_speed;
pub use terrain_speed::{TerrainSpeedSetting, serialize_terrain_speed};
#[path = "config/high_jump.rs"]
mod high_jump;
pub use high_jump::{HighJumpSetting, serialize_high_jump};
#[path="config/no_slow.rs"]
mod no_slow;
pub use no_slow::{NoSlowSetting,serialize_no_slow};
#[path="config/no_web.rs"]
mod no_web;
pub use no_web::{NoWebSetting,serialize_no_web};
#[path="config/liquid_walk.rs"]
mod liquid_walk;
pub use liquid_walk::{LiquidWalkSetting,serialize_liquid_walk};
#[path="config/step.rs"]
mod step;
pub use step::{StepSetting,serialize_step};
#[path = "config/reverse_step.rs"]
mod reverse_step;
pub use reverse_step::{ReverseStepSetting,serialize_reverse_step};
#[path = "config/auto_walk.rs"]
mod auto_walk;
pub use auto_walk::{AutoWalkSetting, serialize_auto_walk};
#[path = "config/edge_assistance.rs"]
mod edge_assistance;
pub use edge_assistance::{ParkourSetting,SafeWalkSetting,serialize_parkour,serialize_safe_walk};
#[path = "config/movement_guard.rs"]
mod movement_guard;
pub use movement_guard::{AnchorSetting,AvoidHazardsSetting,FreezeSetting,serialize_anchor,serialize_avoid_hazards,serialize_freeze};
#[path = "config/damage.rs"]
mod damage;
#[path = "config/one_shots.rs"]
mod one_shots;
pub use damage::{DamageSetting, serialize_damage};
#[path = "config/yggdrasil_signature.rs"]
mod yggdrasil_signature;
pub use yggdrasil_signature::serialize_yggdrasil_signature;
#[path = "config/target_lock.rs"]
mod target_lock;
pub use target_lock::{TargetLockSetting, serialize_target_lock};
#[path = "config/reset_vl.rs"]
mod reset_vl;
pub use one_shots::{OneShotSetting, serialize_auto_leave, serialize_kick};
pub use reset_vl::{ResetVlSetting, serialize_reset_vl};
#[path = "config/item_scroller.rs"]
mod item_scroller;
pub use item_scroller::{ItemScrollerSetting, serialize_item_scroller};
#[path = "config/gui_closer.rs"]
mod gui_closer;
pub use gui_closer::{GuiCloserSetting, serialize_gui_closer};
#[path = "config/text_field_protect.rs"]
mod text_field_protect;
pub use text_field_protect::{TextFieldProtectSetting, serialize_text_field_protect};
#[path = "config/anti_hunger.rs"]
mod anti_hunger;
pub use anti_hunger::{AntiHungerSetting, serialize_anti_hunger};
#[path = "config/vomit.rs"]
mod vomit;
pub use vomit::{VomitSetting, serialize_vomit};
#[path = "config/elytra_swap.rs"]
mod elytra_swap;
pub use elytra_swap::{ElytraSwapSetting, serialize_elytra_swap};
#[path = "config/blink_esp.rs"]
mod blink_esp;
pub use blink_esp::{serialize_global_blink,parse_global_blink,blink_changed_values};
#[path = "config/world_noop.rs"]
mod world_noop;
pub use world_noop::{serialize_auto_shop,serialize_auto_build,serialize_auto_farm,serialize_crystal_aura,serialize_fucker,serialize_auto_trap,serialize_block_trap,serialize_nuker,serialize_tick_base,serialize_backtrack,serialize_fly,serialize_long_jump};
#[path = "config/inventory_manager.rs"]
mod inventory_manager;
pub use inventory_manager::{
    GlobalInventoryPatch, GlobalInventorySetting, parse_global_inventory,
    serialize_global_inventory,
};

macro_rules! modules {
    ($( $kind:ident => ($name:literal, $category:literal, $key:literal) ),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum ModuleKind { $( $kind ),+ }
        impl ModuleKind {
            pub const ALL: &'static [Self] = &[$( Self::$kind ),+];
            pub const fn name(self) -> &'static str {
                match self { $( Self::$kind => $name ),+ }
            }
            pub fn from_name(name: &str) -> Option<Self> {
                // Original ConfigSystem group identifiers are case-sensitive.
                match name { "AimAssist"|"AutoAim"=>Some(Self::Aimbot), "FastEat" => Some(Self::FastUse), "BlockOverlay" => Some(Self::BlockOutline), "BlockUpdateDetector" | "FallingBlockESP" => Some(Self::ProphuntEsp), "ChestESP"=>Some(Self::StorageEsp), "PlayerTrails" => Some(Self::Breadcrumbs), "MultiTask" => Some(Self::MultiActions), "BedGodMode" => Some(Self::SleepWalker), "FastClimb" => Some(Self::TerrainSpeed), "ChestSwap" => Some(Self::ElytraSwap), "ContainerCleaner" => Some(Self::ChestCleaner), "AutoStore"|"AutoTotem"=>Some(Self::Offhand), "ContainerStorer"=>Some(Self::AutoDeposit), "Refill"=>Some(Self::Replenish), "PointerESP"=>Some(Self::Radar),"Triangulation"=>Some(Self::StrongholdFinder), "InventoryManager"=>Some(Self::InventoryCleaner), "AutoGolemRepair"=>Some(Self::AutoMobHeal), "ContainerStealer"=>Some(Self::ChestStealer), "AntiSpam"=>Some(Self::BetterChat), "AutoPot"|"AutoGapple"|"AutoSoup"=>Some(Self::AutoBuff), "BowAssist"|"BowAimbot"=>Some(Self::AutoBow), "PearlFollower"|"PearlTarget"=>Some(Self::AutoPearl), "PearlHelper"|"PearlAssist"|"PearlTP"=>Some(Self::EasyPearl), "FastBridge"|"BridgeAssistant"|"LegitScaffold"=>Some(Self::Eagle), "AutoClose" | "ContainerCloser" => Some(Self::GuiCloser), $( $name => Some(Self::$kind), )+ _ => None }
            }
            pub const fn category(self) -> &'static str {
                match self { $( Self::$kind => $category ),+ }
            }
            pub const fn key(self) -> &'static str {
                match self { $( Self::$kind => concat!("liquidbounce.module.", $key) ),+ }
            }
        }
    }
}

// This is the functioning-module registry, separate from the upstream catalog.
// Add a declaration only together with its actual game phase implementation.
modules! {
    FullBright => ("FullBright", "Render", "fullBright"),
    NoFov => ("NoFOV", "Render", "noFOV"),
    Zoom => ("Zoom", "Render", "zoom"),
    NoBob => ("NoBob", "Render", "noBob"),
    NoHurtCam => ("NoHurtCam", "Render", "noHurtCam"),
    NoJumpDelay => ("NoJumpDelay", "Movement", "noJumpDelay"),
    AntiBounce => ("AntiBounce", "Movement", "antiBounce"),
    AntiLevitation => ("AntiLevitation", "Movement", "antiLevitation"),
    LiquidPlace => ("LiquidPlace", "World", "liquidPlace"),
    PortalMenu => ("PortalMenu", "Exploit", "portalMenu"),
    Disabler => ("Disabler", "Exploit", "disabler"),
    ServerCrasher => ("ServerCrasher", "Exploit", "serverCrasher"),
    Dupe => ("Dupe", "Exploit", "dupe"),
    TeleportReach => ("TeleportReach", "Exploit", "teleportReach"),
    Phase => ("Phase", "Exploit", "phase"),
    AntiReducedDebugInfo => ("AntiReducedDebugInfo", "Exploit", "antiReducedDebugInfo"),
    NoMissCooldown => ("NoMissCooldown", "Combat", "noMissCooldown"),
    AbortBreaking => ("AbortBreaking", "Exploit", "abortBreaking"),
    NoSwing => ("NoSwing", "Render", "noSwing"),
    MoreCarry => ("MoreCarry", "Exploit", "moreCarry"),
    NoPitchLimit => ("NoPitchLimit", "Exploit", "noPitchLimit"),
    NoInterpolation => ("NoInterpolation", "World", "noInterpolation"),
    NoSlowBreak => ("NoSlowBreak", "World", "noSlowBreak"),
    NoSlotSet => ("NoSlotSet", "Player", "noSlotSet"),
    Sprint => ("Sprint", "Movement", "sprint"),
    AntiBlind => ("AntiBlind", "Render", "antiBlind"),
    AirJump => ("AirJump", "Movement", "airJump"),
    NoPush => ("NoPush", "Movement", "noPush"),
    Aspect => ("Aspect", "Render", "aspect"),
    SmoothCamera => ("SmoothCamera", "Render", "smoothCamera"),
    QuickPerspectiveSwap => ("QuickPerspectiveSwap", "Render", "quickPerspectiveSwap"),
    AutoRespawn => ("AutoRespawn", "Player", "autoRespawn"),
    KeepSprint => ("KeepSprint", "Combat", "keepSprint"),
    CrystalView => ("CrystalView", "Render", "crystalView"),
    FastPlace => ("FastPlace", "World", "fastPlace"),
    AutoF5 => ("AutoF5", "Render", "autoF5"),
    AutoFish => ("AutoFish", "Player", "autoFish"),
    Reach => ("Reach", "Player", "reach"),
    NoPose => ("NoPose", "Movement", "noPose"),
    NoEntityInteract => ("NoEntityInteract", "Player", "noEntityInteract"),
    NoBlockInteract => ("NoBlockInteract", "Player", "noBlockInteract"),
    NoRotateSet => ("NoRotateSet", "Player", "noRotateSet"),
    CameraClip => ("CameraClip", "Render", "cameraClip"),
    DankBobbing => ("DankBobbing", "Fun", "dankBobbing"),
    MultiActions => ("MultiActions", "Exploit", "multiActions"),
    Twerk => ("Twerk", "Fun", "twerk"),
    Notebot => ("Notebot", "Fun", "notebot"),
    Hitbox => ("Hitbox", "Combat", "hitbox"),
    AutoWeapon => ("AutoWeapon", "Combat", "autoWeapon"),
    Chams => ("Chams", "Render", "chams"),
    AutoTool => ("AutoTool", "Player", "autoTool"),
    AntiAfk => ("AntiAFK", "Player", "antiAFK"),
    TridentBoost => ("TridentBoost", "Movement", "tridentBoost"),
    SleepWalker => ("SleepWalker", "Exploit", "sleepWalker"),
    Derp => ("Derp", "Fun", "derp"),
    FreeLook => ("FreeLook", "Render", "freeLook"),
    FreeCam=>("FreeCam","Render","freeCam"),
    HandDerp => ("HandDerp", "Fun", "handDerp"),
    VehicleBoost => ("VehicleBoost", "Movement", "vehicleBoost"),
    VehicleControl=>("VehicleControl","Movement","vehicleControl"),
    BlockBounce => ("BlockBounce", "Movement", "blockBounce"),
    SmartEat => ("SmartEat", "Player", "smartEat"),
    SkinDerp => ("SkinDerp", "Fun", "skinDerp"),
    SkinChanger => ("SkinChanger", "Render", "skinChanger"),
    EntityControl => ("EntityControl", "Movement", "entityControl"),
    SnapTap => ("SnapTap", "Movement", "snapTap"),
    BlockWalk => ("BlockWalk", "Movement", "blockWalk"),
    GhostHand => ("GhostHand", "Exploit", "ghostHand"),
    ExtendedFirework => ("ExtendedFirework", "Movement", "extendedFirework"),
    Strafe => ("Strafe", "Movement", "strafe"),
    Speed => ("Speed", "Movement", "speed"),
    ElytraFly => ("ElytraFly", "Movement", "elytraFly"),
    NoFall => ("NoFall", "Movement", "noFall"),
    ElytraRecast => ("ElytraRecast", "Movement", "elytraRecast"),
    FastBreak => ("FastBreak", "World", "fastBreak"),
    Timer => ("Timer", "World", "timer"),
    BetterInventory => ("BetterInventory", "Render", "betterInventory"),
    TerrainSpeed => ("TerrainSpeed", "Movement", "terrainSpeed"),
    AutoBreak => ("AutoBreak", "Player", "autoBreak"),
    HighJump => ("HighJump", "Movement", "highJump"),
    NoSlow => ("NoSlow","Movement","noSlow"),
    NoWeb => ("NoWeb","Movement","noWeb"),
    LiquidWalk => ("LiquidWalk","Movement","liquidWalk"),
    Step => ("Step","Movement","step"),
    ReverseStep => ("ReverseStep", "Movement", "reverseStep"),
    AutoWalk => ("AutoWalk", "Player", "autoWalk"),
    Parkour => ("Parkour", "Movement", "parkour"),
    SafeWalk => ("SafeWalk", "Movement", "safeWalk"),
    Anchor => ("Anchor", "Movement", "anchor"),
    AvoidHazards => ("AvoidHazards", "Movement", "avoidHazards"),
    Freeze => ("Freeze", "Movement", "freeze"),
    ResetVl => ("ResetVL", "Exploit", "resetVL"),
    NameCollector => ("NameCollector", "Exploit", "nameCollector"),
    YggdrasilSignatureFix => ("YggdrasilSignatureFix", "Exploit", "yggdrasilSignatureFix"),
    Kick => ("Kick", "Exploit", "kick"),
    Damage => ("Damage", "Exploit", "damage"),
    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),
    AutoDisable => ("AutoDisable", "World", "autoDisable"),
    AutoAccount => ("AutoAccount", "Misc", "autoAccount"),
    Breadcrumbs => ("Breadcrumbs", "Render", "breadcrumbs"),
    Hats => ("Hats", "Render", "hats"),
    Wings => ("Wings", "Render", "wings"),
    Crosshair => ("Crosshair", "Render", "crosshair"),
    StorageEsp => ("StorageESP", "Render", "storageESP"),
    BlockEsp => ("BlockESP", "Render", "blockESP"),
    LogoffSpot => ("LogoffSpot", "Render", "logoffSpot"),
    InventoryTracker => ("InventoryTracker", "World", "inventoryTracker"),
    NameProtect => ("NameProtect", "Misc", "nameProtect"),
    Particles => ("Particles", "Render", "particles"),
    Trajectories=>("Trajectories","Render","trajectories"),
    CustomAmbience=>("CustomAmbience","Render","customAmbience"),
    TNTTimer=>("TNTTimer","Render","tntTimer"),
    PotionFX=>("PotionFX","Render","potionFX"),
    TotemEffect=>("TotemEffect","Render","totemEffect"),
    HitFx=>("HitFX","Render","hitFX"),
    JumpEffect=>("JumpEffect","Render","jumpEffect"),
    HoleEsp => ("HoleESP", "Render", "holeESP"),
    ProtectionZones => ("ProtectionZones", "Render", "protectionZones"),
    ProphuntEsp => ("ProphuntESP", "Render", "prophuntESP"),
    VoidEsp => ("VoidESP", "Render", "voidEsp"),
    NewChunks => ("NewChunks", "Render", "newChunks"),
    StrongholdFinder=>("StrongholdFinder","World","strongholdFinder"),
    Tracers => ("Tracers", "Render", "tracers"),
    Esp=>("ESP","Render","esp"),
    Radar=>("Radar","Render","radar"),
    ItemEsp=>("ItemESP","Render","itemEsp"),
    ItemChams=>("ItemChams","Render","itemChams"),
    ItemTags=>("ItemTags","Render","itemTags"),
    Nametags=>("Nametags","Render","nametags"),
    BetterChat=>("BetterChat","Render","betterChat"),
    Debug=>("Debug","Render","debug"),
    ClickGui=>("ClickGUI","Render","clickGui"),
    BedPlates=>("BedPlates","Render","bedPlates"),
    DamageParticles => ("DamageParticles", "Render", "damageParticles"),
    BlockOutline => ("BlockOutline", "Render", "blockOutline"),
    BetterTab => ("BetterTab", "Render", "betterTab"),
    CombineMobs => ("CombineMobs", "Render", "combineMobs"),
    MobOwners => ("MobOwners", "Render", "mobOwners"),
    ReportHelper => ("ReportHelper", "Player", "reportHelper"),
    AutoQueue => ("AutoQueue", "Player", "autoQueue"),
    Macros => ("Macros", "Misc", "macros"),
    AutoConfig => ("AutoConfig", "Misc", "autoConfig"),
    DebugRecorder => ("DebugRecorder", "Misc", "debugRecorder"),
    MiddleClickAction => ("MiddleClickAction", "Misc", "middleClickAction"),
    SuperKnockback => ("SuperKnockback", "Combat", "superKnockback"),
    MaceKill => ("MaceKill", "Combat", "maceKill"),
    SpearKill => ("SpearKill", "Combat", "spearKill"),
    TriggerBot => ("TriggerBot", "Combat", "triggerBot"),
    Criticals => ("Criticals", "Combat", "criticals"),
    AutoClicker => ("AutoClicker", "Combat", "autoClicker"),
    Spider => ("Spider", "Movement", "spider"),
    OldRegen => ("OldRegen", "Exploit", "oldRegen"),
    FastUse => ("FastUse", "Player", "fastUse"),
    FastExp=>("FastExp","Player","fastExp"),
    AutoWindCharge=>("AutoWindCharge","Player","autoWindCharge"),
    AutoBuff=>("AutoBuff","Player","autoBuff"),
    AutoBow=>("AutoBow","Combat","autoBow"),
    AutoPearl=>("AutoPearl","Combat","autoPearl"),
    Extinguish=>("Extinguish","World","extinguish"),
    LiquidFiller=>("LiquidFiller","World","liquidFiller"),
    HoleFiller=>("HoleFiller","World","holeFiller"),
    AutoShop=>("AutoShop","World","autoShop"),
    AutoBuild=>("AutoBuild","World","autoBuild"),
    AutoFarm=>("AutoFarm","World","autoFarm"),
    CrystalAura=>("CrystalAura","Combat","crystalAura"),
    Fucker=>("Fucker","World","fucker"),
    AutoTrap=>("AutoTrap","Combat","autoTrap"),
    BlockTrap=>("BlockTrap","Combat","blockTrap"),
    Nuker=>("Nuker","World","nuker"),
    TickBase=>("TickBase","Combat","tickBase"),
    Backtrack=>("Backtrack","Combat","backtrack"),
    Fly=>("Fly","Movement","fly"),
    LongJump=>("LongJump","Movement","longJump"),
    Surround=>("Surround","World","surround"),
    BlockIn=>("BlockIn","World","blockIn"),
    BedDefender=>("BedDefender","World","bedDefender"),
    EasyPearl=>("EasyPearl","Misc","easyPearl"),
    ElytraTarget=>("ElytraTarget","Combat","elytraTarget"),
    AutoRod=>("AutoRod","Combat","autoRod"),
    AutoShoot=>("AutoShoot","Combat","autoShoot"),
    Scaffold=>("Scaffold","World","scaffold"),
    Clutch=>("Clutch","World","clutch"),
    FakeLag=>("FakeLag","Combat","fakeLag"),
    SwordBlock=>("SwordBlock","Combat","swordBlock"),
    Aimbot=>("Aimbot","Combat","aimbot"),
    TargetStrafe=>("TargetStrafe","Movement","targetStrafe"),
    KillAura=>("KillAura","Combat","killAura"),
    TpAura=> ("TpAura", "Combat", "tpAura"),
    Velocity=> ("Velocity", "Combat", "velocity"),
    PacketMine=>("PacketMine","World","packetMine"),
    Hud=>("HUD","Render","hud"),
    Blink=>("Blink","Player","blink"),
    AutoDodge=>("AutoDodge","Combat","autoDodge"),
    AirPlace=>("AirPlace","World","airPlace"),
    FlagCheck=>("FlagCheck","Misc","flagCheck"),
    PacketLogger=>("PacketLogger","Misc","packetLogger"),
    AntiStaff=>("AntiStaff","Misc","antiStaff"),
    Notifier=>("Notifier","Misc","notifier"),
    AntiBot=>("AntiBot","Misc","antiBot"),
    AntiVoid=>("AntiVoid","Player","antiVoid"),
    XRay=>("XRay","Render","xRay"),
    InventoryMove=>("InventoryMove","Movement","inventoryMove"),
    Teams => ("Teams", "Misc", "teams"),
    MurderMystery => ("MurderMystery", "Render", "murderMystery"),
    Offhand => ("Offhand","Player","offhand"),
    ProjectilePuncher=>("ProjectilePuncher","World","projectilePuncher"),
    Spammer => ("Spammer", "Misc", "spammer"),
    AutoChatGame => ("AutoChatGame", "Misc", "autoChatGame"),
    PingSpoof => ("PingSpoof", "Exploit", "pingSpoof"),
    Clip => ("Clip", "Movement", "clip"),
    BookBot => ("BookBot", "Exploit", "bookBot"),
    ChestCleaner => ("ChestCleaner","Player","chestCleaner"),
    AutoDeposit => ("AutoDeposit","Player","autoDeposit"),
    Replenish => ("Replenish","Player","replenish"),
    AutoCrafter=>("AutoCrafter","Player","autoCrafter"),
    InventoryCleaner => ("InventoryCleaner","Player","inventoryCleaner"),
    AutoArmor => ("AutoArmor","Combat","autoArmor"),
    AutoMobHeal => ("AutoMobHeal","World","autoMobHeal"),
    ChestStealer => ("ChestStealer","Player","chestStealer"),
    Eagle => ("Eagle","Player","eagle"),
    AntiExploit => ("AntiExploit","Player","antiExploit"),
    Sneak => ("Sneak", "Movement", "sneak"),
    Plugins => ("Plugins", "Exploit", "plugins"),
    AntiCheatDetect => ("AntiCheatDetect","Misc","antiCheatDetect"),
    TrueSight => ("TrueSight", "Render", "trueSight"),
    NoClip => ("NoClip", "Movement", "noClip"),
    PotionSpoof => ("PotionSpoof", "Player", "potionSpoof"),
    ItemScroller => ("ItemScroller", "Misc", "itemScroller"),
    GuiCloser => ("GUICloser", "Misc", "guiCloser"),
    TextFieldProtect => ("TextFieldProtect", "Misc", "textFieldProtect"),
    TargetLock => ("TargetLock", "Misc", "targetLock"),
    AntiHunger => ("AntiHunger", "Exploit", "antiHunger"),
    Vomit => ("Vomit", "Fun", "vomit"),
    ElytraSwap => ("ElytraSwap", "Player", "elytraSwap"),
    VehicleOneHit => ("VehicleOneHit", "Exploit", "vehicleOneHit"),
    SilentHotbar => ("SilentHotbar", "Render", "silentHotbar"),
}

impl ModuleKind {
    /// Registered placeholders without native gameplay. Keep this explicit so
    /// a missing serializer for an implemented module remains an error.
    pub const fn is_unimplemented(self) -> bool {
        matches!(self,
            Self::Disabler | Self::ServerCrasher | Self::Dupe
            | Self::TeleportReach | Self::Phase | Self::Notebot
            | Self::AutoConfig | Self::DebugRecorder | Self::SkinChanger
            | Self::ElytraFly | Self::NoFall | Self::TpAura | Self::Velocity
            | Self::AutoShop | Self::AutoBuild | Self::AutoFarm
            | Self::CrystalAura | Self::Fucker | Self::AutoTrap | Self::BlockTrap
            | Self::Nuker | Self::TickBase | Self::Backtrack | Self::Fly | Self::LongJump)
    }
}

/// BuiltInRegistries.MENU, verified against the pinned 26.3 registry export.
pub const MENU_IDS: &[&str] = &[
    "minecraft:anvil",
    "minecraft:beacon",
    "minecraft:blast_furnace",
    "minecraft:brewing_stand",
    "minecraft:cartography_table",
    "minecraft:crafter_3x3",
    "minecraft:crafting",
    "minecraft:enchantment",
    "minecraft:furnace",
    "minecraft:generic_3x3",
    "minecraft:generic_9x1",
    "minecraft:generic_9x2",
    "minecraft:generic_9x3",
    "minecraft:generic_9x4",
    "minecraft:generic_9x5",
    "minecraft:generic_9x6",
    "minecraft:grindstone",
    "minecraft:hopper",
    "minecraft:lectern",
    "minecraft:loom",
    "minecraft:merchant",
    "minecraft:shulker_box",
    "minecraft:smithing",
    "minecraft:smoker",
    "minecraft:stonecutter",
];

#[derive(Clone, Debug, PartialEq)]
pub enum AutoF5Setting {
    CheckType(bool),
    Types(std::collections::BTreeSet<Option<String>>),
    TypeFilter(liquidbounce_behaviors::auto_f5::Filter),
    CheckTitle(bool),
    Titles(Vec<liquidbounce_behaviors::auto_f5::ContainerTitle>),
    Custom(std::collections::BTreeSet<String>),
    TitleFilter(liquidbounce_behaviors::auto_f5::Filter),
}

#[derive(Clone, Debug, PartialEq)]
pub enum AutoFishSetting {
    ReelDelay([i32; 2]),
    Sounds(Value),
    SoundDistance(bool),
    MaxDistance(f32),
    RecastRod(bool),
    RecastDelay([i32; 2]),
    AutoCastRod(bool),
    AutoCastDelay([i32; 2]),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsonFormat {
    Interop,
    File,
}

/// Supply actual registry state and the original InputBindAdapter JSON.
/// Key-name/scan-code conversion belongs to the bridge input mapping.
#[derive(Clone, Debug, PartialEq)]
pub struct CommonSettings {
    pub enabled: bool,
    pub hidden: bool,
    pub key_bind: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigError {
    pub path: String,
    pub message: String,
}

impl ConfigError {
    fn new(path: &str, message: impl Into<String>) -> Self {
        Self {
            path: path.to_owned(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path, self.message)
    }
}
impl std::error::Error for ConfigError {}

/// Empty settings used by registered modules whose game hooks are intentionally
/// inert until their protocol-specific implementation is available.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PassiveSetting;
#[derive(Clone, Debug, PartialEq)]
pub enum SettingChange {
    ElytraSwap(ElytraSwapSetting),
    Vomit(VomitSetting),
    FastBreak(FastBreakSetting),
    Timer(TimerSetting),
    BetterInventory(BetterInventorySetting),
    TerrainSpeed(TerrainSpeedSetting),
    HighJump(HighJumpSetting),
    NoSlow(NoSlowSetting),
    NoWeb(NoWebSetting),
    LiquidWalk(LiquidWalkSetting),
    Step(StepSetting),
    ReverseStep(ReverseStepSetting),
    AutoWalk(AutoWalkSetting),
    Parkour(ParkourSetting),
    SafeWalk(SafeWalkSetting),
    Anchor(AnchorSetting),
    AvoidHazards(AvoidHazardsSetting),
    Freeze(FreezeSetting),
    ResetVl(ResetVlSetting),
    OneShot(OneShotSetting),
    AutoDisable(AutoDisableSetting),
    Disabler(PassiveSetting),
    ServerCrasher(PassiveSetting),
    Dupe(PassiveSetting),
    TeleportReach(PassiveSetting),
    Phase(PassiveSetting),
    Notebot(PassiveSetting),
    AutoConfig(PassiveSetting),
    DebugRecorder(PassiveSetting),
    SkinChanger(PassiveSetting),
    AutoAccount(AutoAccountSetting),
    Breadcrumbs(BreadcrumbsSetting),
    Hats(HatsSetting),
    Wings(WingsSetting),
    Crosshair(CrosshairSetting),
    StorageEsp(StorageEspSetting),
    BlockEsp(BlockEspSetting),
    LogoffSpot(LogoffSpotSetting),
    InventoryTracker(InventoryTrackerSetting),
    NameProtect(NameProtectSetting),
    Particles(ParticlesSetting),
    Trajectories(TrajectoriesSetting),
    CustomAmbience(CustomAmbienceSetting),
    TNTTimer(TNTTimerSetting),
    PotionFX(PotionFXSetting),
    TotemEffect(TotemEffectSetting),
    HitFx(HitFxSetting),
    JumpEffect(JumpEffectSetting),
    HoleEsp(HoleEspSetting),
    ProtectionZones(ProtectionZonesSetting),
    ProphuntEsp(ProphuntEspSetting),
    VoidEsp(VoidEspSetting),
    NewChunks(NewChunksSetting),
    StrongholdFinder(StrongholdFinderSetting),
    Tracers(TracersSetting),
    Esp(EspSetting),
    Radar(RadarSetting),
    ItemEsp(ItemEspSetting),
    ItemChams(ItemChamsSetting),
    ItemTags(ItemTagsSetting),
    Nametags(NametagsSetting),
    BetterChat(BetterChatSetting),
    Debug(DebugSetting),
    ClickGui(ClickGuiSetting),
    BedPlates(BedPlatesSetting),
    DamageParticles(DamageParticlesSetting),
    BlockOutline(BlockOutlineSetting),
    BetterTab(BetterTabSetting),
    CombineMobs(CombineMobsSetting),
    MobOwners(MobOwnersSetting),
    ReportHelper(ReportHelperSetting),
    AutoQueue(AutoQueueSetting),
    Macros(MacrosSetting),
    MiddleClickAction(MiddleClickActionSetting),
    SuperKnockback(SuperKnockbackSetting),
    MaceKill(MaceKillSetting),
    SpearKill(SpearKillSetting),
    CrosshairClicker(CrosshairClickerSetting),
    Criticals(CriticalsSetting),
    Spider(SpiderSetting),
    OldRegen(OldRegenSetting),
    FastUse(FastUseSetting),
    FastExp(FastExpSetting),
    AutoWindCharge(AutoWindChargeSetting),
    AutoBuff(AutoBuffSetting),
    AutoBow(AutoBowSetting),
    AutoPearl(AutoPearlSetting),
    ElytraFly(bool),
    NoFall(bool),
    Extinguish(ExtinguishSetting),
    LiquidFiller(LiquidFillerSetting),
    HoleFiller(HoleFillerSetting),
    AutoShop(bool),
    AutoBuild(bool),
    AutoFarm(bool),
    CrystalAura(bool),
    Fucker(bool),
    AutoTrap(bool),
    BlockTrap(bool),
    Nuker(bool),
    TickBase(bool),
    Backtrack(bool),
    Fly(bool),
    LongJump(bool),
    Surround(SurroundSetting),
    BlockIn(BlockInSetting),
    BedDefender(BedDefenderSetting),
    EasyPearl(EasyPearlSetting),
    ElytraTarget(ElytraTargetSetting),
    AutoRod(AutoRodSetting),
    AutoShoot(AutoShootSetting),
    Scaffold(ScaffoldSetting),
    Clutch(ClutchSetting),
    FakeLag(FakeLagSetting),
    SwordBlock(SwordBlockSetting),
    Aimbot(AimbotSetting),
    TargetStrafe(TargetStrafeSetting),
    KillAura(KillAuraSetting),
    TpAura(bool),
    Velocity(bool),
    PacketMine(PacketMineSetting),
    Hud(HudSetting),
    Blink(BlinkSetting),
    AutoDodge(AutoDodgeSetting),
    AirPlace(AirPlaceSetting),
    FlagCheck(FlagCheckSetting),
    PacketLogger(PacketLoggerSetting),
    AntiStaff(AntiStaffSetting),
    Notifier(NotifierSetting),
    AntiBot(AntiBotSetting),
    AntiVoid(AntiVoidSetting),
    XRay(XRaySetting),
    InventoryMove(InventoryMoveSetting),
    Teams(TeamsSetting),
    MurderMystery(MurderMysterySetting),
    Offhand(OffhandSetting),
    ProjectilePuncher(ProjectilePuncherSetting),
    Spammer(SpammerSetting),
    AutoChatGame(AutoChatGameSetting),
    PingSpoof(PingSpoofSetting),
    Clip(ClipSetting),
    BookBot(BookBotSetting),
    ChestCleaner(ChestCleanerSetting),
    AutoDeposit(AutoDepositSetting),
    Replenish(ReplenishSetting),
    AutoCrafter(AutoCrafterSetting),
    InventoryCleaner(InventoryCleanerSetting),
    AutoArmor(AutoArmorSetting),
    AutoMobHeal(AutoMobHealSetting),
    ChestStealer(ChestStealerSetting),
    Eagle(EagleSetting),
    AntiExploit(AntiExploitSetting),
    Sneak(SneakSetting),
    Plugins(u32),
    TrueSight(TrueSightSetting),
    NoClip(NoClipSetting),
    PotionSpoof(PotionSpoofSetting),
    Damage(DamageSetting),
    YggdrasilSignatureFix(bool),
    TargetLock(TargetLockSetting),
    ItemScroller(ItemScrollerSetting),
    GuiCloser(GuiCloserSetting),
    TextFieldProtect(TextFieldProtectSetting),
    AntiHunger(AntiHungerSetting),
    Derp(DerpSetting),
    FreeLook(FreeLookSetting),
    FreeCam(FreeCamSetting),
    HandDerp(HandDerpSetting),
    VehicleBoost(VehicleBoostSetting),
    VehicleControl(VehicleControlSetting),
    BlockBounce(BlockBounceSetting),
    SmartEat(SmartEatSetting),
    SkinDerp(SkinDerpSetting),
    EntityControl(EntityControlSetting),
    BlockWalk(BlockWalkSetting),
    GhostHand(GhostHandSetting),
    ExtendedFirework(ExtendedFireworkSetting),
    Strafe(StrafeSetting),
    Speed(SpeedSetting),
    TridentBoost(TridentBoostSetting),
    AntiAfk(AntiAfkSetting),
    AutoTool(AutoToolSetting),
    NoCooldownProgress(bool),
    Chams(ChamsSetting),
    AutoWeapon(AutoWeaponSetting),
    Hitbox(HitboxSetting),
    MultiActions(MultiActionsSetting),
    Twerk(TwerkSetting),
    CameraClip(CameraClipSetting),
    DankBobbing(DankBobbingSetting),
    NoRotateSet(NoRotateSetSetting),
    NoEntityInteract(NoEntitySetting),
    NoPose(NoPoseSetting),
    Reach(ReachSetting),
    AutoF5(AutoF5Setting),
    AutoFish(AutoFishSetting),
    Enabled(bool),
    Bind(Value),
    Hidden(bool),
    NoFovMode(NoFovMode),
    ConstantFov(i32),
    CustomBaseFov(f32),
    CustomLimit(f32, f32),
    CustomMultiplier(f32),
    Zoom(i32),
    ScrollEnabled(bool),
    ScrollSpeed(f32),
    Transition(Easing),
    DurationFactor(f32),
    FullBrightMode(FullBrightMode),
    Brightness(i32),
    RemoveAttackCooldown(bool),
    CancelAttackOnMiss(bool),
    NoSwingHideFor(liquidbounce_behaviors::no_swing::HideFor),
    ServerSide(bool),
    ReduceInterpolationBy(i32),
    MiningConditions(liquidbounce_behaviors::no_slow_break::NoSlowBreak),
    SprintMode(liquidbounce_behaviors::sprint::SprintMode),
    SprintIgnore(u32),
    SprintStopOn(u32),
    RenderMask(u32),
    FireOpacity(i32),
    NoParticles(bool),
    SkipTerrainLoading(bool),
    AirJumpMode(liquidbounce_behaviors::air_jump::AirJumpMode),
    PushBy(u32),
    AspectRatio(i32),
    SmoothFirstPov(bool),
    SmoothResetPerspective(bool),
    SmoothHorizontal(f32),
    SmoothVertical(f32),
    RearView(bool),
    RespawnDelay(i32),
    KeepMotion([f32; 2]),
    KeepMotionHurt([f32; 2]),
    KeepHurtTime([i32; 2]),
    KeepChance(f32),
    CrystalSize(f32),
    CrystalY(f32),
    CrystalSpin(f32),
    CrystalBounce(f32),
    PlaceCooldown([i32; 2]),
    PlaceApplyTo(u32),
    PlaceStartDelay(i32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct SettingsPatch {
    pub module: ModuleKind,
    /// Declaration order, not input JSON order. Do not partition/reorder these.
    pub changes: Vec<SettingChange>,
    /// Original ConfigSystem catches these errors per leaf/group and continues.
    pub issues: Vec<ConfigError>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeRequest {
    ElytraSwap(ElytraSwapSetting),
    ItemScroller(ItemScrollerSetting),
    GuiCloser(GuiCloserSetting),
    TextFieldProtect(TextFieldProtectSetting),
    AntiHunger(AntiHungerSetting),
    HighJump(HighJumpSetting),
    NoSlow(NoSlowSetting),
    NoWeb(NoWebSetting),
    LiquidWalk(LiquidWalkSetting),
    Step(StepSetting),
    ReverseStep(ReverseStepSetting),
    AutoWalk(AutoWalkSetting),
    Parkour(ParkourSetting),
    SafeWalk(SafeWalkSetting),
    Anchor(AnchorSetting),
    AvoidHazards(AvoidHazardsSetting),
    Freeze(FreezeSetting),
    ResetVl(ResetVlSetting),
    OneShot(OneShotSetting),
    AutoDisable(AutoDisableSetting),
    Disabler(PassiveSetting),
    ServerCrasher(PassiveSetting),
    Dupe(PassiveSetting),
    TeleportReach(PassiveSetting),
    Phase(PassiveSetting),
    Notebot(PassiveSetting),
    AutoConfig(PassiveSetting),
    DebugRecorder(PassiveSetting),
    SkinChanger(PassiveSetting),
    AutoAccount(AutoAccountSetting),
    Breadcrumbs(BreadcrumbsSetting),
    Hats(HatsSetting),
    Wings(WingsSetting),
    Crosshair(CrosshairSetting),
    StorageEsp(StorageEspSetting),
    BlockEsp(BlockEspSetting),
    LogoffSpot(LogoffSpotSetting),
    InventoryTracker(InventoryTrackerSetting),
    NameProtect(NameProtectSetting),
    Particles(ParticlesSetting),
    Trajectories(TrajectoriesSetting),
    CustomAmbience(CustomAmbienceSetting),
    TNTTimer(TNTTimerSetting),
    PotionFX(PotionFXSetting),
    TotemEffect(TotemEffectSetting),
    HitFx(HitFxSetting),
    JumpEffect(JumpEffectSetting),
    HoleEsp(HoleEspSetting),
    ProtectionZones(ProtectionZonesSetting),
    ProphuntEsp(ProphuntEspSetting),
    VoidEsp(VoidEspSetting),
    NewChunks(NewChunksSetting),
    StrongholdFinder(StrongholdFinderSetting),
    Tracers(TracersSetting),
    Esp(EspSetting),
    Radar(RadarSetting),
    ItemEsp(ItemEspSetting),
    ItemChams(ItemChamsSetting),
    ItemTags(ItemTagsSetting),
    Nametags(NametagsSetting),
    BetterChat(BetterChatSetting),
    Debug(DebugSetting),
    ClickGui(ClickGuiSetting),
    BedPlates(BedPlatesSetting),
    DamageParticles(DamageParticlesSetting),
    BlockOutline(BlockOutlineSetting),
    BetterTab(BetterTabSetting),
    CombineMobs(CombineMobsSetting),
    MobOwners(MobOwnersSetting),
    ReportHelper(ReportHelperSetting),
    AutoQueue(AutoQueueSetting),
    Macros(MacrosSetting),
    MiddleClickAction(MiddleClickActionSetting),
    SuperKnockback(SuperKnockbackSetting),
    MaceKill(MaceKillSetting),
    SpearKill(SpearKillSetting),
    CrosshairClicker(CrosshairClickerSetting),
    Criticals(CriticalsSetting),
    Spider(SpiderSetting),
    OldRegen(OldRegenSetting),
    FastUse(FastUseSetting),
    FastExp(FastExpSetting),
    AutoWindCharge(AutoWindChargeSetting),
    AutoBuff(AutoBuffSetting),
    AutoBow(AutoBowSetting),
    AutoPearl(AutoPearlSetting),
    ElytraFly(bool),
    NoFall(bool),
    Extinguish(ExtinguishSetting),
    AutoShop(bool),
    AutoBuild(bool),
    AutoFarm(bool),
    CrystalAura(bool),
    Fucker(bool),
    AutoTrap(bool),
    BlockTrap(bool),
    Nuker(bool),
    TickBase(bool),
    Backtrack(bool),
    Fly(bool),
    LongJump(bool),
    LiquidFiller(LiquidFillerSetting),
    HoleFiller(HoleFillerSetting),
    Surround(SurroundSetting),
    BlockIn(BlockInSetting),
    BedDefender(BedDefenderSetting),
    EasyPearl(EasyPearlSetting),
    ElytraTarget(ElytraTargetSetting),
    AutoRod(AutoRodSetting),
    AutoShoot(AutoShootSetting),
    Scaffold(ScaffoldSetting),
    Clutch(ClutchSetting),
    FakeLag(FakeLagSetting),
    SwordBlock(SwordBlockSetting),
    Aimbot(AimbotSetting),
    TargetStrafe(TargetStrafeSetting),
    KillAura(KillAuraSetting),
    TpAura(bool),
    Velocity(bool),
    PacketMine(PacketMineSetting),
    Hud(HudSetting),
    Blink(BlinkSetting),
    AutoDodge(AutoDodgeSetting),
    AirPlace(AirPlaceSetting),
    FlagCheck(FlagCheckSetting),
    PacketLogger(PacketLoggerSetting),
    AntiStaff(AntiStaffSetting),
    Notifier(NotifierSetting),
    AntiBot(AntiBotSetting),
    AntiVoid(AntiVoidSetting),
    XRay(XRaySetting),
    InventoryMove(InventoryMoveSetting),
    Teams(TeamsSetting),
    MurderMystery(MurderMysterySetting),
    Offhand(OffhandSetting),
    ProjectilePuncher(ProjectilePuncherSetting),
    Spammer(SpammerSetting),
    AutoChatGame(AutoChatGameSetting),
    PingSpoof(PingSpoofSetting),
    Clip(ClipSetting),
    BookBot(BookBotSetting),
    ChestCleaner(ChestCleanerSetting),
    AutoDeposit(AutoDepositSetting),
    Replenish(ReplenishSetting),
    AutoCrafter(AutoCrafterSetting),
    InventoryCleaner(InventoryCleanerSetting),
    AutoArmor(AutoArmorSetting),
    AutoMobHeal(AutoMobHealSetting),
    ChestStealer(ChestStealerSetting),
    Eagle(EagleSetting),
    AntiExploit(AntiExploitSetting),
    Sneak(SneakSetting),
    Plugins(u32),
    TrueSight(TrueSightSetting),
    NoClip(NoClipSetting),
    PotionSpoof(PotionSpoofSetting),
    Damage(DamageSetting),
    YggdrasilSignatureFix(bool),
    TargetLock(TargetLockSetting),
    Vomit(VomitSetting),
    FastBreak(FastBreakSetting),
    Timer(TimerSetting),
    BetterInventory(BetterInventorySetting),
    TerrainSpeed(TerrainSpeedSetting),
    EntityControl(EntityControlSetting),
    BlockWalk(BlockWalkSetting),
    GhostHand(GhostHandSetting),
    ExtendedFirework(ExtendedFireworkSetting),
    Strafe(StrafeSetting),
    Speed(SpeedSetting),
    SmartEat(SmartEatSetting),
    SkinDerp(SkinDerpSetting),
    Derp(DerpSetting),
    FreeLook(FreeLookSetting),
    FreeCam(FreeCamSetting),
    HandDerp(HandDerpSetting),
    VehicleBoost(VehicleBoostSetting),
    VehicleControl(VehicleControlSetting),
    BlockBounce(BlockBounceSetting),
    TridentBoost(TridentBoostSetting),
    AntiAfk(AntiAfkSetting),
    AutoTool(AutoToolSetting),
    NoCooldownProgress(bool),
    Chams(ChamsSetting),
    AutoWeapon(AutoWeaponSetting),
    Hitbox(HitboxSetting),
    MultiActions(MultiActionsSetting),
    Twerk(TwerkSetting),
    CameraClip(CameraClipSetting),
    DankBobbing(DankBobbingSetting),
    NoRotateSet(NoRotateSetSetting),
    NoEntityInteract(NoEntitySetting),
    NoPose(NoPoseSetting),
    Reach(ReachSetting),
    AutoF5(AutoF5Setting),
    AutoFish(AutoFishSetting),
    Enabled(bool),
    Bind(Value),
    Hidden(bool),
    FullBrightMode(FullBrightMode),
    RemoveAttackCooldown(bool),
    CancelAttackOnMiss(bool),
    NoSwingHideFor(liquidbounce_behaviors::no_swing::HideFor),
    ServerSide(bool),
    ReduceInterpolationBy(i32),
    MiningConditions(liquidbounce_behaviors::no_slow_break::NoSlowBreak),
    SprintMode(liquidbounce_behaviors::sprint::SprintMode),
    SprintIgnore(u32),
    SprintStopOn(u32),
    RenderMask(u32),
    FireOpacity(i32),
    NoParticles(bool),
    SkipTerrainLoading(bool),
    AirJumpMode(liquidbounce_behaviors::air_jump::AirJumpMode),
    PushBy(u32),
    AspectRatio(i32),
    SmoothFirstPov(bool),
    SmoothResetPerspective(bool),
    SmoothHorizontal(f32),
    SmoothVertical(f32),
    RearView(bool),
    RespawnDelay(i32),
    KeepMotion([f32; 2]),
    KeepMotionHurt([f32; 2]),
    KeepHurtTime([i32; 2]),
    KeepChance(f32),
    CrystalSize(f32),
    CrystalY(f32),
    CrystalSpin(f32),
    CrystalBounce(f32),
    PlaceCooldown([i32; 2]),
    PlaceApplyTo(u32),
    PlaceStartDelay(i32),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ApplyResult {
    Changed(bool),
    Runtime(RuntimeRequest),
}

struct Tree<'a> {
    format: JsonFormat,
    describe: &'a dyn Fn(&str) -> String,
}

impl Tree<'_> {
    fn group(&self, name: &str, kind: &str, children: Vec<Value>) -> Value {
        let mut object = json!({"name":name,"value":children});
        if self.format == JsonFormat::Interop {
            object["valueType"] = json!(kind);
        }
        object
    }

    fn leaf(
        &self,
        name: &str,
        kind: &str,
        value: Value,
        key: &str,
        description_key: Option<&str>,
    ) -> Value {
        let mut object = json!({"name":name,"value":value});
        if self.format == JsonFormat::Interop {
            object["valueType"] = json!(kind);
            object["key"] = json!(key);
            let fallback = format!("{key}.description");
            object["description"] = json!((self.describe)(description_key.unwrap_or(&fallback)));
        }
        object
    }

    fn ranged(
        &self,
        name: &str,
        kind: &str,
        value: Value,
        range: Value,
        suffix: &str,
        key: &str,
    ) -> Value {
        let mut leaf = self.leaf(name, kind, value, key, None);
        if self.format == JsonFormat::Interop {
            leaf["range"] = range;
            leaf["suffix"] = json!(suffix);
        }
        leaf
    }

    fn mode(&self, active: &str, choices: Map<String, Value>) -> Value {
        let mut mode = self.group("Mode", "CHOICE", vec![]);
        mode["active"] = json!(active);
        mode["choices"] = Value::Object(choices);
        mode
    }

    fn multi(&self, name: &str, mask: u32, choices: &[&str], key: &str) -> Value {
        let selected: Vec<_> = choices
            .iter()
            .enumerate()
            .filter_map(|(index, tag)| (mask & (1 << index) != 0).then_some(*tag))
            .collect();
        let mut value = self.leaf(name, "MULTI_CHOOSE", json!(selected), key, None);
        if self.format == JsonFormat::Interop {
            value["choices"] = json!(choices);
            value["canBeNone"] = json!(true);
            value["isOrderSensitive"] = json!(false);
        }
        value
    }

    fn common(&self, module: ModuleKind, common: &CommonSettings) -> Vec<Value> {
        let key = module.key();
        let mut values = Vec::new();
        // ClientModule marks Enabled notAnOption, excluded by INTEROP only.
        if self.format == JsonFormat::File {
            values.push(self.leaf(
                "Enabled",
                "BOOLEAN",
                json!(common.enabled),
                &format!("{key}.enabled"),
                None,
            ));
        }
        values.push(self.leaf(
            "Bind",
            "BIND",
            common.key_bind.clone(),
            &format!("{key}.bind"),
            Some("liquidbounce.common.bind.description"),
        ));
        values.push(self.leaf(
            "Hidden",
            "BOOLEAN",
            json!(common.hidden),
            &format!("{key}.hidden"),
            Some("liquidbounce.common.hidden.description"),
        ));
        values
    }
}

fn float_json(value: f32, path: &str) -> Result<Value, ConfigError> {
    if !value.is_finite() {
        return Err(ConfigError::new(
            path,
            "non-finite float cannot be serialized by the original Gson",
        ));
    }
    // Java Float.toString emits shortest f32 text; promoting directly to f64
    // would expose binary rounding (0.1 -> 0.10000000149011612) in the UI.
    let mut text = value.to_string();
    if !text.contains(['.', 'e', 'E']) {
        text.push_str(".0");
    }
    text.parse::<Number>()
        .map(Value::Number)
        .map_err(|error| ConfigError::new(path, error.to_string()))
}

fn float_range(from: f32, to: f32, path: &str) -> Result<Value, ConfigError> {
    Ok(json!({"from":float_json(from,path)?,"to":float_json(to,path)?}))
}

/// Preserve common registry state for the explicitly inert native modules.
/// Their upstream-specific options are not presented as implemented settings.
pub fn serialize_unimplemented(
    module: ModuleKind,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    if !module.is_unimplemented() {
        return Err(ConfigError::new(module.name(), "module is not an unimplemented native module"));
    }
    let tree = Tree { format, describe };
    let mut value = tree.group(module.name(), "TOGGLEABLE", tree.common(module, common));
    value["implementation"] = json!("unimplemented");
    Ok(value)
}

/// Modules which declare no values beyond ClientModule's common settings.
pub fn serialize_camera_effect(
    module: ModuleKind,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    if matches!(
        module,
        ModuleKind::NoFov
            | ModuleKind::Zoom
            | ModuleKind::FullBright
            | ModuleKind::NoMissCooldown
            | ModuleKind::NoSwing
            | ModuleKind::NoPitchLimit
            | ModuleKind::NoInterpolation
            | ModuleKind::NoSlowBreak
            | ModuleKind::Sprint
            | ModuleKind::AntiBlind
            | ModuleKind::AirJump
            | ModuleKind::NoPush
            | ModuleKind::Aspect
            | ModuleKind::SmoothCamera
            | ModuleKind::QuickPerspectiveSwap
            | ModuleKind::AutoRespawn
            | ModuleKind::KeepSprint
            | ModuleKind::CrystalView
            | ModuleKind::FastPlace
            | ModuleKind::AutoF5
            | ModuleKind::AutoFish
            | ModuleKind::Reach
            | ModuleKind::NoPose
            | ModuleKind::NoRotateSet
            | ModuleKind::CameraClip
            | ModuleKind::DankBobbing
            | ModuleKind::MultiActions
            | ModuleKind::Twerk
            | ModuleKind::Hitbox
            | ModuleKind::AutoWeapon
            | ModuleKind::Chams
            | ModuleKind::AutoTool
            | ModuleKind::AntiAfk
            | ModuleKind::BlockBounce
            | ModuleKind::SmartEat
            | ModuleKind::SkinDerp
            | ModuleKind::EntityControl
            | ModuleKind::SnapTap
            | ModuleKind::BlockWalk
            | ModuleKind::GhostHand
            | ModuleKind::ExtendedFirework
            | ModuleKind::Strafe
            | ModuleKind::ElytraFly
            | ModuleKind::NoFall
            | ModuleKind::FastBreak
            | ModuleKind::Timer
            | ModuleKind::BetterInventory
            | ModuleKind::TerrainSpeed
            | ModuleKind::HighJump
            | ModuleKind::NoSlow
            | ModuleKind::NoWeb
            | ModuleKind::LiquidWalk
            | ModuleKind::Step
            | ModuleKind::ReverseStep
            | ModuleKind::AutoWalk
            | ModuleKind::Parkour
            | ModuleKind::SafeWalk
            | ModuleKind::Anchor
            | ModuleKind::AvoidHazards
            | ModuleKind::Freeze
            | ModuleKind::ResetVl
            | ModuleKind::YggdrasilSignatureFix
            | ModuleKind::Damage
            | ModuleKind::Kick
            | ModuleKind::AutoLeave
            | ModuleKind::AutoDisable
            | ModuleKind::Disabler
            | ModuleKind::ServerCrasher
            | ModuleKind::Dupe
            | ModuleKind::TeleportReach
            | ModuleKind::Phase
            | ModuleKind::Notebot
            | ModuleKind::AutoConfig
            | ModuleKind::DebugRecorder
            | ModuleKind::SkinChanger
            | ModuleKind::AutoAccount
            | ModuleKind::Breadcrumbs
            | ModuleKind::Hats
            | ModuleKind::Wings
            | ModuleKind::Crosshair
            | ModuleKind::StorageEsp
            | ModuleKind::BlockEsp
            | ModuleKind::LogoffSpot
            | ModuleKind::InventoryTracker
            | ModuleKind::NameProtect
            | ModuleKind::Particles
            | ModuleKind::Trajectories
            | ModuleKind::CustomAmbience
            | ModuleKind::TNTTimer
            | ModuleKind::PotionFX
            | ModuleKind::TotemEffect
            | ModuleKind::HitFx
            | ModuleKind::JumpEffect
            | ModuleKind::HoleEsp
            | ModuleKind::ProtectionZones
            | ModuleKind::ProphuntEsp
            | ModuleKind::VoidEsp
            | ModuleKind::NewChunks
            | ModuleKind::StrongholdFinder
            | ModuleKind::Tracers
            | ModuleKind::Esp
            | ModuleKind::Radar
            | ModuleKind::ItemEsp
            | ModuleKind::ItemChams
            | ModuleKind::ItemTags
            | ModuleKind::Nametags
            | ModuleKind::BetterChat
            | ModuleKind::Debug
            | ModuleKind::ClickGui
            | ModuleKind::BedPlates
            | ModuleKind::DamageParticles
            | ModuleKind::BlockOutline
            | ModuleKind::BetterTab
            | ModuleKind::CombineMobs
            | ModuleKind::MobOwners
            | ModuleKind::ReportHelper
            | ModuleKind::AutoQueue
            | ModuleKind::Macros
            | ModuleKind::MiddleClickAction
            | ModuleKind::SuperKnockback
            | ModuleKind::MaceKill
            | ModuleKind::SpearKill
            | ModuleKind::TriggerBot
            | ModuleKind::Criticals
            | ModuleKind::AutoClicker
            | ModuleKind::Spider
            | ModuleKind::OldRegen
            | ModuleKind::FastUse
            | ModuleKind::FastExp
            | ModuleKind::AutoWindCharge
            | ModuleKind::AutoBuff
            | ModuleKind::AutoBow
            | ModuleKind::AutoPearl
            | ModuleKind::Extinguish
            | ModuleKind::LiquidFiller
            | ModuleKind::HoleFiller
            | ModuleKind::AutoShop
            | ModuleKind::AutoBuild
            | ModuleKind::AutoFarm
            | ModuleKind::CrystalAura
            | ModuleKind::Fucker
            | ModuleKind::AutoTrap
            | ModuleKind::BlockTrap
            | ModuleKind::Nuker
            | ModuleKind::TickBase
            | ModuleKind::Backtrack
            | ModuleKind::Fly
            | ModuleKind::LongJump
            | ModuleKind::Surround
            | ModuleKind::BlockIn
            | ModuleKind::BedDefender
            | ModuleKind::EasyPearl
            | ModuleKind::ElytraTarget
            | ModuleKind::AutoRod
            | ModuleKind::AutoShoot
            | ModuleKind::Scaffold
            | ModuleKind::Clutch
            | ModuleKind::FakeLag
            | ModuleKind::SwordBlock
            | ModuleKind::Aimbot
            | ModuleKind::TargetStrafe
            | ModuleKind::KillAura
            | ModuleKind::TpAura
            | ModuleKind::Velocity
            | ModuleKind::PacketMine
            | ModuleKind::Hud
            | ModuleKind::Blink
            | ModuleKind::AutoDodge
            | ModuleKind::AirPlace
            | ModuleKind::FlagCheck
            | ModuleKind::PacketLogger
            | ModuleKind::AntiStaff
            | ModuleKind::Notifier
            | ModuleKind::AntiBot
            | ModuleKind::AntiVoid
            | ModuleKind::XRay
            | ModuleKind::InventoryMove
            | ModuleKind::Teams
            | ModuleKind::MurderMystery
            | ModuleKind::Offhand
            | ModuleKind::ProjectilePuncher
            | ModuleKind::Spammer
            | ModuleKind::AutoChatGame
            | ModuleKind::PingSpoof
            | ModuleKind::Clip
            | ModuleKind::BookBot
            | ModuleKind::ChestCleaner
            | ModuleKind::AutoDeposit
            | ModuleKind::Replenish
            | ModuleKind::AutoCrafter
            | ModuleKind::InventoryCleaner
            | ModuleKind::AutoArmor
            | ModuleKind::AutoMobHeal
            | ModuleKind::ChestStealer
            | ModuleKind::Eagle
            | ModuleKind::AntiExploit
            | ModuleKind::Sneak
            | ModuleKind::Plugins
            | ModuleKind::TrueSight
            | ModuleKind::NoClip
            | ModuleKind::PotionSpoof
            | ModuleKind::ItemScroller
            | ModuleKind::GuiCloser
            | ModuleKind::TextFieldProtect
            | ModuleKind::TargetLock
            | ModuleKind::AntiHunger
            | ModuleKind::Vomit
            | ModuleKind::ElytraSwap
            | ModuleKind::VehicleBoost
            | ModuleKind::VehicleControl
            | ModuleKind::HandDerp
            | ModuleKind::FreeLook
            | ModuleKind::FreeCam
            | ModuleKind::TridentBoost
            | ModuleKind::SilentHotbar
            | ModuleKind::NoEntityInteract
    ) {
        return Err(ConfigError::new(
            module.name(),
            "module has specific settings",
        ));
    }
    let tree = Tree { format, describe };
    Ok(tree.group(module.name(), "TOGGLEABLE", tree.common(module, common)))
}

pub fn serialize_silent_hotbar(
    state: &liquidbounce_behaviors::silent_hotbar_render::SilentHotbarRender,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::SilentHotbar, common);
    values.push(tree.leaf(
        "NoCooldownProgress",
        "BOOLEAN",
        json!(state.no_cooldown_progress),
        "liquidbounce.module.silentHotbar.noCooldownProgress",
        None,
    ));
    Ok(tree.group("SilentHotbar", "TOGGLEABLE", values))
}

pub fn serialize_no_miss_cooldown(
    state: &NoMissCooldown,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let module = ModuleKind::NoMissCooldown;
    let mut values = tree.common(module, common);
    for (name, value, suffix) in [
        (
            "RemoveAttackCooldown",
            state.remove_attack_cooldown,
            "removeAttackCooldown",
        ),
        (
            "CancelAttackOnMiss",
            state.cancel_attack_on_miss,
            "cancelAttackOnMiss",
        ),
    ] {
        values.push(tree.leaf(
            name,
            "BOOLEAN",
            json!(value),
            &format!("{}.{suffix}", module.key()),
            None,
        ));
    }
    Ok(tree.group(module.name(), "TOGGLEABLE", values))
}

pub fn serialize_no_swing(
    state: &liquidbounce_behaviors::no_swing::NoSwing,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let module = ModuleKind::NoSwing;
    let mut values = tree.common(module, common);
    values.push(
        liquidbounce_behaviors::no_swing::serialize_no_swing_hide_for(
            state.hide_for(),
            format == JsonFormat::Interop,
            &describe("liquidbounce.module.noSwing.hideFor.description"),
        ),
    );
    Ok(tree.group(module.name(), "TOGGLEABLE", values))
}

pub fn serialize_no_pitch_limit(
    state: &liquidbounce_behaviors::no_pitch_limit::NoPitchLimit,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let module = ModuleKind::NoPitchLimit;
    let mut values = tree.common(module, common);
    values.push(tree.leaf(
        "ServerSide",
        "BOOLEAN",
        json!(state.server_side()),
        "liquidbounce.module.noPitchLimit.serverSide",
        None,
    ));
    Ok(tree.group(module.name(), "TOGGLEABLE", values))
}

pub fn serialize_no_interpolation(
    state: &liquidbounce_behaviors::no_interpolation::NoInterpolation,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let module = ModuleKind::NoInterpolation;
    let mut values = tree.common(module, common);
    values.push(tree.ranged(
        "ReduceBy",
        "INT",
        json!(state.reduce_by),
        json!({"from":1,"to":3}),
        "",
        "liquidbounce.module.noInterpolation.reduceBy",
    ));
    Ok(tree.group(module.name(), "TOGGLEABLE", values))
}

pub fn serialize_crystal_view(
    state: &liquidbounce_behaviors::crystal_view::CrystalView,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::CrystalView, common);
    for (name, value, from, to, key) in [
        (
            "Size",
            state.size,
            0.1,
            1.5,
            "liquidbounce.module.crystalView.size",
        ),
        (
            "YTranslate",
            state.y_translate,
            -2.0,
            2.0,
            "liquidbounce.module.crystalView.yTranslate",
        ),
        (
            "SpinSpeed",
            state.spin_speed,
            0.0,
            5.0,
            "liquidbounce.module.crystalView.spinSpeed",
        ),
        (
            "Bounce",
            state.bounce,
            -1.0,
            1.0,
            "liquidbounce.module.crystalView.bounce",
        ),
    ] {
        values.push(tree.ranged(
            name,
            "FLOAT",
            float_json(value, key)?,
            float_range(from, to, key)?,
            "",
            key,
        ));
    }
    Ok(tree.group("CrystalView", "TOGGLEABLE", values))
}

pub fn serialize_auto_fish(
    state: &liquidbounce_behaviors::auto_fish::AutoFish,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let state = &state.settings;
    let mut values = tree.common(ModuleKind::AutoFish, common);
    let delay =
        |name: &str, range: liquidbounce_behaviors::auto_fish::DelayRange, from, to, key: &str| {
            tree.ranged(
                name,
                "INT_RANGE",
                json!({"from":range.first,"to":range.last}),
                json!({"from":from,"to":to}),
                "ticks",
                key,
            )
        };
    values.push(delay(
        "ReelDelay",
        state.reel_delay,
        0,
        20,
        "liquidbounce.module.autoFish.reelDelay",
    ));
    let mut sounds = tree.leaf(
        "Sounds",
        "REGISTRY_LIST",
        liquidbounce_behaviors::auto_fish_sounds::serialize_sounds(&state.sounds),
        "liquidbounce.module.autoFish.sounds",
        None,
    );
    if format == JsonFormat::Interop {
        sounds["registry"] = json!("sound_event");
    }
    values.push(sounds);
    values.push(tree.group(
        "SoundDistance",
        "TOGGLEABLE",
        vec![
            tree.leaf(
                "Enabled",
                "BOOLEAN",
                json!(state.sound_distance),
                "liquidbounce.module.autoFish.soundDistance.enabled",
                None,
            ),
            tree.ranged(
                "MaxDistance",
                "FLOAT",
                float_json(state.max_distance, "AutoFish.SoundDistance.MaxDistance")?,
                float_range(0.0, 10.0, "AutoFish.SoundDistance.MaxDistance")?,
                "blocks",
                "liquidbounce.module.autoFish.soundDistance.maxDistance",
            ),
        ],
    ));
    for (name, enabled, range, from, to, key) in [
        (
            "RecastRod",
            state.recast_rod,
            state.recast_delay,
            10,
            30,
            "liquidbounce.module.autoFish.recastRod",
        ),
        (
            "AutoCastRod",
            state.auto_cast_rod,
            state.auto_cast_delay,
            0,
            30,
            "liquidbounce.module.autoFish.autoCastRod",
        ),
    ] {
        values.push(tree.group(
            name,
            "TOGGLEABLE",
            vec![
                tree.leaf(
                    "Enabled",
                    "BOOLEAN",
                    json!(enabled),
                    &format!("{key}.enabled"),
                    None,
                ),
                delay("Delay", range, from, to, &format!("{key}.delay")),
            ],
        ));
    }
    Ok(tree.group("AutoFish", "TOGGLEABLE", values))
}

pub fn serialize_auto_f5(
    state: &liquidbounce_behaviors::auto_f5::AutoF5,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    use liquidbounce_behaviors::auto_f5::{ContainerTitle, Filter};
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::AutoF5, common);
    let choice = |filter, key: &str| {
        let mut value = tree.leaf(
            "Filter",
            "CHOOSE",
            json!(match filter {
                Filter::Whitelist => "Whitelist",
                Filter::Blacklist => "Blacklist",
            }),
            key,
            None,
        );
        if format == JsonFormat::Interop {
            value["choices"] = json!(["Whitelist", "Blacklist"]);
        }
        value
    };
    let mut types = tree.leaf(
        "Types",
        "REGISTRY_LIST",
        liquidbounce_behaviors::auto_fish_sounds::serialize_sounds(&state.types),
        "liquidbounce.module.autoF5.checkScreenHandlerType.types",
        None,
    );
    if format == JsonFormat::Interop {
        types["registry"] = json!("menu");
    }
    values.push(tree.group(
        "CheckScreenHandlerType",
        "TOGGLEABLE",
        vec![
            tree.leaf(
                "Enabled",
                "BOOLEAN",
                json!(state.check_screen_handler_type),
                "liquidbounce.module.autoF5.checkScreenHandlerType.enabled",
                None,
            ),
            types,
            choice(
                state.type_filter,
                "liquidbounce.module.autoF5.checkScreenHandlerType.filter",
            ),
        ],
    ));
    let tags: Vec<_> = state.titles.iter().map(|title| title.tag()).collect();
    let mut titles = tree.leaf(
        "Titles",
        "MULTI_CHOOSE",
        json!(tags),
        "liquidbounce.module.autoF5.checkScreenTitle.titles",
        None,
    );
    if format == JsonFormat::Interop {
        titles["choices"] = json!(ContainerTitle::ALL.map(ContainerTitle::tag));
        titles["canBeNone"] = json!(true);
        titles["isOrderSensitive"] = json!(false);
    }
    values.push(tree.group(
        "CheckScreenTitle",
        "TOGGLEABLE",
        vec![
            tree.leaf(
                "Enabled",
                "BOOLEAN",
                json!(state.check_screen_title),
                "liquidbounce.module.autoF5.checkScreenTitle.enabled",
                None,
            ),
            titles,
            tree.leaf(
                "Custom",
                "MUTABLE_LIST",
                json!(state.custom_titles),
                "liquidbounce.module.autoF5.checkScreenTitle.custom",
                None,
            ),
            choice(
                state.title_filter,
                "liquidbounce.module.autoF5.checkScreenTitle.filter",
            ),
        ],
    ));
    Ok(tree.group("AutoF5", "TOGGLEABLE", values))
}

pub fn serialize_fast_place(
    state: &liquidbounce_behaviors::fast_place::FastPlace,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::FastPlace, common);
    values.push(tree.ranged(
        "Cooldown",
        "INT_RANGE",
        json!({"from":state.cooldown.0,"to":state.cooldown.1}),
        json!({"from":0,"to":4}),
        "ticks",
        "liquidbounce.module.fastPlace.cooldown",
    ));
    values.push(tree.multi(
        "ApplyTo",
        u32::from(state.projectiles) | u32::from(state.blocks) << 1,
        &["Projectiles", "Blocks"],
        "liquidbounce.module.fastPlace.applyTo",
    ));
    values.push(tree.ranged(
        "StartDelay",
        "INT",
        json!(state.start_delay),
        json!({"from":0,"to":1000}),
        "ms",
        "liquidbounce.module.fastPlace.startDelay",
    ));
    Ok(tree.group("FastPlace", "TOGGLEABLE", values))
}

pub fn serialize_auto_respawn(
    state: &liquidbounce_behaviors::auto_respawn::AutoRespawn,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::AutoRespawn, common);
    values.push(tree.ranged(
        "Delay",
        "INT",
        json!(state.delay()),
        json!({"from":0,"to":20}),
        "ticks",
        "liquidbounce.module.autoRespawn.delay",
    ));
    Ok(tree.group("AutoRespawn", "TOGGLEABLE", values))
}

pub fn serialize_keep_sprint(
    state: &liquidbounce_behaviors::keep_sprint::KeepSprint,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::KeepSprint, common);
    for (name, value, key) in [
        (
            "Motion",
            state.motion,
            "liquidbounce.module.keepSprint.motion",
        ),
        (
            "MotionWhenHurt",
            state.motion_when_hurt,
            "liquidbounce.module.keepSprint.motionWhenHurt",
        ),
    ] {
        values.push(tree.ranged(
            name,
            "FLOAT_RANGE",
            float_range(value[0], value[1], key)?,
            json!({"from":0.0,"to":100.0}),
            "%",
            key,
        ));
    }
    values.push(tree.ranged(
        "HurtTime",
        "INT_RANGE",
        json!({"from":state.hurt_time[0],"to":state.hurt_time[1]}),
        json!({"from":1,"to":10}),
        "",
        "liquidbounce.module.keepSprint.hurtTime",
    ));
    values.push(tree.ranged(
        "Chance",
        "FLOAT",
        float_json(state.chance, "KeepSprint.Chance")?,
        json!({"from":0.0,"to":100.0}),
        "%",
        "liquidbounce.module.keepSprint.chance",
    ));
    Ok(tree.group("KeepSprint", "TOGGLEABLE", values))
}

pub fn serialize_aspect(
    state: &liquidbounce_behaviors::camera_control::Aspect,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::Aspect, common);
    values.push(tree.ranged(
        "Ratio",
        "INT",
        json!(state.ratio),
        json!({"from":1,"to":300}),
        "%",
        "liquidbounce.module.aspect.ratio",
    ));
    Ok(tree.group("Aspect", "TOGGLEABLE", values))
}

pub fn serialize_smooth_camera(
    state: &liquidbounce_behaviors::camera_control::SmoothCamera,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::SmoothCamera, common);
    values.push(tree.leaf(
        "EnableFirstPOV",
        "BOOLEAN",
        json!(state.enable_first_pov),
        "liquidbounce.module.smoothCamera.enableFirstPOV",
        None,
    ));
    values.push(tree.leaf(
        "ResetOnPerspectiveChange",
        "BOOLEAN",
        json!(state.reset_on_perspective_change),
        "liquidbounce.module.smoothCamera.resetOnPerspectiveChange",
        None,
    ));
    for (name, value, key) in [
        (
            "HorizontalFactor",
            state.horizontal_factor,
            "liquidbounce.module.smoothCamera.horizontalFactor",
        ),
        (
            "VerticalFactor",
            state.vertical_factor,
            "liquidbounce.module.smoothCamera.verticalFactor",
        ),
    ] {
        values.push(tree.ranged(
            name,
            "FLOAT",
            float_json(value, key)?,
            json!({"from":0.0,"to":1.0}),
            "",
            key,
        ));
    }
    Ok(tree.group("SmoothCamera", "TOGGLEABLE", values))
}

pub fn serialize_quick_perspective_swap(
    state: &liquidbounce_behaviors::camera_control::QuickPerspectiveSwap,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::QuickPerspectiveSwap, common);
    values.push(tree.leaf(
        "RearView",
        "BOOLEAN",
        json!(state.rear_view),
        "liquidbounce.module.quickPerspectiveSwap.rearView",
        None,
    ));
    Ok(tree.group("QuickPerspectiveSwap", "TOGGLEABLE", values))
}

pub fn serialize_air_jump(
    state: &liquidbounce_behaviors::air_jump::AirJump,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    use liquidbounce_behaviors::air_jump::AirJumpMode;
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::AirJump, common);
    let tag = match state.mode {
        AirJumpMode::JumpFreely => "JumpFreely",
        AirJumpMode::DoubleJump => "DoubleJump",
        AirJumpMode::GhostBlock => "GhostBlock",
    };
    let mut mode = tree.leaf(
        "Mode",
        "CHOOSE",
        json!(tag),
        "liquidbounce.module.airJump.mode",
        None,
    );
    if format == JsonFormat::Interop {
        mode["choices"] = json!(["JumpFreely", "DoubleJump", "GhostBlock"]);
    }
    values.push(mode);
    Ok(tree.group("AirJump", "TOGGLEABLE", values))
}

pub fn serialize_no_push(
    state: &liquidbounce_behaviors::no_push::NoPush,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::NoPush, common);
    let mask = u32::from(state.entities)
        | u32::from(state.blocks) << 1
        | u32::from(state.fishing_rod) << 2
        | u32::from(state.liquids) << 3
        | u32::from(state.sinking) << 4;
    values.push(tree.multi(
        "PushBy",
        mask,
        &["Entities", "Blocks", "FishingRod", "Liquids", "Sinking"],
        "liquidbounce.module.noPush.pushBy",
    ));
    Ok(tree.group("NoPush", "TOGGLEABLE", values))
}

pub fn serialize_no_slow_break(
    state: &liquidbounce_behaviors::no_slow_break::NoSlowBreak,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::NoSlowBreak, common);
    let mask = u32::from(state.mining_fatigue)
        | u32::from(state.on_air) << 1
        | u32::from(state.underwater) << 2;
    values.push(tree.multi(
        "When",
        mask,
        &["MiningFatigue", "OnAir", "Underwater"],
        "liquidbounce.module.noSlowBreak.when",
    ));
    Ok(tree.group("NoSlowBreak", "TOGGLEABLE", values))
}

pub fn serialize_sprint(
    state: &liquidbounce_behaviors::sprint::Sprint,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    use liquidbounce_behaviors::sprint::SprintMode;
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::Sprint, common);
    let tag = match state.mode {
        SprintMode::Legit => "Legit",
        SprintMode::Omnidirectional => "Omnidirectional",
        SprintMode::Omnirotational => "Omnirotational",
    };
    let mut mode = tree.leaf(
        "Mode",
        "CHOOSE",
        json!(tag),
        "liquidbounce.module.sprint.mode",
        None,
    );
    if format == JsonFormat::Interop {
        mode["choices"] = json!(["Legit", "Omnidirectional", "Omnirotational"]);
    }
    values.push(mode);
    values.push(tree.multi(
        "Ignore",
        u32::from(state.ignore.blindness)
            | u32::from(state.ignore.hunger) << 1
            | u32::from(state.ignore.collision) << 2,
        &["Blindness", "Hunger", "Collision"],
        "liquidbounce.module.sprint.ignore",
    ));
    values.push(tree.multi(
        "StopOn",
        u32::from(state.stop_on.ground)
            | u32::from(state.stop_on.air) << 1
            | u32::from(state.stop_on.sneaking) << 2
            | u32::from(state.stop_on.using_item) << 3,
        &["Ground", "Air", "Sneaking", "UsingItem"],
        "liquidbounce.module.sprint.stopOn",
    ));
    Ok(tree.group("Sprint", "TOGGLEABLE", values))
}

pub fn serialize_anti_blind(
    state: &liquidbounce_behaviors::visual_environment::AntiBlind,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::AntiBlind, common);
    values.push(tree.multi(
        "DoRender",
        state.render_mask,
        &liquidbounce_behaviors::visual_environment::DoRender::TAGS,
        "liquidbounce.module.antiBlind.doRender",
    ));
    values.push(tree.ranged(
        "FireOpacity",
        "INT",
        json!(state.fire_opacity),
        json!({"from":0,"to":100}),
        "%",
        "liquidbounce.module.antiBlind.fireOpacity",
    ));
    values.push(tree.leaf(
        "NoParticles",
        "BOOLEAN",
        json!(state.no_particles),
        "liquidbounce.module.antiBlind.noParticles",
        None,
    ));
    values.push(tree.leaf(
        "SkipTerrainLoading",
        "BOOLEAN",
        json!(state.skip_terrain_loading),
        "liquidbounce.module.antiBlind.skipTerrainLoading",
        None,
    ));
    Ok(tree.group("AntiBlind", "TOGGLEABLE", values))
}

pub fn serialize_no_fov(
    state: &NoFov,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let base = ModuleKind::NoFov.key();
    let mut choices = Map::new();
    choices.insert(
        "Constant".to_owned(),
        tree.group(
            "Constant",
            "CONFIGURABLE",
            vec![tree.ranged(
                "FOV",
                "INT",
                json!(state.constant_fov),
                json!({"from":1,"to":179}),
                "",
                &format!("{base}.mode.constant.fOV"),
            )],
        ),
    );
    choices.insert(
        "Custom".to_owned(),
        tree.group(
            "Custom",
            "CONFIGURABLE",
            vec![
                tree.ranged(
                    "BaseFOV",
                    "FLOAT",
                    float_json(state.custom.base_fov, "NoFOV.Mode.Custom.BaseFOV")?,
                    float_range(0.0, 1.5, "BaseFOV.range")?,
                    "",
                    &format!("{base}.mode.custom.baseFOV"),
                ),
                tree.ranged(
                    "Limit",
                    "FLOAT_RANGE",
                    float_range(
                        state.custom.limit.0,
                        state.custom.limit.1,
                        "NoFOV.Mode.Custom.Limit",
                    )?,
                    float_range(0.0, 1.5, "Limit.range")?,
                    "",
                    &format!("{base}.mode.custom.limit"),
                ),
                tree.ranged(
                    "Multiplier",
                    "FLOAT",
                    float_json(state.custom.multiplier, "NoFOV.Mode.Custom.Multiplier")?,
                    float_range(0.1, 1.5, "Multiplier.range")?,
                    "",
                    &format!("{base}.mode.custom.multiplier"),
                ),
            ],
        ),
    );
    let mut children = tree.common(ModuleKind::NoFov, common);
    children.push(tree.mode(state.mode.tag(), choices));
    Ok(tree.group("NoFOV", "TOGGLEABLE", children))
}

pub fn serialize_zoom(
    state: &Zoom,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let base = ModuleKind::Zoom.key();
    let mut children = tree.common(ModuleKind::Zoom, common);
    children.push(tree.ranged(
        "Zoom",
        "INT",
        json!(state.settings.zoom),
        json!({"from":10,"to":150}),
        "",
        &format!("{base}.zoom"),
    ));
    children.push(tree.group(
        "Scroll",
        "TOGGLEABLE",
        vec![
            tree.leaf(
                "Enabled",
                "BOOLEAN",
                json!(state.settings.scroll_enabled),
                &format!("{base}.scroll.enabled"),
                None,
            ),
            tree.ranged(
                "Speed",
                "FLOAT",
                float_json(state.settings.scroll_speed, "Zoom.Scroll.Speed")?,
                float_range(0.5, 8.0, "Speed.range")?,
                "",
                &format!("{base}.scroll.speed"),
            ),
        ],
    ));
    let mut transition = tree.leaf(
        "Transition",
        "CHOOSE",
        json!(state.settings.transition.tag()),
        &format!("{base}.transition"),
        None,
    );
    if format == JsonFormat::Interop {
        transition["choices"] = json!(Easing::ALL.map(Easing::tag));
    }
    children.push(transition);
    children.push(tree.ranged(
        "DurationFactor",
        "FLOAT",
        float_json(state.settings.duration_factor, "Zoom.DurationFactor")?,
        float_range(0.0, 10.0, "DurationFactor.range")?,
        "x",
        &format!("{base}.durationFactor"),
    ));
    Ok(tree.group("Zoom", "TOGGLEABLE", children))
}

pub fn serialize_full_bright(
    state: &FullBright,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut choices = Map::new();
    choices.insert(
        "Gamma".to_owned(),
        tree.group(
            "Gamma",
            "CONFIGURABLE",
            vec![tree.ranged(
                "Brightness",
                "INT",
                json!(state.gamma.brightness),
                json!({"from":1,"to":15}),
                "",
                "liquidbounce.module.fullBright.mode.gamma.brightness",
            )],
        ),
    );
    choices.insert(
        "NightVision".to_owned(),
        tree.group("NightVision", "CONFIGURABLE", vec![]),
    );
    let mut children = tree.common(ModuleKind::FullBright, common);
    children.push(tree.mode(state.mode().tag(), choices));
    Ok(tree.group("FullBright", "TOGGLEABLE", children))
}

type Nodes<'a> = BTreeMap<String, &'a Value>;

fn scalar_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(_) | Value::Bool(_) => Some(value.to_string()),
        _ => None,
    }
}

fn nodes<'a>(json: &'a Value, expected: &str, path: &str) -> Result<Nodes<'a>, ConfigError> {
    let object = json
        .as_object()
        .ok_or_else(|| ConfigError::new(path, "expected a group object"))?;
    let name = object
        .get("name")
        .and_then(scalar_string)
        .ok_or_else(|| ConfigError::new(path, "missing group name"))?;
    if name != expected {
        return Err(ConfigError::new(
            path,
            format!("config name {name:?} does not match {expected:?}"),
        ));
    }
    let values = object
        .get("value")
        .and_then(Value::as_array)
        .ok_or_else(|| ConfigError::new(path, "expected value array"))?;
    let mut by_name = BTreeMap::new();
    for node in values {
        let name = node
            .as_object()
            .and_then(|node| node.get("name"))
            .and_then(scalar_string)
            .ok_or_else(|| ConfigError::new(path, "stored value must be an object with name"))?;
        // The original queues duplicates and each declared value consumes once.
        by_name.entry(name).or_insert(node);
    }
    Ok(by_name)
}

fn value_of<'a>(node: &'a Value, path: &str) -> Result<&'a Value, ConfigError> {
    node.get("value")
        .filter(|value| !value.is_null())
        .ok_or_else(|| ConfigError::new(path, "missing value"))
}

fn parse_bool(value: &Value, path: &str) -> Result<bool, ConfigError> {
    match value {
        Value::Bool(value) => Ok(*value),
        Value::String(value) => Ok(value.eq_ignore_ascii_case("true")),
        _ => Err(ConfigError::new(path, "expected boolean or boolean string")),
    }
}

fn parse_float(value: &Value, path: &str) -> Result<f32, ConfigError> {
    let text = match value {
        Value::Number(value) => value.to_string(),
        Value::String(value) => value
            .trim_matches(|character: char| character <= '\u{20}')
            .to_owned(),
        _ => return Err(ConfigError::new(path, "expected float or numeric string")),
    };
    let text = text.strip_suffix(['f', 'F', 'd', 'D']).unwrap_or(&text);
    text.parse()
        .map_err(|_| ConfigError::new(path, "invalid float"))
}

/// Gson's JsonTreeReader -> LazilyParsedNumber.intValue truncates decimal
/// Numbers and keeps the low 32 bits. Strings instead use Integer.parseInt.
/// serde_json arbitrary_precision retains the original lexical number for this.
fn pair_members<'a>(value: &'a Value, path: &str) -> Result<[&'a Value; 2], ConfigError> {
    let object = value
        .as_object()
        .ok_or_else(|| ConfigError::new(path, "expected {from,to} range"))?;
    Ok([
        object
            .get("from")
            .ok_or_else(|| ConfigError::new(path, "missing range from"))?,
        object
            .get("to")
            .ok_or_else(|| ConfigError::new(path, "missing range to"))?,
    ])
}
fn parse_float_pair(value: &Value, path: &str) -> Result<[f32; 2], ConfigError> {
    let [from, to] = pair_members(value, path)?;
    Ok([parse_float(from, path)?, parse_float(to, path)?])
}
fn parse_int_pair(value: &Value, path: &str) -> Result<[i32; 2], ConfigError> {
    // IntRangeAdapter accepts a JSON number as a singleton; numeric strings
    // are accepted only for the object members through JsonPrimitive.asInt.
    if value.is_number() {
        let number = parse_int(value, path)?;
        return Ok([number, number]);
    }
    let [from, to] = pair_members(value, path)?;
    Ok([parse_int(from, path)?, parse_int(to, path)?])
}

fn parse_int(value: &Value, path: &str) -> Result<i32, ConfigError> {
    match value {
        Value::String(value) => value
            .parse()
            .map_err(|_| ConfigError::new(path, "invalid integer string")),
        Value::Number(value) => {
            decimal_int(&value.to_string()).ok_or_else(|| ConfigError::new(path, "invalid integer"))
        }
        _ => Err(ConfigError::new(path, "expected integer or numeric string")),
    }
}

fn decimal_int(text: &str) -> Option<i32> {
    let (negative, text) = text
        .strip_prefix('-')
        .map_or((false, text), |text| (true, text));
    let (mantissa, exponent) = text
        .split_once(['e', 'E'])
        .map_or(Some((text, 0_i64)), |(mantissa, exp)| {
            exp.parse::<i64>().ok().map(|exp| (mantissa, exp))
        })?;
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let digits: Vec<_> = whole.bytes().chain(fraction.bytes()).collect();
    if digits.is_empty() || digits.iter().any(|digit| !digit.is_ascii_digit()) {
        return None;
    }
    let integer_digits = (digits.len() as i64)
        .saturating_add(exponent)
        .saturating_sub(fraction.len() as i64);
    if integer_digits <= 0 {
        return Some(0);
    }
    let mut value = 0_u32;
    for digit in digits
        .iter()
        .take(integer_digits.min(digits.len() as i64) as usize)
    {
        value = value
            .wrapping_mul(10)
            .wrapping_add(u32::from(*digit - b'0'));
    }
    let zeros = integer_digits.saturating_sub(digits.len() as i64);
    if zeros >= 32 {
        value = 0;
    } else {
        for _ in 0..zeros {
            value = value.wrapping_mul(10);
        }
    }
    Some(if negative {
        value.wrapping_neg() as i32
    } else {
        value as i32
    })
}

fn leaf(
    patch: &mut SettingsPatch,
    by_name: &Nodes<'_>,
    name: &str,
    path: &str,
    parse: impl FnOnce(&Value, &str) -> Result<SettingChange, ConfigError>,
) {
    if let Some(node) = by_name.get(name) {
        match value_of(node, path).and_then(|value| parse(value, path)) {
            Ok(change) => patch.changes.push(change),
            Err(error) => patch.issues.push(error),
        }
    }
}

fn parse_multi_mask(value: &Value, path: &str, choices: &[&str]) -> Result<u32, ConfigError> {
    let mut mask = 0;
    let mut include = |value: &Value| -> Result<(), ConfigError> {
        if !value.is_string() && !value.is_number() && !value.is_boolean() {
            return Err(ConfigError::new(
                path,
                "multichoice entries must be JSON primitives",
            ));
        }
        if let Some(tag) = value.as_str()
            && let Some(index) = choices
                .iter()
                .position(|choice| choice.eq_ignore_ascii_case(tag))
        {
            mask |= 1 << index;
        }
        Ok(())
    };
    match value {
        Value::Array(values) => {
            for value in values {
                include(value)?;
            }
        }
        Value::String(_) | Value::Number(_) | Value::Bool(_) => include(value)?,
        _ => {}
    }
    Ok(mask)
}

pub fn parse_settings_text(module: ModuleKind, source: &str) -> Result<SettingsPatch, ConfigError> {
    let json = serde_json::from_str(source)
        .map_err(|error| ConfigError::new(module.name(), error.to_string()))?;
    parse_settings(module, &json)
}

pub fn parse_settings(module: ModuleKind, json: &Value) -> Result<SettingsPatch, ConfigError> {
    let group_name = if module == ModuleKind::MultiActions && json["name"] == "MultiTask" {
        "MultiTask"
    } else if module == ModuleKind::SleepWalker && json["name"] == "BedGodMode" {
        "BedGodMode"
    } else if module == ModuleKind::TerrainSpeed && json["name"] == "FastClimb" {
        "FastClimb"
    } else {
        module.name()
    };
    let root = nodes(json, group_name, module.name())?;
    let mut patch = SettingsPatch {
        module,
        changes: Vec::new(),
        issues: Vec::new(),
    };
    leaf(&mut patch, &root, "Enabled", "Enabled", |value, path| {
        parse_bool(value, path).map(SettingChange::Enabled)
    });
    leaf(&mut patch, &root, "Bind", "Bind", |value, path| {
        if value.is_array() {
            return Err(ConfigError::new(
                path,
                "expected InputBind object or legacy primitive",
            ));
        }
        // Preserve this request for the bridge's original key-name adapter.
        Ok(SettingChange::Bind(value.clone()))
    });
    leaf(&mut patch, &root, "Hidden", "Hidden", |value, path| {
        parse_bool(value, path).map(SettingChange::Hidden)
    });
    match module {
        ModuleKind::Derp => derp::parse_derp(&mut patch, &root),
        ModuleKind::FreeLook => free_look::parse_free_look(&mut patch, &root),
        ModuleKind::FreeCam=>free_cam::parse_free_cam(&mut patch,&root),
        ModuleKind::HandDerp => hand_derp::parse_hand_derp(&mut patch, &root),
        ModuleKind::VehicleBoost => vehicle_boost::parse_vehicle_boost(&mut patch, &root),
        ModuleKind::VehicleControl=>vehicle_control::parse_vehicle_control(&mut patch,&root),
        ModuleKind::BlockBounce => block_bounce::parse_block_bounce(&mut patch, &root),
        ModuleKind::SmartEat => smart_eat::parse_smart_eat(&mut patch, &root),
        ModuleKind::SkinDerp => skin_derp::parse_skin_derp(&mut patch, &root),
        ModuleKind::EntityControl => entity_control::parse_entity_control(&mut patch, &root),
        ModuleKind::BlockWalk => block_walk::parse_block_walk(&mut patch, &root),
        ModuleKind::GhostHand => ghost_hand::parse_ghost_hand(&mut patch, &root),
        ModuleKind::ExtendedFirework => {
            extended_firework::parse_extended_firework(&mut patch, &root)
        }
        ModuleKind::Strafe => strafe::parse_strafe(&mut patch, &root),
        ModuleKind::Speed => speed::parse_speed(&mut patch, &root),
        ModuleKind::FastBreak => fast_break::parse_fast_break(&mut patch, &root),
        ModuleKind::Timer => timer::parse_timer(&mut patch, &root),
        ModuleKind::BetterInventory => better_inventory::parse_better_inventory(&mut patch, &root),
        ModuleKind::TerrainSpeed => terrain_speed::parse_terrain_speed(&mut patch, &root),
        ModuleKind::HighJump => high_jump::parse_high_jump(&mut patch, &root),
        ModuleKind::NoSlow => no_slow::parse_no_slow(&mut patch,&root),
        ModuleKind::NoWeb => no_web::parse_no_web(&mut patch,&root),
        ModuleKind::LiquidWalk => liquid_walk::parse_liquid_walk(&mut patch,&root),
        ModuleKind::Step => step::parse_step(&mut patch,&root),
        ModuleKind::ReverseStep => reverse_step::parse_reverse_step(&mut patch,&root),
        ModuleKind::AutoWalk => auto_walk::parse_auto_walk(&mut patch, &root),
        ModuleKind::Parkour => edge_assistance::parse_parkour(&mut patch, &root),
        ModuleKind::SafeWalk => edge_assistance::parse_safe_walk(&mut patch, &root),
        ModuleKind::Anchor => movement_guard::parse_anchor(&mut patch,&root),
        ModuleKind::AvoidHazards => movement_guard::parse_avoid_hazards(&mut patch,&root),
        ModuleKind::Freeze => movement_guard::parse_freeze(&mut patch,&root),
        ModuleKind::ResetVl => reset_vl::parse_reset_vl(&mut patch, &root),
        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),
        ModuleKind::AutoDisable => auto_disable::parse_auto_disable(&mut patch, &root),
        ModuleKind::Disabler
        | ModuleKind::ServerCrasher
        | ModuleKind::Dupe
        | ModuleKind::TeleportReach
        | ModuleKind::Phase
        | ModuleKind::Notebot
        | ModuleKind::AutoConfig
        | ModuleKind::DebugRecorder
        | ModuleKind::SkinChanger => {}
        ModuleKind::AutoAccount => auto_account::parse_auto_account(&mut patch, &root),
        ModuleKind::Hats => hats::parse_hats(&mut patch,&root),
        ModuleKind::Wings => wings::parse_wings(&mut patch,&root),
        ModuleKind::Crosshair => crosshair::parse_crosshair(&mut patch,&root),
        ModuleKind::StorageEsp=>storage_esp::parse_storage_esp(&mut patch,&root),
        ModuleKind::BlockEsp=>block_esp::parse_block_esp(&mut patch,&root),
        ModuleKind::LogoffSpot => logoff_spot::parse_logoff_spot(&mut patch,&root),
        ModuleKind::InventoryTracker => inventory_tracker::parse_inventory_tracker(&mut patch,&root),
        ModuleKind::NameProtect => name_protect::parse_name_protect(&mut patch,&root),
        ModuleKind::Particles => particles::parse_particles(&mut patch,&root),
        ModuleKind::Trajectories=>trajectories::parse_trajectories(&mut patch,&root),
        ModuleKind::CustomAmbience=>custom_ambience::parse_custom_ambience(&mut patch,&root),
        ModuleKind::TNTTimer=>tnt_timer::parse_tnt_timer(&mut patch,&root),
        ModuleKind::PotionFX=>potion_fx::parse_potion_fx(&mut patch,&root),
        ModuleKind::TotemEffect=>totem_effect::parse_totem_effect(&mut patch,&root),
        ModuleKind::HitFx=>hit_fx::parse_hit_fx(&mut patch,&root),
        ModuleKind::JumpEffect=>jump_effect::parse_jump_effect(&mut patch,&root),
        ModuleKind::HoleEsp => hole_esp::parse_hole_esp(&mut patch,&root),
        ModuleKind::ProtectionZones => protection_zones::parse_protection_zones(&mut patch,&root),
        ModuleKind::ProphuntEsp => placement_esp::parse_prophunt_esp(&mut patch,&root),
        ModuleKind::VoidEsp => placement_esp::parse_void_esp(&mut patch,&root),
        ModuleKind::NewChunks => new_chunks::parse_new_chunks(&mut patch,&root),
        ModuleKind::StrongholdFinder=>stronghold_finder::parse_stronghold_finder(&mut patch,&root),
        ModuleKind::Tracers => tracers::parse_tracers(&mut patch,&root),
        ModuleKind::Esp=>esp::parse_esp(&mut patch,&root),
        ModuleKind::Radar=>radar::parse_radar(&mut patch,&root),
        ModuleKind::ItemEsp=>item_esp::parse_item_esp(&mut patch,&root),
        ModuleKind::ItemChams=>item_chams::parse_item_chams(&mut patch,&root),
        ModuleKind::ItemTags=>item_tags::parse_item_tags(&mut patch,&root),
        ModuleKind::Nametags=>nametags::parse_nametags(&mut patch,&root),
        ModuleKind::BetterChat=>better_chat::parse_better_chat(&mut patch,&root),
        ModuleKind::Debug=>debug::parse_debug(&mut patch,&root),
        ModuleKind::ClickGui=>click_gui::parse_click_gui(&mut patch,&root),
        ModuleKind::BedPlates=>bed_plates::parse_bed_plates(&mut patch,&root),
        ModuleKind::DamageParticles => damage_particles::parse_damage_particles(&mut patch,&root),
        ModuleKind::Breadcrumbs => breadcrumbs::parse_breadcrumbs(&mut patch,&root),
        ModuleKind::BlockOutline => block_outline::parse_block_outline(&mut patch,&root),
        ModuleKind::BetterTab => better_tab::parse_better_tab(&mut patch, &root),
        ModuleKind::CombineMobs => combine_mobs::parse_combine_mobs(&mut patch, &root),
        ModuleKind::MobOwners => mob_owners::parse_mob_owners(&mut patch, &root),
        ModuleKind::ReportHelper=>report_helper::parse_report_helper(&mut patch,&root),
        ModuleKind::AutoQueue=>auto_queue::parse_auto_queue(&mut patch,&root),
        ModuleKind::Macros=>macros::parse_macros(&mut patch,&root),
        ModuleKind::MiddleClickAction => {
            middle_click_action::parse_middle_click_action(&mut patch, &root)
        }
        ModuleKind::SuperKnockback => super_knockback::parse_super_knockback(&mut patch, &root),
        ModuleKind::MaceKill => mace_kill::parse_mace_kill(&mut patch, &root),
        ModuleKind::SpearKill => spear_kill::parse_spear_kill(&mut patch, &root),
        ModuleKind::TriggerBot => crosshair_clicker::parse_trigger_bot(&mut patch,&root),
        ModuleKind::Criticals=>criticals::parse_criticals(&mut patch,&root),
        ModuleKind::AutoClicker => crosshair_clicker::parse_auto_clicker(&mut patch,&root),
        ModuleKind::Spider => spider::parse_spider(&mut patch, &root),
        ModuleKind::OldRegen => old_regen::parse_old_regen(&mut patch, &root),
        ModuleKind::FastUse => fast_use::parse_fast_use(&mut patch, &root),
        ModuleKind::FastExp=>fast_exp::parse_fast_exp(&mut patch,&root),
        ModuleKind::AutoWindCharge=>auto_wind_charge::parse_auto_wind_charge(&mut patch,&root),
        ModuleKind::AutoBuff=>auto_buff::parse_auto_buff(&mut patch,&root),
        ModuleKind::AutoBow=>auto_bow::parse_auto_bow(&mut patch,&root),
        ModuleKind::AutoPearl=>auto_pearl::parse_auto_pearl(&mut patch,&root),
        ModuleKind::Extinguish=>extinguish::parse_extinguish(&mut patch,&root),
        ModuleKind::LiquidFiller=>liquid_filler::parse_liquid_filler(&mut patch,&root),
        ModuleKind::HoleFiller=>hole_filler::parse_hole_filler(&mut patch,&root),
        ModuleKind::AutoShop=>world_noop::parse_auto_shop(&mut patch,&root),
        ModuleKind::AutoBuild=>world_noop::parse_auto_build(&mut patch,&root),
        ModuleKind::AutoFarm=>world_noop::parse_auto_farm(&mut patch,&root),
        ModuleKind::CrystalAura=>world_noop::parse_crystal_aura(&mut patch,&root),
        ModuleKind::Fucker=>world_noop::parse_fucker(&mut patch,&root),
        ModuleKind::AutoTrap=>world_noop::parse_auto_trap(&mut patch,&root),
        ModuleKind::BlockTrap=>world_noop::parse_block_trap(&mut patch,&root),
        ModuleKind::Nuker=>world_noop::parse_nuker(&mut patch,&root),
        ModuleKind::TickBase=>world_noop::parse_tick_base(&mut patch,&root),
        ModuleKind::Backtrack=>world_noop::parse_backtrack(&mut patch,&root),
        ModuleKind::Fly=>world_noop::parse_fly(&mut patch,&root),
        ModuleKind::LongJump=>world_noop::parse_long_jump(&mut patch,&root),
        ModuleKind::Surround=>surround::parse_surround(&mut patch,&root),
        ModuleKind::BlockIn=>block_in::parse_block_in(&mut patch,&root),
        ModuleKind::BedDefender=>bed_defender::parse_bed_defender(&mut patch,&root),
        ModuleKind::EasyPearl=>easy_pearl::parse_easy_pearl(&mut patch,&root),
        ModuleKind::ElytraTarget=>elytra_target::parse_elytra_target(&mut patch,&root),
        ModuleKind::AutoRod=>auto_rod::parse_auto_rod(&mut patch,&root),
        ModuleKind::AutoShoot=>auto_shoot::parse_auto_shoot(&mut patch,&root),
        ModuleKind::Scaffold=>scaffold::parse_scaffold(&mut patch,&root),
        ModuleKind::Clutch=>scaffold::parse_clutch(&mut patch,&root),
        ModuleKind::FakeLag=>fake_lag::parse_fake_lag(&mut patch,&root),
        ModuleKind::SwordBlock=>sword_block::parse_sword_block(&mut patch,&root),
        ModuleKind::Aimbot=>aimbot::parse_aimbot(&mut patch,&root),
        ModuleKind::TargetStrafe=>target_strafe::parse_target_strafe(&mut patch,&root),
        ModuleKind::KillAura=>kill_aura::parse_kill_aura(&mut patch,&root),
        ModuleKind::PacketMine=>packet_mine::parse_packet_mine(&mut patch,&root),
        ModuleKind::Hud=>hud::parse_hud(&mut patch,&root),
        ModuleKind::Blink=>blink::parse_blink(&mut patch,&root),
        ModuleKind::AutoDodge=>auto_dodge::parse_auto_dodge(&mut patch,&root),
        ModuleKind::AirPlace=>air_place::parse_air_place(&mut patch,&root),
        ModuleKind::FlagCheck=>flag_check::parse_flag_check(&mut patch,&root),
        ModuleKind::PacketLogger=>packet_logger::parse_packet_logger(&mut patch,&root),
        ModuleKind::AntiStaff=>anti_staff::parse_anti_staff(&mut patch,&root),
        ModuleKind::Notifier=>notifier::parse_notifier(&mut patch,&root),
        ModuleKind::AntiBot=>anti_bot::parse_anti_bot(&mut patch,&root),
        ModuleKind::AntiVoid=>anti_void::parse_anti_void(&mut patch,&root),
        ModuleKind::XRay=>xray::parse_xray(&mut patch,&root),
        ModuleKind::InventoryMove=>inventory_move::parse_inventory_move(&mut patch,&root),
        ModuleKind::Teams => teams::parse_teams(&mut patch,&root),
        ModuleKind::MurderMystery=>murder_mystery::parse_murder_mystery(&mut patch,&root),
        ModuleKind::Offhand=>offhand::parse_offhand(&mut patch,&root),
        ModuleKind::ProjectilePuncher=>projectile_puncher::parse_projectile_puncher(&mut patch,&root),
        ModuleKind::Spammer => spammer::parse_spammer(&mut patch,&root),
        ModuleKind::AutoChatGame=>auto_chat_game::parse_auto_chat_game(&mut patch,&root),
        ModuleKind::PingSpoof => ping_spoof::parse_ping_spoof(&mut patch,&root),
        ModuleKind::Clip => clip::parse_clip(&mut patch,&root),
        ModuleKind::BookBot => book_bot::parse_book_bot(&mut patch,&root),
        ModuleKind::ChestCleaner => chest_cleaner::parse_chest_cleaner(&mut patch,&root),
        ModuleKind::AutoDeposit => auto_deposit::parse_auto_deposit(&mut patch,&root),
        ModuleKind::Replenish => replenish::parse_replenish(&mut patch,&root),
        ModuleKind::AutoCrafter=>auto_crafter::parse_auto_crafter(&mut patch,&root),
        ModuleKind::InventoryCleaner => inventory_cleaner::parse_inventory_cleaner(&mut patch,&root),
        ModuleKind::AutoArmor => auto_armor::parse_auto_armor(&mut patch,&root),
        ModuleKind::AutoMobHeal => auto_mob_heal::parse_auto_mob_heal(&mut patch,&root),
        ModuleKind::ChestStealer => chest_stealer::parse_chest_stealer(&mut patch,&root),
        ModuleKind::Eagle => eagle::parse_eagle(&mut patch,&root),
        ModuleKind::AntiExploit => anti_exploit::parse_anti_exploit(&mut patch,&root),
        ModuleKind::Sneak => sneak::parse_sneak(&mut patch,&root),
        ModuleKind::Plugins => plugins::parse_plugins(&mut patch, &root),
        ModuleKind::TrueSight => true_sight::parse_true_sight(&mut patch, &root),
        ModuleKind::NoClip => no_clip::parse_no_clip(&mut patch, &root),
        ModuleKind::PotionSpoof => potion_spoof::parse_potion_spoof(&mut patch, &root),
        ModuleKind::YggdrasilSignatureFix => {
            yggdrasil_signature::parse_yggdrasil_signature(&mut patch, &root)
        }
        ModuleKind::Kick | ModuleKind::AutoLeave => {
            one_shots::parse_one_shots(&mut patch, &root, module)
        }
        ModuleKind::ItemScroller => item_scroller::parse_item_scroller(&mut patch, &root),
        ModuleKind::TargetLock => target_lock::parse_target_lock(&mut patch, &root),
        ModuleKind::GuiCloser => gui_closer::parse_gui_closer(&mut patch, &root),
        ModuleKind::TextFieldProtect => {
            text_field_protect::parse_text_field_protect(&mut patch, &root)
        }
        ModuleKind::AntiHunger => anti_hunger::parse_anti_hunger(&mut patch, &root),
        ModuleKind::Vomit => vomit::parse_vomit(&mut patch, &root),
        ModuleKind::ElytraSwap => elytra_swap::parse_elytra_swap(&mut patch, &root),
        ModuleKind::Reach => reach::parse_reach(&mut patch, &root),
        ModuleKind::NoPose => no_pose::parse_no_pose(&mut patch, &root),
        ModuleKind::NoRotateSet => no_rotate_set::parse_no_rotate_set(&mut patch, &root),
        ModuleKind::CameraClip => camera_clip::parse_camera_clip(&mut patch, &root),
        ModuleKind::DankBobbing => dank_bobbing::parse_dank_bobbing(&mut patch, &root),
        ModuleKind::MultiActions => multi_actions::parse_multi_actions(&mut patch, &root),
        ModuleKind::Twerk => twerk::parse_twerk(&mut patch, &root),
        ModuleKind::Hitbox => hitbox::parse_hitbox(&mut patch, &root),
        ModuleKind::AutoWeapon => auto_weapon::parse_auto_weapon(&mut patch, &root),
        ModuleKind::Chams => chams::parse_chams(&mut patch, &root),
        ModuleKind::AutoTool => auto_tool::parse_auto_tool(&mut patch, &root),
        ModuleKind::AntiAfk => anti_afk::parse_anti_afk(&mut patch, &root),
        ModuleKind::TridentBoost => trident_boost::parse_trident_boost(&mut patch, &root),
        ModuleKind::SilentHotbar => leaf(
            &mut patch,
            &root,
            "NoCooldownProgress",
            "NoCooldownProgress",
            |value, path| parse_bool(value, path).map(SettingChange::NoCooldownProgress),
        ),
        ModuleKind::NoEntityInteract => no_interact::parse_no_entity_interact(&mut patch, &root),
        ModuleKind::AutoFish => {
            leaf(
                &mut patch,
                &root,
                "ReelDelay",
                "AutoFish.ReelDelay",
                |value, path| {
                    parse_int_pair(value, path)
                        .map(|value| SettingChange::AutoFish(AutoFishSetting::ReelDelay(value)))
                },
            );
            leaf(
                &mut patch,
                &root,
                "Sounds",
                "AutoFish.Sounds",
                |value, path| {
                    // Resolve against the live registry later on the actor, while
                    // rejecting the complete malformed leaf before any mutation.
                    liquidbounce_behaviors::auto_fish_sounds::deserialize_sounds(value, &[])
                        .map_err(|error| ConfigError::new(path, error))?;
                    Ok(SettingChange::AutoFish(AutoFishSetting::Sounds(
                        value.clone(),
                    )))
                },
            );
            for name in ["SoundDistance", "RecastRod", "AutoCastRod"] {
                let Some(group) = root.get(name) else {
                    continue;
                };
                let group = match nodes(group, name, name) {
                    Ok(group) => group,
                    Err(error) => {
                        patch.issues.push(error);
                        continue;
                    }
                };
                leaf(&mut patch, &group, "Enabled", name, |value, path| {
                    parse_bool(value, path).map(|value| {
                        SettingChange::AutoFish(match name {
                            "SoundDistance" => AutoFishSetting::SoundDistance(value),
                            "RecastRod" => AutoFishSetting::RecastRod(value),
                            _ => AutoFishSetting::AutoCastRod(value),
                        })
                    })
                });
                if name == "SoundDistance" {
                    leaf(
                        &mut patch,
                        &group,
                        "MaxDistance",
                        "AutoFish.SoundDistance.MaxDistance",
                        |value, path| {
                            parse_float(value, path).map(|value| {
                                SettingChange::AutoFish(AutoFishSetting::MaxDistance(value))
                            })
                        },
                    );
                } else {
                    leaf(&mut patch, &group, "Delay", name, |value, path| {
                        parse_int_pair(value, path).map(|value| {
                            SettingChange::AutoFish(if name == "RecastRod" {
                                AutoFishSetting::RecastDelay(value)
                            } else {
                                AutoFishSetting::AutoCastDelay(value)
                            })
                        })
                    });
                }
            }
        }
        ModuleKind::AutoF5 => {
            for (name, is_type) in [
                ("CheckScreenHandlerType", true),
                ("CheckScreenTitle", false),
            ] {
                let Some(group) = root.get(name) else {
                    continue;
                };
                let group = match nodes(group, name, name) {
                    Ok(group) => group,
                    Err(error) => {
                        patch.issues.push(error);
                        continue;
                    }
                };
                leaf(&mut patch, &group, "Enabled", name, |value, path| {
                    parse_bool(value, path).map(|value| {
                        SettingChange::AutoF5(if is_type {
                            AutoF5Setting::CheckType(value)
                        } else {
                            AutoF5Setting::CheckTitle(value)
                        })
                    })
                });
                if is_type {
                    leaf(
                        &mut patch,
                        &group,
                        "Types",
                        "AutoF5.CheckScreenHandlerType.Types",
                        |value, path| {
                            let registry = MENU_IDS
                                .iter()
                                .map(|name| (*name).to_owned())
                                .collect::<Vec<_>>();
                            let types =
                                liquidbounce_behaviors::auto_fish_sounds::deserialize_sounds(
                                    value, &registry,
                                )
                                .map_err(|error| ConfigError::new(path, error))?;
                            Ok(SettingChange::AutoF5(AutoF5Setting::Types(types)))
                        },
                    );
                } else {
                    leaf(
                        &mut patch,
                        &group,
                        "Titles",
                        "AutoF5.CheckScreenTitle.Titles",
                        |value, path| {
                            use liquidbounce_behaviors::auto_f5::ContainerTitle;
                            let tags = ContainerTitle::ALL.map(ContainerTitle::tag);
                            let mask = parse_multi_mask(value, path, &tags)?;
                            Ok(SettingChange::AutoF5(AutoF5Setting::Titles(
                                ContainerTitle::ALL
                                    .into_iter()
                                    .enumerate()
                                    .filter_map(|(index, title)| {
                                        (mask & (1 << index) != 0).then_some(title)
                                    })
                                    .collect(),
                            )))
                        },
                    );
                    leaf(
                        &mut patch,
                        &group,
                        "Custom",
                        "AutoF5.CheckScreenTitle.Custom",
                        |value, path| {
                            let values = match value {
                                Value::Array(values) => values.clone(),
                                _ => vec![value.clone()],
                            };
                            values
                                .iter()
                                .map(|value| {
                                    scalar_string(value)
                                        .ok_or_else(|| ConfigError::new(path, "expected text"))
                                })
                                .collect::<Result<_, _>>()
                                .map(|values| SettingChange::AutoF5(AutoF5Setting::Custom(values)))
                        },
                    );
                }
                leaf(&mut patch, &group, "Filter", name, |value, path| {
                    use liquidbounce_behaviors::auto_f5::Filter;
                    let tag = scalar_string(value)
                        .ok_or_else(|| ConfigError::new(path, "expected filter tag"))?;
                    let filter = if tag.eq_ignore_ascii_case("Whitelist") {
                        Filter::Whitelist
                    } else if tag.eq_ignore_ascii_case("Blacklist") {
                        Filter::Blacklist
                    } else {
                        return Err(ConfigError::new(path, "unknown filter"));
                    };
                    Ok(SettingChange::AutoF5(if is_type {
                        AutoF5Setting::TypeFilter(filter)
                    } else {
                        AutoF5Setting::TitleFilter(filter)
                    }))
                });
            }
        }
        ModuleKind::Zoom => {
            leaf(&mut patch, &root, "Zoom", "Zoom.Zoom", |value, path| {
                parse_int(value, path).map(SettingChange::Zoom)
            });
            if let Some(scroll) = root.get("Scroll") {
                match nodes(scroll, "Scroll", "Zoom.Scroll") {
                    Ok(scroll) => {
                        leaf(
                            &mut patch,
                            &scroll,
                            "Enabled",
                            "Zoom.Scroll.Enabled",
                            |value, path| parse_bool(value, path).map(SettingChange::ScrollEnabled),
                        );
                        leaf(
                            &mut patch,
                            &scroll,
                            "Speed",
                            "Zoom.Scroll.Speed",
                            |value, path| parse_float(value, path).map(SettingChange::ScrollSpeed),
                        );
                    }
                    Err(error) => patch.issues.push(error),
                }
            }
            leaf(
                &mut patch,
                &root,
                "Transition",
                "Zoom.Transition",
                |value, path| {
                    let tag = scalar_string(value)
                        .ok_or_else(|| ConfigError::new(path, "expected easing tag"))?;
                    Easing::ALL
                        .into_iter()
                        .find(|easing| easing.tag().eq_ignore_ascii_case(&tag))
                        .map(SettingChange::Transition)
                        .ok_or_else(|| ConfigError::new(path, format!("unknown easing {tag:?}")))
                },
            );
            leaf(
                &mut patch,
                &root,
                "DurationFactor",
                "Zoom.DurationFactor",
                |value, path| parse_float(value, path).map(SettingChange::DurationFactor),
            );
        }
        ModuleKind::NoFov | ModuleKind::FullBright => {
            if let Some(mode) = root.get("Mode") {
                parse_mode(&mut patch, mode);
            }
        }
        ModuleKind::NoMissCooldown => {
            leaf(
                &mut patch,
                &root,
                "RemoveAttackCooldown",
                "RemoveAttackCooldown",
                |value, path| parse_bool(value, path).map(SettingChange::RemoveAttackCooldown),
            );
            leaf(
                &mut patch,
                &root,
                "CancelAttackOnMiss",
                "CancelAttackOnMiss",
                |value, path| parse_bool(value, path).map(SettingChange::CancelAttackOnMiss),
            );
        }
        ModuleKind::NoSwing => {
            leaf(&mut patch, &root, "HideFor", "HideFor", |value, path| {
                liquidbounce_behaviors::no_swing::parse_no_swing_hide_for(value)
                    .map(SettingChange::NoSwingHideFor)
                    .map_err(|message| ConfigError::new(path, message))
            });
        }
        ModuleKind::NoPitchLimit => leaf(
            &mut patch,
            &root,
            "ServerSide",
            "ServerSide",
            |value, path| parse_bool(value, path).map(SettingChange::ServerSide),
        ),
        ModuleKind::NoInterpolation => {
            leaf(&mut patch, &root, "ReduceBy", "ReduceBy", |value, path| {
                parse_int(value, path).map(SettingChange::ReduceInterpolationBy)
            })
        }
        ModuleKind::NoSlowBreak => leaf(&mut patch, &root, "When", "When", |value, path| {
            liquidbounce_behaviors::no_slow_break::NoSlowBreak::from_json(value)
                .map(SettingChange::MiningConditions)
                .map_err(|error| ConfigError::new(path, error))
        }),
        ModuleKind::Sprint => {
            leaf(&mut patch, &root, "Mode", "Mode", |value, path| {
                use liquidbounce_behaviors::sprint::SprintMode;
                let tag = scalar_string(value)
                    .ok_or_else(|| ConfigError::new(path, "expected enum tag"))?;
                [
                    ("Legit", SprintMode::Legit),
                    ("Omnidirectional", SprintMode::Omnidirectional),
                    ("Omnirotational", SprintMode::Omnirotational),
                ]
                .into_iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(&tag))
                .map(|(_, mode)| SettingChange::SprintMode(mode))
                .ok_or_else(|| ConfigError::new(path, "unknown Sprint mode"))
            });
            leaf(&mut patch, &root, "Ignore", "Ignore", |value, path| {
                parse_multi_mask(value, path, &["Blindness", "Hunger", "Collision"])
                    .map(SettingChange::SprintIgnore)
            });
            leaf(&mut patch, &root, "StopOn", "StopOn", |value, path| {
                parse_multi_mask(value, path, &["Ground", "Air", "Sneaking", "UsingItem"])
                    .map(SettingChange::SprintStopOn)
            });
        }
        ModuleKind::CrystalView => {
            leaf(&mut patch, &root, "Size", "Size", |value, path| {
                parse_float(value, path).map(SettingChange::CrystalSize)
            });
            leaf(
                &mut patch,
                &root,
                "YTranslate",
                "YTranslate",
                |value, path| parse_float(value, path).map(SettingChange::CrystalY),
            );
            leaf(
                &mut patch,
                &root,
                "SpinSpeed",
                "SpinSpeed",
                |value, path| parse_float(value, path).map(SettingChange::CrystalSpin),
            );
            leaf(&mut patch, &root, "Bounce", "Bounce", |value, path| {
                parse_float(value, path).map(SettingChange::CrystalBounce)
            });
        }
        ModuleKind::FastPlace => {
            leaf(&mut patch, &root, "Cooldown", "Cooldown", |value, path| {
                parse_int_pair(value, path).map(SettingChange::PlaceCooldown)
            });
            leaf(&mut patch, &root, "ApplyTo", "ApplyTo", |value, path| {
                parse_multi_mask(value, path, &["Projectiles", "Blocks"])
                    .map(SettingChange::PlaceApplyTo)
            });
            leaf(
                &mut patch,
                &root,
                "StartDelay",
                "StartDelay",
                |value, path| parse_int(value, path).map(SettingChange::PlaceStartDelay),
            );
        }
        ModuleKind::AutoRespawn => leaf(&mut patch, &root, "Delay", "Delay", |value, path| {
            parse_int(value, path).map(SettingChange::RespawnDelay)
        }),
        ModuleKind::KeepSprint => {
            leaf(&mut patch, &root, "Motion", "Motion", |value, path| {
                parse_float_pair(value, path).map(SettingChange::KeepMotion)
            });
            leaf(
                &mut patch,
                &root,
                "MotionWhenHurt",
                "MotionWhenHurt",
                |value, path| parse_float_pair(value, path).map(SettingChange::KeepMotionHurt),
            );
            leaf(&mut patch, &root, "HurtTime", "HurtTime", |value, path| {
                parse_int_pair(value, path).map(SettingChange::KeepHurtTime)
            });
            leaf(&mut patch, &root, "Chance", "Chance", |value, path| {
                parse_float(value, path).map(SettingChange::KeepChance)
            });
        }
        ModuleKind::Aspect => leaf(&mut patch, &root, "Ratio", "Ratio", |value, path| {
            parse_int(value, path).map(SettingChange::AspectRatio)
        }),
        ModuleKind::SmoothCamera => {
            leaf(
                &mut patch,
                &root,
                "EnableFirstPOV",
                "EnableFirstPOV",
                |value, path| parse_bool(value, path).map(SettingChange::SmoothFirstPov),
            );
            leaf(
                &mut patch,
                &root,
                "ResetOnPerspectiveChange",
                "ResetOnPerspectiveChange",
                |value, path| parse_bool(value, path).map(SettingChange::SmoothResetPerspective),
            );
            leaf(
                &mut patch,
                &root,
                "HorizontalFactor",
                "HorizontalFactor",
                |value, path| parse_float(value, path).map(SettingChange::SmoothHorizontal),
            );
            leaf(
                &mut patch,
                &root,
                "VerticalFactor",
                "VerticalFactor",
                |value, path| parse_float(value, path).map(SettingChange::SmoothVertical),
            );
        }
        ModuleKind::QuickPerspectiveSwap => {
            leaf(&mut patch, &root, "RearView", "RearView", |value, path| {
                parse_bool(value, path).map(SettingChange::RearView)
            })
        }
        ModuleKind::AirJump => leaf(&mut patch, &root, "Mode", "Mode", |value, path| {
            use liquidbounce_behaviors::air_jump::AirJumpMode;
            let tag =
                scalar_string(value).ok_or_else(|| ConfigError::new(path, "expected enum tag"))?;
            [
                ("JumpFreely", AirJumpMode::JumpFreely),
                ("DoubleJump", AirJumpMode::DoubleJump),
                ("GhostBlock", AirJumpMode::GhostBlock),
            ]
            .into_iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(&tag))
            .map(|(_, mode)| SettingChange::AirJumpMode(mode))
            .ok_or_else(|| ConfigError::new(path, "unknown AirJump mode"))
        }),
        ModuleKind::NoPush => leaf(&mut patch, &root, "PushBy", "PushBy", |value, path| {
            parse_multi_mask(
                value,
                path,
                &["Entities", "Blocks", "FishingRod", "Liquids", "Sinking"],
            )
            .map(SettingChange::PushBy)
        }),
        ModuleKind::AntiBlind => {
            leaf(&mut patch, &root, "DoRender", "DoRender", |value, path| {
                parse_multi_mask(
                    value,
                    path,
                    &liquidbounce_behaviors::visual_environment::DoRender::TAGS,
                )
                .map(SettingChange::RenderMask)
            });
            leaf(
                &mut patch,
                &root,
                "FireOpacity",
                "FireOpacity",
                |value, path| parse_int(value, path).map(SettingChange::FireOpacity),
            );
            leaf(
                &mut patch,
                &root,
                "NoParticles",
                "NoParticles",
                |value, path| parse_bool(value, path).map(SettingChange::NoParticles),
            );
            leaf(
                &mut patch,
                &root,
                "SkipTerrainLoading",
                "SkipTerrainLoading",
                |value, path| parse_bool(value, path).map(SettingChange::SkipTerrainLoading),
            );
        }
        _ => {}
    }
    Ok(patch)
}

fn parse_mode(patch: &mut SettingsPatch, mode: &Value) {
    let active = mode.get("active").and_then(scalar_string);
    let change = match (patch.module, active.as_deref()) {
        (ModuleKind::NoFov, Some("Constant")) => {
            Some(SettingChange::NoFovMode(NoFovMode::Constant))
        }
        (ModuleKind::NoFov, Some("Custom")) => Some(SettingChange::NoFovMode(NoFovMode::Custom)),
        (ModuleKind::FullBright, Some("Gamma")) => {
            Some(SettingChange::FullBrightMode(FullBrightMode::Gamma))
        }
        (ModuleKind::FullBright, Some("NightVision")) => {
            Some(SettingChange::FullBrightMode(FullBrightMode::NightVision))
        }
        _ => None,
    };
    if let Some(change) = change {
        patch.changes.push(change);
    } else {
        patch.issues.push(ConfigError::new(
            "Mode.active",
            "missing or unknown case-sensitive mode",
        ));
    }
    // Original tries active first, even if choices or group name are malformed.
    let Some(choices) = mode.get("choices").and_then(Value::as_object) else {
        patch
            .issues
            .push(ConfigError::new("Mode.choices", "expected choices object"));
        return;
    };
    let names: &[&str] = if patch.module == ModuleKind::NoFov {
        &["Constant", "Custom"]
    } else {
        &["Gamma", "NightVision"]
    };
    for name in names {
        let path = format!("Mode.{name}");
        let Some(choice) = choices.get(*name) else {
            patch
                .issues
                .push(ConfigError::new(&path, "choice not found"));
            continue;
        };
        let choice = match nodes(choice, name, &path) {
            Ok(choice) => choice,
            Err(error) => {
                patch.issues.push(error);
                continue;
            }
        };
        match *name {
            "Constant" => leaf(patch, &choice, "FOV", "Mode.Constant.FOV", |value, path| {
                parse_int(value, path).map(SettingChange::ConstantFov)
            }),
            "Custom" => {
                leaf(
                    patch,
                    &choice,
                    "BaseFOV",
                    "Mode.Custom.BaseFOV",
                    |value, path| parse_float(value, path).map(SettingChange::CustomBaseFov),
                );
                leaf(
                    patch,
                    &choice,
                    "Limit",
                    "Mode.Custom.Limit",
                    |value, path| {
                        let object = value
                            .as_object()
                            .ok_or_else(|| ConfigError::new(path, "expected {from,to} range"))?;
                        let from = object
                            .get("from")
                            .ok_or_else(|| ConfigError::new(path, "missing range from"))?;
                        let to = object
                            .get("to")
                            .ok_or_else(|| ConfigError::new(path, "missing range to"))?;
                        Ok(SettingChange::CustomLimit(
                            parse_float(from, path)?,
                            parse_float(to, path)?,
                        ))
                    },
                );
                leaf(
                    patch,
                    &choice,
                    "Multiplier",
                    "Mode.Custom.Multiplier",
                    |value, path| parse_float(value, path).map(SettingChange::CustomMultiplier),
                );
            }
            "Gamma" => leaf(
                patch,
                &choice,
                "Brightness",
                "Mode.Gamma.Brightness",
                |value, path| parse_int(value, path).map(SettingChange::Brightness),
            ),
            _ => {}
        }
    }
    if let Err(error) = nodes(mode, "Mode", "Mode") {
        patch.issues.push(error);
    }
}

fn replace<T: PartialEq>(place: &mut T, value: T) -> bool {
    if *place == value {
        false
    } else {
        *place = value;
        true
    }
}

fn float_equal(a: f32, b: f32) -> bool {
    // Value<T>.set uses boxed Float.equals: all NaNs equal, signed zeros differ.
    (a.is_nan() && b.is_nan()) || a.to_bits() == b.to_bits()
}

fn replace_float(place: &mut f32, value: f32) -> bool {
    if float_equal(*place, value) {
        false
    } else {
        *place = value;
        true
    }
}

fn range_equal(a: (f32, f32), b: (f32, f32)) -> bool {
    let empty = |range: (f32, f32)| range.0.partial_cmp(&range.1).is_none() || range.0 > range.1;
    (empty(a) && empty(b)) || (a.0 == b.0 && a.1 == b.1)
}

pub fn apply_setting(
    change: &SettingChange,
    no_fov: &mut NoFov,
    zoom: &mut Zoom,
    full_bright: &mut FullBright,
) -> ApplyResult {
    let changed = match change {
        SettingChange::Enabled(value) => {
            return ApplyResult::Runtime(RuntimeRequest::Enabled(*value));
        }
        SettingChange::Bind(value) => {
            return ApplyResult::Runtime(RuntimeRequest::Bind(value.clone()));
        }
        SettingChange::Hidden(value) => {
            return ApplyResult::Runtime(RuntimeRequest::Hidden(*value));
        }
        SettingChange::RemoveAttackCooldown(value) => {
            return ApplyResult::Runtime(RuntimeRequest::RemoveAttackCooldown(*value));
        }
        SettingChange::CancelAttackOnMiss(value) => {
            return ApplyResult::Runtime(RuntimeRequest::CancelAttackOnMiss(*value));
        }
        SettingChange::NoSwingHideFor(value) => {
            return ApplyResult::Runtime(RuntimeRequest::NoSwingHideFor(*value));
        }
        SettingChange::ServerSide(value) => {
            return ApplyResult::Runtime(RuntimeRequest::ServerSide(*value));
        }
        SettingChange::ReduceInterpolationBy(value) => {
            return ApplyResult::Runtime(RuntimeRequest::ReduceInterpolationBy(*value));
        }
        SettingChange::MiningConditions(value) => {
            return ApplyResult::Runtime(RuntimeRequest::MiningConditions(*value));
        }
        SettingChange::SprintMode(value) => {
            return ApplyResult::Runtime(RuntimeRequest::SprintMode(*value));
        }
        SettingChange::SprintIgnore(value) => {
            return ApplyResult::Runtime(RuntimeRequest::SprintIgnore(*value));
        }
        SettingChange::SprintStopOn(value) => {
            return ApplyResult::Runtime(RuntimeRequest::SprintStopOn(*value));
        }
        SettingChange::RenderMask(value) => {
            return ApplyResult::Runtime(RuntimeRequest::RenderMask(*value));
        }
        SettingChange::FireOpacity(value) => {
            return ApplyResult::Runtime(RuntimeRequest::FireOpacity(*value));
        }
        SettingChange::NoParticles(value) => {
            return ApplyResult::Runtime(RuntimeRequest::NoParticles(*value));
        }
        SettingChange::SkipTerrainLoading(value) => {
            return ApplyResult::Runtime(RuntimeRequest::SkipTerrainLoading(*value));
        }
        SettingChange::AirJumpMode(value) => {
            return ApplyResult::Runtime(RuntimeRequest::AirJumpMode(*value));
        }
        SettingChange::PushBy(value) => {
            return ApplyResult::Runtime(RuntimeRequest::PushBy(*value));
        }
        SettingChange::CrystalSize(value) => {
            return ApplyResult::Runtime(RuntimeRequest::CrystalSize(*value));
        }
        SettingChange::CrystalY(value) => {
            return ApplyResult::Runtime(RuntimeRequest::CrystalY(*value));
        }
        SettingChange::CrystalSpin(value) => {
            return ApplyResult::Runtime(RuntimeRequest::CrystalSpin(*value));
        }
        SettingChange::CrystalBounce(value) => {
            return ApplyResult::Runtime(RuntimeRequest::CrystalBounce(*value));
        }
        SettingChange::AutoF5(value) => {
            return ApplyResult::Runtime(RuntimeRequest::AutoF5(value.clone()));
        }
        SettingChange::NoPose(value) => {
            return ApplyResult::Runtime(RuntimeRequest::NoPose(*value));
        }
        SettingChange::NoRotateSet(value) => {
            return ApplyResult::Runtime(RuntimeRequest::NoRotateSet(*value));
        }
        SettingChange::CameraClip(value) => {
            return ApplyResult::Runtime(RuntimeRequest::CameraClip(value.clone()));
        }
        SettingChange::DankBobbing(value) => {
            return ApplyResult::Runtime(RuntimeRequest::DankBobbing(*value));
        }
        SettingChange::MultiActions(value) => {
            return ApplyResult::Runtime(RuntimeRequest::MultiActions(value.clone()));
        }
        SettingChange::Twerk(value) => {
            return ApplyResult::Runtime(RuntimeRequest::Twerk(*value));
        }
        SettingChange::Hitbox(value) => {
            return ApplyResult::Runtime(RuntimeRequest::Hitbox(*value));
        }
        SettingChange::AutoWeapon(value) => {
            return ApplyResult::Runtime(RuntimeRequest::AutoWeapon(value.clone()));
        }
        SettingChange::Chams(value) => {
            return ApplyResult::Runtime(RuntimeRequest::Chams(value.clone()));
        }
        SettingChange::AutoTool(value) => {
            return ApplyResult::Runtime(RuntimeRequest::AutoTool(value.clone()));
        }
        SettingChange::AntiAfk(value) => {
            return ApplyResult::Runtime(RuntimeRequest::AntiAfk(value.clone()));
        }
        SettingChange::TridentBoost(value) => {
            return ApplyResult::Runtime(RuntimeRequest::TridentBoost(*value));
        }
        SettingChange::BlockBounce(value) => {
            return ApplyResult::Runtime(RuntimeRequest::BlockBounce(*value));
        }
        SettingChange::SmartEat(value) => {
            return ApplyResult::Runtime(RuntimeRequest::SmartEat(*value));
        }
        SettingChange::SkinDerp(value) => {
            return ApplyResult::Runtime(RuntimeRequest::SkinDerp(value.clone()));
        }
        SettingChange::ExtendedFirework(value) => {
            return ApplyResult::Runtime(RuntimeRequest::ExtendedFirework(*value));
        }
        SettingChange::Strafe(value) => {
            return ApplyResult::Runtime(RuntimeRequest::Strafe(*value));
        }
        SettingChange::Speed(value) => {
            return ApplyResult::Runtime(RuntimeRequest::Speed(value.clone()));
        }
        SettingChange::FastBreak(value) => {
            return ApplyResult::Runtime(RuntimeRequest::FastBreak(*value));
        }
        SettingChange::Timer(value) => {
            return ApplyResult::Runtime(RuntimeRequest::Timer(*value));
        }
        SettingChange::BetterInventory(value) => {
            return ApplyResult::Runtime(RuntimeRequest::BetterInventory(*value));
        }
        SettingChange::TerrainSpeed(value) => {
            return ApplyResult::Runtime(RuntimeRequest::TerrainSpeed(*value));
        }
        SettingChange::ReverseStep(value) => { return ApplyResult::Runtime(RuntimeRequest::ReverseStep(*value)); }
        SettingChange::Step(value) => { return ApplyResult::Runtime(RuntimeRequest::Step(*value)); }
        SettingChange::LiquidWalk(value) => { return ApplyResult::Runtime(RuntimeRequest::LiquidWalk(*value)); }
        SettingChange::NoWeb(value) => { return ApplyResult::Runtime(RuntimeRequest::NoWeb(value.clone())); }
        SettingChange::NoSlow(value) => { return ApplyResult::Runtime(RuntimeRequest::NoSlow(*value)); }
        SettingChange::HighJump(value) => {
            return ApplyResult::Runtime(RuntimeRequest::HighJump(*value));
        }
        SettingChange::AutoWalk(value) => {
            return ApplyResult::Runtime(RuntimeRequest::AutoWalk(value.clone()));
        }
        SettingChange::Parkour(value) => { return ApplyResult::Runtime(RuntimeRequest::Parkour(value.clone())); }
        SettingChange::SafeWalk(value) => { return ApplyResult::Runtime(RuntimeRequest::SafeWalk(value.clone())); }
        SettingChange::Anchor(value) => { return ApplyResult::Runtime(RuntimeRequest::Anchor(*value)); }
        SettingChange::AvoidHazards(value) => { return ApplyResult::Runtime(RuntimeRequest::AvoidHazards(*value)); }
        SettingChange::Freeze(value) => { return ApplyResult::Runtime(RuntimeRequest::Freeze(*value)); }
        SettingChange::OneShot(value) => {
            return ApplyResult::Runtime(RuntimeRequest::OneShot(*value));
        }
        SettingChange::TargetLock(value) => {
            return ApplyResult::Runtime(RuntimeRequest::TargetLock(value.clone()));
        }
        SettingChange::YggdrasilSignatureFix(value) => {
            return ApplyResult::Runtime(RuntimeRequest::YggdrasilSignatureFix(*value));
        }
        SettingChange::AutoDisable(value) => {
            return ApplyResult::Runtime(RuntimeRequest::AutoDisable(value.clone()));
        }
        SettingChange::Disabler(value) => return ApplyResult::Runtime(RuntimeRequest::Disabler(*value)),
        SettingChange::ServerCrasher(value) => return ApplyResult::Runtime(RuntimeRequest::ServerCrasher(*value)),
        SettingChange::Dupe(value) => return ApplyResult::Runtime(RuntimeRequest::Dupe(*value)),
        SettingChange::TeleportReach(value) => return ApplyResult::Runtime(RuntimeRequest::TeleportReach(*value)),
        SettingChange::Phase(value) => return ApplyResult::Runtime(RuntimeRequest::Phase(*value)),
        SettingChange::Notebot(value) => return ApplyResult::Runtime(RuntimeRequest::Notebot(*value)),
        SettingChange::AutoConfig(value) => return ApplyResult::Runtime(RuntimeRequest::AutoConfig(*value)),
        SettingChange::DebugRecorder(value) => return ApplyResult::Runtime(RuntimeRequest::DebugRecorder(*value)),
        SettingChange::SkinChanger(value) => return ApplyResult::Runtime(RuntimeRequest::SkinChanger(*value)),
        SettingChange::PotionSpoof(value) => {
            return ApplyResult::Runtime(RuntimeRequest::PotionSpoof(*value));
        }
        SettingChange::NoClip(value) => {
            return ApplyResult::Runtime(RuntimeRequest::NoClip(*value));
        }
        SettingChange::TrueSight(value) => {
            return ApplyResult::Runtime(RuntimeRequest::TrueSight(*value));
        }
        SettingChange::Plugins(value) => {
            return ApplyResult::Runtime(RuntimeRequest::Plugins(*value));
        }
        SettingChange::Sneak(value) => {return ApplyResult::Runtime(RuntimeRequest::Sneak(*value));}
        SettingChange::AntiExploit(value) => {return ApplyResult::Runtime(RuntimeRequest::AntiExploit(*value));}
        SettingChange::Eagle(value) => {return ApplyResult::Runtime(RuntimeRequest::Eagle(*value));}
        SettingChange::ChestStealer(value)=>{return ApplyResult::Runtime(RuntimeRequest::ChestStealer(value.clone()));}
        SettingChange::AutoArmor(value)=>{return ApplyResult::Runtime(RuntimeRequest::AutoArmor(value.clone()));}
        SettingChange::AutoMobHeal(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoMobHeal(value.clone())),
        SettingChange::InventoryCleaner(value)=>{return ApplyResult::Runtime(RuntimeRequest::InventoryCleaner(value.clone()));}
        SettingChange::Replenish(value)=>{return ApplyResult::Runtime(RuntimeRequest::Replenish(value.clone()));}
        SettingChange::AutoCrafter(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoCrafter(value.clone())),
        SettingChange::AutoDeposit(value)=>{return ApplyResult::Runtime(RuntimeRequest::AutoDeposit(value.clone()));}
        SettingChange::ChestCleaner(value) => {return ApplyResult::Runtime(RuntimeRequest::ChestCleaner(value.clone()));}
        SettingChange::BookBot(value) => {return ApplyResult::Runtime(RuntimeRequest::BookBot(value.clone()));}
        SettingChange::Clip(value) => {return ApplyResult::Runtime(RuntimeRequest::Clip(*value));}
        SettingChange::PingSpoof(value) => { return ApplyResult::Runtime(RuntimeRequest::PingSpoof(*value)); }
        SettingChange::AutoChatGame(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoChatGame(value.clone())),
        SettingChange::Spammer(value) => { return ApplyResult::Runtime(RuntimeRequest::Spammer(value.clone())); }
        SettingChange::Offhand(value)=>return ApplyResult::Runtime(RuntimeRequest::Offhand(value.clone())),
        SettingChange::ProjectilePuncher(value)=>return ApplyResult::Runtime(RuntimeRequest::ProjectilePuncher(value.clone())),
        SettingChange::FreeCam(value)=>return ApplyResult::Runtime(RuntimeRequest::FreeCam(value.clone())),
        SettingChange::MurderMystery(value)=>return ApplyResult::Runtime(RuntimeRequest::MurderMystery(value.clone())),
        SettingChange::Teams(value) => { return ApplyResult::Runtime(RuntimeRequest::Teams(value.clone())); }
        SettingChange::InventoryMove(value)=>return ApplyResult::Runtime(RuntimeRequest::InventoryMove(value.clone())),
        SettingChange::XRay(value)=>return ApplyResult::Runtime(RuntimeRequest::XRay(value.clone())),
        SettingChange::AntiVoid(value)=>return ApplyResult::Runtime(RuntimeRequest::AntiVoid(value.clone())),
        SettingChange::AntiBot(value)=>return ApplyResult::Runtime(RuntimeRequest::AntiBot(value.clone())),
        SettingChange::Notifier(value)=>return ApplyResult::Runtime(RuntimeRequest::Notifier(value.clone())),
        SettingChange::AntiStaff(value)=>return ApplyResult::Runtime(RuntimeRequest::AntiStaff(*value)),
        SettingChange::PacketLogger(value)=>return ApplyResult::Runtime(RuntimeRequest::PacketLogger(value.clone())),
        SettingChange::FlagCheck(value)=>return ApplyResult::Runtime(RuntimeRequest::FlagCheck(value.clone())),
        SettingChange::AirPlace(value)=>return ApplyResult::Runtime(RuntimeRequest::AirPlace(value.clone())),
        SettingChange::AutoDodge(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoDodge(value.clone())),
        SettingChange::Blink(value)=>return ApplyResult::Runtime(RuntimeRequest::Blink(value.clone())),
        SettingChange::FastExp(value)=>return ApplyResult::Runtime(RuntimeRequest::FastExp(value.clone())),
        SettingChange::AutoWindCharge(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoWindCharge(value.clone())),
        SettingChange::AutoBuff(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoBuff(value.clone())),
        SettingChange::AutoBow(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoBow(value.clone())),
        SettingChange::AutoPearl(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoPearl(value.clone())),
        SettingChange::ElytraFly(value) => return ApplyResult::Runtime(RuntimeRequest::ElytraFly(*value)),
        SettingChange::NoFall(value) => return ApplyResult::Runtime(RuntimeRequest::NoFall(*value)),
        SettingChange::Extinguish(value)=>{return ApplyResult::Runtime(RuntimeRequest::Extinguish(value.clone()));}
        SettingChange::LiquidFiller(value)=>return ApplyResult::Runtime(RuntimeRequest::LiquidFiller(value.clone())),
        SettingChange::HoleFiller(value)=>return ApplyResult::Runtime(RuntimeRequest::HoleFiller(value.clone())),
        SettingChange::AutoShop(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoShop(*value)),
        SettingChange::AutoBuild(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoBuild(*value)),
        SettingChange::AutoFarm(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoFarm(*value)),
        SettingChange::CrystalAura(value)=>return ApplyResult::Runtime(RuntimeRequest::CrystalAura(*value)),
        SettingChange::Fucker(value)=>return ApplyResult::Runtime(RuntimeRequest::Fucker(*value)),
        SettingChange::AutoTrap(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoTrap(*value)),
        SettingChange::BlockTrap(value)=>return ApplyResult::Runtime(RuntimeRequest::BlockTrap(*value)),
        SettingChange::Nuker(value)=>return ApplyResult::Runtime(RuntimeRequest::Nuker(*value)),
        SettingChange::TickBase(value)=>return ApplyResult::Runtime(RuntimeRequest::TickBase(*value)),
        SettingChange::Backtrack(value)=>return ApplyResult::Runtime(RuntimeRequest::Backtrack(*value)),
        SettingChange::Fly(value)=>return ApplyResult::Runtime(RuntimeRequest::Fly(*value)),
        SettingChange::LongJump(value)=>return ApplyResult::Runtime(RuntimeRequest::LongJump(*value)),
        SettingChange::Surround(value)=>return ApplyResult::Runtime(RuntimeRequest::Surround(value.clone())),
        SettingChange::BlockIn(value)=>return ApplyResult::Runtime(RuntimeRequest::BlockIn(value.clone())),
        SettingChange::BedDefender(value)=>return ApplyResult::Runtime(RuntimeRequest::BedDefender(value.clone())),
        SettingChange::EasyPearl(value)=>return ApplyResult::Runtime(RuntimeRequest::EasyPearl(value.clone())),
        SettingChange::ElytraTarget(value)=>return ApplyResult::Runtime(RuntimeRequest::ElytraTarget(value.clone())),
        SettingChange::AutoRod(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoRod(value.clone())),
        SettingChange::AutoShoot(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoShoot(value.clone())),
        SettingChange::Scaffold(value)=>return ApplyResult::Runtime(RuntimeRequest::Scaffold(value.clone())),
        SettingChange::Clutch(value)=>return ApplyResult::Runtime(RuntimeRequest::Clutch(value.clone())),
        SettingChange::FakeLag(value)=>return ApplyResult::Runtime(RuntimeRequest::FakeLag(value.clone())),
        SettingChange::SwordBlock(value)=>return ApplyResult::Runtime(RuntimeRequest::SwordBlock(value.clone())),
        SettingChange::Aimbot(value)=>return ApplyResult::Runtime(RuntimeRequest::Aimbot(value.clone())),
        SettingChange::TargetStrafe(value)=>return ApplyResult::Runtime(RuntimeRequest::TargetStrafe(value.clone())),
        SettingChange::KillAura(value)=>return ApplyResult::Runtime(RuntimeRequest::KillAura(value.clone())),
        SettingChange::TpAura(value) => return ApplyResult::Runtime(RuntimeRequest::TpAura(*value)),
        SettingChange::Velocity(value) => return ApplyResult::Runtime(RuntimeRequest::Velocity(*value)),
        SettingChange::PacketMine(value)=>return ApplyResult::Runtime(RuntimeRequest::PacketMine(value.clone())),
        SettingChange::Hud(value)=>return ApplyResult::Runtime(RuntimeRequest::Hud(value.clone())),
        SettingChange::FastUse(value) => { return ApplyResult::Runtime(RuntimeRequest::FastUse(*value)); }
        SettingChange::OldRegen(value) => {
            return ApplyResult::Runtime(RuntimeRequest::OldRegen(*value));
        }
        SettingChange::Spider(value) => {
            return ApplyResult::Runtime(RuntimeRequest::Spider(*value));
        }
        SettingChange::Criticals(value)=>return ApplyResult::Runtime(RuntimeRequest::Criticals(value.clone())),
        SettingChange::CrosshairClicker(value) => {
            return ApplyResult::Runtime(RuntimeRequest::CrosshairClicker(value.clone()));
        }
        SettingChange::SpearKill(value) => {
            return ApplyResult::Runtime(RuntimeRequest::SpearKill(*value));
        }
        SettingChange::MaceKill(value) => {
            return ApplyResult::Runtime(RuntimeRequest::MaceKill(*value));
        }
        SettingChange::SuperKnockback(value) => {
            return ApplyResult::Runtime(RuntimeRequest::SuperKnockback(*value));
        }
        SettingChange::MiddleClickAction(value) => {
            return ApplyResult::Runtime(RuntimeRequest::MiddleClickAction(*value));
        }
        SettingChange::ReportHelper(value)=>return ApplyResult::Runtime(RuntimeRequest::ReportHelper(value.clone())),
        SettingChange::AutoQueue(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoQueue(value.clone())),
        SettingChange::Macros(value)=>return ApplyResult::Runtime(RuntimeRequest::Macros(value.clone())),
        SettingChange::AutoAccount(value) => {
            return ApplyResult::Runtime(RuntimeRequest::AutoAccount(value.clone()));
        }
        SettingChange::MobOwners(value) => {
            return ApplyResult::Runtime(RuntimeRequest::MobOwners(*value));
        }
        SettingChange::CombineMobs(value) => {
            return ApplyResult::Runtime(RuntimeRequest::CombineMobs(*value));
        }
        SettingChange::BetterTab(value) => {
            return ApplyResult::Runtime(RuntimeRequest::BetterTab(value.clone()));
        }
        SettingChange::BlockOutline(value) => { return ApplyResult::Runtime(RuntimeRequest::BlockOutline(*value)); }
        SettingChange::Breadcrumbs(value) => { return ApplyResult::Runtime(RuntimeRequest::Breadcrumbs(value.clone())); }
        SettingChange::DamageParticles(value) => { return ApplyResult::Runtime(RuntimeRequest::DamageParticles(*value)); }
        SettingChange::Esp(value)=>return ApplyResult::Runtime(RuntimeRequest::Esp(value.clone())),
        SettingChange::Tracers(value) => { return ApplyResult::Runtime(RuntimeRequest::Tracers(value.clone())); }
        SettingChange::Radar(value)=>return ApplyResult::Runtime(RuntimeRequest::Radar(value.clone())),
        SettingChange::ItemEsp(value)=>return ApplyResult::Runtime(RuntimeRequest::ItemEsp(value.clone())),
        SettingChange::ItemChams(value)=>return ApplyResult::Runtime(RuntimeRequest::ItemChams(value.clone())),
        SettingChange::ItemTags(v)=>return ApplyResult::Runtime(RuntimeRequest::ItemTags(v.clone())),
        SettingChange::Nametags(v)=>return ApplyResult::Runtime(RuntimeRequest::Nametags(v.clone())),
        SettingChange::BetterChat(v)=>return ApplyResult::Runtime(RuntimeRequest::BetterChat(v.clone())),
        SettingChange::Debug(v)=>return ApplyResult::Runtime(RuntimeRequest::Debug(v.clone())),
        SettingChange::ClickGui(v)=>return ApplyResult::Runtime(RuntimeRequest::ClickGui(v.clone())),
        SettingChange::BedPlates(v)=>return ApplyResult::Runtime(RuntimeRequest::BedPlates(v.clone())),
        SettingChange::NewChunks(value) => { return ApplyResult::Runtime(RuntimeRequest::NewChunks(*value)); }
        SettingChange::StrongholdFinder(value)=>return ApplyResult::Runtime(RuntimeRequest::StrongholdFinder(value.clone())),
        SettingChange::ProphuntEsp(value) => { return ApplyResult::Runtime(RuntimeRequest::ProphuntEsp(*value)); }
        SettingChange::VoidEsp(value) => { return ApplyResult::Runtime(RuntimeRequest::VoidEsp(*value)); }
        SettingChange::ProtectionZones(value) => { return ApplyResult::Runtime(RuntimeRequest::ProtectionZones(value.clone())); }
        SettingChange::HoleEsp(value) => { return ApplyResult::Runtime(RuntimeRequest::HoleEsp(value.clone())); }
        SettingChange::JumpEffect(value)=>return ApplyResult::Runtime(RuntimeRequest::JumpEffect(value.clone())),
        SettingChange::HitFx(value)=>return ApplyResult::Runtime(RuntimeRequest::HitFx(value.clone())),
        SettingChange::TotemEffect(value)=>return ApplyResult::Runtime(RuntimeRequest::TotemEffect(value.clone())),
        SettingChange::PotionFX(value)=>return ApplyResult::Runtime(RuntimeRequest::PotionFX(value.clone())),
        SettingChange::TNTTimer(value)=>return ApplyResult::Runtime(RuntimeRequest::TNTTimer(value.clone())),
        SettingChange::CustomAmbience(value)=>return ApplyResult::Runtime(RuntimeRequest::CustomAmbience(value.clone())),
        SettingChange::Trajectories(value)=>return ApplyResult::Runtime(RuntimeRequest::Trajectories(value.clone())),
        SettingChange::Particles(value) => { return ApplyResult::Runtime(RuntimeRequest::Particles(value.clone())); }
        SettingChange::NameProtect(value) => { return ApplyResult::Runtime(RuntimeRequest::NameProtect(value.clone())); }
        SettingChange::InventoryTracker(value) => { return ApplyResult::Runtime(RuntimeRequest::InventoryTracker(*value)); }
        SettingChange::LogoffSpot(value) => { return ApplyResult::Runtime(RuntimeRequest::LogoffSpot(*value)); }
        SettingChange::StorageEsp(value)=>{return ApplyResult::Runtime(RuntimeRequest::StorageEsp(value.clone()));}
        SettingChange::BlockEsp(value)=>{return ApplyResult::Runtime(RuntimeRequest::BlockEsp(value.clone()));}
        SettingChange::Crosshair(value) => { return ApplyResult::Runtime(RuntimeRequest::Crosshair(value.clone())); }
        SettingChange::Wings(value) => { return ApplyResult::Runtime(RuntimeRequest::Wings(value.clone())); }
        SettingChange::Hats(value) => { return ApplyResult::Runtime(RuntimeRequest::Hats(value.clone())); }
        SettingChange::Damage(value) => {
            return ApplyResult::Runtime(RuntimeRequest::Damage(*value));
        }
        SettingChange::ResetVl(value) => {
            return ApplyResult::Runtime(RuntimeRequest::ResetVl(*value));
        }
        SettingChange::ItemScroller(value) => {
            return ApplyResult::Runtime(RuntimeRequest::ItemScroller(*value));
        }
        SettingChange::GuiCloser(value) => {
            return ApplyResult::Runtime(RuntimeRequest::GuiCloser(value.clone()));
        }
        SettingChange::TextFieldProtect(value) => {
            return ApplyResult::Runtime(RuntimeRequest::TextFieldProtect(value.clone()));
        }
        SettingChange::AntiHunger(value) => {
            return ApplyResult::Runtime(RuntimeRequest::AntiHunger(*value));
        }
        SettingChange::Vomit(value) => {
            return ApplyResult::Runtime(RuntimeRequest::Vomit(value.clone()));
        }
        SettingChange::ElytraSwap(value) => {
            return ApplyResult::Runtime(RuntimeRequest::ElytraSwap(value.clone()));
        }
        SettingChange::GhostHand(value) => {
            return ApplyResult::Runtime(RuntimeRequest::GhostHand(value.clone()));
        }
        SettingChange::BlockWalk(value) => {
            return ApplyResult::Runtime(RuntimeRequest::BlockWalk(value.clone()));
        }
        SettingChange::EntityControl(value) => {
            return ApplyResult::Runtime(RuntimeRequest::EntityControl(value.clone()));
        }
        SettingChange::VehicleControl(value)=>return ApplyResult::Runtime(RuntimeRequest::VehicleControl(value.clone())),
        SettingChange::VehicleBoost(value) => {
            return ApplyResult::Runtime(RuntimeRequest::VehicleBoost(*value));
        }
        SettingChange::HandDerp(value) => {
            return ApplyResult::Runtime(RuntimeRequest::HandDerp(*value));
        }
        SettingChange::FreeLook(value) => {
            return ApplyResult::Runtime(RuntimeRequest::FreeLook(*value));
        }
        SettingChange::Derp(value) => {
            return ApplyResult::Runtime(RuntimeRequest::Derp(value.clone()));
        }
        SettingChange::NoCooldownProgress(value) => {
            return ApplyResult::Runtime(RuntimeRequest::NoCooldownProgress(*value));
        }
        SettingChange::NoEntityInteract(value) => {
            return ApplyResult::Runtime(RuntimeRequest::NoEntityInteract(value.clone()));
        }
        SettingChange::Reach(value) => {
            return ApplyResult::Runtime(RuntimeRequest::Reach(*value));
        }
        SettingChange::AutoFish(value) => {
            return ApplyResult::Runtime(RuntimeRequest::AutoFish(value.clone()));
        }
        SettingChange::PlaceCooldown(value) => {
            return ApplyResult::Runtime(RuntimeRequest::PlaceCooldown(*value));
        }
        SettingChange::PlaceApplyTo(value) => {
            return ApplyResult::Runtime(RuntimeRequest::PlaceApplyTo(*value));
        }
        SettingChange::PlaceStartDelay(value) => {
            return ApplyResult::Runtime(RuntimeRequest::PlaceStartDelay(*value));
        }
        SettingChange::RespawnDelay(value) => {
            return ApplyResult::Runtime(RuntimeRequest::RespawnDelay(*value));
        }
        SettingChange::KeepMotion(value) => {
            return ApplyResult::Runtime(RuntimeRequest::KeepMotion(*value));
        }
        SettingChange::KeepMotionHurt(value) => {
            return ApplyResult::Runtime(RuntimeRequest::KeepMotionHurt(*value));
        }
        SettingChange::KeepHurtTime(value) => {
            return ApplyResult::Runtime(RuntimeRequest::KeepHurtTime(*value));
        }
        SettingChange::KeepChance(value) => {
            return ApplyResult::Runtime(RuntimeRequest::KeepChance(*value));
        }
        SettingChange::AspectRatio(value) => {
            return ApplyResult::Runtime(RuntimeRequest::AspectRatio(*value));
        }
        SettingChange::SmoothFirstPov(value) => {
            return ApplyResult::Runtime(RuntimeRequest::SmoothFirstPov(*value));
        }
        SettingChange::SmoothResetPerspective(value) => {
            return ApplyResult::Runtime(RuntimeRequest::SmoothResetPerspective(*value));
        }
        SettingChange::SmoothHorizontal(value) => {
            return ApplyResult::Runtime(RuntimeRequest::SmoothHorizontal(*value));
        }
        SettingChange::SmoothVertical(value) => {
            return ApplyResult::Runtime(RuntimeRequest::SmoothVertical(*value));
        }
        SettingChange::RearView(value) => {
            return ApplyResult::Runtime(RuntimeRequest::RearView(*value));
        }
        SettingChange::NoFovMode(value) => replace(&mut no_fov.mode, *value),
        SettingChange::ConstantFov(value) => replace(&mut no_fov.constant_fov, *value),
        SettingChange::CustomBaseFov(value) => replace_float(&mut no_fov.custom.base_fov, *value),
        SettingChange::CustomMultiplier(value) => {
            replace_float(&mut no_fov.custom.multiplier, *value)
        }
        SettingChange::CustomLimit(from, to) => {
            let next = (*from, *to);
            if range_equal(no_fov.custom.limit, next) {
                false
            } else {
                no_fov.custom.limit = next;
                true
            }
        }
        SettingChange::Zoom(value) => replace(&mut zoom.settings.zoom, *value),
        SettingChange::ScrollEnabled(value) => replace(&mut zoom.settings.scroll_enabled, *value),
        SettingChange::ScrollSpeed(value) => replace_float(&mut zoom.settings.scroll_speed, *value),
        SettingChange::Transition(value) => replace(&mut zoom.settings.transition, *value),
        SettingChange::DurationFactor(value) => {
            replace_float(&mut zoom.settings.duration_factor, *value)
        }
        SettingChange::FullBrightMode(value) => {
            if full_bright.mode() == *value {
                false
            } else {
                return ApplyResult::Runtime(RuntimeRequest::FullBrightMode(*value));
            }
        }
        SettingChange::Brightness(value) => replace(&mut full_bright.gamma.brightness, *value),
    };
    ApplyResult::Changed(changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn common() -> CommonSettings {
        CommonSettings {
            enabled: false,
            hidden: false,
            key_bind: json!({"boundKey":"key.keyboard.unknown","action":"Hold"}),
        }
    }
    fn describe(key: &str) -> String {
        key.to_owned()
    }

    #[test]
    fn camera_effects_have_only_original_common_settings_in_both_formats() {
        let common = CommonSettings {
            enabled: true,
            hidden: true,
            key_bind: json!({"boundKey":"key.keyboard.h","action":"Toggle"}),
        };
        for module in [ModuleKind::NoBob, ModuleKind::NoHurtCam] {
            let interop =
                serialize_camera_effect(module, &common, JsonFormat::Interop, &describe).unwrap();
            assert_eq!(interop["name"], module.name());
            assert_eq!(interop["valueType"], "TOGGLEABLE");
            let children = interop["value"].as_array().unwrap();
            assert_eq!(children.len(), 2);
            assert_eq!(children[0]["name"], "Bind");
            assert_eq!(children[0]["valueType"], "BIND");
            assert_eq!(children[0]["key"], format!("{}.bind", module.key()));
            assert_eq!(
                children[0]["description"],
                "liquidbounce.common.bind.description"
            );
            assert_eq!(children[1]["name"], "Hidden");
            assert_eq!(children[1]["value"], true);
            assert_eq!(
                children[1]["description"],
                "liquidbounce.common.hidden.description"
            );
            let file =
                serialize_camera_effect(module, &common, JsonFormat::File, &describe).unwrap();
            assert_eq!(
                file,
                json!({"name":module.name(),"value":[
                    {"name":"Enabled","value":true},
                    {"name":"Bind","value":common.key_bind},
                    {"name":"Hidden","value":true}
                ]})
            );
            let parsed = parse_settings(module, &file).unwrap();
            assert!(parsed.issues.is_empty());
            assert_eq!(
                parsed.changes,
                [
                ]
            );
        }
        assert!(
            serialize_camera_effect(ModuleKind::Zoom, &common, JsonFormat::File, &describe)
                .is_err()
        );
    }

    #[test]
    fn camera_effect_parser_keeps_declaration_order_ignores_invented_settings_and_continues() {
        for module in [ModuleKind::NoBob, ModuleKind::NoHurtCam] {
            let binding = json!({"boundKey":"key.keyboard.b","action":"Hold"});
            let input = json!({"name":module.name(),"value":[
                {"name":"Mode","active":"Invented","choices":false},
                {"name":"Hidden","value":true},
                {"name":"Bind","value":binding},
                {"name":"Enabled","value":true},
                {"name":"Hidden","value":false}
            ]});
            let patch = parse_settings(module, &input).unwrap();
            assert!(patch.issues.is_empty());
            assert_eq!(
                patch.changes,
                [
                ]
            );
            let malformed = json!({"name":module.name(),"value":[
                {"name":"Enabled","value":[]},
                {"name":"Bind","value":binding},
                {"name":"Hidden","value":false}
            ]});
            let patch = parse_settings(module, &malformed).unwrap();
            assert_eq!(patch.issues.len(), 1);
            assert_eq!(patch.issues[0].path, "Enabled");
            assert_eq!(
                patch.changes,
                [SettingChange::Bind(binding), SettingChange::Hidden(false)]
            );
            assert_eq!(ModuleKind::from_name(module.name()), Some(module));
            assert_eq!(
                ModuleKind::from_name(&module.name().to_ascii_lowercase()),
                None
            );
            assert!(parse_settings(module, &json!({"name":"OtherModule","value":[]})).is_err());
        }
    }

    #[test]
    fn original_tree_shapes_ranges_metadata_and_hidden_enabled_filter() {
        let zoom =
            serialize_zoom(&Zoom::default(), &common(), JsonFormat::Interop, &describe).unwrap();
        let values = zoom["value"].as_array().unwrap();
        assert_eq!(
            values
                .iter()
                .map(|value| value["name"].as_str().unwrap())
                .collect::<Vec<_>>(),
            [
                "Bind",
                "Hidden",
                "Zoom",
                "Scroll",
                "Transition",
                "DurationFactor"
            ]
        );
        assert_eq!(zoom["valueType"], "TOGGLEABLE");
        assert_eq!(values[3]["value"][0]["name"], "Enabled");
        assert_eq!(
            values[4]["choices"],
            json!([
                "Linear",
                "QuadIn",
                "QuadOut",
                "QuadInOut",
                "ExponentialIn",
                "ExponentialOut",
                "None"
            ])
        );
        assert_eq!(values[5]["range"], json!({"from":0.0,"to":10.0}));
        assert_eq!(values[5]["suffix"], "x");
        let file =
            serialize_zoom(&Zoom::default(), &common(), JsonFormat::File, &describe).unwrap();
        assert_eq!(file["value"][0], json!({"name":"Enabled","value":false}));
        assert!(file.get("valueType").is_none());
        assert!(file["value"][3].get("range").is_none());
        let no_fov =
            serialize_no_fov(&NoFov::default(), &common(), JsonFormat::Interop, &describe).unwrap();
        assert_eq!(
            no_fov["value"][2]["choices"]["Custom"]["value"][1]["value"],
            json!({"from":0.0,"to":1.5})
        );
        assert_eq!(
            no_fov["value"][2]["choices"]["Constant"]["value"][0]["key"],
            "liquidbounce.module.noFOV.mode.constant.fOV"
        );
    }

    #[test]
    fn parser_uses_declaration_order_first_duplicate_and_leaf_failure_continuation() {
        let patch = parse_settings(ModuleKind::Zoom,&json!({"name":"Zoom","value":[
            {"name":"DurationFactor","value":0.0},{"name":"Unknown","value":1},
            {"name":"Zoom","value":55},{"name":"Zoom","value":99},
            {"name":"Transition","value":"quadout"},
            {"name":"Scroll","value":[{"name":"Speed","value":3.0},{"name":"Enabled","value":false}]}
        ]})).unwrap();
        assert_eq!(
            patch.changes,
            [
            ]
        );
        let patch = parse_settings(ModuleKind::Zoom,&json!({"name":"Zoom","value":[{"name":"Zoom","value":null},{"name":"Transition","value":"unknown"},{"name":"DurationFactor","value":12.0}]})).unwrap();
        assert_eq!(patch.issues.len(), 2);
        assert_eq!(patch.changes, [SettingChange::DurationFactor(12.0)]);
    }

    #[test]
    fn fullbright_mode_is_deferred_before_gamma_parameter_changes() {
        let patch = parse_settings(ModuleKind::FullBright,&json!({"name":"FullBright","value":[{"name":"Mode","active":"NightVision","value":[],"choices":{"Gamma":{"name":"Gamma","value":[{"name":"Brightness","value":7}]},"NightVision":{"name":"NightVision","value":[]}}}]})).unwrap();
        assert_eq!(
            patch.changes,
            [
            ]
        );
        let mut no_fov = NoFov::default();
        let mut zoom = Zoom::default();
        let mut bright = FullBright::new();
        assert_eq!(
            apply_setting(&patch.changes[0], &mut no_fov, &mut zoom, &mut bright),
            ApplyResult::Runtime(RuntimeRequest::FullBrightMode(FullBrightMode::NightVision))
        );
        assert_eq!(bright.mode(), FullBrightMode::Gamma);
        assert_eq!(bright.gamma.brightness, 15);
        bright.switch_mode(FullBrightMode::NightVision, false, 0.0, |_| {});
        apply_setting(&patch.changes[1], &mut no_fov, &mut zoom, &mut bright);
        assert_eq!(bright.gamma.brightness, 7);
    }

    #[test]
    fn invalid_mode_does_not_skip_valid_choices_and_root_mismatch_rejects() {
        let patch = parse_settings(ModuleKind::NoFov,&json!({"name":"NoFOV","value":[{"name":"Mode","active":"custom","value":[],"choices":{"Constant":{"name":"Constant","value":[{"name":"FOV","value":123}]},"Custom":{"name":"Custom","value":[{"name":"Limit","value":{"from":2.0,"to":1.0}}]}}}]})).unwrap();
        assert_eq!(patch.issues.len(), 1);
        assert_eq!(
            patch.changes,
            [
            ]
        );
        assert!(parse_settings(ModuleKind::NoFov, &json!({"name":"NoFov","value":[]})).is_err());
    }

    #[test]
    fn setters_preserve_raw_ranges_and_boxed_float_equality() {
        let mut no_fov = NoFov::default();
        let mut zoom = Zoom::default();
        let mut bright = FullBright::new();
        apply_setting(
            &SettingChange::Zoom(999),
            &mut no_fov,
            &mut zoom,
            &mut bright,
        );
        assert_eq!(zoom.settings.zoom, 999);
        zoom.settings.duration_factor = 0.0;
        assert_eq!(
            apply_setting(
                &SettingChange::DurationFactor(-0.0),
                &mut no_fov,
                &mut zoom,
                &mut bright
            ),
            ApplyResult::Changed(true)
        );
        zoom.settings.duration_factor = f32::NAN;
        assert_eq!(
            apply_setting(
                &SettingChange::DurationFactor(f32::from_bits(0x7fc00001)),
                &mut no_fov,
                &mut zoom,
                &mut bright
            ),
            ApplyResult::Changed(false)
        );
        no_fov.custom.limit = (2.0, 1.0);
        assert_eq!(
            apply_setting(
                &SettingChange::CustomLimit(f32::NAN, 1.5),
                &mut no_fov,
                &mut zoom,
                &mut bright
            ),
            ApplyResult::Changed(false)
        );
    }

    #[test]
    fn numeric_coercion_matches_json_number_intvalue_without_clamping() {
        assert_eq!(parse_int(&json!(2147483648_u64), "test").unwrap(), i32::MIN);
        assert_eq!(parse_int(&json!(-1.75), "test").unwrap(), -1);
        assert_eq!(decimal_int("4294967297.99"), Some(1));
        assert_eq!(decimal_int("-2147483649"), Some(i32::MAX));
        assert_eq!(decimal_int("1e32"), Some(0));
        assert!(parse_int(&json!("1.2"), "test").is_err());
        assert!(!parse_bool(&json!("anything"), "test").unwrap());
    }

    #[test]
    fn attack_settings_roundtrip_and_defer_mutation_in_declaration_order() {
        let original = NoMissCooldown::default();
        let tree = serialize_no_miss_cooldown(&original, &common(), JsonFormat::Interop, &describe)
            .unwrap();
        assert_eq!(tree["value"][2]["value"], true);
        assert_eq!(tree["value"][3]["value"], false);
        let patch = parse_settings(
            ModuleKind::NoMissCooldown,
            &json!({"name":"NoMissCooldown","value":[
                {"name":"CancelAttackOnMiss","value":true},
                {"name":"RemoveAttackCooldown","value":false}
            ]}),
        )
        .unwrap();
        assert!(patch.issues.is_empty());
        let mut no_fov = NoFov::default();
        let mut zoom = Zoom::default();
        let mut bright = FullBright::new();
        let dispatched: Vec<_> = patch
            .changes
            .iter()
            .map(|change| apply_setting(change, &mut no_fov, &mut zoom, &mut bright))
            .collect();
        assert_eq!(
            dispatched,
            [
                ApplyResult::Runtime(RuntimeRequest::RemoveAttackCooldown(false)),
                ApplyResult::Runtime(RuntimeRequest::CancelAttackOnMiss(true))
            ]
        );
        assert!(
            serialize_camera_effect(
                ModuleKind::NoMissCooldown,
                &common(),
                JsonFormat::File,
                &describe
            )
            .is_err()
        );
    }

    #[test]
    fn no_swing_multichoice_keeps_wire_tags_and_allows_empty_selection() {
        use liquidbounce_behaviors::no_swing::{HideFor, NoSwing};
        let state = NoSwing::default();
        let ui = serialize_no_swing(&state, &common(), JsonFormat::Interop, &describe).unwrap();
        assert_eq!(ui["value"][2]["choices"], json!(["Client", "Server"]));
        assert_eq!(ui["value"][2]["value"], json!(["Client", "Server"]));
        assert_eq!(ui["value"][2]["canBeNone"], true);
        for (input, expected) in [
            (json!(["server", "unrecognized", "SERVER"]), HideFor::SERVER),
            (json!([]), HideFor::NONE),
            (json!("client"), HideFor::CLIENT),
        ] {
            let patch = parse_settings(
                ModuleKind::NoSwing,
                &json!({"name":"NoSwing","value":[{"name":"HideFor","value":input}]}),
            )
            .unwrap();
            assert!(patch.issues.is_empty());
            assert_eq!(patch.changes, [SettingChange::NoSwingHideFor(expected)]);
            state.set_hide_for(expected);
            let file = serialize_no_swing(&state, &common(), JsonFormat::File, &describe).unwrap();
            let restored = parse_settings(ModuleKind::NoSwing, &file).unwrap();
            assert!(
                restored
                    .changes
                    .contains(&SettingChange::NoSwingHideFor(expected))
            );
            assert!(
                file["value"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|leaf| leaf.get("valueType").is_none())
            );
        }
    }

    #[test]
    fn float_output_uses_shortest_float_value_and_rejects_nonfinite_values() {
        assert_eq!(float_json(0.1, "test").unwrap().to_string(), "0.1");
        assert!(float_json(f32::NAN, "test").is_err());
        let mut zoom = Zoom::default();
        zoom.settings.duration_factor = f32::INFINITY;
        assert!(serialize_zoom(&zoom, &common(), JsonFormat::Interop, &describe).is_err());
    }

    #[test]
    fn registry_tokens_and_auto_fish_ranges_follow_original_adapters() {
        let patch = |value| {
            parse_settings(ModuleKind::AutoF5, &json!({"name":"AutoF5","value":[{"name":"CheckScreenHandlerType","value":[{"name":"Types","value":value}]}]})).unwrap()
        };
        for invalid in [json!(true), Value::Null, json!(["hopper", false])] {
            let result = patch(invalid);
            assert_eq!(result.issues.len(), 1);
            assert!(result.changes.is_empty());
        }
        assert_eq!(
            patch(json!(":hopper")).changes,
            vec![SettingChange::AutoF5(AutoF5Setting::Types(
                [Some("minecraft:hopper".to_owned())].into()
            ))]
        );
        let result = parse_settings(
            ModuleKind::AutoFish,
            &json!({"name":"AutoFish","value":[
                {"name":"ReelDelay","value":-2},
                {"name":"Sounds","value":["custom:bite",true]},
                {"name":"RecastRod","value":[{"name":"Delay","value":{"from":20,"to":-3}}]},
                {"name":"AutoCastRod","value":[{"name":"Delay","value":"15"}]}
            ]}),
        )
        .unwrap();
        assert_eq!(result.issues.len(), 2);
        assert_eq!(
            result.changes,
            vec![
            ]
        );
        let tree = serialize_auto_fish(
            &Default::default(),
            &common(),
            JsonFormat::Interop,
            &describe,
        )
        .unwrap();
        assert_eq!(tree["value"][2]["range"], json!({"from":0,"to":20}));
        assert_eq!(tree["value"][3]["registry"], "sound_event");
        assert_eq!(ModuleKind::NoSlotSet.category(), "Player");
    }
}
