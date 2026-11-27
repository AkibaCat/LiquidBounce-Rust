    /// LiquidBounce DamageParticles: tick snapshots or real setHealth events, then overlay text.
    pub(super) fn native_client_damage_particles(&self,input:&str,draws:&mut String){
        mod_api::dispatch_client_damage_particles(self,mod_api::mods(),input,draws);
    }
