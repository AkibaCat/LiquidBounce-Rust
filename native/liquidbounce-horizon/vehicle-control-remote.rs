    /// Explicit target version of MultiPlayerGameMode.interact, main hand only.
    pub(super) fn vehicle_control_remote_interact(&mut self,id:i32){
        let sneaking=self.remote_shift_down();
        self.send(S::Interact(play::Interact{id,hand:0,location:[0.;3],secondary:sneaking}));
        let swings=if let Some((i,b))=self.boats.iter().enumerate().find(|(_,b)|b.id as i32==id){
            !boat_interact_passes(b.chest,sneaking,b.out_of_control>=60.,self.boat_has_room(i))
        }else if let Some(i)=self.mobs.iter().position(|m|m.id as i32==id){
            self.remote_mob_takes(i,Hand::Main)&&self.remote_mob_swings(i,Hand::Main)
        }else{false};
        if swings{self.with_hand(Hand::Main,Game::swing_arm);}
    }
