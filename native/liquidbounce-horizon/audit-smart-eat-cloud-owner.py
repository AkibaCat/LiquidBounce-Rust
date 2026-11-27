#!/usr/bin/env python3
"""Prove the original 26.3 ClientLevel cloud owner is not network-synchronized.

This deliberately does not infer an owner from particles, potion content or
Horizon Breath.from_dragon. Server NBT ownership and client ownership differ.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import runpy
import subprocess
import zipfile

ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/'build/smart-eat-tests'

def methods(text):
    result=[]
    for line in text.splitlines():
        if line.startswith('  ') and not line.startswith('    ') and '(' in line:
            result.append([line])
        elif result:
            result[-1].append(line)
    return ['\n'.join(block) for block in result]

def one(text,name):
    blocks=[block for block in methods(text) if name+'(' in block.splitlines()[0]]
    if name=='getOwner':
        blocks=[block for block in blocks if 'LivingEntity getOwner(' in block.splitlines()[0]]
    assert len(blocks)==1,(name,len(blocks))
    return blocks[0]

def main():
    OUT.mkdir(parents=True,exist_ok=True)
    jar=Path(os.environ['USERPROFILE'])/'.gradle/caches/fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar'
    javap=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/javap.exe'
    names=['net.minecraft.world.entity.AreaEffectCloud','net.minecraft.world.entity.Entity','net.minecraft.client.multiplayer.ClientPacketListener','net.minecraft.world.item.BottleItem']
    dumps={}
    for name in names:
        dumps[name]=subprocess.run([str(javap),'-c','-p','--class-path',str(jar),name],capture_output=True,text=True,check=True).stdout
    cloud,entity,client,bottle=(dumps[name] for name in names)
    constructors=[b for b in methods(cloud) if ' AreaEffectCloud(' in b.splitlines()[0] or ' net.minecraft.world.entity.AreaEffectCloud(' in b.splitlines()[0]]
    assert len(constructors)==2
    assert all('Field owner:' not in b for b in constructors)
    writers=[b.splitlines()[0].strip() for b in methods(cloud) if re.search(r'putfield\s+#[0-9]+\s+// Field owner:',b)]
    assert len(writers)==2 and any(' setOwner(' in b for b in writers) and any(' readAdditionalSaveData(' in b for b in writers)
    synced=sorted(set(re.findall(r'// Field (DATA_[A-Z_]+):',one(cloud,'defineSynchedData'))))
    assert synced==['DATA_PARTICLE','DATA_RADIUS','DATA_WAITING']
    assert 'recreateFromPacket(' not in cloud
    recreate=one(entity,'recreateFromPacket')
    assert 'getData:' not in recreate and 'readAdditionalSaveData:' not in recreate and 'load:' not in recreate
    add=one(client,'handleAddEntity')
    create=one(client,'createEntityFromPacket')
    assert 'createEntityFromPacket:' in add and 'Entity.recreateFromPacket:' in add and 'ClientLevel.addEntity:' in add
    assert 'EntityType.create:' in create and 'AreaEffectCloud' not in create and 'load:' not in create
    predicate=one(bottle,'lambda$use$0')
    assert 'AreaEffectCloud.getOwner:' in predicate and 'entity/boss/enderdragon/EnderDragon' in predicate
    # No native client package class directly sets or deserializes a cloud's
    # owner. Resolve constant-pool member references, rather than text grep.
    ClassFile=runpy.run_path(str(ROOT/'native/liquidbounce-horizon/audit-smart-eat-entities.py'))['ClassFile']
    setters=[]
    with zipfile.ZipFile(jar) as source:
        for path in source.namelist():
            if not path.startswith('net/minecraft/client/') or not path.endswith('.class'):continue
            parsed=ClassFile(source.read(path))
            for value in parsed.cp:
                if not isinstance(value,tuple) or len(value)!=2:continue
                cls,member=value
                if not isinstance(cls,int) or not isinstance(member,int):continue
                if cls>=len(parsed.cp) or member>=len(parsed.cp):continue
                class_ref,member_ref=parsed.cp[cls],parsed.cp[member]
                if not isinstance(class_ref,int) or not isinstance(member_ref,tuple):continue
                if parsed.cp[class_ref]!='net/minecraft/world/entity/AreaEffectCloud':continue
                method=parsed.cp[member_ref[0]]
                if method in ('setOwner','readAdditionalSaveData'):setters.append((path,method))
    assert not setters,setters
    evidence='\n\n'.join(constructors+[one(cloud,'defineSynchedData'),one(cloud,'setOwner'),one(cloud,'getOwner'),recreate,create,add,one(bottle,'use'),predicate])
    (OUT/'cloud-client-owner-bytecode.txt').write_text(evidence+'\n',encoding='utf-8')
    result={'minecraftSha1':hashlib.sha1(jar.read_bytes()).hexdigest(),'constructorOwnerWrites':0,'ownerWriters':writers,'synchedFields':synced,'cloudRecreateOverride':False,'baseRecreateReadsPacketDataOrNbt':False,'clientCloudOwnerSetterReferences':setters,'conclusion':'Vanilla ClientLevel cloud owners remain null, including the client copy in integrated play. BottleItem client prediction cannot collect dragon breath through its owner branch. The authoritative server still can; its owner must not be substituted into the client result.','fullClientDifferential':'not_run'}
    (OUT/'cloud-client-owner-proof.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(result,indent=2))

if __name__=='__main__':main()
