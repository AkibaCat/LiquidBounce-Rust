#!/usr/bin/env python3
"""Source TargetStrafe input/model-state and SELF move-event native adapters."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 model=(HERE.parent/'liquidbounce-behaviors/src/target_strafe.rs').read_text(encoding='utf8').replace('crate::auto_bow_math','crate::target_strafe_math')
 edit('crates/horizon-client/src/target_strafe.rs',lambda _:model)
 edit('crates/horizon-client/src/target_strafe_math.rs',lambda _:(HERE.parent/'liquidbounce-behaviors/src/auto_bow_math.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/target_strafe_bridge.rs',lambda _:(HERE/'target-strafe-bridge.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/client_lib.rs',lambda s:once(s,'pub mod strafe;','pub mod strafe;\npub mod target_strafe;\npub mod target_strafe_bridge;\npub mod target_strafe_math;'))
 edit('crates/horizon-client/src/game/target_strafe.rs',lambda _:(HERE/'target-strafe-game.rs').read_text(encoding='utf8'))
 def game(s):
  return once(s,'mod aimbot;','mod aimbot;\nmod target_strafe;')
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(s):
  anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  s=once(s,anchor,'    pub(super) fn native_client_target_strafe_query(&mut self,text:&str)->String{self.client_target_strafe_query(text)}\n'+anchor)
  return once(s,'        super::edge_assistance::clear();','        super::edge_assistance::clear();\n        horizon_client::target_strafe_bridge::clear();')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 edit('crates/horizon-client/src/target-strafe-motion.rs',lambda _:(HERE/'target-strafe-motion.rs').read_text(encoding='utf8'))
 def motion(s):
  s=once(s,'    pub fn move_self(&mut self, movement: Vec3) -> Result<()> {\n','    pub fn move_self(&mut self, movement: Vec3) -> Result<()> {\n        let movement = self.target_strafe_move(movement);\n')
  return s+'\ninclude!("target-strafe-motion.rs");\n'
 edit('crates/horizon-client/src/motion.rs',motion)
 edit('crates/horizon-client/src/world_primitives.rs',lambda s:once(s,'pub enum PrimitiveShape {','pub enum PrimitiveShape {\n    /// Per-vertex ARGB quad, used by TargetStrafe orbit gradients.\n    LiquidBounceGradientQuad { points: [[f64;3];4], colors: [u32;4] },'))
 def primitives(s):
  s=once(s,'        self.native_world_primitives(alpha as f32, camera.to_array(), &mut primitives);','        self.native_world_primitives(alpha as f32, camera.to_array(), &mut primitives);\n        self.client_target_strafe_frame(&mut primitives,alpha);')
  anchor='            match &primitive.shape {\n'
  return once(s,anchor,anchor+'''                PrimitiveShape::LiquidBounceGradientQuad { points,colors } if points.iter().all(finite) => {
                    quads.extend(std::array::from_fn::<_,4,_>(|i|plain(relative(points[i]),white,vertex_color(colors[i],255))));
                }
''')
 edit('crates/horizon-client/src/game/world_primitives.rs',primitives)
 return {'hook_names':[],'helper_names':['client_target_strafe_query']}
