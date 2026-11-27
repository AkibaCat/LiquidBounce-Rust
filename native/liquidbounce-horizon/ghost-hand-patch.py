#!/usr/bin/env python3
"""Original global BlockGetter.clip target-first behavior at every runtime clip."""
from pathlib import Path
HERE = Path(__file__).resolve().parent
HOOK_NAMES = ['client_ghost_hand']
HELPER_NAMES = []
HOOKS = (HERE / 'ghost-hand-host.rs').read_text(encoding='utf-8')


def apply(edit, once):
    anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    def native(text):
        text = once(text, anchor, HOOKS + '\n' + anchor)
        # SmartEat owns the actual Item.use ray; transform its composed source.
        text = once(text, 'RayWorld, RayResult, clip_world}', 'RayWorld, RayResult}')
        return once(text, 'clip_world(&mut ClientItemUseRay(game),',
                    'crate::game::client_ghost_hand::clip(game, &mut ClientItemUseRay(game),')
    edit('crates/horizon-client/src/game/native_hooks.rs', native)
    edit('crates/horizon-client/src/game.rs', lambda text: once(text,
        'mod clip_ray;\n', 'mod clip_ray;\nmod client_ghost_hand;\n'))
    game = (HERE / 'ghost-hand-game.rs').read_text(encoding='utf-8').replace('__RAY__',
        (HERE / 'ghost-hand-ray.rs').read_text(encoding='utf-8'))
    edit('crates/horizon-client/src/game/client_ghost_hand.rs', lambda _: game)
    edit('crates/horizon-client/src/game/clip_ray.rs', lambda text: once(text,
        '    ) -> Option<Hit> {\n',
        '    ) -> Option<Hit> {\n        if let Some(hit) = self.client_ghost_hand_outline(origin, dir, reach) { return Some(hit); }\n'))
    for path in ['boats/placement.rs', 'perspective.rs', 'explosion.rs']:
        def transform(text):
            text = once(text, 'RayWorld, clip_world}', 'RayWorld}')
            return once(text, 'let hit = clip_world(' if path != 'explosion.rs' else 'let missed = clip_world(',
                'let hit = crate::game::client_ghost_hand::clip(self,' if path != 'explosion.rs' else
                'let missed = crate::game::client_ghost_hand::clip(self,')
        edit('crates/horizon-client/src/game/' + path, transform)
    return {'hook_names': HOOK_NAMES, 'helper_names': HELPER_NAMES,
            'clip_contexts': ['outline_game', 'item_use', 'boat_item', 'camera_visual', 'explosion_collider']}
