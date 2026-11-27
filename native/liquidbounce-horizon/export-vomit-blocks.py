#!/usr/bin/env python3
"""Export actual MC26.3 BuiltInRegistries.BLOCK order and Block.asItem."""
from pathlib import Path
import argparse, hashlib, json, os, subprocess, time

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
SOURCE = '''import com.google.gson.*;
import java.nio.file.*;
import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.world.flag.FeatureFlags;
public class VomitBlockRegistry {
  public static void main(String[] args) throws Exception {
    SharedConstants.tryDetectVersion();
    Bootstrap.bootStrap();
    JsonArray result = new JsonArray();
    for (var block : BuiltInRegistries.BLOCK) {
      var row = new JsonObject();
      row.addProperty("id", BuiltInRegistries.BLOCK.getId(block));
      row.addProperty("block", BuiltInRegistries.BLOCK.getKey(block).toString());
      var item = block.asItem();
      row.addProperty("item", BuiltInRegistries.ITEM.getKey(item).toString());
      var flags = new JsonArray();
      FeatureFlags.REGISTRY.toNames(item.requiredFeatures()).stream().map(Object::toString).sorted().forEach(flags::add);
      row.add("flags", flags);
      result.add(row);
    }
    Files.writeString(Path.of(args[0]), new GsonBuilder().setPrettyPrinting().create().toJson(result) + "\\n");
    JsonArray items = new JsonArray();
    for (var item : BuiltInRegistries.ITEM) {
      var row = new JsonObject();
      row.addProperty("item", BuiltInRegistries.ITEM.getKey(item).toString());
      var flags = new JsonArray();
      FeatureFlags.REGISTRY.toNames(item.requiredFeatures()).stream().map(Object::toString).sorted().forEach(flags::add);
      row.add("flags", flags);
      items.add(row);
    }
    Files.writeString(Path.of(args[1]), new GsonBuilder().setPrettyPrinting().create().toJson(items) + "\\n");
  }
}
'''

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', required=True)
    args = parser.parse_args()
    out = (ROOT / args.out).resolve()
    if (out / 'report.json').exists():
        raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True, exist_ok=True)
    source = out / 'VomitBlockRegistry.java'
    source.write_text(SOURCE, encoding='utf-8')
    minecraft = Path('C:/Users/AkibaCat/.gradle/caches/fabric-loom/26.3/minecraft-client.jar')
    java = ROOT.parent / 'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin'
    libraries = sorted((ROOT / 'build/resource-runtime/libraries').rglob('*.jar'))
    classpath = os.pathsep.join(map(str, [minecraft] + libraries))
    commands = []
    for name, command in [
        ('compile', [str(java / 'javac.exe'), '-cp', classpath, '-d', str(out), str(source)]),
        ('export', [str(java / 'java.exe'), '-cp', str(out) + os.pathsep + classpath, 'VomitBlockRegistry', str(out / 'blocks.json'), str(out / 'items.json')]),
    ]:
        start = time.monotonic()
        with (out / (name + '.stdout.log')).open('w', encoding='utf-8') as stdout, (out / (name + '.stderr.log')).open('w', encoding='utf-8') as stderr:
            result = subprocess.run(command, cwd=out, stdout=stdout, stderr=stderr)
        commands.append({'stage': name, 'exitCode': result.returncode, 'seconds': round(time.monotonic() - start, 3)})
        if result.returncode:
            break
    passed = all(command['exitCode'] == 0 for command in commands)
    rows = json.loads((out / 'blocks.json').read_text()) if passed else []
    items = json.loads((out / 'items.json').read_text()) if passed else []
    if passed:
        assert [r['id'] for r in rows] == list(range(len(rows)))
        rust = '// Generated from the pinned official Minecraft 26.3 client.\n'
        rust += 'pub(super) const CLIENT_VOMIT_BLOCKS: &[(&str, &str, &[&str])] = &[\n'
        for row in rows:
            rust += '    (' + json.dumps(row['block']) + ', ' + json.dumps(row['item']) + ', &[' + ', '.join(json.dumps(f) for f in row['flags']) + ']),\n'
        rust += '];\n'
        rust += 'pub(super) const CLIENT_ITEM_FEATURES: &[(&str, &[&str])] = &[\n'
        for row in sorted(items, key=lambda row: row['item']):
            rust += '    (' + json.dumps(row['item']) + ', &[' + ', '.join(json.dumps(f) for f in row['flags']) + ']),\n'
        rust += '];\n'
        (out / 'vomit-block-registry.rs').write_text(rust, encoding='utf-8')
        (HERE / 'vomit-block-registry.rs').write_text(rust, encoding='utf-8')
    report = {'passed': passed, 'commands': commands, 'blocks': len(rows), 'items': len(items), 'airItems': sum(r['item'] == 'minecraft:air' for r in rows), 'minecraftSha256': hashlib.sha256(minecraft.read_bytes()).hexdigest(), 'sourceSha256': hashlib.sha256(source.read_bytes()).hexdigest(), 'scope': 'Official MC26.3 bootstrap and actual block registry iteration, Block.asItem and requiredFeatures for every item; no inferred alphabetical names or state-id ordering.'}
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report))
    raise SystemExit(0 if passed else 1)

if __name__ == '__main__':
    main()
