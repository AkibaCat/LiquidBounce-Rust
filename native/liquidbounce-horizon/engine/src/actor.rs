// SPDX-License-Identifier: GPL-3.0-or-later
//! Actual registry/settings actor, invoked only from the native client callback.
use crate::{
    behaviors::RawOptions,
    config::{self, ApplyResult, CommonSettings, JsonFormat, ModuleKind, RuntimeRequest},
    runtime::State,
};
use liquidbounce_core::{BindAction, InputBind, InputKey, Modifier, Modifiers, ModuleId};
use liquidbounce_interop::protocol::ModuleAccess;
use serde_json::{Value, json};
use std::path::Path;

const MODULES: &[ModuleKind] = ModuleKind::ALL;

// Verbatim strings from the pinned LiquidBounce en_us.json. Shared Minecraft
// input names continue to use Horizon's actually loaded language separately.
fn description(kind: ModuleKind) -> String {
    liquidbounce_interop::localization::module_description(
        &horizon_client::lang::code(),
        kind.name(),
    )
    .map(str::to_owned)
    .unwrap_or_else(|| format!("{}.description", kind.key()))
}

pub(crate) fn bind_json(bind: Option<InputBind>) -> Value {
    let bind = bind.unwrap_or(InputBind::new(InputKey::Unbound, BindAction::Toggle));
    let key = match bind.key {
        InputKey::Unbound => "key.keyboard.unknown".to_owned(),
        InputKey::Keyboard(code) => horizon_client::key_names::KEYBOARD
            .iter()
            .find_map(|(scan, name)| (*scan == code).then_some((*name).to_owned()))
            .unwrap_or_else(|| format!("scancode.{code}")),
        InputKey::Mouse(code) => horizon_client::key_names::MOUSE
            .iter()
            .find_map(|(button, name)| (*button == code).then_some((*name).to_owned()))
            .unwrap_or_else(|| format!("key.mouse.{code}")),
    };
    let mut value = json!({"boundKey":key,"action":match bind.action { BindAction::Toggle=>"Toggle",BindAction::Hold=>"Hold",BindAction::Smart=>"Smart" }});
    let modifiers: Vec<_> = [
        (Modifier::Shift, "Shift"),
        (Modifier::Control, "Control"),
        (Modifier::Alt, "Alt"),
        (Modifier::Super, "Super"),
    ]
    .into_iter()
    .filter_map(|(m, name)| bind.modifiers.contains(m).then_some(name))
    .collect();
    if !modifiers.is_empty() {
        value["modifiers"] = json!(modifiers);
    }
    value
}

fn parse_bind(value: &Value) -> Result<InputBind, String> {
    if let Some(code) = value.as_i64() {
        return Ok(InputBind::new(
            InputKey::Keyboard(code as i32),
            BindAction::Toggle,
        ));
    }
    if !value.is_object() {
        return Ok(InputBind::new(InputKey::Unbound, BindAction::Toggle));
    }
    let name = value["boundKey"]
        .as_str()
        .ok_or("Bind.boundKey must be an input name")?;
    let key = crate::input::configured_bind(name)
        .or_else(|| {
            name.strip_prefix("scancode.")
                .and_then(|n| n.parse().ok())
                .map(InputKey::Keyboard)
        })
        .unwrap_or(InputKey::Unbound);
    let action = match value["action"].as_str() {
        Some("Hold") => BindAction::Hold,
        Some("Smart") => BindAction::Smart,
        _ => BindAction::Toggle,
    };
    let mut modifiers = Vec::new();
    if let Some(values) = value["modifiers"].as_array() {
        for value in values {
            if let Some(modifier) = match value.as_str() {
                Some("Shift") => Some(Modifier::Shift),
                Some("Control") => Some(Modifier::Control),
                Some("Alt") => Some(Modifier::Alt),
                Some("Super") => Some(Modifier::Super),
                _ => None,
            } {
                modifiers.push(modifier);
            }
        }
    }
    Ok(InputBind::new(key, action).with_modifiers(Modifiers::new(&modifiers)))
}

