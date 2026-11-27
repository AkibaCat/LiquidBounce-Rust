#!/usr/bin/env python3
"""Criticals real packets, visuals, receive probe and borrowed live player."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,(HERE/'criticals-host.rs').read_text(encoding='utf8')+'\n'+anchor))
    edit('crates/horizon-client/src/game/criticals_target.rs',lambda _:(HERE/'criticals-target.rs').read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod crosshair_falling;\n','mod crosshair_falling;\nmod criticals_target;\n'))
    edit('crates/horizon-client/src/game/remote_poll.rs',lambda s:once(s,'    pub(in crate::game) fn poll_remote(&mut self) -> Result<()> {\n','    pub(in crate::game) fn poll_remote(&mut self) -> Result<()> {\n        self.native_client_criticals_receive_probe();\n'))
    def render(s):
        s=once(s,'        let player = self.player.as_ref();\n        let mobs = &self.mobs;','        let player = self.player.as_ref();\n        let armor_stands=&self.armor_stands;let dragon=&self.dragon;\n        let mobs = &self.mobs;')
        return once(s,'                mob_index.get(mobs, id as u32).map(mob_tracking_body)', '''                if let Some(a)=armor_stands.iter().find(|a|a.id as i32==id){let b=a.bounds();let min=b.min_position();let max=b.max_position();return Some(TrackingBody{feet:[a.position.x,a.position.y,a.position.z],width:max.x-min.x,height:max.y-min.y});}
                if let Some(d)=dragon.as_ref().filter(|d|d.id as i32==id){return Some(TrackingBody{feet:[d.position.x,d.position.y,d.position.z],width:16.,height:8.});}
                mob_index.get(mobs, id as u32).map(mob_tracking_body)''')
    edit('crates/horizon-client/src/game/render_tick.rs',render)
    edit('crates/horizon-client/src/anti_hunger_bridge.rs',lambda s:once(s,'    fn jump_from_ground(&self);','    fn criticals_snapshot(&self)->Option<serde_json::Value>{None}\n    fn jump_from_ground(&self);'))
    def remote(s):
        s=once(s,'    destroying: bool,\n','    destroying: bool,\n    criticals:Option<serde_json::Value>,\n')
        s=once(s,'    fn jump_from_ground(&self) {', '''    fn criticals_snapshot(&self)->Option<serde_json::Value>{
        let mut v=self.criticals.clone()?;let world=self.world.borrow();let p=&world.player;
        let no_liquid_walk=!(horizon_world::liquid_walk::mode().is_some()&&horizon_client::liquid_walk_native::row(world.scene,world.collision_binding,p.bounds(),0.01));
        let allows=v["staticAllows"].as_bool().unwrap_or(false)&&!p.fluids.was_touching_water&&!p.is_in_lava()&&!p.on_climbable&&!p.flying&&no_liquid_walk;
        v["allowsCritical"]=serde_json::json!(allows);v["onGround"]=serde_json::json!(p.ground.on_ground);v["sprinting"]=serde_json::json!(p.sprinting);v["fallDistance"]=serde_json::json!(p.fall_distance);v["y"]=serde_json::json!(p.position.y);v["oldY"]=serde_json::json!(p.old_position.y);v["tickCount"]=serde_json::json!(p.health.tick_count);Some(v)
    }
    fn jump_from_ground(&self) {''')
        anchor='        let passenger = self.riding.is_some();\n'
        s=once(s,anchor,'''        let mut criticals=self.native_client_criticals_context();
        if let Some(v)=criticals.as_mut(){v["staticAllows"]=serde_json::json!(!self.hands_busy()&&self.riding.is_none()&&!self.client_crosshair_in_web()&&!["levitation","blindness","slow_falling"].into_iter().any(|id|self.potions.effects.get(id).is_some()));}
'''+anchor)
        return once(s,'            destroying,\n','            destroying,\n            criticals,\n')
    edit('crates/horizon-client/src/game/remote.rs',remote)
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'fn client_crosshair_in_web(&self) -> bool {','pub(super) fn client_crosshair_in_web(&self) -> bool {'))
    return {'hook_names':['client_criticals_receive_probe'],'helper_names':['client_criticals_context','client_criticals_candidates','client_criticals_packet','client_criticals_stop','client_criticals_visual']}
