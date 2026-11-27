#!/usr/bin/env python3
"""Generate exact phase hooks against the untouched recovered Horizon source."""

from pathlib import Path
import difflib
import importlib.util
import json

HERE = Path(__file__).resolve().parent
SOURCE = HERE.parent.parent / "upstream/Horizon"


def replace_once(text: str, original: str, replacement: str) -> str:
    if text.count(original) != 1:
        raise ValueError(f"Expected one original source occurrence: {original[:100]}")
    return text.replace(original, replacement, 1)


hooks = '''    /// Integer Options.fov value before Camera.calculateFov / getNearPlane.
    pub(super) fn native_camera_fov_option(&self, original: i32, option: &mut i32) {
        mod_api::dispatch_camera_fov_option(self, mod_api::mods(), original, option);
    }
    /// Camera.calculateFov return, before Horizon's separate zoom extra.
    pub(super) fn native_camera_fov_return(&self, fov: &mut f32) {
        mod_api::dispatch_camera_fov_return(self, mod_api::mods(), fov);
    }
    /// Independent GameRenderer.bobHurt / bobView HEAD cancellations.
    pub(super) fn native_camera_bob_effects(&self, hurt: &mut bool, view: &mut bool) {
        mod_api::dispatch_camera_bob_effects(self, mod_api::mods(), hurt, view);
    }
    /// Read-only observation of actual avatar inputs and the matrices returned
    /// by the rendering phase, including the separate native shake result.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn native_camera_bob_result(&self, partial: f32, hurt_time: f32, hurt_duration: i32, hurt_dir: f32, death_time: Option<f32>, walk: f32, bob: f32, damage_tilt: f64, original: [f32; 16], applied: [f32; 16], shake: [f32; 16], combined: [f32; 16]) {
        mod_api::dispatch_camera_bob_result(self, mod_api::mods(), partial, hurt_time, hurt_duration, hurt_dir, death_time, walk, bob, damage_tilt, original, applied, shake, combined);
    }
    /// Read-only observation of the independent portal / nausea matrix.
    pub(super) fn native_camera_spinning_result(&self, portal: f32, nausea: f32, scale: f32, intensity: f32, angle: f32, matrix: [f32; 16]) {
        mod_api::dispatch_camera_spinning_result(self, mod_api::mods(), portal, nausea, scale, intensity, angle, matrix);
    }
    /// AbstractClientPlayer.getFieldOfViewModifier's accepted return.
    /// Unlike the older generic hook's guard, vanilla may return zero here.
    pub(super) fn native_player_fov_return(&self, modifier: &mut f32) {
        mod_api::dispatch_player_fov_return(self, mod_api::mods(), modifier);
    }
    /// LightmapRenderStateExtractor's brightnessOption, without an upper clamp.
    pub(super) fn native_lightmap_gamma_option(&self, gamma: &mut f32) {
        mod_api::dispatch_lightmap_gamma_option(self, mod_api::mods(), gamma);
    }
    /// MouseHandler's first-person-and-scoping sensitivity branch.
    pub(super) fn native_mouse_scoping_condition(&self, scoping: &mut bool) {
        mod_api::dispatch_mouse_scoping_condition(self, mod_api::mods(), scoping);
    }
    /// Tail of the real window LocalPlayer tick, never the server player's turn.
    pub(super) fn native_client_player_tick_post(&mut self) {
        mod_api::dispatch_client_player_tick_post(self, mod_api::mods());
    }
    /// The official direct-control path has no Winit KeyEvent. Keep it separate
    /// from native_input_key so a real window key can never be delivered twice.
    pub(super) fn native_client_control_key(&mut self, key: KeyCode, pressed: bool, repeat: bool, modifiers: u32) {
        mod_api::dispatch_client_control_key(self, mod_api::mods(), key, pressed, repeat, modifiers);
    }
    /// Unmodified FOV option for onEnabled/onDisabled animation endpoints.
    pub(super) fn native_client_option_fov(&self) -> i32 { self.settings.fov as i32 }
    /// Unmodified stored gamma option (the module converts it to float).
    pub(super) fn native_client_option_gamma(&self) -> f64 { f64::from(self.settings.gamma) }
    /// The configured Minecraft 26.3 debug modifier's actual held state.
    pub(super) fn native_client_debug_modifier_down(&self) -> bool {
        keybinds::key_code(self.settings.keys.get("key.debug.modifier"))
            .is_some_and(|key| self.physical_keys.contains(&key))
    }
    /// Only the local client's Night Vision, using the same effect add/remove
    /// code as client potion state. No server-player helper or packet is used.
    pub(super) fn native_client_night_vision(&mut self, duration: Option<i32>) -> bool {
        if !self.native_window() || self.player.is_none() { return false; }
        if let Some(duration) = duration {
            self.add_player_effect(horizon_client::effects::EffectInstance::new("night_vision", duration, 0))
        } else {
            let before = self.potions.effects.list.len();
            self.potions.effects.list.retain(|effect| effect.id != "night_vision");
            self.apply_effect_status();
            self.potions.effects.list.len() != before
        }
    }

'''

