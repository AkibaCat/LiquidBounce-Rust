// SPDX-License-Identifier: GPL-3.0-or-later
//! Exact-source Horizon adapter; optional real modules require its phase patch.
#[cfg(feature = "webview")]
mod actor;
#[cfg(feature = "webview")]
mod menu_backend;
#[cfg(feature = "behavior-hooks")]
mod anti_afk_runtime;
#[cfg(feature = "behavior-hooks")]
mod anti_hunger_notifications;
#[cfg(feature = "behavior-hooks")]
mod anti_hunger_packets;
#[cfg(feature = "behavior-hooks")]
mod auto_account_runtime;
#[cfg(feature="behavior-hooks")]
mod report_helper_runtime;
#[cfg(feature="behavior-hooks")]
mod auto_queue_runtime;
#[cfg(feature="behavior-hooks")]
mod macros_runtime;
#[cfg(feature="behavior-hooks")]
mod client_commands_runtime;
#[cfg(feature = "behavior-hooks")]
mod auto_break_runtime;
#[cfg(feature = "behavior-hooks")]
mod old_regen_runtime;
#[cfg(feature = "behavior-hooks")]
mod auto_disable_runtime;
#[cfg(feature = "behavior-hooks")]
mod packet_mine_runtime;
#[cfg(feature = "behavior-hooks")]
mod hud_runtime;
#[cfg(feature = "behavior-hooks")]
mod breadcrumbs_runtime;
#[cfg(feature="behavior-hooks")]
mod storage_esp_runtime;
#[cfg(feature="behavior-hooks")]
mod block_esp_runtime;
#[cfg(feature = "behavior-hooks")]
mod hats_runtime;
#[cfg(feature="behavior-hooks")]
mod target_renderer_runtime;
#[cfg(feature="webview")]
mod theme_font_runtime;
#[cfg(feature = "behavior-hooks")]
mod wings_runtime;
#[cfg(feature = "behavior-hooks")]
mod crosshair_runtime;
#[cfg(feature="behavior-hooks")]
mod logoff_spot_runtime;
#[cfg(feature="behavior-hooks")]
mod inventory_tracker_runtime;
#[cfg(feature="behavior-hooks")]
mod name_protect_runtime;
#[cfg(feature="behavior-hooks")]
mod particles_runtime;
#[cfg(feature="behavior-hooks")]
mod trajectories_runtime;
#[cfg(feature="behavior-hooks")]
mod custom_ambience_runtime;
#[cfg(feature="behavior-hooks")]
mod tnt_timer_runtime;
#[cfg(feature="behavior-hooks")]
mod potion_fx_runtime;
#[cfg(feature="behavior-hooks")]
mod totem_effect_runtime;
#[cfg(feature="behavior-hooks")]
mod hit_fx_runtime;
#[cfg(feature="behavior-hooks")]
mod jump_effect_runtime;
#[cfg(feature="behavior-hooks")]
mod hole_esp_runtime;
#[cfg(feature="behavior-hooks")]
mod blink_esp_runtime;
#[cfg(feature = "behavior-hooks")]
mod protection_zones_runtime;
#[cfg(feature = "behavior-hooks")]
mod placement_esp_runtime;
#[cfg(feature = "behavior-hooks")]
mod new_chunks_runtime;
#[cfg(feature="behavior-hooks")]
mod stronghold_finder_runtime;
#[cfg(feature = "behavior-hooks")]
mod tracers_runtime;
#[cfg(feature="behavior-hooks")]
mod esp_runtime;
#[cfg(feature="behavior-hooks")]
mod radar_runtime;
#[cfg(feature="behavior-hooks")]
mod item_esp_runtime;
#[cfg(feature="behavior-hooks")]
mod item_chams_runtime;
#[cfg(feature="behavior-hooks")]
mod item_tags_runtime;
#[cfg(feature="behavior-hooks")]
mod nametags_runtime;
#[cfg(feature="behavior-hooks")]
mod better_chat_runtime;
#[cfg(feature="behavior-hooks")]
mod debug_runtime;
#[cfg(feature="behavior-hooks")]
mod bed_plates_runtime;
#[cfg(feature = "behavior-hooks")]
mod damage_particles_runtime;
#[cfg(feature = "behavior-hooks")]
mod block_outline_runtime;
#[cfg(feature = "behavior-hooks")]
mod better_tab_runtime;
#[cfg(feature = "behavior-hooks")]
mod combine_mobs_runtime;
#[cfg(feature = "behavior-hooks")]
mod mob_owners_runtime;
#[cfg(feature = "behavior-hooks")]
mod auto_tool;
#[cfg(feature = "behavior-hooks")]
#[path = "inventory_packets.rs"]
mod auto_tool_packets;
#[cfg(feature = "behavior-hooks")]
#[path = "inventory_runtime.rs"]
mod auto_tool_runtime;
#[cfg(feature = "behavior-hooks")]
mod behaviors;
#[cfg(feature = "behavior-hooks")]
mod better_inventory_draw;
#[cfg(feature = "behavior-hooks")]
mod better_inventory_runtime;
#[cfg(feature = "webview")]
mod browser;
#[cfg(feature = "behavior-hooks")]
mod hud_components;
#[cfg(feature = "behavior-hooks")]
pub mod config;
#[cfg(feature = "behavior-hooks")]
mod coroutine_ticker;
#[cfg(feature = "behavior-hooks")]
mod damage_runtime;
#[cfg(feature = "behavior-hooks")]
mod derp_runtime;
#[cfg(feature = "behavior-hooks")]
mod fast_break_packets;
#[cfg(feature = "behavior-hooks")]
mod fast_break_runtime;
#[cfg(feature = "behavior-hooks")]
pub mod fishing_packets;
#[cfg(feature = "behavior-hooks")]
mod friends;
#[cfg(feature = "behavior-hooks")]
mod gui_closer_runtime;
#[cfg(feature = "behavior-hooks")]
mod hand_derp_packets;
#[cfg(feature = "behavior-hooks")]
mod high_jump_runtime;
#[cfg(feature = "behavior-hooks")]
mod no_slow_runtime;
#[cfg(feature = "behavior-hooks")]
mod no_web_runtime;
#[cfg(feature="behavior-hooks")]
mod vehicle_control_runtime;
#[cfg(feature = "behavior-hooks")]
mod liquid_walk_runtime;
#[cfg(feature = "behavior-hooks")]
mod step_runtime;
#[cfg(feature = "behavior-hooks")]
mod reverse_step_runtime;
#[cfg(feature = "behavior-hooks")]
mod auto_walk_runtime;
#[cfg(feature = "behavior-hooks")]
mod edge_assistance_runtime;
#[cfg(feature = "behavior-hooks")]
mod movement_guard_runtime;
#[cfg(feature = "behavior-hooks")]
mod freeze_runtime;
#[cfg(feature = "behavior-hooks")]
mod inventory;
#[cfg(feature = "behavior-hooks")]
mod item_scroller_runtime;
#[cfg(feature = "behavior-hooks")]
mod middle_click_action_runtime;
#[cfg(feature = "behavior-hooks")]
mod super_knockback_runtime;
#[cfg(feature = "behavior-hooks")]
mod mace_kill_runtime;
#[cfg(feature = "behavior-hooks")]
mod spear_kill_runtime;
#[cfg(feature = "behavior-hooks")]
mod crosshair_clicker_runtime;
#[cfg(feature="behavior-hooks")]
mod criticals_runtime;
#[cfg(feature = "behavior-hooks")]
mod no_clip_packets;
#[cfg(feature = "behavior-hooks")]
mod no_clip_runtime;
#[cfg(feature = "behavior-hooks")]
mod fast_use_runtime;
#[cfg(feature="behavior-hooks")]
mod fast_exp_runtime;
#[cfg(feature="behavior-hooks")]
mod auto_wind_charge_runtime;
#[cfg(feature="behavior-hooks")]
mod auto_buff_runtime;
#[cfg(feature="behavior-hooks")]
mod auto_bow_runtime;
#[cfg(feature="behavior-hooks")]
mod auto_pearl_runtime;
#[cfg(feature="behavior-hooks")]
mod extinguish_runtime;
#[cfg(feature="behavior-hooks")]
mod block_placer_runtime;
#[cfg(feature="behavior-hooks")]
mod liquid_filler_runtime;
#[cfg(feature="behavior-hooks")]
mod hole_filler_runtime;
#[cfg(feature="behavior-hooks")]
mod world_noop_runtime;
#[cfg(feature="behavior-hooks")]
mod bed_defender_runtime;
#[cfg(feature="behavior-hooks")]
mod easy_pearl_runtime;
#[cfg(feature="behavior-hooks")]
mod elytra_target_runtime;
#[cfg(feature="behavior-hooks")]
mod auto_rod_runtime;
#[cfg(feature="behavior-hooks")]
mod auto_shoot_runtime;
#[cfg(feature="behavior-hooks")]
mod scaffold_runtime;
#[cfg(feature="behavior-hooks")]
mod scaffold_features_runtime;
#[cfg(feature="behavior-hooks")]
mod fake_lag_runtime;
#[cfg(feature="behavior-hooks")]
mod sword_block_runtime;
#[cfg(feature="behavior-hooks")]
mod aimbot_runtime;
#[cfg(feature="behavior-hooks")]
mod target_strafe_runtime;
#[cfg(feature="behavior-hooks")]
mod speed_runtime;
#[cfg(feature="behavior-hooks")]
mod kill_aura_runtime;
#[cfg(feature="behavior-hooks")]
mod blink_runtime;
#[cfg(feature="behavior-hooks")]
mod auto_dodge_runtime;
#[cfg(feature="behavior-hooks")]
mod air_place_runtime;
#[cfg(feature="behavior-hooks")]
mod flag_check_runtime;
#[cfg(feature="behavior-hooks")]
mod packet_logger_runtime;
#[cfg(feature="behavior-hooks")]
mod anti_staff_runtime;
#[cfg(feature="behavior-hooks")]
mod notifier_runtime;
#[cfg(feature="behavior-hooks")]
mod anti_bot_runtime;
#[cfg(feature="behavior-hooks")]
mod anti_void_runtime;
#[cfg(feature="behavior-hooks")]
mod xray_runtime;
#[cfg(feature="behavior-hooks")]
mod inventory_move_runtime;
#[cfg(feature="behavior-hooks")]
mod packet_logger_fields;
#[cfg(feature = "behavior-hooks")]
mod one_shots_runtime;
#[cfg(feature = "behavior-hooks")]
pub mod packet_modules;
#[cfg(feature = "behavior-hooks")]
pub mod packets;
#[cfg(feature = "behavior-hooks")]
mod plugins_runtime;
#[cfg(feature="behavior-hooks")]
mod anti_cheat_detect_runtime;
#[cfg(feature="behavior-hooks")]
mod anti_exploit_runtime;
#[cfg(feature = "behavior-hooks")]
mod potion_spoof_runtime;
#[cfg(feature = "behavior-hooks")]
mod reset_vl_runtime;
#[cfg(feature = "behavior-hooks")]
pub mod rotation_packets;
#[cfg(feature = "behavior-hooks")]
mod skin_derp_runtime;
#[cfg(feature = "behavior-hooks")]
pub mod slots;
#[cfg(feature = "behavior-hooks")]
mod smart_eat;
#[cfg(feature = "behavior-hooks")]
mod smart_eat_runtime;
#[cfg(feature = "behavior-hooks")]
mod snap_tap;
#[cfg(feature = "behavior-hooks")]
mod spider_runtime;
#[cfg(feature = "behavior-hooks")]
mod target_lock_runtime;
#[cfg(feature = "behavior-hooks")]
mod teams_runtime;
#[cfg(feature="behavior-hooks")]
mod murder_mystery_runtime;
#[cfg(feature="behavior-hooks")]
mod offhand_runtime;
#[cfg(feature="behavior-hooks")]
mod projectile_puncher_runtime;
#[cfg(feature="behavior-hooks")]
mod free_cam_runtime;
#[cfg(feature = "behavior-hooks")]
mod spammer_runtime;
#[cfg(feature="behavior-hooks")]
mod auto_chat_game_runtime;
#[cfg(feature = "behavior-hooks")]
mod ping_spoof_runtime;
#[cfg(feature = "behavior-hooks")]
mod clip_runtime;
#[cfg(feature = "behavior-hooks")]
mod book_bot_runtime;
#[cfg(feature = "behavior-hooks")]
mod chest_cleaner_runtime;
#[cfg(feature = "behavior-hooks")]
mod auto_deposit_runtime;
#[cfg(feature = "behavior-hooks")]
mod replenish_runtime;
#[cfg(feature="behavior-hooks")]
mod auto_crafter_runtime;
#[cfg(feature = "behavior-hooks")]
mod inventory_cleaner_runtime;
#[cfg(feature="behavior-hooks")]
mod auto_armor_runtime;
#[cfg(feature="behavior-hooks")]
mod auto_mob_heal_runtime;
#[cfg(feature="behavior-hooks")]
mod chest_stealer_runtime;
#[cfg(feature = "behavior-hooks")]
mod sneak_runtime;
#[cfg(feature = "behavior-hooks")]
mod targeting;
#[cfg(feature = "behavior-hooks")]
mod text_field_protect_runtime;
#[cfg(feature = "behavior-hooks")]
mod timer_runtime;
#[cfg(feature = "behavior-hooks")]
mod true_sight_runtime;
#[cfg(feature = "webview")]
pub use browser::{BrowserConfig, browser_hooks, shutdown_browser_current_thread};
pub mod input;
mod runtime;
pub use runtime::{HORIZON_REVISION, LiquidBounceHooks, MOD_ID, ObservedState, hooks};
