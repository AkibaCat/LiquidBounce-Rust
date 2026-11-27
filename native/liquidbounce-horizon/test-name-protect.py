#!/usr/bin/env python3
"""Read-only NameProtect composition, temporary Rust parse and Chinese locale checks."""
from pathlib import Path
import argparse,ast,json,runpy,shutil,subprocess,tempfile
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--native',action='store_true');args=parser.parse_args()
    for name in ['integrate-name-protect.py','name-protect-patch.py','update-name-protect-locales.py']:
        ast.parse((HERE/name).read_text(encoding='utf8'))
    installed='pub mod name_protect;'in(ROOT/'native/liquidbounce-behaviors/src/lib.rs').read_text(encoding='utf8')
    if installed:
        shared=[ROOT/'native/liquidbounce-behaviors/src/lib.rs',*[HERE/f'engine/src/{name}'for name in ['lib.rs','behaviors.rs','config.rs','actor.rs','runtime.rs','client_commands_runtime.rs']],HERE/'make-behavior-patch.py']
        changes={p:p.read_text(encoding='utf8')for p in shared}
    else:changes=runpy.run_path(str(HERE/'integrate-name-protect.py'))['prepare']().changed
    files={f'shared-{Path(p).name}':s for p,s in changes.items()if p.suffix=='.rs'}
    for p in [ROOT/'native/liquidbounce-behaviors/src/name_protect.rs',ROOT/'native/liquidbounce-behaviors/src/name_protect_username.rs',HERE/'engine/src/name_protect_runtime.rs',HERE/'engine/src/config/name_protect.rs']:
        files['owned-'+p.name]=p.read_text(encoding='utf8')
    if args.native:
        source=(HERE/'make-behavior-patch.py').read_text(encoding='utf8');source=source[:source.index('\noutput = []')]
        if "'name-protect-patch.py'"not in source:source=source.replace("'inventory-tracker-patch.py',","'inventory-tracker-patch.py', 'name-protect-patch.py',")
        context={'__file__':str(HERE/'make-behavior-patch.py'),'__name__':'name_protect_static'}
        exec(compile(source,context['__file__'],'exec'),context)
        assert context['helper_names'].count('client_name_protect_snapshot')==1
        assert context['helper_names'].count('client_name_protect_friend')==1
        for p,(_,s) in context['changes'].items():
            if p in ['crates/horizon-client-core/src/text.rs','crates/horizon-client-render/src/font.rs','crates/horizon-client-render/src/hud.rs','crates/horizon-client/src/game/native_hooks.rs']:
                files['native-'+Path(p).name]=s
        font=files['native-font.rs'];splits=font[font.index('    pub fn split_lines('):]
        assert 'protect_string('not in splits and 'protect_text('not in splits
        assert 'return self.width_text(&Text::legacy(s))'not in font
        hud=files['native-hud.rs'];assert 'shadow, false, None, false)'in hud
    with tempfile.TemporaryDirectory(prefix='liquidbounce-name-protect-static-')as folder:
        temporary=Path(folder);paths=[]
        for n,s in files.items():p=temporary/n;p.write_text(s,encoding='utf8');paths.append(str(p))
        result=subprocess.run(['rustfmt','--edition','2024','--config','skip_children=true',*paths],capture_output=True,text=True)
        if result.returncode:raise RuntimeError(result.stderr)
        locale=temporary/'locales';locale.mkdir()
        for code in ['zh_cn','zh_tw']:shutil.copyfile(HERE.parent/f'liquidbounce-interop/resources/localization/display_{code}.json',locale/f'display_{code}.json')
        update=runpy.run_path(str(HERE/'update-name-protect-locales.py'))['update'];update(locale)
        first={p.name:p.read_bytes()for p in locale.iterdir()};update(locale)
        assert first=={p.name:p.read_bytes()for p in locale.iterdir()}
        for raw in first.values():
            data=json.loads(raw);assert data['moduleNames']['NameProtect'];assert data['labels']['ObfuscateFriends']
            assert data['descriptions']['liquidbounce.module.nameProtect.obfuscateOthers.colorMode.static.color.description']
    runtime=files['owned-name_protect_runtime.rs'];service=runtime[:runtime.index('impl LiquidBounceHooks')]
    assert 'self.state()'not in service and 'host.'not in service
    assert 'text.chars().map(|(c,_)|c)'in runtime
    for name in ['adjectives','animals']:
        assert (ROOT/f'native/liquidbounce-behaviors/src/name_protect_assets/{name}.txt').read_bytes()==(ROOT/f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/data/usernames/{name}.txt').read_bytes()
    print(f'NameProtect static checks passed: {len(changes)} shared files; {len(files)} complete Rust sources parsed in temporary files; native composition and Chinese updater idempotent; no compilation/shared/staging writes')


if __name__=='__main__':main()
