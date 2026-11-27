#![cfg(test)]
extern crate self as horizon_client;
pub use horizon_items::items;
pub mod mod_api {
    pub use horizon_protocol::packets::play::{Clientbound as ClientboundPlayPacket, Serverbound as ServerboundPlayPacket};
}
#[path="SLOT_BRIDGE_PATH"]
pub mod slot_bridge;
#[path="SLOTS_PATH"]
pub mod slots;

#[allow(dead_code)]
mod fixture {
use super::*;
use horizon_items::items::ItemStack;
use horizon_world::player_inventory::EquipmentSlot;
use liquidbounce_behaviors::silent_hotbar::Requester;
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Copy)]
enum Hand { Main, Off }
mod shield { pub const OFFHAND_SLOT: usize = 40; }
#[derive(Clone, Copy, PartialEq, Eq)]
enum Species { Pig, Strider }
struct Mob { kind: Species, extra: Extra }
#[derive(Default)]
struct Extra { saddled: bool, saddle: Option<ItemStack> }
impl Mob {
    fn species(&self) -> Species { self.kind }
    fn set_saddle(&mut self, stack: Option<ItemStack>) { self.extra.saddled = true; self.extra.saddle = stack; }
}
mod equine { pub fn boost(_: &mut super::Mob) -> bool { true } }
struct ItemInfo { max_damage: u32 }
struct Resources { items: HashMap<Arc<str>, ItemInfo> }
impl Resources { fn items(&self) -> &HashMap<Arc<str>, ItemInfo> { &self.items } }
#[derive(Default)]
struct Player { server_copy: bool, replaying: bool }
impl Player { fn native_client_queries_allowed(&self) -> bool { !self.server_copy && !self.replaying } }
struct Shelf { items: Vec<Option<ItemStack>> }
enum BlockEntity { Shelf(Shelf), Other }
#[derive(Default)]
struct World { block_entities: HashMap<(i32,i32,i32), BlockEntity> }
enum Slot { MainHand, Inv(usize) }
struct Game {
    selected: usize, inventory: Vec<Option<ItemStack>>, armor: [Option<ItemStack>;4], offhand: Option<ItemStack>,
    player: Option<Player>, lan_acting_uuid: Option<u128>, window: bool,
    creative: bool, hand: Hand, res: Resources, mobs: Vec<Mob>, world: World,
    dropped: Vec<ItemStack>, replaced: Vec<usize>, equips: Vec<(String,Option<ItemStack>,Option<ItemStack>)>, broken: Vec<ItemStack>, swings: usize,
}
impl Game {
    fn new() -> Self {
        let mut inventory = vec![None;36];
        inventory[2] = Some(ItemStack::new("minecraft:stone", 8));
        inventory[7] = Some(ItemStack::new("minecraft:diamond", 3));
        Self {selected:2, inventory, armor:Default::default(), offhand:Some(ItemStack::new("minecraft:shield",1)), player:Some(Player::default()), lan_acting_uuid:None,window:true,creative:false,hand:Hand::Main,res:Resources{items:HashMap::new()},mobs:vec![Mob{kind:Species::Pig,extra:Extra::default()}],world:World::default(),dropped:vec![],replaced:vec![],equips:vec![],broken:vec![],swings:0}
    }
    fn native_window(&self) -> bool { self.window }
    fn stack_in(&self, slot:usize) -> Option<&ItemStack> { if slot==shield::OFFHAND_SLOT {self.offhand.as_ref()} else {self.inventory[slot].as_ref()} }
    fn stack_in_mut(&mut self, slot:usize) -> &mut Option<ItemStack> { if slot==shield::OFFHAND_SLOT {&mut self.offhand} else {&mut self.inventory[slot]} }
    fn eye_position(&self) -> Option<[f64;3]> { self.player.as_ref().map(|_| [0.0;3]) }
    fn spectator(&self) -> bool { false }
    fn player_drop(&mut self, stack:ItemStack) { self.dropped.push(stack); }
    fn item_tagged(&self, tag:&str, item:&str) -> bool {tag=="minecraft:nautilus_bucket_food" && item.ends_with("_bucket")}
    fn give(&mut self, stack:ItemStack) -> Option<ItemStack> { Some(stack) }
    fn spawn_item(&mut self, stack:ItemStack, _: [f64;3], _: [f64;3], _:i32) {self.dropped.push(stack);}
    fn has_death_protection(&self, stack:&ItemStack) -> bool { &*stack.item=="minecraft:totem_of_undying" }
    fn player_on_equip(&mut self, slot:&str, old:Option<&ItemStack>, new:Option<&ItemStack>) {self.equips.push((slot.into(),old.cloned(),new.cloned()));}
    fn hand_stack_replaced(&mut self, slot:usize) {self.replaced.push(slot);}
    fn mount_index(&self) -> Option<usize> {Some(0)}
    fn wear(&self, _: &ItemStack, wear:u32) -> u32 {wear}
    fn player_equipment_broke(&mut self, _:EquipmentSlot, stack:&ItemStack) {self.broken.push(stack.clone());}
    fn swing_arm(&mut self) {self.swings+=1;}
    /* HELPERS */
    /* HELD */
    /* MAIN_ACCESS */
    /* MUTATIONS */
    fn use_totem_cell(&mut self) -> ItemStack {
        const TOTEM:&str="minecraft:totem_of_undying";
        /* TOTEM */
        protector
    }
    fn swap_hands(&mut self) {
        /* SWAP */
    }
    fn shelf_swap(&mut self, pos:[i32;3], slot:usize) -> Option<()> {
        /* SHELF */
        Some(())
    }
    fn saddle(&mut self, index:usize, item:&str, tame:bool) {
        /* SADDLE */
    }
    fn equipment(&self, slot:EquipmentSlot) -> Option<ItemStack> {
        /* EQUIPMENT_CALL */
    }
    fn enchant_main(&mut self, e:String, level:u32) -> Result<(), &'static str> {
        /* ENCHANT */
        Ok(())
    }
}
/* EQUIPMENT */
/* TAKE_FUNCTIONS */
/* STICK_BREAK */
const OFFHAND:usize=36;

