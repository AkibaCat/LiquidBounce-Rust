from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/eagle_model.rs',lambda _:(HERE.parent/'liquidbounce-behaviors/src/eagle.rs').read_text(encoding='utf8').replace('crate::random::','crate::eagle_random::'))
 edit('crates/horizon-client/src/eagle_random.rs',lambda _:(HERE.parent/'liquidbounce-behaviors/src/random.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/client_lib.rs',lambda s:once(s,'pub mod keymapping_clicks;','pub mod keymapping_clicks;\npub mod eagle_model;\nmod eagle_random;'))
 def edge(s):
  s=once(s,'struct State {','struct State {\n    eagle:horizon_client::eagle_model::Eagle,')
  s=once(s,'            epoch,\n            player:', '            epoch,\n            eagle:Default::default(),\n            player:')
  s=once(s,'    fn safe_walk(&mut self, config: &Value, mut keys: Input) -> Input {',(HERE/'eagle-native.rs').read_text(encoding='utf8')+'\n    fn safe_walk(&mut self, config: &Value, mut keys: Input) -> Input {')
  return once(s,'                    state.safe_walk(&config, input(raw))','                    let input=state.eagle(&config,input(raw));\n                    state.safe_walk(&config, input)')
 edit('crates/horizon-client/src/game/edge_assistance.rs',edge)
 return {'hook_names':[],'helper_names':[]}
