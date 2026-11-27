#[path="__NO_ENTITY_PATH__"] mod no_entity_interact;
struct NoEntityState {running:bool,module:no_entity_interact::NoEntityInteract}
static NO_ENTITY:LazyLock<Mutex<NoEntityState>>=LazyLock::new(||Mutex::new(NoEntityState {running:false,module:Default::default()}));
static SILENT_SLOT:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(usize::MAX);
struct Options {resources:std::path::PathBuf}
fn item_catalog()->&'static horizon_items::items::Items {
    static ITEMS:std::sync::OnceLock<horizon_items::items::Items>=std::sync::OnceLock::new();
    ITEMS.get_or_init(||horizon_items::items::Items::catalog(std::path::Path::new("__RESOURCE_ROOT__"),&horizon_world::loot::PackOverlay::NONE).unwrap())
}
impl Game {
    fn items(&self)->&horizon_items::items::Items {item_catalog()}
    fn stack_in(&self,slot:usize)->Option<&Stack> {self.inventory.get(slot)?.as_ref()}
    fn client_hotbar_slot(&self)->usize {let slot=SILENT_SLOT.load(std::sync::atomic::Ordering::Acquire);if slot==usize::MAX {self.selected} else {slot}}
    fn native_registry_name(&self,_:&str,_:i32)->Option<String> {None}
}
#[cfg(test)] mod no_entity_tests {
    use super::*;
    fn reset() {
        *NO_ENTITY.lock().unwrap()=NoEntityState {running:true,module:Default::default()};
        *MODULE.lock().unwrap()=ModuleState {enabled:false,settings:Default::default(),rotation:None};
        SILENT_SLOT.store(usize::MAX,std::sync::atomic::Ordering::Release);
    }
    fn all_hits(game:&Game)->[bool;12] {[
        game.mob_hit().is_some(),game.painting_hit().is_some(),game.frame_hit().is_some(),game.armor_stand_hit().is_some(),
        game.cushion_hit().is_some(),game.boat_hit().is_some(),game.cart_hit().is_some(),game.interaction_hit().is_some(),
        game.redirectable_hit().is_some(),game.remote_knot_hit().is_some(),game.dragon_hit().is_some(),game.remote_object_hit(true).is_some(),
    ]}
    #[test] fn actual_twelve_query_paths_drop_only_plain_pick_and_keep_block_result() {
        let _scene=SCENE.lock().unwrap();reset();
        for kind in 0..12 {
            let mut game=Game::new();game.wall=None;game.add(kind,40,2.0);
            NO_ENTITY.lock().unwrap().running=false;
            assert!(all_hits(&game).into_iter().any(|hit|hit),"category {kind}");
            NO_ENTITY.lock().unwrap().running=true;
            game.inventory[0]=Some(Stack {item:"minecraft:diamond_sword".into(),range:None});
            assert!(!all_hits(&game).into_iter().any(|hit|hit),"category {kind}");
            game.wall=Some(bounds(2.5));assert!(game.pick().is_some());
        }
    }
    #[test] fn source_pig_fallback_and_holding_filter_use_silent_main_hand_only() {
        let _scene=SCENE.lock().unwrap();reset();let mut game=Game::new();game.wall=None;game.add(0,50,2.0);
        assert!(game.mob_hit().is_some()); // empty main hand is AIR, in defaults
        game.inventory[0]=Some(Stack {item:"minecraft:diamond_sword".into(),range:None});assert!(game.mob_hit().is_none());
        game.inventory.push(Some(Stack {item:"minecraft:copper_pickaxe".into(),range:None}));
        SILENT_SLOT.store(1,std::sync::atomic::Ordering::Release);assert!(game.mob_hit().is_some());
        NO_ENTITY.lock().unwrap().module.entity_type_filter=no_entity_interact::Filter::Whitelist;assert!(game.mob_hit().is_none());
        NO_ENTITY.lock().unwrap().module.entity_types.insert("minecraft:pig".into());assert!(game.mob_hit().is_some());
        SILENT_SLOT.store(usize::MAX,std::sync::atomic::Ordering::Release);
    }
    #[test] fn active_component_and_reach_through_walls_bypass_the_projectile_result_filter() {
        let _scene=SCENE.lock().unwrap();reset();let mut game=Game::new();game.wall=None;game.add(0,50,2.0);
        let range=serde_json::json!({"min_reach":0.0,"max_reach":4.5,"hitbox_margin":0.3});
        game.inventory[0]=Some(Stack {item:"minecraft:diamond_sword".into(),range:Some(range)});
        assert!(game.mob_hit().is_some());
        game.inventory[0].as_mut().unwrap().range=None;
        MODULE.lock().unwrap().enabled=true;MODULE.lock().unwrap().settings.through_walls_range=4.0;
        assert!(game.mob_hit().is_some());
    }
    #[test] fn native_identity_guards_do_not_dispatch_module_policy() {
        let _scene=SCENE.lock().unwrap();
        for mode in 0..5 {reset();let mut game=Game::new();game.wall=None;game.add(0,50,2.0);
            game.inventory[0]=Some(Stack {item:"minecraft:diamond_sword".into(),range:None});
            match mode {0=>game.window=false,1=>game.lan_acting=Some(3),2=>game.lan_acting_uuid=Some(9),3=>game.player.as_mut().unwrap().server_copy=true,_=>game.player.as_mut().unwrap().replaying=true}
            assert!(game.mob_hit().is_some());
        }
    }
    #[test] fn default_holding_items_match_real_minecraft_template_tags_and_registry_comparator_oracle() {
        let _scene=SCENE.lock().unwrap();reset();let game=Game::new();
        let facts:serde_json::Value=serde_json::from_str(include_str!("../no-interact-facts.json")).unwrap();
        let expected:Vec<String>=serde_json::from_value(facts["defaultHoldingItems"].clone()).unwrap();
        assert_eq!(game.native_client_default_no_entity_holding_items(),expected);
        assert_eq!(NO_ENTITY.lock().unwrap().module.holding_items.iter().cloned().collect::<Vec<_>>(),expected);
        assert_eq!(facts["entityFilter"]["entityRegistryFallback"],"minecraft:pig");
        assert_eq!(facts["entityFilter"]["defaultEntitySetContainsInstance"],false);
        assert_eq!(facts["entityFilter"]["withPigContainsInstance"],true);
        let (entities,items)=game.native_client_no_interact_registries();
        let expected=std::fs::read_to_string(game.options.resources.join("entity-types.json")).unwrap();
        let expected:serde_json::Value=serde_json::from_str(&expected).unwrap();
        assert_eq!(entities,expected.as_object().unwrap().keys().cloned().collect::<Vec<_>>());
        assert_eq!(entities.len(),161);assert_eq!(items.len(),1658);
        assert!(entities.contains(&"minecraft:player".into()));assert!(items.contains(&"minecraft:air".into()));
    }
}
