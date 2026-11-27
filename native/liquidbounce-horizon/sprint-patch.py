#!/usr/bin/env python3
"""Sprint's distinct query phases and shared client rotation integration.

Compose with ``apply(edit, replace_once)``. Original shared/server entry points
retain their signatures and defaults; no upstream files are written here.
"""

HOOK_NAMES = ["client_directional_input", "client_sprint_queries", "client_sprint_network",
              "client_rotation_network", "client_rotation_model"]
HELPER_NAMES = ["client_player_rotation", "client_mouse_sensitivity", "client_inventory_open",
                "client_vanilla_sin_cos", "client_rotation_reset_yaw"]

HOOKS = '''    /// KeyboardInput.tick after MovementInputEvent, before the input packet.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn native_client_directional_input(&self, forward: &mut bool, backward: &mut bool, left: &mut bool, right: &mut bool, jump: &mut bool, shift: &mut bool, sprint: &mut bool) {
        if !self.native_window() { return; }
        mod_api::dispatch_client_directional_input(self, mod_api::mods(), forward, backward, left, right, jump, shift, sprint);
    }
    /// Actual current input and item-use state, before walking or riding.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn native_client_sprint_queries(&self, sideways: f32, forward: f32, on_ground: bool, slow_item: bool, shift: bool, player_yaw: f32, movement_override: &mut i8, omnidirectional: &mut bool, ignore_blindness: &mut bool, ignore_hunger: &mut bool, ignore_collision: &mut bool, jump_yaw: &mut Option<f32>, movement_yaw: &mut Option<f32>, managed_jump_yaw: &mut Option<f32>) {
        if !self.native_window() { return; }
        mod_api::dispatch_client_sprint_queries(self, mod_api::mods(), sideways, forward, on_ground, slow_item, shift, player_yaw, movement_override, omnidirectional, ignore_blindness, ignore_hunger, ignore_collision, jump_yaw, movement_yaw, managed_jump_yaw);
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn client_sprint_queries_for_input(&self, sideways: f32, forward: f32, shift: bool, movement_override: &mut i8, omnidirectional: &mut bool, ignore_blindness: &mut bool, ignore_hunger: &mut bool, ignore_collision: &mut bool, jump_yaw: &mut Option<f32>, movement_yaw: &mut Option<f32>, managed_jump_yaw: &mut Option<f32>) {
        let Some(player) = self.player.as_ref() else { return; };
        let slow_item = self.using.and_then(|(slot, _)| {
            if slot == crate::game::shield::OFFHAND_SLOT { self.offhand.as_ref() }
            else { self.inventory.get(slot)?.as_ref() }
        }).and_then(|stack| self.items().get(&stack.item)).is_some_and(|item| !item.can_sprint);
        self.native_client_sprint_queries(sideways, forward, player.ground.on_ground, slow_item, shift, self.yaw, movement_override, omnidirectional, ignore_blindness, ignore_hunger, ignore_collision, jump_yaw, movement_yaw, managed_jump_yaw);
    }
    /// sendIsSprintingIfNeeded's own event does not alter avatar sprint state.
    pub(super) fn native_client_sprint_network(&self, sprinting: &mut bool) {
        if !self.native_window() { return; }
        mod_api::dispatch_client_sprint_network(self, mod_api::mods(), sprinting);
    }
    /// sendPosition/sendChanges rotation, independent of camera look.
    pub(super) fn native_client_rotation_network(&self, yaw: &mut f32, pitch: &mut f32) {
        if !self.native_window() { return; }
        mod_api::dispatch_client_rotation_network(self, mod_api::mods(), yaw, pitch);
    }
    /// Extracted local renderer rotations, after vanilla rider body solving.
    pub(super) fn native_client_rotation_model(&self, partial: f32, body: &mut f32, head: &mut f32, pitch: &mut f32) {
        if !self.native_window() { return; }
        mod_api::dispatch_client_rotation_model(self, mod_api::mods(), partial, body, head, pitch);
    }
    pub(super) fn native_client_player_rotation(&self) -> [f32; 2] { [self.yaw, self.pitch] }
    pub(super) fn native_client_mouse_sensitivity(&self) -> f64 { f64::from(self.settings.sensitivity) }
    pub(super) fn native_client_inventory_open(&self) -> bool { matches!(self.screen, Screen::Inventory | Screen::Creative) }
    pub(super) fn native_client_vanilla_sin_cos(&self, radians: f64) -> [f32; 2] {
        [horizon_core::geometry::trig::sin(radians), horizon_core::geometry::trig::cos(radians)]
    }
    pub(super) fn native_client_rotation_reset_yaw(&mut self, yaw: f32) -> bool {
        if !self.native_window() || !yaw.is_finite() { return false; }
        let Some(player) = self.player.as_mut() else { return false; };
        self.yaw = yaw;
        player.yaw = yaw;
        self.sway.reset_yaw(yaw);
        true
    }

'''

