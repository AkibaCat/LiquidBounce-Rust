#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/auto_mob_heal.rs"]
mod model;
use model::*;
fn entity(id: i32, kind: Kind, missing: f32, distance: f64) -> Entity {
    Entity { id, kind, alive: true, visible: true, health: 20.-missing, max_health: 20., distance_squared: distance, baby: false, tame: true, owned: true, sitting: true, wearing_body_armor: false, armor_empty: true, armor_max_damage: 0, armor_damage: 0 }
}
fn item(index: u8, name: &str) -> Slot {
    Slot { index, item: format!("minecraft:{name}"), nutrition: None, wolf_food: false, cat_food: false, repairs_armor: false }
}
fn main() {
    let mut settings = Settings::default();
    assert_eq!(Kind::ALL.map(Kind::tag), ["IronGolem","Wolf","Cat","HorseFamily","ZombieHorse","Llama","Camel","CamelHusk","Nautilus"]);
    let horse = entity(1, Kind::HorseFamily, 10., 1.);
    let slots = vec![item(40,"apple"), item(5,"hay_block"), item(3,"wheat")];
    assert_eq!(slot(&settings, &horse, &slots, 3), Some(40));
    let almost_full = entity(2, Kind::HorseFamily, 0.5, 1.);
    assert_eq!(slot(&settings, &almost_full, &slots, 3), None);
    settings.targets[Kind::HorseFamily as usize].no_waste = false;
    assert_eq!(slot(&settings, &almost_full, &slots, 3), Some(3));
    let mut wolf = entity(3, Kind::Wolf, 0., 1.);
    settings.wolf_armor = true; settings.wolf_equip = true;
    let mut food = item(1,"beef"); food.wolf_food = true; food.nutrition = Some(3);
    let mut scute = item(4,"armadillo_scute"); scute.repairs_armor = true;
    let slots = vec![item(40,"wolf_armor"), food, scute];
    assert_eq!(slot(&settings, &wolf, &slots, 1), Some(40));
    wolf.wearing_body_armor = true; wolf.armor_empty = false; wolf.armor_max_damage = 64; wolf.armor_damage = 8;
    assert_eq!(slot(&settings, &wolf, &slots, 1), Some(4));
    wolf.health = 10.; assert_eq!(slot(&settings, &wolf, &slots, 1), Some(1));
    wolf.health = 20.; wolf.sitting = false; assert_eq!(slot(&settings, &wolf, &slots, 1), None);
    settings.only_when_sitting = false; assert_eq!(slot(&settings, &wolf, &slots, 1), Some(4));
    wolf.owned = false; assert_eq!(slot(&settings, &wolf, &slots, 1), None);
    let fish = entity(4, Kind::Nautilus, 10., 1.);
    let slots = vec![item(0,"cod_bucket"), item(8,"cod")];
    settings.allow_bucket_food = true;
    assert_eq!(slot(&settings, &fish, &slots, 0), Some(8));
    let golem = entity(5, Kind::IronGolem, 10., 4.);
    let cat = entity(6, Kind::Cat, 10., 4.);
    let mut fish_food = item(1,"cod"); fish_food.cat_food = true; fish_food.nutrition = Some(2);
    let slots = vec![item(0,"iron_ingot"), fish_food];
    let targets = [cat.clone(), golem.clone()];
    assert_eq!(plan(&settings,&targets,0,6.,false,|_|slots.clone()).unwrap().id,5);
    assert!(plan(&settings,&targets,0,1.,false,|_|slots.clone()).is_none());
    let mut hidden = golem.clone(); hidden.visible = false;
    assert!(plan(&settings,&[hidden],0,6.,false,|_|slots.clone()).is_none());
    assert!(plan(&settings,&[horse],0,6.,true,|_|vec![item(0,"wheat")]).is_none());
    let mut unowned = cat; unowned.owned = false;
    assert!(plan(&settings,&[unowned],0,6.,false,|_|slots.clone()).is_none());
    println!("AutoMobHeal: stable targets, food waste, wolf armor priority/ownership, non-bucket preference, visibility, secondary use and native range passed");
}
