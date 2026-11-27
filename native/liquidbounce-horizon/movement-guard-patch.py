#!/usr/bin/env python3
"""Anchor SELF motion, AvoidHazards shapes and two-tick real-body input prediction."""
from pathlib import Path
HERE=Path(__file__).resolve().parent

def apply(edit,once):
    sources=[('crates/horizon-client/src/game/movement_guard.rs',HERE/'movement-guard.rs'),
      ('crates/horizon-client/src/anchor.rs',HERE.parent/'liquidbounce-behaviors/src/anchor.rs'),
      ('crates/horizon-client/src/anchor_bridge.rs',HERE/'anchor-bridge.rs'),
      ('crates/horizon-client/src/avoid_hazards.rs',HERE.parent/'liquidbounce-behaviors/src/avoid_hazards.rs'),
      ('crates/horizon-world/src/avoid_hazards_model.rs',HERE.parent/'liquidbounce-behaviors/src/avoid_hazards.rs'),
      ('crates/horizon-world/src/avoid_hazards.rs',HERE/'avoid-hazards-policy.rs')]
    for target,source in sources:edit(target,lambda _,p=source:p.read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod edge_assistance;\n','mod edge_assistance;\nmod movement_guard;\n'))
    edit('crates/horizon-client/src/client_lib.rs',lambda s:once(s,'pub mod strafe;\n','pub mod strafe;\npub mod anchor;\npub mod anchor_bridge;\npub mod avoid_hazards;\n'))
    edit('crates/horizon-world/src/lib.rs',lambda s:once(s,'pub mod block_walk;\n','pub mod block_walk;\npub mod avoid_hazards;\npub mod avoid_hazards_model;\n'))
    def hooks(s):
        anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
        s=once(s,anchor,'    pub(super) fn native_client_movement_guard_prepare(&mut self,value:&str){self.movement_guard_prepare(value);}\n'+anchor)
        anchor='        [*forward,*backward,*left,*right,*jump,*shift,*sprint]=self.edge_assistance_input([*forward,*backward,*left,*right,*jump,*shift,*sprint],true);\n'
        s=once(s,anchor,anchor+'        [*forward,*backward,*left,*right,*jump,*shift,*sprint]=self.movement_guard_input([*forward,*backward,*left,*right,*jump,*shift,*sprint]);\n')
        return once(s,'        super::edge_assistance::clear();\n','        super::edge_assistance::clear();\n        super::movement_guard::clear();\n')
    edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
    edit('crates/horizon-client/src/motion.rs',lambda s:once(s,'        let movement = self.client_strafe_movement(movement);\n        let movement=crate::terrain_speed_motion::move_event(self,movement)?;\n','        let movement = self.client_strafe_movement(movement);\n        let movement=crate::terrain_speed_motion::move_event(self,movement)?;\n        let movement=if self.player.client_side&&self.player.native_client_queries_allowed(){crate::anchor_bridge::movement(self.player.position,self.player.bounds(),movement)}else{movement};\n'))
    def scan(s):
        anchor='''            let shape = self.world.modify_collision_shape(block, p, shape)
                .map_err(CollisionScanError::World)?;
'''
        return once(s,anchor,anchor+'''            let shape=if crate::avoid_hazards::active(){
                let below=self.world.collision_block(chunk,p.offset(0,-1,0)).map_err(CollisionScanError::World)?;
                crate::avoid_hazards::shape(block.state,below.state,shape)
            }else{shape};
''')
    edit('crates/horizon-world/src/block_collision_scan.rs',scan)
    for target in ['crates/horizon-client-render/src/collisions.rs','crates/horizon-world/src/registry_collision_scan.rs']:
        def active(s):
            old='horizon_world::block_walk::active()' if 'client-render' in target else 'crate::block_walk::active()'
            new='('+old+' || '+('horizon_world' if 'client-render' in target else 'crate')+'::avoid_hazards::active())'
            count=s.count(old)
            if count==0:raise ValueError('BlockShapeEvent active seam missing')
            return s.replace(old,new)
        edit(target,active)
    return {'hook_names':[],'helper_names':['client_movement_guard_prepare']}
