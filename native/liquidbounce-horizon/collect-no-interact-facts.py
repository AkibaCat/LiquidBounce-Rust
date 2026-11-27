#!/usr/bin/env python3
"""Read NoInteract's exact registry lookup, tags and item classes from MC 26.3."""
import hashlib
import json
import os
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
OUT = ROOT / 'build/no-interact-oracle'


def write_item_facts(facts):
    classes = sorted({name for item in facts['items'] for name in item['classes']})
    masks = {name: 1 << i for i, name in enumerate(classes)}
    groups = {}
    for item in facts['items']:
        mask = sum(masks[name] for name in item['classes'])
        if mask:
            groups.setdefault(mask, []).append(item['id'])
    source = '// Generated from actual MC26.3 class hierarchy and four official mining-tool tags.\n'
    source += '// Regenerate with native/liquidbounce-horizon/collect-no-interact-facts.py.\n'
    source += '#[rustfmt::skip]\npub const DEFAULT_HOLDING_ITEMS: &[&str] = &[\n'
    source += ''.join('    ' + json.dumps(name) + ',\n' for name in facts['defaultHoldingItems']) + '];\n'
    source += '#[rustfmt::skip]\npub fn item_is(id: &str, class: &str) -> bool {\n    let bit = match class {\n'
    source += ''.join(f'        {json.dumps(name)} => {mask}u32,\n' for name, mask in masks.items())
    source += '        _ => return false,\n    };\n    let mask = match id {\n'
    for mask, ids in sorted(groups.items()):
        source += '        ' + '\n        | '.join(json.dumps(name) for name in sorted(ids)) + f' => {mask}u32,\n'
    source += '        _ => 0,\n    };\n    mask & bit != 0\n}\n'
    classes = sorted({name for block in facts['blocks'] for name in block['classes']})
    masks = {name: 1 << i for i, name in enumerate(classes)}
    groups = {}
    for block in facts['blocks']:
        mask = sum(masks[name] for name in block['classes'])
        if mask:
            groups.setdefault(mask, []).append(block['id'])
    source += '#[rustfmt::skip]\npub fn block_is(id: &str, class: &str) -> bool {\n    let bit = match class {\n'
    source += ''.join(f'        {json.dumps(name)} => {mask}u64,\n' for name, mask in masks.items())
    source += '        _ => return false,\n    };\n    let mask = match id {\n'
    for mask, ids in sorted(groups.items()):
        source += '        ' + '\n        | '.join(json.dumps(name) for name in sorted(ids)) + f' => {mask}u64,\n'
    source += '        _ => 0,\n    };\n    mask & bit != 0\n}\n'
    (ROOT / 'native/liquidbounce-behaviors/src/no_interact_items.rs').write_text(source, encoding='utf-8')


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    cache = Path(os.environ['USERPROFILE']) / '.gradle/caches'
    minecraft = cache / 'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar'
    libraries = [str(minecraft), str(ROOT.parent / 'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib/kotlin-stdlib-2.4.21.jar')]
    for group in ['com.mojang', 'com.google.guava', 'com.google.code.gson', 'it.unimi.dsi', 'org.slf4j', 'org.joml', 'io.netty', 'org.apache.logging.log4j', 'org.apache.commons']:
        libraries.extend(str(path) for path in (cache / 'modules-2/files-2.1' / group).rglob('*.jar'))
    java = ROOT.parent / 'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    run = subprocess.run([str(java), '--class-path', os.pathsep.join(libraries), str(HERE / 'oracles/NoInteractFacts.java'), str(OUT / 'facts.json')], capture_output=True, text=True)
    (OUT / 'java-oracle.log').write_text(run.stdout + run.stderr, encoding='utf-8')
    if run.returncode:
        raise SystemExit(run.stdout + run.stderr)
    facts = json.loads((OUT / 'facts.json').read_text(encoding='utf-8'))
    write_item_facts(facts)
    provenance = {'minecraftVersion': '26.3', 'minecraftJarSha1': hashlib.sha1(minecraft.read_bytes()).hexdigest(),
                  'source': 'NoInteractFacts.java; actual MC registries/classes/tags + Kotlin ComparisonsKt + Fastutil ObjectRBTreeSet',
                  'entityFilter': facts['entityFilter'], 'defaultHoldingItems': facts['defaultHoldingItems'], 'items': len(facts['items'])}
    (OUT / 'result.json').write_text(json.dumps(provenance, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(provenance, indent=2))


if __name__ == '__main__':
    main()
