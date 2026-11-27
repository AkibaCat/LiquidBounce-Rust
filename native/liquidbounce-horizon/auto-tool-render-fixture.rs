mod render_probe {
use super::*;
#[derive(Default)]struct Sway;impl Sway{fn tick_wrapped(&mut self,_:f32,_:f32){}}
#[derive(Default)]struct Swing;impl Swing{fn tick(&mut self){}}
#[derive(Default)]struct Heights{scale:f32}impl Heights{fn tick_with(&mut self,_:bool,scale:f32,_:[bool;2],_:bool)->[bool;2]{self.scale=scale;[true;2]}}
__ATTACK_STRENGTH__
const OFFHAND:usize=usize::MAX;
__SHOWS_HELD__
__MATCHES_SWAP__
struct RenderGame {
    game:Game,sway:Sway,swing:Swing,pitch:f32,yaw:f32,hand_strength:AttackStrength,heights:Heights,
    visible_hands:[(usize,Option<ItemStack>);2],hands_replaced:[bool;2],attack_ticks:i32,attack_item:Option<std::sync::Arc<str>>,equipment:Option<std::sync::Arc<str>>,
}
impl std::ops::Deref for RenderGame{type Target=Game;fn deref(&self)->&Game{&self.game}}
impl std::ops::DerefMut for RenderGame{fn deref_mut(&mut self)->&mut Game{&mut self.game}}
impl RenderGame{
    fn apply_equipment(&mut self){self.equipment=self.game.held().map(|s|s.item.clone());}
    fn main_hand_attack_speed(&self)->f64{4.}
    fn hands_busy(&self)->bool{false}fn pvp_legacy_look(&self)->bool{false}
    __TICK_HAND__
}
#[cfg(test)]mod tests{
use super::*;
#[test]fn actual_tick_hand_keeps_server_equipment_and_cooldown_but_renders_clientside_stack(){let mut game=Game::new();game.silent=Some(5);game.inventory[5]=Some(ItemStack::new("minecraft:diamond_sword",1));game.inventory[2]=Some(ItemStack::new("minecraft:stone",1));let mut render=RenderGame{game,sway:Sway,swing:Swing,pitch:0.,yaw:0.,hand_strength:Default::default(),heights:Default::default(),visible_hands:[(0,None),(OFFHAND,None)],hands_replaced:[false;2],attack_ticks:10,attack_item:None,equipment:None};render.tick_hand();assert_eq!(render.attack_item.as_deref(),Some("minecraft:diamond_sword"));assert_eq!(render.equipment.as_deref(),Some("minecraft:diamond_sword"));assert_eq!(render.attack_ticks,0);assert_eq!(render.visible_hands[0].0,2);assert_eq!(render.visible_hands[0].1.as_ref().map(|s|&*s.item),Some("minecraft:stone"));assert_eq!(render.heights.scale,1.);}
}
}
