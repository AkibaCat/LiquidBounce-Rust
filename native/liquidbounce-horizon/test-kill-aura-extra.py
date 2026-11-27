#!/usr/bin/env python3
"""Check non-mob action API composition without compiling or running Rust."""
from pathlib import Path
import ast
import runpy
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
UPSTREAM = ROOT / 'upstream/Horizon'


def main():
    names = ['kill-aura-extra-candidates.rs', 'kill-aura-extra-attack.rs',
             'kill-aura-extra-patch.py']
    before = {name: (HERE / name).read_bytes() for name in names}
    ast.parse((HERE / 'kill-aura-extra-patch.py').read_text(encoding='utf8'))
    ast.parse(Path(__file__).read_text(encoding='utf8'))
    composed = {}

    def once(text, old, new):
        assert text.count(old) == 1, (old, text.count(old))
        return text.replace(old, new)

    def edit(name, transform):
        if name not in composed:
            composed[name] = (UPSTREAM / name).read_text(encoding='utf8')
        composed[name] = transform(composed[name])

    result = runpy.run_path(str(HERE / 'kill-aura-extra-patch.py'))['apply'](edit, once)
    assert result == {'hook_names': [], 'helper_names': []}
    assert len(composed) == 2
    dragon = composed['crates/horizon-client/src/game/dragon.rs']
    mace = composed['crates/horizon-client/src/game/mace.rs']
    assert 'pub(super) fn knockback(&mut self, power: f64' in dragon
    assert 'pub(super) fn mace_smash_self(' in mace
    assert 'pub(super) fn smash_knockback(' in mace
    assert 'const BODY: usize = 2;' in dragon
    candidates = before['kill-aura-extra-candidates.rs'].decode('utf8')
    attacks = before['kill-aura-extra-attack.rs'].decode('utf8')
    assert 'const KILL_AURA_GUEST: u8 = 12;' in candidates
    assert 'const KILL_AURA_DRAGON_PARENT: u8 = 13;' in candidates
    assert 'out.retain(|c| c.id != guest.entity_id)' in candidates
    assert 'kind: KILL_AURA_GUEST' in candidates
    assert 'body.dimensions.bounds(Vec3::new(x, y, z))' in candidates
    assert 'guest_root' in candidates and 'root == Some(guest_root)' in candidates
    assert 'kind: KILL_AURA_DRAGON_PARENT' in candidates
    assert 'local_id: super::dragon::SPECIAL + KILL_AURA_DRAGON_BODY as u32' in candidates
    assert 'self.kill_aura_candidates()' in candidates
    assert 'self.mobs[' not in attacks
    guest = attacks[attacks.index('fn kill_aura_attack_guest'):attacks.index('fn kill_aura_attack_dragon_parent')]
    assert guest.index('self.native_attack_entity') < guest.index('self.pvp_strike()')
    assert guest.index('self.pvp_strike()') < guest.index('self.lan.take()')
    assert guest.index('self.pvp_land(') < guest.index('self.lan = Some(host)') < guest.index('self.pvp_hit_after(')
    assert 'Victim::Guest(conn)' in guest and 'strike.stops_sprint()' in guest
    assert 'melee_damage_with_admission("minecraft:ender_dragon", true)' in attacks
    assert 'self.hit_special(' in attacks and 'c.local_id,' in attacks
    assert 'self.stop_sprint_after_extra_knockback()' in attacks
    assert 'self.native_hurt_enemy(' in attacks and 'self.weapon_wear(&stack)' in attacks
    assert 'self.mace_smash_self(false)' in attacks and 'self.smash_knockback(' in attacks
    assert 'world::read' not in attacks and 'world::write' not in attacks
    # Required API signatures belong to the actual frozen host, not a stub.
    api = {
        'lan_pvp.rs': ['pub(super) fn pvp_strike(', 'pub(super) fn pvp_land(',
                       'pub(super) fn pvp_victim_on_ground(', 'pub(super) fn stops_sprint('],
        'mace.rs': ['pub(super) fn pvp_hit_after(', 'pub(super) fn weapon_wear('],
        'melee.rs': ['pub(super) fn melee_damage_with_admission(', 'pub(super) fn attack_effects('],
        'native_hooks.rs': ['pub(super) fn native_attack_entity(', 'pub(super) fn native_hurt_enemy('],
    }
    for name, signatures in api.items():
        text = (UPSTREAM / 'crates/horizon-client/src/game' / name).read_text(encoding='utf8')
        for signature in signatures:
            assert signature in text, (name, signature)
    with tempfile.TemporaryDirectory(prefix='liquidbounce-kill-aura-extra-static-') as directory:
        files = []
        for name, content in list(composed.items()) + [
                ('candidates.rs', candidates), ('attacks.rs', attacks)]:
            path = Path(directory) / Path(name).name
            path.write_text(content, encoding='utf8')
            files.append(str(path))
        subprocess.run(['rustfmt', '--edition', '2024', '--config', 'skip_children=true', *files], check=True)
    for name, raw in before.items():
        assert (HERE / name).read_bytes() == raw, name
    print('KillAura non-mob static checks passed: guest category/connection PVP route, dragon parent BODY/living route, real frozen API signatures, two native visibility edits in memory, temporary rustfmt; no Rust execution or shared/staging writes')


if __name__ == '__main__':
    main()
