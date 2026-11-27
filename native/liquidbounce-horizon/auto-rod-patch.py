from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/auto_rod.rs',lambda _:(HERE/'auto-rod-native.rs').read_text(encoding='utf8'))
 def game(s):
  s=once(s,'mod auto_bow;','mod auto_bow;\nmod auto_rod;')
  s=once(s,'    inventory: Vec<Option<ItemStack>>,','    inventory: Vec<Option<ItemStack>>,\n    client_auto_rod_swing:Option<bool>,')
  return once(s,'            selected: 0,','            selected: 0,\n            client_auto_rod_swing:None,')
 edit('crates/horizon-client/src/game.rs',game)
 def expose(s):
  # Keep the already validated AutoBow source unchanged; only its staged
  # entity fact record is shared with AutoRod's real world queries.
  for field in ['dimensions: [f32; 2]','health: f32','hurt: i32','age: i32','weight: i32','target: Value']:
   s=once(s,'    '+field+',','    pub(super) '+field+',')
  return s
 edit('crates/horizon-client/src/game/auto_bow.rs',expose)
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,(HERE/'auto-rod-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 edit('crates/horizon-client/src/game/fishing.rs',lambda s:s+'''\nimpl Hook {
    pub(super) fn auto_rod_facts(&self)->serde_json::Value {
        serde_json::json!({"id":self.id,"hooked":matches!(self.state,State::Hooked(_)),"stationary":self.velocity.x.to_bits()==0&&self.velocity.y.to_bits()==0&&self.velocity.z.to_bits()==0})
    }
}
''')
 edit('crates/horizon-client/src/game/hand.rs',lambda s:once(s,'    pub(super) fn swing_arm(&mut self) {\n','    pub(super) fn swing_arm(&mut self) {\n        if let Some(hide)=self.client_auto_rod_swing{if !hide{self.swing_hand(self.hand,self.auto_rod_animation(self.hand));}return;}\n'))
 return {'hook_names':[],'helper_names':['client_auto_rod_query']}
