#!/usr/bin/env python3
"""Actual MC26.3 dispatch ownership, without modifying upstream."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
HERE=Path(__file__).resolve().parent
ROOT=HERE.parent.parent
OUT=ROOT/'build/smart-eat-tests'
def main():
    OUT.mkdir(parents=True,exist_ok=True)
    cache=Path(os.environ['USERPROFILE'])/'.gradle/caches'
    minecraft=cache/'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar'
    libraries=[str(minecraft)]
    for group in ['com.mojang','com.google.guava','com.google.code.gson','it.unimi.dsi','org.slf4j','org.joml','io.netty','org.apache.logging.log4j','org.apache.commons','org.lwjgl']:
        libraries.extend(str(path) for path in (cache/'modules-2/files-2.1'/group).rglob('*.jar'))
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    result=subprocess.run([str(java),'--class-path',os.pathsep.join(libraries),str(HERE/'oracles/SmartEatFacts.java'),str(OUT/'oracle.json')],capture_output=True,text=True)
    (OUT/'java-oracle.log').write_text(result.stdout+result.stderr,encoding='utf-8')
    if result.returncode:raise SystemExit(result.stdout+result.stderr)
    facts=json.loads((OUT/'oracle.json').read_text())
    facts['minecraftSha1']=hashlib.sha1(minecraft.read_bytes()).hexdigest()
    (OUT/'oracle.json').write_text(json.dumps(facts,indent=2)+'\n',encoding='utf-8')
    (HERE/'smart-eat-animation-overrides.rs').write_text('// Generated from actual MC26.3 Item.getUseAnimation owners.\n'+' | '.join(json.dumps(row['item']) for row in facts['animationOverrides'])+' => false,\n',encoding='utf-8')
    owners={}
    for row in facts['useOwners']:
        owners.setdefault(row['owner'].rsplit('.',1)[-1],[]).append(row['item'])
    dispatch='// Generated from actual MC26.3 Item.use dispatch owners.\n'
    for owner,items in sorted(owners.items()):
        dispatch+=' | '.join(json.dumps(item) for item in sorted(items))+f' => Some(UseOwner::{owner}),\n'
    dispatch+='_ => None,\n'
    (HERE/'smart-eat-use-owners.rs').write_text(dispatch,encoding='utf-8')
    print(json.dumps({'minecraftSha1':facts['minecraftSha1'],'animationOverrides':len(facts['animationOverrides']),'useOwners':len(facts['useOwners']),'owners':sorted({row['owner'] for row in facts['useOwners']})},indent=2))
if __name__=='__main__':main()
