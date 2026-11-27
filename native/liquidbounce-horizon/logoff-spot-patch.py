"""Real client fake players; windows of local/LAN worlds use the remote protocol too."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/logoff_spot.rs',lambda _:(HERE/'logoff-spot-game.rs').read_text(encoding='utf8'))
 def game(t):
  t=once(t,'mod native_visuals;','mod native_visuals;\nmod logoff_spot;')
  t=once(t,'    player_model: Option<Mob>,','    player_model: Option<Mob>,\n    client_logoff_spots:std::collections::HashMap<u128,logoff_spot::LoggedOffPlayer>,')
  t=once(t,'            player_model: None,','            player_model: None,\n            client_logoff_spots:Default::default(),')
  return once(t,'    pub(super) fn clear_entities(&mut self) {','    pub(super) fn clear_entities(&mut self) {\n        self.client_logoff_clear();')
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(t):
  anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  t=once(t,anchor,(HERE/'logoff-spot-host.rs').read_text(encoding='utf8')+'\n'+anchor)
  return once(t,'    pub(super) fn native_client_tick_pre(&mut self) {','    pub(super) fn native_client_tick_pre(&mut self) {\n        self.client_logoff_tick();')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 def remote(t):
  t=once(t,'    fn remote_use_entity(&mut self, hand: Hand) -> bool {',(HERE/'logoff-spot-remote.rs').read_text(encoding='utf8')+'\n    fn remote_use_entity(&mut self, hand: Hand) -> bool {')
  t=once(t,'            C::RemoveEntities(p) => {','            C::RemoveEntities(p) => {\n                self.client_logoff_capture(&p.ids);')
  old='        self.mobs.clear();'
  if old not in t:raise RuntimeError('remote world entity reset anchor missing')
  return t.replace(old,'        self.client_logoff_clear();\n'+old)
 edit('crates/horizon-client/src/game/remote.rs',remote)
 edit('crates/horizon-client/src/game/remote_removal.rs',lambda t:once(t,'    pub(super) fn remove_entities(&mut self, ids: &[i32]) {','    pub(super) fn remove_entities(&mut self, ids: &[i32]) {\n        self.client_logoff_capture(ids);'))
 return {'hook_names':['client_logoff_spot'],'helper_names':['client_logoff_spot_clear']}
