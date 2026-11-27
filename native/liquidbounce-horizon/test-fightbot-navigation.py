#!/usr/bin/env python3
"""FightBot owning Navigator: compose in memory; focused owner tests run in the next batch."""
from pathlib import Path
import argparse,ast,runpy,subprocess,tempfile
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
FILES=['crates/horizon-client/src/game/navigation_owners.rs','crates/horizon-client/src/game.rs','crates/horizon-client/src/game/auto_walk_runtime.rs','crates/horizon-client/src/game/auto_walk_host.rs','crates/horizon-client/src/game/native_hooks.rs','crates/horizon-client/src/game/auto_walk_world.rs']

def static_checks():
    texts={};original={};staged=ROOT/'build/staging/horizon'
    def read(name):
        if name not in texts:
            path=staged/name;original[name]=path.read_bytes()if path.exists()else None
            texts[name]=original[name].decode('utf8').replace('\r\n','\n')if original[name]is not None else''
        return texts[name]
    def edit(name,transform):texts[name]=transform(read(name))
    def once(text,old,new):
        assert text.count(old)==1,(text.count(old),old[:100])
        return text.replace(old,new)
    if 'mod navigation_owners;'not in read(FILES[1]):
        fragment=runpy.run_path(str(HERE/'fightbot-navigation-patch.py'))['apply'](edit,once)
        assert fragment=={'hook_names':[],'helper_names':['client_kill_aura_navigation_config','client_kill_aura_navigation_result']}
    for name in FILES:read(name)
    runtime=texts[FILES[2]];host=texts[FILES[3]];hooks=texts[FILES[4]]
    assert runtime.count('Navigator::new(walking, travel)')==1
    assert 'runtime: Runtime::new(epoch)'in runtime
    assert 'FIGHT_RUNTIME' in runtime and 'FIGHT_EVENTS' in runtime and 'FIGHT_RETIRED' in runtime
    assert 'OwnerScope::enter(Owner::AutoWalk)'in runtime and 'OwnerScope::enter(Owner::FightBot)'in runtime
    assert runtime.count('push(event)')==2
    assert 'runtime.navigator.stop(&mut runtime.host)'in runtime and 'runtime.navigator.halt(&mut runtime.host)'in runtime
    assert 'runtime.runtime.navigator.stop(&mut runtime.runtime.host)'in runtime
    assert 'world::scope_is_active()'in runtime and 'pub(super) fn scope_is_active()->bool'in texts[FILES[5]]
    assert 'g.kill_aura_fight_goal(value)'in runtime and 'if plain && assist_jump'in runtime
    assert all('"'+key+'"'in runtime for key in ['acting','movementRotation','head','previousHead','ahead','points','breaks','places','exactInput'])
    assert 'releasing_destroying()'in host and 'claim_slot(slot as usize,old)'in host and 'release_slot()'in host
    assert host.count('navigation_safe_walk(')==2
    assert hooks.index('let navigation = self.native_client_auto_walk_input')<hooks.index('let fight_navigation = self.native_client_kill_aura_navigation_input')
    assert hooks.index('if let Some((keys,true)) = navigation')<hooks.index('if let Some((_,Some(keys))) = fight_navigation')
    for path in [Path(__file__),HERE/'fightbot-navigation-patch.py']:ast.parse(path.read_text(encoding='utf8'))
    # Parse only temporary copies of the two transformed Rust files; production
    # adapters and staging are never modified by static verification.
    with tempfile.TemporaryDirectory(prefix='liquidbounce-fight-static-')as temporary:
        paths=[]
        for name in FILES[2:4]:
            p=Path(temporary)/Path(name).name;p.write_text(texts[name],encoding='utf8');paths.append(str(p))
        subprocess.run(['rustfmt','--edition','2024','--config','skip_children=true',*paths],check=True)
    for name,raw in original.items():
        p=staged/name;assert(p.read_bytes()if p.exists()else None)==raw,name
    print(f'FightBot navigation static checks passed; {len(texts)} native files in memory; no production/shared/staging writes')

def model_checks():
    with tempfile.TemporaryDirectory(prefix='liquidbounce-fight-')as temporary:
        temporary=Path(temporary);source=(HERE/'fightbot-navigation-smoke.rs').read_text(encoding='utf8').replace('"navigation-owners.rs"','"'+(HERE/'navigation-owners.rs').as_posix()+'"')
        fixture=temporary/'fight-navigation-smoke.rs';fixture.write_text(source,encoding='utf8');exe=temporary/'fight-navigation-smoke.exe'
        subprocess.run(['rustc','--edition=2024','--test',str(fixture),'-o',str(exe)],check=True)
        subprocess.run([str(exe),'fight_navigation_smoke','--nocapture'],check=True)

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    group=parser.add_mutually_exclusive_group();group.add_argument('--static-only',action='store_true');group.add_argument('--model-only',action='store_true')
    args=parser.parse_args()
    if not args.model_only:static_checks()
    if not args.static_only:model_checks()
    else:print('Rust execution deferred to combined batch')
if __name__=='__main__':main()
