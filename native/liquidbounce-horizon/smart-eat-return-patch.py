#!/usr/bin/env python3
"""MultiPlayerGameMode.useItem RETURN boundaries, after Fishing's transform.

The native action bool controls Horizon effects only. PASS comes exclusively
from pre-use, actual Java dispatch/component/ray/entity facts.
"""
from pathlib import Path

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]

def block(text,signature):
    start=text.index(signature)
    brace=text.index('{',start)
    end=brace+1
    depth=1
    while depth:
        depth+=(text[end]=='{')-(text[end]=='}')
        end+=1
    return text[start:end]

LOCAL_WRAPPER='''    pub(super) fn use_hand(&mut self, block_hit: Option<&Hit>) -> Result<bool> {
        // Guest/server/replay turns keep their existing packet phase.
        if !self.client_smart_eat_player() || self.guest_use.is_some() {
            return self.client_use_hand_committed(block_hit);
        }
        if let Some(hit)=block_hit {
            let previous=self.guest_use;
            self.guest_use=Some(GuestUse::OnBlock(self.hand));
            let result=self.client_use_hand_committed(Some(hit));
            self.guest_use=previous;
            if result? {return Ok(true);}
        }
        self.client_use_item_only(false)
    }
    /// The existing GuestUse value distinguishes useOn from use; it never
    /// changes player identity. Always restore it before emitting RETURN.
    pub(super) fn client_use_item_only(&mut self, include_empty:bool)->Result<bool> {
        if !self.client_smart_eat_player() {return self.client_use_hand_committed(None);}
        let held=self.held().cloned().filter(|s|s.count>0&&&*s.item!="minecraft:air");
        if !include_empty&&held.is_none(){return Ok(false);}
        if self.spectator(){self.native_client_item_interacted(true);return Ok(false);}
        let stack=held.unwrap_or_else(||ItemStack::new("minecraft:air",0));
        let pass=self.client_item_use_pass(&stack);
        let previous=self.guest_use;
        self.guest_use=Some(GuestUse::Item(self.hand));
        let result=self.client_use_hand_committed(None);
        self.guest_use=previous;
        if result.is_ok() {
            if let Some(pass)=pass {self.native_client_item_interacted(pass);}
            // Original client PASS proceeds to the other hand even when the
            // combined local server simulation performed a server-only use.
            if pass==Some(true) {return Ok(false);}
        }
        result
    }
'''

def wrap_remote(text,signature,args,rotated,once):
    original=block(text,signature)
    name=signature.split('fn ',1)[1].split('(',1)[0]
    committed='client_'+name+'_committed'
    renamed=original.replace('fn '+name+'(','fn '+committed+'(',1)
    held='g.held().cloned()'
    if rotated:held+=' .or_else(||Some(ItemStack::new("minecraft:air",0)))'
    predicate='g.with_hand(hand,|g|'+held+'.and_then(|stack|{if !g.spectator()&&g.native_input.claimant(&stack.item).is_some(){None}else{g.client_item_use_pass(&stack)}}))'
    if rotated:predicate='self.native_with_item_rotation(yaw,pitch,|g|'+predicate+')'
    else:predicate=predicate.replace('g.with_hand','self.with_hand',1)
    wrapper=signature+'\n        let pass='+predicate+';\n        if self.spectator()&&pass==Some(true){self.native_client_item_interacted(true);return false;}\n        let result=self.'+committed+'('+args+');\n        if let Some(pass)=pass {self.native_client_item_interacted(pass);}\n        result\n    }'
    return once(text,original,wrapper+'\n\n'+renamed)

