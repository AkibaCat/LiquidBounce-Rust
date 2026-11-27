#!/usr/bin/env python3
"""Record the exact vanilla phase dispatch used by upstream ModuleChams."""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
JAR = Path.home() / '.gradle/caches/fabric-loom/26.3/minecraft-client.jar'
JAVA = ROOT.parent / 'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/javap.exe'
OUT = ROOT / 'build/chams-oit-inspection'


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    classes = [
        'net.minecraft.client.renderer.feature.FeatureRenderDispatcher',
        'net.minecraft.client.renderer.feature.FeatureRenderDispatcher$PreparedFrame',
        'net.minecraft.client.renderer.feature.FeatureRenderDispatcher$PreparedGroup',
        'net.minecraft.client.renderer.feature.RenderTypeFeatureRenderer',
        'net.minecraft.client.renderer.SubmitNodeStorage',
        'net.minecraft.client.renderer.SubmitNodeCollection',
    ]
    records = {}
    for name in classes:
        result = subprocess.run([str(JAVA), '-classpath', str(JAR), '-p', '-c', name],
                                capture_output=True, text=True, encoding='utf-8')
        result.check_returncode()
        path = OUT / (name.rsplit('.', 1)[1] + '.txt')
        path.write_text(result.stdout, encoding='utf-8')
        records[path.name] = hashlib.sha256(path.read_bytes()).hexdigest()
    libraries = list((ROOT / 'build/resource-runtime/libraries').rglob('*.jar'))
    classpath = ';'.join(str(path) for path in [JAR, *libraries])
    javac = JAVA.with_name('javac.exe')
    source = Path(__file__).with_name('ChamsPhaseOracle.java')
    compiled = subprocess.run([str(javac), '-J-Duser.language=en', '-encoding', 'UTF-8', '-classpath', classpath, '-d', str(OUT), str(source)],
                              capture_output=True, text=True, encoding='utf-8', errors='replace')
    (OUT / 'compile.log').write_text(compiled.stdout + compiled.stderr, encoding='utf-8')
    if compiled.returncode:
        raise RuntimeError(compiled.stdout + compiled.stderr)
    oracle = subprocess.run([str(JAVA.with_name('java.exe')), '-Duser.language=en', '-Dfile.encoding=UTF-8', '-classpath', str(OUT) + ';' + classpath, 'ChamsPhaseOracle'],
                            capture_output=True, text=True, encoding='utf-8', errors='replace')
    (OUT / 'oracle.log').write_text(oracle.stdout + oracle.stderr, encoding='utf-8')
    if oracle.returncode:
        raise RuntimeError(oracle.stdout + oracle.stderr)
    print(oracle.stdout)
    report = {'minecraftClientJarSha1': hashlib.sha1(JAR.read_bytes()).hexdigest(),
              'javapSha256': records,
              'upstreamSourceSha256': hashlib.sha256((ROOT / 'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/render/ModuleChams.kt').read_bytes()).hexdigest(),
              'oracle': json.loads(oracle.stdout.strip().splitlines()[-1]),
              'oracleSourceSha256': hashlib.sha256(source.read_bytes()).hexdigest(),
              'scope': 'Official 26.3 bytecode, real renderAllFeatures/PreparedFrame dispatch with counting FeatureRenderer and RenderPass proxies, and pinned ModuleChams. No GPU or in-game pixel assertion.'}
    (OUT / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
