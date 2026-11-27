// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleVehicleOneHit's ordinary attack packets and local-only swings.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Attack(i32),
    Swing,
}

/// Concrete Boat/Minecart subclasses from MC 26.3's EntityTypes. In particular,
/// Raft, ChestBoat, ChestRaft and container/command/furnace minecarts are not eligible.
pub fn eligible(kind: &str) -> bool {
    matches!(
        kind,
        "minecraft:acacia_boat"
            | "minecraft:birch_boat"
            | "minecraft:cherry_boat"
            | "minecraft:dark_oak_boat"
            | "minecraft:jungle_boat"
            | "minecraft:mangrove_boat"
            | "minecraft:oak_boat"
            | "minecraft:pale_oak_boat"
            | "minecraft:poplar_boat"
            | "minecraft:spruce_boat"
            | "minecraft:minecart"
    )
}

/// Do not stop when an earlier AttackEntityEvent listener cancelled the event.
/// Packet cancellation also does not cancel the following local swing.
pub fn actions(running: bool, kind: &str, target: i32, legacy: bool) -> Vec<Action> {
    if !running || !eligible(kind) {
        return Vec::new();
    }
    let pair = if legacy {
        [Action::Swing, Action::Attack(target)]
    } else {
        [Action::Attack(target), Action::Swing]
    };
    (0..if legacy { 3 } else { 19 })
        .flat_map(|_| pair)
        .collect()
}
