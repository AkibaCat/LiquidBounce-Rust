#!/usr/bin/env python3
"""Actual pinned ItemStack, unchanged Kotlin and exact original mixin body."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
OUT = ROOT/'build/elytra-recast-tests'


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    cache = Path(os.environ['USERPROFILE'])/'.gradle/caches'
    minecraft = cache/'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar'
    libraries = [str(minecraft)]
    for group in ['com.mojang', 'com.google.guava', 'com.google.code.gson', 'it.unimi.dsi', 'org.slf4j', 'org.joml',
                  'io.netty', 'org.apache.logging.log4j', 'org.apache.commons', 'org.lwjgl']:
        libraries.extend(str(path) for path in (cache/'modules-2/files-2.1'/group).rglob('*.jar'))
    jdk = ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin'
    classes = OUT/'java-classes'; classes.mkdir(exist_ok=True)
    oracle = HERE/'oracles/ElytraRecastFacts.java'
    for name, command in [
        ('java-compile', [str(jdk/'javac.exe'), '-classpath', os.pathsep.join(libraries), '-d', str(classes), str(oracle)]),
        ('java-oracle', [str(jdk/'java.exe'), '-classpath', os.pathsep.join([str(classes), *libraries]), 'ElytraRecastFacts', str(OUT/'oracle.json'), str(ROOT/'build/resources/26.3/items.json')]),
    ]:
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, encoding='utf-8', errors='replace')
        (OUT/(name+'.log')).write_text(result.stdout+result.stderr, encoding='utf-8')
        if result.returncode: raise SystemExit(result.stdout+result.stderr)
    facts = json.loads((OUT/'oracle.json').read_text(encoding='utf-8'))
    facts['minecraftSha1'] = hashlib.sha1(minecraft.read_bytes()).hexdigest()
    facts['preparedItemsSha256'] = hashlib.sha256((ROOT/'build/resources/26.3/items.json').read_bytes()).hexdigest()
    originals = ['features/module/modules/movement/ModuleElytraRecast.kt', 'utils/navigation/LocalPlayerBody.kt']
    base = ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce'
    paths = [base/path for path in originals]
    paths.append(ROOT/'upstream/LiquidBounce/src/main/java/net/ccbluex/liquidbounce/injection/mixins/minecraft/entity/MixinLivingEntity.java')
    facts['sourceSha256'] = {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest() for path in [oracle, *paths]}
    (OUT/'oracle.json').write_text(json.dumps(facts, indent=2)+'\n', encoding='utf-8')
    result=subprocess.run([sys.executable,str(HERE/'collect-elytra-recast-original.py')],cwd=ROOT)
    if result.returncode:raise SystemExit(result.returncode)


if __name__ == '__main__': main()