changes: dict[str, tuple[str, str]] = {}


def edit(path: str, transform) -> None:
    if path in changes:
        original, current = changes[path]
    else:
        source = SOURCE / path
        original = source.read_text(encoding="utf-8") if source.is_file() else ""
        current = original
    changes[path] = (original, transform(current))


edit("crates/horizon-client/src/game/native_hooks.rs", lambda text: replace_once(text,
    "    /// Player FOV scale, after vanilla effects and before zoom and easing.\n",
    hooks + "    /// Player FOV scale, after vanilla effects and before zoom and easing.\n"))


def fov(text: str) -> str:
    text = replace_once(text, '''        let intensity = portal.max(self.nausea_blend(partial)) * (scale * scale);
        spinning_effect_matrix(intensity, fx.spinning_time + partial * fx.spinning_speed)
''', '''        let nausea = self.nausea_blend(partial);
        let intensity = portal.max(nausea) * (scale * scale);
        let angle = fx.spinning_time + partial * fx.spinning_speed;
        let matrix = spinning_effect_matrix(intensity, angle);
        self.native_camera_spinning_result(portal, nausea, scale, intensity, angle, matrix.to_cols_array());
        matrix
''')
    text = replace_once(text, '''        let tilt = self.settings.file.double("damageTiltStrength");
        glam::Mat4::from_cols_array(&bob_matrix(&state, tilt, self.settings.view_bobbing).m)
            * self.native_shake_matrix(partial)
''', '''        let tilt = self.settings.file.double("damageTiltStrength");
        let original = bob_matrix(&state, tilt, self.settings.view_bobbing).m;
        let mut hurt_enabled = true;
        let mut view_enabled = self.settings.view_bobbing;
        self.native_camera_bob_effects(&mut hurt_enabled, &mut view_enabled);
        let mut filtered = state;
        if !hurt_enabled {
            filtered.living = None;
        }
        let applied = bob_matrix(&filtered, tilt, view_enabled).m;
        let shake = self.native_shake_matrix(partial);
        let combined = glam::Mat4::from_cols_array(&applied) * shake;
        let living = state.living.expect("local player's camera has living state");
        let (walk, bob) = state.player.expect("local player's camera has bob history");
        self.native_camera_bob_result(partial, living.hurt_time, living.hurt_duration, living.hurt_dir, living.death_time, walk, bob, tilt, original, applied, shake.to_cols_array(), combined.to_cols_array());
        combined
''')
    text = replace_once(text, "            valid_fov_modifier(modifier, base)\n", '''            let mut returned = valid_fov_modifier(modifier, base);
            self.native_player_fov_return(&mut returned);
            returned
''')
    text = replace_once(text, "    /// `Camera.calculateFov` with `modifyFovBasedOnDeathOrFluid`.\n", '''    /// Raw integer option phase, shared with the near-plane fluid sample.
    pub(super) fn camera_fov_option(&self) -> i32 {
        let original = self.settings.fov as i32;
        let mut option = original;
        self.native_camera_fov_option(original, &mut option);
        option
    }
    /// `Camera.calculateFov` with `modifyFovBasedOnDeathOrFluid`.
''')
    text = replace_once(text, "        let fov = self.view.calculate_fov(\n", "        let mut fov = self.view.calculate_fov(\n")
    text = replace_once(text, "            self.settings.fov as i32,\n", "            self.camera_fov_option(),\n")
    text = replace_once(text, "        let vanilla = self.zoomed_fov_degrees(fov);\n", "        self.native_camera_fov_return(&mut fov);\n        let vanilla = self.zoomed_fov_degrees(fov);\n")
    return text