QUERIES = '''/// Per-input client policy; all shared/server calls retain vanilla queries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientSprintQueries {
    /// -1 retains each original query; 0/1 overrides it independently.
    pub movement_override: i8,
    pub omnidirectional: bool,
    pub ignore_blindness: bool,
    pub ignore_hunger: bool,
    pub ignore_collision: bool,
    pub jump_yaw: Option<f32>,
    pub movement_yaw: Option<f32>,
    pub managed_jump_yaw: Option<f32>,
}
impl Default for ClientSprintQueries {
    fn default() -> Self {
        Self { movement_override: -1, omnidirectional: false, ignore_blindness: false,
            ignore_hunger: false, ignore_collision: false, jump_yaw: None,
            movement_yaw: None, managed_jump_yaw: None }
    }
}
impl ClientSprintQueries {
    pub(crate) fn for_body(self, client_side: bool, server_copy: bool, replaying: bool) -> Self {
        if client_side && !server_copy && !replaying { self } else { Self::default() }
    }
    fn movement(self, original: bool) -> bool {
        match self.movement_override { 0 => false, 1 => true, _ => original }
    }
    fn forward(self, axes: [f32; 2]) -> bool {
        if self.omnidirectional { axes[0].abs() > 1.0e-5 || axes[1].abs() > 1.0e-5 }
        else { axes[1] > 1.0e-5 }
    }
    fn start_forward(self, axes: [f32; 2], under_water: bool) -> bool {
        if !self.omnidirectional || under_water { return self.forward(axes); }
        f64::from(axes[0].abs()) >= 0.8 || f64::from(axes[1].abs()) >= 0.8
    }
}
'''

TICK_WRAPPER = '''        self.tick_with_queries(keys, conditions, window, ClientSprintQueries::default(), set_sprinting)
    }
    /// Separate event outcomes for canStartSprinting, Input.sprint and forced stop.
    pub fn tick_with_queries<E>(
        &mut self,
        keys: Keys,
        mut conditions: Conditions,
        window: i32,
        queries: ClientSprintQueries,
        mut set_sprinting: impl FnMut(bool) -> Result<(), E>,
    ) -> Result<(), E> {
        conditions.mobility_restricted &= !queries.ignore_blindness;
        conditions.enough_food |= queries.ignore_hunger;
        conditions.horizontal_collision &= !queries.ignore_collision;
'''

