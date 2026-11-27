#!/usr/bin/env python3
"""Exact client query phases for six source-derived LiquidBounce modules.

Import this helper and call ``apply(edit, replace_once)``. ``edit`` must compose
with previous edits to the same path, rather than reload pristine text each
time. The returned hook fragment is merged into the caller's fragment.
No upstream file is written by this module.
"""

HOOK_NAMES = [
    "client_movement_queries",
    "client_pick_fluid",
    "client_portal_allowed",
    "client_player_reduced_debug",
]

HOOKS = '''    /// Client simulation policy captured once for the window's actual input.
    /// The body retains this snapshot; server copies and speculative server
    /// replay never apply it. AirJump/Scaffold ownership is resolved by the mod.
    pub(super) fn native_client_movement_queries(&self, reset_jump_delay: &mut bool, suppress_bounce: &mut bool, ignore_levitation: &mut bool, ignore_slow_falling: &mut bool) {
        if !self.native_window() { return; }
        mod_api::dispatch_client_movement_queries(self, mod_api::mods(), reset_jump_delay, suppress_bounce, ignore_levitation, ignore_slow_falling);
    }
    /// Only the local player camera's block pick, not item/fluid-specific rays.
    pub(super) fn native_client_pick_fluid(&self, any: &mut bool) {
        if !self.native_window() { return; }
        mod_api::dispatch_client_pick_fluid(self, mod_api::mods(), any);
    }
    /// Screen.isAllowedInPortal inside the local portal-transition effect.
    pub(super) fn native_client_portal_allowed(&self, allowed: &mut bool) {
        if !self.native_window() { return; }
        mod_api::dispatch_client_portal_allowed(self, mod_api::mods(), allowed);
    }
    /// The player/server-rule query, before the separate local option OR.
    pub(super) fn native_client_player_reduced_debug(&self, reduced: &mut bool) {
        if !self.native_window() { return; }
        mod_api::dispatch_client_player_reduced_debug(self, mod_api::mods(), reduced);
    }

'''

QUERIES = '''/// The window's decisions for one actual client input. These flags do
/// not mutate input keys or stored status effects, and survive a body clone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClientMovementQueries {
    pub reset_jump_delay: bool,
    pub suppress_bounce: bool,
    pub ignore_levitation: bool,
    pub ignore_slow_falling: bool,
}
impl ClientMovementQueries {
    /// `replaying` is server movement/anticheat prediction, not replay of a
    /// local client's history (MotionWorld.replayed_tick). Never let a local
    /// module change a guest, bot or server's simulation in this process.
    pub(crate) fn for_body(self, client_side: bool, server_copy: bool, replaying: bool) -> Self {
        if client_side && !server_copy && !replaying {
            self
        } else {
            Self::default()
        }
    }
}

#[cfg(test)]
mod client_movement_query_tests {
    use super::ClientMovementQueries;

    #[test]
    fn input_snapshot_survives_clone_but_never_changes_server_or_replay() {
        let actual = ClientMovementQueries {
            reset_jump_delay: true,
            suppress_bounce: true,
            ignore_levitation: true,
            ignore_slow_falling: true,
        };
        let retained = actual;
        assert_eq!(retained.for_body(true, false, false), actual);
        for (client_side, server_copy, replaying) in [
            (false, false, false),
            (true, true, false),
            (true, false, true),
            (false, true, true),
        ] {
            assert_eq!(
                retained.for_body(client_side, server_copy, replaying),
                ClientMovementQueries::default()
            );
        }
        // A server query never clears the retained client input snapshot.
        assert_eq!(retained.for_body(true, false, false), actual);
    }
}

'''

MOB_QUERIES = '''
impl Mob {
    /// Only the controller's client simulation receives this scoped snapshot.
    /// Server mounts mark their rider as `client`, even if copied from a client.
    pub(super) fn client_movement_queries(&self) -> crate::motion::ClientMovementQueries {
        self.extra.client_movement_queries.for_body(
            self.extra.client_side,
            self.rider.is_some_and(|r| r.client),
            false,
        )
    }
}
'''

