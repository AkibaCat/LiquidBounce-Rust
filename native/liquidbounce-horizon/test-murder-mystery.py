#!/usr/bin/env python3
"""One focused three-mode/OCR smoke and in-memory native integration check."""
from pathlib import Path
import ast,runpy,subprocess,tempfile
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
    texts={};staged=ROOT/'build/staging/horizon'
    def edit(name,f):
        if name not in texts:texts[name]=(staged/name).read_text(encoding='utf8').replace('\r\n','\n')
        texts[name]=f(texts[name])
    def once(s,old,new):
        if s.count(old)!=1:raise RuntimeError(f'MurderMystery native anchor count {s.count(old)}: {old[:120]}')
        return s.replace(old,new)
    fragment=runpy.run_path(str(HERE/'murder-mystery-patch.py'))['apply'](edit,once)
    assert fragment=={'hook_names':['client_murder_mystery'],'helper_names':[]}
    native=texts['crates/horizon-client/src/game/native_hooks.rs'];net=texts['crates/horizon-client/src/net.rs'];remote=texts['crates/horizon-client/src/game/remote.rs']
    assert native.count('fn native_client_murder_mystery(')==1 and 'SoundInstance::for_ui(sound,1.,0.25'in native
    assert 'team.prefix=serde_json::json!'in native and 'a.equipment[0]'in native and 'self.maps.data.get'in native
    assert net.index('send.send(Event::MurderMysteryPacket')<net.index('if cancelled {continue;}')
    assert 'slot<=1' in remote and 'r.level_players.get(&entity)'in remote and 'minecraft:block.note_block.basedrum'in remote
    plan=runpy.run_path(str(HERE/'integrate-murder-mystery.py'))['build_plan']()
    runtime=plan.text['native/liquidbounce-horizon/engine/src/runtime.rs'];assert 'crate::murder_mystery_runtime::apply_tag'in runtime and 'self.murder_mystery_reset();'in runtime
    with tempfile.TemporaryDirectory(prefix='liquidbounce-murder-mystery-')as temp:
        temp=Path(temp);source=(HERE/'murder-mystery-smoke.rs').read_text(encoding='utf8')
        for name in ['murder_mystery','murder_mystery_font']:source=source.replace('../liquidbounce-behaviors/src/'+name+'.rs',(ROOT/'native/liquidbounce-behaviors/src'/f'{name}.rs').as_posix())
        fixture=temp/'murder-mystery-smoke.rs';fixture.write_text(source,encoding='utf8');exe=temp/'murder-mystery-smoke.exe'
        subprocess.run(['rustc','--edition=2024','--test',str(fixture),'-o',str(exe)],check=True);subprocess.run([str(exe),'--nocapture'],check=True)
    for f in [HERE/'murder-mystery-patch.py',HERE/'integrate-murder-mystery.py',ROOT/'scripts/update-murder-mystery-localization.py']:ast.parse(f.read_text(encoding='utf8'))
    locales=runpy.run_path(str(ROOT/'scripts/update-murder-mystery-localization.py'));assert len(locales['TEXTS'])==6
    print(f'MurderMystery checked: {len(texts)} native files in memory; {len(plan.steps)} shared changes across {len(plan.text)} files; no shared/staging writes')
if __name__=='__main__':main()