pub(crate) struct Actor<'a> {
    pub state: &'a mut State,
    pub options: RawOptions,
    /// Raw local-player attribute observed before locking the actor. Reach's
    /// ThroughWallsRange onChange consumes this snapshot only when written.
    pub entity_interaction_attribute: Option<f64>,
}
impl Actor<'_> {
    fn id(&self, kind: ModuleKind) -> ModuleId {
        self.state
            .registry
            .modules()
            .find(|module| module.definition().name == kind.name())
            .expect("implemented module is registered")
            .id()
    }
    fn common(&self, kind: ModuleKind) -> CommonSettings {
        let module = self
            .state
            .registry
            .module(self.id(kind))
            .expect("registered module");
        CommonSettings {
            enabled: module.enabled(),
            hidden: self.state.hidden.get(kind.name()).copied().unwrap_or(false),
            key_bind: bind_json(module.definition().bind),
        }
    }
    fn serialize(&self, kind: ModuleKind, format: JsonFormat) -> Result<Value, String> {
        let common = self.common(kind);
        let language = horizon_client::lang::code();
        let describe = |key: &str| liquidbounce_interop::localization::describe(&language, key);
        if kind.is_unimplemented() {
            return config::serialize_unimplemented(kind, &common, format, &describe)
                .map_err(|error| error.to_string());
        }
        let b = self.state.behaviors.borrow();
        match kind {
            ModuleKind::NoFov => config::serialize_no_fov(&b.no_fov, &common, format, &describe),
            ModuleKind::Zoom => config::serialize_zoom(&b.zoom, &common, format, &describe),
            ModuleKind::FullBright => {
                config::serialize_full_bright(&b.full_bright, &common, format, &describe)
            }
            ModuleKind::NoMissCooldown => {
                config::serialize_no_miss_cooldown(&b.no_miss_cooldown, &common, format, &describe)
            }
            ModuleKind::NoSwing => {
                config::serialize_no_swing(&b.no_swing, &common, format, &describe)
            }
            ModuleKind::NoPitchLimit => {
                config::serialize_no_pitch_limit(&b.no_pitch_limit, &common, format, &describe)
            }
            ModuleKind::NoInterpolation => {
                config::serialize_no_interpolation(&b.no_interpolation, &common, format, &describe)
            }
            ModuleKind::NoSlowBreak => {
                config::serialize_no_slow_break(&b.no_slow_break, &common, format, &describe)
            }
            ModuleKind::Sprint => config::serialize_sprint(&b.sprint, &common, format, &describe),
            ModuleKind::AntiBlind => {
                config::serialize_anti_blind(&b.anti_blind, &common, format, &describe)
            }
            ModuleKind::AirJump => {
                config::serialize_air_jump(&b.air_jump, &common, format, &describe)
            }
            ModuleKind::NoPush => config::serialize_no_push(&b.no_push, &common, format, &describe),
            ModuleKind::CrystalView => {
                config::serialize_crystal_view(&b.crystal_view, &common, format, &describe)
            }
            ModuleKind::FastPlace => {
                config::serialize_fast_place(&b.fast_place, &common, format, &describe)
            }
            ModuleKind::AutoF5 => config::serialize_auto_f5(&b.auto_f5, &common, format, &describe),
            ModuleKind::Reach => config::serialize_reach(&b.reach, &common, format, &describe),
            ModuleKind::NoPose => config::serialize_no_pose(&b.no_pose, &common, format, &describe),
            ModuleKind::NoRotateSet => {
                config::serialize_no_rotate_set(&b.no_rotate_set, &common, format, &describe)
            }
            ModuleKind::CameraClip => {
                config::serialize_camera_clip(&b.camera_clip, &common, format, &describe)
            }
            ModuleKind::DankBobbing => {
                config::serialize_dank_bobbing(&b.dank_bobbing, &common, format, &describe)
            }
            ModuleKind::MultiActions => {
                config::serialize_multi_actions(&b.multi_actions, &common, format, &describe)
            }
            ModuleKind::Twerk => config::serialize_twerk(&b.twerk, &common, format, &describe),
            ModuleKind::Hitbox => config::serialize_hitbox(&b.hitbox, &common, format, &describe),
            ModuleKind::AutoWeapon => {
                config::serialize_auto_weapon(&b.auto_weapon, &common, format, &describe)
            }
            ModuleKind::Chams => config::serialize_chams(&b.chams, &common, format, &describe),
            ModuleKind::AutoTool => {
                config::serialize_auto_tool(&b.auto_tool, &common, format, &describe)
            }
            ModuleKind::AntiAfk => {
                config::serialize_anti_afk(&b.anti_afk, &common, format, &describe)
            }
            ModuleKind::TridentBoost => {
                config::serialize_trident_boost(&b.trident_boost, &common, format, &describe)
            }
            ModuleKind::SleepWalker => config::serialize_sleep_walker(&common, format, &describe),
            ModuleKind::Derp => config::serialize_derp(&b.derp, &common, format, &describe),
            ModuleKind::FreeLook => {
                config::serialize_free_look(&b.free_look, &common, format, &describe)
            }
            ModuleKind::HandDerp => config::serialize_hand_derp(
                &b.hand_derp.behavior.lock().expect("HandDerp poisoned"),
                &common,
                format,
                &describe,
            ),
            ModuleKind::VehicleControl=>config::serialize_vehicle_control(&b.vehicle_control.settings,&common,format,&describe),
            ModuleKind::VehicleBoost => {
                config::serialize_vehicle_boost(&b.vehicle_boost, &common, format, &describe)
            }
            ModuleKind::BlockBounce => {
                config::serialize_block_bounce(&b.block_bounce, &common, format, &describe)
            }
            ModuleKind::SmartEat => {
                config::serialize_smart_eat(&b.smart_eat, &common, format, &describe)
            }
            ModuleKind::SkinDerp => {
                config::serialize_skin_derp(&b.skin_derp, &common, format, &describe)
            }
            ModuleKind::EntityControl => {
                config::serialize_entity_control(&b.entity_control, &common, format, &describe)
            }
            ModuleKind::SnapTap => config::serialize_snap_tap(&common, format, &describe),
            ModuleKind::ElytraRecast => config::serialize_elytra_recast(&common, format, &describe),
            ModuleKind::Vomit => config::serialize_vomit(&b.vomit, &common, format, &describe),
            ModuleKind::ElytraSwap => {
                config::serialize_elytra_swap(&b.elytra_swap, &common, format, &describe)
            }
            ModuleKind::Timer => config::serialize_timer(
                &b.timer.state.lock().expect("Timer poisoned").settings,
                &common,
                format,
                &describe,
            ),
            ModuleKind::BetterInventory => {
                config::serialize_better_inventory(&b.better_inventory, &common, format, &describe)
            }
            ModuleKind::TerrainSpeed => {
                config::serialize_terrain_speed(&b.terrain_speed, &common, format, &describe)
            }
            ModuleKind::ReverseStep => config::serialize_reverse_step(&b.reverse_step,&common,format,&describe),
            ModuleKind::Step => config::serialize_step(&b.step.settings(),&common,format,&describe),
            ModuleKind::LiquidWalk => config::serialize_liquid_walk(&b.liquid_walk.settings(),&common,format,&describe),
            ModuleKind::NoWeb => config::serialize_no_web(&b.no_web,&common,format,&describe),
            ModuleKind::NoSlow => config::serialize_no_slow(&b.no_slow.settings(),&common,format,&describe),
            ModuleKind::HighJump => {
                config::serialize_high_jump(&b.high_jump, &common, format, &describe)
            }
            ModuleKind::AutoWalk => {
                config::serialize_auto_walk(&b.auto_walk, &common, format, &describe)
            }
            ModuleKind::Parkour => config::serialize_parkour(&b.parkour,&common,format,&describe),
            ModuleKind::SafeWalk => config::serialize_safe_walk(&b.safe_walk,&common,format,&describe),
            ModuleKind::Anchor => config::serialize_anchor(&b.anchor,&common,format,&describe),
            ModuleKind::AvoidHazards => config::serialize_avoid_hazards(&b.avoid_hazards,&common,format,&describe),
            ModuleKind::Freeze => config::serialize_freeze(&b.freeze.settings(),&common,format,&describe),
            ModuleKind::Kick => config::serialize_kick(b.kick_mode, &common, format, &describe),
            ModuleKind::AutoLeave => {
                config::serialize_auto_leave(&b.auto_leave.settings, &common, format, &describe)
            }
            ModuleKind::TargetLock => {
                config::serialize_target_lock(&b.target_lock, &common, format, &describe)
            }
            ModuleKind::YggdrasilSignatureFix => config::serialize_yggdrasil_signature(
                b.yggdrasil_force_slim,
                &common,
                format,
                &describe,
            ),
            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),
            ModuleKind::AutoDisable => config::serialize_auto_disable(&b.auto_disable, &common, format, &describe),
            ModuleKind::AutoAccount => config::serialize_auto_account(&b.auto_account,&common,format,&describe),
            ModuleKind::Hats => config::serialize_hats(&b.hats,&common,format,&describe),
            ModuleKind::Wings => config::serialize_wings(&b.wings,&common,format,&describe),
            ModuleKind::Crosshair => config::serialize_crosshair(&b.crosshair,&common,format,&describe),
            ModuleKind::StorageEsp=>config::serialize_storage_esp(&b.storage_esp,&common,format,&describe),
            ModuleKind::BlockEsp=>config::serialize_block_esp(&b.block_esp,&common,format,&describe),
            ModuleKind::LogoffSpot => config::serialize_logoff_spot(b.logoff_spot.send_in_chat(),&common,format,&describe),
            ModuleKind::InventoryTracker => config::serialize_inventory_tracker(b.inventory_tracker.model.save_players,&common,format,&describe),
            ModuleKind::NameProtect => config::serialize_name_protect(&b.name_protect.settings(),&common,format,&describe),
            ModuleKind::Particles => config::serialize_particles(&b.particles.settings,&common,format,&describe),
            ModuleKind::Trajectories=>config::serialize_trajectories(&b.trajectories,&common,format,&describe),
            ModuleKind::CustomAmbience=>config::serialize_custom_ambience(&b.custom_ambience,&common,format,&describe),
            ModuleKind::TNTTimer=>config::serialize_tnt_timer(&b.tnt_timer,&common,format,&describe),
            ModuleKind::PotionFX=>config::serialize_potion_fx(&b.potion_fx,&common,format,&describe),
            ModuleKind::TotemEffect=>config::serialize_totem_effect(&b.totem_effect,&common,format,&describe),
            ModuleKind::HitFx=>config::serialize_hit_fx(&b.hit_fx.settings,&common,format,&describe),
            ModuleKind::JumpEffect=>config::serialize_jump_effect(&b.jump_effect,&common,format,&describe),
            ModuleKind::HoleEsp => config::serialize_hole_esp(&b.hole_esp,&common,format,&describe),
            ModuleKind::ProtectionZones => config::serialize_protection_zones(&b.protection_zones,&common,format,&describe),
            ModuleKind::ProphuntEsp => config::serialize_prophunt_esp(&b.prophunt_esp,&common,format,&describe),
            ModuleKind::VoidEsp => config::serialize_void_esp(&b.void_esp,&common,format,&describe),
            ModuleKind::NewChunks => config::serialize_new_chunks(&b.new_chunks,&common,format,&describe),
            ModuleKind::StrongholdFinder=>config::serialize_stronghold_finder(&b.stronghold_finder,&common,format,&describe),
            ModuleKind::Tracers => config::serialize_tracers(&b.tracers,&common,format,&describe),
            ModuleKind::Esp=>config::serialize_esp(&b.esp,&common,format,&describe),
            ModuleKind::Radar=>config::serialize_radar(&b.radar,&common,format,&describe),
            ModuleKind::ItemEsp=>config::serialize_item_esp(&b.item_esp,&common,format,&describe),
            ModuleKind::ItemChams=>config::serialize_item_chams(&b.item_chams,&common,format,&describe),
            ModuleKind::ItemTags=>config::serialize_item_tags(&b.item_tags.settings,&common,format,&describe),
            ModuleKind::Nametags=>config::serialize_nametags(&b.nametags.settings,&common,format,&describe),
            ModuleKind::BetterChat=>config::serialize_better_chat(&b.better_chat.settings,&common,format,&describe),
            ModuleKind::Debug=>config::serialize_debug(&b.debug.model.settings,&common,format,&describe),
            ModuleKind::ClickGui=>config::serialize_click_gui(&b.click_gui.settings,&common,format,&describe,b.click_gui.refresh_hz),
            ModuleKind::BedPlates=>config::serialize_bed_plates(&b.bed_plates.settings,&common,format,&describe),
            ModuleKind::DamageParticles => config::serialize_damage_particles(&b.damage_particles,&common,format,&describe),
            ModuleKind::Breadcrumbs => config::serialize_breadcrumbs(&b.breadcrumbs,&common,format,&describe),
            ModuleKind::BlockOutline => config::serialize_block_outline(&b.block_outline,&common,format,&describe),
            ModuleKind::BetterTab => config::serialize_better_tab(&b.better_tab, &common, format, &describe),
            ModuleKind::CombineMobs => config::serialize_combine_mobs(&b.combine_mobs, &common, format, &describe),
            ModuleKind::MobOwners => config::serialize_mob_owners(&b.mob_owners, &common, format, &describe),
            ModuleKind::ReportHelper=>config::serialize_report_helper(&b.report_helper,&common,format,&describe),
            ModuleKind::AutoQueue=>config::serialize_auto_queue(&b.auto_queue,&common,format,&describe),
            ModuleKind::Macros=>config::serialize_macros(&b.macros,&common,format,&describe),
            ModuleKind::AutoAccount => {
                config::serialize_auto_account(&b.auto_account, &common, format, &describe)
            }
            ModuleKind::MiddleClickAction => config::serialize_middle_click_action(
                &b.middle_click_action,
                &common,
                format,
                &describe,
            ),
            ModuleKind::SuperKnockback => config::serialize_super_knockback(&b.super_knockback, &common, format, &describe),
            ModuleKind::MaceKill => config::serialize_mace_kill(&b.mace_kill, &common, format, &describe),
            ModuleKind::SpearKill => config::serialize_spear_kill(&b.spear_kill, &common, format, &describe),
            ModuleKind::TriggerBot => config::serialize_trigger_bot(&b.crosshair_clicker,&common,format,&describe),
            ModuleKind::Criticals=>config::serialize_criticals(&b.criticals.settings(),&common,format,&describe),
            ModuleKind::AutoClicker => config::serialize_auto_clicker(&b.crosshair_clicker,&common,format,&describe),
            ModuleKind::Spider => config::serialize_spider(&b.spider, &common, format, &describe),
            ModuleKind::FastExp=>config::serialize_fast_exp(&b.fast_exp,&common,format,&describe),
            ModuleKind::AutoWindCharge=>config::serialize_auto_wind_charge(&b.auto_wind_charge,&common,format,&describe),
            ModuleKind::AutoBuff=>config::serialize_auto_buff(&b.auto_buff,&common,format,&describe),
            ModuleKind::AutoBow=>config::serialize_auto_bow(&b.auto_bow,&common,format,&describe),
            ModuleKind::AutoPearl=>config::serialize_auto_pearl(&b.auto_pearl,&common,format,&describe),
            ModuleKind::Extinguish=>config::serialize_extinguish(&b.extinguish,&common,format,&describe),
            ModuleKind::LiquidFiller=>config::serialize_liquid_filler(&b.liquid_filler,&common,format,&describe),
            ModuleKind::HoleFiller=>config::serialize_hole_filler(&b.hole_filler,&common,format,&describe),
            ModuleKind::Surround=>config::serialize_surround(&b.surround,&common,format,&describe),
            ModuleKind::BlockIn=>config::serialize_block_in(&b.block_in,&common,format,&describe),
            ModuleKind::BedDefender=>config::serialize_bed_defender(&b.bed_defender,&common,format,&describe),
            ModuleKind::EasyPearl=>config::serialize_easy_pearl(&b.easy_pearl,&common,format,&describe),
            ModuleKind::ElytraTarget=>config::serialize_elytra_target(&b.elytra_target,&common,format,&describe),
            ModuleKind::AutoRod=>config::serialize_auto_rod(&b.auto_rod,&common,format,&describe),
            ModuleKind::AutoShoot=>config::serialize_auto_shoot(&b.auto_shoot,&common,format,&describe),
            ModuleKind::Scaffold=>config::serialize_scaffold(&b.scaffold,&common,format,&describe),
            ModuleKind::Clutch=>config::serialize_clutch(b.clutch.min_fall_distance,&common,format,&describe),
            ModuleKind::FakeLag=>config::serialize_fake_lag(&b.fake_lag,&common,format,&describe),
            ModuleKind::SwordBlock=>config::serialize_sword_block(&b.sword_block.settings(),&common,format,&describe),
            ModuleKind::Aimbot=>config::serialize_aimbot(&b.aimbot,&common,format,&describe),
            ModuleKind::TargetStrafe=>config::serialize_target_strafe(&b.target_strafe.settings,&common,format,&describe),
            ModuleKind::KillAura=>config::serialize_kill_aura(&b.kill_aura,&common,format,&describe),
            ModuleKind::PacketMine=>config::serialize_packet_mine(&b.packet_mine,&common,format,&describe),
            ModuleKind::Hud=>config::serialize_hud(&b.hud,&common,format,&describe),
            ModuleKind::Blink=>config::serialize_blink(&b.blink,&common,format,&describe),
            ModuleKind::AutoDodge=>config::serialize_auto_dodge(&b.auto_dodge.settings,&common,format,&describe),
            ModuleKind::AirPlace=>config::serialize_air_place(&b.air_place,&common,format,&describe),
            ModuleKind::FlagCheck=>config::serialize_flag_check(&b.flag_check.settings(),&common,format,&describe),
            ModuleKind::PacketLogger=>config::serialize_packet_logger(&b.packet_logger.settings(),&common,format,&describe),
            ModuleKind::AntiStaff=>config::serialize_anti_staff(b.anti_staff.show(),&common,format,&describe),
            ModuleKind::Notifier=>config::serialize_notifier(&b.notifier.model.settings,&common,format,&describe),
            ModuleKind::AntiBot=>config::serialize_anti_bot(&b.anti_bot.state,&common,format,&describe),
            ModuleKind::AntiVoid=>config::serialize_anti_void(&b.anti_void.state.lock().expect("AntiVoid"),&common,format,&describe),
            ModuleKind::XRay=>config::serialize_xray(&b.xray.settings,&common,format,&describe),
            ModuleKind::InventoryMove=>config::serialize_inventory_move(&b.inventory_move.settings(),&common,format,&describe),
            ModuleKind::FastUse => config::serialize_fast_use(&b.fast_use,&common,format,&describe),
            ModuleKind::Teams => config::serialize_teams(&b.teams,&common,format,&describe),
            ModuleKind::MurderMystery=>config::serialize_murder_mystery(&b.murder_mystery,&common,format,&describe),
            ModuleKind::Offhand=>config::serialize_offhand(&b.offhand.state.settings,&common,format,&describe),
            ModuleKind::ProjectilePuncher=>config::serialize_projectile_puncher(&b.projectile_puncher,&common,format,&describe),
            ModuleKind::FreeCam=>config::serialize_free_cam(&b.free_cam,&common,format,&describe),
            ModuleKind::Spammer => config::serialize_spammer(&b.spammer.settings,&common,format,&describe),
            ModuleKind::AutoChatGame=>config::serialize_auto_chat_game(&b.auto_chat_game.state.settings,&common,format,&describe),
            ModuleKind::PingSpoof => config::serialize_ping_spoof(&b.ping_spoof,&common,format,&describe),
            ModuleKind::Clip => config::serialize_clip(&b.clip,&common,format,&describe),
            ModuleKind::BookBot => config::serialize_book_bot(&b.book_bot.settings,&common,format,&describe),
            ModuleKind::ChestCleaner => config::serialize_chest_cleaner(&b.chest_cleaner.settings,&common,format,&describe),
            ModuleKind::AutoDeposit => config::serialize_auto_deposit(&b.auto_deposit,&common,format,&describe),
            ModuleKind::Replenish => config::serialize_replenish(&b.replenish.settings,&common,format,&describe),
            ModuleKind::AutoCrafter=>config::serialize_auto_crafter(&b.auto_crafter.settings,&common,format,&describe),
            ModuleKind::InventoryCleaner => config::serialize_inventory_cleaner(&b.inventory_cleaner.settings,&common,format,&describe),
            ModuleKind::AutoArmor => config::serialize_auto_armor(&b.auto_armor.settings,&common,format,&describe),
            ModuleKind::AutoMobHeal => config::serialize_auto_mob_heal(&b.auto_mob_heal.settings,&common,format,&describe),
            ModuleKind::ChestStealer => config::serialize_chest_stealer(&b.chest_stealer,&common,format,&describe),
            ModuleKind::Eagle => config::serialize_eagle(&b.eagle,&common,format,&describe),
            ModuleKind::AntiExploit => config::serialize_anti_exploit(&b.anti_exploit,&common,format,&describe),
            ModuleKind::Sneak => config::serialize_sneak(&b.sneak.settings(),&common,format,&describe),
            ModuleKind::OldRegen => {
                config::serialize_old_regen(&b.old_regen, &common, format, &describe)
            }
            ModuleKind::Plugins => config::serialize_plugins(
                &b.plugins.behavior.lock().expect("Plugins poisoned"),
                &common,
                format,
                &describe,
            ),
            ModuleKind::TrueSight => {
                config::serialize_true_sight(&b.true_sight, &common, format, &describe)
            }
            ModuleKind::NoClip => config::serialize_no_clip(&b.no_clip, &common, format, &describe),
            ModuleKind::PotionSpoof => {
                config::serialize_potion_spoof(&b.potion_spoof, &common, format, &describe)
            }
            ModuleKind::AutoDisable => {
                config::serialize_auto_disable(&b.auto_disable, &common, format, &describe)
            }
            ModuleKind::ResetVl => {
                config::serialize_reset_vl(&b.reset_vl, &common, format, &describe)
            }
            ModuleKind::ItemScroller => {
                config::serialize_item_scroller(&b.item_scroller, &common, format, &describe)
            }
            ModuleKind::GuiCloser => {
                config::serialize_gui_closer(&b.gui_closer, &common, format, &describe)
            }
            ModuleKind::TextFieldProtect => config::serialize_text_field_protect(
                &b.text_field_protect,
                &common,
                format,
                &describe,
            ),
            ModuleKind::AntiHunger => config::serialize_anti_hunger(
                &b.anti_hunger.behavior.lock().expect("AntiHunger poisoned"),
                &common,
                format,
                &describe,
            ),
            ModuleKind::FastBreak => config::serialize_fast_break(
                &b.fast_break.behavior.lock().expect("FastBreak poisoned"),
                &common,
                format,
                &describe,
            ),
            ModuleKind::Strafe => config::serialize_strafe(&b.strafe, &common, format, &describe),
            ModuleKind::Speed => config::serialize_speed(&b.speed.settings, &common, format, &describe),
            ModuleKind::GhostHand => {
                config::serialize_ghost_hand(&b.ghost_hand, &common, format, &describe)
            }
            ModuleKind::ExtendedFirework => config::serialize_extended_firework(
                &b.extended_firework,
                &common,
                format,
                &describe,
            ),
            ModuleKind::BlockWalk => {
                config::serialize_block_walk(&b.block_walk, &common, format, &describe)
            }
            ModuleKind::SilentHotbar => {
                config::serialize_silent_hotbar(&b.silent_hotbar, &common, format, &describe)
            }
            ModuleKind::NoEntityInteract => config::serialize_no_entity_interact(
                &b.no_entity_interact,
                &common,
                format,
                &describe,
            ),
            ModuleKind::AutoFish => config::serialize_auto_fish(
                &b.auto_fish.behavior.lock().expect("AutoFish poisoned"),
                &common,
                format,
                &describe,
            ),
            ModuleKind::AutoRespawn => {
                config::serialize_auto_respawn(&b.auto_respawn, &common, format, &describe)
            }
            ModuleKind::KeepSprint => {
                config::serialize_keep_sprint(&b.keep_sprint, &common, format, &describe)
            }
            ModuleKind::Aspect => config::serialize_aspect(&b.aspect, &common, format, &describe),
            ModuleKind::SmoothCamera => {
                config::serialize_smooth_camera(&b.smooth_camera, &common, format, &describe)
            }
            ModuleKind::QuickPerspectiveSwap => config::serialize_quick_perspective_swap(
                &b.quick_perspective_swap,
                &common,
                format,
                &describe,
            ),
            _ => config::serialize_camera_effect(kind, &common, format, &describe),
        }
        .map_err(|e| e.to_string())
    }
    pub fn file(&self) -> Result<Value, String> {
        Ok(
            json!({"name":"modules","value":MODULES.iter().copied().map(|kind|self.serialize(kind,JsonFormat::File)).collect::<Result<Vec<_>,_>>()?}),
        )
    }
    pub fn global_settings(&self) -> Value {
        let language = horizon_client::lang::code();
        let describe = |key: &str| liquidbounce_interop::localization::describe(&language, key);
        let mut tree = config::serialize_global_targets(
            &self.state.global_targets,
            JsonFormat::Interop,
            &describe,
        );
        tree["value"].as_array_mut().expect("Settings array").push(
            config::serialize_global_inventory(
                &self.state.behaviors.borrow().inventory.manager,
                JsonFormat::Interop,
                &describe,
            ),
        );
        tree["value"].as_array_mut().expect("Settings array").push(config::serialize_global_blink(&self.state.behaviors.borrow().blink_esp,JsonFormat::Interop,&describe));
        tree["value"].as_array_mut().expect("Settings array").push(config::serialize_client_commands(&self.state.behaviors.borrow().client_commands,JsonFormat::Interop,&describe));
        tree["value"].as_array_mut().expect("Settings array").push(config::serialize_global_translate(&self.state.behaviors.borrow().better_chat.translation,JsonFormat::Interop,&describe));
        tree
    }

    pub fn set_global_settings(&mut self, value: &Value) -> Result<(), String> {
        let patch = config::parse_global_targets(value).map_err(|error| error.to_string())?;
        let inventory_patch =
            config::parse_global_inventory(value).map_err(|error| error.to_string())?;
        for issue in patch.issues {
            eprintln!("[LiquidBounce] global config leaf skipped: {issue}");
        }
        let before = self.global_settings();
        {
            let old=config::serialize_global_translate(&self.state.behaviors.borrow().better_chat.translation,JsonFormat::Interop,&str::to_owned);
            for issue in config::apply_global_translate(&mut self.state.behaviors.borrow_mut().better_chat.translation,value){eprintln!("[LiquidBounce] global translator setting skipped: {issue}");}
            self.state.behaviors.borrow_mut().better_chat.provider_changed();
            let language=horizon_client::lang::code();
            let new=config::serialize_global_translate(&self.state.behaviors.borrow().better_chat.translation,JsonFormat::Interop,&|k|liquidbounce_interop::localization::describe(&language,k));
            for leaf in config::global_translate_changes(&old,&new){self.state.events.borrow_mut().push(("valueChanged".into(),json!({"value":leaf})));}
        }
        for issue in config::apply_client_commands(&mut self.state.behaviors.borrow_mut().client_commands,value){eprintln!("[LiquidBounce] global config leaf skipped: {issue}");}
        let after=self.global_settings();
        for name in ["Prefix","HintCount"]{
            let find=|tree:&Value|tree["value"].as_array().and_then(|groups|groups.iter().find(|group|group["name"]=="CommandManager")).and_then(|group|group["value"].as_array()).and_then(|values|values.iter().find(|leaf|leaf["name"]==name)).cloned();
            if find(&before)!=find(&after){if let Some(leaf)=find(&after){self.state.events.borrow_mut().push(("valueChanged".into(),json!({"value":leaf})));}}
        }
        for setting in patch.changes {
            let name = match &setting {
                config::TargetSetting::Combat(_) => "Combat",
                config::TargetSetting::Visual(_) => "Visual",
            };
            match setting.apply(&mut self.state.global_targets) {
                Ok(()) => {
                    let after = self.global_settings();
                    let find = |tree: &Value| {
                        tree["value"][0]["value"]
                            .as_array()
                            .and_then(|values| values.iter().find(|leaf| leaf["name"] == name))
                            .cloned()
                    };
                    if find(&before) != find(&after) {
                        if let Some(leaf) = find(&after) {
                            self.state
                                .events
                                .borrow_mut()
                                .push(("valueChanged".into(), json!({"value":leaf})));
                        }
                    }
                }
                Err(error) => eprintln!("[LiquidBounce] global config leaf skipped: {error}"),
            }
        }
        for issue in inventory_patch.issues {
            eprintln!("[LiquidBounce] global config leaf skipped: {issue}");
        }
        for setting in inventory_patch.changes {
            let name = setting.name();
            setting.apply(&mut self.state.behaviors.borrow_mut().inventory.manager);
            let after = self.global_settings();
            let find = |tree: &Value| {
                tree["value"]
                    .as_array()
                    .and_then(|groups| groups.iter().find(|group| group["name"] == "Inventory"))
                    .and_then(|group| group["value"].as_array())
                    .and_then(|values| values.iter().find(|leaf| leaf["name"] == name))
                    .cloned()
            };
            if find(&before) != find(&after) {
                if let Some(leaf) = find(&after) {
                    self.state
                        .events
                        .borrow_mut()
                        .push(("valueChanged".into(), json!({"value":leaf})));
                }
            }
        }
        let blink_patch=config::parse_global_blink(value).map_err(|e|e.to_string())?;
        for issue in blink_patch.issues{eprintln!("[LiquidBounce] global config leaf skipped: {issue}");}
        for setting in blink_patch.changes {
            let before=config::serialize_global_blink(&self.state.behaviors.borrow().blink_esp,JsonFormat::Interop,&|_|String::new());
            setting.apply(&mut self.state.behaviors.borrow_mut().blink_esp);
            let language=horizon_client::lang::code();
            let after=config::serialize_global_blink(&self.state.behaviors.borrow().blink_esp,JsonFormat::Interop,&|k|liquidbounce_interop::localization::describe(&language,k));
            for leaf in config::blink_changed_values(&before,&after){self.state.events.borrow_mut().push(("valueChanged".into(),json!({"value":leaf})));}
        }
        // Keep imported settings for systems that have not been ported yet.
        // Live Targets and Inventory are exposed by GET.
        if let Some(groups) = value["value"].as_array() {
            let preserved = self.state.global_preserved["value"]
                .as_array_mut()
                .expect("Settings array");
            let mut seen = std::collections::BTreeSet::new();
            for group in groups {
                let Some(name) = group["name"].as_str() else {
                    continue;
                };
                if matches!(name, "Targets" | "Enemies" | "Inventory" | "BlinkManager" | "CommandManager" | "Commands" | "AutoTranslate")
                    || !seen.insert(name.to_owned())
                {
                    continue;
                }
                if let Some(old) = preserved.iter_mut().find(|old| old["name"] == name) {
                    *old = group.clone();
                } else {
                    preserved.push(group.clone());
                }
            }
        }
        Ok(())
    }

    pub fn global_file(&self) -> Value {
        let mut tree = config::serialize_global_targets(
            &self.state.global_targets,
            JsonFormat::File,
            &str::to_owned,
        );
        tree["value"].as_array_mut().expect("Settings array").push(
            config::serialize_global_inventory(
                &self.state.behaviors.borrow().inventory.manager,
                JsonFormat::File,
                &str::to_owned,
            ),
        );
        tree["value"].as_array_mut().expect("Settings array").push(config::serialize_global_blink(&self.state.behaviors.borrow().blink_esp,JsonFormat::File,&str::to_owned));
        if let Some(groups) = self.state.global_preserved["value"].as_array() {
            let actual = tree["value"].as_array_mut().expect("Settings array");
            actual.extend(
                groups
                    .iter()
                    .filter(|group| {
                        !matches!(
                            group["name"].as_str(),
                            Some("Targets" | "Enemies" | "Inventory" | "BlinkManager" | "CommandManager" | "Commands" | "AutoTranslate")
                        )
                    })
                    .cloned(),
            );
        }
        tree["value"].as_array_mut().expect("Settings array").push(config::serialize_client_commands(&self.state.behaviors.borrow().client_commands,JsonFormat::File,&str::to_owned));
        tree["value"].as_array_mut().expect("Settings array").push(config::serialize_global_translate(&self.state.behaviors.borrow().better_chat.translation,JsonFormat::File,&str::to_owned));
        tree
    }

    pub fn save_global(&self, path: &Path) -> Result<(), String> {
        let bytes =
            serde_json::to_vec_pretty(&self.global_file()).map_err(|error| error.to_string())?;
        let temporary = path.with_extension("json.tmp");
        std::fs::write(&temporary, bytes)
            .and_then(|()| std::fs::rename(&temporary, path))
            .map_err(|error| error.to_string())
    }

    pub fn load_global(&mut self, path: &Path) -> Result<(), String> {
        if !path.is_file() {
            return Ok(());
        }
        let value: Value =
            serde_json::from_slice(&std::fs::read(path).map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())?;
        self.set_global_settings(&value)
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let bytes = serde_json::to_vec_pretty(&self.file()?).map_err(|e| e.to_string())?;
        let temp = path.with_extension("json.tmp");
        std::fs::write(&temp, bytes)
            .and_then(|()| std::fs::rename(&temp, path))
            .map_err(|e| e.to_string())
    }
    pub fn load(&mut self, path: &Path) -> Result<(), String> {
        if !path.is_file() {
            return Ok(());
        }
        let value: Value = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let previous = self.state.registry.set_loading(true);
        let result = (|| {
            for group in value["value"]
                .as_array()
                .ok_or("modules.value must be an array")?
            {
                if let Some(name) = group["name"]
                    .as_str()
                    .filter(|name| ModuleKind::from_name(name).is_some())
                {
                    self.set_settings(name, group)?;
                }
            }
            Ok(())
        })();
        self.state.registry.set_loading(previous);
        result
    }
    pub fn actual_modules(&self) -> Vec<Value> {
        self.modules()
            .into_iter()
            .map(|mut value| {
                let kind = ModuleKind::from_name(value["name"].as_str().unwrap()).unwrap();
                value["running"] = json!(if kind==ModuleKind::ClickGui {true} else if kind == ModuleKind::AutoAccount {
                    self.state.registry.enabled_without_world(self.id(kind))
                } else {
                    self.state.registry.running(self.id(kind))
                });
                value
            })
            .collect()
    }
}

