impl Game<'_>{
    pub(super) fn client_trajectories_hooks(&self)->Vec<(u32,u32,Vec3,Vec3)>{
        let local=self.remote.as_ref().map_or(0,|r|r.entity_id)as u32;
        self.hook.iter().filter(|h|!matches!(h.state,State::Hooked(_))).map(|h|(h.id,local,h.position,h.velocity))
            .chain(self.remote_hooks.iter().filter(|h|!matches!(h.state,State::Hooked(_))).map(|h|(h.id,h.owner,h.position,h.velocity))).collect()
    }
    pub(super) fn client_trajectories_has_hook(&self,owner:u32)->bool{
        let local=self.remote.as_ref().map_or(0,|r|r.entity_id)as u32;
        (owner==local&&self.hook.is_some())||self.remote_hooks.iter().any(|h|h.owner==owner&&h.owner_fishing)
    }
}