edit("crates/horizon-client/src/game/fov.rs", fov)
edit("crates/horizon-client/src/game/render_tick.rs", lambda text: replace_once(text,
    "            self.settings.fov as i32 as f32,\n", "            self.camera_fov_option() as f32,\n"))


def pointer(text: str) -> str:
    text = replace_once(text, "    pub(super) fn look_options(&self) -> LookOptions {\n        LookOptions {\n", '''    pub(super) fn look_options(&self) -> LookOptions {
        let mut scoping = self.camera_type.first_person() && self.scoping();
        self.native_mouse_scoping_condition(&mut scoping);
        LookOptions {
''')
    return replace_once(text, "            scoping: self.scoping(),\n", "            scoping,\n")


edit("crates/horizon-client/src/game/pointer_capture.rs", pointer)
edit("crates/horizon-client/src/game/remote.rs", lambda text: replace_once(text,
    "        self.remote_send_state(input | u8::from(auto_jumped) << 4);\n        Ok(())\n",
    "        self.remote_send_state(input | u8::from(auto_jumped) << 4);\n        self.native_client_player_tick_post();\n        Ok(())\n"))


def render(text: str) -> str:
    text = replace_once(text, "        let debug_passes = self.debug_lists_gpu_passes();\n        let result = match self.renderer.as_mut() {\n", '''        let debug_passes = self.debug_lists_gpu_passes();
        let mut lightmap_gamma = self.settings.gamma;
        self.native_lightmap_gamma_option(&mut lightmap_gamma);
        let result = match self.renderer.as_mut() {
''')
    return replace_once(text, "                renderer.set_pass_timing(debug_passes || self.gpu_passes || recording);\n", "                renderer.gamma = lightmap_gamma;\n                renderer.set_pass_timing(debug_passes || self.gpu_passes || recording);\n")


edit("crates/horizon-client/src/game.rs", render)


def control(text: str) -> str:
    text = replace_once(text, '                if action != "up" && self.screen_key_pressed(physical, action == "repeat") {\n', '''                self.native_client_control_key(physical, action != "up", action == "repeat", super::glfw_modifiers(self.modifiers));
                if action != "up" && self.screen_key_pressed(physical, action == "repeat") {
''')
    # Both early-consumed taps and normal taps still have an actual release.
    original = 'self.native_key_event(physical, false, false);'
    if text.count(original) != 2:
        raise ValueError("Expected the two original direct-control tap releases")
    return text.replace(original, original + '\n                    self.native_client_control_key(physical, false, false, super::glfw_modifiers(self.modifiers));')


edit("crates/horizon-client/src/game/control.rs", control)

