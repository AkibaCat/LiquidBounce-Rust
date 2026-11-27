#!/usr/bin/env python3
"""Run the unchanged pinned MC26.3 ClientLevel scheduler with observable entities."""
import hashlib
import itertools
import json
import os
from pathlib import Path
import subprocess
import sys

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
OUT=ROOT/'build/terrain-speed-tests/scheduler'

def inputs():
    rows=[]
    base=dict(count=5,frozen=False,insertion=[0,1,2,3,4],passengers=[],removed=[],actions=[])
    for insertion in itertools.permutations(range(4)):
        for passengers in [[],[[1,0]],[[1,3,0],[3,2]],[[0,2,1],[2,3]],[[4,0]],[[1,4],[4,0]]]:
            for frozen in [False,True]:
                rows.append(base|dict(insertion=[*insertion,4],passengers=passengers,frozen=frozen))
    for removed in [[0],[1],[2],[4],[1,4]]:
        rows.append(base|dict(passengers=[[1,0],[0,2]],removed=removed))
    for kind in ['remove','detach','add']:
        for turn,target in [(0,3),(1,0),(1,2),(2,0)]:
            rows.append(base|dict(insertion=[0,1,2,4],passengers=[[1,0,2]],actions=[dict(turn=turn,target=target,kind=kind)]))
    return rows

def main():
    sys.stdout.reconfigure(encoding='utf-8')
    OUT.mkdir(parents=True,exist_ok=True);classes=OUT/'classes';classes.mkdir(exist_ok=True)
    cache=Path(os.environ['USERPROFILE'])/'.gradle/caches'
    minecraft=cache/'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar'
    jars=[minecraft]
    for group in ['com.mojang','com.google.guava','com.google.code.gson','it.unimi.dsi','org.slf4j','org.joml','io.netty','org.apache.logging.log4j','org.apache.commons','org.lwjgl']:
        jars.extend((cache/'modules-2/files-2.1'/group).rglob('*.jar'))
    classpath=os.pathsep.join(map(str,[classes,*jars]));jdk=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin'
    source=HERE/'oracles/terrain-speed/SchedulerOracle.java'
    (OUT/'inputs.json').write_text(json.dumps(inputs()),encoding='utf-8')
    commands=[]
    for name,argv in [('compile',[str(jdk/'javac.exe'),'-J-Dfile.encoding=UTF-8','-encoding','UTF-8','-cp',classpath,'-d',str(classes),str(source)]),('run',[str(jdk/'java.exe'),'-Dfile.encoding=UTF-8','-cp',classpath,'fixture.SchedulerOracle',str(OUT/'inputs.json'),str(OUT/'oracle.json'),str(OUT/'pushes.json')])]:
        r=subprocess.run(argv,cwd=ROOT,capture_output=True,text=True,encoding='utf-8',errors='replace');(OUT/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr,end='');commands.append(dict(name=name,exitCode=r.returncode))
        if r.returncode:raise SystemExit(r.returncode)
    report=dict(commands=commands,cases=len(inputs()),pushCases=144,minecraftSha256=hashlib.sha256(minecraft.read_bytes()).hexdigest(),sourceSha256=hashlib.sha256(source.read_bytes()).hexdigest(),oracleSha256=hashlib.sha256((OUT/'oracle.json').read_bytes()).hexdigest(),pushesSha256=hashlib.sha256((OUT/'pushes.json').read_bytes()).hexdigest())
    (OUT/'result.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')

if __name__=='__main__':main()
