// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete ScaffoldClutchTechnique over the shared real-world placement kernel.
use super::{
    Game, ItemStack,
    block_placement::{self as placement, Target},
    scaffold_clutch_rules as rules,
};
use serde_json::{Value, json};

fn write_target(t: &Target) -> Value {
    json!({"interacted":t.interacted,"placed":t.placed,"direction":t.direction,"point":t.point,"minY":t.min_y,"rotation":t.rotation})
}
fn read_target(v: &Value) -> Option<Target> {
    Some(Target {
        interacted: serde_json::from_value(v["interacted"].clone()).ok()?,
        placed: serde_json::from_value(v["placed"].clone()).ok()?,
        direction: serde_json::from_value(v["direction"].clone()).ok()?,
        point: serde_json::from_value(v["point"].clone()).ok()?,
        min_y: v["minY"].as_f64()?,
        rotation: serde_json::from_value(v["rotation"].clone()).ok()?,
    })
}
fn write_hit(click: placement::Click) -> Value {
    let hit = click.hit;
    json!({"block":hit.pos,"face":hit.face,"point":hit.point,"inside":click.inside})
}
fn click(
    game: &Game<'_>,
    target: &Target,
    rotation: [f32; 2],
    block_range: f64,
    verification_range: f64,
) -> Option<placement::Click> {
    if let Some(hit) = placement::verify_click(
        game,
        target,
        rotation,
        verification_range,
        placement::ClickTrace::World,
        placement::FailedClick::Nothing,
    ) {
        return Some(hit);
    }
    let eye = game.eye_position()?;
    let faces_player = (0..3)
        .map(|i| (eye[i] - target.point[i]) * f64::from(target.direction[i]))
        .sum::<f64>()
        > 0.0;
    // Source evaluates this query even when the planned face points away.
    // It clips only the interacted block, so a different face remains valid.
    let pointed_at =
        placement::raytrace_block(game, target.interacted, rotation, block_range).is_some();
    (faces_player && pointed_at).then(|| placement::planned_click(target))
}
struct Adapter<'g, 'r, 's> {
    game: &'g mut Game<'r>,
    stack: &'s ItemStack,
    rotation: [f32; 2],
    block_range: f64,
    verification_range: f64,
}
impl rules::Host for Adapter<'_, '_, '_> {
    type Target = Target;
    fn can_stand_on(&mut self, position: [i32; 3]) -> bool {
        placement::holds_solid(self.game, position)
    }
    fn find_target(&mut self, search: rules::Search<'_>) -> Option<Target> {
        let location = placement::capture_location(self.game, Some(search.from), self.rotation)?;
        let mut options = placement::Options::center(self.stack, location);
        options.offsets = search.offsets.to_vec();
        options.priority = placement::Priority::Position(search.priority);
        options.range = self.block_range;
        placement::find(self.game, search.below, &mut options)
    }
    fn can_click(&mut self, target: &Target) -> bool {
        click(
            self.game,
            target,
            self.rotation,
            self.block_range,
            self.verification_range,
        )
        .is_some()
    }
}
impl Game<'_> {
    pub(super) fn scaffold_clutch_query(&mut self, text: &str) -> Option<String> {
        let request = serde_json::from_str::<Value>(text).ok()?;
        let ahead = request["aimsAhead"].as_bool()?;
        let server_rotation: [f32; 2] =
            serde_json::from_value(request["serverRotation"].clone()).ok()?;
        let block_range = self.interaction_range("minecraft:block_interaction_range", 4.5, 5.0);
        let entity_range = self.interaction_range("minecraft:entity_interaction_range", 3.0, 5.0);
        let verification_range = block_range.max(entity_range);
        match request["op"].as_str()? {
            "plan" => {
                let fall: Vec<[f64; 3]> = serde_json::from_value(request["path"].clone()).ok()?;
                if fall.is_empty() {
                    return None;
                }
                let player = self.player.as_ref()?;
                let current = [player.position.x, player.position.y, player.position.z];
                let height = player.dimensions.eye_height;
                // ModuleScaffold plans with sandstone when its best hotbar
                // slot is absent, even if a real offhand block makes it run.
                let stack = if request["slot"].is_null() {
                    ItemStack::new("minecraft:sandstone", 64)
                } else {
                    let slot = usize::try_from(request["slot"].as_u64()?).ok()?;
                    if slot > 8 {
                        return None;
                    }
                    self.inventory.get(slot)?.as_ref()?.clone()
                };
                let offsets = placement::offsets(4);
                let mut host = Adapter {
                    game: self,
                    stack: &stack,
                    rotation: server_rotation,
                    block_range,
                    verification_range,
                };
                let target = rules::find_placement_target(
                    &mut host,
                    current,
                    &fall,
                    height,
                    block_range,
                    &offsets,
                    ahead,
                )?;
                Some(write_target(&target).to_string())
            }
            "crosshair" => {
                let target = read_target(&request["target"])?;
                let hit = if ahead {
                    click(
                        self,
                        &target,
                        server_rotation,
                        block_range,
                        verification_range,
                    )
                } else {
                    let rotation = serde_json::from_value(request["rotation"].clone()).ok()?;
                    placement::verify_click(
                        self,
                        &target,
                        rotation,
                        verification_range,
                        placement::ClickTrace::World,
                        placement::FailedClick::Nothing,
                    )
                }?;
                Some(write_hit(hit).to_string())
            }
            _ => None,
        }
    }
}