#[test]
fn real_transformed_mainhand_bodies_follow_effective_slot_and_preserve_raw_selection() {
    let service=Arc::new(slots::SlotService::default());
    let previous=slot_bridge::set_policy(Some(service.clone()));
    let owner=Requester::allocate();
    service.select_slot_silently(owner,7,i32::MAX,||Some(2)).unwrap();
    let mut game=Game::new();
    let raw=game.inventory[2].clone();
    assert_eq!(game.native_client_real_selected_slot(),Some(2));
    assert_eq!(game.held().unwrap().item.as_ref(),"minecraft:diamond");
    assert_eq!(game.native_held(),Some(("minecraft:diamond".into(),3,true)));
    assert_eq!(game.get_main(Slot::MainHand),game.inventory[7]);
    assert_eq!(game.get_main(Slot::Inv(2)),raw);
    assert_eq!(game.equipment(EquipmentSlot::MainHand),game.inventory[7]);
    assert_eq!(game.equipment(EquipmentSlot::OffHand),game.offhand);
    game.drop_held(false);
    assert_eq!(game.dropped[0].item.as_ref(),"minecraft:diamond");
    assert_eq!(game.dropped[0].count,1);
    assert_eq!(game.inventory[7].as_ref().unwrap().count,2);
    assert_eq!(game.inventory[2],raw);
    game.swap_hands();
    assert_eq!(game.offhand.as_ref().unwrap().item.as_ref(),"minecraft:diamond");
    assert_eq!(game.inventory[7].as_ref().unwrap().item.as_ref(),"minecraft:shield");
    assert_eq!(game.replaced,vec![7,shield::OFFHAND_SLOT]);
    assert_eq!(game.equips.len(),2);
    assert_eq!(game.selected,2);
    assert!(game.set_main(Slot::MainHand,Some(ItemStack::new("minecraft:stick",4))));
    assert_eq!(game.inventory[7].as_ref().unwrap().count,4);
    assert_eq!(game.equips.last().unwrap().0,"mainhand");
    game.enchant_main("minecraft:unbreaking".into(),2).unwrap();
    assert!(!game.inventory[7].as_ref().unwrap().crafting_enchantments().is_empty());
    game.inventory[7]=Some(ItemStack::new("minecraft:stone",3));
    assert_eq!(game.native_take_items("minecraft:stone",2),2);
    assert_eq!(game.inventory[7].as_ref().unwrap().count,1);
    assert_eq!(game.inventory[2],raw);
    game.inventory[7]=Some(ItemStack::new("minecraft:totem_of_undying",1));
    assert_eq!(game.use_totem_cell().item.as_ref(),"minecraft:totem_of_undying");
    assert!(game.inventory[7].is_none());
    assert_eq!(game.inventory[2],raw);
    game.inventory[7]=Some(ItemStack::new("minecraft:stone",1));
    game.offhand=Some(ItemStack::new("minecraft:totem_of_undying",1));
    assert_eq!(game.use_totem_cell().item.as_ref(),"minecraft:totem_of_undying");
    assert!(game.offhand.is_none());
    assert_eq!(game.inventory[7].as_ref().unwrap().item.as_ref(),"minecraft:stone");
    game.inventory[7]=Some(ItemStack::new("minecraft:cod_bucket",1));
    game.use_nautilus_food("minecraft:cod_bucket");
    assert_eq!(game.inventory[7].as_ref().unwrap().item.as_ref(),"minecraft:water_bucket");
    game.inventory[7]=Some(ItemStack::new("minecraft:cod_bucket",2));
    game.use_nautilus_food("minecraft:cod_bucket");
    assert_eq!(game.inventory[7].as_ref().unwrap().count,1);
    assert_eq!(game.dropped.last().unwrap().item.as_ref(),"minecraft:water_bucket");
    game.inventory[7]=Some(ItemStack::new("minecraft:saddle",1));
    game.saddle(0,"minecraft:saddle",true);
    assert_eq!(game.mobs[0].extra.saddle,game.inventory[7]);
    game.world.block_entities.insert((0,0,0),BlockEntity::Shelf(Shelf{items:vec![Some(ItemStack::new("minecraft:apple",1))]}));
    game.shelf_swap([0,0,0],0).unwrap();
    assert_eq!(game.inventory[7].as_ref().unwrap().item.as_ref(),"minecraft:apple");
    let BlockEntity::Shelf(shelf)=&game.world.block_entities[&(0,0,0)] else {panic!()};
    assert_eq!(shelf.items[0].as_ref().unwrap().item.as_ref(),"minecraft:saddle");
    game.inventory[7]=Some(ItemStack::new("minecraft:carrot_on_a_stick",1));
    game.inventory[7].as_mut().unwrap().damage=24;
    assert!(game.use_steering_stick("minecraft:carrot_on_a_stick"));
    assert_eq!(game.inventory[7].as_ref().unwrap().item.as_ref(),"minecraft:fishing_rod");
    assert_eq!(game.broken.len(),1);
    assert_eq!(game.inventory[2],raw);
    assert_eq!(game.selected,2);
    // Swapped guests, server copies and speculative bodies keep their own slot.
    for scenario in 0..4 {
        game.window=scenario!=0;
        game.lan_acting_uuid=(scenario==1).then_some(9);
        let player=game.player.as_mut().unwrap();
        player.server_copy=scenario==2;
        player.replaying=scenario==3;
        assert_eq!(game.native_client_real_selected_slot(),None);
        assert_eq!(game.client_hotbar_slot(),2);
        assert_eq!(game.held(),game.inventory[2].as_ref());
        assert_eq!(game.get_main(Slot::MainHand),game.inventory[2]);
        assert_eq!(game.equipment(EquipmentSlot::MainHand),game.inventory[2]);
    }
    slot_bridge::set_policy(previous);
}
}
