#!/usr/bin/env python3
"""NoBlockInteract at Minecraft.startUseItem's useItemOn invocation, after Reach."""
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
HOOK_NAMES = ['client_block_interact']
HELPER_NAMES = ['client_start_use_item']
HOOKS = (HERE / 'no-block-interact-host.rs').read_text(encoding='utf-8')
rules = (ROOT / 'native/liquidbounce-behaviors/src/no_block_interact.rs').read_text(encoding='utf-8')
facts = (ROOT / 'native/liquidbounce-behaviors/src/no_interact_items.rs').read_text(encoding='utf-8')
facts_anchor = 'pub use crate::no_entity_interact::item_facts;'
if rules.count(facts_anchor) != 1:
    raise ValueError('Expected one shared NoInteract item-facts import')
RULES = '#[allow(dead_code)]\nmod client_no_block_rules {\n' + rules.replace(facts_anchor, 'pub mod item_facts {\n' + facts + '\n}') + '\n}\n'


def apply(edit, replace_once):
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda text: replace_once(text,
        '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',
        HOOKS + '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n') + '\n' + RULES)
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda text: replace_once(text,
        '    Game, GameLevel, LevelDimension, Screen,\n', '    Game, GameLevel, Hit, LevelDimension, Screen,\n'))
    edit('crates/horizon-client/src/game/item_actions.rs', lambda text: replace_once(text,
        '            if self.with_hand(hand, |game| game.use_hand(block_hit.as_ref()))? {\n',
        '            // MixinMinecraft intercepts the useItemOn invocation and cancels startUseItem.\n'
        '            if guest.is_none() && block_hit.as_ref().is_some_and(|hit| self.client_cancel_block_interact(hit)) { return Ok(None); }\n'
        '            if self.with_hand(hand, |game| game.use_hand(block_hit.as_ref()))? {\n'))
    def remote(text):
        text = replace_once(text, '    fn remote_use(&mut self) {\n', '    pub(super) fn remote_use(&mut self) {\n')
        return replace_once(text, '    fn remote_use_block(&mut self, hand: Hand) -> bool {\n',
            '    fn remote_use_block(&mut self, hand: Hand) -> bool {\n'
            '        // true consumes the whole click, including the item and offhand fallbacks.\n'
            '        if self.pick().as_ref().is_some_and(|hit| self.client_cancel_block_interact(hit)) { return true; }\n')
    edit('crates/horizon-client/src/game/remote.rs', remote)
    return {'hook_names': HOOK_NAMES, 'helper_names': HELPER_NAMES}
