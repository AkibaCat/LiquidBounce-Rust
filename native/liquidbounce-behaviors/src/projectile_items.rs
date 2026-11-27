// SPDX-License-Identifier: GPL-3.0-or-later
// ProjectileItem membership read by oracles/FastPlaceFacts.java from Minecraft 26.3.
// Includes registered ArrowItem classes; thrown ender pearls do not implement it.
pub fn is_projectile_item(name: &str) -> bool {
    matches!(
        name,
        "minecraft:arrow"
            | "minecraft:snowball"
            | "minecraft:egg"
            | "minecraft:blue_egg"
            | "minecraft:brown_egg"
            | "minecraft:experience_bottle"
            | "minecraft:fire_charge"
            | "minecraft:wind_charge"
            | "minecraft:firework_rocket"
            | "minecraft:splash_potion"
            | "minecraft:spectral_arrow"
            | "minecraft:tipped_arrow"
            | "minecraft:lingering_potion"
            | "minecraft:trident"
    )
}
