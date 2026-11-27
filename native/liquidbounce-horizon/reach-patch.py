#!/usr/bin/env python3
"""ModuleReach's interaction getters, receiver bounds and LocalPlayer.pick."""
from pathlib import Path

HERE = Path(__file__).resolve().parent
HOOK_NAMES = ["client_interaction_range", "client_attack_range", "client_reach_policy", "client_rotation_aim"]
HELPER_NAMES = []
HOOKS = (HERE / "reach-host-methods.rs").read_text(encoding="utf-8")
PICKER = (HERE / "reach-picker.rs").read_text(encoding="utf-8")


def apply(edit, replace_once):
    edit("crates/horizon-client/src/game/native_hooks.rs", lambda text: replace_once(text,
        "    /// Player FOV scale, after vanilla effects and before zoom and easing.\n", HOOKS + "    /// Player FOV scale, after vanilla effects and before zoom and easing.\n"))
    edit("crates/horizon-client/src/motion.rs", lambda text: replace_once(text, "impl PlayerMotion {\n", "impl PlayerMotion {\n    pub fn native_client_reach_queries_allowed(&self) -> bool { !self.server_copy && !self.replaying }\n"))

    def game(text):
        text = replace_once(text, "/// How far along `dir` (within `reach`) a ray from `eye` enters `b`.\n", PICKER + "\n/// How far along `dir` (within `reach`) a ray from `eye` enters `b`.\n")
        text = replace_once(text, "        Some((self.eye_position()?, pick_view_vector(self.yaw, self.pitch)))\n", "        let (mut yaw, mut pitch, mut managed) = (self.yaw, self.pitch, false);\n        self.native_client_rotation_aim(&mut yaw, &mut pitch, &mut managed);\n        let direction = if managed { client_rotation_direction(yaw, pitch) } else { pick_view_vector(yaw, pitch) };\n        Some((self.eye_position()?, direction))\n")
        text = replace_once(text, "        self.mob_hit_within(self.entity_reach)\n", "        if let Some(hit) = self.client_reach_target() {\n            return (hit.kind == 0 && hit.distance_squared < self.block_reach * self.block_reach).then_some((hit.index, hit.distance, hit.location));\n        }\n        self.mob_hit_within(self.entity_reach)\n")
        for name in ["pick", "pick_unfiltered"]:
            start = text.index(f"    fn {name}(&self)")
            eye = text.index("        let (origin, dir) = self.aim()?;", start)
            text = text[:eye] + "        if self.client_reach_target().is_some() { return None; }\n" + text[eye:].replace("        let (origin, dir) = self.aim()?;", "        let (origin, dir) = self.client_block_aim()?;", 1)
        return text
    edit("crates/horizon-client/src/game.rs", game)

    def hand(text):
        text = replace_once(text, '''        self.player
            .as_mut()
            .and_then(|p| p.attribute(attribute).ok())
            .unwrap_or(otherwise)
''', '''        let original = self.player
            .as_mut()
            .and_then(|p| p.attribute(attribute).ok())
            .unwrap_or(otherwise);
        let mut range = original;
        self.native_client_interaction_range(attribute, original, &mut range);
        range
''')
        text = replace_once(text, "range.is_none_or(|r| attack_range_contains(&r, distance, self.creative))", "range.is_none_or(|r| self.client_attack_range_contains(&r, distance))")
        text = replace_once(text, "&& !attack_range_contains(range, living, self.creative)", "&& !self.client_attack_range_contains(range, living)")
        text = replace_once(text, "    fn active_attack_range(&self)", "    pub(super) fn active_attack_range(&self)")
        text = replace_once(text, "    fn attack_range_pick(\n", "    pub(super) fn attack_range_pick(\n")
        # The unmodified helper now remains only in the host's own unit tests.
        text = replace_once(text, "fn attack_range_contains(range:", "#[cfg(test)]\nfn attack_range_contains(range:")
        return text
    edit("crates/horizon-client/src/game/hand.rs", hand)

    # Existing action paths keep their native return shape while sharing the
    # canonical EntityHitResult selected before the original block result.
    wrappers = [
        ("paintings", "painting_hit", "(usize, f64)", 1, "(hit.index, hit.distance)"),
        ("frames", "frame_hit", "(usize, f64)", 2, "(hit.index, hit.distance)"),
        ("armor_stands", "armor_stand_hit", "(usize, f64, [f64; 3])", 3, "(hit.index, hit.distance, hit.point)"),
        ("cushions", "cushion_hit", "(usize, f64, [f64; 3])", 4, "(hit.index, hit.distance, hit.point)"),
        ("boats", "boat_hit", "(usize, f64)", 5, "(hit.index, hit.distance)"),
        ("minecarts", "cart_hit", "(usize, f64)", 6, "(hit.index, hit.distance)"),
        ("interactions", "interaction_hit", "(i32, f64)", 7, "(hit.id, hit.distance)"),
        ("deflect", "redirectable_hit", "(usize, f64)", 8, "(hit.index, hit.distance)"),
        ("remote_vehicles", "remote_knot_hit", "(i32, f64, [f64; 3])", 9, "(hit.id, hit.distance, std::array::from_fn(|i| hit.point[i]-hit.location[i]))"),
    ]
    for module, name, result, kind, value in wrappers:
        def add(text, name=name, result=result, kind=kind, value=value):
            needle = f"fn {name}(&self) -> Option<{result}> {{\n"
            replacement = needle + f"        if let Some(hit) = self.client_reach_target() {{\n            return (hit.kind == {kind} && hit.distance_squared < self.block_reach * self.block_reach).then_some({value});\n        }}\n"
            return replace_once(text, needle, replacement)
        edit(f"crates/horizon-client/src/game/{module}.rs", add)
    def dragon(text):
        # The native hook is a sibling of dragon.rs and reads the same actual
        # multipart bounds. Keep one implementation with sibling visibility.
        text = replace_once(text, "    fn part_box(&self, i: usize) -> Aabb {", "    pub(super) fn part_box(&self, i: usize) -> Aabb {")
        return replace_once(text,
            "fn dragon_hit(&self) -> Option<(u32, f64)> {\n",
            "fn dragon_hit(&self) -> Option<(u32, f64)> {\n        if let Some(hit) = self.client_reach_target() {\n            return (matches!(hit.kind, 10 | 11) && hit.distance_squared < self.block_reach * self.block_reach).then_some((hit.local_id, hit.distance));\n        }\n")
    edit("crates/horizon-client/src/game/dragon.rs", dragon)
    edit("crates/horizon-client/src/game/remote_vehicles.rs", lambda text: replace_once(text,
        "fn remote_object_hit(&self, attacking: bool) -> Option<ObjectHit> {\n",
        "fn remote_object_hit(&self, attacking: bool) -> Option<ObjectHit> {\n        if let Some(hit) = self.client_reach_target() {\n            return (hit.kind != 0 && hit.kind != 8 && hit.distance_squared < self.block_reach * self.block_reach).then_some(ObjectHit { id:hit.id, distance:hit.distance, location:hit.location });\n        }\n"))
    # The local projectile attack used a raw block comparison, whereas the
    # remote path already asks before_block. Read the shared hit result here.
    edit("crates/horizon-client/src/game/deflect.rs", lambda text: replace_once(text,
        "        if let (Some(hit), Some(eye)) = (self.pick_unfiltered(), self.eye_position()) {\n",
        "        if !self.before_block(t) { return false; }\n        if let (Some(hit), Some(eye)) = (self.pick_unfiltered(), self.eye_position()) {\n"))
    return {"hook_names": HOOK_NAMES, "helper_names": HELPER_NAMES}
