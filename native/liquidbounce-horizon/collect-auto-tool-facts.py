#!/usr/bin/env python3
"""Read actual 26.3 components and menu provider method ownership, read-only."""
import hashlib
import json
import os
from pathlib import Path
import subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parent.parent
OUT=ROOT/'build/auto-tool-tests'

def main():
    OUT.mkdir(parents=True,exist_ok=True)
    cache=Path(os.environ['USERPROFILE'])/'.gradle/caches'
    minecraft=cache/'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar'
    libraries=[str(minecraft)]
    for group in ['com.mojang','com.google.guava','com.google.code.gson','it.unimi.dsi','org.slf4j','org.joml','io.netty','org.apache.logging.log4j','org.apache.commons','org.lwjgl']:
        libraries.extend(str(path) for path in (cache/'modules-2/files-2.1'/group).rglob('*.jar'))
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    result=subprocess.run([str(java),'--class-path',os.pathsep.join(libraries),str(HERE/'oracles/AutoToolFacts.java'),str(OUT/'oracle.json')],capture_output=True,text=True)
    (OUT/'java-oracle.log').write_text(result.stdout+result.stderr,encoding='utf-8')
    if result.returncode:raise SystemExit(result.stdout+result.stderr)
    facts=json.loads((OUT/'oracle.json').read_text())
    unconditional=[];chests=[]
    for block in facts['blocks']:
        owner=block['menuOwner']
        if owner=='ChestBlock':chests.append(block['id'])
        elif owner=='BaseEntityBlock' and block.get('entityIsMenu') or owner not in ['BlockBehaviour','BaseEntityBlock','ChestBlock','LecternBlock']:
            unconditional.append(block['id'])
    source='// Generated from MC26.3 getMenuProvider ownership and BlockEntity MenuProvider implementations.\n'
    source+='match name {\n'+' | '.join(json.dumps(v) for v in sorted(unconditional))+' => true,\n'
    source+=' | '.join(json.dumps(v) for v in sorted(chests))+' => !self.chest_menu_blocked((pos[0],pos[1],pos[2]),name),\n'
    source+='"minecraft:lectern" => self.pack.assets.registry.state(state).and_then(|state|state.properties.get("has_book")).is_some_and(|book|book=="true"),\n_=>false,\n}\n'
    (HERE/'auto-tool-menu-providers.rs').write_text(source,encoding='utf-8')
    print(json.dumps({'minecraftSha1':hashlib.sha1(minecraft.read_bytes()).hexdigest(),'menuOwners':sorted(set(v['menuOwner'] for v in facts['blocks'])),'stacks':facts['stacks']},indent=2))

if __name__=='__main__':main()