names = ["camera_fov_option", "camera_fov_return", "camera_bob_effects", "camera_bob_result", "camera_spinning_result", "player_fov_return", "lightmap_gamma_option", "mouse_scoping_condition", "client_player_tick_post", "client_control_key"]
helper_names = ["client_option_fov", "client_option_gamma", "client_debug_modifier_down", "client_night_vision", "chat"]
for extension in ("movement-patch.py", "entity-bounce-patch.py", "attack-patch.py", "interpolation-patch.py", "mining-patch.py", "sprint-patch.py", "visual-environment-patch.py", "airjump-push-patch.py", "keep-sprint-patch.py", "crystal-view-patch.py", "fast-place-patch.py", "auto-f5-patch.py", "reach-patch.py", "no-pose-patch.py", "no-entity-interact-patch.py", "no-block-interact-patch.py"):
    spec = importlib.util.spec_from_file_location(extension.removesuffix('.py').replace('-', '_'), HERE / extension)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    additions = module.apply(edit, replace_once)
    if isinstance(additions, dict):
        added_hooks, added_helpers = additions["hook_names"], additions["helper_names"]
    else:
        added_hooks, added_helpers = additions
    names.extend(added_hooks)
    helper_names.extend(added_helpers)
packet_spec = importlib.util.spec_from_file_location('packet_patch', HERE / 'packet-patch.py')
packet_module = importlib.util.module_from_spec(packet_spec)
packet_spec.loader.exec_module(packet_module)
for path, text in packet_module.apply(edit, replace_once).items():
    if path in changes:
        raise ValueError(f"Duplicate generated packet source: {path}")
    changes[path] = ("", text)
pitch_spec = importlib.util.spec_from_file_location('pitch_patch', HERE / 'pitch-patch.py')
pitch_module = importlib.util.module_from_spec(pitch_spec)
pitch_spec.loader.exec_module(pitch_module)
for path, text in pitch_module.apply(edit, replace_once).items():
    if path in changes:
        raise ValueError(f"Duplicate generated pitch source: {path}")
    changes[path] = ("", text)
slot_spec = importlib.util.spec_from_file_location('slot_patch', HERE / 'slot-patch.py')
slot_module = importlib.util.module_from_spec(slot_spec)
slot_spec.loader.exec_module(slot_module)
for path, text in slot_module.apply(edit, replace_once).items():
    if path in changes:
        raise ValueError(f"Duplicate generated slot source: {path}")
    changes[path] = ("", text)
respawn_spec = importlib.util.spec_from_file_location('respawn_patch', HERE / 'respawn-patch.py')
respawn_module = importlib.util.module_from_spec(respawn_spec)
respawn_spec.loader.exec_module(respawn_module)
for path, text in respawn_module.apply(edit, replace_once).items():
    if path in changes:
        raise ValueError(f"Duplicate generated respawn source: {path}")
    changes[path] = ("", text)
fishing_spec = importlib.util.spec_from_file_location('fishing_patch', HERE / 'fishing-patch.py')
fishing_module = importlib.util.module_from_spec(fishing_spec)
fishing_spec.loader.exec_module(fishing_module)
for path, text in fishing_module.apply(edit, replace_once).items():
    if path in changes:
        raise ValueError(f"Duplicate generated fishing source: {path}")
    changes[path] = ("", text)
# This extension rewrites visual camera-type consumers, so it must follow the
# earlier helpers whose exact source anchors still read the stored F5 option.
no_rotate_spec = importlib.util.spec_from_file_location('no_rotate_patch', HERE / 'no-rotate-patch.py')
no_rotate_module = importlib.util.module_from_spec(no_rotate_spec)
no_rotate_spec.loader.exec_module(no_rotate_module)
for path, text in no_rotate_module.apply(edit, replace_once).items():
    if path in changes:
        raise ValueError(f"Duplicate generated NoRotateSet source: {path}")
    changes[path] = ("", text)
