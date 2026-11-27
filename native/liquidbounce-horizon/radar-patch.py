"""Real RenderedEntities, head/name HUD and original image arrows; after indicator."""
from pathlib import Path
import base64
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
def apply(edit,once):
 edit('crates/horizon-client/src/game/radar.rs',lambda _:(HERE/'radar-game.rs').read_text(encoding='utf8'))
 def game(s):
  s=once(s,'mod indicator;','mod indicator;\nmod radar;')
  s=once(s,'        let mut hud = horizon_client::spans::time("hud", || self.hud(size));','        radar::clear();\n        if !title {self.client_radar_frame(alpha,camera);}\n        let mut hud = horizon_client::spans::time("hud", || self.hud(size));')
  return once(s,'            self.client_indicator_overlay(&mut batch);','            self.client_radar_submit(&mut batch);\n            self.client_indicator_overlay(&mut batch);')
 edit('crates/horizon-client/src/game.rs',game)
 def tracers(s):
  s=once(s,'    fn client_tracers_entities(','    pub(super) fn client_tracers_entities(')
  return once(s,'    fn client_tracers_perspective(','    pub(super) fn client_tracers_perspective(')
 edit('crates/horizon-client/src/game/tracers.rs',tracers)
 def hooks(s):
  anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  s=once(s,anchor,(HERE/'radar-host.rs').read_text(encoding='utf8')+'\n'+anchor)
  s=once(s,'        self.client_tracers_tick();','        self.client_tracers_tick();\n        self.client_radar_tick();')
  return once(s,'        self.client_tracers_reset();','        self.client_tracers_reset();\n        super::radar::clear();')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 images=HERE/'radar-images.rs'
 edit('crates/horizon-client-render/src/hud.rs',lambda s:once(s,'        images.push(("horizon/white".into(), 4, 4, vec![255; 64]));',images.read_text(encoding='utf8')+'\n        images.push(("horizon/white".into(), 4, 4, vec![255; 64]));'))
 return {'hook_names':['client_radar'],'helper_names':[]}
