#!/usr/bin/env python3
"""Read real MC26.3 Entity.isPickable/getPickRadius overrides from class files."""
import json
import os
from pathlib import Path
import struct
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'build/smart-eat-tests'


class ClassFile:
    def __init__(self, data):
        self.data, self.at = data, 8
        self.cp = [None]
        count = self.u2()
        while len(self.cp) < count:
            tag = self.u1()
            if tag == 1:
                size = self.u2()
                value = self.take(size).decode('utf-8', errors='replace')
            elif tag in (7, 8, 16, 19, 20):
                value = self.u2()
            elif tag in (3, 4):
                value = self.take(4)
            elif tag in (5, 6):
                value = self.take(8)
            elif tag in (9, 10, 11, 12, 17, 18):
                value = (self.u2(), self.u2())
            elif tag == 15:
                value = (self.u1(), self.u2())
            else:
                raise ValueError(f'Unknown constant-pool tag {tag}')
            self.cp.append(value)
            if tag in (5, 6):
                self.cp.append(None)
        self.u2()
        self.name = self.cp[self.cp[self.u2()]]
        super_index = self.u2()
        self.parent = self.cp[self.cp[super_index]] if super_index else None
        self.take(self.u2() * 2)
        self.members()
        self.methods = self.members()

    def take(self, count):
        value = self.data[self.at:self.at + count]
        self.at += count
        return value

    def u1(self):
        return self.take(1)[0]

    def u2(self):
        return struct.unpack('>H', self.take(2))[0]

    def u4(self):
        return struct.unpack('>I', self.take(4))[0]

    def members(self):
        result = []
        for _ in range(self.u2()):
            flags, name, descriptor = self.u2(), self.u2(), self.u2()
            result.append((self.cp[name], self.cp[descriptor], flags))
            for _ in range(self.u2()):
                self.u2()
                self.take(self.u4())
        return result


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    jar = Path(os.environ['USERPROFILE']) / '.gradle/caches/fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar'
    javap = ROOT.parent / 'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/javap.exe'
    classes = {}
    with zipfile.ZipFile(jar) as source:
        for path in source.namelist():
            if path.startswith('net/minecraft/world/entity/') and path.endswith('.class'):
                parsed = ClassFile(source.read(path))
                classes[parsed.name] = parsed
    methods = ['isPickable', 'getPickRadius', 'isSpectator', 'isAlive']
    owners = {method: sorted(name for name, parsed in classes.items() if any(row[0] == method for row in parsed.methods)) for method in methods}
    result = []
    for name in sorted({owner for values in owners.values() for owner in values}):
        dump = subprocess.run([str(javap), '-c', '-p', '--class-path', str(jar), name.replace('/', '.')], capture_output=True, text=True, check=True).stdout
        keep = False
        for line in dump.splitlines():
            if line.startswith('  ') and not line.startswith('    ') and line.strip():
                keep = any(f' {method}(' in line for method in methods)
                if keep:
                    result.append(name)
            if keep:
                result.append(line)
    (OUT / 'entity-owners.json').write_text(json.dumps(owners, indent=2) + '\n', encoding='utf-8')
    (OUT / 'entity-pick-bytecode.txt').write_text('\n'.join(result) + '\n', encoding='utf-8')
    print(json.dumps({'classes': len(classes), 'owners': owners}, indent=2))


if __name__ == '__main__':
    main()
