#!/usr/bin/env python3
"""Entity-owned ElytraRecast at the real aiStep tail, before tick bookkeeping.

Apply after mining's identity seam. Existing simulation APIs retain their no-op
tail, so server prediction never calls a window's sound or sends its packets.
"""
from pathlib import Path
HERE = Path(__file__).resolve().parent
HOOK_NAMES = ['client_elytra_recast_running']
HELPER_NAMES = ['client_recasts_elytra']


def method(text, signature):
    start = text.index(signature)
    cursor = text.index('{', start) + 1
    depth = 1
    while depth:
        depth += (text[cursor] == '{') - (text[cursor] == '}')
        cursor += 1
    return text[start:cursor]


CAPTURE = '''        // CHEST only: GLIDER is independent of EQUIPPABLE. Capture the
        // immutable stack components; environmental queries run at aiStep TAIL.
        let elytra_recast_local = self.native_window() && self.lan_acting_uuid.is_none()
            && self.player.as_ref().is_some_and(|p| p.native_client_queries_allowed());
        let elytra_recast_running = elytra_recast_local && self.native_client_recasts_elytra();
        let elytra_recast_chest = self.armor[1].as_ref().filter(|stack| stack.count > 0 && &*stack.item != "minecraft:air").map_or(Default::default(), |stack| {
            let view = horizon_client::item_render::stack_view(stack);
            horizon_client::elytra_recast_chest::chest(self.pack.item_renderer.models.components_view(&view))
        });
        let elytra_recast_jump_key = self.keys.held(Action::Jump);
        let elytra_recast_input_jump = input & 16 != 0;
        let mut elytra_recast_sent = false;
        let elytra_recast_audio = &mut self.audio;
        let elytra_recast_remote = &self.remote;
        let mut elytra_ai_tail = |world: &mut MotionWorld<'_, '_, '_>| -> Result<()> {
            if !elytra_recast_local { return Ok(()); }
            let input_jump = elytra_recast_input_jump || world.player.was_auto_jump;
            horizon_client::elytra_recast_motion::ai_step_tail(world, elytra_recast_running,
                elytra_recast_jump_key, input_jump,
                elytra_recast_chest, &mut || {
                    elytra_recast_audio.stop_sounds(Some("minecraft:item.elytra.flying"), Some(7));
                }, &mut || {
                    if let Some(remote) = elytra_recast_remote {
                        // Ordinary Session.send: packet listeners and acceptance
                        // execute synchronously between local flag and cooldown.
                        remote.session.send(S::PlayerCommand(play::PlayerCommand {
                            id: remote.entity_id, action: 6, data: 0,
                        }));
                    }
                    elytra_recast_sent = true;
                })
        };
'''

FALLBACK = '''        if sleeping.is_some() || dead {
            // These branches omit travel, but the local entity still has an
            // aiStep tail. In particular, disabled tails still sample history.
            let mut immobile = MotionWorld {
                entities: &entity_collisions, player: &mut *player,
                scene: &mut self.world, assets: &self.pack.assets,
                collision_binding: self.collision, resources: self.motion,
                health: &self.health_resources,
            };
            elytra_ai_tail(&mut immobile)?;
        }
'''


