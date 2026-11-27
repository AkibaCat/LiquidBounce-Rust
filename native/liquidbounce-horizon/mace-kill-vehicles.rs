// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use super::entity::{Entity as _, with_entity_mut};
use horizon_core::geometry::Vec3;
use horizon_protocol::packets::play::MoveVehicleRequest;
impl Game<'_> {
    pub(super) fn client_mace_kill_vehicle(
        &mut self,
        id: u32,
        position: Option<[f64; 3]>,
    ) -> Option<MoveVehicleRequest> {
        let tracked = self
            .remote
            .as_mut()
            .and_then(|r| r.liquidbounce_mace_kill_state(id as i32, None, false));
        // setPos leaves old positions, velocity, passengers and ground intact.
        if let Some(handle) = self.native_walk_entity(id) {
            return Some(with_entity_mut!(self,handle,e=>{
                if let Some([x,y,z])=position{*e.position_mut()=Vec3::new(x,y,z);}
                let p=e.position();let [yaw,pitch]=e.rotation();
                MoveVehicleRequest{position:[p.x,p.y,p.z],yaw,pitch,on_ground:e.on_ground()}
            }));
        }
        if let Some(display) = self.text_displays.get_mut(&(id as i32)) {
            let (position, [yaw, pitch]) = display.liquidbounce_mace_kill_position(position);
            return Some(MoveVehicleRequest {
                position,
                yaw,
                pitch,
                on_ground: tracked.is_some_and(|(_, _, ground)| ground),
            });
        }
        let target = self
            .native_entity(id)
            .and_then(|entity| self.native_entity_target(entity));
        let mut rotation = tracked.map_or([0.0, 0.0], |(_, rotation, _)| rotation);
        if let Some(e) = self.others.iter().find(|e| e.id == id) {
            rotation = [e.yaw, e.pitch];
        }
        if let Some(e) = self.fangs.list.iter().find(|e| e.id == id) {
            rotation[0] = e.yaw;
        }
        if let Some(e) = self.dragon.as_ref().filter(|e| e.id == id) {
            rotation[0] = e.yaw;
        }
        if let Some(e) = self.cushions.iter().find(|e| e.id == id) {
            rotation[0] = e.yaw;
        }
        let mut hanging = None;
        let mut frame_codec = false;
        if let Some([x, y, z]) = position {
            let at = Vec3::new(x, y, z);
            if let Some(e) = self.others.iter_mut().find(|e| e.id == id) {
                e.position = at;
            }
            if let Some(e) = self.crystals.iter_mut().find(|e| e.id == id) {
                e.position = at;
            }
            if let Some(e) = self.dragon.as_mut().filter(|e| e.id == id) {
                e.position = at;
            }
            if let Some(e) = self.cushions.iter_mut().find(|e| e.id == id) {
                e.position = at;
                e.pos = [x.floor() as i32, y.floor() as i32, z.floor() as i32];
            }
            if let Some(e) = self.fangs.list.iter_mut().find(|e| e.id == id) {
                e.pos = [x, y, z];
            }
            if let Some(e) = self.ominous_spawners.iter_mut().find(|e| e.id == id) {
                e.position = [x, y, z];
            }
            if let Some(e) = self.breath_clouds.iter_mut().find(|e| e.id == id) {
                e.at = [x, y, z];
            }
            if let Some(e) = self.interactions.get_mut(&(id as i32)) {
                e.position = at;
            }
            // BlockAttachedEntity.setPos stores BlockPos.containing, then
            // HangingEntity.recalculateBoundingBox sets its centre as position.
            if let Some(e) = self.paintings.iter_mut().find(|e| e.id == id) {
                e.pos = [x.floor() as i32, y.floor() as i32, z.floor() as i32];
            }
            if let Some(e) = self.item_frames.iter_mut().find(|e| e.id == id) {
                e.pos = [x.floor() as i32, y.floor() as i32, z.floor() as i32];
                frame_codec = true;
            }
            if let Some(block) = self.remote_knots.get_mut(&id) {
                *block = [x.floor() as i32, y.floor() as i32, z.floor() as i32];
            }
        }
        if let Some(e) = self.paintings.iter().find(|e| e.id == id) {
            let (lo, hi) = e.bounds();
            hanging = Some(std::array::from_fn(|i| (lo[i] + hi[i]) / 2.0));
            if tracked.is_none() {
                rotation = [f32::from(e.facing & 3) * 90.0, 0.0];
            }
        }
        if let Some(e) = self.item_frames.iter().find(|e| e.id == id) {
            let (lo, hi) = e.bounds();
            hanging = Some(std::array::from_fn(|i| (lo[i] + hi[i]) / 2.0));
        }
        if let Some(block) = self.remote_knots.get(&id) {
            let p = horizon_client::mobs::leash::knot_state(*block).position;
            hanging = Some([p.x, p.y, p.z]);
        }
        let at = target
            .and_then(|target| self.target_position(target))
            .or(hanging)
            .or_else(|| {
                self.interactions.get(&(id as i32)).map(|e| {
                    let p = e.position;
                    [p.x, p.y, p.z]
                })
            })
            .or_else(|| {
                self.remote
                    .as_mut()
                    .and_then(|r| r.liquidbounce_mace_kill_state(id as i32, position, false))
                    .map(|(p, _, _)| p)
            })?;
        if position.is_some() && (target.is_none() || frame_codec) {
            // ItemFrame.recalculateBoundingBox also syncs its packet codec;
            // other setPos calls leave that server delta base alone.
            if let Some(r) = self.remote.as_mut() {
                r.liquidbounce_mace_kill_state(id as i32, Some(at), frame_codec);
            }
        }
        let on_ground = self.crystals.iter().find(|e| e.id == id).map_or_else(
            || tracked.is_some_and(|(_, _, ground)| ground),
            |e| e.fall.0,
        );
        let [yaw, pitch] = rotation;
        Some(MoveVehicleRequest {
            position: at,
            yaw,
            pitch,
            on_ground,
        })
    }
}
