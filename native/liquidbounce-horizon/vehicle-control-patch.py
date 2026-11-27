"""VehicleControl real client owner/controller, speed and rider-link seams."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/vehicle_control.rs',lambda _:(HERE/'vehicle-control-native.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod no_web;\n','mod no_web;\nmod vehicle_control;\n'))
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'vehicle-control-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
 edit('crates/horizon-client/src/game/remote.rs',lambda s:once(s,'    fn remote_use_entity(&mut self, hand: Hand) -> bool {',(HERE/'vehicle-control-remote.rs').read_text(encoding='utf8')+'\n    fn remote_use_entity(&mut self, hand: Hand) -> bool {'))
 edit('crates/horizon-client/src/game/remote_vehicles.rs',lambda s:once(s,'    pub(super) fn driven_mount(&self) -> Option<&Mob> {','''    pub(super) fn vehicle_control_mount_id(&self)->Option<u32>{
        let id=self.riding?;
        self.mobs.iter().find(|m|m.id==id&&client_moves(m,self.remote.as_ref().is_none_or(|r|r.first_passenger),self.steers_mount(m))).map(|m|m.id)
    }
    pub(super) fn driven_mount(&self) -> Option<&Mob> {'''))
 def boats(s):
  s=once(s,'    pub on_ground: bool,','    pub on_ground: bool,\n    pub vehicle_control_in_water: bool,')
  s=once(s,'            on_ground: false,','            on_ground: false,\n            vehicle_control_in_water: false,')
  return once(s,'        if b.push_by_fluid(&self.world, &self.res.motion().fluids).0 > 0.0 {','        b.vehicle_control_in_water=b.push_by_fluid(&self.world, &self.res.motion().fluids).0 > 0.0;\n        if b.vehicle_control_in_water {')
 edit('crates/horizon-client/src/game/boats.rs',boats)
 return {'hook_names':[],'helper_names':['client_vehicle_control_state','client_vehicle_control_apply']}