def apply(edit, once):
    for target, source in [
        ('elytra_recast.rs', HERE.parent/'liquidbounce-behaviors/src/elytra_recast.rs'),
        ('elytra_recast_chest.rs', HERE/'elytra-recast-chest.rs'),
        ('elytra_recast_motion.rs', HERE/'elytra-recast-motion.rs'),
    ]:
        edit('crates/horizon-client/src/'+target, lambda _, p=source: p.read_text(encoding='utf-8'))
    edit('crates/horizon-client/src/client_lib.rs', lambda text: once(text, 'pub mod block_contact;\n',
        'pub mod block_contact;\npub mod elytra_recast;\npub mod elytra_recast_chest;\npub mod elytra_recast_motion;\n'))
    anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda text: once(text, anchor,
        (HERE/'elytra-recast-host.rs').read_text(encoding='utf-8')+'\n'+anchor))

    def motion(text):
        text = once(text, '    pub no_jump_delay: i32,\n',
            '    pub no_jump_delay: i32,\n    pub client_elytra_recast: crate::elytra_recast::ElytraRecast,\n')
        # Add separate tail-aware entry points, preserving every original caller.
        signature = '    fn dry_tick(&mut self, input: TickInput) -> Result<()> {'
        body = method(text, signature)
        extended = once(body, signature, '''    fn dry_tick_with_ai_tail(&mut self, input: TickInput,
        ai_tail: &mut dyn FnMut(&mut Self) -> Result<()>) -> Result<()> {''')
        extended = once(extended, '        Ok(())\n    }', '        ai_tail(self)?;\n        Ok(())\n    }')
        wrapper = '''    fn dry_tick(&mut self, input: TickInput) -> Result<()> {
        self.dry_tick_with_ai_tail(input, &mut |_| Ok(()))
    }
'''
        text = once(text, body, wrapper+extended)
        signature = '''    fn player_movement_tick_with_fluids(
        &mut self,
        direction: [f32; 2],
        jumping: bool,
    ) -> Result<()> {'''
        body = method(text, signature)
        extended = once(body, signature, '''    fn player_movement_tick_with_fluids_and_ai_tail(
        &mut self, direction: [f32; 2], jumping: bool,
        ai_tail: &mut dyn FnMut(&mut Self) -> Result<()>,
    ) -> Result<()> {''')
        extended = once(extended, '        self.dry_tick(TickInput::PreparedLocal {', '        self.dry_tick_with_ai_tail(TickInput::PreparedLocal {')
        extended = once(extended, '            jumping: jumping && !self.player.flying,\n        })?;',
            '            jumping: jumping && !self.player.flying,\n        }, ai_tail)?;')
        wrapper = signature+'\n        self.player_movement_tick_with_fluids_and_ai_tail(direction, jumping, &mut |_| Ok(()))\n    }\n'
        text = once(text, body, wrapper+extended)
        signature = '    fn simulation_tick_with_fluids(&mut self, direction: [f32; 2], jumping: bool) -> Result<()> {'
        body = method(text, signature)
        extended = once(body, signature, '''    fn simulation_tick_with_fluids_and_ai_tail(&mut self, direction: [f32; 2], jumping: bool,
        ai_tail: &mut dyn FnMut(&mut Self) -> Result<()>) -> Result<()> {''')
        extended = once(extended, '        self.player_movement_tick_with_fluids(direction, jumping)?;',
            '        self.player_movement_tick_with_fluids_and_ai_tail(direction, jumping, ai_tail)?;')
        wrapper = signature+'\n        self.simulation_tick_with_fluids_and_ai_tail(direction, jumping, &mut |_| Ok(()))\n    }\n'
        text = once(text, body, wrapper+extended)
        signature = '''    fn local_tick(
        &mut self,
        keys: crate::sprint::Keys,
        sprint: impl FnOnce(&mut PlayerMotion, crate::sprint::Keys) -> Result<()>,
    ) -> Result<()> {'''
        body = method(text, signature)
        extended = once(body, signature, '''    fn local_tick_with_ai_tail(
        &mut self, keys: crate::sprint::Keys,
        sprint: impl FnOnce(&mut PlayerMotion, crate::sprint::Keys) -> Result<()>,
        ai_tail: &mut dyn FnMut(&mut Self) -> Result<()>,
    ) -> Result<()> {''')
        extended = once(extended, '        self.simulation_tick_with_fluids(modified, keys.jump)',
            '        self.simulation_tick_with_fluids_and_ai_tail(modified, keys.jump, ai_tail)')
        wrapper = signature+'\n        self.local_tick_with_ai_tail(keys, sprint, &mut |_| Ok(()))\n    }\n'
        text = once(text, body, wrapper+extended)
        anchor = '    /// A client\'s tick played again by the server\'s movement prediction:'
        extended = '''    /// Window-only aiStep tail; server/history APIs retain their ordinary path.
    pub fn controlled_tick_with_ai_tail(&mut self, keys: crate::sprint::Keys, sprint_window: i32,
        ai_tail: &mut dyn FnMut(&mut Self) -> Result<()>) -> Result<()> {
        self.local_tick_with_ai_tail(keys, |player, keys| {
            player.sprint_input_tick_with_window(keys, sprint_window)
        }, ai_tail)
    }
'''
        return once(text, anchor, extended+anchor)

    edit('crates/horizon-client/src/motion.rs', motion)
    edit('crates/horizon-client/src/motion/body.rs', lambda text: once(text, '            no_jump_delay: 0,\n',
        '            no_jump_delay: 0,\n            client_elytra_recast: Default::default(),\n'))

    def remote(text):
        # Capture after every other snapshot / before disjoint mutable borrows.
        anchor = '        let Some(player) = self.player.as_mut() else {\n            return Ok(());\n        };\n        let was_gliding = player.fall_flying;\n'
        # Other owned patches add player snapshots before was_gliding.
        tick = text.index('        let was_flying = player.flying;')
        head, tail = text[:tick], text[tick:]
        insert = tail.index('        let Some(player) = self.player.as_mut() else {')
        tail = tail[:insert]+CAPTURE+tail[insert:]
        tail = once(tail, '            .controlled_simulation_tick_with_sprint_window(keys, sprint_window)?;',
            '            .controlled_tick_with_ai_tail(keys, sprint_window, &mut elytra_ai_tail)?;')
        tail = once(tail, '                passenger.passenger_block_effects_tick()?;',
            '                passenger.passenger_block_effects_tick()?;\n                elytra_ai_tail(&mut passenger)?;')
        anchor = '        // The server decides fire the window cannot predict (blaze fireballs,'
        tail = once(tail, anchor, FALLBACK+anchor)
        tail = once(tail, '        let started_gliding = player.fall_flying && !was_gliding;',
            '        let started_gliding = player.fall_flying && !was_gliding && !elytra_recast_sent;')
        return head+tail
    edit('crates/horizon-client/src/game/remote.rs', remote)
    return {'hook_names': HOOK_NAMES, 'helper_names': HELPER_NAMES}
