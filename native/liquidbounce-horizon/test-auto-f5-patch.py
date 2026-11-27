#!/usr/bin/env python3
"""Compile actual AutoF5 screen/title helpers with the real generated Host API."""
import hashlib
import json
import os
from pathlib import Path
import re
import runpy
import subprocess
import zipfile

HERE=Path(__file__).resolve().parent
ROOT=HERE.parent.parent
UPSTREAM=ROOT/'upstream/Horizon'
OUT=ROOT/'build/auto-f5-hook-tests'

def main():
    helper=runpy.run_path(str(HERE/'auto-f5-patch.py'))
    block=runpy.run_path(str(HERE/'test-camera-control-patch.py'))['block']
    changed={}
    def edit(path,transform):
        src=UPSTREAM/path
        changed[path]=transform(changed.get(path,src.read_text(encoding='utf-8') if src.exists() else ''))
    def once(text,before,after):
        if text.count(before)!=1:raise ValueError(f'Expected one anchor: {before!r}')
        return text.replace(before,after,1)
    contract=helper['apply'](edit,once)
    OUT.mkdir(parents=True,exist_ok=True)
    original=(ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/utils/inventory/InventoryValueGroups.kt').read_text(encoding='utf-8')
    enum=original.split('private enum class ContainerTitle(',1)[1].split('fun matches(',1)[0]
    parsed=[]
    for match in re.finditer(r'\b[A-Z_]+\s*\(\s*"([^"]+)"\s*,(.*?)\)',enum,re.S):
        parsed.append((match[1],re.findall(r'"((?:container|entity)\.[^"]+)"',match[2])))
    assert parsed==helper['TITLE_KEYS'], 'AutoF5 title choices differ from the complete pinned Kotlin enum'
    # The original adapter is registered without nullSafe(). Verify Gson's
    # actual wrapper, registry lookup and null-token handling on the game JAR.
    cache=Path(os.environ['USERPROFILE'])/'.gradle/caches'
    libraries=[str(cache/'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar')]
    for group in ['com.mojang','com.google.guava','com.google.code.gson','it.unimi.dsi','org.slf4j','org.joml','io.netty','org.apache.logging.log4j','org.apache.commons']:
        libraries.extend(str(path) for path in (cache/'modules-2/files-2.1'/group).rglob('*.jar'))
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    oracle=subprocess.run([str(java),'--class-path',os.pathsep.join(libraries),str(HERE/'oracles/MenuRegistryAdapterFacts.java'),str(OUT/'menu-registry-adapter.json')],capture_output=True,text=True)
    (OUT/'java-oracle.log').write_text(oracle.stdout+oracle.stderr,encoding='utf-8')
    if oracle.returncode:raise SystemExit(oracle.stdout+oracle.stderr)
    facts=json.loads((OUT/'menu-registry-adapter.json').read_text(encoding='utf-8'))
    cases={case['input']:case for case in facts['cases']}
    assert cases['null']['error']==cases['[null]']['error']=='JsonSyntaxException'
    assert cases['"minecraft:missing"']['null'] and cases['"bad ID"']['null'] and cases['123']['null']
    assert cases['true']['error']=='JsonSyntaxException'
    assert cases['"generic_9x3"']['id']=='minecraft:generic_9x3'
    assert len(facts['menuIds'])==25
    (OUT/'game').mkdir(exist_ok=True)
    (OUT/'game/auto_f5.rs').write_text(helper['title_source'](),encoding='utf-8')
    (OUT/'behavior.rs').write_text((ROOT/'native/liquidbounce-behaviors/src/auto_f5.rs').read_text(encoding='utf-8'),encoding='utf-8')
    # These are the actual Minecraft 26.3 language resources recovered by the
    # parent's asset pipeline, not invented test translations.
    dictionaries={}
    for name in ['en_us','zh_cn','zh_tw']:
        resource=ROOT/'build/resources/26.3/assets/minecraft/lang'/f'{name}.json'
        if resource.exists():dictionaries[name]=json.loads(resource.read_text(encoding='utf-8'))
        elif name=='en_us':
            with zipfile.ZipFile(Path('C:/Users/AkibaCat/.gradle/caches/fabric-loom/26.3/minecraft-client.jar')) as archive:
                dictionaries[name]=json.loads(archive.read('assets/minecraft/lang/en_us.json'))
        else:raise FileNotFoundError(resource)
    dictionary_source=''
    keys={key for _,keys in helper['TITLE_KEYS'] for key in keys}|{'container.crafting','entity.minecraft.horse','container.enchant','container.repair',
        'container.grindstone_title','container.upgrade','container.stonecutter','merchant.trades','container.cartography_table','container.loom'}
    for constant,name in [('EN','en_us'),('CN','zh_cn'),('TW','zh_tw')]:
        dictionary_source+='const '+constant+':&[(&str,&str)]=&[\n'+''.join('('+json.dumps(key,ensure_ascii=False)+','+json.dumps(dictionaries[name].get(key,key),ensure_ascii=False)+'),\n' for key in sorted(keys))+'];\n'
    source=(HERE/'auto-f5-fixture.rs').read_text(encoding='utf-8')
    source=source.replace('__LANGUAGE_DICTIONARIES__',dictionary_source).replace('__NATIVE_HELPERS__',helper['NATIVE'])
    source=source.replace('__STORAGE_HELPER__',helper['STORAGE'])
    source=source.replace('__CHEST_TITLE__',block((UPSTREAM/'crates/horizon-client/src/game/storage.rs').read_text(encoding='utf-8'),'fn chest_title('))
    (OUT/'lib.rs').write_text(source,encoding='utf-8')
    api=OUT/'api-source';game=api/'crates/horizon-client/src/game';game.mkdir(parents=True,exist_ok=True)
    (api/'tools/native-hook-fragments').mkdir(parents=True,exist_ok=True)
    (api/'crates/horizon-client/src/mod_api.rs').write_text('',encoding='utf-8')
    (game/'native_hooks.rs').write_text("impl Game<'_>{"+helper['NATIVE']+'''fn native_fixture_marker(&self){mod_api::dispatch_fixture_marker(self,mod_api::mods());}}
''',encoding='utf-8')
    render=api/'crates/horizon-client-render/src/gfx/native_render/generated.rs';render.parent.mkdir(parents=True,exist_ok=True)
    render.write_text('use super::*;pub trait RenderHooks:Send+Sync{}',encoding='utf-8')
    (api/'native-hooks.json').write_text(json.dumps({'helpers':[{'name':'auto_f5_screen',
        'target':"horizon_client::game::native_hooks::Game<'_>::native_auto_f5_screen"}],
        'hooks':[{'name':'fixture_marker','target':"horizon_client::game::native_hooks::Game<'_>::native_fixture_marker",'mod_api':{'contract':'listen'}}]},indent=2)+'\n',encoding='utf-8')
    (OUT/'build.rs').write_text('''fn main(){
        let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");
        let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
        horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();
    }''',encoding='utf-8')
    (OUT/'Cargo.toml').write_text('''[package]
name="auto-f5-hook-tests"
version="0.0.0"
edition="2024"
[lib]
path="lib.rs"
[build-dependencies]
horizon-mod-api-gen={path="'''+(UPSTREAM/'crates/horizon-mod-api-gen').as_posix()+'"}\n',encoding='utf-8')
    result=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(OUT/'Cargo.toml'),'--','--test-threads=1'],cwd=ROOT,capture_output=True,text=True)
    (OUT/'test.log').write_text(result.stdout+result.stderr,encoding='utf-8');print(result.stdout+result.stderr)
    (OUT/'result.json').write_text(json.dumps({'hookContract':contract,
        'sourceFiles':{p:hashlib.sha256(t.encode()).hexdigest() for p,t in sorted(changed.items())},
        'testedProductionBodies':['native_auto_f5_screen','auto_f5_menu_title','auto_f5_storage_menu','original chest_title','complete original current-language title key matching'],
        'apiGenerator':'horizon_mod_api_gen::generate(Target::Client); actual generated Host trait and implementation compiled',
        'sourceEnumEquality':'all 16 title choices and every translation key match the complete pinned Kotlin enum',
        'registryOracle':'actual 26.3 Menu registry + Gson wrapper; source-derived IdentifierWithRegistryAdapter read/write bodies',
        'boundaries':['game screen/menu/storage state shell','current-language translator with actual 26.3 language resources'],
        'fixtureResult':'passed' if result.returncode==0 else 'failed','fullGameIntegrationPassed':False},indent=2)+'\n',encoding='utf-8')
    raise SystemExit(result.returncode)

if __name__=='__main__':main()