camera_spec = importlib.util.spec_from_file_location('camera_control_patch', HERE / 'camera-control-patch.py')
camera_module = importlib.util.module_from_spec(camera_spec)
camera_spec.loader.exec_module(camera_module)
camera_additions = camera_module.apply(edit, replace_once)
names.extend(camera_additions['hook_names'])
helper_names.extend(camera_additions['helper_names'])
for extension in ('camera-clip-patch.py', 'free-look-patch.py', 'dank-bobbing-patch.py', 'multi-actions-patch.py', 'twerk-patch.py', 'hitbox-patch.py', 'target-lock-patch.py', 'teams-patch.py', 'spammer-patch.py', 'ping-spoof-patch.py', 'clip-patch.py', 'book-bot-patch.py', 'item-identity-patch.py', 'cooldown-components-patch.py', 'auto-weapon-patch.py', 'chams-patch.py', 'auto-tool-patch.py', 'anti-afk-patch.py', 'trident-boost-patch.py', 'sleep-walker-patch.py', 'derp-patch.py', 'vehicle-boost-patch.py', 'hand-derp-patch.py', 'coroutine-ticker-patch.py', 'timer-patch.py', 'block-bounce-patch.py', 'smart-eat-patch.py', 'skin-derp-patch.py', 'entity-control-patch.py', 'snap-tap-patch.py', 'block-walk-patch.py', 'ghost-hand-patch.py', 'extended-firework-patch.py', 'strafe-patch.py', 'elytra-recast-patch.py', 'fast-break-patch.py', 'inventory-manager-patch.py', 'elytra-swap-patch.py', 'better-inventory-patch.py', 'vehicle-one-hit-patch.py', 'terrain-speed-patch.py', 'auto-break-patch.py', 'high-jump-patch.py', 'spider-patch.py', 'anti-hunger-patch.py', 'item-scroller-patch.py', 'gui-closer-patch.py', 'chest-cleaner-patch.py', 'auto-deposit-patch.py', 'text-field-protect-patch.py', 'reset-vl-patch.py', 'one-shots-patch.py', 'auto-disable-patch.py', 'old-regen-patch.py', 'fast-use-patch.py', 'plugins-patch.py', 'auto-account-patch.py', 'anti-cheat-detect-patch.py', 'middle-click-action-patch.py', 'super-knockback-patch.py', 'mace-kill-patch.py', 'spear-kill-patch.py', 'damage-patch.py', 'yggdrasil-signature-patch.py', 'potion-spoof-patch.py', 'no-clip-patch.py', 'true-sight-patch.py', 'mob-owners-patch.py', 'combine-mobs-patch.py', 'better-tab-patch.py', 'auto-walk-patch.py', 'edge-assistance-patch.py', 'movement-guard-patch.py', 'sneak-patch.py', 'freeze-patch.py', 'reverse-step-patch.py', 'block-outline-patch.py', 'breadcrumbs-patch.py', 'damage-particles-patch.py', 'tracers-patch.py', 'new-chunks-patch.py', 'placement-esp-patch.py', 'protection-zones-patch.py', 'hats-patch.py', 'wings-patch.py', 'semantic-pointer-patch.py', 'liquid-walk-patch.py', 'crosshair-clicker-patch.py', 'crosshair-patch.py', 'particles-patch.py', 'target-renderer-patch.py', 'custom-ambience-patch.py', 'tnt-timer-patch.py', 'trajectories-patch.py', 'radar-patch.py', 'potion-fx-patch.py', 'totem-effect-patch.py', 'jump-effect-patch.py', 'hit-fx-patch.py', 'hole-esp-patch.py', 'blink-esp-patch.py', 'anti-exploit-patch.py', 'eagle-patch.py', 'no-web-patch.py', 'criticals-patch.py', 'no-slow-patch.py', 'report-helper-patch.py', 'auto-queue-patch.py', 'macros-patch.py', 'auto-chat-game-patch.py', 'vehicle-control-patch.py', 'fast-exp-patch.py', 'auto-wind-charge-patch.py', 'auto-buff-patch.py', 'auto-bow-patch.py', 'elytra-target-patch.py', 'auto-pearl-patch.py', 'easy-pearl-patch.py', 'auto-rod-patch.py', 'auto-shoot-patch.py', 'fake-lag-patch.py', 'aimbot-patch.py',  'replenish-patch.py', 'auto-crafter-patch.py', 'inventory-cleaner-patch.py', 'chest-stealer-patch.py', 'item-tags-patch.py', 'bed-plates-patch.py', 'nametags-patch.py', 'placement-use-patch.py', 'extinguish-patch.py', 'auto-armor-patch.py', 'auto-mob-heal-patch.py', 'blink-patch.py', 'logoff-spot-patch.py', 'inventory-tracker-patch.py', 'name-protect-patch.py', 'storage-esp-patch.py', 'esp-patch.py', 'item-esp-patch.py', 'item-chams-patch.py', 'murder-mystery-patch.py', 'auto-dodge-patch.py', 'air-place-patch.py', 'flag-check-patch.py', 'offhand-patch.py', 'packet-logger-patch.py', 'anti-staff-patch.py', 'notifier-patch.py', 'anti-bot-patch.py', 'projectile-puncher-patch.py', 'xray-patch.py', 'inventory-move-patch.py', 'anti-void-patch.py', 'block-placement-patch.py', 'stronghold-finder-patch.py', 'kill-aura-patch.py', 'kill-aura-block-input-patch.py', 'sword-block-patch.py', 'fightbot-navigation-patch.py', 'free-cam-patch.py', 'falling-forecast-patch.py', 'scaffold-replay-patch.py', 'scaffold-selection-patch.py', 'scaffold-clutch-patch.py', 'scaffold-action-patch.py', 'scaffold-planner-patch.py', 'scaffold-prediction-patch.py', 'scaffold-techniques-patch.py', 'scaffold-click-inside-patch.py', 'clutch-prediction-patch.py', 'scaffold-features-patch.py', 'scaffold-special-techniques-patch.py', 'block-placer-patch.py', 'hole-filler-patch.py', 'bed-defender-patch.py', 'debug-patch.py', 'click-gui-patch.py', 'block-esp-patch.py', 'better-chat-patch.py', 'speed-patch.py', 'target-strafe-patch.py', 'step-patch.py'):
    spec = importlib.util.spec_from_file_location(extension.removesuffix('.py').replace('-', '_'), HERE / extension)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    additions = module.apply(edit, replace_once, changes) if extension == 'step-patch.py' else module.apply(edit, replace_once)
    names.extend(additions['hook_names'])
    helper_names.extend(additions['helper_names'])
