#!/usr/bin/env python3
"""AirJump's three modes and the original NoPush query phases."""

HOOK_NAMES = ["client_airjump_push_queries", "client_player_jump"]
HELPER_NAMES = ["client_motion_state", "client_motion_velocity_y"]

HOOKS = '''    #[allow(clippy::too_many_arguments)]
    pub(super) fn native_client_airjump_push_queries(&self, physical_jump: bool, air_jump: &mut bool, double_jump_mode: &mut bool, ghost_block: &mut bool, allow_entities: &mut bool, allow_blocks: &mut bool, allow_fishing: &mut bool, allow_liquids: &mut bool) {
        if !self.native_window() { return; }
        mod_api::dispatch_client_airjump_push_queries(self, mod_api::mods(), physical_jump, air_jump, double_jump_mode, ghost_block, allow_entities, allow_blocks, allow_fishing, allow_liquids);
    }
    /// A source-compatible default, reused at independent push query sites.
    pub(super) fn native_client_airjump_push_policy(&self) -> [bool; 7] {
        let (mut air, mut double, mut ghost) = (false, false, false);
        let (mut entities, mut blocks, mut fishing, mut liquids) = (true, true, true, true);
        self.native_client_airjump_push_queries(self.keys.held(horizon_client::actions::Action::Jump), &mut air, &mut double, &mut ghost, &mut entities, &mut blocks, &mut fishing, &mut liquids);
        [air, double, ghost, entities, blocks, fishing, liquids]
    }
    /// Ordered record of the actual local PlayerJumpEvent's original ground.
    pub(super) fn native_client_player_jump(&self, on_ground: bool) {
        if !self.native_window() { return; }
        mod_api::dispatch_client_player_jump(self, mod_api::mods(), on_ground);
    }
    /// GameTick's actual option keys and player state, before input transforms.
    pub(super) fn native_client_motion_state(&self) -> [f64; 5] {
        let Some(p) = self.player.as_ref() else { return [0.0; 5]; };
        [f64::from(u8::from(p.ground.on_ground)), f64::from(u8::from(p.fluids.was_touching_water || p.is_in_lava())), f64::from(u8::from(self.keys.held(horizon_client::actions::Action::Jump))), f64::from(u8::from(self.keys.held(horizon_client::actions::Action::Sneak))), p.velocity.y]
    }
    pub(super) fn native_client_motion_velocity_y(&mut self, velocity_y: f64) -> bool {
        if !self.native_window() || !velocity_y.is_finite() { return false; }
        let Some(p) = self.player.as_mut() else { return false; };
        p.velocity.y = velocity_y;
        true
    }

'''

QUERIES = '''#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClientAirJumpPushQueries {
    pub air_jump: bool,
    pub double_jump_mode: bool,
    pub suppress_liquid_flow: bool,
}
impl ClientAirJumpPushQueries {
    pub(crate) fn for_body(self, client_side: bool, server_copy: bool, replaying: bool) -> Self {
        if client_side && !server_copy && !replaying { self } else { Self::default() }
    }
}

'''

GHOST_SCOPE = '''// A window's actual ClientLevel collision iteration. A lexical scope keeps
// this policy out of server worlds and other threads; unwinding restores it.
thread_local! {
    static CLIENT_GHOST_BLOCK_Y: std::cell::Cell<Option<i32>> = const { std::cell::Cell::new(None) };
}
pub fn with_client_ghost_block<R>(block_y: Option<i32>, run: impl FnOnce() -> R) -> R {
    struct Restore(Option<i32>);
    impl Drop for Restore {
        fn drop(&mut self) { CLIENT_GHOST_BLOCK_Y.with(|y| y.set(self.0)); }
    }
    let previous = CLIENT_GHOST_BLOCK_Y.with(|y| y.replace(block_y));
    let _restore = Restore(previous);
    run()
}
/// A local player's movement updates the value source BlockShapeEvent reads.
pub fn update_client_ghost_block_y(block_y: i32) {
    CLIENT_GHOST_BLOCK_Y.with(|y| { if y.get().is_some() { y.set(Some(block_y)); } });
}

'''