FLUID_RAY = '''
/// A fluid-only provider: retain the existing exact block outline traversal
/// and compare its hit against FluidState.getShape, rather than treating a
/// partial/flowing liquid as a full block. Solid hits win equal distances.
struct ClientPickFluidRay<'a, 'r>(&'a Game<'r>);
impl horizon_world::world_ray::RayWorld for ClientPickFluidRay<'_, '_> {
    type Error = std::io::Error;
    type Block = ();
    type Fluid = Option<(f32, bool, Fluid)>;
    fn block_state(&mut self, _: horizon_core::position::BlockPos) -> std::io::Result<()> {
        Ok(())
    }
    fn fluid_state(&mut self, p: horizon_core::position::BlockPos) -> std::io::Result<Self::Fluid> {
        Ok(self.0.fluid_height_of(p.x, p.y, p.z, None).map(|(height, source)| {
            (height, source, self.0.res.facts().get(self.0.world.block(p.x, p.y, p.z)).fluid)
        }))
    }
    fn block_shape(&mut self, _: (), _: horizon_core::position::BlockPos, _: horizon_world::world_ray::RayBlockMode) -> std::io::Result<horizon_core::geometry::VoxelShape> {
        Ok(horizon_core::geometry::VoxelShape::empty())
    }
    fn interaction_shape(&mut self, _: (), _: horizon_core::position::BlockPos) -> std::io::Result<horizon_core::geometry::VoxelShape> {
        Ok(horizon_core::geometry::VoxelShape::empty())
    }
    fn fluid_is_source(&mut self, fluid: Self::Fluid) -> std::io::Result<bool> {
        Ok(fluid.is_some_and(|f| f.1))
    }
    fn fluid_is_empty(&mut self, fluid: Self::Fluid) -> std::io::Result<bool> {
        Ok(fluid.is_none())
    }
    fn fluid_is_water(&mut self, fluid: Self::Fluid) -> std::io::Result<bool> {
        Ok(fluid.is_some_and(|f| f.2 == Fluid::Water))
    }
    fn fluid_shape(&mut self, fluid: Self::Fluid, _: horizon_core::position::BlockPos) -> std::io::Result<horizon_core::geometry::VoxelShape> {
        use horizon_core::geometry::{Aabb, Vec3, VoxelShape};
        match fluid {
            None => Ok(VoxelShape::empty()),
            Some((height, _, _)) => VoxelShape::from_box(Aabb::new(
                Vec3::ZERO,
                Vec3::new(1.0, f64::from(height), 1.0),
            )).map_err(|error| std::io::Error::other(format!("{error:?}"))),
        }
    }
}

impl Game<'_> {
    /// LocalPlayer.pick's own-player camera ray. Item, bucket, boat and
    /// third-person collision rays continue through their original methods.
    pub(super) fn pick_camera_ray(&self, origin: [f64; 3], dir: [f64; 3], reach: f64) -> Option<Hit> {
        let block = self.clip_ray(origin, dir, reach);
        let mut any = false;
        if self.native_window() && self.controls_camera() && self.player.is_some() {
            self.native_client_pick_fluid(&mut any);
        }
        if !any {
            return block;
        }
        use horizon_core::geometry::Vec3;
        use horizon_world::world_ray::{RayBlockMode, RayFluidMode, clip_world};
        let start = Vec3::new(origin[0], origin[1], origin[2]);
        let end = start.add_vector(Vec3::new(dir[0], dir[1], dir[2]).scale(reach));
        let fluid = clip_world(
            &mut ClientPickFluidRay(self), start, end,
            RayBlockMode::Outline, RayFluidMode::Any, 4096,
        ).ok().filter(|result| !result.miss).map(|result| Hit {
            pos: [result.hit.block.x, result.hit.block.y, result.hit.block.z],
            face: result.hit.direction.normal(),
            point: [result.hit.location.x, result.hit.location.y, result.hit.location.z],
        });
        match (block, fluid) {
            (Some(block), Some(fluid)) => {
                let squared = |hit: &Hit| hit.point.iter().zip(origin).map(|(v, o)| (v - o).powi(2)).sum::<f64>();
                Some(if squared(&block) <= squared(&fluid) { block } else { fluid })
            }
            (block, fluid) => block.or(fluid),
        }
    }
}
'''


