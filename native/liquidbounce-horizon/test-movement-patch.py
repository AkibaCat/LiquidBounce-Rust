#!/usr/bin/env python3
"""Compile the generated query/shape code against Horizon's real ray math.

The tiny Game fixture supplies stored blocks and precomputed original outline
hits. The code under test is taken directly from movement-patch.py, and the
ray/voxel implementation is compiled from untouched Horizon source.
"""
from pathlib import Path
import json
import runpy
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
OUT = ROOT / "build/query-hook-tests"

FIXTURE = r'''
#![cfg(test)]
extern crate self as horizon_world;

pub mod entity_obstruction {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct EntityIdentity(pub u64);
}

#[allow(dead_code)]
#[path = "__WORLD_RAY__"]
pub mod world_ray;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fluid { None, Water, Lava }

#[derive(Clone, Copy, Debug, PartialEq)]
struct Hit { pos: [i32; 3], face: [i32; 3], point: [f64; 3] }

struct Facts { fluid: Fluid }
struct Registry([Facts; 3]);
impl Registry {
    fn facts(&self) -> &Self { self }
    fn get(&self, state: usize) -> &Facts { &self.0[state] }
}
struct World { cells: std::collections::BTreeMap<[i32; 3], (usize, f32, bool)> }
impl World {
    fn block(&self, x: i32, y: i32, z: i32) -> usize {
        self.cells.get(&[x,y,z]).map_or(0, |cell| cell.0)
    }
}
struct Game<'r> {
    world: World,
    res: Registry,
    player: Option<()>,
    native: bool,
    camera: bool,
    enabled: bool,
    block: Option<Hit>,
    marker: std::marker::PhantomData<&'r ()>,
}
impl Game<'_> {
    fn new() -> Self {
        Self {
            world: World { cells: Default::default() },
            res: Registry([Facts {fluid: Fluid::None}, Facts {fluid: Fluid::Water}, Facts {fluid: Fluid::Lava}]),
            player: Some(()), native: true, camera: true, enabled: true,
            block: None, marker: Default::default(),
        }
    }
    fn clip_ray(&self, _: [f64;3], _: [f64;3], _: f64) -> Option<Hit> { self.block }
    fn native_window(&self) -> bool { self.native }
    fn controls_camera(&self) -> bool { self.camera }
    fn native_client_pick_fluid(&self, any: &mut bool) { *any = self.enabled; }
    fn fluid_height_of(&self, x: i32, y: i32, z: i32, _: Option<Fluid>) -> Option<(f32,bool)> {
        self.world.cells.get(&[x,y,z]).map(|cell| (cell.1,cell.2))
    }
}
'''

TESTS = r'''
#[cfg(test)]
mod integration_tests {
    use super::*;

    fn downward(game: &Game<'_>) -> Option<Hit> {
        game.pick_camera_ray([0.5, 2.0, 0.5], [0.0,-1.0,0.0], 4.5)
    }
    #[test]
    fn source_and_flowing_liquids_use_surface_height_and_any_mode() {
        let mut game = Game::new();
        game.world.cells.insert([0,0,0], (1,8.0/9.0,true));
        let hit = downward(&game).unwrap();
        assert_eq!(hit.pos,[0,0,0]);
        assert_eq!(hit.face,[0,1,0]);
        assert!((hit.point[1] - f64::from(8.0f32/9.0)).abs()<1e-10);
        game.world.cells.insert([0,0,0], (1,2.0/9.0,false));
        let flowing = downward(&game).unwrap();
        assert!((flowing.point[1] - f64::from(2.0f32/9.0)).abs()<1e-10);
        // A ray above the partial fluid must pass through, rather than hit a
        // full voxel as the old generic fluids=true traversal would do.
        assert!(game.pick_camera_ray([-1.0,0.8,0.5],[1.0,0.0,0.0],4.5).is_none());
        game.world.cells.insert([0,0,0], (2,4.0/9.0,false));
        assert!(downward(&game).is_some()); // ANY also includes flowing lava.
    }
    #[test]
    fn nearest_fluid_and_outline_hit_share_one_filtered_target() {
        let mut game = Game::new();
        game.world.cells.insert([0,0,0], (1,8.0/9.0,true));
        let fluid = downward(&game).unwrap();
        game.block = Some(Hit {pos:[0,1,0],face:[0,1,0],point:[0.5,1.5,0.5]});
        assert_eq!(downward(&game),game.block);
        game.block = Some(Hit {pos:[0,-1,0],face:[0,1,0],point:[0.5,-0.5,0.5]});
        assert_eq!(downward(&game),Some(fluid));
        let equal = Hit {face:[1,0,0],..fluid};
        game.block = Some(equal);
        assert_eq!(downward(&game),Some(equal)); // Solid wins a waterlogged tie.
    }
    #[test]
    fn only_the_actual_local_player_camera_adds_fluid_targets() {
        let mut game = Game::new();
        game.world.cells.insert([0,0,0], (1,8.0/9.0,true));
        assert!(downward(&game).is_some());
        game.enabled = false;
        assert!(downward(&game).is_none());
        game.enabled = true;
        game.native = false;
        assert!(downward(&game).is_none());
        game.native = true;
        game.camera = false;
        assert!(downward(&game).is_none());
        game.camera = true;
        game.player = None;
        assert!(downward(&game).is_none());
        // Shared item/boat/bucket callers still receive the original block
        // trace, even when a client module would add a camera fluid hit.
        assert!(game.clip_ray([0.5,2.0,0.5],[0.0,-1.0,0.0],4.5).is_none());
    }
}
'''


