// SPDX-License-Identifier: GPL-3.0-or-later
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::damage::{Damage,Host as DamageHost};
struct Adapter<'a>(&'a mut dyn Host);
impl DamageHost for Adapter<'_> {
    fn position(&mut self)->[f64;3] {let p=self.0.client_damage_context().expect("Damage requires the active player");[p[0],p[1],p[2]]}
    fn horizontal_collision(&mut self)->bool {self.0.client_damage_context().is_some_and(|p|p[3]!=0.0)}
    fn send_position(&mut self,position:[f64;3],ground:bool,collision:bool) {self.0.client_damage_send(position,ground,collision);}
    fn set_vertical_velocity(&mut self,value:f64) {self.0.client_damage_velocity(value);}
}
pub(crate) fn run(host:&mut dyn Host,settings:Damage) {if host.client_damage_context().is_some(){settings.run(&mut Adapter(host));}}