def apply(edit, replace_once):
    edit("crates/horizon-client/src/game/native_hooks.rs", lambda text: replace_once(
        text, "    /// Player FOV scale, after vanilla effects and before zoom and easing.\n",
        HOOKS + "    /// Player FOV scale, after vanilla effects and before zoom and easing.\n"))

    def sprint(text):
        text = replace_once(text, "    pub previous_forward: bool,\n", "    pub previous_forward: bool,\n    pub previous_axes: [f32; 2],\n")
        text = replace_once(text, "        mut set_sprinting: impl FnMut(bool) -> Result<(), E>,\n", "        set_sprinting: impl FnMut(bool) -> Result<(), E>,\n")
        text = replace_once(text, "    ) -> Result<(), E> {\n        if self.trigger_time > 0 {\n", "    ) -> Result<(), E> {\n" + TICK_WRAPPER + "        if self.trigger_time > 0 {\n")
        text = replace_once(text, "        let old_forward = self.previous_forward;\n", "        let old_forward = if queries.omnidirectional { queries.forward(self.previous_axes) } else { self.previous_forward };\n")
        text = replace_once(text, "        let forward = keys.direction()[1] > 1.0e-5;\n        self.previous_forward = forward;\n", "        let axes = keys.direction();\n        let forward = queries.forward(axes);\n        self.previous_forward = axes[1] > 1.0e-5;\n        self.previous_axes = axes;\n")
        text = replace_once(text, "        if conditions.can_start(forward) {\n", "        if queries.movement(conditions.can_start(queries.start_forward(axes, conditions.under_water))) {\n")
        text = replace_once(text, "            if keys.sprint {\n", "            if queries.movement(keys.sprint) {\n")
        text = replace_once(text, "        if sprinting && conditions.should_stop(forward, keys.shift) {\n", "        if sprinting && (conditions.should_stop(forward, keys.shift) || !queries.movement(true)) {\n")
        return text + "\n" + QUERIES
    edit("crates/horizon-client/src/sprint.rs", sprint)

    def motion(text):
        text = replace_once(text, "    pub sprint_input: crate::sprint::SprintState,\n", "    pub sprint_input: crate::sprint::SprintState,\n    pub client_sprint_queries: crate::sprint::ClientSprintQueries,\n")
        text = replace_once(text, "    fn run_sprint_tick(\n", "    pub(crate) fn client_sprint_queries(&self) -> crate::sprint::ClientSprintQueries {\n        self.client_sprint_queries.for_body(self.client_side, self.server_copy, self.replaying)\n    }\n    fn run_sprint_tick(\n")
        text = replace_once(text, "        let result = state.tick(keys, conditions, sprint_window.clamp(0, 10), |value| {\n", "        let queries = self.client_sprint_queries();\n        let result = state.tick_with_queries(keys, conditions, sprint_window.clamp(0, 10), queries, |value| {\n")
        original = "    fn yaw(&mut self) -> Result<f32> {\n        Ok(self.player.yaw)\n    }\n"
        if text.count(original) != 2: raise ValueError("Expected the two distinct LivingAir/Motion yaw implementations")
        text = text.replace(original, "    fn yaw(&mut self) -> Result<f32> {\n        Ok(self.player.client_sprint_queries().movement_yaw.unwrap_or(self.player.yaw))\n    }\n", 1)
        text = replace_once(text, original, '''    fn yaw(&mut self) -> Result<f32> {
        Ok(self.player.client_sprint_queries().jump_yaw.unwrap_or(self.player.yaw))
    }
    fn sprint_jump_boost(&mut self) -> Result<Vec3> {
        use horizon_core::geometry::trig;
        let queries = self.player.client_sprint_queries();
        let yaw = queries.managed_jump_yaw.or(queries.jump_yaw).unwrap_or(self.player.yaw);
        let angle = f64::from(yaw * trig::DEG_TO_RAD);
        Ok(if queries.managed_jump_yaw.is_some() {
            Vec3::new(f64::from(-trig::sin(angle) * 0.2f32), 0.0, f64::from(trig::cos(angle) * 0.2f32))
        } else {
            Vec3::new(f64::from(-trig::sin(angle)) * 0.2, 0.0, f64::from(trig::cos(angle)) * 0.2)
        })
    }
''')
        return text
    edit("crates/horizon-client/src/motion.rs", motion)
    edit("crates/horizon-client/src/motion/body.rs", lambda text: replace_once(text,
        "            sprint_input: crate::sprint::SprintState::default(),\n",
        "            sprint_input: crate::sprint::SprintState::default(),\n            client_sprint_queries: crate::sprint::ClientSprintQueries::default(),\n"))

    def living_motion(text):
        text = replace_once(text, "    fn yaw(&mut self) -> Result<f32, Self::Error>;\n", '''    fn yaw(&mut self) -> Result<f32, Self::Error>;
    /// Default retains the original double multiplication for shared entities.
    fn sprint_jump_boost(&mut self) -> Result<Vec3, Self::Error> {
        let angle = f64::from(self.yaw()? * trig::DEG_TO_RAD);
        Ok(Vec3::new(f64::from(-trig::sin(angle)) * 0.2, 0.0, f64::from(trig::cos(angle)) * 0.2))
    }
''')
        return replace_once(text, '''        let angle = f64::from(world.yaw()? * trig::DEG_TO_RAD);
        add_delta_movement(
            world,
            Vec3::new(
                f64::from(-trig::sin(angle)) * 0.2,
                0.0,
                f64::from(trig::cos(angle)) * 0.2,
            ),
        )?;
''', '''        let boost = world.sprint_jump_boost()?;
        add_delta_movement(world, boost)?;
''')
    edit("crates/horizon-world/src/living_motion.rs", living_motion)

    def remote(text):
        text = replace_once(text, "        let input = u8::from(keys.forward)\n", '''        let directional_before = [keys.forward, keys.backward, keys.left, keys.right];
        self.native_client_directional_input(&mut keys.forward, &mut keys.backward, &mut keys.left, &mut keys.right, &mut keys.jump, &mut keys.shift, &mut keys.sprint);
        if directional_before != [keys.forward, keys.backward, keys.left, keys.right] {
            keys.analog = None;
        }
        let input = u8::from(keys.forward)
''')
        text = replace_once(text, "        // A passenger moves with its vehicle (Entity.rideTick), not by itself.\n", '''        let [sideways, forward] = keys.direction();
        let mut sprint_queries = horizon_client::sprint::ClientSprintQueries::default();
        self.client_sprint_queries_for_input(sideways, forward, keys.shift,
            &mut sprint_queries.movement_override, &mut sprint_queries.omnidirectional,
            &mut sprint_queries.ignore_blindness, &mut sprint_queries.ignore_hunger,
            &mut sprint_queries.ignore_collision, &mut sprint_queries.jump_yaw,
            &mut sprint_queries.movement_yaw, &mut sprint_queries.managed_jump_yaw);
        if let Some(player) = self.player.as_mut() {
            player.client_side = true;
            player.client_sprint_queries = sprint_queries;
        }
        // A passenger moves with its vehicle (Entity.rideTick), not by itself.
''')
        text = replace_once(text, "        let (pos, on_ground, collision, sprinting) = (\n", "        let (pos, on_ground, collision, actual_sprinting) = (\n")
        text = replace_once(text, "        let (yaw, pitch) = (self.yaw, self.pitch);\n        let riding = self.riding.is_some();\n", "        let (mut yaw, mut pitch) = (self.yaw, self.pitch);\n        self.native_client_rotation_network(&mut yaw, &mut pitch);\n        let mut sprinting = actual_sprinting;\n        self.native_client_sprint_network(&mut sprinting);\n        let riding = self.riding.is_some();\n")
        # Recorded entity metadata represents actual avatar state, whereas the
        # network sprint event is scoped exclusively to PlayerCommand decisions.
        return replace_once(text, "            if sprinting {\n                shared |= 0x08;\n", "            if actual_sprinting {\n                shared |= 0x08;\n")
    edit("crates/horizon-client/src/game/remote.rs", remote)
    edit("crates/horizon-client/src/first_person.rs", lambda text: replace_once(text,
        "impl HandSway {\n", "impl HandSway {\n    pub fn reset_yaw(&mut self, yaw: f32) {\n        self.y_bob = yaw;\n        self.y_bob_o = yaw;\n        self.y_rot = yaw;\n    }\n"))

    def perspective(text):
        original = '''        let Some(m) = self.player_model.as_ref() else {
            return;
        };
        let eye = self
'''
        replacement = '''        let Some(m) = self.player_model.as_ref() else {
            return;
        };
        let partial = alpha as f32;
        let body = horizon_client::mobs::anim::body_rot(m, partial);
        let wrap = horizon_client::camera::wrap_degrees;
        let head = m.previous_head_yaw + wrap(m.head_yaw - m.previous_head_yaw) * partial;
        let pitch = m.previous_head_pitch + (m.head_pitch - m.previous_head_pitch) * partial;
        let (mut rendered_body, mut rendered_head, mut rendered_pitch) = (body, head, pitch);
        self.native_client_rotation_model(partial, &mut rendered_body, &mut rendered_head, &mut rendered_pitch);
        // Only the rendering view is changed. Simulation body turn and rider
        // orientation remain untouched, and other animations keep their partial.
        let rendered_model = if [body, head, pitch] != [rendered_body, rendered_head, rendered_pitch] {
            let mut rendered = m.clone();
            rendered.extra.vehicle_body = None;
            rendered.body_yaw = rendered_body;
            rendered.previous_body_yaw = rendered_body;
            rendered.head_yaw = rendered_head;
            rendered.previous_head_yaw = rendered_head;
            rendered.head_pitch = rendered_pitch;
            rendered.previous_head_pitch = rendered_pitch;
            Some(rendered)
        } else { None };
        let m = rendered_model.as_ref().unwrap_or(m);
        let eye = self
'''
        return replace_once(text, original, replacement)
    edit("crates/horizon-client/src/game/perspective.rs", perspective)
    return {"hook_names": HOOK_NAMES, "helper_names": HELPER_NAMES}
