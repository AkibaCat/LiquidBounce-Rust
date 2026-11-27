// Actual split borrow of the client body, only for this ordinary send chain.
struct AntiHungerPacketPlayer<'p, 'w, 'r> {
    world: std::cell::RefCell<horizon_client::motion::MotionWorld<'p, 'w, 'r>>,
    passenger: bool,
    destroying: bool,
}
impl horizon_client::anti_hunger_bridge::ClientPacketPlayer for AntiHungerPacketPlayer<'_, '_, '_> {
    fn snapshot(&self) -> horizon_client::anti_hunger_bridge::Snapshot {
        let world = self.world.borrow();
        let p = &world.player;
        horizon_client::anti_hunger_bridge::Snapshot {
            swimming: p.swimming,
            passenger: self.passenger,
            destroying: self.destroying,
            in_water: p.fluids.was_touching_water,
            under_water: p.is_under_water(),
            fall_distance: p.fall_distance,
        }
    }
    fn jump_from_ground(&self) {
        if let Err(error) = self.world.borrow_mut().jump_from_ground() {
            eprintln!("[LiquidBounce] AntiHunger jump: {error}");
        }
    }
}
impl Game<'_> {
    pub(super) fn native_send_with_anti_hunger(&mut self, packet: S) {
        let owner = self.native_window()
            && self.lan_acting_uuid.is_none()
            && self.lan_acting.is_none()
            && self
                .player
                .as_ref()
                .is_some_and(|p| p.client_side && p.native_client_queries_allowed());
        let Some(remote) = self.remote.as_ref() else {
            return;
        };
        if !owner {
            remote.session.send(packet);
            return;
        }
        let passenger = self.riding.is_some();
        let destroying = remote.is_destroying;
        let Some(player) = self.player.as_mut() else {
            return;
        };
        player.client_high_jump_owner = true;
        let live = AntiHungerPacketPlayer {
            world: std::cell::RefCell::new(horizon_client::motion::MotionWorld {
                entities: &[],
                player,
                scene: &mut self.world,
                assets: &self.pack.assets,
                collision_binding: self.collision,
                resources: self.motion,
                health: &self.health_resources,
            }),
            passenger,
            destroying,
        };
        remote.session.send_with_client_player(packet, &live);
    }
}
