// SPDX-License-Identifier: GPL-3.0-or-later
//! The PASS predicate of MC26.3 MultiPlayerGameMode.useItem, evaluated from
//! pre-use facts. SUCCESS and FAIL are both non-PASS; a consumed bool is never
//! an input. Native Item.use effects and the RETURN event remain separate.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UseOwner {
    Item,
    BoatItem,
    BottleItem,
    BowItem,
    BucketItem,
    BundleItem,
    CrossbowItem,
    EggItem,
    EmptyMapItem,
    EnderEyeItem,
    EnderpearlItem,
    ExperienceBottleItem,
    FireworkRocketItem,
    FishingRodItem,
    FoodOnAStickItem,
    InstrumentItem,
    KnowledgeBookItem,
    LingeringPotionItem,
    PlaceOnWaterBlockItem,
    SnowballItem,
    SpawnEggItem,
    SplashPotionItem,
    SpyglassItem,
    TridentItem,
    WindChargeItem,
    WritableBookItem,
    WrittenBookItem,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UseFacts {
    pub spectator: bool,
    pub cooldown: bool,
    pub consumable: bool,
    pub swappable_equippable: bool,
    pub equipment_allows_player: bool,
    pub blocks_attacks: bool,
    pub kinetic_weapon: bool,
    pub empty_bucket: bool,
    pub source_ray_hit: bool,
    pub solid_ray_hit: bool,
    pub boat_ray_hit: bool,
    pub boat_eye_inside_pickable: bool,
    pub collectible_dragon_cloud: bool,
    pub source_is_water: bool,
    pub end_portal_frame: bool,
    pub gliding: bool,
}

pub fn passes(owner: UseOwner, facts: &UseFacts) -> bool {
    if facts.spectator || facts.cooldown {
        return true;
    }
    match owner {
        UseOwner::Item => {
            if facts.consumable {
                false
            } else if facts.swappable_equippable {
                !facts.equipment_allows_player
            } else {
                !facts.blocks_attacks && !facts.kinetic_weapon
            }
        }
        UseOwner::BoatItem => !facts.boat_ray_hit || facts.boat_eye_inside_pickable,
        UseOwner::BottleItem => {
            !facts.collectible_dragon_cloud
                && (!facts.source_ray_hit || !facts.source_is_water)
        }
        UseOwner::BucketItem => {
            if facts.empty_bucket {
                !facts.source_ray_hit
            } else {
                !facts.solid_ray_hit
            }
        }
        UseOwner::EnderEyeItem => facts.end_portal_frame,
        UseOwner::FireworkRocketItem => !facts.gliding,
        // ClientLevel returns PASS before the server's mount/boost branch.
        UseOwner::FoodOnAStickItem => true,
        // The client's early SUCCESS is before the server's LiquidBlock test.
        UseOwner::SpawnEggItem => !facts.source_ray_hit,
        // BlockItem.place only returns SUCCESS/FAIL. If it fails, useOn's
        // transformer/consumable fallback still cannot return PASS.
        UseOwner::PlaceOnWaterBlockItem
        | UseOwner::BowItem
        | UseOwner::BundleItem
        | UseOwner::CrossbowItem
        | UseOwner::EggItem
        | UseOwner::EmptyMapItem
        | UseOwner::EnderpearlItem
        | UseOwner::ExperienceBottleItem
        | UseOwner::FishingRodItem
        | UseOwner::InstrumentItem
        | UseOwner::KnowledgeBookItem
        | UseOwner::LingeringPotionItem
        | UseOwner::SnowballItem
        | UseOwner::SplashPotionItem
        | UseOwner::SpyglassItem
        | UseOwner::TridentItem
        | UseOwner::WindChargeItem
        | UseOwner::WritableBookItem
        | UseOwner::WrittenBookItem => false,
    }
}
