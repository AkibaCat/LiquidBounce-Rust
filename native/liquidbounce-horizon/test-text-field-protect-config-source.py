#!/usr/bin/env python3
"""Original ListValue/Gson against the registered complete production configuration."""
from pathlib import Path
import argparse, hashlib, json, os, shutil, subprocess, time
ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent


def main():
    p = argparse.ArgumentParser(); p.add_argument('--out', required=True); a = p.parse_args()
    out = (ROOT / a.out).resolve()
    if (out / 'report.json').exists():
        raise SystemExit('Refusing to overwrite evidence')
    config = (HERE / 'engine/src/config.rs').read_text(encoding='utf8')
    if 'mod text_field_protect;' not in config:
        raise SystemExit('Production registration required; no registration shim')
    out.mkdir(parents=True, exist_ok=True)
    shutil.copytree(ROOT / 'native/liquidbounce-behaviors', out / 'behaviors', ignore=shutil.ignore_patterns('target'), dirs_exist_ok=True)
    shutil.copytree(HERE / 'engine/src/config', out / 'config', dirs_exist_ok=True)
    (out / 'lib.rs').write_text(config + '\n#[cfg(test)] mod source_oracle;\n', encoding='utf8')
    shutil.copyfile(HERE / 'text-field-protect-source-fixture/config_oracle.rs', out / 'source_oracle.rs')
    (out / 'Cargo.toml').write_text('[package]\nname="text-field-protect-config-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nliquidbounce-behaviors={path="behaviors"}\nserde_json={version="1",features=["arbitrary_precision"]}\n', encoding='utf8')
    fixture = out / 'fixture'
    shutil.copytree(HERE / 'gui-closer-config-source-fixture', fixture, dirs_exist_ok=True)
    shutil.copyfile(HERE / 'text-field-protect-source-fixture/ConfigMain.kt.txt', fixture / 'Main.kt')
    original = ROOT / 'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/config'
    names = ['types/list/ListValue.kt', 'gson/adapter/SimpleStringTypeAdapter.kt', 'gson/serializer/ValueGroupSerializer.kt', 'gson/stategies/ExcludeStrategy.kt', 'gson/stategies/ProtocolExcludeStrategy.kt']
    unchanged = {}
    for name in names:
        src = original / name; dst = fixture / src.name
        shutil.copyfile(src, dst)
        assert src.read_bytes() == dst.read_bytes()
        unchanged[name] = hashlib.sha256(dst.read_bytes()).hexdigest()
    inputs = [[], None, '^/login.*', '', True, False, 0, 1.5, {}, [None], [None, None],
              ['a', None, 'a', None], ['a', 'a'], ['ok', '['], ['[', 'ok'], ['ok', True],
              ['ok', {}], [['x']], ['(?i)login', r'\Q['], r'(?<x>a)\k<x>', r'\p{L}+', r'\p{Unknown}', '😀']
    (out / 'inputs.json').write_text(json.dumps(inputs, ensure_ascii=True), encoding='utf8')
    java = ROOT.parent / 'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    libs = ROOT.parent / 'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib'
    stdlib = next(libs.glob('kotlin-stdlib-*.jar')); gson = next(libs.glob('gson-*.jar'))
    classes = out / 'classes'; classes.mkdir(exist_ok=True)
    env = os.environ.copy(); env['CARGO_TARGET_DIR'] = str(ROOT / 'build/text-field-protect-target')
    invocations = [
        ('javac', [str(java.with_name('javac.exe')), '-d', str(classes), *map(str, fixture.glob('*.java'))]),
        ('kotlin', [str(java), '-cp', str(libs / '*'), 'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler', '-no-stdlib', '-no-reflect', '-classpath', os.pathsep.join(map(str, [stdlib, gson, classes])), '-d', str(classes), *map(str, fixture.glob('*.kt'))]),
        ('oracle', [str(java), '-cp', os.pathsep.join(map(str, [classes, stdlib, gson])), 'MainKt', str(out / 'inputs.json')]),
        ('test', ['cargo', '+stable', 'test', '--manifest-path', str(out / 'Cargo.toml')]),
        ('clippy', ['cargo', '+stable', 'clippy', '--locked', '--manifest-path', str(out / 'Cargo.toml'), '--all-targets', '--', '-D', 'warnings']),
    ]
    commands = []
    for name, argv in invocations:
        start = time.monotonic(); result = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True)
        for stream in ['stdout', 'stderr']:
            (out / f'{name}.{stream}.log').write_bytes(getattr(result, stream))
        commands.append(dict(argv=argv, exitCode=result.returncode, seconds=round(time.monotonic() - start, 3)))
        if result.returncode:
            break
        if name == 'oracle':
            (out / 'expected.json').write_bytes(result.stdout)
    report = dict(passed=all(c['exitCode'] == 0 for c in commands), registrationShim=False, vectors=len(inputs), commands=commands,
                  originalSha256=unchanged,
                  scope='Complete registered production configuration/behavior snapshot and unchanged original ListValue, regex adapter, group serializer, exclusion strategies. Explicit minimal Value/group/MC providers, actual Gson/JDK25. Linked-set order, Regex identity duplicates, null deduplication, strict scalar types and atomic replacement failures. Native rendering remains a separate check.',
                  snapshotSha256={str(f.relative_to(out)): hashlib.sha256(f.read_bytes()).hexdigest() for f in out.rglob('*') if f.is_file() and f.suffix in ['.rs', '.kt', '.java', '.json', '.toml', '.bin'] and f.name != 'report.json'})
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf8')
    print(json.dumps(dict(passed=report['passed'], report=str(out / 'report.json'))))


if __name__ == '__main__':
    main()