def apply(edit, replace_once):
    """Compose the exact source transforms and return the four-hook fragment."""
    edit("crates/horizon-client/src/game/native_hooks.rs", lambda text: replace_once(
        text,
        "    /// Player FOV scale, after vanilla effects and before zoom and easing.\n",
        HOOKS + "    /// Player FOV scale, after vanilla effects and before zoom and easing.\n",
    ))

    def motion(text):
        text = replace_once(text, "#[derive(Clone)]\npub struct PlayerMotion {\n", QUERIES + "#[derive(Clone)]\npub struct PlayerMotion {\n")
        text = replace_once(text, "    pub no_jump_delay: i32,\n", "    pub no_jump_delay: i32,\n    pub client_movement_queries: ClientMovementQueries,\n")
        text = replace_once(text, "impl PlayerMotion {\n", '''impl PlayerMotion {
    pub(crate) fn client_movement_queries(&self) -> ClientMovementQueries {
        self.client_movement_queries.for_body(self.client_side, self.server_copy, self.replaying)
    }
''')
        text = replace_once(text, "    fn dry_tick(&mut self, input: TickInput) -> Result<()> {\n", '''    fn dry_tick(&mut self, input: TickInput) -> Result<()> {
        if self.player.client_movement_queries().reset_jump_delay {
            self.player.no_jump_delay = 0;
        }
''')
        text = replace_once(text, '''    fn suppressing_bounce(&mut self) -> Result<bool> {
        Ok(self.player.shift_key_down)
    }
''', '''    fn suppressing_bounce(&mut self) -> Result<bool> {
        Ok(self.player.shift_key_down || self.player.client_movement_queries().suppress_bounce)
    }
''')
        text = replace_once(text, '''    fn levitation_amplifier(&mut self) -> Result<Option<i32>> {
        Ok(self.player.status.levitation)
    }
''', '''    fn levitation_amplifier(&mut self) -> Result<Option<i32>> {
        Ok(if self.player.client_movement_queries().ignore_levitation {
            None
        } else {
            self.player.status.levitation
        })
    }
''')
        return replace_once(text, '''    fn has_slow_falling(&mut self) -> Result<bool> {
        Ok(self.player.status.slow_falling)
    }
''', '''    fn has_slow_falling(&mut self) -> Result<bool> {
        Ok(self.player.status.slow_falling && !self.player.client_movement_queries().ignore_slow_falling)
    }
''')

    edit("crates/horizon-client/src/motion.rs", motion)
    edit("crates/horizon-client/src/landing.rs", lambda text: replace_once(text, '''    fn suppressing_bounce(&mut self) -> Result<bool> {
        Ok(self.player.shift_key_down)
    }
''', '''    fn suppressing_bounce(&mut self) -> Result<bool> {
        Ok(self.player.shift_key_down || self.player.client_movement_queries().suppress_bounce)
    }
'''))
    edit("crates/horizon-client/src/motion/body.rs", lambda text: replace_once(
        text, "            no_jump_delay: 0,\n",
        "            no_jump_delay: 0,\n            client_movement_queries: ClientMovementQueries::default(),\n",
    ))

    def local_input(text):
        text = replace_once(text, '''        self.sync_player_body();
        // LivingEntity.tickDeath: the local player counts its deathTime once
''', '''        self.sync_player_body();
        let mut client_movement_queries = horizon_client::motion::ClientMovementQueries::default();
        self.native_client_movement_queries(
            &mut client_movement_queries.reset_jump_delay,
            &mut client_movement_queries.suppress_bounce,
            &mut client_movement_queries.ignore_levitation,
            &mut client_movement_queries.ignore_slow_falling,
        );
        // LivingEntity.tickDeath: the local player counts its deathTime once
''')
        return replace_once(text, '''        let was_gliding = player.fall_flying;
        let dead = self.screen == Screen::Dead;
''', '''        player.client_movement_queries = client_movement_queries;
        let was_gliding = player.fall_flying;
        let dead = self.screen == Screen::Dead;
''')

    edit("crates/horizon-client/src/game/remote.rs", local_input)

    def mount_input(text):
        text = replace_once(text, '''    fn drive_mount(&mut self, i: usize, keys: Keys, jump: f32) {
        let steer = self.steers_mount(&self.mobs[i]);
''', '''    fn drive_mount(&mut self, i: usize, keys: Keys, jump: f32) {
        let mut client_movement_queries = horizon_client::motion::ClientMovementQueries::default();
        self.native_client_movement_queries(
            &mut client_movement_queries.reset_jump_delay,
            &mut client_movement_queries.suppress_bounce,
            &mut client_movement_queries.ignore_levitation,
            &mut client_movement_queries.ignore_slow_falling,
        );
        let steer = self.steers_mount(&self.mobs[i]);
''')
        text = replace_once(text, '''        mob.extra.client_side = true;
        let mut random = self.world.random.clone();
''', '''        mob.extra.client_side = true;
        mob.extra.client_movement_queries = client_movement_queries;
        let mut random = self.world.random.clone();
''')
        return replace_once(text, '''            mob.tick(&world, &mut random, &mut events);
        }
        // Only the server's packets take the mount away.
''', '''            mob.tick(&world, &mut random, &mut events);
        }
        // The policy belongs only to this actual controlling-client tick.
        mob.extra.client_movement_queries = Default::default();
        // Only the server's packets take the mount away.
''')

    edit("crates/horizon-client/src/game/remote_vehicles.rs", mount_input)
    edit("crates/horizon-client/src/mobs.rs", lambda text: replace_once(
        text,
        "    pub client_side: bool, // `Level.isClientSide`: a client copy, which `Entity.hurt` leaves alone.\n",
        "    pub client_side: bool, // `Level.isClientSide`: a client copy, which `Entity.hurt` leaves alone.\n"
        "    pub client_movement_queries: crate::motion::ClientMovementQueries,\n",
    ) + MOB_QUERIES)

    def mob_ai(text):
        text = replace_once(text, '''    if m.no_ai {
        update_fall_flying(m, w, events);
''', '''    if m.no_ai {
        if m.client_movement_queries().reset_jump_delay {
            m.ai.no_jump_delay = 0;
        }
        update_fall_flying(m, w, events);
''')
        return replace_once(text, '''    if m.ai.no_jump_delay > 0 {
        m.ai.no_jump_delay -= 1;
    }
''', '''    if m.client_movement_queries().reset_jump_delay {
        m.ai.no_jump_delay = 0;
    }
    if m.ai.no_jump_delay > 0 {
        m.ai.no_jump_delay -= 1;
    }
''')

    edit("crates/horizon-client/src/mobs/ai.rs", mob_ai)

    def mob_living(text):
        text = replace_once(text, '''    /// The entity's own suppression is its sneak pose, and a mob never
    /// sneaks; the block's tag is `block_suppresses_bounce` below.
''', '''    /// A mob has no vanilla sneak suppression. A controlling client may
    /// supply its module policy; server copies retain the original false.
''')
        text = replace_once(text, '''        if self.velocity.y <= 0.0 && self.has_effect("minecraft:slow_falling") && !gravity.is_nan()
        {
''', '''        if self.velocity.y <= 0.0
            && self.has_effect("minecraft:slow_falling")
            && !self.client_movement_queries().ignore_slow_falling
            && !gravity.is_nan()
        {
''')
        text = replace_once(text, '''            .find(|e| e.0 == "minecraft:levitation")
        {
            y += (0.05 * f64::from(amp + 1) - movement.y) * 0.2;
''', '''            .find(|e| e.0 == "minecraft:levitation" && !self.client_movement_queries().ignore_levitation)
        {
            y += (0.05 * f64::from(amp + 1) - movement.y) * 0.2;
''')
        return replace_once(text, '''    fn suppressing_bounce(&mut self) -> Result<bool, ()> {
        Ok(false)
    }
''', '''    fn suppressing_bounce(&mut self) -> Result<bool, ()> {
        Ok(self.mob.client_movement_queries().suppress_bounce)
    }
''')

    edit("crates/horizon-client/src/mobs/living.rs", mob_living)
    edit("crates/horizon-client/src/mobs/landing.rs", lambda text: replace_once(text, '''    // A mob never sneaks, so nothing suppresses a slime block's bounce.
    fn suppressing_bounce(&mut self) -> Result<bool, String> {
        Ok(false)
    }
''', '''    // Match the same client-only policy used by movement restitution.
    fn suppressing_bounce(&mut self) -> Result<bool, String> {
        Ok(self.mob.client_movement_queries().suppress_bounce)
    }
'''))

    def camera_pick(text):
        text = replace_once(text, '''        self.clip_ray(origin, dir, self.block_reach.max(self.entity_reach))
            .filter(|hit| {
''', '''        self.pick_camera_ray(origin, dir, self.block_reach.max(self.entity_reach))
            .filter(|hit| {
''')
        return replace_once(text, '''        self.clip_ray(origin, dir, self.block_reach.max(self.entity_reach))
    }
    /// The first block outline hit within `reach` along the unit `dir`.
''', '''        self.pick_camera_ray(origin, dir, self.block_reach.max(self.entity_reach))
    }
    /// The first block outline hit within `reach` along the unit `dir`.
''')

    edit("crates/horizon-client/src/game.rs", camera_pick)
    edit("crates/horizon-client/src/game/clip_ray.rs", lambda text: text + FLUID_RAY)

    def portal(text):
        original = '''        match self.screen {
            Screen::Playing | Screen::Chat | Screen::Dead => {}
            Screen::Inventory | Screen::Creative => {
                self.close_container();
                self.carried = None;
                self.screen = Screen::Playing;
                self.capture(true);
            }
            _ => {
                self.screen = Screen::Playing;
                self.capture(true);
            }
        }
        if self.screen_fx.portal == 0.0 {
'''
        replacement = '''        let mut allowed = matches!(self.screen, Screen::Playing | Screen::Chat | Screen::Dead);
        self.native_client_portal_allowed(&mut allowed);
        if !allowed {
            match self.screen {
                Screen::Playing | Screen::Chat | Screen::Dead => {}
                Screen::Inventory | Screen::Creative => {
                    self.close_container();
                    self.carried = None;
                    self.screen = Screen::Playing;
                    self.capture(true);
                }
                _ => {
                    self.screen = Screen::Playing;
                    self.capture(true);
                }
            }
        }
        if self.screen_fx.portal == 0.0 {
'''
        return replace_once(text, original, replacement)

    edit("crates/horizon-client/src/game/nether_portal.rs", portal)
    edit("crates/horizon-client/src/game/debug_keys.rs", lambda text: replace_once(text, '''    pub(super) fn reduced_debug_info(&self) -> bool {
        self.settings.reduced_debug_info
            || self.ui.rules.bool("reduced_debug_info")
            || self.remote.as_ref().is_some_and(|r| r.reduced_debug_info)
    }
''', '''    pub(super) fn reduced_debug_info(&self) -> bool {
        let mut reduced = self.ui.rules.bool("reduced_debug_info")
            || self.remote.as_ref().is_some_and(|r| r.reduced_debug_info);
        self.native_client_player_reduced_debug(&mut reduced);
        self.settings.reduced_debug_info || reduced
    }
'''))

    return {"hook_names": list(HOOK_NAMES), "helper_names": []}
