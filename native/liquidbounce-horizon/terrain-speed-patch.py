#!/usr/bin/env python3
"""Complete TerrainSpeed source seams. Apply after Strafe and mining identity."""
from pathlib import Path
import runpy
HERE=Path(__file__).resolve().parent
HOOK_NAMES=['client_terrain_speed']
HELPER_NAMES=[]

def apply(edit,once):
    for target,source in [('terrain_speed.rs',HERE.parent/'liquidbounce-behaviors/src/terrain_speed.rs'),
                          ('terrain_speed_bridge.rs',HERE/'terrain-speed-bridge.rs'),
                          ('terrain_speed_motion.rs',HERE/'terrain-speed-motion.rs')]:
        edit('crates/horizon-client/src/'+target,lambda _,p=source:p.read_text(encoding='utf-8'))
    edit('crates/horizon-client/src/client_lib.rs',lambda t:once(t,'pub mod block_contact;\n',
        'pub mod block_contact;\npub mod terrain_speed;\npub mod terrain_speed_bridge;\npub mod terrain_speed_motion;\n'))
    def host(t):
        anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
        t=once(t,anchor,(HERE/'terrain-speed-host.rs').read_text(encoding='utf-8')+'\n'+anchor)
        return once(t,'        mod_api::dispatch_movement_input(self, mod_api::mods(), forward, strafe, jump, sneak);',
            '''        mod_api::dispatch_movement_input(self, mod_api::mods(), forward, strafe, jump, sneak);
        if let Some(s)=horizon_client::terrain_speed_bridge::snapshot(){
            s.settings.water_speed.movement_input(true,self.player.as_ref().is_some_and(|p|p.fluids.was_touching_water),s.sneak,jump);
        }''')
    edit('crates/horizon-client/src/game/native_hooks.rs',host)
    def motion(t):
        t=once(t,'    pub blocks: BoundMovementProperties,\n','    pub blocks: BoundMovementProperties,\n    pub terrain_climb_blocks: BTreeSet<BlockStateId>,\n')
        t=once(t,'            climbing,\n','''            terrain_climb_blocks: (0..assets.registry.len()).map(|id| BlockStateId(id as u32))
                .filter(|&id|{let f=facts.get(id);f.has_class("LadderBlock")||f.has_class("VineBlock")}).collect(),
            climbing,
''')
        # Strafe already precedes this line in SELF move. Terrain then runs at
        # LocalPlayer.move HEAD, before the original captures its old position.
        t=once(t,'    pub fn move_self(&mut self, movement: Vec3) -> Result<()> {\n        let movement = self.client_strafe_movement(movement);\n',
            '    pub fn move_self(&mut self, movement: Vec3) -> Result<()> {\n        let movement = self.client_strafe_movement(movement);\n        let movement=crate::terrain_speed_motion::move_event(self,movement)?;\n')
        t=once(t,'    pub fn shulker_box_push(&mut self, movement: Vec3) -> Result<()> {\n',
            '    pub fn shulker_box_push(&mut self, movement: Vec3) -> Result<()> {\n        let movement=crate::terrain_speed_motion::move_event(self,movement)?;\n')
        old='''        Ok(self
            .resources
            .blocks
            .get(self.block(p))?
            .coefficients
            .friction)'''
        return once(t,old,'''        let state=self.block(p);
        let base=self.resources.blocks.get(state)?.coefficients.friction;
        let name=&self.assets.registry.state(state).ok_or("friction state outside registry")?.name;
        if self.player.native_client_queries_allowed(){
            Ok(crate::terrain_speed_bridge::player_friction(name,base,&mut self.player.velocity))
        }else{
            Ok(crate::terrain_speed_bridge::friction_for(self.player.entity_id as u32,name,base))
        }''')
    edit('crates/horizon-client/src/motion.rs',motion)
    def remote(t):
        t=once(t,'    fn tick_remote_with_motion_modules(&mut self) -> Result<()> {\n        self.tick_remote_level()?;',
            '''    fn tick_remote_with_motion_modules(&mut self) -> Result<()> {
        let _terrain_scope=horizon_client::terrain_speed_bridge::enter(self.native_terrain_snapshot());
        let terrain_result=self.tick_remote_level();
        self.native_terrain_finish_player();
        terrain_result?;''')
        t=once(t,'''                let before: std::collections::HashSet<u32> =
                    self.mobs.iter().map(|m| m.id).collect();''','''                let before: std::collections::HashSet<u32> = self.mobs.iter().map(|m|m.id)
                    .chain(self.dropped.iter().map(|e|e.id)).chain(self.orbs.iter().map(|e|e.id))
                    .chain(self.armor_stands.iter().map(|e|e.id)).chain(self.boats.iter().map(|e|e.id)).collect();''')
        t=once(t,'        let stick = self.input_move();\n',
            '        self.native_terrain_player_tick();\n        let stick = self.input_move();\n')
        t=once(t,'        self.tick_remote_entities(running);\n','        self.tick_remote_entities(running);\n        self.native_terrain_flush();\n')
        t=once(t,'        let _ = self.handle_mob_events_reusable(&mut mob_fx);\n',
            '        self.native_terrain_flush();\n        let _ = self.handle_mob_events_reusable(&mut mob_fx);\n')
        # Tick completed its last local move; commit absSnapTo's xo/yo/zo before
        # an event callback, avatar model or packet listener can observe it.
        t=once(t,'        let feet = [player.position.x, player.position.y, player.position.z];\n        for event in events {',
            '        let feet = [player.position.x, player.position.y, player.position.z];\n        self.native_terrain_flush();\n        for event in events {')
        return t
    edit('crates/horizon-client/src/game/remote.rs',remote)
    def entities(t):
        assert t.count('friction *= facts.get(below).friction;')==2
        t=t.replace('friction *= facts.get(below).friction;','friction *= crate::terrain_speed_bridge::friction_for(self.id,&facts.get(below).name,facts.get(below).friction);')
        return once(t,'            facts.get(below).friction\n','            crate::terrain_speed_bridge::friction_for(self.id,&facts.get(below).name,facts.get(below).friction)\n')
    edit('crates/horizon-client/src/entities.rs',entities)
    edit('crates/horizon-client/src/mobs/living.rs',lambda t:once(t,
        '        w.facts.get(w.world.block(x, y, z)).friction\n',
        '        let f=w.facts.get(w.world.block(x,y,z));\n        crate::terrain_speed_bridge::friction_for(self.id,&f.name,f.friction)\n'))
    edit('crates/horizon-client/src/mobs/leash.rs',lambda t:once(t,
        '            w.facts.get(w.world.block(x, y, z)).friction * 0.91_f32',
        '            ({let f=w.facts.get(w.world.block(x,y,z));crate::terrain_speed_bridge::friction_for(m.id,&f.name,f.friction)}) * 0.91_f32'))
    def boats(t):
        t=once(t,'        let friction = self.boat_ground_friction(bb);','        let friction = self.boat_ground_friction(b.id,bb);')
        t=once(t,'    fn boat_ground_friction(&self, bb: Aabb) -> f32 {','    fn boat_ground_friction(&self, entity:u32, bb: Aabb) -> f32 {')
        t=once(t,'                        friction += f.friction;','                        friction += horizon_client::terrain_speed_bridge::friction_for(entity,&f.name,f.friction);')
        t=once(t,'        b.status = self.boat_status(&mut b);\n','        b.status = self.boat_status(&mut b);\n        self.native_terrain_flush();\n')
        return once(t,'        let status = self.boat_status(&mut probe);\n','        let status = self.boat_status(&mut probe);\n        self.native_terrain_flush();\n')
    edit('crates/horizon-client/src/game/boats.rs',boats)
    edit('crates/horizon-client/src/game/boat_leads.rs',lambda t:once(t,
        '            self.res.facts().get(self.world.block(x, y, z)).friction * 0.91',
        '            ({let f=self.res.facts().get(self.world.block(x,y,z));horizon_client::terrain_speed_bridge::friction_for(b.id,&f.name,f.friction)}) * 0.91'))
    def ride(t):
        t=once(t,'            mob.tick(&world, &mut random, &mut events);\n        }\n',
            '            mob.tick(&world, &mut random, &mut events);\n        }\n        self.native_terrain_flush();\n')
        return t
    edit('crates/horizon-client/src/game/remote_vehicles.rs',ride)
    def item(t):
        t=once(t,'        let no_physics = escape_speed.is_some();\n',
            '        self.native_terrain_flush();\n        let no_physics = escape_speed.is_some();\n')
        return once(t,'            .settle_item(item, no_physics, step, splash);\n        (alive, false)',
            '            .settle_item(item, no_physics, step, splash);\n        self.native_terrain_flush();\n        (alive, false)')
    edit('crates/horizon-client/src/game/entity.rs',item)
    def piston(t):
        anchor='    fn shulker_push_player(&mut self, movement: Vec3) {\n'
        helper='''    fn terrain_local_move_head(&mut self,movement:Vec3)->std::result::Result<Vec3,Box<dyn std::error::Error>>{
        if horizon_client::terrain_speed_bridge::climb().is_none(){return Ok(movement);}
        let entities=self.player_entity_collisions();
        let result=if let Some(player)=self.player.as_mut(){
            player.yaw=self.yaw;
            horizon_client::terrain_speed_motion::move_event(&mut MotionWorld{
                entities:&entities,player,scene:&mut self.world,assets:&self.pack.assets,
                collision_binding:self.collision,resources:self.motion,health:&self.health_resources,
            },movement)
        }else{Ok(movement)};
        self.native_terrain_flush();result
    }
'''
        t=once(t,anchor,helper+anchor)
        t=once(t,'    fn shulker_push_player(&mut self, movement: Vec3) {\n        let entities = self.player_entity_collisions();\n        if let Some(player) = self.player.as_mut() {',
            '    fn shulker_push_player(&mut self, movement: Vec3) {\n        let entities = self.player_entity_collisions();\n        if let Some(player) = self.player.as_mut() {\n            if horizon_client::terrain_speed_bridge::climb().is_some(){player.yaw=self.yaw;}')
        t=once(t,'            .shulker_box_push(movement);\n        }\n',
            '            .shulker_box_push(movement);\n        }\n        self.native_terrain_flush();\n')
        anchor='''        let Some(limited) = self.limit_piston(target, movement, accumulators) else {
'''
        return once(t,anchor,'''        // This optimized piston path bypasses MotionWorld::move_self.
        // LocalPlayer.move still dispatches before the piston limit, including
        // zero pushes and before reading the post-Clip collision box.
        let (movement,before,at)=if target==Target::Player {
            let Ok(movement)=self.terrain_local_move_head(movement) else{return};
            let Some(p)=self.player.as_ref() else{return};
            (movement,p.bounds(),[p.position.x,p.position.z])
        }else{(movement,before,at)};
'''+anchor)
    edit('crates/horizon-client/src/game/piston_push.rs',piston)
    runpy.run_path(str(HERE/'terrain-speed-order-patch.py'))['apply'](edit,once)
    runpy.run_path(str(HERE/'terrain-speed-passenger-patch.py'))['apply'](edit,once)
    return {'hook_names':HOOK_NAMES,'helper_names':HELPER_NAMES}
