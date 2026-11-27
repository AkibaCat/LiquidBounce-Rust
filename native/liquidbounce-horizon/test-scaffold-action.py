#!/usr/bin/env python3
"""Scaffold lifecycle and direct placement bridge: static only, no Rust execution."""
from pathlib import Path
import ast
import hashlib
import json
import runpy
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
UPSTREAM = ROOT / 'upstream/Horizon'
RULES = ROOT / 'native/liquidbounce-behaviors/src/scaffold_lifecycle.rs'


def main():
    sources = [RULES, HERE / 'scaffold-action-native.rs', HERE / 'scaffold-action-host.rs',
               HERE / 'scaffold-action-patch.py', HERE / 'scaffold-placement-state.rs',
               HERE / 'scaffold-lifecycle-smoke.rs', HERE / 'scaffold-BlockItem.javap.txt',
               HERE / 'scaffold-SpecialBlockItem.javap.txt',
               HERE / 'scaffold-click-inside-native.rs', HERE / 'scaffold-click-inside-patch.py']
    before = {path: path.read_bytes() for path in sources}
    for path in [Path(__file__), HERE / 'scaffold-action-patch.py', HERE / 'scaffold-click-inside-patch.py']:
        ast.parse(path.read_text(encoding='utf8'))
    generated = {}

    def once(text, old, new):
        assert text.count(old) == 1, (old, text.count(old))
        return text.replace(old, new)

    def edit(name, transform):
        if name not in generated:
            path = UPSTREAM / name
            generated[name] = path.read_text(encoding='utf8') if path.exists() else ''
        generated[name] = transform(generated[name])

    runpy.run_path(str(HERE / 'block-placement-patch.py'))['apply'](edit, once)
    fragment = runpy.run_path(str(HERE / 'scaffold-action-patch.py'))['apply'](edit, once)
    assert runpy.run_path(str(HERE / 'scaffold-click-inside-patch.py'))['apply'](edit, once) == {'hook_names': [], 'helper_names': []}
    assert fragment == {'hook_names': ['client_scaffold_success'], 'helper_names': ['client_scaffold_action']}
    assert len(generated) == 9, generated.keys()
    native = generated['crates/horizon-client/src/game/scaffold_action.rs']
    compact = ''.join(native.split()).replace(',)', ')')
    for token in ['p.move_vector!=[0.0,0.0]', 'keys[4]||p.was_auto_jump',
                  'self.keys.held(horizon_client::actions::Action::Jump)',
                  'self.with_hand(hand,|g|g.held().cloned())?',
                  'inside:inside(&request)?',
                  'self.inventory.get(self.client_hotbar_slot())',
                  'CollisionContext::placement(Some(CollisionEntity',
                  'descending:player.shift_key_down', 'player.powder_snow_subject()',
                  'unobstructed(level,pos,state,&CollisionContext::empty(false),None)',
                  'collision_shape_with_context(&mutscene,state,position,context)',
                  'BuildingObstruction(level.building_blocker_boxes())',
                  'self.client_scaffold_do_placement(hit,inside(&request)?,rotation,hand,swing,|g,_|{g.native_client_scaffold_success(placed,simulated);true})',
                  'MovePlayerPosRot', 'u8::from(p.ground.on_ground)|(u8::from(p.collision.horizontal)<<1)']:
        assert token in compact, token
    assert 'controls.jump' not in native and 'keys[..4]' not in native
    placement = ''.join((HERE / 'scaffold-placement-state.rs').read_text(encoding='utf8').split())
    for token in ['ctx.replace_clicked=sim.can_be_replaced(sim.state(input.clicked_pos),&ctx)',
                  'ifbinding.game_master&&!input.can_use_game_master_blocks{returnNone;}',
                  'ItemKind::StandingAndWall|ItemKind::HangingSign',
                  'sim.wall_hanging_sign_can_place(candidate,ctx.pos)',
                  'unobstructed(sim.level,ctx.pos,*state,false)',
                  'letmust_survive=binding.kind!=ItemKind::Scaffolding',
                  'unobstructed(sim.level,ctx.pos,*state,true)']:
        assert token in placement, token
    assert 'scaffolding_update_context' not in placement
    assert 'sim.can_be_replaced(sim.state(ctx.pos)' not in placement
    assert 'sim.level.is_unobstructed' not in placement
    rules = ''.join(RULES.read_text(encoding='utf8').split())
    for token in ['self.jumps=self.jumps.wrapping_add(1)', 'self.start_y=block_y;self.jumps=2',
                  '(!facts.physical_jump).then_some(self.placement_y)',
                  '(facts.velocity_y<0.2).then_some(self.placement_y)',
                  'facts.velocity_y==-0.15233518685055708&&self.jumps>=2',
                  'a.same_y!=SameY::JumpKey||a.physical_jump',
                  '(point[axis]-eye[axis]).abs()>=f64::from(min_dist)']:
        assert token in rules, token
    original = (ROOT / 'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/world/scaffold/ModuleScaffold.kt').read_text(encoding='utf8')
    for token in ['input.moveVector', 'sameYMode != SameYMode.JUMP_KEY || mc.options.keyJump.isDown',
                  '(stack.item as BlockItem).getPlacementState(BlockPlaceContext(context)) != null',
                  'if (hasBlockInMainHand) InteractionHand.MAIN_HAND else InteractionHand.OFF_HAND']:
        if token == 'input.moveVector':
            entity = (ROOT / 'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/utils/entity/EntityExtensions.kt').read_text(encoding='utf8')
            assert token in entity
        else:
            assert token in original, token
    hooks = generated['crates/horizon-client/src/game/native_hooks.rs']
    assert hooks.count('if !self.native_window() || !self.native_client_slot_owner()') == 2
    do_placement = (HERE / 'placement-use-native.rs').read_text(encoding='utf8')
    assert do_placement.index('let swing = on_success(g, false)') < do_placement.index('g.swing_hand(hand, animation)')
    scoped = generated['crates/horizon-client/src/game/scaffold_click_inside.rs']
    assert scoped.index('self.client_scaffold_click_inside.replace(inside)') < scoped.index('self.client_do_placement(') < scoped.index('self.client_scaffold_click_inside = previous')
    assert 'resume_unwind' in scoped
    assert generated['crates/horizon-client/src/game/remote.rs'].count('inside: self.client_scaffold_click_inside.unwrap_or(hit.face == [0; 3])') == 2
    assert 'inside: self.client_scaffold_click_inside.unwrap_or(false)' in generated['crates/horizon-client/src/game/use_hand.rs']
    with tempfile.TemporaryDirectory(prefix='liquidbounce-scaffold-action-') as temporary:
        paths = []
        for i, (name, content) in enumerate(generated.items()):
            path = Path(temporary) / (str(i) + '-' + Path(name).name)
            path.write_text(content, encoding='utf8')
            paths.append(str(path))
        for path in [RULES, HERE / 'scaffold-lifecycle-smoke.rs']:
            copy = Path(temporary) / path.name
            copy.write_bytes(path.read_bytes())
            paths.append(str(copy))
        subprocess.run(['rustfmt', '--edition', '2024', '--config', 'skip_children=true', *paths], check=True)
    assert all(path.read_bytes() == raw for path, raw in before.items())
    evidence = {'compiled': False, 'rustExecuted': False, 'generatedRustSourcesParsed': 9,
                'preparedRustSmokeParsed': True, 'checks': [
                    'physical mapping versus transformed input including auto-jump',
                    'actual move-vector moving predicate', 'direct virtual getPlacementState',
                    'GameMaster override retained; canPlace and scaffolding context update skipped',
                    'ordinary placement collision context versus standing-wall empty context',
                    'effective main-hand collision state and real entity obstruction',
                    'synchronous success callback before swing', 'scoped inside survives packet/local/preview contexts and unwind', 'source SameY and attempt branches',
                    'source silent-slot cancellation quirk and main-hand priority'],
                'sha256': {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in sources}}
    (HERE / 'scaffold-action-evidence.json').write_text(json.dumps(evidence, ensure_ascii=False, indent=2) + '\n', encoding='utf8')
    print('Scaffold action/lifecycle static checks passed: 9 complete native sources, rules and prepared smoke parsed; no Cargo/rustc/Rust execution or shared/staging writes')


if __name__ == '__main__':
    main()