def method(text, signature):
    """Extract the complete transformed production method, retaining its body."""
    if text.count(signature) != 1:
        raise ValueError(f"Expected one production method: {signature}")
    start = text.index(signature)
    opening = text.index("{", start)
    depth = 1
    end = opening + 1
    while depth:
        depth += (text[end] == "{") - (text[end] == "}")
        end += 1
    return text[start:end]


def mob_fixture(changed, ns):
    living = changed["crates/horizon-client/src/mobs/living.rs"]
    ai = changed["crates/horizon-client/src/mobs/ai.rs"]
    landing = changed["crates/horizon-client/src/mobs/landing.rs"]
    replacements = {
        "MOB_QUERIES": ns["MOB_QUERIES"],
        "GRAVITY": method(living, "pub fn gravity(&self) -> f64"),
        "EFFECTIVE_GRAVITY": method(living, "pub fn effective_gravity(&self) -> f64"),
        "MODIFIED_FRICTION": method(living, "fn modified_friction(friction: f32, modifier: f32) -> f32"),
        "AIR_DRAG_MODIFIER": method(living, "fn air_drag_modifier(&self) -> f32"),
        "CLIMBABLE_VELOCITY": method(living, "fn climbable_velocity(v: Vec3) -> Vec3"),
        "TRAVEL_IN_AIR": method(living, "fn travel_in_air(&mut self, w: &MobWorld, input: Vec3, events: &mut Vec<MobEvent>)"),
        "RESET_EFFECT_FALL_DISTANCE": method(ai, "fn reset_effect_fall_distance(m: &mut Mob)"),
        "AI_STEP": method(ai, "fn ai_step(m: &mut Mob, w: &MobWorld, level_random: &mut LegacyRandom, events: &mut Vec<MobEvent>)"),
        "RESTITUTION_SUPPRESS": method(living, "fn suppressing_bounce(&mut self) -> Result<bool, ()>"),
        "LANDING_SUPPRESS": method(landing, "fn suppressing_bounce(&mut self) -> Result<bool, String>"),
    }
    fixture = (HERE / "movement-mob-fixture.rs").read_text(encoding="utf-8")
    for name, code in replacements.items():
        placeholder = f"__{name}__"
        if fixture.count(placeholder) != 1:
            raise ValueError(f"Expected one fixture placeholder: {placeholder}")
        fixture = fixture.replace(placeholder, code)
    mount = changed["crates/horizon-client/src/game/remote_vehicles.rs"]
    # The actual drive path installs the hook snapshot before mob.tick and
    # discards it immediately after, before reinserting/cloning the body.
    drive = method(mount, "fn drive_mount(&mut self, i: usize, keys: Keys, jump: f32)")
    assert drive.index("self.native_client_movement_queries(") < drive.index("let m = &mut self.mobs[i];")
    assert drive.index("mob.extra.client_movement_queries = client_movement_queries;") < drive.index("mob.tick(")
    assert drive.index("mob.tick(") < drive.index("mob.extra.client_movement_queries = Default::default();") < drive.index("self.mobs.insert(i, mob);")
    # NoAI still enters LivingEntity.aiStep; its separate simulation route must
    # also reset the cooldown without mutating stored effects.
    assert "if m.no_ai {\n        if m.client_movement_queries().reset_jump_delay {\n            m.ai.no_jump_delay = 0;\n        }\n" in ai
    return fixture, list(replacements)


