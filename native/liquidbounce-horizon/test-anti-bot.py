#!/usr/bin/env python3
"""One combined AntiBot smoke; all shared/native/localization integrations stay in memory."""
from pathlib import Path
import ast
import json
import runpy
import subprocess
import tempfile
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]


def main():
    texts={}
    staged=ROOT/'build/staging/horizon'
    def edit(name,transform):
        if name not in texts:
            path=staged/name
            texts[name]=path.read_text(encoding='utf8')if path.exists()else''
        texts[name]=transform(texts[name])
    def once(text,old,new):
        if text.count(old)!=1:
            raise RuntimeError(f'AntiBot native anchor count {text.count(old)}: {old[:100]}')
        return text.replace(old,new)
    fragment=runpy.run_path(str(HERE/'anti-bot-patch.py'))['apply'](edit,once)
    assert fragment=={'hook_names':['client_anti_bot'],'helper_names':['client_anti_bot_snapshot']}
    hooks=texts['crates/horizon-client/src/game/native_hooks.rs']
    assert hooks.count('fn native_client_anti_bot(')==1 and hooks.count('fn native_client_anti_bot_snapshot(')==1
    assert '"antiBot":self.client_anti_bot_query(entity)'in hooks
    net=texts['crates/horizon-client/src/net.rs']
    assert net.index('send.send(Event::AntiBotPacket(')<net.index('        if cancelled {continue;}')
    remote=texts['crates/horizon-client/src/game/remote.rs']
    assert 'or_insert((e.properties.is_empty(),Some(0)))'in remote
    assert 'propertiesEmpty' in remote and 'report_helper_player_order'in remote and 'object_identity()'in texts['crates/horizon-client/src/game/anti_bot_native.rs']
    assert 'self.client_anti_bot.scales.insert(id,scale);'in texts['crates/horizon-client/src/game/remote_attributes.rs']
    plan=runpy.run_path(str(HERE/'integrate-anti-bot.py'))['build_plan']()
    runtime=plan.text['native/liquidbounce-horizon/engine/src/runtime.rs']
    assert runtime.index('self.anti_bot_tick(host);')<runtime.index('self.notifier_tick(host);')
    assert 'WaitOwner::AntiBotMatrix=>self.anti_bot_callback' in runtime and 'if let Some(q)=&snapshot.anti_bot' in runtime and 'info.ignore();' in runtime
    loc=runpy.run_path(str(ROOT/'scripts/update-anti-bot-localization.py'))
    for locale,index in [('zh_cn',1),('zh_tw',2)]:
        data=loc['update'](json.loads((loc['FOLDER']/f'display_{locale}.json').read_text(encoding='utf8')),index)
        assert all(data['descriptions'][loc['description_key'](path)] for path in loc['TEXTS'])
        assert all(data['labels'][tag] for tag in loc['LABELS'])
    with tempfile.TemporaryDirectory(prefix='liquidbounce-antibot-')as temporary:
        temporary=Path(temporary)
        model=(HERE/'anti-bot-smoke.rs').read_text(encoding='utf8').replace('../liquidbounce-behaviors/src/',(ROOT/'native/liquidbounce-behaviors/src').as_posix()+'/')
        source='extern crate self as liquidbounce_behaviors;\n'+model+'\nuse serde_json::{Value,Number,Map,json};\nuse std::{fmt,collections::BTreeMap};\n'
        shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
        start=shared.index('#[derive(Clone, Copy, Debug, PartialEq, Eq)]\npub enum JsonFormat')
        end=shared.index('#[derive(Clone, Debug, PartialEq)]\npub enum SettingChange',start)
        source+=shared[start:end]
        source+='enum ModuleKind{AntiBot} impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.antiBot"}}\n'
        source+='#[derive(Default)]struct SettingsPatch{changes:Vec<SettingChange>,issues:Vec<ConfigError>} enum SettingChange{AntiBot(anti_bot_config::AntiBotSetting)}\n'
        source+=shared[shared.index('struct Tree<'):shared.index('fn float_range(')]
        source+=shared[shared.index("type Nodes<'a>"):shared.index('pub fn parse_settings_text(')]
        source+='#[path="'+(HERE/'engine/src/config/anti_bot.rs').as_posix()+'"]mod anti_bot_config;\n'
        source+=(HERE/'anti-bot-config-smoke.rs').read_text(encoding='utf8')
        fixture=temporary/'anti-bot-smoke.rs';fixture.write_text(source,encoding='utf8');exe=temporary/'anti-bot-smoke.exe'
        candidates=list(ROOT.glob('build/**/libserde_json-*.rlib'))
        dependency=max(candidates,key=lambda p:p.stat().st_mtime_ns)
        subprocess.run(['rustc','--edition=2024','--test',str(fixture),'-L','dependency='+str(dependency.parent),'--extern','serde_json='+str(dependency),'-o',str(exe)],check=True)
        subprocess.run([str(exe),'anti_bot_smoke','--nocapture'],check=True)
    for name,raw in plan.original.items():
        assert(ROOT/name).read_bytes()==raw,'Shared file changed during smoke: '+name
    for path in [HERE/'anti-bot-patch.py',HERE/'integrate-anti-bot.py',ROOT/'scripts/update-anti-bot-localization.py',Path(__file__)]:
        ast.parse(path.read_text(encoding='utf8'))
    print(f'AntiBot checked: 11 focused model/config tests; {len(texts)} native files in memory; {len(plan.steps)} shared changes/{len(plan.text)} files; {len(loc["TEXTS"])} descriptions; no shared/staging writes')


if __name__=='__main__':
    main()
