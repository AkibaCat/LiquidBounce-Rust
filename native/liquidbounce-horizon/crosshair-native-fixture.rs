pub mod items{
    #[derive(Clone)]pub struct ItemStack{pub item:String,pub quick:i32,pub group:Option<String>}
    impl ItemStack{pub fn enchantment_level(&self,_:&str)->i32{self.quick}}
}
pub mod cooldowns{
    pub struct UseCooldown;
    pub fn group(s:&crate::items::ItemStack,_:Option<&UseCooldown>)->String{s.group.clone().unwrap_or(s.item.clone())}
}
pub mod hud{
    pub const fn rgba(r:u8,g:u8,b:u8,a:u8)->u32{(a as u32)<<24|(b as u32)<<16|(g as u32)<<8|r as u32}
    #[derive(Default)]pub struct HudAtlas;
    pub struct HudBatch{pub width:f32,pub height:f32,pub scale:f32,pub quads:Vec<([f32;4],u32)>}
    impl HudBatch{pub fn new()->Self{Self{width:320.,height:180.,scale:3.,quads:vec![]}}pub fn fill(&mut self,_:&HudAtlas,x:f32,y:f32,w:f32,h:f32,c:u32){self.quads.push(([x,y,w,h],c));}}
}
mod native{
    use crate::{runtime::LiquidBounceHooks,items::ItemStack,mod_api::{ClientCrosshairEvent,NullHost}};
    use std::{cell::RefCell,rc::Rc};
    use serde_json::Value;
    use crate::hud::{HudBatch,rgba};
    #[derive(Clone,Copy,PartialEq)]pub enum Screen{Playing,Inventory,Creative}
    pub struct Camera(pub bool);impl Camera{fn first_person(&self)->bool{self.0}}
    pub struct Clock(pub f64);impl Clock{fn fraction(&self)->f64{self.0}}
    pub struct Join(pub u64);impl Join{fn get(&self)->u64{self.0}}
    pub struct Mob{pub id:u32,pub removed:bool,pub join:Join,pub hurt_time:i32}
    pub struct Stand{pub id:u32,pub join:Join}
    pub struct Dragon{pub id:u32,pub hurt_time:i32}
    pub struct Hurt{pub hurt_time:i32}pub struct Health{pub hurt:Hurt}pub struct Player{pub health:Health}
    pub struct Guest{pub placed:bool,pub entity_id:i32,pub conn:u32,pub motion:Option<Player>}
    pub struct Hub{pub guests:Vec<Guest>}pub struct Lan{pub hub:Hub}
    pub struct Info{pub use_cooldown:Option<crate::cooldowns::UseCooldown>}
    pub struct Items;impl Items{fn get(&self,_:&str)->Option<&Info>{None}}
    pub struct Res;impl Res{fn items(&self)->&Items{&Items}}
    pub struct Cooldowns;impl Cooldowns{fn percent(&self,group:&str,partial:f32)->f32{if group=="shared"{(10.-partial)/20.}else{0.}}}
    pub struct Gui{pub atlas:crate::hud::HudAtlas}
    pub struct Mount(pub Option<u32>);impl Mount{fn opened_mob(&self)->Option<u32>{self.0}}
    pub struct Game<'a>{
        pub player:Option<Player>,pub screen:Screen,pub camera_type:Camera,pub clock:Clock,pub hand_strength:crate::attack_strength::AttackStrength,
        pub using:Option<(usize,u32)>,pub inventory:Vec<Option<ItemStack>>,pub offhand:Option<ItemStack>,pub res:Res,pub cooldowns:Cooldowns,
        pub gui:Gui,pub mount_menu:Mount,pub menu:Option<String>,pub mobs:Vec<Mob>,pub armor_stands:Vec<Stand>,pub dragon:Option<Dragon>,pub lan:Vec<Lan>,
        pub target:Option<i32>,pub hooks:Rc<LiquidBounceHooks>,pub inputs:RefCell<Vec<Value>>,pub marker:std::marker::PhantomData<&'a ()>
    }
    impl Default for Game<'_>{fn default()->Self{let mut hand=crate::attack_strength::AttackStrength::default();hand.ticker=5;Self{player:Some(Player{health:Health{hurt:Hurt{hurt_time:0}}}),screen:Screen::Playing,camera_type:Camera(true),clock:Clock(0.5),hand_strength:hand,using:None,inventory:vec![None],offhand:None,res:Res,cooldowns:Cooldowns,gui:Gui{atlas:Default::default()},mount_menu:Mount(None),menu:None,mobs:vec![],armor_stands:vec![],dragon:None,lan:vec![],target:None,hooks:Rc::new(Default::default()),inputs:RefCell::new(vec![]),marker:Default::default()}}}
    impl Game<'_>{
        fn native_window(&self)->bool{true}fn main_hand_attack_speed(&self)->f64{4.}
        fn stack_in(&self,slot:usize)->Option<&ItemStack>{if slot==40{self.offhand.as_ref()}else{self.inventory.get(slot)?.as_ref()}}
        fn held(&self)->Option<&ItemStack>{self.inventory[0].as_ref()}
        fn client_container_menu_text(&self,_:Screen)->Option<(Option<String>,String)>{Some((self.menu.clone(),String::new()))}
        fn client_crosshair_entity(&self)->Option<i32>{self.target}
        fn native_client_crosshair(&self,input:&str,geometry:&mut String){self.inputs.borrow_mut().push(serde_json::from_str(input).unwrap());self.hooks.crosshair(&NullHost,&mut ClientCrosshairEvent{input,geometry});}
        NATIVE_FILL
        CHARGE_TICKS
    }
    #[path="NATIVE_PATH"]mod bridge;
    #[test]fn real_hud_bridge_uses_tick_identity_two_hands_charge_and_visibility(){
        use crate::hud::HudBatch;use serde_json::json;
        let mut g=Game::default();assert!(g.client_crosshair_hide());
        g.hooks.state().registry.enabled.set(false);assert!(!g.client_crosshair_hide());g.hooks.state().registry.enabled.set(true);
        g.hooks.state().behaviors.borrow_mut().crosshair.values.insert("CooldownBar.Enabled".into(),json!(true));
        g.inventory[0]=Some(ItemStack{item:"minecraft:bow".into(),quick:0,group:None});g.offhand=Some(ItemStack{item:"minecraft:ender_pearl".into(),quick:0,group:Some("shared".into())});g.using=Some((0,10));
        let mut b=HudBatch::new();g.client_crosshair_overlay(&mut b);assert!(b.quads.len()>100);assert!(b.quads.iter().any(|p|p.1==0xffff7746));assert!(b.quads.iter().any(|p|p.1.to_be_bytes()[0]<255));
        let input=g.inputs.borrow().last().unwrap().clone();assert!((input["charge"].as_f64().unwrap()-0.41666666).abs()<1e-6);assert!((input["item"].as_f64().unwrap()-0.525).abs()<1e-6);assert_eq!(input["attack"],1.);
        g.offhand.as_mut().unwrap().item="minecraft:crossbow".into();g.offhand.as_mut().unwrap().quick=2;g.using=Some((40,7));g.client_crosshair_overlay(&mut b);assert_eq!(g.inputs.borrow().last().unwrap()["charge"],0.5);
        g.offhand.as_mut().unwrap().item="minecraft:trident".into();g.client_crosshair_overlay(&mut b);assert_eq!(g.inputs.borrow().last().unwrap()["charge"],0.75);
        g.mobs.push(Mob{id:7,removed:false,join:Join(12),hurt_time:6});g.target=Some(7);g.client_crosshair_tick();let input=g.inputs.borrow().last().unwrap().clone();assert_eq!(input["target"],"mob:12");assert_eq!(input["hurt"],6);
        g.lan.push(Lan{hub:Hub{guests:vec![Guest{placed:true,entity_id:8,conn:77,motion:Some(Player{health:Health{hurt:Hurt{hurt_time:9}}})}]}});g.target=Some(8);g.client_crosshair_tick();assert_eq!(g.inputs.borrow().last().unwrap()["hurt"],9);
        g.screen=Screen::Inventory;let mut b=HudBatch::new();g.client_crosshair_overlay(&mut b);assert!(b.quads.is_empty());
        g.menu=Some("minecraft:generic_9x3".into());g.client_crosshair_overlay(&mut b);assert!(b.quads.is_empty());
        g.menu=Some("minecraft:anvil".into());g.client_crosshair_overlay(&mut b);assert!(!b.quads.is_empty());
        g.menu=None;g.mount_menu.0=Some(8);let mut b=HudBatch::new();g.client_crosshair_overlay(&mut b);assert!(!b.quads.is_empty());
        g.screen=Screen::Playing;g.mount_menu.0=None;g.camera_type.0=false;
        {let state=g.hooks.state();let mut m=state.behaviors.borrow_mut();m.crosshair.values.insert("Mode".into(),json!("CS2"));m.crosshair.values.insert("Mode.CS2.ShowInThirdPerson".into(),json!(false));m.crosshair.values.insert("CooldownBar.Enabled".into(),json!(false));}
        let mut b=HudBatch::new();g.client_crosshair_overlay(&mut b);assert!(b.quads.is_empty());
        g.camera_type.0=true;g.client_crosshair_overlay(&mut b);assert_eq!(b.quads.len(),4);
        g.client_crosshair_reset();assert_eq!(g.hooks.state().behaviors.borrow().crosshair.state,Default::default());
    }
}