def apply(edit,once):
    def native(text):
        helpers=(HERE/'smart-eat-use-result-host.rs').read_text(encoding='utf-8')+'\n'+(HERE/'smart-eat-pickable-host.rs').read_text(encoding='utf-8')
        text=once(text,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',helpers+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n')
        policy=(ROOT/'native/liquidbounce-behaviors/src/smart_eat_use_result.rs').read_text(encoding='utf-8')
        owners=(HERE/'smart-eat-use-owners.rs').read_text(encoding='utf-8')
        text+='\nmod client_smart_eat_use {\n'+policy+'\npub(super) fn owner(item:&str)->Option<UseOwner>{match item {\n'+owners+'\n}}\n}\n'
        return text+'\nmod client_smart_eat_ray {\n'+(HERE/'smart-eat-item-ray.rs').read_text(encoding='utf-8')+'\n}\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',native)
    def remote(text):
        text=wrap_remote(text,'    fn remote_use_item(&mut self, hand: Hand) -> bool {','hand',False,once)
        return wrap_remote(text,'    pub(super) fn remote_use_item_rotated(&mut self, hand: Hand, yaw: f32, pitch: f32) -> bool {','hand,yaw,pitch',True,once)
    edit('crates/horizon-client/src/game/remote.rs',remote)
    def local(text):
        signature='    pub(super) fn use_hand(&mut self, block_hit: Option<&Hit>) -> Result<bool> {'
        return once(text,signature,LOCAL_WRAPPER+'\n'+signature.replace('fn use_hand(','fn client_use_hand_committed('))
    edit('crates/horizon-client/src/game/use_hand.rs',local)
    # InteractionUtils.useItem must emit a result even for an empty hand;
    # normal Minecraft.startUseItem skips empty stacks.
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda text:once(text,'g.use_hand(None).unwrap_or(false) && !g.use_failed','g.client_use_item_only(true).unwrap_or(false) && !g.use_failed'))
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda text:once(text,'        if !self.native_client_fishing_owner() || self.spectator() { return false; }','        if !self.native_client_fishing_owner() { return false; }'))
    # A filled frame's useOn is PASS and must reach Item.use's RETURN.
    edit('crates/horizon-client/src/game/end.rs',lambda text:once(text,'            // Both useOn and use pass on a filled frame.\n            return Ok(true);','            // Both useOn and use pass on a filled frame.\n            return Ok(false);'))
    # Local server simulation formerly boosted before use_hand, bypassing the
    # client's FoodOnAStickItem PASS event. Run that server effect in Item.use.
    def actions(text):
        text=once(text,'        if !on_block\n            && self.riding.is_some()\n','        if !on_block\n            && !self.client_smart_eat_player()\n            && self.riding.is_some()\n')
        return once(text,'                self.spectator_open_menu(hit.pos);\n            }\n            return Ok(None);','                self.spectator_open_menu(hit.pos);\n            }\n            if self.client_smart_eat_player() {\n                for hand in Hand::BOTH {self.with_hand(hand,|g|g.client_use_item_only(false))?;}\n            }\n            return Ok(None);')
    edit('crates/horizon-client/src/game/item_actions.rs',actions)
    def steering(text):
        anchor='        if !on_block\n            && matches!(\n                &*stack.item,\n                "minecraft:carrot_on_a_stick" | "minecraft:warped_fungus_on_a_stick"\n            )\n'
        return once(text,anchor,'        if !on_block && self.client_smart_eat_player() && self.riding.is_some()\n            && self.use_steering_stick(&stack.item) {return Ok(true);}\n'+anchor)
    edit('crates/horizon-client/src/game/use_hand.rs',steering)
    # PlaceOnWaterBlockItem overrides useOn with unconditional PASS. Its
    # superclass placement is invoked only from Item.use's dedicated ray.
    edit('crates/horizon-client/src/game/use_hand.rs',lambda text:once(text,'        // ItemStack.useOn: after the block\'s own useItemOn/useWithoutItem\n','        if on_block&&matches!(&*stack.item,"minecraft:lily_pad"|"minecraft:frogspawn"){return Ok(false);}\n        // ItemStack.useOn: after the block\'s own useItemOn/useWithoutItem\n'))
    return {'item_return_seam':'actual_preuse_dispatch_then_RETURN;native_consumed_bool_never_classifies_PASS','remote':'normal_and_rotated','local':'block_and_item_phases_restored_before_RETURN'}
