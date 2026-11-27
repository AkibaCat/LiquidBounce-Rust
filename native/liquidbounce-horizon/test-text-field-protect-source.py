#!/usr/bin/env python3
"""Original TextFieldProtect and original Char.repeat, actual Kotlin/JDK25 Regex."""
from pathlib import Path
import argparse, hashlib, json, os, re, shutil, subprocess, time
ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent


def vectors():
    rows = []
    inputs = ['', '/login', '/login secret', '/register 密码 密码', '/email a@b.c',
              'hello', '/LOGIN secret', '/login\nsecret', '/login😀', '😀/login secret',
              '/login secret suffix', 'secret', '/register', '/email', '中文😀']
    patterns = [None, [], ['^/login.*'], ['(?i)^/login.*'], ['.*'], [''],
                ['^/register.*', '^/login.*', '^/email.*'], ['Never', '^/login.*'],
                [None], [None, '^/login.*'], ['^/login.*', None], [None, None],
                [r'(?<=a*)b'], [r'\p{L}+'], [r'\X+'], [r'(?iu)i'],
                [r'(?<x>login)\k<x>'], [r'[a-z&&[^aeiou]]+'], ['[']]
    for pattern in patterns:
        for text in inputs + ['aaab', '中文', '😀', 'loginlogin', 'İ', 'bcdf']:
            for index in [-1, 0, 1, 2, 9, 129]:
                for running in [False, True]:
                    rows.append(dict(patterns=pattern, input=text, index=index, running=running))
    # Actual pre/post-cursor arguments can match differently. The second call
    # still receives displayPos, independently of the formatter's offset.
    for value in ['/login secret', '/login 中文😀 secret', '/register secret secret', '/email a@b.c']:
        for display in range(len(value) + 1):
            for cursor in range(display, len(value) + 1):
                shown = value[display:]
                split = cursor - display
                for substring in [shown[:split], shown[split:]]:
                    rows.append(dict(patterns=None, input=substring,
                                     index=len(value[:display].encode('utf-16-le')) // 2, running=True))
    return rows


def main():
    p = argparse.ArgumentParser(); p.add_argument('--out', required=True); a = p.parse_args()
    out = (ROOT / a.out).resolve()
    if (out / 'report.json').exists():
        raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True, exist_ok=True)
    fixture = out / 'fixture'
    shutil.copytree(HERE / 'text-field-protect-source-fixture', fixture, dirs_exist_ok=True)
    module = ROOT / 'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/misc/ModuleTextFieldProtect.kt'
    shutil.copyfile(module, fixture / module.name)
    repeat_source = ROOT / 'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/utils/text/TextExtensions.kt'
    repeat = re.search(r'(?m)^fun Char\.repeat\(n: Int\): String = .*$', repeat_source.read_text(encoding='utf8')).group()
    (fixture / 'repeat.kt').write_text('package net.ccbluex.liquidbounce.utils.text\n' + repeat + '\n', encoding='utf8')
    inputs = vectors()
    (out / 'inputs.json').write_text(json.dumps(inputs, ensure_ascii=True), encoding='utf8')
    java = ROOT.parent / 'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    libs = ROOT.parent / 'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib'
    stdlib = next(libs.glob('kotlin-stdlib-*.jar')); gson = next(libs.glob('gson-*.jar'))
    classes = out / 'classes'; classes.mkdir(exist_ok=True)
    commands = []
    env = os.environ.copy(); env['CARGO_TARGET_DIR'] = str(ROOT / 'build/text-field-protect-target')
    def run(name, argv):
        start = time.monotonic()
        result = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True)
        for stream in ['stdout', 'stderr']:
            (out / f'{name}.{stream}.log').write_bytes(getattr(result, stream))
        commands.append(dict(argv=argv, exitCode=result.returncode, seconds=round(time.monotonic() - start, 3)))
        return result
    result = run('compile', [str(java), '-cp', str(libs / '*'), 'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler',
                            '-no-stdlib', '-no-reflect', '-classpath', os.pathsep.join(map(str, [stdlib, gson])),
                            '-d', str(classes), *map(str, fixture.glob('*.kt'))])
    if result.returncode == 0:
        result = run('oracle', [str(java), '-cp', os.pathsep.join(map(str, [classes, stdlib, gson])), 'MainKt', str(out / 'inputs.json')])
    if result.returncode == 0:
        (out / 'expected.json').write_bytes(result.stdout)
        shutil.copyfile(ROOT / 'native/liquidbounce-behaviors/src/text_field_protect.rs', out / 'text_field_protect.rs')
        shutil.copyfile(fixture / 'oracle.rs', out / 'oracle.rs')
        (out / 'lib.rs').write_text('pub mod text_field_protect;\n#[cfg(test)] mod oracle;\n', encoding='utf8')
        shutil.copytree(ROOT / 'native/liquidbounce-behaviors/vendor/java_regex', out / 'vendor/java_regex', dirs_exist_ok=True)
        (out / 'Cargo.toml').write_text('[package]\nname="text-field-protect-source-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\njava_regex={path="vendor/java_regex"}\n', encoding='utf8')
        result = run('test', ['cargo', '+stable', 'test', '--manifest-path', str(out / 'Cargo.toml')])
        if result.returncode == 0:
            run('clippy', ['cargo', '+stable', 'clippy', '--locked', '--manifest-path', str(out / 'Cargo.toml'), '--all-targets', '--', '-D', 'warnings'])
    report = dict(passed=all(c['exitCode'] == 0 for c in commands), vectors=len(inputs), commands=commands,
                  unchangedModule=module.read_bytes() == (fixture / module.name).read_bytes(),
                  sourceSha256=hashlib.sha256(module.read_bytes()).hexdigest(),
                  originalRepeat=repeat,
                  scope='Byte-identical original module, exact original Char.repeat body and actual Kotlin/JDK25 Regex. Explicit module/linked-set providers. Running gates, defaults, nullable ordered patterns, compile failures, negative indices, displayPos rather than input length, pre/post-cursor matching. Render integration and game display are separate.',
                  snapshotSha256={str(f.relative_to(out)): hashlib.sha256(f.read_bytes()).hexdigest() for f in out.rglob('*') if f.is_file() and f.suffix in ['.rs', '.kt', '.json', '.toml', '.bin'] and f.name != 'report.json'})
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf8')
    print(json.dumps(dict(passed=report['passed'], report=str(out / 'report.json'))))


if __name__ == '__main__':
    main()
