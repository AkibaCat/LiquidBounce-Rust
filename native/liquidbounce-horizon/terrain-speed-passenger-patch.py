"""Keep Entity.rideTick's ZERO -> tick -> positionRider consumption order."""
from pathlib import Path
import runpy
HERE=Path(__file__).resolve().parent
method=runpy.run_path(str(HERE/'terrain-speed-order-patch.py'))['method']

def apply(edit,once):
    def vehicles(t):
        body=method(t,'    pub(super) fn remote_ride(')
        start=body.index('        // Settle an invisible seat')
        prefix=body[body.index('\n')+1:start]
        position=body[start:body.rfind('    }')]
        position=once(position,'        self.position_riders();','')
        position=once(position,'        if let Some(i) = self.mobs.iter().position(|m| m.id == id) {','        if self.mobs.iter().any(|m| m.id == id) {')
        position=once(position,'            let mut b = self.boats[i].clone();','            let b = self.boats[i].clone();')
        position=once(position,'''            // LocalPlayer.aiStep: the jump bar charges while jump is held.
            let jump = self.charge_mount_jump(i, keys);
            if self.drives(&self.mobs[i]) {
                self.drive_mount(i, keys, jump);
            }
''','')
        position=once(position,'''            if self.controls_boat(id) {
                if !horizon_client::terrain_speed_bridge::was_driven(id){b=self.tick_boat(b,true);}
                b.input = [keys.left, keys.right, keys.forward, keys.backward];
            }
''','')
        position='''    pub(super) fn native_terrain_position_player_ride(&mut self)->Result<bool>{
        let Some(id)=self.riding else{return Ok(false)};
'''+position+'    }\n'
        inputs='''    fn native_terrain_ride_input(&mut self,keys:Keys,sprint_window:i32)->Result<bool>{
'''+prefix+'''        if let Some(i)=self.mobs.iter().position(|m|m.id==id){
            let jump=self.charge_mount_jump(i,keys);
            if self.drives(&self.mobs[i]){self.drive_mount(i,keys,jump);}
            return Ok(true);
        }
        if let Some(i)=self.boats.iter().position(|b|b.id==id)&&self.controls_boat(id){
            self.boats[i].input=[keys.left,keys.right,keys.forward,keys.backward];
        }
        let exists=self.native_terrain_entity_exists(id)
            ||self.cushions.iter().any(|c|c.id==id);
        if !exists{self.riding=None;}
        Ok(exists)
    }
'''
        replacement=once(body,'        let Some(id) = self.riding else {','        if horizon_client::terrain_speed_bridge::ordered(){return self.native_terrain_ride_input(keys,sprint_window);}\n        let Some(id) = self.riding else {')
        return once(t,body,position+inputs+replacement)
    edit('crates/horizon-client/src/game/remote_vehicles.rs',vehicles)
    def mounts(t):
        body=method(t,'    pub(super) fn position_rider(')
        body=once(body,'            p.velocity = Vec3::ZERO;','            if !horizon_client::terrain_speed_bridge::ordered(){p.velocity=Vec3::ZERO;}')
        return once(t,method(t,'    pub(super) fn position_rider('),body)
    edit('crates/horizon-client/src/game/mounts.rs',mounts)
    def passengers(t):
        body=method(t,'    pub(super) fn position_riders(')
        body=once(body,'            for i in 0..self.mobs.len() {','            for i in 0..self.mobs.len() {\n                if !horizon_client::terrain_speed_bridge::selected(self.mobs[i].id){continue;}')
        body=once(body,'            for display in self.text_displays.values_mut() {','            for (&id,display) in self.text_displays.iter_mut() {\n                if !horizon_client::terrain_speed_bridge::selected(id as u32){continue;}')
        body=once(body,'            for interaction in self.interactions.values_mut() {','            for (&id,interaction) in self.interactions.iter_mut() {\n                if !horizon_client::terrain_speed_bridge::selected(id as u32){continue;}')
        body=once(body,'        for i in 0..self.armor_stands.len() {','        for i in 0..self.armor_stands.len() {\n            if !horizon_client::terrain_speed_bridge::selected(self.armor_stands[i].id){continue;}')
        body=body.replace('m.velocity = Vec3::ZERO;','if !horizon_client::terrain_speed_bridge::ordered(){m.velocity=Vec3::ZERO;}')
        body=body.replace('stand.velocity = Vec3::ZERO;','if !horizon_client::terrain_speed_bridge::ordered(){stand.velocity=Vec3::ZERO;}')
        return once(t,method(t,'    pub(super) fn position_riders('),body)
    edit('crates/horizon-client/src/game/passengers.rs',passengers)
    for file,signature in [('boat_passengers.rs','    pub(super) fn seat_boat_mobs('),('minecarts.rs','    pub(super) fn seat_cart_mobs('),('tnt.rs','    pub(super) fn seat_tnt_mobs('),('cushions.rs','    pub(super) fn seat_cushion_mobs('),('seats.rs','    pub(super) fn seat_static_mobs(')]:
        def seating(t,signature=signature):
            original=method(t,signature)
            body=once(original,'                let m = &mut self.mobs[j];','                if !horizon_client::terrain_speed_bridge::selected(self.mobs[j].id){continue;}\n                let m = &mut self.mobs[j];')
            body=once(body,'                m.velocity = Vec3::ZERO;','                if !horizon_client::terrain_speed_bridge::ordered(){m.velocity=Vec3::ZERO;}')
            return once(t,original,body)
        edit('crates/horizon-client/src/game/'+file,seating)
