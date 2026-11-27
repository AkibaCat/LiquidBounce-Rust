#!/usr/bin/env python3
"""Exact ResetVL runtime/Host consumers with explicit State and Game providers."""
from pathlib import Path
import argparse,hashlib,json,os,runpy,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent

def body(text,signature):
    start=text.index(signature);end=text.index('{',start)+1;depth=1
    while depth:depth+=(text[end]=='{')-(text[end]=='}');end+=1
    return text[start:end]

def main():
    p=argparse.ArgumentParser();p.add_argument('--out',required=True);p.add_argument('--native',default='build/gui-closer-current-03');a=p.parse_args();out=ROOT/a.out
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True,exist_ok=True)
    host=(HERE/'reset-vl-host.rs').read_text(encoding='utf8');native=(ROOT/a.native/'horizon/crates/horizon-client/src/game/native_hooks.rs').read_text(encoding='utf8')
    changes={}
    def once(text,old,new):
        assert text.count(old)==1,(old,text.count(old));return text.replace(old,new,1)
    def edit(path,change):changes[path]=change(native)
    contract=runpy.run_path(str(HERE/'reset-vl-patch.py'))['apply'](edit,once)
    assert contract=={'hook_names':[],'helper_names':['client_reset_vl_ground','client_reset_vl_velocity']}
    assert host in changes['crates/horizon-client/src/game/native_hooks.rs']
    (out/'host.rs').write_text(host,encoding='utf8')
    (out/'native.rs').write_text((HERE/'reset-vl-native-fixture.rs').read_text(encoding='utf8').replace('/*HOST*/',host),encoding='utf8')
    ticker=(HERE/'engine/src/coroutine_ticker.rs').read_text(encoding='utf8');assert ticker.count('    InventoryManager,')==1
    (out/'coroutine-ticker.rs').write_text(ticker if '    ResetVl,'in ticker else ticker.replace('    InventoryManager,','    InventoryManager,\n    ResetVl,'),encoding='utf8')
    service=(HERE/'engine/src/timer_runtime.rs').read_text(encoding='utf8');service=service[service.index('use liquidbounce_behaviors::timer'):service.index('impl PacketListener for TimerService')]
    service=service.replace('{Context, Mode, ModuleTimer, Provider, Settings, Timer}', '{Context, ModuleTimer, Provider, Settings, Timer}') # Mode is used only by omitted PacketListener.
    (out/'timer-service.rs').write_text(service,encoding='utf8')
    runtime=(HERE/'engine/src/reset_vl_runtime.rs').read_text(encoding='utf8')
    runtime+='''\n#[cfg(all(test,feature="webview"))]mod message_tests{#[test]fn localized_success_preserves_original_fallback(){assert_eq!(super::success_message("en_us"),"Successfully reset violations");assert_eq!(super::success_message("zh-CN"),"已成功重置违规计数");assert_eq!(super::success_message("zh_TW"),"已成功重設違規計數");}}\n'''
    (out/'runtime-source.rs').write_text(runtime,encoding='utf8')
    fixture=(HERE/'reset-vl-runtime-fixture.rs').read_text(encoding='utf8').replace('__RESET_VL__',(ROOT/'native/liquidbounce-behaviors/src/reset_vl.rs').as_posix()).replace('__TIMER__',(ROOT/'native/liquidbounce-behaviors/src/timer.rs').as_posix())
    (out/'runtime.rs').write_text(fixture,encoding='utf8')
    (out/'lib.rs').write_text('#![cfg(test)]\n#![allow(dead_code)]\n#[path="native.rs"]mod native;\n#[path="runtime.rs"]mod runtime_consumer;\n',encoding='utf8')
    # Runtime fixture's self aliases and absolute crate paths require it as the lib root.
    (out/'lib.rs').write_text('#![cfg(test)]\n#![allow(dead_code)]\n'+fixture+'\n#[path="native.rs"]mod native;\n',encoding='utf8')
    (out/'Cargo.toml').write_text('[package]\nname="reset-vl-consumption-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[features]\ndefault=["webview"]\nwebview=[]\n[dependencies]\nserde_json="1"\nglam="0.30"\nliquidbounce-core={path="'+(ROOT/'native/liquidbounce-core').as_posix()+'"}\n',encoding='utf8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');commands=[]
    for name,argv in [('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings']),('no-webview',['cargo','+stable','test','--offline','--no-default-features','--manifest-path',str(out/'Cargo.toml')])]:
        start=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf8');commands.append({'name':name,'exitCode':r.returncode,'seconds':round(time.monotonic()-start,3)})
        if r.returncode:break
    localization=out/'localization';localization.mkdir(exist_ok=True)
    for code in ['zh_cn','zh_tw']:shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{code}.json',localization/f'display_{code}.json')
    subprocess.run(['python',str(HERE/'update-reset-vl-locales.py'),'--directory',str(localization)],cwd=ROOT,check=True)
    for code in ['zh_cn','zh_tw']:
        data=json.loads((localization/f'display_{code}.json').read_text(encoding='utf8'))
        for suffix in ['description','times.description','timer.description']:assert data['descriptions']['liquidbounce.module.resetVL.'+suffix]
    report={'passed':len(commands)==3 and all(c['exitCode']==0 for c in commands),'commands':commands,'hostContract':contract,
        'scope':'Exact ResetVL runtime and native helper bodies; actual core registry and CancelSequences, TimerService implementation, shared CoroutineTicker body and actual glam DVec3. Explicit State/registration, Host, Game/player providers. Webview/no-webview and localization copies only; no complete concrete Game/engine or game claim.',
        'productionSha256':{path.relative_to(ROOT).as_posix():hashlib.sha256(path.read_bytes()).hexdigest() for path in [HERE/'engine/src/reset_vl_runtime.rs',HERE/'reset-vl-host.rs',HERE/'reset-vl-patch.py',HERE/'engine/src/timer_runtime.rs',HERE/'engine/src/coroutine_ticker.rs']}}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}));raise SystemExit(0 if report['passed'] else 1)

if __name__=='__main__':main()