def apply(edit, replace_once):
    anchor = "    /// Player FOV scale, after vanilla effects and before zoom and easing.\n"
    edit("crates/horizon-client/src/game/native_hooks.rs", lambda text: replace_once(text, anchor, HOOKS + anchor))

    def motion(text):
        text = replace_once(text, "#[derive(Clone)]\npub struct PlayerMotion {\n", QUERIES + "#[derive(Clone)]\npub struct PlayerMotion {\n")
        text = replace_once(text, "    pub no_jump_delay: i32,\n", "    pub no_jump_delay: i32,\n    pub client_airjump_push_queries: ClientAirJumpPushQueries,\n")
        text = replace_once(text, "pub enum MotionEvent {\n", "pub enum MotionEvent {\n    ClientJump { on_ground: bool },\n")
        text = replace_once(text, "impl PlayerMotion {\n", '''impl PlayerMotion {
    pub(crate) fn client_airjump_push_queries(&self) -> ClientAirJumpPushQueries {
        self.client_airjump_push_queries.for_body(self.client_side, self.server_copy, self.replaying)
    }
''')
        text = replace_once(text, "    pub fn jump_from_ground(&mut self) -> Result<()> {\n        living_motion::jump_from_ground(self)?;\n", '''    pub fn jump_from_ground(&mut self) -> Result<()> {
        if self.player.client_side && !self.player.server_copy && !self.player.replaying {
            let on_ground = self.player.ground.on_ground;
            self.player.events.push(MotionEvent::ClientJump { on_ground });
            if self.player.client_airjump_push_queries.double_jump_mode && !on_ground {
                self.player.client_airjump_push_queries.air_jump = false;
            }
        }
        living_motion::jump_from_ground(self)?;
''')
        return replace_once(text, "    fn set_position(&mut self, p: Vec3) -> Result<()> {\n        self.player.position = p;\n", '''    fn set_position(&mut self, p: Vec3) -> Result<()> {
        self.player.position = p;
        if self.player.client_side && !self.player.server_copy && !self.player.replaying {
            horizon_world::block_collision_scan::update_client_ghost_block_y(p.y.floor() as i32);
        }
''')
    edit("crates/horizon-client/src/motion.rs", motion)
    edit("crates/horizon-client/src/motion/body.rs", lambda text: replace_once(text,
        "            no_jump_delay: 0,\n", "            no_jump_delay: 0,\n            client_airjump_push_queries: ClientAirJumpPushQueries::default(),\n"))
    edit("crates/horizon-client/src/motion/environment.rs", lambda text: replace_once(text,
        "    pub(super) fn jump_phase(&mut self) -> Result<()> {\n", '''    pub(super) fn jump_phase(&mut self) -> Result<()> {
        // LivingEntity.aiStep's injection runs at the jumping field read,
        // before vanilla's liquid/ground decision; cooldown is set even when
        // jumpFromGround itself returns early for a negligible jump power.
        if self.player.client_airjump_push_queries().air_jump && self.player.jumping && self.player.no_jump_delay == 0 {
            self.jump_from_ground()?;
            self.player.no_jump_delay = 10;
        }
'''))
    edit("crates/horizon-client/src/motion/fluids.rs", lambda text: replace_once(text,
        "    fn flow(&mut self, f: FluidState, p: BlockPos) -> Result<Vec3> {\n        flow_at(self, f, p)\n    }\n", '''    fn flow(&mut self, f: FluidState, p: BlockPos) -> Result<Vec3> {
        if self.player.client_airjump_push_queries().suppress_liquid_flow { return Ok(Vec3::ZERO); }
        flow_at(self, f, p)
    }
'''))

    def remote(text):
        text = replace_once(text, "    pub(super) fn tick_remote(&mut self) -> Result<()> {\n", '''    pub(super) fn tick_remote(&mut self) -> Result<()> {
        let ghost = self.native_window() && self.native_client_airjump_push_policy()[2];
        let floor = self.player.as_ref().filter(|_| ghost).map(|p| p.position.y.floor() as i32);
        horizon_world::block_collision_scan::with_client_ghost_block(floor, || self.tick_remote_with_motion_modules())
    }
    fn tick_remote_with_motion_modules(&mut self) -> Result<()> {
''')
        text = replace_once(text, "        // A passenger moves with its vehicle (Entity.rideTick), not by itself.\n", '''        let [air_jump, double_jump_mode, _, _, _, _, allow_liquids] = self.native_client_airjump_push_policy();
        if let Some(player) = self.player.as_mut() {
            player.client_side = true;
            player.client_airjump_push_queries = horizon_client::motion::ClientAirJumpPushQueries {
                air_jump, double_jump_mode, suppress_liquid_flow: !allow_liquids,
            };
        }
        // A passenger moves with its vehicle (Entity.rideTick), not by itself.
''')
        text = replace_once(text, "        self.position_riders();\n        // Entity.getFluidInteractionBox", '''        self.position_riders();
        if let Some(player) = self.player.as_ref() {
            horizon_world::block_collision_scan::update_client_ghost_block_y(player.position.y.floor() as i32);
        }
        // Entity.getFluidInteractionBox''')
        text = replace_once(text, "        for event in events {\n            match event {\n                horizon_client::motion::MotionEvent::Emission", '''        for event in events {
            match event {
                horizon_client::motion::MotionEvent::ClientJump { on_ground } => self.native_client_player_jump(on_ground),
                horizon_client::motion::MotionEvent::Emission''')
        text = replace_once(text, "        let order = |id: u32| r.ticked_before_player.contains(&id) == before_player;\n        for m in self.mobs.iter().filter(|m| order(m.id)) {\n", '''        let order = |id: u32| r.ticked_before_player.contains(&id) == before_player;
        let entities_allowed = self.native_client_airjump_push_policy()[3];
        for m in self.mobs.iter().filter(|m| entities_allowed && order(m.id)) {
''')
        text = replace_once(text, "                bounds.intersects(bb)\n                    && horizon_client::scoreboard::pushable_by(player_team, None, true),\n", "                entities_allowed && bounds.intersects(bb)\n                    && horizon_client::scoreboard::pushable_by(player_team, None, true),\n")
        return replace_once(text, "                self.remote_boat_event(p.id, p.event);\n", "                self.remote_boat_event(p.id, p.event);\n                if p.event == 31 { self.remote_hook_pull_event(p.id as u32); }\n")
    edit("crates/horizon-client/src/game/remote.rs", remote)
    edit("crates/horizon-client/src/game.rs", lambda text: replace_once(text,
        "    fn closest_space_nudge(&self) -> [Option<f64>; 2] {\n", "    fn closest_space_nudge(&self) -> [Option<f64>; 2] {\n        if !self.native_client_airjump_push_policy()[4] { return [None; 2]; }\n"))

    def mounts(text):
        text = replace_once(text, "    fn drive_mount(&mut self, i: usize, keys: Keys, jump: f32) {\n", "    fn drive_mount(&mut self, i: usize, keys: Keys, jump: f32) {\n        let client_air_jump = self.native_client_airjump_push_policy()[0];\n")
        text = replace_once(text, "        mob.extra.client_side = true;\n", "        mob.extra.client_side = true;\n        mob.extra.client_air_jump = client_air_jump;\n")
        return replace_once(text, "        mob.removed = false;\n", "        mob.extra.client_air_jump = false;\n        mob.removed = false;\n")
    edit("crates/horizon-client/src/game/remote_vehicles.rs", mounts)
    edit("crates/horizon-client/src/mobs.rs", lambda text: replace_once(text,
        "pub struct Extra {\n", "pub struct Extra {\n    pub client_air_jump: bool,\n"))
    edit("crates/horizon-client/src/mobs/living.rs", lambda text: replace_once(text,
        "    pub fn handle_jumping(&mut self, w: &MobWorld) {\n", '''    pub fn handle_jumping(&mut self, w: &MobWorld) {
        if self.extra.client_side && !self.rider.is_some_and(|r| r.client)
            && self.extra.client_air_jump && self.ai.jumping && self.ai.no_jump_delay == 0 {
            self.jump_from_ground(w);
            self.ai.no_jump_delay = 10;
        }
'''))

    def fishing(text):
        return replace_once(text, "    pub(super) fn spawn_remote_hook(&mut self, id: u32, at: Vec3, velocity: Vec3, owner: i32) {\n", '''    /// FishingHook.handleEntityEvent(31): only its hooked local player
    /// predicts this pull. Server velocity packets remain independent.
    pub(super) fn remote_hook_pull_event(&mut self, id: u32) {
        if !self.native_client_airjump_push_policy()[5] { return; }
        let Some(local_id) = self.remote.as_ref().map(|r| r.entity_id as u32) else { return; };
        let Some(hook) = self.remote_hooks.iter().find(|h| h.id == id) else { return; };
        if !matches!(hook.state, State::Hooked(target) if target == local_id) { return; }
        let Some(owner) = self.mobs.iter().find(|m| m.id == hook.owner).map(|m| m.position) else { return; };
        if let Some(player) = self.player.as_mut() {
            player.velocity = pull_hooked(player.velocity, hook.position, owner);
        }
    }
    pub(super) fn spawn_remote_hook(&mut self, id: u32, at: Vec3, velocity: Vec3, owner: i32) {
''')
    edit("crates/horizon-client/src/game/fishing.rs", fishing)

    def scan(text):
        text = replace_once(text, "#[derive(Clone, Copy, Debug)]\npub struct CollisionBlock", GHOST_SCOPE + "#[derive(Clone, Copy, Debug)]\npub struct CollisionBlock")
        text = replace_once(text, "    failed: bool,\n", "    failed: bool,\n    client_ghost_block_y: Option<i32>,\n")
        text = replace_once(text, "            failed: false,\n", "            failed: false,\n            client_ghost_block_y: CLIENT_GHOST_BLOCK_Y.with(std::cell::Cell::get),\n")
        text = replace_once(text, "            if facts.empty {\n", "            if facts.empty && self.client_ghost_block_y.is_none() {\n")
        # Outer-cell selection remains vanilla: GhostBlock changes the shape,
        # not whether the original block was large or a moving piston.
        text = replace_once(text, "            if kind != 0 && !facts.reaches {\n", "            if kind != 0 && !facts.reaches && self.client_ghost_block_y.is_none() {\n")
        text = replace_once(text, "            if block.empty_shape {\n", "            if block.empty_shape && !self.client_ghost_block_y.is_some_and(|y| p.y < y) {\n")
        text = replace_once(text, "            let shape = self\n                .world\n                .context_shape(block, p, &self.context)\n                .map_err(CollisionScanError::World)?;\n", '''            let original_shape = self
                .world
                .context_shape(block, p, &self.context)
                .map_err(CollisionScanError::World)?;
            let shape = if self.client_ghost_block_y.is_some_and(|y| p.y < y) {
                VoxelShape::block()
            } else { original_shape };
''')
        return text
    edit("crates/horizon-world/src/block_collision_scan.rs", scan)
    return {"hook_names": HOOK_NAMES, "helper_names": HELPER_NAMES}
