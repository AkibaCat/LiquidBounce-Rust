#!/usr/bin/env python3
"""Redirect only the real stdlib MonotonicTimeSource's nanoTime provider.

The unchanged manager and the complete stdlib mark/Duration implementation run
normally. The fixture classpath must place --classes before the stdlib jar.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import zipfile

CLASS = 'kotlin/time/MonotonicTimeSource.class'


def patch(stdlib: Path, classes: Path) -> dict:
    with zipfile.ZipFile(stdlib) as jar:
        original = jar.read(CLASS)
    assert original[:4] == b'\xca\xfe\xba\xbe'
    u2 = lambda at: struct.unpack_from('>H', original, at)[0]
    u4 = lambda at: struct.unpack_from('>I', original, at)[0]
    count = u2(8)
    cp, offsets, cursor, index = {}, {}, 10, 1
    while index < count:
        offsets[index] = cursor
        tag = original[cursor]
        cursor += 1
        if tag == 1:
            length = u2(cursor)
            # Metadata uses class-file Modified UTF-8. Only ASCII names are
            # inspected; undecodable metadata is retained byte-for-byte.
            cp[index] = (tag, original[cursor + 2:cursor + 2 + length].decode('utf-8', 'surrogateescape'))
            cursor += 2 + length
        elif tag in [3, 4]:
            cp[index] = (tag,)
            cursor += 4
        elif tag in [5, 6]:
            cp[index] = (tag,)
            cursor += 8
            index += 1
        elif tag in [7, 8, 16, 19, 20]:
            cp[index] = (tag, u2(cursor))
            cursor += 2
        elif tag in [9, 10, 11, 12, 17, 18]:
            cp[index] = (tag, u2(cursor), u2(cursor + 2))
            cursor += 4
        elif tag == 15:
            cp[index] = (tag, original[cursor], u2(cursor + 1))
            cursor += 3
        else:
            raise AssertionError(('constant-pool tag', tag))
        index += 1
    cp_end = cursor
    utf = lambda index: cp[index][1]
    refs = []
    for index, value in cp.items():
        if value[0] != 10:
            continue
        owner = utf(cp[value[1]][1])
        name_type = cp[value[2]]
        if (owner, utf(name_type[1]), utf(name_type[2])) == ('java/lang/System', 'nanoTime', '()J'):
            refs.append(index)
    assert len(refs) == 1, refs
    target = refs[0]
    # access, this/super, interfaces
    cursor = cp_end + 6
    cursor += 2 + u2(cursor) * 2

    def skip_attributes(at):
        attributes = []
        length = u2(at)
        at += 2
        for _ in range(length):
            name, size = utf(u2(at)), u4(at + 2)
            attributes.append((name, at + 6, size))
            at += 6 + size
        return at, attributes

    fields = u2(cursor)
    cursor += 2
    for _ in range(fields):
        cursor, _ = skip_attributes(cursor + 6)
    methods = u2(cursor)
    cursor += 2
    calls = []
    operand_sizes = {**{op: 1 for op in [0x10, 0x12, *range(0x15, 0x1a), *range(0x36, 0x3b), 0xa9, 0xbc]},
                     **{op: 2 for op in [0x11, 0x13, 0x14, 0x84, *range(0x99, 0xa9), *range(0xb2, 0xb9), 0xbb, 0xbd, 0xc0, 0xc1, 0xc6, 0xc7]},
                     0xb9: 4, 0xba: 4, 0xc5: 3, 0xc8: 4, 0xc9: 4}
    for _ in range(methods):
        name, descriptor = utf(u2(cursor + 2)), utf(u2(cursor + 4))
        cursor, attributes = skip_attributes(cursor + 6)
        for attribute, at, _ in attributes:
            if attribute != 'Code':
                continue
            code_start, code_size = at + 8, u4(at + 4)
            position = 0
            while position < code_size:
                opcode = original[code_start + position]
                if opcode == 0xb8 and u2(code_start + position + 1) == target:
                    calls.append({'method': name, 'descriptor': descriptor,
                                  'offset': code_start + position, 'instructionOffset': position})
                # The pinned stdlib has no variable-length bytecodes here.
                assert opcode not in [0xaa, 0xab, 0xc4], ('unexpected variable opcode', name, opcode)
                position += 1 + operand_sizes.get(opcode, 0)
            assert position == code_size
    assert [(c['method'], c['descriptor']) for c in calls] == [('read', '()J'), ('<clinit>', '()V')], calls
    owner = b'fixture/Clock'
    additions = b'\x01' + struct.pack('>H', len(owner)) + owner + b'\x07' + struct.pack('>H', count)
    original_cp = bytearray(original[10:cp_end])
    struct.pack_into('>H', original_cp, offsets[target] - 10 + 1, count + 1)
    patched = original[:8] + struct.pack('>H', count + 2) + original_cp + additions + original[cp_end:]
    destination = classes / CLASS
    assert not destination.exists(), destination
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(patched)
    return {'class': CLASS, 'originalSha256': hashlib.sha256(original).hexdigest(),
            'patchedSha256': hashlib.sha256(patched).hexdigest(), 'methodReference': target,
            'originalOwner': 'java/lang/System', 'replacementOwner': 'fixture/Clock',
            'method': 'nanoTime', 'descriptor': '()J', 'calls': calls,
            'constantPoolOwnerOffset': offsets[target] + 1,
            'scope': 'Only one Methodref owner changes; all method bytecodes and all Kotlin mark/Duration arithmetic remain byte-exact.'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--stdlib', required=True, type=Path)
    parser.add_argument('--classes', required=True, type=Path)
    parser.add_argument('--report', required=True, type=Path)
    args = parser.parse_args()
    report = patch(args.stdlib, args.classes)
    args.report.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report))
