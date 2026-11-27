"""Real container interactions, screen replacement, pointer and silent tag hooks."""
from pathlib import Path
import re
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
def apply(edit,once):
 edit('crates/horizon-client/src/game/chest_stealer.rs',lambda _:(HERE/'chest-stealer-native.rs').read_text(encoding='utf8'))
 def game(s):
  s=once(s,'mod auto_deposit;\n','mod auto_deposit;\nmod chest_stealer;\n')
  s=once(s,'    inventory: Vec<Option<ItemStack>>,\n','    inventory: Vec<Option<ItemStack>>,\n    client_chest_stealer_result: Option<bool>,\n    client_chest_stealer_hide_swing: bool,\n')
  s=once(s,'            selected: 0,\n','            selected: 0,\n            client_chest_stealer_result: None,\n            client_chest_stealer_hide_swing: false,\n')
  s=once(s,'    fn draw_inventory(&mut self, batch: &mut HudBatch) {\n','    fn draw_inventory(&mut self, batch: &mut HudBatch) {\n        if self.client_chest_stealer_hidden(){return;}\n')
  return once(s,'            self.client_damage_particles_overlay(&mut batch);','            self.client_damage_particles_overlay(&mut batch);\n            self.client_chest_stealer_tag(&mut batch);')
 edit('crates/horizon-client/src/game.rs',game)
 def auto(s):
  s=once(s,'    fn auto_deposit_rotation_hit(','    pub(super) fn auto_deposit_rotation_hit(')
  anchor='        value["blocks"]\n            .as_array()'
  return once(s,anchor,'''        if value["aura"]==true {
            let key=(pos[0],pos[1],pos[2]);
            if horizon_client::containers::is_chest_block(name)&&self.chest_menu_blocked(key,name){return false;}
            let other=self.chest_partner(key).map(|(p,_)|[p.0,p.1,p.2]);
            if value["interacted"].as_array().is_some_and(|positions|positions.iter().any(|p|serde_json::from_value::<[i32;3]>(p.clone()).is_ok_and(|p|p==pos||Some(p)==other))){return false;}
        }
'''+anchor)
 edit('crates/horizon-client/src/game/auto_deposit.rs',auto)
 def remote(s):
  s=once(s,'    pub(super) fn remote_use_item_on(&mut self, hand: Hand) -> bool {\n','    pub(super) fn remote_use_item_on(&mut self, hand: Hand) -> bool {self.remote_use_item_on_at(hand,self.pick())}\n    pub(super) fn remote_use_item_on_at(&mut self, hand: Hand, block_hit:Option<super::Hit>) -> bool {\n')
  start=s.index('    pub(super) fn remote_use_item_on_at(')
  end=s.index('    /// `BoneMealItem.useOn`',start)
  body=s[start:end]
  body=once(body,'        if let Some(hit) = self.pick() {','        if let Some(hit) = block_hit {')
  body=once(body,'            let reacts = result.consumes_action();','            if self.client_chest_stealer_result.is_some(){self.client_chest_stealer_result=Some(matches!(result,InteractionResult::Success));}\n            let reacts = result.consumes_action();')
  s=s[:start]+body+s[end:]
  return once(s,'    pub(super) fn send(&mut self, packet: S) {\n','    pub(super) fn send(&mut self, packet: S) {\n        if let S::UseItemOn(p)=&packet{self.client_chest_stealer_sent(p);}\n')
 edit('crates/horizon-client/src/game/remote.rs',remote)
 edit('crates/horizon-client/src/game/hand.rs',lambda s:once(s,'    pub(super) fn swing_arm(&mut self) {\n','    pub(super) fn swing_arm(&mut self) {\n        if self.client_chest_stealer_result.is_some(){self.client_chest_stealer_result=Some(true);if !self.client_chest_stealer_hide_swing{self.swing_hand(self.hand,self.client_chest_stealer_animation());}return;}\n'))
 def pointer(s):
  s=once(s,'    pub(super) fn capture(&mut self, capture: bool) {\n','    pub(super) fn capture(&mut self, capture: bool) {\n        if !capture&&self.focused {let p=self.client_chest_stealer_options();if p["hidden"]==true&&p["unlock"]!=true{return;}}\n')
  return once(s,'            && self.screen == Screen::Playing\n            && (self.player.is_some() || self.studio.flying());','            && (self.screen == Screen::Playing||self.client_chest_stealer_hidden())\n            && (self.player.is_some() || self.studio.flying());')
 edit('crates/horizon-client/src/game/pointer_capture.rs',pointer)
 edit('crates/horizon-client/src/game/window_handler.rs',lambda s:once(s,'            && self.screen == Screen::Playing\n            && let DeviceEvent::MouseMotion','            && (self.screen == Screen::Playing||self.client_chest_stealer_hidden())\n            && let DeviceEvent::MouseMotion'))
 def clicks(s):
  for name,args in [('inventory_hotkey','key: KeyCode) -> bool'),('inventory_pick_key',')'),('inventory_press','right: bool)'),('inventory_drag_move',')'),('inventory_mouse_release','right: bool)'),('inventory_pick_press',')'),('inventory_pick_release',')')]:
   signature='    pub(super) fn '+name+'(&mut self'+(''if args==')'else', ')+args+' {\n'
   result='false'if name=='inventory_hotkey'else''
   s=once(s,signature,signature+'        if self.client_chest_stealer_hidden(){return '+result+';}\n')
  return s
 edit('crates/horizon-client/src/game/inventory_input.rs',clicks)
 # These are the already observed real RHS values, not temporary menu switches.
 def screens(s):
  pattern=r'(?P<owner>(?:self|game|g|guest)(?:\.game)?)\.native_client_replenish_screen\(replenish_screen\)'
  return re.sub(pattern,lambda m:'{ let chest_stealer_screen = '+m[0]+'; '+m['owner']+'.native_client_chest_stealer_screen(chest_stealer_screen) }',s)
 paths=[ROOT/'upstream/Horizon/crates/horizon-client/src/game.rs',*sorted((ROOT/'upstream/Horizon/crates/horizon-client/src/game').rglob('*.rs'))]
 for path in paths:
  if path.name=='native_machines.rs':continue
  source=path.read_text(encoding='utf8')
  if path.name in ('game.rs','native_hooks.rs') or re.search(r'\b(?:self|game|g|guest)(?:\.game)?\.screen\s*=(?![=>])',source):edit(path.relative_to(ROOT/'upstream/Horizon').as_posix(),screens)
 def hooks(s):
  s=once(s,'    fn client_better_inventory_query(&self,','    pub(super) fn client_better_inventory_query(&self,')
  anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  return once(s,anchor,'''    pub(super) fn native_client_chest_stealer_context(&self)->Option<String>{self.chest_stealer_context()}
    pub(super) fn native_client_chest_stealer_use_on(&mut self,payload:&str)->bool{self.chest_stealer_use_on(payload)}
'''+anchor)
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 return {'hook_names':[],'helper_names':['client_chest_stealer_context','client_chest_stealer_use_on']}
