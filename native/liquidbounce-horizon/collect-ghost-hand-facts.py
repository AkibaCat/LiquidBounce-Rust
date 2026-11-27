#!/usr/bin/env python3
"""Run untouched TweakedMethods.java with actual 26.3 registries/shapes."""
import hashlib
import json
import os
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
OUT = ROOT / 'build/ghost-hand-tests'


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    cache = Path(os.environ['USERPROFILE']) / '.gradle/caches'
    minecraft = cache / 'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar'
    libraries = [str(minecraft)]
    for group in ['com.mojang', 'com.google.guava', 'com.google.code.gson', 'it.unimi.dsi', 'org.slf4j',
                  'org.joml', 'io.netty', 'org.apache.logging.log4j', 'org.apache.commons', 'org.lwjgl']:
        libraries.extend(str(path) for path in (cache / 'modules-2/files-2.1' / group).rglob('*.jar'))
    jdk = ROOT.parent / 'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin'
    classes = OUT / 'java-classes'
    classes.mkdir(exist_ok=True)
    original = ROOT / 'upstream/LiquidBounce/src/main/java/net/ccbluex/liquidbounce/common/TweakedMethods.java'
    commands = [
        [str(jdk / 'javac.exe'), '-classpath', os.pathsep.join(libraries), '-d', str(classes),
         str(original), str(HERE / 'oracles/ghost-hand/ModuleGhostHand.java'), str(HERE / 'oracles/GhostHandFacts.java')],
        [str(jdk / 'java.exe'), '-classpath', os.pathsep.join([str(classes), *libraries]), 'GhostHandFacts', str(OUT / 'oracle.json')],
    ]
    for name, command in zip(['java-compile', 'java-oracle'], commands):
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, encoding='utf-8', errors='replace')
        (OUT / (name + '.log')).write_text(result.stdout + result.stderr, encoding='utf-8')
        if result.returncode:
            raise SystemExit(result.stdout + result.stderr)
    facts = json.loads((OUT / 'oracle.json').read_text(encoding='utf-8'))
    facts['minecraftSha1'] = hashlib.sha1(minecraft.read_bytes()).hexdigest()
    facts['originalTweakedMethodsSha256'] = hashlib.sha256(original.read_bytes()).hexdigest()
    (OUT / 'oracle.json').write_text(json.dumps(facts, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'defaults': len(facts['defaults']), 'states': len(facts['states']), 'cases': len(facts['cases']),
                      'originalTweakedMethodsSha256': facts['originalTweakedMethodsSha256']}))


if __name__ == '__main__':
    main()
