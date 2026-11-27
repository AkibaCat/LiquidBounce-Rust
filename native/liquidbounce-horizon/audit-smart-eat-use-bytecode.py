#!/usr/bin/env python3
"""Read exact Item.use return branches from the local pinned MC26.3 JAR."""
import json
import os
from pathlib import Path
import subprocess
HERE=Path(__file__).resolve().parent
ROOT=HERE.parent.parent
OUT=ROOT/'build/smart-eat-tests'
def main():
    facts=json.loads((OUT/'oracle.json').read_text())
    jar=Path(os.environ['USERPROFILE'])/'.gradle/caches/fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar'
    javap=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/javap.exe'
    owners=sorted({row['owner'] for row in facts['useOwners']})
    methods=[]
    for owner in owners:
        result=subprocess.run([str(javap),'-c','-p','--class-path',str(jar),'net.minecraft.world.item.'+owner],capture_output=True,text=True,check=True)
        lines=result.stdout.splitlines();keep=False;body=[]
        for line in lines:
            if line.startswith('  ') and not line.startswith('    ') and line.strip():
                if keep:break
                keep=' InteractionResult use(' in line or ' net.minecraft.world.InteractionResult use(' in line
            if keep:body.append(line)
        methods.append(owner+'\n'+'\n'.join(body)+'\n')
    (OUT/'use-bytecode.txt').write_text('\n'.join(methods),encoding='utf-8')
    player=subprocess.run([str(javap),'-c','-p','--class-path',str(jar),'net.minecraft.world.entity.player.Player'],capture_output=True,text=True,check=True).stdout
    lines=player.splitlines();keep=False;body=[]
    for line in lines:
        if line.startswith('  ') and not line.startswith('    ') and line.strip():
            if keep:break
            keep=' boolean canEat(boolean)' in line
        if keep:body.append(line)
    can_eat='\n'.join(body)+'\n'
    assert 'Abilities.invulnerable' in can_eat and 'FoodData.needsFood' in can_eat and 'instabuild' not in can_eat
    (OUT/'can-eat-bytecode.txt').write_text(can_eat,encoding='utf-8')
    print(json.dumps({'owners':owners,'path':str(OUT/'use-bytecode.txt')}))
if __name__=='__main__':main()
