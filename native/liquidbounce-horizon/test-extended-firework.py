#!/usr/bin/env python3
"""Unedited Kotlin modes/Vec3d adapter, original Java expression, actual MC Vec3."""
from pathlib import Path
import argparse
import hashlib
import json
import os
import re
import runpy
import shutil
import subprocess
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def block(text, anchor):
    start = text.index(anchor)
    end = text.index('{', start) + 1
    depth = 1
    while depth:
        depth += (text[end] == '{') - (text[end] == '}')
        end += 1
    return text[start:end]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', default='build/extended-firework-01')
    out = (ROOT / parser.parse_args().out).resolve()
    out.relative_to(ROOT / 'build')
    if out.exists():
        raise SystemExit('Choose a fresh output directory')
    out.mkdir(parents=True)
    upstream = ROOT / 'upstream/LiquidBounce/src/main'
    originals = [
        upstream / 'kotlin/net/ccbluex/liquidbounce/features/module/modules/exploit/ModuleExtendedFirework.kt',
        upstream / 'kotlin/net/ccbluex/liquidbounce/config/gson/adapter/VectorAdapter.kt',
        upstream / 'java/net/ccbluex/liquidbounce/injection/mixins/minecraft/entity/projectile/MixinFireworkRocketEntity.java',
    ]
    for path in originals[:2]:
        shutil.copyfile(path, out / path.name)
    expressions = re.findall(r'args.set\([012], (.*)\);', originals[2].read_text(encoding='utf-8'))
    assert len(expressions) == 3
    (out / 'OriginalExpression.java').write_text('''import net.minecraft.world.phys.Vec3;
public class OriginalExpression {
 public static Vec3 apply(Vec3 lookAngle, Vec3 movement, Vec3 multiplier) {
  return movement.add(''' + ','.join(expressions) + ''');
 }
}
''', encoding='utf-8')
    (out / 'Mode.kt').write_text('''package net.ccbluex.liquidbounce.config.types.group
open class Mode(val name:String) { open val parent:ModeValueGroup<*> get()=error("no parent") }
class ModeValueGroup<T:Mode>(val modes:Array<T>, var activeMode:T)
''', encoding='utf-8')
    (out / 'Module.kt').write_text('''package net.ccbluex.liquidbounce.features.module
import kotlin.reflect.KProperty
import net.ccbluex.liquidbounce.config.types.group.*
import net.minecraft.world.phys.Vec3
import net.minecraft.client.player.LocalPlayer
enum class ModuleCategories { MOVEMENT }
class VectorValue(var value:Vec3) { operator fun getValue(receiver:Any?,property:KProperty<*>)=value }
open class ClientModule(val name:String,val category:ModuleCategories) {
 val groups=mutableMapOf<String,ModeValueGroup<*>>()
 val vectors=mutableMapOf<String,VectorValue>()
 fun <T:Mode> choices(name:String,default:T,modes:Array<T>)=ModeValueGroup(modes,default).also{groups[name]=it}
 fun vec3d(name:String,default:Vec3,useLocateButton:Boolean)=VectorValue(default).also{vectors[name]=it}
}
''', encoding='utf-8')
    (out / 'Main.kt').write_text('''import com.google.gson.*
import net.ccbluex.liquidbounce.features.module.modules.exploit.ModuleExtendedFirework as M
import net.ccbluex.liquidbounce.config.types.group.*
import net.ccbluex.liquidbounce.config.gson.adapter.Vec3dAdapter
import net.minecraft.world.phys.Vec3
import net.minecraft.client.player.LocalPlayer
fun bits(v:Vec3)=listOf(v.x.toRawBits(),v.y.toRawBits(),v.z.toRawBits())
fun main(args:Array<String>) {
 net.minecraft.SharedConstants.tryDetectVersion();net.minecraft.server.Bootstrap.bootStrap()
 val gson=GsonBuilder().serializeNulls().registerTypeHierarchyAdapter(Vec3::class.java,Vec3dAdapter).create()
 val source=JsonParser.parseString(java.io.File(args[0]).readText()).asJsonObject
 val output=JsonObject()
 val imports=JsonArray()
 for(input in source.getAsJsonArray("imports")) {
  val row=JsonObject(); row.add("input",input)
  try { val v=gson.fromJson(input,Vec3::class.java);row.addProperty("ok",v!=null);if(v!=null)row.add("bits",gson.toJsonTree(bits(v))) }
  catch(t:Throwable) {row.addProperty("ok",false)}
  imports.add(row)
 }
 output.add("imports",imports)
 val boosts=JsonArray()
 val field=sun.misc.Unsafe::class.java.getDeclaredField("theUnsafe");field.isAccessible=true
 val player=(field.get(null) as sun.misc.Unsafe).allocateInstance(LocalPlayer::class.java) as LocalPlayer
 for(input in source.getAsJsonArray("boosts")) {
  val row=input.asJsonObject
  @Suppress("UNCHECKED_CAST") val group=M.groups["Mode"] as ModeValueGroup<Mode>
  group.activeMode=group.modes.first{it.name==row["mode"].asString}
  M.vectors["VelocityMultiplier"]!!.value=gson.fromJson(row["custom"],Vec3::class.java)
  player.xRot=row["pitch"].asFloat;player.yRot=row["yaw"].asFloat
  val look=if(row["managed"].asBoolean) Vec3.directionFromRotation(player.xRot,player.yRot) else player.lookAngle
  val movement=gson.fromJson(row["movement"],Vec3::class.java)
  val result=OriginalExpression.apply(look,movement,if(row["enabled"].asBoolean) M.velocityMultiplier else Vec3(0.1,1.5,0.5))
  val actual=JsonObject();actual.add("input",row);actual.add("lookBits",gson.toJsonTree(bits(look)))
  actual.add("multiplierBits",gson.toJsonTree(bits(M.velocityMultiplier)));actual.add("resultBits",gson.toJsonTree(bits(result)));boosts.add(actual)
 }
 output.add("boosts",boosts)
 java.io.File(args[1]).writeText(gson.toJson(output))
}
''', encoding='utf-8')
    imports = [{}, {'x': 1, 'y': -2.5, 'z': 3}, {'x': '  +2.5f '}, {'x': '0x1.8p2'},
               {'x': '-0x0p0'}, {'x': 'NaN'}, {'x': 'Infinity'}, {'x': '-Infinity'},
               {'x': '1e400'}, {'x': '0x1p1024'}, {'x': '0x1p-1074'}, {'x': '0x1p-1075'},
               {'x': '0x1.8p-1074'}, {'x': '0x1.fffffffffffffp1023'}, {'x': None}, {'x': False},
               {'x': []}, {'x': 1, 'y': {}}, {'ignored': [False]}, [], 5, None]
    imports += [{'x': text} for text in ['nan','infinity','inf','NaNf','InfinityD','+NaN','-NaN','+Infinity',
        '-+0x1p2','0x.p0','0x1p999999999999999999999','-0x1p-999999999999999999999',
        '0x1.fffffffffffff8p1023','0x1.00000000000008p0','0x1.00000000000018p0',
        '.5','5.',' 1\t','1 2','0x1p-1076']]
    import random
    rng = random.Random(263)
    for _ in range(400):
        digits = ''.join(rng.choice('0123456789abcdef') for _ in range(rng.randrange(1, 35)))
        split = rng.randrange(len(digits) + 1)
        imports.append({'x': rng.choice(['','-','+'])+'0x'+digits[:split]+'.'+digits[split:]+'p'+str(rng.randrange(-1120,1050))})
    boosts = []
    for mode in ['Normal', 'Custom']:
        for custom in [[0.1, 1.65, 0.5], [0, 0, 0], [-3.5, 7.25, -0.5], [100, -2, 1]]:
            for pitch, yaw in [[0, 0], [30, 70], [-90, -180], [90, 360], [-70.25, 3500.125], [0, 1e9]]:
                for movement in [[0, 0, 0], [0.123, -7.5, 2.75], [1e8, -1e-5, 0.7]]:
                    vec = lambda v: dict(zip('xyz', v))
                    for managed in [False, True]:
                        for enabled in [False, True]:
                            boosts.append(dict(mode=mode, custom=vec(custom), pitch=pitch, yaw=yaw, movement=vec(movement), managed=managed, enabled=enabled))
    (out / 'inputs.json').write_text(json.dumps(dict(imports=imports, boosts=boosts)), encoding='utf-8')
    cache = Path(os.environ['USERPROFILE']) / '.gradle/caches'
    jars = [cache / 'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar']
    for group in ['com.mojang', 'com.google.guava', 'com.google.code.gson', 'it.unimi.dsi', 'org.slf4j', 'org.joml', 'io.netty', 'org.apache.logging.log4j', 'org.apache.commons', 'org.lwjgl']:
        jars.extend((cache / 'modules-2/files-2.1' / group).rglob('*.jar'))
    java_dir = ROOT.parent / 'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin'
    kotlin = ROOT.parent / 'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib'
    stdlib = next(kotlin.glob('kotlin-stdlib-*.jar'))
    classpath = os.pathsep.join(map(str, jars + [stdlib, out / 'classes']))
    records = []
    def run(name, argv, env=None):
        start = time.monotonic()
        result = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace')
        (out / (name + '.log')).write_text(result.stdout + result.stderr, encoding='utf-8')
        records.append(dict(argv=argv, exitCode=result.returncode, durationSeconds=round(time.monotonic()-start, 3)))
        if result.returncode:
            raise RuntimeError((result.stdout + result.stderr)[-14000:])
        return result
    run('javac', [str(java_dir / 'javac.exe'), '-cp', classpath, '-d', str(out / 'classes'), str(out / 'OriginalExpression.java')])
    run('kotlin', [str(java_dir / 'java.exe'), '-cp', str(kotlin / '*'), 'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler', '-no-stdlib', '-no-reflect', '-classpath', classpath, '-d', str(out / 'classes')] + list(map(str, out.glob('*.kt'))))
    result = run('oracle', [str(java_dir / 'java.exe'), '-cp', classpath, 'MainKt', str(out / 'inputs.json'), str(out / 'oracle.json')])
    oracle = json.loads((out / 'oracle.json').read_text(encoding='utf-8'))
    (out / 'oracle.json').write_text(json.dumps(oracle, indent=2), encoding='utf-8')
    changed = {}
    upstream_hashes = {}
    upstream_horizon = ROOT / 'upstream/Horizon'
    def edit(path, action):
        source = upstream_horizon / path
        if source.exists(): upstream_hashes[path] = hashlib.sha256(source.read_bytes()).hexdigest()
        changed[path] = action(changed.get(path, source.read_text(encoding='utf-8') if source.exists() else ''))
    def once(text, before, after):
        assert text.count(before) == 1, (before, text.count(before))
        return text.replace(before, after, 1)
    runpy.run_path(str(HERE / 'extended-firework-patch.py'))['apply'](edit, once)
    native = "impl Game<'_>{\n" + block(changed['crates/horizon-client/src/game/native_hooks.rs'], '    pub(super) fn native_client_firework_boost(') + '\n}\n'
    follow = block(changed['crates/horizon-client/src/game/fireworks.rs'], '    pub(super) fn follow_own_player(')
    source = (HERE / 'extended-firework-fixture.rs').read_text(encoding='utf-8').replace('/* NATIVE */', native).replace('/* FOLLOW */', follow)
    (out / 'lib.rs').write_text(source, encoding='utf-8')
    shutil.copyfile(HERE / 'engine/src/config.rs', out / 'config.rs')
    shutil.copytree(HERE / 'engine/src/config', out / 'config')
    api = out / 'api-source'
    game = api / 'crates/horizon-client/src/game';game.mkdir(parents=True)
    (api / 'tools/native-hook-fragments').mkdir(parents=True)
    (game.parent / 'mod_api.rs').write_text('', encoding='utf-8')
    (game / 'native_hooks.rs').write_text(native, encoding='utf-8')
    render = api / 'crates/horizon-client-render/src/gfx/native_render/generated.rs';render.parent.mkdir(parents=True)
    render.write_text('use super::*;pub trait RenderHooks:Send+Sync{}\n', encoding='utf-8')
    (api / 'native-hooks.json').write_text(json.dumps({'helpers':[], 'hooks':[{'name':'client_firework_boost', 'target':"horizon_client::game::native_hooks::Game<'_>::native_client_firework_boost", 'mod_api':{'contract':'listen'}}]}), encoding='utf-8')
    (out / 'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();}\n', encoding='utf-8')
    manifest = '[package]\nname="liquidbounce-extended-firework-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\n'
    manifest += 'liquidbounce-behaviors={path="'+(ROOT/'native/liquidbounce-behaviors').as_posix()+'"}\n'
    for name in ['horizon-core', 'horizon-world']:
        manifest += name+'={path="'+(upstream_horizon/'crates'/name).as_posix()+'"}\n'
    manifest += '[build-dependencies]\nhorizon-mod-api-gen={path="'+(upstream_horizon/'crates/horizon-mod-api-gen').as_posix()+'"}\n'
    (out / 'Cargo.toml').write_text(manifest, encoding='utf-8')
    env = os.environ.copy();env['CARGO_TARGET_DIR'] = str(ROOT / 'build/no-pose-hook-tests/target')
    run('test', ['cargo', '+stable', 'test', '--offline', '--manifest-path', str(out / 'Cargo.toml'), '--', '--test-threads=1'], env)
    run('clippy', ['cargo', '+stable', 'clippy', '--offline', '--manifest-path', str(out / 'Cargo.toml'), '--all-targets', '--', '-D', 'warnings'], env)
    report = dict(passed=True, commands=records, scope='Unmodified Kotlin module modes and Vec3d adapter with minimal settings shells; exact extracted Java mixin arithmetic; actual MC LocalPlayer.getLookAngle/Vec3.directionFromRotation; complete transformed follow_own_player consumer and generated native dispatch, production configuration imports.',
                  sourceSha256={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in originals},
                  imports=len(oracle['imports']), boosts=len(oracle['boosts']), fullGameIntegrationPassed=False)
    report['upstreamUnchanged'] = all(hashlib.sha256((upstream_horizon / p).read_bytes()).hexdigest() == digest for p,digest in upstream_hashes.items())
    (out / 'report.json').write_text(json.dumps(report, indent=2), encoding='utf-8')


if __name__ == '__main__':
    main()