def player_fixture(changed):
    motion = changed["crates/horizon-client/src/motion.rs"]
    landing = changed["crates/horizon-client/src/landing.rs"]
    query = method(motion, "pub(crate) fn client_movement_queries(&self) -> ClientMovementQueries")
    movement_bounce = method(motion, "fn suppressing_bounce(&mut self) -> Result<bool>")
    landing_bounce = method(landing, "fn suppressing_bounce(&mut self) -> Result<bool>")
    return r'''
#[cfg(test)]
mod player_landing_production_tests {
    use crate::ClientMovementQueries;
    type Result<T> = std::result::Result<T, ()>;
    struct PlayerMotion {
        client_side: bool, server_copy: bool, replaying: bool,
        shift_key_down: bool, client_movement_queries: ClientMovementQueries,
    }
    impl PlayerMotion { __QUERY__ }
    trait Bounce { fn suppressing_bounce(&mut self) -> Result<bool>; }
    struct MotionWorld<'a> { player: &'a PlayerMotion }
    struct Landing<'a> { player: &'a PlayerMotion }
    impl Bounce for MotionWorld<'_> { __MOVEMENT_BOUNCE__ }
    impl Bounce for Landing<'_> { __LANDING_BOUNCE__ }
    #[test]
    fn player_landing_and_restitution_share_scope_without_changing_sneak() {
        for (client_side, server_copy, replaying, active) in [
            (true,false,false,true), (false,false,false,false),
            (true,true,false,false), (true,false,true,false),
        ] {
            let mut p = PlayerMotion {
                client_side, server_copy, replaying, shift_key_down:false,
                client_movement_queries: ClientMovementQueries {suppress_bounce:true,..Default::default()},
            };
            assert_eq!(MotionWorld {player:&p}.suppressing_bounce().unwrap(), active);
            assert_eq!(Landing {player:&p}.suppressing_bounce().unwrap(), active);
            assert!(!p.shift_key_down);
            p.shift_key_down = true;
            assert!(MotionWorld {player:&p}.suppressing_bounce().unwrap());
            assert!(Landing {player:&p}.suppressing_bounce().unwrap());
        }
    }
}
'''.replace("__QUERY__", query).replace("__MOVEMENT_BOUNCE__", movement_bounce).replace("__LANDING_BOUNCE__", landing_bounce)


def main():
    ns = runpy.run_path(str(HERE / "movement-patch.py"))
    changed = {}

    def replace_once(text, original, replacement):
        if text.count(original) != 1:
            raise ValueError(f"Expected one source occurrence: {original[:100]}")
        return text.replace(original, replacement, 1)

    def edit(path, transform):
        original = changed.get(path)
        if original is None:
            original = (ROOT / "upstream/Horizon" / path).read_text(encoding="utf-8")
        changed[path] = transform(original)

    fragment = ns["apply"](edit, replace_once)
    OUT.mkdir(parents=True, exist_ok=True)
    manifest = '''[package]
name = "liquidbounce-query-hook-tests"
version = "0.1.0"
edition = "2024"

[dependencies]
horizon-core = { path = "__CORE__" }
'''.replace("__CORE__", (ROOT / "upstream/Horizon/crates/horizon-core").as_posix())
    (OUT / "Cargo.toml").write_text(manifest, encoding="utf-8")
    fixture = FIXTURE.replace("__WORLD_RAY__", (ROOT / "upstream/Horizon/crates/horizon-world/src/world_ray.rs").as_posix())
    (OUT / "src").mkdir(exist_ok=True)
    mounts, methods = mob_fixture(changed, ns)
    source = fixture + ns["QUERIES"] + "\nmod ray_implementation {\nuse super::*;\n" + ns["FLUID_RAY"] + "\n}\n" + TESTS + mounts + player_fixture(changed)
    (OUT / "src/lib.rs").write_text(source, encoding="utf-8")
    result = subprocess.run(["cargo", "+stable", "test", "--manifest-path", str(OUT / "Cargo.toml")], cwd=ROOT)
    evidence = {"pristinePaths": list(changed), "hookFragment": fragment, "exitCode": result.returncode, "productionMobMethods": methods, "scope": "generated_query_and_fluid_shape_code_with_real_Horizon_world_ray_and_voxel_math_and_complete_transformed_mob_methods"}
    (OUT / "result.json").write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
    raise SystemExit(result.returncode)


if __name__ == "__main__":
    main()
