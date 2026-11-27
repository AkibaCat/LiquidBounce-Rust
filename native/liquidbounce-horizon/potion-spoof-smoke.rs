#[test]fn enabled_attributes_and_zero_duration_cleanup(){
 use potion_spoof::*;
 #[derive(Default)]struct H{calls:Vec<String>}
 impl Host for H{fn duration(&mut self,id:&str)->Option<i32>{if id=="slowness"{Some(0)}else{Some(80)}}fn add_effect(&mut self,id:&str,amp:i32){self.calls.push(format!("add:{id}:{amp}"));}fn add_attributes(&mut self,id:&str,amp:i32){self.calls.push(format!("attr:{id}:{amp}"));}fn remove_effect(&mut self,id:&str){self.calls.push(format!("remove:{id}"));}fn remove_attributes(&mut self,id:&str){self.calls.push(format!("unattr:{id}"));}}
 let mut p=PotionSpoof::default();assert_eq!(EFFECTS.len(),40);p.effects[0]=EffectSetting{enabled:true,level:3};let mut h=H::default();p.tick(true,&mut h);assert_eq!(h.calls,vec!["add:speed:2","attr:speed:2","remove:slowness","unattr:slowness"]);assert_eq!(EffectSetting{enabled:true,level:i32::MIN}.amplifier(),255);h.calls.clear();p.on_disabled(&mut h);assert!(h.calls.is_empty());
}