impl ModuleAccess for Actor<'_> {
    fn modules(&self) -> Vec<Value> {
        let mut modules: Vec<_> = MODULES.iter().copied().map(|kind|{
        let common=self.common(kind);
        let tag = if kind == ModuleKind::MaceKill {
            Some(self.state.behaviors.borrow().mace_kill.fall_height.to_string())
        } else { match kind {
            ModuleKind::Crosshair => Some(if self.state.behaviors.borrow().crosshair.mode()=="CS2"{"CS2"}else{"Circle"}),
            ModuleKind::JumpEffect=>Some(if self.state.behaviors.borrow().jump_effect.active()=="Simple"{"Simple"}else{"Image"}),
            ModuleKind::TotemEffect=>Some(if self.state.behaviors.borrow().totem_effect.active()=="Shockwave"{"Shockwave"}else{"Soul"}),
            ModuleKind::Wings => Some("Lines"),
            ModuleKind::Hats => Some(liquidbounce_behaviors::hats::MODES[self.state.behaviors.borrow().hats.mode]),
            ModuleKind::FakeLag=>Some(if self.state.behaviors.borrow().fake_lag.settings().constant{"Constant"}else{"Dynamic"}),
            ModuleKind::Scaffold=>Some(match self.state.behaviors.borrow().scaffold.tag("Technique"){"Expand"=>"Expand","GodBridge"=>"GodBridge","Breezily"=>"Breezily",_=>"Normal"}),
            ModuleKind::AutoShoot=>Some(["Auto","Linear","Projectile"][usize::from(self.state.behaviors.borrow().auto_shoot.behavior.settings.gravity.min(2))]),
            ModuleKind::AntiVoid=>Some(liquidbounce_behaviors::anti_void::MODES[self.state.behaviors.borrow().anti_void.state.lock().expect("AntiVoid").settings.mode]),
            ModuleKind::AntiBot=>Some(liquidbounce_behaviors::anti_bot::MODES[self.state.behaviors.borrow().anti_bot.state.settings.mode]),
            ModuleKind::InventoryMove=>Some(liquidbounce_behaviors::inventory_move::BEHAVIORS[self.state.behaviors.borrow().inventory_move.settings().behavior as usize]),
            ModuleKind::AutoQueue=>Some(liquidbounce_behaviors::auto_queue::PRESETS[self.state.behaviors.borrow().auto_queue.settings.preset]),
            ModuleKind::Timer => Some(self.state.behaviors.borrow().timer.state.lock().expect("Timer poisoned").settings.mode.tag()),
            ModuleKind::HighJump => Some(self.state.behaviors.borrow().high_jump.settings.mode.tag()),
            ModuleKind::NoWeb => Some(self.state.behaviors.borrow().no_web.behavior.settings.mode.tag()),
            ModuleKind::LiquidWalk => Some(self.state.behaviors.borrow().liquid_walk.settings().mode.tag()),
            ModuleKind::Step => Some(self.state.behaviors.borrow().step.settings().mode.tag()),
            ModuleKind::ReverseStep => Some(self.state.behaviors.borrow().reverse_step.mode.tag()),
            ModuleKind::Freeze => Some(self.state.behaviors.borrow().freeze.settings().mode.tag()),
            ModuleKind::AutoWalk => Some(match self.state.behaviors.borrow().auto_walk.mode.as_str() {
                "Direction" => "Direction", "Goto" => "Goto", "Follow" => "Follow", _ => "Forward",
            }),
            ModuleKind::BookBot => Some(if self.state.behaviors.borrow().book_bot.settings.mode==0 {"Random"}else{"File"}),
            ModuleKind::Sneak => Some(self.state.behaviors.borrow().sneak.settings().tag()),
            ModuleKind::FastUse => Some(self.state.behaviors.borrow().fast_use.settings.mode.tag()),
            ModuleKind::Offhand=>Some(self.state.behaviors.borrow().offhand.state.active.name()),
            ModuleKind::SuperKnockback => Some(self.state.behaviors.borrow().super_knockback.settings.mode.tag()),
            _ => None,
        }.map(str::to_owned) };
            json!({"name":kind.name(),"category":kind.category(),"keyBind":common.key_bind,"enabled":common.enabled,"hidden":common.hidden,"description":description(kind),"aliases":match kind { ModuleKind::Aimbot=>vec!["AimAssist","AutoAim"], ModuleKind::AutoMobHeal=>vec!["AutoGolemRepair"], ModuleKind::FlagCheck=>vec!["FlagDetect"], ModuleKind::Offhand=>vec!["AutoTotem"], ModuleKind::ProjectilePuncher=>vec!["AntiFireball"], ModuleKind::MoreCarry=>vec!["XCarry"], ModuleKind::AntiBlind=>vec!["NoRender"], ModuleKind::NoPose=>vec!["NoSwim","NoCrawl"], ModuleKind::MultiActions=>vec!["MultiTask"], ModuleKind::SleepWalker=>vec!["BedGodMode"], ModuleKind::SnapTap=>vec!["NullMove","SOCD"], ModuleKind::TerrainSpeed=>vec!["FastClimb"], ModuleKind::ElytraSwap=>vec!["ChestSwap"], ModuleKind::GuiCloser=>vec!["AutoClose","ContainerCloser"], ModuleKind::AutoAccount=>vec!["AutoLogin","AutoRegister"], ModuleKind::VehicleControl=>vec!["BoatFly"], ModuleKind::ReportHelper=>vec!["AutoReport"], ModuleKind::AutoQueue=>vec!["AutoPlay"], ModuleKind::MiddleClickAction=>vec!["FriendClicker","MiddleClickPearl"], ModuleKind::SuperKnockback=>vec!["WTap"], ModuleKind::SpearKill=>vec!["AutoSpear"], ModuleKind::Spider=>vec!["WallClimb"], ModuleKind::OldRegen=>vec!["Regen","Zoot"], ModuleKind::FastUse=>vec!["FastEat"], _=>vec![] },"tag":tag})
    }).collect();
        // ModuleManager iterates its ObjectRBTreeSet(VALUE_NAME_ORDER).
        // All registered builtin names here are ASCII, matching that ordering.
        modules.sort_by_cached_key(|module| {
            module["name"]
                .as_str()
                .expect("registered module has a name")
                .to_ascii_lowercase()
        });
        modules
    }
    fn settings(&self, name: &str) -> Option<Value> {
        self.serialize(ModuleKind::from_name(name)?, JsonFormat::Interop)
            .ok()
    }
    fn set_enabled(&mut self, name: &str, enabled: bool) -> Result<(), String> {
        let kind = ModuleKind::from_name(name).ok_or("Unknown module")?;
        let id = self.id(kind);
        let report = self
            .state
            .with_registry(self.options, |registry, callbacks| {
                registry.set_enabled_sync(id, enabled, callbacks)
            })
            .map_err(|e| format!("{e:?}"))?;
        if report.failures.is_empty() {
            Ok(())
        } else {
            Err(format!("{:?}", report.failures))
        }
    }
    fn set_settings(&mut self, name: &str, value: &Value) -> Result<(), String> {
        let kind = ModuleKind::from_name(name).ok_or("Unknown module")?;
        let name = kind.name();
        let id = self.id(kind);
        let patch = config::parse_settings(kind, value).map_err(|e| e.to_string())?;
        for issue in patch.issues {
            eprintln!("[LiquidBounce] config leaf skipped: {issue}");
        }
        for change in patch.changes {
            let result = {
                let mut b = self.state.behaviors.borrow_mut();
                let b = &mut *b;
                config::apply_setting(&change, &mut b.no_fov, &mut b.zoom, &mut b.full_bright)
            };
            match result {
                ApplyResult::Runtime(RuntimeRequest::Enabled(value)) => {
                    self.set_enabled(name, value)?
                }
                ApplyResult::Runtime(RuntimeRequest::Bind(value)) => self
                    .state
                    .registry
                    .set_bind(id, Some(parse_bind(&value)?))
                    .map_err(|e| format!("{e:?}"))?,
                ApplyResult::Runtime(RuntimeRequest::Hidden(value)) => {
                    self.state.hidden.insert(name.into(), value);
                }
                ApplyResult::Runtime(RuntimeRequest::NoPose(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().no_pose);
                }
                ApplyResult::Runtime(RuntimeRequest::NoRotateSet(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().no_rotate_set);
                }
                ApplyResult::Runtime(RuntimeRequest::AntiAfk(setting)) => {
                    if let Err(error) =
                        setting.apply(&mut self.state.behaviors.borrow_mut().anti_afk)
                    {
                        eprintln!("[LiquidBounce] AntiAFK config leaf skipped: {error}");
                    }
                }
                ApplyResult::Runtime(RuntimeRequest::TridentBoost(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().trident_boost);
                }
                ApplyResult::Runtime(RuntimeRequest::BlockBounce(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().block_bounce);
                }
                ApplyResult::Runtime(RuntimeRequest::SkinDerp(setting)) => {
                    if let Err(error) =
                        setting.apply(&mut self.state.behaviors.borrow_mut().skin_derp)
                    {
                        eprintln!("[LiquidBounce] SkinDerp config leaf skipped: {error}");
                    }
                }
                ApplyResult::Runtime(RuntimeRequest::ExtendedFirework(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().extended_firework);
                }
                ApplyResult::Runtime(RuntimeRequest::Strafe(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().strafe);
                }
                ApplyResult::Runtime(RuntimeRequest::Speed(setting)) => {
                    let mut b = self.state.behaviors.borrow_mut();
                    if let Err(error) = setting.apply(&mut b.speed.settings) {
                        eprintln!("[LiquidBounce] Speed config leaf skipped: {error}");
                    }
                }
                ApplyResult::Runtime(RuntimeRequest::ElytraFly(value)) => {
                    self.state.behaviors.borrow_mut().elytra_fly = value;
                }
                ApplyResult::Runtime(RuntimeRequest::NoFall(value)) => {
                    self.state.behaviors.borrow_mut().no_fall = value;
                }
                ApplyResult::Runtime(RuntimeRequest::Timer(setting)) => {
                    self.state
                        .behaviors
                        .borrow()
                        .timer
                        .update_settings(|settings| {
                            setting.apply(settings);
                        });
                }
                ApplyResult::Runtime(RuntimeRequest::BetterInventory(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().better_inventory);
                }
                ApplyResult::Runtime(RuntimeRequest::TerrainSpeed(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().terrain_speed);
                }
                ApplyResult::Runtime(RuntimeRequest::ReverseStep(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().reverse_step); }
                ApplyResult::Runtime(RuntimeRequest::Step(setting)) => {self.state.behaviors.borrow().step.update(|s|setting.apply(s));}
                ApplyResult::Runtime(RuntimeRequest::LiquidWalk(setting)) => {self.state.behaviors.borrow().liquid_walk.update(|s|setting.apply(s));}
                ApplyResult::Runtime(RuntimeRequest::NoWeb(setting)) => {setting.apply(&mut self.state.behaviors.borrow_mut().no_web);}
                ApplyResult::Runtime(RuntimeRequest::NoSlow(setting)) => {self.state.behaviors.borrow().no_slow.update(|s|setting.apply(s));}
                ApplyResult::Runtime(RuntimeRequest::HighJump(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().high_jump);
                }
                ApplyResult::Runtime(RuntimeRequest::AutoWalk(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().auto_walk);
                }
                ApplyResult::Runtime(RuntimeRequest::Parkour(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().parkour); }
                ApplyResult::Runtime(RuntimeRequest::SafeWalk(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().safe_walk); }
                ApplyResult::Runtime(RuntimeRequest::Anchor(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().anchor); }
                ApplyResult::Runtime(RuntimeRequest::AvoidHazards(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().avoid_hazards); }
                ApplyResult::Runtime(RuntimeRequest::Freeze(setting)) => { self.state.behaviors.borrow().freeze.update(|s|setting.apply(s)); }
                ApplyResult::Runtime(RuntimeRequest::OneShot(setting)) => {
                    use config::OneShotSetting;
                    let mut modules = self.state.behaviors.borrow_mut();
                    match setting {
                        OneShotSetting::Kick(mode) => modules.kick_mode = mode,
                        OneShotSetting::Health(value) => modules.auto_leave.settings.health = value,
                        OneShotSetting::Delay(value) => modules.auto_leave.settings.delay = value,
                        OneShotSetting::LeaveMode(mode) => modules.auto_leave.settings.mode = mode,
                    }
                }
                ApplyResult::Runtime(RuntimeRequest::TargetLock(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().target_lock);
                }
                ApplyResult::Runtime(RuntimeRequest::YggdrasilSignatureFix(value)) => {
                    let mut modules = self.state.behaviors.borrow_mut();
                    modules.yggdrasil_force_slim = value;
                    modules.sync_packet_modules(&self.state.registry);
                }
                ApplyResult::Runtime(RuntimeRequest::AutoDisable(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().auto_disable);
                }
                ApplyResult::Runtime(RuntimeRequest::Disabler(_))
                | ApplyResult::Runtime(RuntimeRequest::ServerCrasher(_))
                | ApplyResult::Runtime(RuntimeRequest::Dupe(_))
                | ApplyResult::Runtime(RuntimeRequest::TeleportReach(_))
                | ApplyResult::Runtime(RuntimeRequest::Phase(_))
                | ApplyResult::Runtime(RuntimeRequest::Notebot(_))
                | ApplyResult::Runtime(RuntimeRequest::AutoConfig(_))
                | ApplyResult::Runtime(RuntimeRequest::DebugRecorder(_))
                | ApplyResult::Runtime(RuntimeRequest::SkinChanger(_)) => {}
                ApplyResult::Runtime(RuntimeRequest::PotionSpoof(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().potion_spoof);
                }
                ApplyResult::Runtime(RuntimeRequest::NoClip(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().no_clip);
                }
                ApplyResult::Runtime(RuntimeRequest::TrueSight(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().true_sight);
                }
                ApplyResult::Runtime(RuntimeRequest::Plugins(value)) => {
                    self.state
                        .behaviors
                        .borrow()
                        .plugins
                        .behavior
                        .lock()
                        .expect("Plugins poisoned")
                        .try_methods = value;
                }
                ApplyResult::Runtime(RuntimeRequest::Clip(setting)) => {let mut b=self.state.behaviors.borrow_mut();let old=b.clip.mode;setting.apply(&mut b.clip);if old!=b.clip.mode&&b.clip.mode==1&&b.query_running(&self.state.registry,"Clip"){let settings=b.clip;b.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::Clip(settings));}}
                ApplyResult::Runtime(RuntimeRequest::Sneak(setting)) => {
                    let mut b=self.state.behaviors.borrow_mut();let old=b.sneak.settings().mode;
                    setting.apply(&mut b.sneak.settings.lock().expect("Sneak settings"));
                    if old!=b.sneak.settings().mode&&b.sneak.settings().protocol_warning()&&b.query_running(&self.state.registry,"Sneak"){b.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::SneakSwitchWarn);}
                }
                ApplyResult::Runtime(RuntimeRequest::AntiExploit(setting)) => {
                    let mut b=self.state.behaviors.borrow_mut(); setting.apply(&mut b.anti_exploit); b.sync_packet_modules(&self.state.registry);
                }
                ApplyResult::Runtime(RuntimeRequest::Eagle(setting)) => {setting.apply(&mut self.state.behaviors.borrow_mut().eagle);}
                ApplyResult::Runtime(RuntimeRequest::ChestStealer(setting))=>{
                    if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().chest_stealer,&self.state.item_registry,&self.state.block_registry){eprintln!("[LiquidBounce] ChestStealer config: {e}");}
                }
                ApplyResult::Runtime(RuntimeRequest::AutoMobHeal(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_mob_heal.settings){eprintln!("[LiquidBounce] AutoMobHeal config: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::AutoArmor(setting))=>{
                    let mut b=self.state.behaviors.borrow_mut();
                    if let Err(e)=setting.apply(&mut b.auto_armor.settings){eprintln!("[LiquidBounce] AutoArmor config: {e}");}
                    b.item_policy.armor=b.auto_armor.settings.policy();
                }
                ApplyResult::Runtime(RuntimeRequest::InventoryCleaner(setting))=>{
                    if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().inventory_cleaner.settings,&self.state.item_registry){eprintln!("[LiquidBounce] InventoryCleaner config: {e}");}
                }
                ApplyResult::Runtime(RuntimeRequest::AutoCrafter(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_crafter.settings,&self.state.item_registry){eprintln!("[LiquidBounce] AutoCrafter config: {e}");}},
                ApplyResult::Runtime(RuntimeRequest::Replenish(setting))=>{
                    if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().replenish.settings){eprintln!("[LiquidBounce] Replenish config: {e}");}
                }
                ApplyResult::Runtime(RuntimeRequest::AutoDeposit(setting))=>{
                    if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_deposit,&self.state.item_registry,&self.state.block_registry){eprintln!("[LiquidBounce] AutoDeposit config: {e}");}
                }
                ApplyResult::Runtime(RuntimeRequest::ChestCleaner(setting)) => {
                    if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().chest_cleaner.settings,&self.state.item_registry){eprintln!("[LiquidBounce] ChestCleaner config: {e}");}
                }
                ApplyResult::Runtime(RuntimeRequest::BookBot(setting)) => {
                    if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().book_bot.settings){eprintln!("[LiquidBounce] BookBot config: {e}");}
                }
                ApplyResult::Runtime(RuntimeRequest::PingSpoof(setting)) => { setting.apply(&self.state.behaviors.borrow().ping_spoof); }
                ApplyResult::Runtime(RuntimeRequest::AutoChatGame(setting))=>{if let Err(error)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_chat_game.state.settings){eprintln!("[LiquidBounce] AutoChatGame setting: {error}");}}
                ApplyResult::Runtime(RuntimeRequest::Spammer(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().spammer.settings); }
                ApplyResult::Runtime(RuntimeRequest::ProjectilePuncher(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().projectile_puncher){eprintln!("[LiquidBounce] ProjectilePuncher config: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::FreeCam(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().free_cam){eprintln!("[LiquidBounce] FreeCam config: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::Offhand(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().offhand.state.settings){eprintln!("[LiquidBounce] Offhand config: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::MurderMystery(setting))=>{let mut b=self.state.behaviors.borrow_mut();setting.clone().apply(&mut b.murder_mystery);b.auto_dodge.murder_mystery_disallows=b.murder_mystery.disallows_arrow_dodge(b.query_running(&self.state.registry,"MurderMystery"));b.sync_packet_modules(&self.state.registry);}
                ApplyResult::Runtime(RuntimeRequest::Teams(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().teams); }
                ApplyResult::Runtime(RuntimeRequest::InventoryMove(setting))=>{let b=self.state.behaviors.borrow();b.inventory_move.update(|s|{if let Err(e)=setting.apply(s){eprintln!("[LiquidBounce] InventoryMove config: {e}");}});b.sync_packet_modules(&self.state.registry);}
                ApplyResult::Runtime(RuntimeRequest::XRay(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().xray.settings){eprintln!("[LiquidBounce] XRay config: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::AntiVoid(setting))=>{let service=self.state.behaviors.borrow().anti_void.clone();if let Err(e)=setting.apply(&mut service.state.lock().expect("AntiVoid"),&self.state.item_registry){eprintln!("[LiquidBounce] AntiVoid configuration: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::AntiBot(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().anti_bot.state){eprintln!("[LiquidBounce] AntiBot configuration: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::Notifier(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().notifier.model.settings){eprintln!("[LiquidBounce] Notifier configuration: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::AntiStaff(setting))=>{self.state.behaviors.borrow().anti_staff.set_show(setting.0);}
                ApplyResult::Runtime(RuntimeRequest::PacketLogger(setting))=>{self.state.behaviors.borrow().packet_logger.edit(|s|{if let Err(e)=setting.apply(s){eprintln!("[LiquidBounce] PacketLogger config: {e}");}});}
                ApplyResult::Runtime(RuntimeRequest::FlagCheck(setting))=>{let state=&self.state;let b=state.behaviors.borrow();b.flag_check.edit(|s|{if let Err(e)=setting.apply(s){eprintln!("[LiquidBounce] FlagCheck setting: {e}");}});b.flag_check.configure(b.query_running(&state.registry,"FlagCheck"));}
                ApplyResult::Runtime(RuntimeRequest::AirPlace(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().air_place){eprintln!("[LiquidBounce] AirPlace setting: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::AutoDodge(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_dodge.settings){eprintln!("[LiquidBounce] AutoDodge setting: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::Blink(setting))=>{self.state.behaviors.borrow().blink.update(|s|{if let Err(e)=setting.apply(s){eprintln!("[LiquidBounce] Blink setting: {e}");}});}
                ApplyResult::Runtime(RuntimeRequest::AutoWindCharge(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_wind_charge){eprintln!("[LiquidBounce] AutoWindCharge config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::PacketMine(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().packet_mine){eprintln!("[LiquidBounce] PacketMine config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::Hud(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().hud){eprintln!("[LiquidBounce] HUD config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::KillAura(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().kill_aura){eprintln!("[LiquidBounce] KillAura config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::TpAura(value)) => {
                    self.state.behaviors.borrow_mut().tp_aura = value;
                }
                ApplyResult::Runtime(RuntimeRequest::Velocity(value)) => {
                    self.state.behaviors.borrow_mut().velocity = value;
                }
                ApplyResult::Runtime(RuntimeRequest::TargetStrafe(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().target_strafe.settings){eprintln!("[LiquidBounce] TargetStrafe config: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::Aimbot(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().aimbot){eprintln!("[LiquidBounce] Aimbot config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::AutoRod(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_rod,&self.state.item_registry){eprintln!("[LiquidBounce] AutoRod config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::SwordBlock(setting))=>{let service=self.state.behaviors.borrow().sword_block.clone();let mut settings=service.settings();setting.apply(&mut settings);service.configure(settings);}
                ApplyResult::Runtime(RuntimeRequest::FakeLag(setting))=>{self.state.behaviors.borrow().fake_lag.update(|s|{if let Err(e)=setting.apply(s){eprintln!("[LiquidBounce] FakeLag setting: {e}");}});}
                ApplyResult::Runtime(RuntimeRequest::Scaffold(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().scaffold,&self.state.block_registry){eprintln!("[LiquidBounce] Scaffold config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::Clutch(setting))=>{self.state.behaviors.borrow_mut().clutch.min_fall_distance=setting.0;}
                ApplyResult::Runtime(RuntimeRequest::AutoShoot(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_shoot,&self.state.item_registry){eprintln!("[LiquidBounce] AutoShoot config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::ElytraTarget(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().elytra_target){eprintln!("[LiquidBounce] ElytraTarget config: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::AutoBow(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_bow){eprintln!("[LiquidBounce] AutoBow config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::EasyPearl(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().easy_pearl){eprintln!("[LiquidBounce] EasyPearl config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::BedDefender(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().bed_defender,&self.state.block_registry){eprintln!("[LiquidBounce] BedDefender config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::Surround(setting))=>{setting.apply(&mut self.state.behaviors.borrow_mut().surround,&self.state.block_registry).ok();}
                ApplyResult::Runtime(RuntimeRequest::BlockIn(setting))=>{setting.apply(&mut self.state.behaviors.borrow_mut().block_in,&self.state.block_registry).ok();}
                ApplyResult::Runtime(RuntimeRequest::AutoShop(value))=>{self.state.behaviors.borrow_mut().auto_shop.apply(value);}
                ApplyResult::Runtime(RuntimeRequest::AutoBuild(value))=>{self.state.behaviors.borrow_mut().auto_build.apply(value);}
                ApplyResult::Runtime(RuntimeRequest::AutoFarm(value))=>{self.state.behaviors.borrow_mut().auto_farm.apply(value);}
                ApplyResult::Runtime(RuntimeRequest::CrystalAura(value))=>{self.state.behaviors.borrow_mut().crystal_aura.apply(value);}
                ApplyResult::Runtime(RuntimeRequest::Fucker(value))=>{self.state.behaviors.borrow_mut().fucker.apply(value);}
                ApplyResult::Runtime(RuntimeRequest::AutoTrap(value))=>{self.state.behaviors.borrow_mut().auto_trap.apply(value);}
                ApplyResult::Runtime(RuntimeRequest::BlockTrap(value))=>{self.state.behaviors.borrow_mut().block_trap.apply(value);}
                ApplyResult::Runtime(RuntimeRequest::Nuker(value))=>{self.state.behaviors.borrow_mut().nuker.apply(value);}
                ApplyResult::Runtime(RuntimeRequest::TickBase(value))=>{self.state.behaviors.borrow_mut().tick_base.apply(value);}
                ApplyResult::Runtime(RuntimeRequest::Backtrack(value))=>{self.state.behaviors.borrow_mut().backtrack.apply(value);}
                ApplyResult::Runtime(RuntimeRequest::Fly(value))=>{self.state.behaviors.borrow_mut().fly.apply(value);}
                ApplyResult::Runtime(RuntimeRequest::LongJump(value))=>{self.state.behaviors.borrow_mut().long_jump.apply(value);}
                ApplyResult::Runtime(RuntimeRequest::HoleFiller(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().hole_filler,&self.state.block_registry){eprintln!("[LiquidBounce] HoleFiller config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::LiquidFiller(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().liquid_filler,&self.state.block_registry){eprintln!("[LiquidBounce] LiquidFiller config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::Extinguish(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().extinguish){eprintln!("[LiquidBounce] Extinguish config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::AutoPearl(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_pearl){eprintln!("[LiquidBounce] AutoPearl config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::AutoBuff(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_buff){eprintln!("[LiquidBounce] AutoBuff config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::FastExp(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().fast_exp){eprintln!("[LiquidBounce] FastExp config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::FastUse(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().fast_use); }
                ApplyResult::Runtime(RuntimeRequest::OldRegen(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().old_regen);
                }
                ApplyResult::Runtime(RuntimeRequest::Spider(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().spider);
                }
                ApplyResult::Runtime(RuntimeRequest::Criticals(setting))=>{
                    self.state.behaviors.borrow().criticals.update(|s|{if let Err(error)=setting.apply(s){eprintln!("[LiquidBounce] Criticals config leaf skipped: {error}");}});
                }
                ApplyResult::Runtime(RuntimeRequest::CrosshairClicker(setting)) => {
                    if let Err(error)=setting.apply(&mut self.state.behaviors.borrow_mut().crosshair_clicker,&self.state.item_registry,&self.state.block_registry){eprintln!("[LiquidBounce] config leaf skipped: {error}");}
                }
                ApplyResult::Runtime(RuntimeRequest::SpearKill(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().spear_kill);
                }
                ApplyResult::Runtime(RuntimeRequest::MaceKill(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().mace_kill);
                }
                ApplyResult::Runtime(RuntimeRequest::SuperKnockback(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().super_knockback);
                }
                ApplyResult::Runtime(RuntimeRequest::MiddleClickAction(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().middle_click_action);
                }
                ApplyResult::Runtime(RuntimeRequest::Macros(setting))=>{
                    if let Err(error)=setting.clone().apply(&mut self.state.behaviors.borrow_mut().macros,&self.state.item_registry){eprintln!("[LiquidBounce] Macros config leaf skipped: {error}");}
                }
                ApplyResult::Runtime(RuntimeRequest::AutoQueue(setting))=>{
                    let mut b=self.state.behaviors.borrow_mut();let running=b.query_running(&self.state.registry,"AutoQueue");let old=b.auto_queue.settings.preset;
                    if let Err(error)=setting.clone().apply(&mut b.auto_queue,&self.state.item_registry){eprintln!("[LiquidBounce] AutoQueue config leaf skipped: {error}");}
                    if old!=b.auto_queue.settings.preset{if old==1{b.pending_slot_resets.push(crate::auto_queue_runtime::GOMME_REQUESTER);b.auto_queue.reset_gomme=false;}if running&&b.auto_queue.settings.preset==1{b.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::AutoQueueEnabled);}}
                }
                ApplyResult::Runtime(RuntimeRequest::ReportHelper(setting))=>{
                    if let Err(error)=setting.clone().apply(&mut self.state.behaviors.borrow_mut().report_helper){eprintln!("[LiquidBounce] ReportHelper config leaf skipped: {error}");}
                }
                ApplyResult::Runtime(RuntimeRequest::AutoAccount(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().auto_account);
                }
                ApplyResult::Runtime(RuntimeRequest::MobOwners(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().mob_owners);
                }
                ApplyResult::Runtime(RuntimeRequest::CombineMobs(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().combine_mobs);
                }
                ApplyResult::Runtime(RuntimeRequest::BetterTab(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().better_tab);
                }
                ApplyResult::Runtime(RuntimeRequest::BlockOutline(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().block_outline); }
                ApplyResult::Runtime(RuntimeRequest::Breadcrumbs(setting)) => { setting.clone().apply(&mut self.state.behaviors.borrow_mut().breadcrumbs); }
                ApplyResult::Runtime(RuntimeRequest::DamageParticles(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().damage_particles); }
                ApplyResult::Runtime(RuntimeRequest::Esp(setting))=>setting.apply(&mut self.state.behaviors.borrow_mut().esp),
                ApplyResult::Runtime(RuntimeRequest::Tracers(setting)) => { setting.clone().apply(&mut self.state.behaviors.borrow_mut().tracers); }
                ApplyResult::Runtime(RuntimeRequest::Radar(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().radar){eprintln!("[LiquidBounce] Radar config: {e}");}},
                ApplyResult::Runtime(RuntimeRequest::ItemEsp(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().item_esp,&self.state.item_registry){eprintln!("[LiquidBounce] ItemESP config: {e}");}},
                ApplyResult::Runtime(RuntimeRequest::ItemChams(setting))=>setting.apply(&mut self.state.behaviors.borrow_mut().item_chams),
                ApplyResult::Runtime(RuntimeRequest::ItemTags(v))=>{if let Err(e)=v.apply(&mut self.state.behaviors.borrow_mut().item_tags.settings,&self.state.item_registry){eprintln!("[LiquidBounce] ItemTags config: {e}");}},
                ApplyResult::Runtime(RuntimeRequest::Nametags(v))=>{if let Err(e)=v.apply(&mut self.state.behaviors.borrow_mut().nametags.settings,&[]){eprintln!("[LiquidBounce] Nametags config: {e}");}},
                ApplyResult::Runtime(RuntimeRequest::BetterChat(v))=>{if let Err(e)=v.apply(&mut self.state.behaviors.borrow_mut().better_chat.settings){eprintln!("[LiquidBounce] BetterChat config: {e}");}},
                ApplyResult::Runtime(RuntimeRequest::Debug(v))=>{if let Err(e)=v.apply(&mut self.state.behaviors.borrow_mut().debug.model){eprintln!("[LiquidBounce] Debug config: {e}");}},
                ApplyResult::Runtime(RuntimeRequest::ClickGui(v))=>{
                    let change=v.apply(&mut self.state.behaviors.borrow_mut().click_gui);
                    if change.scale {let scale=self.state.behaviors.borrow().click_gui.settings.scale;self.state.events.borrow_mut().push(("clickGuiScaleChange".into(),json!({"value":scale})));}
                    if change.values {if let Some(tree)=self.settings("ClickGUI"){self.state.events.borrow_mut().push(("clickGuiValueChange".into(),json!({"configurable":tree})));}}
                },
                ApplyResult::Runtime(RuntimeRequest::BedPlates(v))=>{if let Err(e)=v.apply(&mut self.state.behaviors.borrow_mut().bed_plates,&self.state.block_registry){eprintln!("[LiquidBounce] BedPlates config: {e}");}},
                ApplyResult::Runtime(RuntimeRequest::NewChunks(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().new_chunks); }
                ApplyResult::Runtime(RuntimeRequest::StrongholdFinder(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().stronghold_finder){eprintln!("[LiquidBounce] StrongholdFinder config: {e}");}},
                ApplyResult::Runtime(RuntimeRequest::ProphuntEsp(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().prophunt_esp); }
                ApplyResult::Runtime(RuntimeRequest::VoidEsp(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().void_esp); }
                ApplyResult::Runtime(RuntimeRequest::ProtectionZones(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().protection_zones); }
                ApplyResult::Runtime(RuntimeRequest::HoleEsp(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().hole_esp); }
                ApplyResult::Runtime(RuntimeRequest::JumpEffect(setting))=>setting.apply(&mut self.state.behaviors.borrow_mut().jump_effect),
                ApplyResult::Runtime(RuntimeRequest::HitFx(setting))=>setting.apply(&mut self.state.behaviors.borrow_mut().hit_fx.settings),
                ApplyResult::Runtime(RuntimeRequest::TotemEffect(setting))=>setting.apply(&mut self.state.behaviors.borrow_mut().totem_effect),
                ApplyResult::Runtime(RuntimeRequest::PotionFX(setting))=>setting.apply(&mut self.state.behaviors.borrow_mut().potion_fx),
                ApplyResult::Runtime(RuntimeRequest::TNTTimer(setting))=>setting.apply(&mut self.state.behaviors.borrow_mut().tnt_timer),
                ApplyResult::Runtime(RuntimeRequest::CustomAmbience(setting))=>setting.apply(&mut self.state.behaviors.borrow_mut().custom_ambience),
                ApplyResult::Runtime(RuntimeRequest::Trajectories(setting))=>setting.apply(&mut self.state.behaviors.borrow_mut().trajectories),
                ApplyResult::Runtime(RuntimeRequest::Particles(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().particles.settings); }
                ApplyResult::Runtime(RuntimeRequest::NameProtect(setting)) => { setting.apply(&self.state.behaviors.borrow().name_protect); }
                ApplyResult::Runtime(RuntimeRequest::InventoryTracker(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().inventory_tracker); }
                ApplyResult::Runtime(RuntimeRequest::LogoffSpot(setting)) => { self.state.behaviors.borrow().logoff_spot.set_send_in_chat(setting.0); }
                ApplyResult::Runtime(RuntimeRequest::StorageEsp(setting))=>{setting.apply(&mut self.state.behaviors.borrow_mut().storage_esp);}
                ApplyResult::Runtime(RuntimeRequest::BlockEsp(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().block_esp,&self.state.block_registry){eprintln!("[LiquidBounce] BlockESP config: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::Crosshair(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().crosshair); }
                ApplyResult::Runtime(RuntimeRequest::Wings(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().wings); }
                ApplyResult::Runtime(RuntimeRequest::Hats(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().hats); }
                ApplyResult::Runtime(RuntimeRequest::AutoAccount(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().auto_account); }
                ApplyResult::Runtime(RuntimeRequest::AutoDisable(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().auto_disable); }
                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().damage);
                }
                ApplyResult::Runtime(RuntimeRequest::ResetVl(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().reset_vl);
                }
                ApplyResult::Runtime(RuntimeRequest::ItemScroller(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().item_scroller);
                }
                ApplyResult::Runtime(RuntimeRequest::GuiCloser(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().gui_closer);
                }
                ApplyResult::Runtime(RuntimeRequest::TextFieldProtect(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().text_field_protect);
                }
                ApplyResult::Runtime(RuntimeRequest::AntiHunger(setting)) => {
                    setting.apply(
                        &mut self
                            .state
                            .behaviors
                            .borrow()
                            .anti_hunger
                            .behavior
                            .lock()
                            .expect("AntiHunger poisoned"),
                    );
                }
                ApplyResult::Runtime(RuntimeRequest::FastBreak(setting)) => {
                    setting.apply(
                        &mut self
                            .state
                            .behaviors
                            .borrow()
                            .fast_break
                            .behavior
                            .lock()
                            .expect("FastBreak poisoned"),
                    );
                }
                ApplyResult::Runtime(RuntimeRequest::Vomit(setting)) => {
                    if let Err(error) = setting.apply(&mut self.state.behaviors.borrow_mut().vomit)
                    {
                        eprintln!("[LiquidBounce] Vomit config leaf skipped: {error}");
                    }
                }
                ApplyResult::Runtime(RuntimeRequest::ElytraSwap(setting)) => {
                    if let Err(error) =
                        setting.apply(&mut self.state.behaviors.borrow_mut().elytra_swap)
                    {
                        eprintln!("[LiquidBounce] ElytraSwap config leaf skipped: {error}");
                    }
                }
                ApplyResult::Runtime(RuntimeRequest::GhostHand(setting)) => {
                    if let Err(error) = setting.apply(
                        &mut self.state.behaviors.borrow_mut().ghost_hand,
                        &self.state.block_registry,
                    ) {
                        eprintln!("[LiquidBounce] GhostHand config leaf skipped: {error}");
                    }
                }
                ApplyResult::Runtime(RuntimeRequest::BlockWalk(setting)) => {
                    if let Err(error) = setting.apply(
                        &mut self.state.behaviors.borrow_mut().block_walk,
                        &self.state.block_registry,
                    ) {
                        eprintln!("[LiquidBounce] BlockWalk config leaf skipped: {error}");
                    }
                }
                ApplyResult::Runtime(RuntimeRequest::EntityControl(setting)) => {
                    if let Err(error) =
                        setting.apply(&mut self.state.behaviors.borrow_mut().entity_control)
                    {
                        eprintln!("[LiquidBounce] EntityControl config leaf skipped: {error}");
                    }
                }
                ApplyResult::Runtime(RuntimeRequest::SmartEat(setting)) => {
                    use config::SmartEatSetting;
                    use liquidbounce_behaviors::smart_eat::{
                        AUTO_EAT_REQUESTER, SILENT_OFFHAND_REQUESTER,
                    };
                    let running = self.state.registry.running(id);
                    let mut modules = self.state.behaviors.borrow_mut();
                    match setting {
                        SmartEatSetting::SilentOffhand(false)
                            if running && modules.smart_eat.silent_offhand =>
                        {
                            modules.pending_slot_resets.push(SILENT_OFFHAND_REQUESTER);
                        }
                        SmartEatSetting::AutoEat(false)
                            if running && modules.smart_eat.auto_eat =>
                        {
                            modules.smart_eat.cancel_auto_eat();
                            modules.pending_slot_resets.push(AUTO_EAT_REQUESTER);
                        }
                        _ => {}
                    }
                    setting.apply(&mut modules.smart_eat);
                }
                ApplyResult::Runtime(RuntimeRequest::VehicleControl(setting))=>{
                    let mut m=self.state.behaviors.borrow_mut();
                    if let Err(e)=setting.apply(&mut m.vehicle_control.settings){eprintln!("[LiquidBounce] VehicleControl config leaf skipped: {e}");}
                    if !m.vehicle_control.settings.rehook{m.vehicle_control.cancel();}
                }
                ApplyResult::Runtime(RuntimeRequest::VehicleBoost(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().vehicle_boost);
                }
                ApplyResult::Runtime(RuntimeRequest::HandDerp(setting)) => {
                    setting.apply(
                        &mut self
                            .state
                            .behaviors
                            .borrow()
                            .hand_derp
                            .behavior
                            .lock()
                            .expect("HandDerp poisoned"),
                    );
                }
                ApplyResult::Runtime(RuntimeRequest::FreeLook(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().free_look);
                }
                ApplyResult::Runtime(RuntimeRequest::Derp(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().derp);
                }
                ApplyResult::Runtime(RuntimeRequest::CameraClip(setting)) => {
                    let enabled = self
                        .state
                        .registry
                        .module(id)
                        .expect("registered module")
                        .enabled();
                    let changed = setting.apply(
                        &mut self.state.behaviors.borrow_mut().camera_clip,
                        enabled,
                        self.state.registry.world().in_game(),
                    );
                    if changed {
                        self.state.camera_distance_changed();
                    }
                }
                ApplyResult::Runtime(RuntimeRequest::DankBobbing(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().dank_bobbing);
                }
                ApplyResult::Runtime(RuntimeRequest::MultiActions(setting)) => {
                    if let Err(error) =
                        setting.apply(&mut self.state.behaviors.borrow_mut().multi_actions)
                    {
                        eprintln!("[LiquidBounce] config leaf skipped: {error}");
                    }
                }
                ApplyResult::Runtime(RuntimeRequest::Twerk(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().twerk);
                }
                ApplyResult::Runtime(RuntimeRequest::Hitbox(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().hitbox);
                }
                ApplyResult::Runtime(RuntimeRequest::AutoWeapon(setting)) => {
                    if let Err(error) =
                        setting.apply(&mut self.state.behaviors.borrow_mut().auto_weapon)
                    {
                        eprintln!("[LiquidBounce] config leaf skipped: {error}");
                    }
                }
                ApplyResult::Runtime(RuntimeRequest::Chams(setting)) => {
                    let setting = match setting {
                        config::ChamsSetting::File(file) => config::ChamsSetting::File(
                            liquidbounce_interop::desktop::normalize_to_client_folder(
                                &self.state.config_root,
                                &file,
                            ),
                        ),
                        setting => setting,
                    };
                    setting.apply(&mut self.state.behaviors.borrow_mut().chams);
                }
                ApplyResult::Runtime(RuntimeRequest::AutoTool(setting)) => {
                    let mut b = self.state.behaviors.borrow_mut();
                    let before = b.auto_tool.consider_inventory;
                    if let Err(error) = setting.apply(&mut b.auto_tool, &self.state.block_registry)
                    {
                        eprintln!("[LiquidBounce] config leaf skipped: {error}");
                    }
                    if before
                        && !b.auto_tool.consider_inventory
                        && self.state.registry.world().in_game()
                    {
                        let b = &mut *b;
                        b.inventory.cancel_child(&mut b.auto_tool);
                    }
                }
                ApplyResult::Runtime(RuntimeRequest::NoCooldownProgress(value)) => {
                    self.state
                        .behaviors
                        .borrow_mut()
                        .silent_hotbar
                        .no_cooldown_progress = value;
                }
                ApplyResult::Runtime(RuntimeRequest::NoEntityInteract(setting)) => {
                    if let Err(error) = setting.apply(
                        &mut self.state.behaviors.borrow_mut().no_entity_interact,
                        &self.state.entity_type_registry,
                        &self.state.item_registry,
                    ) {
                        eprintln!("[LiquidBounce] config leaf skipped: {error}");
                    }
                }
                ApplyResult::Runtime(RuntimeRequest::Reach(setting)) => {
                    setting.apply(
                        &mut self.state.behaviors.borrow_mut().reach,
                        self.entity_interaction_attribute,
                    );
                }
                ApplyResult::Runtime(RuntimeRequest::FullBrightMode(mode)) => {
                    let mut b = self.state.behaviors.borrow_mut();
                    let running = self.state.registry.running(id);
                    let mut intents = Vec::new();
                    b.full_bright
                        .switch_mode(mode, running, self.options.gamma, |intent| {
                            intents.push(intent)
                        });
                    for intent in intents {
                        b.push_night_vision(intent);
                    }
                }
                ApplyResult::Runtime(RuntimeRequest::RemoveAttackCooldown(value)) => {
                    self.state
                        .behaviors
                        .borrow_mut()
                        .no_miss_cooldown
                        .remove_attack_cooldown = value;
                }
                ApplyResult::Runtime(RuntimeRequest::CancelAttackOnMiss(value)) => {
                    self.state
                        .behaviors
                        .borrow_mut()
                        .no_miss_cooldown
                        .cancel_attack_on_miss = value;
                }
                ApplyResult::Runtime(RuntimeRequest::NoSwingHideFor(value)) => {
                    self.state.behaviors.borrow().no_swing.set_hide_for(value);
                }
                ApplyResult::Runtime(RuntimeRequest::ServerSide(value)) => {
                    self.state
                        .behaviors
                        .borrow()
                        .no_pitch_limit
                        .set_server_side(value);
                }
                ApplyResult::Runtime(RuntimeRequest::ReduceInterpolationBy(value)) => {
                    self.state.behaviors.borrow_mut().no_interpolation.reduce_by = value;
                }
                ApplyResult::Runtime(RuntimeRequest::MiningConditions(value)) => {
                    self.state.behaviors.borrow_mut().no_slow_break = value;
                }
                ApplyResult::Runtime(RuntimeRequest::SprintMode(value)) => {
                    self.state.behaviors.borrow_mut().sprint.mode = value
                }
                ApplyResult::Runtime(RuntimeRequest::SprintIgnore(mask)) => {
                    self.state.behaviors.borrow_mut().sprint.ignore =
                        liquidbounce_behaviors::sprint::Ignore {
                            blindness: mask & 1 != 0,
                            hunger: mask & 2 != 0,
                            collision: mask & 4 != 0,
                        };
                }
                ApplyResult::Runtime(RuntimeRequest::SprintStopOn(mask)) => {
                    self.state.behaviors.borrow_mut().sprint.stop_on =
                        liquidbounce_behaviors::sprint::StopOn {
                            ground: mask & 1 != 0,
                            air: mask & 2 != 0,
                            sneaking: mask & 4 != 0,
                            using_item: mask & 8 != 0,
                        };
                }
                ApplyResult::Runtime(RuntimeRequest::RenderMask(value)) => {
                    self.state.behaviors.borrow_mut().anti_blind.render_mask = value
                }
                ApplyResult::Runtime(RuntimeRequest::FireOpacity(value)) => {
                    self.state.behaviors.borrow_mut().anti_blind.fire_opacity = value
                }
                ApplyResult::Runtime(RuntimeRequest::NoParticles(value)) => {
                    self.state.behaviors.borrow_mut().anti_blind.no_particles = value
                }
                ApplyResult::Runtime(RuntimeRequest::SkipTerrainLoading(value)) => {
                    self.state
                        .behaviors
                        .borrow_mut()
                        .anti_blind
                        .skip_terrain_loading = value
                }
                ApplyResult::Runtime(RuntimeRequest::AirJumpMode(value)) => {
                    self.state.behaviors.borrow_mut().air_jump.mode = value
                }
                ApplyResult::Runtime(RuntimeRequest::PushBy(mask)) => {
                    self.state.behaviors.borrow_mut().no_push =
                        liquidbounce_behaviors::no_push::NoPush {
                            entities: mask & 1 != 0,
                            blocks: mask & 2 != 0,
                            fishing_rod: mask & 4 != 0,
                            liquids: mask & 8 != 0,
                            sinking: mask & 16 != 0,
                        }
                }
                ApplyResult::Runtime(RuntimeRequest::AspectRatio(value)) => {
                    self.state.behaviors.borrow_mut().aspect.ratio = value
                }
                ApplyResult::Runtime(RuntimeRequest::SmoothFirstPov(value)) => {
                    self.state
                        .behaviors
                        .borrow_mut()
                        .smooth_camera
                        .enable_first_pov = value
                }
                ApplyResult::Runtime(RuntimeRequest::SmoothResetPerspective(value)) => {
                    self.state
                        .behaviors
                        .borrow_mut()
                        .smooth_camera
                        .reset_on_perspective_change = value
                }
                ApplyResult::Runtime(RuntimeRequest::SmoothHorizontal(value)) => {
                    self.state
                        .behaviors
                        .borrow_mut()
                        .smooth_camera
                        .horizontal_factor = value
                }
                ApplyResult::Runtime(RuntimeRequest::SmoothVertical(value)) => {
                    self.state
                        .behaviors
                        .borrow_mut()
                        .smooth_camera
                        .vertical_factor = value
                }
                ApplyResult::Runtime(RuntimeRequest::RearView(value)) => {
                    self.state
                        .behaviors
                        .borrow_mut()
                        .quick_perspective_swap
                        .rear_view = value
                }
                ApplyResult::Runtime(RuntimeRequest::RespawnDelay(value)) => self
                    .state
                    .behaviors
                    .borrow_mut()
                    .auto_respawn
                    .set_delay(value),
                ApplyResult::Runtime(RuntimeRequest::KeepMotion(value)) => {
                    self.state.behaviors.borrow_mut().keep_sprint.motion = value
                }
                ApplyResult::Runtime(RuntimeRequest::KeepMotionHurt(value)) => {
                    self.state
                        .behaviors
                        .borrow_mut()
                        .keep_sprint
                        .motion_when_hurt = value
                }
                ApplyResult::Runtime(RuntimeRequest::KeepHurtTime(value)) => {
                    self.state.behaviors.borrow_mut().keep_sprint.hurt_time = value
                }
                ApplyResult::Runtime(RuntimeRequest::KeepChance(value)) => {
                    self.state.behaviors.borrow_mut().keep_sprint.chance = value
                }
                ApplyResult::Runtime(RuntimeRequest::CrystalSize(value)) => {
                    self.state.behaviors.borrow_mut().crystal_view.size = value
                }
                ApplyResult::Runtime(RuntimeRequest::CrystalY(value)) => {
                    self.state.behaviors.borrow_mut().crystal_view.y_translate = value
                }
                ApplyResult::Runtime(RuntimeRequest::CrystalSpin(value)) => {
                    self.state.behaviors.borrow_mut().crystal_view.spin_speed = value
                }
                ApplyResult::Runtime(RuntimeRequest::CrystalBounce(value)) => {
                    self.state.behaviors.borrow_mut().crystal_view.bounce = value
                }
                ApplyResult::Runtime(RuntimeRequest::PlaceCooldown(value)) => {
                    self.state.behaviors.borrow_mut().fast_place.cooldown = (value[0], value[1])
                }
                ApplyResult::Runtime(RuntimeRequest::PlaceApplyTo(value)) => {
                    let mut modules = self.state.behaviors.borrow_mut();
                    modules.fast_place.projectiles = value & 1 != 0;
                    modules.fast_place.blocks = value & 2 != 0;
                }
                ApplyResult::Runtime(RuntimeRequest::PlaceStartDelay(value)) => {
                    self.state.behaviors.borrow_mut().fast_place.start_delay = value
                }
                ApplyResult::Runtime(RuntimeRequest::AutoFish(setting)) => {
                    use config::AutoFishSetting;
                    use liquidbounce_behaviors::auto_fish::DelayRange;
                    let modules = self.state.behaviors.borrow();
                    let mut fish = modules
                        .auto_fish
                        .behavior
                        .lock()
                        .expect("AutoFish poisoned");
                    let settings = &mut fish.settings;
                    match setting {
                        AutoFishSetting::ReelDelay([first, last]) => {
                            settings.reel_delay = DelayRange::new(first, last)
                        }
                        AutoFishSetting::Sounds(value) => {
                            settings.sounds =
                                liquidbounce_behaviors::auto_fish_sounds::deserialize_sounds(
                                    &value,
                                    &self.state.sound_registry,
                                )
                                .map_err(str::to_owned)?
                        }
                        AutoFishSetting::SoundDistance(value) => settings.sound_distance = value,
                        AutoFishSetting::MaxDistance(value) => settings.max_distance = value,
                        AutoFishSetting::RecastRod(value) => settings.recast_rod = value,
                        AutoFishSetting::RecastDelay([first, last]) => {
                            settings.recast_delay = DelayRange::new(first, last)
                        }
                        AutoFishSetting::AutoCastRod(value) => settings.auto_cast_rod = value,
                        AutoFishSetting::AutoCastDelay([first, last]) => {
                            settings.auto_cast_delay = DelayRange::new(first, last)
                        }
                    }
                }
                ApplyResult::Runtime(RuntimeRequest::AutoF5(setting)) => {
                    use config::AutoF5Setting;
                    let mut modules = self.state.behaviors.borrow_mut();
                    let module = &mut modules.auto_f5;
                    match setting {
                        AutoF5Setting::CheckType(value) => module.check_screen_handler_type = value,
                        AutoF5Setting::Types(value) => module.types = value,
                        AutoF5Setting::TypeFilter(value) => module.type_filter = value,
                        AutoF5Setting::CheckTitle(value) => module.check_screen_title = value,
                        AutoF5Setting::Titles(value) => module.titles = value,
                        AutoF5Setting::Custom(value) => module.custom_titles = value,
                        AutoF5Setting::TitleFilter(value) => module.title_filter = value,
                    }
                }
                ApplyResult::Changed(_) => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_file_preserves_supported_options_and_marks_inert_modules() {
        let mut state = State::default();
        let options = RawOptions {
            fov: 70, gamma: 0.5, player_rotation: [0.0; 2], actual_main_hand: 1, model_parts: 127,
        };
        let mut actor = Actor { state: &mut state, options, entity_interaction_attribute: None };
        actor.set_settings("Zoom", &json!({"name":"Zoom","value":[{"name":"Zoom","value":123}]})).unwrap();
        let inert = MODULES.iter().copied().filter(|kind| kind.is_unimplemented()).collect::<Vec<_>>();
        assert_eq!(inert.len(), 25);
        let binding = json!({"boundKey":"key.keyboard.k","action":"Hold","modifiers":["Shift"]});
        for kind in &inert {
            actor.set_settings(kind.name(), &json!({"name":kind.name(),"value":[
                {"name":"Enabled","value":true},
                {"name":"Bind","value":binding.clone()},
                {"name":"Hidden","value":true}
            ]})).unwrap();
        }
        let file = actor.file().expect("one inert module must not prevent saving the entire registry");
        let modules = file["value"].as_array().unwrap();
        assert_eq!(modules.len(), MODULES.len());
        for kind in MODULES.iter().copied() {
            let saved = modules.iter().find(|value| value["name"] == kind.name()).unwrap();
            let interop = actor.serialize(kind, JsonFormat::Interop).unwrap();
            if !kind.is_unimplemented() {
                assert!(saved.get("implementation").is_none(), "{} lost its native settings", kind.name());
                assert!(interop.get("implementation").is_none(), "{} lost its native settings", kind.name());
                assert!(config::serialize_unimplemented(kind, &actor.common(kind), JsonFormat::File, &str::to_owned).is_err());
                continue;
            }
            assert_eq!(saved["implementation"], "unimplemented");
            let leaves = saved["value"].as_array().unwrap();
            assert_eq!(leaves.iter().map(|value| value["name"].as_str().unwrap()).collect::<Vec<_>>(),
                ["Enabled", "Bind", "Hidden"]);
            assert_eq!(interop["implementation"], "unimplemented");
            assert_eq!(interop["value"].as_array().unwrap().len(), 2);
            actor.set_settings(kind.name(), &json!({"name":kind.name(),"value":[
                {"name":"Enabled","value":false},
                {"name":"Bind","value":{"boundKey":"key.keyboard.unknown","action":"Toggle"}},
                {"name":"Hidden","value":false}
            ]})).unwrap();
            actor.set_settings(kind.name(), saved).unwrap();
            let restored = actor.common(kind);
            assert!(restored.enabled && restored.hidden, "{} common flags were lost", kind.name());
            assert_eq!(restored.key_bind, binding, "{} bind was lost", kind.name());
        }
        // HoleFiller has a real native BlockPlacer despite the older parity
        // report still listing it as unfinished. Preserve its full tree.
        let hole_filler = modules.iter().find(|value| value["name"] == "HoleFiller").unwrap();
        assert!(!ModuleKind::HoleFiller.is_unimplemented());
        assert!(hole_filler["value"].as_array().unwrap().len() > 3);
        let zoom = modules.iter().find(|value| value["name"] == "Zoom").unwrap();
        assert!(zoom.get("implementation").is_none());
        assert_eq!(zoom["value"].as_array().unwrap().iter()
            .find(|value| value["name"] == "Zoom").unwrap()["value"], 123);
        actor.set_settings("Zoom", &json!({"name":"Zoom","value":[{"name":"Zoom","value":55}]})).unwrap();
        actor.set_settings("Zoom", zoom).unwrap();
        assert_eq!(actor.state.behaviors.borrow().zoom.settings.zoom, 123);
    }

    #[test]
    fn skin_derp_lifecycle_captures_in_game_options_and_restores_outside_the_actor() {
        let mut state = State::default();
        let options = RawOptions {
            fov: 70,
            gamma: 0.5,
            player_rotation: [0.0; 2],
            actual_main_hand: 1,
            model_parts: 17,
        };
        let mut actor = Actor {
            state: &mut state,
            options,
            entity_interaction_attribute: None,
        };
        actor.set_enabled("SkinDerp", true).unwrap();
        actor.set_enabled("SkinDerp", false).unwrap();
        assert!(
            actor
                .state
                .behaviors
                .borrow()
                .pending_model_parts
                .is_empty()
        );
        actor.state.with_registry(options, |registry, callbacks| {
            registry.update_world_sync(
                liquidbounce_core::WorldState {
                    world_present: true,
                    player_present: true,
                },
                callbacks,
            )
        });
        actor.set_enabled("SkinDerp", true).unwrap();
        assert_eq!(
            actor.state.behaviors.borrow().skin_derp.previous_parts(),
            17
        );
        actor.options.model_parts = 126;
        actor
            .set_settings(
                "SkinDerp",
                &json!({"name":"SkinDerp","value":[
            {"name":"Sync","value":true},{"name":"Delay","value":-5}]}),
            )
            .unwrap();
        actor.set_enabled("SkinDerp", false).unwrap();
        let mut modules = actor.state.behaviors.borrow_mut();
        assert_eq!(std::mem::take(&mut modules.pending_model_parts), vec![17]);
        assert_eq!(modules.skin_derp.settings.delay, -5);
    }

    #[test]
    fn smart_eat_child_disable_cancels_only_its_wait_and_resets_its_requester() {
        use liquidbounce_behaviors::smart_eat::{AUTO_EAT_REQUESTER, SILENT_OFFHAND_REQUESTER};
        let mut state = State::default();
        let options = RawOptions {
            fov: 70,
            gamma: 0.5,
            player_rotation: [0.0; 2],
            actual_main_hand: 1,
            model_parts: 127,
        };
        state.with_registry(options, |registry, callbacks| {
            registry.update_world_sync(
                liquidbounce_core::WorldState {
                    world_present: true,
                    player_present: true,
                },
                callbacks,
            )
        });
        let mut actor = Actor {
            state: &mut state,
            options,
            entity_interaction_attribute: None,
        };
        actor.set_enabled("SmartEat", true).unwrap();
        actor
            .set_settings(
                "SmartEat",
                &json!({"name":"SmartEat","value":[
            {"name":"AutoEat","value":[{"name":"Enabled","value":false}]},
            {"name":"SilentOffhand","value":[{"name":"Enabled","value":false}]}]}),
            )
            .unwrap();
        assert_eq!(
            std::mem::take(&mut actor.state.behaviors.borrow_mut().pending_slot_resets),
            vec![SILENT_OFFHAND_REQUESTER, AUTO_EAT_REQUESTER]
        );
        actor.set_enabled("SmartEat", false).unwrap();
        assert!(
            actor
                .state
                .behaviors
                .borrow()
                .pending_slot_resets
                .is_empty()
        );
    }

    #[test]
    fn free_look_enable_uses_immediate_player_angles_and_keeps_hold_lifecycle() {
        use liquidbounce_core::WorldState;
        let mut state = State::default();
        let options = RawOptions {
            fov: 70,
            gamma: 0.5,
            player_rotation: [725.0, -50.0],
            actual_main_hand: 1,
            model_parts: 127,
        };
        state.with_registry(options, |registry, callbacks| {
            registry.update_world_sync(
                WorldState {
                    world_present: true,
                    player_present: true,
                },
                callbacks,
            )
        });
        let mut actor = Actor {
            state: &mut state,
            options,
            entity_interaction_attribute: None,
        };
        assert_eq!(
            actor.common(ModuleKind::FreeLook).key_bind["action"],
            "Hold"
        );
        actor.set_enabled("FreeLook", true).unwrap();
        assert_eq!(
            actor
                .state
                .behaviors
                .borrow()
                .free_look
                .camera_rotation(true),
            Some([725.0, -50.0])
        );
        actor
            .set_settings(
                "FreeLook",
                &json!({"name":"FreeLook","value":[{"name":"SenseBoost","value":2.0}]}),
            )
            .unwrap();
        assert_eq!(actor.state.behaviors.borrow().free_look.camera_yaw, 725.0);
        actor.set_enabled("FreeLook", false).unwrap();
        actor.options.player_rotation = [-30.0, 12.0];
        actor.set_enabled("FreeLook", true).unwrap();
        assert_eq!(
            actor
                .state
                .behaviors
                .borrow()
                .free_look
                .camera_rotation(true),
            Some([-30.0, 12.0])
        );
        actor
            .state
            .with_registry(actor.options, |registry, callbacks| {
                registry.update_world_sync(WorldState::default(), callbacks)
            });
        assert!(
            actor
                .state
                .registry
                .module(actor.id(ModuleKind::FreeLook))
                .unwrap()
                .enabled(),
            "a world change is distinct from the original DisconnectEvent"
        );
        actor
            .state
            .with_registry(actor.options, |registry, callbacks| {
                registry.update_world_sync(
                    WorldState {
                        world_present: true,
                        player_present: true,
                    },
                    callbacks,
                );
                for report in registry.disconnect_sync(callbacks) {
                    assert!(report.unwrap().failures.is_empty());
                }
            });
        assert!(
            !actor
                .state
                .registry
                .module(actor.id(ModuleKind::FreeLook))
                .unwrap()
                .enabled()
        );
    }

    #[test]
    fn global_targets_partial_failure_persists_and_preserves_unported_settings() {
        use liquidbounce_behaviors::targeting::{Target, TargetSet};
        let mut state = State::default();
        let mut actor = Actor {
            state: &mut state,
            options: RawOptions {
                fov: 70,
                gamma: 0.5,
                player_rotation: [0.0; 2],
                actual_main_hand: 1,
                model_parts: 127,
            },
            entity_interaction_attribute: None,
        };
        actor
            .set_global_settings(&json!({"name":"Settings","value":[
                {"name":"Targets","value":[
                    {"name":"Combat","value":["Passive",null,"Players"]},
                    {"name":"Visual","value":["Self","Players"]}
                ]},
                {"name":"Inventory","value":[
                    {"name":"ResponseTime","value":275},
                    {"name":"LockAfterRejections","value":5}
                ]},
                {"name":"FutureSystem","value":[{"name":"Opaque","value":{"nested":[1,2,3]}}]}
            ]}))
            .unwrap();
        assert_eq!(
            actor.state.global_targets.combat,
            TargetSet::from_targets([Target::Passive])
        );
        assert!(
            actor
                .state
                .global_targets
                .visual
                .contains(Target::SelfEntity)
        );
        assert_eq!(
            actor.global_settings()["value"].as_array().unwrap().len(),
            2
        );
        assert_eq!(
            actor
                .state
                .behaviors
                .borrow()
                .inventory
                .manager
                .response_time,
            275
        );
        assert_eq!(
            actor
                .state
                .behaviors
                .borrow()
                .inventory
                .manager
                .lock_after_rejections,
            5
        );
        let saved = actor.global_file();
        assert_eq!(
            saved["value"]
                .as_array()
                .unwrap()
                .iter()
                .find(|group| group["name"] == "FutureSystem")
                .unwrap()["value"][0]["value"],
            json!({"nested":[1,2,3]})
        );
        assert_eq!(
            saved["value"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|group| group["name"] == "Inventory")
                .count(),
            1
        );
        let events = actor.state.events.borrow();
        assert!(events.iter().any(|(name, event)| name == "valueChanged"
            && event["value"]["name"] == "ResponseTime"
            && event["value"]["value"] == 275));
        drop(events);
        actor
            .set_global_settings(
                &json!({"name":"Settings","value":[{"name":"Enemies","value":[
                    {"name":"Combat","value":null},{"name":"Visual","value":[]}
                ]}]}),
            )
            .unwrap();
        assert_eq!(actor.state.global_targets.combat, TargetSet::EMPTY);
        actor.set_global_settings(&saved).unwrap();
        assert_eq!(actor.global_file(), saved);
        assert!(
            actor
                .set_global_settings(&json!({"name":"Global","value":[]}))
                .is_err()
        );
        assert_eq!(actor.global_file(), saved);
    }

    #[test]
    fn multi_actions_alias_partial_failure_and_enabled_query_are_preserved() {
        let mut state = State::default();
        let mut actor = Actor {
            state: &mut state,
            options: RawOptions {
                fov: 70,
                gamma: 0.5,
                player_rotation: [0.0; 2],
                actual_main_hand: 1,
                model_parts: 127,
            },
            entity_interaction_attribute: None,
        };
        actor
            .set_settings(
                "MultiTask",
                &json!({"name":"MultiTask","value":[
                    {"name":"Enabled","value":true}, {"name":"Hidden","value":true},
                    {"name":"Actions","value":["BreakingWhileUsing",null,"PlacingWhileBreaking"]}
                ]}),
            )
            .unwrap();
        let id = actor.id(ModuleKind::MultiActions);
        assert!(!actor.state.registry.running(id));
        let enabled = actor.state.registry.module(id).unwrap().enabled();
        assert!(enabled);
        assert_eq!(
            actor.state.behaviors.borrow().multi_actions.policy(enabled),
            (false, false, true)
        );
        assert_eq!(actor.state.hidden.get("MultiActions"), Some(&true));
        actor
            .set_settings(
                "MultiTask",
                &json!({"name":"MultiActions","value":[{"name":"Actions","value":null}]}),
            )
            .unwrap();
        assert_eq!(actor.state.behaviors.borrow().multi_actions.actions, 0);
        let saved = actor
            .serialize(ModuleKind::MultiActions, JsonFormat::File)
            .unwrap();
        actor.set_settings("MultiTask", &saved).unwrap();
        assert_eq!(
            actor
                .serialize(ModuleKind::MultiActions, JsonFormat::File)
                .unwrap(),
            saved
        );
    }

    #[test]
    fn camera_clip_remembered_distance_follows_child_lifecycle_and_publishes_full_leaf() {
        let mut state = State::default();
        let options = RawOptions {
            fov: 70,
            gamma: 0.5,
            player_rotation: [0.0; 2],
            actual_main_hand: 1,
            model_parts: 127,
        };
        state.with_registry(options, |registry, callbacks| {
            registry.update_world_sync(
                liquidbounce_core::WorldState {
                    world_present: true,
                    player_present: true,
                },
                callbacks,
            )
        });
        let mut actor = Actor {
            state: &mut state,
            options,
            entity_interaction_attribute: None,
        };
        actor
            .set_settings(
                "CameraClip",
                &json!({"name":"CameraClip","value":[{"name":"CameraDistance","value":10}]}),
            )
            .unwrap();
        actor.set_enabled("CameraClip", true).unwrap();
        assert_eq!(
            actor.state.behaviors.borrow().camera_clip.scrolled_distance,
            10.0
        );
        actor.set_settings("CameraClip", &json!({"name":"CameraClip","value":[{"name":"ScrollAdjust","value":[{"name":"RememberScrolled","value":true}]}]})).unwrap();
        actor
            .state
            .behaviors
            .borrow_mut()
            .camera_clip
            .scrolled_distance = 9.0;
        actor.set_enabled("CameraClip", false).unwrap();
        assert_eq!(
            actor.state.behaviors.borrow().camera_clip.camera_distance,
            10.0
        );
        actor.set_enabled("CameraClip", true).unwrap();
        assert_eq!(
            actor.state.behaviors.borrow().camera_clip.camera_distance,
            9.0
        );
        actor.set_settings("CameraClip", &json!({"name":"CameraClip","value":[{"name":"ScrollAdjust","value":[{"name":"Enabled","value":false}]}]})).unwrap();
        actor
            .state
            .behaviors
            .borrow_mut()
            .camera_clip
            .scrolled_distance = 8.0;
        actor.set_settings("CameraClip", &json!({"name":"CameraClip","value":[{"name":"ScrollAdjust","value":[{"name":"Enabled","value":true}]}]})).unwrap();
        assert_eq!(
            actor.state.behaviors.borrow().camera_clip.camera_distance,
            8.0
        );
        let events = actor.state.events.borrow();
        let (_, payload) = events
            .iter()
            .rev()
            .find(|(name, _)| name == "valueChanged")
            .unwrap();
        assert_eq!(payload["value"]["value"], 8.0);
        assert_eq!(payload["value"]["valueType"], "FLOAT");
        assert_eq!(
            payload["value"]["key"],
            "liquidbounce.module.cameraClip.cameraDistance"
        );
        assert_eq!(payload["value"]["range"], json!({"from":1.0,"to":48.0}));
        assert_eq!(payload["value"]["suffix"], "");
    }

    #[test]
    fn no_rotate_settings_restore_all_processors_through_the_actor() {
        use liquidbounce_behaviors::no_rotate_set::{
            FloatRange, NoRotateMode, NoRotateSet, SmoothMode,
        };
        use liquidbounce_behaviors::rotations::MovementCorrection;
        let mut state = State::default();
        let mut actor = Actor {
            state: &mut state,
            options: RawOptions {
                fov: 70,
                gamma: 0.5,
                player_rotation: [0.0; 2],
                actual_main_hand: 1,
                model_parts: 127,
            },
            entity_interaction_attribute: None,
        };
        let mut expected = NoRotateSet {
            mode: NoRotateMode::ResetRotation,
            ..Default::default()
        };
        expected.rotations.active = SmoothMode::Acceleration;
        expected.rotations.linear.horizontal_turn_speed = FloatRange(34.0, -2.0);
        expected.rotations.sigmoid.steepness = 17.0;
        expected.rotations.acceleration.yaw_acceleration = FloatRange(2.0, 8.0);
        expected.rotations.acceleration.sigmoid_deceleration = true;
        expected.rotations.movement_correction = MovementCorrection::ChangeLook;
        let file = config::serialize_no_rotate_set(
            &expected,
            &actor.common(ModuleKind::NoRotateSet),
            JsonFormat::File,
            &str::to_owned,
        )
        .unwrap();
        actor.set_settings("NoRotateSet", &file).unwrap();
        assert_eq!(actor.state.behaviors.borrow().no_rotate_set, expected);
        let saved = actor
            .serialize(ModuleKind::NoRotateSet, JsonFormat::File)
            .unwrap();
        actor.state.behaviors.borrow_mut().no_rotate_set = NoRotateSet::default();
        actor.set_settings("NoRotateSet", &saved).unwrap();
        assert_eq!(actor.state.behaviors.borrow().no_rotate_set, expected);
    }

    #[test]
    fn no_entity_settings_resolve_live_registries_and_skip_only_failed_leaf() {
        use liquidbounce_behaviors::no_entity_interact::Filter;
        let mut state = State::default();
        state.entity_type_registry = ["minecraft:pig", "minecraft:villager", "custom:target"]
            .map(str::to_owned)
            .into();
        state.item_registry = ["minecraft:air", "minecraft:apple", "custom:tool"]
            .map(str::to_owned)
            .into();
        let mut actor = Actor {
            state: &mut state,
            options: RawOptions {
                fov: 70,
                gamma: 0.5,
                player_rotation: [0.0; 2],
                actual_main_hand: 1,
                model_parts: 127,
            },
            entity_interaction_attribute: None,
        };
        actor
            .set_settings(
                "NoEntityInteract",
                &json!({"name":"NoEntityInteract","value":[
                    {"name":"HoldingItems","value":["custom:tool","APPLE","missing",34,"air"]},
                    {"name":"HoldingItemFilter","value":"Blacklist"},
                    {"name":"EntityTypes","value":["villager","custom:target","missing"]},
                    {"name":"EntityTypeFilter","value":"Whitelist"}
                ]}),
            )
            .unwrap();
        {
            let modules = actor.state.behaviors.borrow();
            let module = &modules.no_entity_interact;
            assert_eq!(module.entity_type_filter, Filter::Whitelist);
            assert_eq!(
                module.entity_types,
                ["custom:target", "minecraft:pig", "minecraft:villager"]
                    .map(str::to_owned)
                    .into()
            );
            assert_eq!(
                module.holding_items,
                ["custom:tool", "minecraft:air"].map(str::to_owned).into()
            );
            assert_eq!(module.holding_item_filter, Filter::Blacklist);
        }
        let saved = actor
            .serialize(ModuleKind::NoEntityInteract, JsonFormat::File)
            .unwrap();
        actor.set_settings("NoEntityInteract", &saved).unwrap();
        assert_eq!(
            actor
                .serialize(ModuleKind::NoEntityInteract, JsonFormat::File)
                .unwrap(),
            saved
        );
        actor
            .set_settings(
                "NoEntityInteract",
                &json!({"name":"NoEntityInteract","value":[
                    {"name":"EntityTypes","value":[null]},
                    {"name":"HoldingItemFilter","value":"Whitelist"},
                    {"name":"HoldingItems","value":["apple"]}
                ]}),
            )
            .unwrap();
        let modules = actor.state.behaviors.borrow();
        assert!(
            modules
                .no_entity_interact
                .entity_types
                .contains("minecraft:pig")
        );
        assert_eq!(
            modules.no_entity_interact.holding_item_filter,
            Filter::Whitelist
        );
        assert_eq!(
            modules.no_entity_interact.holding_items,
            ["minecraft:apple"].map(str::to_owned).into()
        );
    }

    #[test]
    fn reach_uses_request_attribute_and_declaration_order_without_query_reclamp() {
        let mut state = State::default();
        let mut actor = Actor {
            state: &mut state,
            options: RawOptions {
                fov: 70,
                gamma: 0.5,
                player_rotation: [0.0; 2],
                actual_main_hand: 1,
                model_parts: 127,
            },
            entity_interaction_attribute: Some(6.0),
        };
        // Input order differs from declaration order. The clamp must see the
        // newly applied increase plus the real attribute observed for this request.
        actor
            .set_settings(
                "Reach",
                &json!({"name":"Reach","value":[
                    {"name":"BlockRangeIncrease","value":-2.5},
                    {"name":"Entity","value":[
                        {"name":"ThroughWallsRange","value":20},
                        {"name":"RangeIncrease","value":2.0}
                    ]}
                ]}),
            )
            .unwrap();
        assert_eq!(
            actor.state.behaviors.borrow().reach.through_walls_range,
            8.0
        );
        actor.entity_interaction_attribute = Some(3.0);
        actor
            .set_settings(
                "Reach",
                &json!({"name":"Reach","value":[
                    {"name":"Entity","value":[{"name":"RangeIncrease","value":0.0}]}
                ]}),
            )
            .unwrap();
        assert_eq!(
            actor.state.behaviors.borrow().reach.through_walls_range,
            8.0
        );
        assert_eq!(
            actor.state.behaviors.borrow().reach.block_range_increase,
            -2.5
        );
        let saved = actor
            .serialize(ModuleKind::Reach, JsonFormat::File)
            .unwrap();
        actor.entity_interaction_attribute = None;
        actor.set_settings("Reach", &saved).unwrap();
        assert_eq!(
            actor.state.behaviors.borrow().reach.through_walls_range,
            8.0
        );
        actor
            .set_settings(
                "Reach",
                &json!({"name":"Reach","value":[
                    {"name":"Entity","value":[{"name":"ThroughWallsRange","value":9.0}]}
                ]}),
            )
            .unwrap();
        assert_eq!(
            actor.state.behaviors.borrow().reach.through_walls_range,
            3.0
        );
        assert_eq!(
            actor
                .modules()
                .iter()
                .find(|v| v["name"] == "Reach")
                .unwrap()["category"],
            "Player"
        );
    }

    #[test]
    fn auto_fish_roundtrip_registry_tokens_and_real_lifecycle() {
        use liquidbounce_behaviors::auto_fish::{DelayRange, FishHand};
        use liquidbounce_core::WorldState;
        let mut state = State::default();
        state.sound_registry = vec![
            "minecraft:entity.fishing_bobber.splash".into(),
            "custom:bite".into(),
        ];
        let options = RawOptions {
            fov: 70,
            gamma: 0.5,
            player_rotation: [0.0; 2],
            actual_main_hand: 1,
            model_parts: 127,
        };
        let mut actor = Actor {
            state: &mut state,
            entity_interaction_attribute: None,
            options,
        };
        actor.set_settings("AutoFish", &json!({"name":"AutoFish","value":[
            {"name":"ReelDelay","value":0},
            {"name":"Sounds","value":["entity.fishing_bobber.splash","custom:bite"]},
            {"name":"SoundDistance","value":[{"name":"Enabled","value":false},{"name":"MaxDistance","value":-2.5}]},
            {"name":"RecastRod","value":[{"name":"Enabled","value":true},{"name":"Delay","value":{"from":25,"to":-4}}]},
            {"name":"AutoCastRod","value":[{"name":"Enabled","value":true},{"name":"Delay","value":-5}]}
        ]})).unwrap();
        let service = actor.state.behaviors.borrow().auto_fish.clone();
        {
            let fish = service.behavior.lock().unwrap();
            assert_eq!(fish.settings.reel_delay, DelayRange::new(0, 0));
            assert_eq!(fish.settings.recast_delay, DelayRange::new(25, -4));
            assert_eq!(fish.settings.auto_cast_delay, DelayRange::new(-5, -5));
            assert_eq!(fish.settings.max_distance, -2.5);
            assert!(!fish.settings.sound_distance);
            assert_eq!(fish.settings.sounds.len(), 2);
        }
        let exported = actor
            .serialize(ModuleKind::AutoFish, JsonFormat::File)
            .unwrap();
        actor.set_settings("AutoFish", &exported).unwrap();
        assert_eq!(
            actor
                .serialize(ModuleKind::AutoFish, JsonFormat::File)
                .unwrap(),
            exported
        );
        let settings = actor.settings("AutoFish").unwrap();
        assert_eq!(
            settings["value"]
                .as_array()
                .unwrap()
                .iter()
                .map(|node| node["name"].as_str().unwrap())
                .collect::<Vec<_>>(),
            [
                "Bind",
                "Hidden",
                "ReelDelay",
                "Sounds",
                "SoundDistance",
                "RecastRod",
                "AutoCastRod"
            ]
        );
        assert_eq!(settings["value"][3]["registry"], "sound_event");
        actor.set_enabled("AutoFish", true).unwrap();
        assert!(!service.is_running());
        actor.state.with_registry(options, |registry, callbacks| {
            registry.update_world_sync(
                WorldState {
                    world_present: true,
                    player_present: true,
                },
                callbacks,
            )
        });
        assert!(service.is_running());
        {
            let mut fish = service.behavior.lock().unwrap();
            fish.sound_packet(true, Some("custom:bite"), Some([0.0; 3]), [0.0; 3]);
            fish.start_tick_handler(true, Some(FishHand::Main));
            assert!(fish.caught_fish() && fish.busy());
        }
        actor.state.with_registry(options, |registry, callbacks| {
            registry.update_world_sync(WorldState::default(), callbacks)
        });
        assert!(!service.is_running());
        actor.set_enabled("AutoFish", false).unwrap();
        assert!(
            service.behavior.lock().unwrap().caught_fish(),
            "out-of-game enabled edits skip onDisabled"
        );
        actor.set_enabled("AutoFish", true).unwrap();
        actor.state.with_registry(options, |registry, callbacks| {
            registry.update_world_sync(
                WorldState {
                    world_present: true,
                    player_present: true,
                },
                callbacks,
            )
        });
        actor.set_enabled("AutoFish", false).unwrap();
        assert!(!service.is_running());
        assert!(!service.behavior.lock().unwrap().caught_fish());
        assert!(!service.behavior.lock().unwrap().busy());

        actor.set_settings("AutoF5", &json!({"name":"AutoF5","value":[{"name":"CheckScreenHandlerType","value":[{"name":"Types","value":":hopper"}]}]})).unwrap();
        let expected = std::collections::BTreeSet::from([Some("minecraft:hopper".to_owned())]);
        assert_eq!(actor.state.behaviors.borrow().auto_f5.types, expected);
        for invalid in [json!(true), Value::Null, json!(["hopper", false])] {
            actor.set_settings("AutoF5", &json!({"name":"AutoF5","value":[{"name":"CheckScreenHandlerType","value":[{"name":"Types","value":invalid}]}]})).unwrap();
            assert_eq!(actor.state.behaviors.borrow().auto_f5.types, expected);
        }
    }

    #[test]
    fn original_bind_roundtrip_keeps_action_and_modifier() {
        let bind = InputBind::new(InputKey::Keyboard(29), BindAction::Hold)
            .with_modifiers(Modifiers::new(&[Modifier::Control]));
        assert_eq!(parse_bind(&bind_json(Some(bind))).unwrap(), bind);
    }
    #[test]
    fn gameplay_settings_restore_original_tags_and_committed_runtime_state() {
        use liquidbounce_behaviors::{sprint::SprintMode, visual_environment::DoRender};
        let mut state = State::default();
        let mut actor = Actor {
            state: &mut state,
            entity_interaction_attribute: None,
            options: RawOptions {
                fov: 70,
                gamma: 0.5,
                player_rotation: [0.0; 2],
                actual_main_hand: 1,
                model_parts: 127,
            },
        };
        let options = actor.options;
        actor.state.with_registry(options, |registry, callbacks| {
            registry.update_world_sync(
                liquidbounce_core::WorldState {
                    world_present: true,
                    player_present: true,
                },
                callbacks,
            )
        });
        for (name, values) in [
            (
                "NoSlowBreak",
                json!([{"name":"When","value":["Underwater"]}]),
            ),
            (
                "Sprint",
                json!([{"name":"StopOn","value":["UsingItem"]},{"name":"Ignore","value":["Blindness","Hunger"]},{"name":"Mode","value":"Omnirotational"}]),
            ),
            (
                "AntiBlind",
                json!([{"name":"SkipTerrainLoading","value":true},{"name":"NoParticles","value":true},{"name":"FireOpacity","value":-125},{"name":"DoRender","value":["Title","BossBars"]}]),
            ),
            ("NoSlotSet", json!([])),
            ("AirJump", json!([{"name":"Mode","value":"DoubleJump"}])),
            (
                "NoPush",
                json!([{"name":"PushBy","value":["Blocks","Sinking"]}]),
            ),
            ("Aspect", json!([{"name":"Ratio","value":-25}])),
            (
                "SmoothCamera",
                json!([
                    {"name":"EnableFirstPOV","value":true},
                    {"name":"ResetOnPerspectiveChange","value":false},
                    {"name":"HorizontalFactor","value":1.5},
                    {"name":"VerticalFactor","value":-0.25}
                ]),
            ),
            (
                "QuickPerspectiveSwap",
                json!([{"name":"RearView","value":true}]),
            ),
            (
                "NoPose",
                json!([{"name":"NoSwim","value":true},{"name":"SneakHeight","value":"1.9"}]),
            ),
            ("AutoRespawn", json!([{"name":"Delay","value":-2}])),
            (
                "KeepSprint",
                json!([
                    {"name":"Motion","value":{"from":25.0,"to":75.0}},
                    {"name":"MotionWhenHurt","value":{"from":-20.0,"to":120.0}},
                    {"name":"HurtTime","value":{"from":-1,"to":20}},
                    {"name":"Chance","value":72.5}
                ]),
            ),
            (
                "CrystalView",
                json!([
                    {"name":"Size","value":2.5}, {"name":"YTranslate","value":-3.0},
                    {"name":"SpinSpeed","value":-0.5}, {"name":"Bounce","value":1.5}
                ]),
            ),
            (
                "FastPlace",
                json!([
                    {"name":"Cooldown","value":{"from":-1,"to":6}},
                    {"name":"ApplyTo","value":["Blocks"]},
                    {"name":"StartDelay","value":-200}
                ]),
            ),
            (
                "AutoF5",
                json!([
                    {"name":"CheckScreenHandlerType","value":[
                        {"name":"Enabled","value":false},
                        {"name":"Types","value":["hopper","minecraft:generic_9x6"]},
                        {"name":"Filter","value":"Blacklist"}
                    ]},
                    {"name":"CheckScreenTitle","value":[
                        {"name":"Enabled","value":true},
                        {"name":"Titles","value":["ChestBoat","Beacon"]},
                        {"name":"Custom","value":["商店","Shop","商店"]},
                        {"name":"Filter","value":"Whitelist"}
                    ]}
                ]),
            ),
        ] {
            actor
                .set_settings(name, &json!({"name":name,"value":values}))
                .unwrap();
            actor.set_enabled(name, true).unwrap();
        }
        {
            let modules = actor.state.behaviors.borrow();
            assert_eq!(modules.sprint.mode, SprintMode::Omnirotational);
            assert!(modules.sprint.ignore.blindness && modules.sprint.ignore.hunger);
            assert!(!modules.sprint.ignore.collision);
            assert!(
                !modules.sprint.stop_on.ground
                    && !modules.sprint.stop_on.air
                    && !modules.sprint.stop_on.sneaking
                    && modules.sprint.stop_on.using_item
            );
            assert!(
                !modules.no_slow_break.mining_fatigue
                    && !modules.no_slow_break.on_air
                    && modules.no_slow_break.underwater
            );
            assert_eq!(
                modules.anti_blind.render_mask,
                DoRender::Title.bit() | DoRender::BossBars.bit()
            );
            assert_eq!(modules.anti_blind.fire_opacity, -125);
            assert_eq!(modules.aspect.ratio, -25);
            assert!(modules.smooth_camera.enable_first_pov);
            assert!(!modules.smooth_camera.reset_on_perspective_change);
            assert_eq!(modules.smooth_camera.horizontal_factor, 1.5);
            assert_eq!(modules.smooth_camera.vertical_factor, -0.25);
            assert!(modules.quick_perspective_swap.rear_view);
            assert_eq!(modules.no_pose.policy(true), (true, 2));
            assert_eq!(modules.auto_respawn.delay(), -2);
            assert_eq!(modules.keep_sprint.motion, [25.0, 75.0]);
            assert_eq!(modules.keep_sprint.motion_when_hurt, [-20.0, 120.0]);
            assert_eq!(modules.keep_sprint.hurt_time, [-1, 20]);
            assert_eq!(modules.keep_sprint.chance, 72.5);
            assert_eq!(modules.crystal_view.size, 2.5);
            assert_eq!(modules.crystal_view.y_translate, -3.0);
            assert_eq!(modules.crystal_view.spin_speed, -0.5);
            assert_eq!(modules.crystal_view.bounce, 1.5);
            assert_eq!(modules.fast_place.cooldown, (-1, 6));
            assert!(!modules.fast_place.projectiles && modules.fast_place.blocks);
            assert_eq!(modules.fast_place.start_delay, -200);
            assert!(
                !modules.auto_f5.check_screen_handler_type && modules.auto_f5.check_screen_title
            );
            assert_eq!(
                modules.auto_f5.types,
                [
                    Some("minecraft:generic_9x6".to_owned()),
                    Some("minecraft:hopper".to_owned())
                ]
                .into()
            );
            assert_eq!(
                modules.auto_f5.type_filter,
                liquidbounce_behaviors::auto_f5::Filter::Blacklist
            );
            assert_eq!(
                modules.auto_f5.titles,
                [
                    liquidbounce_behaviors::auto_f5::ContainerTitle::Beacon,
                    liquidbounce_behaviors::auto_f5::ContainerTitle::ChestBoat
                ]
            );
            assert_eq!(
                modules.auto_f5.custom_titles,
                ["Shop".to_owned(), "商店".to_owned()].into()
            );
            assert!(modules.anti_blind.no_particles && modules.anti_blind.skip_terrain_loading);
            assert_eq!(
                modules.air_jump.mode,
                liquidbounce_behaviors::air_jump::AirJumpMode::DoubleJump
            );
            assert!(
                !modules.no_push.entities
                    && modules.no_push.blocks
                    && !modules.no_push.fishing_rod
                    && !modules.no_push.liquids
                    && modules.no_push.sinking
            );
        }
        // Exported file settings retain stable English keys, including raw
        // imported values outside the slider range, across an actor restore.
        for name in [
            "NoSlowBreak",
            "Sprint",
            "AntiBlind",
            "NoSlotSet",
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
        ] {
            let kind = ModuleKind::from_name(name).unwrap();
            let exported = actor.serialize(kind, JsonFormat::File).unwrap();
            actor.set_settings(name, &exported).unwrap();
            assert_eq!(actor.serialize(kind, JsonFormat::File).unwrap(), exported);
        }
        let quick = actor
            .state
            .registry
            .module(actor.id(ModuleKind::QuickPerspectiveSwap))
            .unwrap();
        assert!(quick.definition().disable_on_quit);
        assert_eq!(
            actor.common(ModuleKind::QuickPerspectiveSwap).key_bind["action"],
            "Hold"
        );
        let smooth = actor.settings("SmoothCamera").unwrap();
        let children = smooth["value"].as_array().unwrap();
        assert_eq!(
            children
                .iter()
                .find(|v| v["name"] == "EnableFirstPOV")
                .unwrap()["valueType"],
            "BOOLEAN"
        );
        assert_eq!(
            children
                .iter()
                .find(|v| v["name"] == "HorizontalFactor")
                .unwrap()["range"],
            json!({"from":0.0,"to":1.0})
        );
    }
    #[test]
    fn settings_execute_real_registry_and_mode_state() {
        let mut state = State::default();
        let mut actor = Actor {
            state: &mut state,
            entity_interaction_attribute: None,
            options: RawOptions {
                fov: 70,
                gamma: 0.5,
                player_rotation: [0.0; 2],
                actual_main_hand: 1,
                model_parts: 127,
            },
        };
        let mut expected_order: Vec<_> = ModuleKind::ALL.iter().map(|kind| kind.name()).collect();
        expected_order.sort_by_key(|name| name.to_ascii_lowercase());
        for modules in [actor.modules(), actor.actual_modules()] {
            assert_eq!(
                modules
                    .iter()
                    .map(|module| module["name"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                expected_order
            );
        }
        actor.set_enabled("NoFOV", true).unwrap();
        assert!(
            actor
                .modules()
                .iter()
                .find(|v| v["name"] == "NoFOV")
                .unwrap()["enabled"]
                .as_bool()
                .unwrap()
        );
        assert!(actor.actual_modules().iter().all(|v|v["running"]==json!(v["name"]=="ClickGUI")));
        actor.set_settings("FullBright",&json!({"name":"FullBright","value":[{"name":"Mode","active":"NightVision","choices":{}}]})).unwrap();
        assert_eq!(
            actor.state.behaviors.borrow().full_bright.mode().tag(),
            "NightVision"
        );
    }

    #[test]
    fn gui_closer_aliases_atomic_nullable_filter_and_file_interop_roundtrip() {
        use liquidbounce_core::WorldState;
        let mut state = State::default();
        let options = RawOptions {
            fov: 70,
            gamma: 0.5,
            player_rotation: [0.0; 2],
            actual_main_hand: 1,
            model_parts: 127,
        };
        state.with_registry(options, |registry, callbacks| {
            registry.update_world_sync(
                WorldState {
                    world_present: true,
                    player_present: true,
                },
                callbacks,
            )
        });
        let mut actor = Actor {
            state: &mut state,
            options,
            entity_interaction_attribute: None,
        };
        let kind = ModuleKind::from_name("GUICloser").unwrap();
        for alias in ["AutoClose", "ContainerCloser"] {
            assert_eq!(ModuleKind::from_name(alias), Some(kind));
        }
        actor.set_enabled("AutoClose", true).unwrap();
        assert!(
            actor
                .state
                .behaviors
                .borrow()
                .query_running(&actor.state.registry, "GUICloser")
        );
        actor
            .set_settings(
                "ContainerCloser",
                &json!({"name":"GUICloser","value":[
                    {"name":"Filter","value":["a",null,"a"]},
                    {"name":"Mode","value":"Contains"},
                    {"name":"PrintScreenTitle","value":true}
                ]}),
            )
            .unwrap();
        {
            let modules = actor.state.behaviors.borrow();
            let filters: Vec<_> = modules
                .gui_closer
                .filters
                .iter()
                .map(|value| value.as_ref().map(|v| v.pattern()))
                .collect();
            assert_eq!(filters, [Some("a"), None, Some("a")]);
            let decision = modules.gui_closer.screen_opening(true, true, "Vote");
            assert!(decision.null_filter);
            assert!(!decision.cancel);
            assert!(!decision.print_title);
        }
        actor
            .set_settings(
                "GUICloser",
                &json!({"name":"GUICloser","value":[
                    {"name":"Filter","value":["valid","["]},
                    {"name":"PrintScreenTitle","value":false}
                ]}),
            )
            .unwrap();
        assert!(!actor.state.behaviors.borrow().gui_closer.print_screen_title);
        let file = actor.serialize(kind, JsonFormat::File).unwrap();
        actor.set_settings("AutoClose", &file).unwrap();
        assert_eq!(actor.serialize(kind, JsonFormat::File).unwrap(), file);
        let interop = actor.settings("ContainerCloser").unwrap();
        let leaves = interop["value"].as_array().unwrap();
        assert_eq!(
            leaves.iter().find(|v| v["name"] == "Filter").unwrap()["value"],
            json!(["a", null, "a"])
        );
        assert!(leaves.iter().any(|v| v["name"] == "PrintScreenTitle"));
        let modules = actor.modules();
        let module = modules.iter().find(|v| v["name"] == "GUICloser").unwrap();
        assert_eq!(module["category"], "Misc");
        assert_eq!(module["aliases"], json!(["AutoClose", "ContainerCloser"]));
    }

    #[test]
    fn elytra_swap_alias_settings_enable_reset_and_disable_on_quit() {
        use liquidbounce_core::WorldState;
        let mut state = State::default();
        let options = RawOptions {
            fov: 70,
            gamma: 0.5,
            player_rotation: [0.0; 2],
            actual_main_hand: 1,
            model_parts: 127,
        };
        state.with_registry(options, |registry, callbacks| {
            registry.update_world_sync(
                WorldState {
                    world_present: true,
                    player_present: true,
                },
                callbacks,
            )
        });
        let mut actor = Actor {
            state: &mut state,
            options,
            entity_interaction_attribute: None,
        };
        let kind = ModuleKind::from_name("ChestSwap").unwrap();
        assert_eq!(kind, ModuleKind::ElytraSwap);
        assert_eq!(kind.category(), "Player");
        assert!(
            actor
                .state
                .registry
                .module(actor.id(kind))
                .unwrap()
                .definition()
                .disable_on_quit
        );
        actor.state.behaviors.borrow_mut().elytra_swap.wants_glider = Some(false);
        actor.set_enabled("ChestSwap", true).unwrap();
        assert_eq!(
            actor.state.behaviors.borrow().elytra_swap.wants_glider,
            None
        );
        actor
            .set_settings(
                "ChestSwap",
                &json!({"name":"ElytraSwap","value":[
                    {"name":"Constraints","valueType":"CONFIGURABLE","value":[
                        {"name":"ClickDelay","value":{"from":-1,"to":2}},
                        {"name":"Requires","value":["InventoryOpen","NoMovement"]}
                    ]}
                ]}),
            )
            .unwrap();
        assert_eq!(
            actor
                .state
                .behaviors
                .borrow()
                .elytra_swap
                .constraints
                .click_delay,
            [-1, 2]
        );
        let settings = actor.serialize(kind, JsonFormat::File).unwrap();
        actor.set_settings("ElytraSwap", &settings).unwrap();
        assert_eq!(actor.serialize(kind, JsonFormat::File).unwrap(), settings);
        let modules = actor.modules();
        let entry = modules
            .iter()
            .find(|entry| entry["name"] == "ElytraSwap")
            .unwrap();
        assert_eq!(entry["aliases"], json!(["ChestSwap"]));
        actor.state.behaviors.borrow_mut().elytra_swap.wants_glider = Some(true);
        let reports = actor.state.with_registry(options, |registry, callbacks| {
            registry.disconnect_sync(callbacks)
        });
        assert!(reports.iter().all(|report| {
            report
                .as_ref()
                .is_ok_and(|report| report.failures.is_empty())
        }));
        assert!(!actor.common(kind).enabled);
        assert_eq!(
            actor.state.behaviors.borrow().elytra_swap.wants_glider,
            Some(true)
        );
        actor.state.with_registry(options, |registry, callbacks| {
            registry.update_world_sync(
                WorldState {
                    world_present: false,
                    player_present: false,
                },
                callbacks,
            )
        });
        assert!(!actor.common(kind).enabled);
        assert_eq!(
            actor.state.behaviors.borrow().elytra_swap.wants_glider,
            Some(true)
        );
    }

    #[test]
    fn packet_running_snapshot_follows_committed_toggle_and_world_lifecycle() {
        use liquidbounce_core::WorldState;
        let mut state = State::default();
        let options = RawOptions {
            fov: 70,
            gamma: 0.5,
            player_rotation: [0.0; 2],
            actual_main_hand: 1,
            model_parts: 127,
        };
        let no_swing = state.behaviors.borrow().no_swing.clone();
        let mut actor = Actor {
            state: &mut state,
            entity_interaction_attribute: None,
            options,
        };
        actor.set_enabled("NoSwing", true).unwrap();
        assert!(!no_swing.should_hide_for_server());
        actor.state.with_registry(options, |registry, callbacks| {
            registry.update_world_sync(
                WorldState {
                    world_present: true,
                    player_present: true,
                },
                callbacks,
            )
        });
        assert!(no_swing.should_hide_for_client());
        assert!(no_swing.should_hide_for_server());
        actor
            .set_settings(
                "NoSwing",
                &json!({"name":"NoSwing","value":[{"name":"HideFor","value":["Server"]}]}),
            )
            .unwrap();
        assert!(!no_swing.should_hide_for_client());
        assert!(no_swing.should_hide_for_server());
        actor.set_enabled("NoSwing", false).unwrap();
        assert!(!no_swing.should_hide_for_server());
        actor.set_enabled("NoSwing", true).unwrap();
        assert!(no_swing.should_hide_for_server());
        actor.state.with_registry(options, |registry, callbacks| {
            registry.update_world_sync(
                WorldState {
                    world_present: false,
                    player_present: false,
                },
                callbacks,
            )
        });
        assert!(!no_swing.should_hide_for_client());
        assert!(!no_swing.should_hide_for_server());
        assert!(
            actor
                .modules()
                .iter()
                .find(|value| value["name"] == "NoSwing")
                .unwrap()["enabled"]
                .as_bool()
                .unwrap()
        );
    }
}
