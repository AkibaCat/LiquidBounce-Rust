use super::client_smart_eat_use::{UseFacts, UseOwner, passes, owner};

const OWNERS:[UseOwner;27]=[
    UseOwner::Item,UseOwner::BoatItem,UseOwner::BottleItem,UseOwner::BowItem,
    UseOwner::BucketItem,UseOwner::BundleItem,UseOwner::CrossbowItem,UseOwner::EggItem,
    UseOwner::EmptyMapItem,UseOwner::EnderEyeItem,UseOwner::EnderpearlItem,
    UseOwner::ExperienceBottleItem,UseOwner::FireworkRocketItem,UseOwner::FishingRodItem,
    UseOwner::FoodOnAStickItem,UseOwner::InstrumentItem,UseOwner::KnowledgeBookItem,
    UseOwner::LingeringPotionItem,UseOwner::PlaceOnWaterBlockItem,UseOwner::SnowballItem,
    UseOwner::SpawnEggItem,UseOwner::SplashPotionItem,UseOwner::SpyglassItem,
    UseOwner::TridentItem,UseOwner::WindChargeItem,UseOwner::WritableBookItem,
    UseOwner::WrittenBookItem,
];

#[test]
fn all_real_use_owners_honor_spectator_and_cooldown_before_item_dispatch(){
    for owner in OWNERS {
        assert!(passes(owner,&UseFacts {spectator:true,..UseFacts::default()}));
        assert!(passes(owner,&UseFacts {cooldown:true,..UseFacts::default()}));
    }
}

#[test]
fn generic_item_preserves_effective_component_precedence(){
    let mut f=UseFacts::default();
    assert!(passes(UseOwner::Item,&f));
    f.blocks_attacks=true;
    assert!(!passes(UseOwner::Item,&f));
    f.kinetic_weapon=true;
    assert!(!passes(UseOwner::Item,&f));
    f.swappable_equippable=true;
    assert!(passes(UseOwner::Item,&f)); // denied equipment returns PASS before block/weapon
    f.equipment_allows_player=true;
    assert!(!passes(UseOwner::Item,&f)); // SUCCESS and FAIL both remain non-PASS
    f.equipment_allows_player=false;
    f.consumable=true;
    assert!(!passes(UseOwner::Item,&f)); // consumable FAIL never falls through
}

#[test]
fn special_item_pass_rules_distinguish_miss_fail_and_success(){
    let mut f=UseFacts::default();
    assert!(passes(UseOwner::BoatItem,&f));
    f.boat_ray_hit=true;
    assert!(!passes(UseOwner::BoatItem,&f));
    f.boat_eye_inside_pickable=true;
    assert!(passes(UseOwner::BoatItem,&f));
    assert!(passes(UseOwner::BottleItem,&f));
    f.source_ray_hit=true;
    assert!(passes(UseOwner::BottleItem,&f));
    f.source_is_water=true;
    assert!(!passes(UseOwner::BottleItem,&f));
    f.source_ray_hit=false;
    f.collectible_dragon_cloud=true;
    assert!(!passes(UseOwner::BottleItem,&f));
    f.empty_bucket=true;
    assert!(passes(UseOwner::BucketItem,&f));
    f.source_ray_hit=true;
    assert!(!passes(UseOwner::BucketItem,&f));
    f.empty_bucket=false;
    assert!(passes(UseOwner::BucketItem,&f));
    f.solid_ray_hit=true;
    assert!(!passes(UseOwner::BucketItem,&f));
    assert!(!passes(UseOwner::EnderEyeItem,&f));
    f.end_portal_frame=true;
    assert!(passes(UseOwner::EnderEyeItem,&f));
    assert!(passes(UseOwner::FireworkRocketItem,&f));
    f.gliding=true;
    assert!(!passes(UseOwner::FireworkRocketItem,&f));
    assert!(passes(UseOwner::FoodOnAStickItem,&f));
    assert!(!passes(UseOwner::SpawnEggItem,&f));
    f.source_ray_hit=false;
    assert!(passes(UseOwner::SpawnEggItem,&f));
}

#[test]
fn nonpassing_subclasses_do_not_fall_back_to_generic_components(){
    for owner in OWNERS {
        if matches!(owner,UseOwner::Item|UseOwner::BoatItem|UseOwner::BottleItem|UseOwner::BucketItem|UseOwner::EnderEyeItem|UseOwner::FireworkRocketItem|UseOwner::FoodOnAStickItem|UseOwner::SpawnEggItem){continue;}
        assert!(!passes(owner,&UseFacts::default()),"{owner:?}");
    }
}

#[test]
fn actual_java_oracle_covers_every_original_item_dispatch(){
    let oracle:serde_json::Value=serde_json::from_str(include_str!("oracle.json")).unwrap();
    let rows=oracle["useOwners"].as_array().unwrap();
    assert_eq!(rows.len(),1658);
    for row in rows {
        let item=row["item"].as_str().unwrap();
        let name=row["owner"].as_str().unwrap().rsplit('.').next().unwrap();
        assert_eq!(format!("{:?}",owner(item).unwrap()),name,"{item}");
    }
    assert_eq!(owner("mod:unknown_item"),None);
}
