#!/usr/bin/env python3
"""KillAura AutoBlock release suppression and local sword render expressions.

Apply after kill-aura-patch.py. That patch owns the existing two-field
client_kill_aura_state callback and its dispatcher registration.
"""
from pathlib import Path

HERE = Path(__file__).resolve().parent


def apply(edit, once):
    anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    host = (HERE / 'kill-aura-block-input-host.rs').read_text(encoding='utf8')
    edit('crates/horizon-client/src/game/native_hooks.rs',
         lambda s: once(s, anchor, host + '\n' + anchor))

    def remote(s):
        # SmartEat owns KeyMapping.isDown and AutoBow suppresses release while
        # updating its held bow. Replace only the release expression's key.
        variants = [
            'self.mod_use_tick(slot, ticks, !self.right || !in_hand)',
            'self.mod_use_tick(slot, ticks, !self.client_use_key_down() || !in_hand)',
            'self.mod_use_tick(slot, ticks, (!super::auto_bow::updating()&&!self.client_use_key_down()) || !in_hand)',
        ]
        matches = [v for v in variants if v in s]
        if len(matches) != 1:
            raise ValueError('KillAura expected exactly one vanilla/SmartEat/AutoBow remote release expression')
        old = matches[0]
        new = old.replace('!self.right', '!self.client_kill_aura_use_down()').replace(
            '!self.client_use_key_down()', '!self.client_kill_aura_use_down()')
        s = once(s, old, new)
        # AutoWalk owns a separate release guard at the same entry. Both
        # guards compose regardless of which patch inserts its guard first.
        return once(s, '    fn release_using(&mut self) {\n', '''    fn release_using(&mut self) {
        if self.0.using.is_some() && self.0.client_kill_aura_use_down() { return; }
''')
    edit('crates/horizon-client/src/game/remote.rs', remote)
    def integrated(s):
        variants = [
            '        if !self.right || hand_changed || self.screen != Screen::Playing {\n',
            '        if !self.client_use_key_down() || hand_changed || self.screen != Screen::Playing {\n',
            '        if hand_changed || (!super::auto_bow::updating()&&(!self.client_use_key_down()||self.screen!=Screen::Playing)) {\n',
        ]
        matches = [v for v in variants if v in s]
        if len(matches) != 1:
            raise ValueError('KillAura expected exactly one vanilla/SmartEat/AutoBow integrated release condition')
        old = matches[0]
        return once(s, old, old.replace('!self.right', '!self.client_kill_aura_use_down()').replace(
            '!self.client_use_key_down()', '!self.client_kill_aura_use_down()'))
    edit('crates/horizon-client/src/game/using_item.rs', integrated)

    def first_person(s):
        s = once(s, '        let used_hand = self.used_hand();\n',
                 '        let used_hand = self.used_hand();\n        let kill_aura_block_pose = self.client_kill_aura_block_pose();\n')
        # Keep model property resolution's owner real. The source mixin only
        # changes renderer expressions, not LivingEntity's actual item use.
        s = once(s, '''            let using = owner.using_this_item && owner.use_item_remaining_ticks > 0;
            let ticks_used =
                owner.use_duration as f32 - (owner.use_item_remaining_ticks as f32 - a + 1.0);
''', '''            let render_using = if kill_aura_block_pose {
                hand == Hand::Main
            } else {
                owner.using_this_item
            };
            let render_remaining = if kill_aura_block_pose {
                7200
            } else {
                owner.use_item_remaining_ticks
            };
            let using = render_using && render_remaining > 0;
            let ticks_used =
                owner.use_duration as f32 - (render_remaining as f32 - a + 1.0);
''')
        s = once(s, '            let pose = match &*stack.item {\n', '''            let pose = match &*stack.item {
                // ItemUseAnimation.BLOCK shares the host's modern no-swing
                // Using transform. It does not use the optional 1.8 pose.
                _ if kill_aura_block_pose && hand == Hand::Main
                    && self.client_kill_aura_sword(&stack) => ItemPose::Using,
''')
        s = once(s, '                    remaining: owner.use_item_remaining_ticks as f32,\n',
                 '                    remaining: render_remaining as f32,\n')
        return once(s, '                        owner.use_item_remaining_ticks as f32,\n',
                    '                        render_remaining as f32,\n')
    edit('crates/horizon-client/src/game/frame_geometry.rs', first_person)

    # AvatarRenderer.getArmPose changes only the main arm. A separate render
    # flag preserves actual off-hand use, item model properties and CEM state.
    # SwordBlock.ApplyToThirdPersonView defaults true in the frozen source.
    # Its future module adapter must gate this flag with that real setting.
    edit('crates/horizon-client/src/mobs.rs', lambda s: once(s,
         '    pub blocking: bool,\n', '''    pub blocking: bool,
    /// Local-only AvatarRenderer main-hand pose override, not gameplay use.
    pub client_main_hand_block_pose: bool,
'''))
    def third_person(s):
        s = once(s, '        let using = self.using.map(|(_, ticks)| ticks as i32 + 1).unwrap_or(0);\n',
                 '        let client_main_hand_block_pose = self.client_kill_aura_block_pose();\n        let using = self.using.map(|(_, ticks)| ticks as i32 + 1).unwrap_or(0);\n')
        return once(s, '        m.extra.blocking = blocking;\n',
                    '        m.extra.blocking = blocking;\n        m.extra.client_main_hand_block_pose = client_main_hand_block_pose;\n')
    edit('crates/horizon-client/src/game/perspective.rs', third_person)
    edit('crates/horizon-client/src/mobs/anim.rs', lambda s: once(s,
         '                    Some(_) if using_main && m.extra.blocking => ArmPose::Block,\n',
         '                    Some(_) if m.extra.client_main_hand_block_pose => ArmPose::Block,\n                    Some(_) if using_main && m.extra.blocking => ArmPose::Block,\n'))
    return {'hook_names': [], 'helper_names': []}
