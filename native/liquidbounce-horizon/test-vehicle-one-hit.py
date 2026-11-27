#!/usr/bin/env python3
"""Unmodified VehicleOneHit + SwingMode against actual MC classes/packets."""
from pathlib import Path
import argparse, hashlib, importlib.util, json, os, shutil, subprocess, time
ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent

def block(source, signature):
    start=source.index(signature); opening=source.index('{',start); depth=1; end=opening+1
    while depth:
        depth+=(source[end]=='{')-(source[end]=='}');end+=1
    return source[start:end]

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', default='build/vehicle-one-hit-01')
    args = parser.parse_args()
    out = (ROOT/args.out).resolve()
    if out.exists():
        raise SystemExit('Use a fresh verification directory')
    out.mkdir(parents=True)
    commands = []
    def run(name, argv, env=None):
        start = time.monotonic()
        result = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace')
        for stream in ['stdout', 'stderr']:
            (out/f'{name}.{stream}.log').write_text(getattr(result, stream), encoding='utf-8')
        commands.append({'argv': argv, 'exitCode': result.returncode, 'seconds': time.monotonic()-start})
        return result
    base = ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce'
    module = base/'features/module/modules/exploit/ModuleVehicleOneHit.kt'
    swing = base/'utils/block/SwingMode.kt'
    shutil.copyfile(module, out/module.name)
    shutil.copyfile(swing, out/swing.name)
    fixtures = {
        'Event.kt': '''package net.ccbluex.liquidbounce.event.events
import net.minecraft.world.entity.Entity
class AttackEntityEvent(val entity:Entity,val cancelled:Boolean)
''',
        'Handler.kt': '''package net.ccbluex.liquidbounce.event
fun <E> handler(callback:(E)->Unit):(E)->Unit=callback
''',
        'Module.kt': '''package net.ccbluex.liquidbounce.features.module
import net.ccbluex.liquidbounce.utils.client.network
enum class ModuleCategories {EXPLOIT}
open class ClientModule(val name:String,val category:ModuleCategories) {val network get()=net.ccbluex.liquidbounce.utils.client.network}
''',
        'Tagged.kt': '''package net.ccbluex.liquidbounce.config.types.list
interface Tagged {val tag:String}
''',
        'Addon.kt': '''package net.ccbluex.liquidbounce.features.addon
annotation class AddonApi
''',
        'Client.kt': '''package net.ccbluex.liquidbounce.utils.client
import net.minecraft.world.InteractionHand
import net.minecraft.world.item.ItemStack
import net.minecraft.world.item.component.SwingAnimation
import net.minecraft.network.protocol.game.ServerboundAttackPacket
val trace=mutableListOf<String>()
var isOlderThanOrEqual1_8=false
class Network {fun send(packet:Any) {trace.add(if(packet is ServerboundAttackPacket) "attack:${packet.entityId()}" else "punch")}}
val network=Network()
class Player {
 lateinit var mainHandItem:ItemStack
 var noSwing=false
 fun getItemInHand(hand:InteractionHand)=mainHandItem
 fun swing(hand:InteractionHand,animation:SwingAnimation,server:Boolean) {
  trace.add("swing:${hand.ordinal}:${animation.type().ordinal}:${animation.duration()}:$server:$noSwing")
 }
}
val player=Player()
''',
        'Main.kt': '''import com.google.gson.*
import java.lang.reflect.*
import net.ccbluex.liquidbounce.features.module.modules.exploit.ModuleVehicleOneHit as M
import net.ccbluex.liquidbounce.event.events.AttackEntityEvent
import net.ccbluex.liquidbounce.utils.client.*
import net.minecraft.SharedConstants
import net.minecraft.server.Bootstrap
import net.minecraft.core.registries.BuiltInRegistries
import net.minecraft.core.component.*
import net.minecraft.world.entity.*
import net.minecraft.world.entity.vehicle.boat.Boat
import net.minecraft.world.entity.vehicle.minecart.Minecart
import net.minecraft.world.item.*
import net.minecraft.world.item.component.SwingAnimation
fun main() {
 SharedConstants.tryDetectVersion();Bootstrap.bootStrap()
 val unsafeField=sun.misc.Unsafe::class.java.getDeclaredField("theUnsafe");unsafeField.isAccessible=true
 val unsafe=unsafeField.get(null) as sun.misc.Unsafe
 for(holder in BuiltInRegistries.ITEM.listElements())holder.bindComponents(DataComponentMap.EMPTY)
 player.mainHandItem=ItemStack(Items.STICK)
 val rows=JsonArray();val types=JsonArray()
 for(field in EntityTypes::class.java.fields) {
  if(field.type!=EntityType::class.java)continue
  val type=field.get(null) as EntityType<*>
  val clazz=(field.genericType as ParameterizedType).actualTypeArguments[0] as Class<*>
  val id=EntityType.getKey(type).toString()
  val eligible=Boat::class.java.isAssignableFrom(clazz)||Minecart::class.java.isAssignableFrom(clazz)
  val info=JsonObject();info.addProperty("kind",id);info.addProperty("class",clazz.name);info.addProperty("eligible",eligible);types.add(info)
  if(Modifier.isAbstract(clazz.modifiers))continue
  // Constructors need a live world; the unmodified handler reads only runtime
  // type and id. Use actual MC objects, setting the actual Entity id field.
  val entity=unsafe.allocateInstance(clazz) as Entity
  for(target in listOf(19,-1,Int.MIN_VALUE,Int.MAX_VALUE)) {
   entity.id=target
   for(legacy in listOf(false,true))for(running in listOf(false,true))for(cancelled in listOf(false,true)) {
    for(animation in listOf(SwingAnimation.DEFAULT,SwingAnimation(SwingAnimationType.STAB,11),SwingAnimation(SwingAnimationType.NONE,0))) {
     player.mainHandItem.set(DataComponents.ATTACK_ANIMATION,animation)
     isOlderThanOrEqual1_8=legacy;trace.clear()
     if(running)M.attackHandler(AttackEntityEvent(entity,cancelled))
     val row=JsonObject();row.addProperty("kind",id);row.addProperty("target",target);row.addProperty("legacy",legacy);row.addProperty("running",running);row.addProperty("cancelled",cancelled)
     row.addProperty("animation",animation.type().ordinal);row.addProperty("duration",animation.duration())
     val events=JsonArray();trace.forEach{events.add(it)};row.add("events",events);rows.add(row)
    }
   }
  }
 }
 val output=JsonObject();output.add("types",types);output.add("cases",rows)
 java.io.File(System.getProperty("oracle")).writeText(output.toString())
}
''',
    }
    for name, text in fixtures.items():
        (out/name).write_text(text, encoding='utf-8')
    cache = Path(os.environ['USERPROFILE'])/'.gradle/caches'
    jars = [cache/'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar']
    for group in ['com.mojang','com.google.guava','com.google.code.gson','it.unimi.dsi','org.slf4j','org.joml','io.netty','org.apache.logging.log4j','org.apache.commons','org.lwjgl']:
        jars.extend((cache/'modules-2/files-2.1'/group).rglob('*.jar'))
    jars.extend((ROOT/'build/resource-runtime/libraries').rglob('*.jar'))
    kotlin = ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib'
    stdlib = next(kotlin.glob('kotlin-stdlib-*.jar'))
    cp = os.pathsep.join(map(str, [stdlib, *jars]))
    java = ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    result = run('compile', [str(java), '-cp', str(kotlin/'*'), 'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler', '-no-stdlib','-no-reflect','-classpath',cp,'-d',str(out/'classes'),*[str(p) for p in sorted(out.glob('*.kt'))]])
    if not result.returncode:
        result = run('oracle', [str(java),'-Dfile.encoding=UTF-8','-Doracle='+str(out/'oracle.json'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'MainKt'])
    shutil.copyfile(ROOT/'native/liquidbounce-behaviors/src/vehicle_one_hit.rs',out/'vehicle_one_hit.rs')
    native=(HERE/'vehicle-one-hit-fixture.rs').read_text(encoding='utf-8')
    first=(ROOT/'upstream/Horizon/crates/horizon-client/src/first_person.rs').read_text(encoding='utf-8')
    version=(ROOT/'upstream/Horizon/crates/horizon-core/src/version.rs').read_text(encoding='utf-8')
    for key,text in {
        'VERSION':next(line for line in version.splitlines() if line.startswith('pub const PROTOCOL_VERSION')),
        'HAND':'#[derive(Clone,Copy,Debug,PartialEq,Eq)]\n'+block(first,'pub enum Hand'),
        'ANIMATION':'#[derive(Clone,Copy,Debug,PartialEq,Eq)]\n'+block(first,'pub struct SwingAnimation'),
        'ANIMATION_IMPL':block(first,'impl SwingAnimation'),
        'HOST':(HERE/'vehicle-one-hit-host.rs').read_text(encoding='utf-8'),
    }.items():native=native.replace('/* '+key+' */',text)
    shutil.copyfile(ROOT/'native/liquidbounce-behaviors/src/no_swing.rs',out/'no_swing.rs')
    # Compile the actual transformed local entries as well as the Host helper.
    transformed={}
    def edit(path, transform):
        source=transformed.get(path)
        if source is None:source=(ROOT/'upstream/Horizon'/path).read_text(encoding='utf-8')
        transformed[path]=transform(source)
    def once(text,before,after):
        if text.count(before)!=1:raise ValueError(f'Expected one local attack anchor: {before[:80]}')
        return text.replace(before,after)
    spec=importlib.util.spec_from_file_location('vehicle_patch',HERE/'vehicle-one-hit-patch.py')
    patch=importlib.util.module_from_spec(spec);spec.loader.exec_module(patch)
    patch.apply(edit,once)
    entries=(HERE/'vehicle-local-attack-fixture.rs').read_text(encoding='utf-8')
    for marker,path,signature in [
        ('BOAT_ENTRY','boats.rs','pub(super) fn attack_boat('),
        ('CART_ENTRY','special_attacks.rs','pub(super) fn attack_cart('),
    ]:
        entries=entries.replace('/* '+marker+' */',block(transformed['crates/horizon-client/src/game/'+path],signature))
    (out/'lib.rs').write_text(native+entries+'''
#[test]
fn unchanged_kotlin_actual_types_packets_and_swing_mode() {
 let oracle:serde_json::Value=serde_json::from_str(include_str!("oracle.json")).unwrap();
 for row in oracle["types"].as_array().unwrap() {
  assert_eq!(vehicle_one_hit::eligible(row["kind"].as_str().unwrap()),row["eligible"].as_bool().unwrap(),"{row}");
 }
 for row in oracle["cases"].as_array().unwrap() {
  let events:Vec<String>=vehicle_one_hit::actions(row["running"].as_bool().unwrap(),row["kind"].as_str().unwrap(),row["target"].as_i64().unwrap() as i32,row["legacy"].as_bool().unwrap()).into_iter().map(|action|match action {
   vehicle_one_hit::Action::Attack(id)=>format!("attack:{id}"),
   vehicle_one_hit::Action::Swing=>format!("swing:0:{}:{}:false:false",row["animation"],row["duration"]),
  }).collect();
  assert_eq!(serde_json::json!(events),row["events"],"{row}");
 }
}
''',encoding='utf-8')
    (out/'Cargo.toml').write_text('[package]\nname="vehicle-one-hit-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\nhorizon-protocol={path="'+(ROOT/'upstream/Horizon/crates/horizon-protocol').as_posix()+'"}\n',encoding='utf-8')
    if not result.returncode:
        env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/vehicle-one-hit-target')
        result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
        if not result.returncode:
            result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
    files=[module,swing,Path(__file__),ROOT/'native/liquidbounce-behaviors/src/vehicle_one_hit.rs',HERE/'vehicle-one-hit-host.rs',HERE/'vehicle-one-hit-patch.py',HERE/'vehicle-one-hit-fixture.rs',HERE/'vehicle-local-attack-fixture.rs']
    oracle=json.loads((out/'oracle.json').read_text()) if (out/'oracle.json').exists() else {}
    report={'passed':len(commands)==4 and all(c['exitCode']==0 for c in commands),'commands':commands,'caseCount':len(oracle.get('cases',[])),'entityTypeCount':len(oracle.get('types',[])),
        'scope':'Unmodified ModuleVehicleOneHit and SwingMode; actual MC entity class hierarchy, Entity ids, ServerboundAttackPacket, ItemStack attack animations; Rust production behavior and byte-exact native Host helpers with real protocol packet types.',
        'boundaries':['event registration/activation shell','worldless Unsafe-allocated actual entities','Player.swing observation shell; native item resource/game/send shell; full-game verification separate'],
        'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in files},'fullIntegrationPassed':False}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
    raise SystemExit(0 if report['passed'] else 1)
if __name__=='__main__':main()
