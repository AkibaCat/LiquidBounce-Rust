"""Real trajectory simulation/rendering and item-capable indicator consumer."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 for dst,src in [('trajectories.rs','trajectories-game.rs'),('trajectories_ray.rs','trajectories-ray.rs'),('trajectories_math.rs','trajectories.rs')]:
  source=HERE/src
  if not source.is_file() and src=='trajectories.rs':source=HERE.parent/'liquidbounce-behaviors/src/trajectories.rs'
  edit('crates/horizon-client/src/game/'+dst,lambda _,source=source:source.read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game/trajectories_curve.rs',lambda _:(HERE.parent/'liquidbounce-behaviors/src/tracers_curve.rs').read_text(encoding='utf8'))
 def game(t):
  t=once(t,'mod target_renderer;','mod target_renderer;\nmod trajectories;\nmod trajectories_ray;\nmod trajectories_math;\nmod trajectories_curve;')
  return once(t,'            self.client_indicator_overlay(&mut batch);','            self.client_trajectories_overlay(&mut batch);\n            self.client_indicator_overlay(&mut batch);')
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(t):
  a='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  t=once(t,a,(HERE/'trajectories-host.rs').read_text(encoding='utf8')+'\n'+a)
  t=once(t,'        self.client_breadcrumbs_reset();','        self.client_trajectories_reset();\n        self.client_breadcrumbs_reset();')
  public_root='    pub(super) fn client_reach_root_vehicle(&self, entity: u32) -> u32 {'
  # AutoWalk already exposes the shared vehicle-root helper in the full patch chain.
  return t if public_root in t else once(t,'    fn client_reach_root_vehicle(&self, entity: u32) -> u32 {',public_root)
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 edit('crates/horizon-client/src/game/world_primitives.rs',lambda t:once(t,'        self.client_tracers_frame(alpha,camera,view);','        self.client_tracers_frame(alpha,camera,view);\n        self.client_trajectories_geometry(&mut primitives,alpha,camera,view);'))
 def fishing(t):
  if 'fn client_trajectories_hooks(' in t:return t
  return t+'\n'+(HERE/'trajectories-fishing.rs').read_text(encoding='utf8')
 edit('crates/horizon-client/src/game/fishing.rs',fishing)
 def potions(t):
  if 'fn client_trajectories_potions(' in t:return t
  return t+'\n'+(HERE/'trajectories-potions.rs').read_text(encoding='utf8')
 edit('crates/horizon-client/src/game/potions.rs',potions)
 def indicator(t):
  if 'fn client_indicator_primitive(' in t:return t
  t=once(t,'pub(super) enum Primitive {','pub(super) enum Primitive {\n    Item{stack:horizon_client::items::ItemStack,origin:[f32;2],scale:f32},\n    LiquidBounceText{text:Text,origin:[f32;2],scale:f32,color:u32,shadow:bool},')
  start=t.index('                match primitive {');end=t.index('\n            }\n            let style = ',start)
  match=t[start:end].replace('layout.tag','at')
  t=t[:start]+'                self.client_indicator_primitive(batch,primitive,layout.tag);'+t[end:]
  method='\nimpl Game<\'_>{\n    pub(super) fn client_indicator_primitive(&self,batch:&mut HudBatch,primitive:Primitive,at:[f32;2]){\n'+match+'\n    }\n}\n'
  method=method.replace('                match primitive {','                match primitive {\n'+(HERE/'trajectories-indicator.rs').read_text(encoding='utf8'))
  return t+method
 edit('crates/horizon-client/src/game/indicator.rs',indicator)
 return {'hook_names':['client_trajectories'],'helper_names':[]}
