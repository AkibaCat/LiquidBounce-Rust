from pathlib import Path
HERE=Path(__file__).resolve().parent

def apply(edit,once):
 edit('crates/horizon-client/src/game/block_outline.rs',lambda _:(HERE/'block-outline-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game.rs',lambda t:once(once(t,'mod native_visuals;','mod native_visuals;\nmod block_outline;'),'        let outline = if block_outline_drawn(','        let outline = if !self.client_block_outline_active() && block_outline_drawn('))
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'block-outline-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
 edit('crates/horizon-client/src/world_primitives.rs',lambda t:once(t,'pub enum PrimitiveShape {','pub enum PrimitiveShape {\n    /// Source-colored quad, used by actual LiquidBounce world render handlers.\n    LiquidBounceQuad { corners: [[f64; 3]; 4], colors: [u32; 4] },'))
 def primitives(t):
  t=once(t,'        self.native_world_primitives(alpha as f32, camera.to_array(), &mut primitives);','        self.native_world_primitives(alpha as f32, camera.to_array(), &mut primitives);\n        self.client_block_outline_geometry(&mut primitives);')
  return once(t,'            match &primitive.shape {','            match &primitive.shape {\n                PrimitiveShape::LiquidBounceQuad { corners, colors } if corners.iter().all(finite) => {\n                    quads.extend((0..4).map(|i|plain(relative(corners[i]),white,vertex_color(colors[i],255))));\n                }')
 edit('crates/horizon-client/src/game/world_primitives.rs',primitives)
 return {'hook_names':['client_block_outline'],'helper_names':[]}
