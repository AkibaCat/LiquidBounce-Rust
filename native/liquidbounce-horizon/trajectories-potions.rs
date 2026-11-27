impl Game<'_>{
    pub(super) fn client_trajectories_potions(&self)->Vec<(u32,Option<u32>,Vec3,Vec3,ItemStack)>{
        self.potions.thrown.iter().map(|t|{
            let owner=match t.thrower{
                Some(Who::Mob(id))=>Some(id),
                Some(Who::Player)=>Some(t.guest.and_then(|conn|self.lan.as_ref()?.hub.guests.iter().find(|g|g.conn==conn).map(|g|g.entity_id as u32)).unwrap_or(self.remote.as_ref().map_or(0,|r|r.entity_id)as u32)),
                None=>None,
            };
            (t.id,owner,t.position,t.velocity,t.stack.clone())
        }).collect()
    }
}
