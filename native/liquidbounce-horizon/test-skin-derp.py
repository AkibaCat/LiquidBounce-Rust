#!/usr/bin/env python3
"""Complete SkinDerp original-source oracle, whole config and full native API."""
from pathlib import Path
import argparse,hashlib,importlib.util,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent

def load(path,name):
    spec=importlib.util.spec_from_file_location(name,path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--suite',choices=['source','config','native'],default='source');parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite verification evidence')
    out.mkdir(parents=True,exist_ok=True);commands=[];source_files={};rows=None
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target')
    def run(name,argv,cwd=ROOT):
        start=time.monotonic()
        with (out/f'{name}.stdout.log').open('w',encoding='utf-8') as stdout,(out/f'{name}.stderr.log').open('w',encoding='utf-8') as stderr:result=subprocess.run(argv,cwd=cwd,env=env,stdout=stdout,stderr=stderr)
        commands.append({'argv':argv,'exitCode':result.returncode,'durationSeconds':round(time.monotonic()-start,3)});return result.returncode==0
    if args.suite=='source':
        fixture=out/'fixture';shutil.copytree(HERE/'skin-derp-source-fixture',fixture,dirs_exist_ok=True)
        upstream=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce'
        for name,relative in [('ModuleSkinDerp','features/module/modules/fun/ModuleSkinDerp.kt'),('CoroutineTicker','event/CoroutineTicker.kt')]:
            source=upstream/relative;shutil.copyfile(source,fixture/f'{name}.kt');source_files[name]={'sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'unchanged':source.read_bytes()==(fixture/f'{name}.kt').read_bytes()}
        commands_text='''actual 85
enabled true
tick
sync true
tick
parts Hat Cape
tick
delay 2
tick
delay 0
sync false
parts Cape RightSleeve LeftPants Hat Jacket RightPants LeftSleeve
actual 33
tick
sync true
parts Hat Cape
actual 64
tick
delay -2147483648
tick
actual 7
tick
enabled false
actual 19
inGame false
enabled true
tick
inGame true
tick
enabled false
enabled true
sync false
parts
delay 0
tick
enabled false
actual 7
inGame false
enabled true
enabled false
inGame true
enabled true
delay 2147483647
tick
tick
enabled false
enabled true
delay -1
tick
tick
delay 20
tick
'''
        commands_text+='tick\n'*21
        tags=['Hat','Jacket','LeftPants','RightPants','LeftSleeve','RightSleeve','Cape']
        commands_text+='delay 0\n'
        for sync in ['false','true']:
            commands_text+=f'sync {sync}\n'
            for mask in range(128):
                commands_text+='parts'+''.join(' '+tags[index] for index in reversed(range(7)) if mask&(1<<index))+'\n'
                commands_text+=f'actual {(mask*39)%128}\ntick\n'
        (out/'commands.txt').write_text(commands_text,encoding='utf-8')
        java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib'
        jars=[next(kotlin.glob(name)) for name in ['kotlin-stdlib-*.jar','kotlinx-coroutines-core-jvm-*.jar','slf4j-api-*.jar']];classpath=os.pathsep.join(str(p) for p in jars)
        okay=run('kotlin-compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',classpath,'-d',str(out/'classes')]+[str(p) for p in sorted(fixture.glob('*.kt'))])
        if okay:okay=run('kotlin-source',[str(java),'--add-opens','java.base/java.lang=ALL-UNNAMED','-cp',os.pathsep.join([str(out/'classes'),classpath]),'MainKt',str(out/'commands.txt')])
        if okay:
            data=json.loads((out/'kotlin-source.stdout.log').read_text());rows=len(data['rows']);(out/'source-vectors.json').write_text(json.dumps(data),encoding='utf-8')
            for name in ['skin_derp','random']:shutil.copyfile(ROOT/f'native/liquidbounce-behaviors/src/{name}.rs',out/f'{name}.rs')
            shutil.copyfile(HERE/'skin-derp-source-fixture/oracle.rs',out/'oracle.rs')
            (out/'lib.rs').write_text('pub mod skin_derp;\npub mod random;\n#[cfg(test)] mod oracle;\n',encoding='utf-8')
            okay=run('random',[str(java),'--add-opens','java.base/java.lang=ALL-UNNAMED',str(HERE/'AntiAfkRandomReference.java')])
            shutil.copyfile(out/'random.stdout.log',out/'java-random-vectors.json')
            deps='serde_json="1"\n'
    elif args.suite=='config':
        hand=load(HERE/'test-hand-derp.py','hand_derp_fixture');hand.behaviors(out)
        lib=out/'behaviors/src/lib.rs';text=lib.read_text(encoding='utf-8')
        if 'pub mod skin_derp;' not in text:lib.write_text(text+'\npub mod skin_derp;\n',encoding='utf-8')
        config=(HERE/'engine/src/config.rs').read_text(encoding='utf-8')
        if 'mod skin_derp;' not in config:
            def once(before,after):
                nonlocal config
                if config.count(before)!=1:raise ValueError((before,config.count(before)))
                config=config.replace(before,after,1)
            once('mod hand_derp;','mod hand_derp;\n#[path="config/skin_derp.rs"]\nmod skin_derp;\npub use skin_derp::{SkinDerpSetting,serialize_skin_derp};')
            once('    HandDerp => ("HandDerp", "Fun", "handDerp"),','    HandDerp => ("HandDerp", "Fun", "handDerp"),\n    SkinDerp => ("SkinDerp", "Fun", "skinDerp"),')
            if config.count('    HandDerp(HandDerpSetting),')!=2:raise ValueError('SettingChange/RuntimeRequest anchor')
            config=config.replace('    HandDerp(HandDerpSetting),','    HandDerp(HandDerpSetting),\n    SkinDerp(SkinDerpSetting),')
            once('        ModuleKind::HandDerp => hand_derp::parse_hand_derp(&mut patch, &root),','        ModuleKind::HandDerp => hand_derp::parse_hand_derp(&mut patch, &root),\n        ModuleKind::SkinDerp => skin_derp::parse_skin_derp(&mut patch, &root),')
            once('        SettingChange::HandDerp(value) => {','        SettingChange::SkinDerp(value) => { return ApplyResult::Runtime(RuntimeRequest::SkinDerp(value.clone())); }\n        SettingChange::HandDerp(value) => {')
        (out/'lib.rs').write_text(config,encoding='utf-8');shutil.copytree(HERE/'engine/src/config',out/'config',dirs_exist_ok=True)
        deps='liquidbounce-behaviors={path="behaviors"}\nserde_json={version="1",features=["arbitrary_precision"]}\n';okay=True
        env['CARGO_TARGET_DIR']=str(HERE/'config-tests/target')
    else:
        hand=load(HERE/'test-hand-derp.py','skin_native_base');changes=hand.snapshot_changes()
        path='crates/horizon-client/src/game/native_hooks.rs'
        if 'fn native_client_skin_derp_parts(' not in changes[path][1]:
            patch=load(HERE/'skin-derp-patch.py','skin_derp_patch')
            def once(text,before,after):
                if text.count(before)!=1:raise ValueError((before,text.count(before)))
                return text.replace(before,after,1)
            def edit(path,transform):
                old,new=changes.get(path,('',(ROOT/'upstream/Horizon'/path).read_text(encoding='utf-8') if (ROOT/'upstream/Horizon'/path).exists() else ''));changes[path]=(old,transform(new))
            added=patch.apply(edit,once);fragment='tools/native-hook-fragments/liquidbounce_behavior.json';data=json.loads(changes[fragment][1]);data['helpers'].extend({'name':name,'target':f"horizon_client::game::native_hooks::Game<'_>::native_{name}"} for name in added['helper_names']);changes[fragment]=('',json.dumps(data,indent=2)+'\n')
        stage=out/'horizon';shutil.copytree(ROOT/'upstream/Horizon',stage,dirs_exist_ok=True,ignore=shutil.ignore_patterns('target','.git','node_modules'))
        for path,(_,text) in changes.items():
            destination=stage/path;destination.parent.mkdir(parents=True,exist_ok=True);destination.write_text(text,encoding='utf-8',newline='\n')
        gen=out/'generator';gen.mkdir(exist_ok=True)
        (gen/'Cargo.toml').write_text('[package]\nname="skin-derp-api-gen"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[[bin]]\nname="generate"\npath="main.rs"\n[dependencies]\nhorizon-mod-api-gen={path="../horizon/crates/horizon-mod-api-gen"}\n',encoding='utf-8')
        (gen/'main.rs').write_text('fn main(){horizon_mod_api_gen::fill_hooks(&std::path::PathBuf::from(std::env::args_os().nth(1).unwrap())).unwrap();}\n',encoding='utf-8')
        env['CARGO_TARGET_DIR']=str(ROOT/'build/auto-weapon-native-target')
        okay=run('generate',['cargo','+stable','run','--manifest-path',str(gen/'Cargo.toml'),'--',str(stage)])
        if okay:okay=run('check',['cargo','+stable','check','-p','horizon-client','--lib'],stage)
    if args.suite!='native' and okay:
        (out/'Cargo.toml').write_text(f'[package]\nname="skin-derp-{args.suite}-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\n'+deps,encoding='utf-8')
        okay=run('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')])
        if okay:okay=run('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])
    snapshot={str(path.relative_to(out)):hashlib.sha256(path.read_bytes()).hexdigest() for path in out.rglob('*.rs') if 'target' not in path.parts}
    report={'passed':okay and all(c['exitCode']==0 for c in commands),'suite':args.suite,'commands':commands,'sourceFiles':source_files,'rows':rows,'snapshotSha256':snapshot,'scope':{'source':'Unchanged ModuleSkinDerp+CoroutineTicker with real Kotlin kotlinx.coroutines and JDK25 Random.Default. Actual options and saved enable snapshot plus exact RNG seed compared for every row against production Rust; 128 selections in sync and random modes, lifecycle/world/raw delays. Minimal Minecraft/settings/event stubs.','config':'Complete production config with explicit registration-only SkinDerp shim if absent; 128 part sets, raw Delay, tree metadata, mutable prefix on invalid Gson entry.','native':'Independent full pinned Horizon source with cumulative owned SkinDerp helpers, actual fill_hooks and complete horizon-client --lib check. Shared staging untouched. No full game execution.'}[args.suite]}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'rows':rows,'report':str(out/'report.json')}))
if __name__=='__main__':main()
