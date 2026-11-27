"""Observe actual doPlacement callbacks without equating an action bool with success."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/placement_use.rs',lambda _:(HERE/'placement-use-native.rs').read_text(encoding='utf8'))
 def game(s):
  s=once(s,'mod chest_stealer;','mod chest_stealer;\nmod placement_use;')
  s=once(s,'    client_chest_stealer_result: Option<bool>,','    client_chest_stealer_result: Option<bool>,\n    client_placement_result: Option<i8>,')
  return once(s,'            client_chest_stealer_result: None,','            client_chest_stealer_result: None,\n            client_placement_result: None,')
 edit('crates/horizon-client/src/game.rs',game)
 edit('crates/horizon-client/src/game/hand.rs',lambda s:once(s,'    pub(super) fn swing_arm(&mut self) {\n','    pub(super) fn swing_arm(&mut self) {\n        if let Some(result)=self.client_placement_result{if result!=-1{self.client_placement_result=Some(1);}return;}\n'))
 # A local server's SUCCESS_SERVER swing must never become the client's callback.
 edit('crates/horizon-client/src/game/use_hand.rs',lambda s:once(s,'                        if server_only_block_use(self.res.facts().get(cake), branch) {\n                            self.swing_arm();','                        if server_only_block_use(self.res.facts().get(cake), branch) {\n                            if self.client_placement_result.is_some(){self.client_placement_result=Some(-1);}else{self.swing_arm();}'))
 def remote(s):
  # Keep the server-only Success distinction that Horizon's three-variant
  # InteractionResult erases. Its normal callers retain the same result/changes.
  start=s.index('    fn remote_block_reacts(');end=s.index('    fn remote_cauldron_reacts(',start)
  body=s[start:end]
  body=once(body,'        self.with_level(|b, level, _| {','        let server_class = ["AbstractBedBlock","LightBlock","RespawnAnchorBlock","SignBlock"].iter().any(|c|self.res.facts().get(self.world.block(hit.pos[0],hit.pos[1],hit.pos[2])).has_class(c));\n        let (result,changes,server_success)=self.with_level(|b, level, _| {')
  body=once(body,'            let result = b.use_item_on(','            let (result,branch) = b.use_item_on_branch(')
  body=once(body,'            (result, (level.changed.into_iter().collect(), level.sounds))\n        })','            (result, (level.changed.into_iter().collect(), level.sounds),server_class&&branch==horizon_world::level::UseBranch::WithoutItem&&result==InteractionResult::Success)\n        });\n        if self.client_placement_result.is_some()&&server_success{self.client_placement_result=Some(-1);}\n        (result,changes)')
  s=s[:start]+body+s[end:]
  start=s.index('    pub(super) fn remote_use_item_on_at(');end=s.index('    /// `BoneMealItem.useOn`',start)
  body=s[start:end]
  # True is the click terminator for both FAIL and success; preserve real FAIL.
  for old in ['if self.spawn_egg_allowed(&stack.item) {','if self.predict_fire_starter(stack, &hit, hand, seq) {']:
   anchor=old+'\n                    self.with_hand(hand, Game::swing_arm);\n                }'
   body=once(body,anchor,anchor+' else if self.client_placement_result.is_some(){self.client_placement_result=Some(-2);}')
  body=once(body,'                    self.with_hand(hand, |g| g.start_eating(info));','                    let eating=self.with_hand(hand, |g| g.start_eating(info));\n                    if !eating&&self.client_placement_result.is_some(){self.client_placement_result=Some(-2);}')
  body=once(body,'                if placed.is_some() {','                if !reacts&&placed.is_none()&&consumable.is_none()&&self.client_placement_result.is_some(){self.client_placement_result=Some(-2);}\n                if placed.is_some() {')
  return s[:start]+body+s[end:]
 edit('crates/horizon-client/src/game/remote.rs',remote)
 return {'hook_names':[],'helper_names':[]}
