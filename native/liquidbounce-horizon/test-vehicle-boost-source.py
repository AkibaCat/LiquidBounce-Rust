#!/usr/bin/env python3
"""Run unmodified ModuleVehicleBoost.kt against the production Rust behavior."""
from pathlib import Path
import argparse, hashlib, json, os, shutil, subprocess, time
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', default='build/vehicle-boost-source-01')
    out = (ROOT / parser.parse_args().out).resolve()
    out.relative_to(ROOT / 'build')
    if out.exists(): raise SystemExit('Choose a fresh evidence directory')
    out.mkdir(parents=True)
    source = ROOT / 'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/movement/ModuleVehicleBoost.kt'
    shutil.copyfile(source, out / 'ModuleVehicleBoost.kt')
    fixtures = {
        'Event.kt': '''package net.ccbluex.liquidbounce.event.events
class GameTickEvent
''',
        'Handler.kt': '''package net.ccbluex.liquidbounce.event
fun <E> handler(callback: (E) -> Unit): (E) -> Unit = callback
''',
        'Vec3.kt': '''package net.minecraft.world.phys
data class Vec3(val x: Double, val y: Double, val z: Double)
''',
        'Module.kt': '''package net.ccbluex.liquidbounce.features.module
import kotlin.reflect.KProperty
import net.minecraft.world.phys.Vec3
enum class ModuleCategories { MOVEMENT }
class Player { var isPassenger = false; var yRot = 0f; var deltaMovement = Vec3(9.0,8.0,7.0) }
class FloatValue(var value: Float) {
 operator fun getValue(receiver: Any?, property: KProperty<*>): Float = value
 operator fun setValue(receiver: Any?, property: KProperty<*>, newValue: Float) { value = newValue }
}
open class ClientModule(val name: String, val category: ModuleCategories) {
 val player = Player(); val values = mutableMapOf<String,FloatValue>()
 fun float(name: String, value: Float, range: ClosedFloatingPointRange<Float>): FloatValue = FloatValue(value).also { values[name]=it }
}
''',
        'Main.kt': '''import net.ccbluex.liquidbounce.features.module.modules.movement.ModuleVehicleBoost as M
import net.ccbluex.liquidbounce.event.events.GameTickEvent
import net.minecraft.world.phys.Vec3
fun main() {
 val commands=java.io.File(System.getProperty("commands")).readLines()
 for (line in commands) {
  val t=line.split(" "); M.player.yRot=t[0].toFloat(); M.player.isPassenger=t[1].toBoolean()
  M.values["HorizontalSpeed"]!!.value=t[3].toFloat(); M.values["VerticalSpeed"]!!.value=t[4].toFloat()
  M.player.deltaMovement=Vec3(9.0,8.0,7.0)
  if(t[2].toBoolean()) M.repeatable(GameTickEvent())
  val v=M.player.deltaMovement
  println(listOf(v.x,v.y,v.z).joinToString(",") { java.lang.Double.doubleToRawLongBits(it).toULong().toString() })
 }
}
'''
    }
    for name, text in fixtures.items(): (out / name).write_text(text, encoding='utf-8')
    commands = ['0 false true 2 1', '45 true true 2 1', '90 false false 2 1',
                '90 false true 2 1', '90 false true 2 1']
    for yaw in [0, -0.0, 45, 90, -180, 361.25, -725.5, 16777216, -12345.75]:
        for horizontal, vertical in [(2,1),(-9,-7),(0.1,5),(13.125,0.42)]:
            commands += [f'{yaw} true true {horizontal} {vertical}', f'{yaw} false true {horizontal} {vertical}']
    (out / 'commands.txt').write_text('\n'.join(commands)+'\n', encoding='utf-8')
    java = ROOT.parent / 'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    kotlin = ROOT.parent / 'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib'
    stdlib = next(kotlin.glob('kotlin-stdlib-*.jar'))
    records = []
    def run(name, args, env=None):
        start = time.monotonic()
        result = subprocess.run(args, cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace')
        (out / (name+'.log')).write_text(result.stdout+result.stderr, encoding='utf-8')
        records.append({'argv':args, 'exitCode':result.returncode, 'seconds':round(time.monotonic()-start,3)})
        return result
    result = run('compile', [str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',str(stdlib),'-d',str(out/'classes'),*[str(p) for p in sorted(out.glob('*.kt'))]])
    if result.returncode == 0:
        result = run('kotlin', [str(java),'-Dcommands='+str(out/'commands.txt'),'-cp',os.pathsep.join([str(out/'classes'),str(stdlib)]),'MainKt'])
    if result.returncode == 0:
        vectors = [[int(x) for x in line.split(',')] for line in result.stdout.strip().splitlines()]
        assert len(vectors) == len(commands)
        (out/'vectors.json').write_text(json.dumps(vectors), encoding='utf-8')
        shutil.copyfile(ROOT/'native/liquidbounce-behaviors/src/vehicle_boost.rs', out/'vehicle_boost.rs')
        (out/'lib.rs').write_text('''pub mod vehicle_boost;
#[cfg(test)] mod oracle {
 #[test] fn unchanged_kotlin_source_transition_vectors() {
  let vectors:Vec<[u64;3]>=serde_json::from_str(include_str!("vectors.json")).unwrap();
  let mut behavior=crate::vehicle_boost::VehicleBoost::default();
  for (line,expected) in include_str!("commands.txt").lines().zip(vectors) {
   let t:Vec<_>=line.split(' ').collect();
   behavior.horizontal_speed=t[3].parse().unwrap();behavior.vertical_speed=t[4].parse().unwrap();
   let actual=behavior.game_tick(t[2]=="true",t[1]=="true",t[0].parse().unwrap(),|angle| [horizon_core::jmath::sin(angle),horizon_core::jmath::cos(angle)]).unwrap_or([9.,8.,7.]);
   assert_eq!(actual.map(f64::to_bits),expected,"{line}");
  }
 }
}
''', encoding='utf-8')
        core = (ROOT/'upstream/Horizon/crates/horizon-core').as_posix()
        (out/'Cargo.toml').write_text('[package]\nname="vehicle-boost-source-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\nhorizon-core={path="'+core+'"}\n', encoding='utf-8')
        env = os.environ.copy(); env['CARGO_TARGET_DIR'] = str(ROOT/'build/no-pose-hook-tests/target')
        result = run('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')],env)
        if result.returncode == 0:
            run('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
    report = {'passed':all(r['exitCode']==0 for r in records), 'commands':records,
              'moduleSha256':hashlib.sha256(source.read_bytes()).hexdigest(),
              'rows':len(commands), 'scope':'Unmodified pinned Kotlin module, real JDK25 trigonometry, production Rust state and Horizon Java Math; environment storage stubs. Full native API and in-game vehicle interactions are separate.'}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
    raise SystemExit(0 if report['passed'] else 1)

if __name__ == '__main__': main()
