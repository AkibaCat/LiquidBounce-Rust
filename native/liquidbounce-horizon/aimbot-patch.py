"""Aimbot WorldRender timing and actual native boxes; no staged files here."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def generalized(s,once):
 s=once(s,'pub fn projected_points(eye: V, b: B) -> Vec<V> {','pub fn projected_points(eye: V, b: B) -> Vec<V> { projected_points_limit(eye,b,128) }\npub fn projected_points_limit(eye: V, b: B, maximum: i32) -> Vec<V> {')
 s=once(s,'(true, false) => (1, 128),','(true, false) => (1, maximum),')
 s=once(s,'(false, true) => (128, 1),','(false, true) => (maximum, 1),')
 s=once(s,'((128. / (dz.len() / dy.len())).sqrt().floor() as i32).clamp(1, 128)','((f64::from(maximum) / (dz.len() / dy.len())).sqrt().floor() as i32).clamp(1, maximum)')
 return once(s,'(n, (128 / n).max(1))','(n, (maximum / n).max(1))')
def apply(edit,once):
 edit('crates/horizon-client/src/game/aimbot.rs',lambda _:(HERE/'aimbot-native.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game/aimbot_ray.rs',lambda _:(HERE.parent/'liquidbounce-behaviors/src/aimbot_ray.rs').read_text(encoding='utf8').replace('crate::auto_bow_math','super::auto_bow_math'))
 def math(s):
  return s if 'pub fn projected_points_limit' in s else generalized(s,once)
 edit('crates/horizon-client/src/game/auto_bow_math.rs',math)
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod auto_rod;','mod auto_rod;\nmod aimbot;\nmod aimbot_ray;'))
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,(HERE/'aimbot-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 edit('crates/horizon-client/src/game/world_primitives.rs',lambda s:once(s,'        let mut primitives = WorldPrimitives::default();','        self.client_aimbot_frame(alpha as f32);\n        let mut primitives = WorldPrimitives::default();'))
 return {'hook_names':['client_aimbot_render'],'helper_names':['client_aimbot_query']}
