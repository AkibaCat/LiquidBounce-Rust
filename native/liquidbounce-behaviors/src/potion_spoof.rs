// SPDX-License-Identifier: GPL-3.0-or-later
//! ModulePotionSpoof, LiquidBounce ae0eb396. Registry order is observable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectSetting { pub enabled: bool, pub level: i32 }
impl Default for EffectSetting { fn default()->Self {Self {enabled:false,level:1}} }
impl EffectSetting {
    /// The JVM subtracts with wrapping, then MobEffectInstance clamps 0..255.
    pub fn amplifier(self)->i32 {self.level.wrapping_sub(1).clamp(0,255)}
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PotionSpoof { pub effects: Vec<EffectSetting> }
impl Default for PotionSpoof {fn default()->Self {Self {effects:vec![EffectSetting::default();EFFECTS.len()]}}}
pub trait Host {
    fn duration(&mut self,id:&str)->Option<i32>;
    fn add_effect(&mut self,id:&str,amplifier:i32);
    fn add_attributes(&mut self,id:&str,amplifier:i32);
    fn remove_effect(&mut self,id:&str);
    fn remove_attributes(&mut self,id:&str);
}
impl PotionSpoof {
    pub fn tick(&self,running:bool,host:&mut impl Host) {
        if !running {return;}
        for ((id,_),setting) in EFFECTS.iter().zip(&self.effects) {
            if setting.enabled {let amp=setting.amplifier();host.add_effect(id,amp);host.add_attributes(id,amp);}
            else if host.duration(id)==Some(0) {host.remove_effect(id);host.remove_attributes(id);}
        }
    }
    pub fn on_disabled(&self,host:&mut impl Host) {
        for ((id,_),setting) in EFFECTS.iter().zip(&self.effects) {
            if setting.enabled && host.duration(id)==Some(0) {host.remove_effect(id);}
        }
    }
}
// Generated from pinned Horizon MobEffects registration table and the official
// MC 26.3 client JAR's assets/minecraft/lang/en_us.json by generate-potion-spoof.py.
include!("potion_spoof_effects.rs");
