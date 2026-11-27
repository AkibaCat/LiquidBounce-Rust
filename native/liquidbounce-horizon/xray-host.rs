    /// Change the native XRay policy and reload every loaded section when it changes.
    pub(super) fn native_client_xray_policy(&mut self,input:&str) {
        let changed=horizon_client::xray::configure(input,||self.res.facts().states.iter().enumerate().map(|(i,s)|{
            let flags=self.res.behaviours().facts(horizon_world::registry::BlockStateId(i as u32)).flags;
            horizon_client::xray::State{name:s.name.clone(),air:flags&horizon_world::level::data::state_flags::AIR!=0,conductor:flags&horizon_world::level::data::state_flags::CONDUCTOR!=0}
        }).collect());
        if changed {let context=self.worker_context();self.clear_lod();self.streaming.reload_resources(&mut self.world,context);}
    }
    /// Actual loaded block registry names and vanilla translated descriptions.
    pub(super) fn native_client_xray_blocks(&self)->String {
        let mut blocks=std::collections::BTreeMap::new();
        for state in &self.res.facts().states {let id=state.name.as_ref();let (namespace,path)=id.split_once(':').unwrap_or(("minecraft",id));let key=format!("block.{namespace}.{}",path.replace('/',"."));blocks.entry(id.to_owned()).or_insert_with(||horizon_client::lang::tr(&key,&[]));}
        serde_json::to_string(&blocks).unwrap_or_else(|_|"{}".into())
    }