menu_spec = importlib.util.spec_from_file_location('menu_backend_patch', HERE / 'menu-backend-patch.py')
menu_module = importlib.util.module_from_spec(menu_spec)
menu_spec.loader.exec_module(menu_module)
menu_additions = menu_module.apply(edit, replace_once)
names.extend(menu_additions['hook_names'])
helper_names.extend(menu_additions['helper_names'])
fragment = {
    "hooks": [{"name": name, "target": f"horizon_client::game::native_hooks::Game<'_>::native_{name}", "side": "client", "cancellable": False, "mod_api": {"contract": "listen"}} for name in names],
    "helpers": [{"name": name, "target": f"horizon_client::game::native_hooks::Game<'_>::native_{name}"} for name in helper_names],
    "rust_summary": "Exact camera FOV and bobbing, local-player post-tick, mouse sensitivity and lightmap option phases for native client modules, with read-only camera matrix observations.",
    "doc_summary": "Local source extension for the pinned LiquidBounce port; these hooks are absent from original Horizon 9f42064 and require this patch.",
}
changes["tools/native-hook-fragments/liquidbounce_behavior.json"] = ("", json.dumps(fragment, indent=2) + "\n")

output = []
for path, (before, after) in changes.items():
    if before == after:
        continue
    output.append(f"diff --git a/{path} b/{path}\n")
    if not before:
        output.append("new file mode 100644\n")
    output.extend(difflib.unified_diff(before.splitlines(keepends=True), after.splitlines(keepends=True), fromfile=f"a/{path}" if before else "/dev/null", tofile=f"b/{path}"))
destination = HERE / "patches/horizon-behavior-hooks.patch"
destination.parent.mkdir(exist_ok=True)
content = "".join(output)
if not destination.is_file() or destination.read_text(encoding="utf-8") != content:
    destination.write_text(content, encoding="utf-8", newline="\n")
print(destination)

