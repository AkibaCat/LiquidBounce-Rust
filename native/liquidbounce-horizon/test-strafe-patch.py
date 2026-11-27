#!/usr/bin/env python3
"""Run unchanged Kotlin ModuleStrafe against real MC vectors and native movement."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import runpy
import shutil
import subprocess
import time
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
UPSTREAM=ROOT/'upstream/Horizon'


def block(text,anchor):
    start=text.index(anchor)
    end=text.index('{',start)+1
    depth=1
    while depth:
        depth+=(text[end]=='{')-(text[end]=='}')
        end+=1
    return text[start:end]


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--out',default='build/strafe-01')
    args=parser.parse_args()
    out=(ROOT/args.out).resolve()
    out.relative_to(ROOT/'build')
    if out.exists(): raise SystemExit('Choose a fresh evidence directory')
    out.mkdir(parents=True)
    records=[]
    def run(name,argv,env=None):
        start=time.monotonic()
        result=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        (out/(name+'.log')).write_text(result.stdout+result.stderr,encoding='utf-8')
        records.append({'argv':argv,'exitCode':result.returncode,'seconds':round(time.monotonic()-start,3)})
        if result.returncode: print((result.stdout+result.stderr)[-12000:])
        return result
    module=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/movement/ModuleStrafe.kt'
    entity=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/utils/entity/EntityExtensions.kt'
    directional=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/utils/movement/DirectionalInput.kt'
    math=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/utils/math/MinecraftVectorExtensions.kt'
    shutil.copyfile(module,out/'ModuleStrafe.kt')
    shutil.copyfile(directional,out/'DirectionalInput.kt')
    original=entity.read_text(encoding='utf-8')
    helpers='''package net.ccbluex.liquidbounce.utils.entity
import net.ccbluex.liquidbounce.features.addon.AddonApi
import net.ccbluex.liquidbounce.utils.movement.DirectionalInput
import net.ccbluex.liquidbounce.features.module.FixtureInput
import net.ccbluex.liquidbounce.features.module.modules.movement.ModuleStrafe
import net.minecraft.client.player.ClientInput
import net.minecraft.client.player.LocalPlayer
import net.minecraft.world.phys.Vec2
import net.minecraft.world.phys.Vec3
import kotlin.math.sin
import kotlin.math.cos
val player get()=ModuleStrafe.player
val ClientInput.untransformed get()=(this as FixtureInput).original
'''
    helpers+=original[original.index('val LocalPlayer.moving'):original.index('val ClientInput.untransformed')]
    helpers+=block(original,'fun LocalPlayer.getMovementDirectionOfInput(')+'\n'
    helpers+=block(original,'fun getMovementDirectionOfInput(')+'\n'
    helpers+=block(original,'fun Vec3.withStrafe(')+'\n'
    (out/'EntityHelpers.kt').write_text(helpers,encoding='utf-8')
    copy=next(line for line in math.read_text(encoding='utf-8').splitlines() if line.startswith('inline fun Vec3.copy('))
    (out/'Math.kt').write_text('package net.ccbluex.liquidbounce.utils.math\nimport net.minecraft.world.phys.Vec3\n'+copy+'\n',encoding='utf-8')
    fixtures={
        'AddonApi.kt':'''package net.ccbluex.liquidbounce.features.addon
annotation class AddonApi
''',
        'Event.kt':'''package net.ccbluex.liquidbounce.event.events
import net.minecraft.world.entity.MoverType
import net.minecraft.world.phys.Vec3
class PlayerMoveEvent(val type:MoverType,var movement:Vec3)
''',
        'Handler.kt':'''package net.ccbluex.liquidbounce.event
fun <E> handler(callback:(E)->Unit):(E)->Unit=callback
''',
        'Module.kt':'''package net.ccbluex.liquidbounce.features.module
import kotlin.reflect.KProperty
import net.minecraft.client.player.ClientInput
import net.minecraft.client.player.LocalPlayer
import net.minecraft.world.entity.player.Input
import net.minecraft.world.phys.Vec2
enum class ModuleCategories {MOVEMENT}
class FixtureInput:ClientInput() {
 var original=Input.EMPTY
 fun vector(x:Float,y:Float) {moveVector=Vec2(x,y)}
}
class Value<T>(var value:T) {
 operator fun getValue(receiver:Any?,property:KProperty<*>):T=value
 operator fun setValue(receiver:Any?,property:KProperty<*>,newValue:T){value=newValue}
}
open class ClientModule(val name:String,val category:ModuleCategories) {
 val player:LocalPlayer
 val floats=mutableMapOf<String,Value<Float>>()
 val bools=mutableMapOf<String,Value<Boolean>>()
 init {
  val field=sun.misc.Unsafe::class.java.getDeclaredField("theUnsafe");field.isAccessible=true
  player=(field.get(null) as sun.misc.Unsafe).allocateInstance(LocalPlayer::class.java) as LocalPlayer
  player.input=FixtureInput()
 }
 fun float(name:String,value:Float,range:ClosedFloatingPointRange<Float>):Value<Float> = Value(value).also {floats[name]=it}
 fun boolean(name:String,value:Boolean):Value<Boolean> = Value(value).also {bools[name]=it}
}
''',
        'Main.kt':'''import net.ccbluex.liquidbounce.features.module.modules.movement.ModuleStrafe as M
import net.ccbluex.liquidbounce.features.module.FixtureInput
import net.ccbluex.liquidbounce.event.events.PlayerMoveEvent
import net.minecraft.world.entity.Entity
import net.minecraft.world.entity.MoverType
import net.minecraft.world.entity.player.Input
import net.minecraft.world.phys.Vec3
fun main() {
 net.minecraft.SharedConstants.tryDetectVersion();net.minecraft.server.Bootstrap.bootStrap()
 val ground=Entity::class.java.getDeclaredField("onGround");ground.isAccessible=true
 val yaw=Entity::class.java.getDeclaredField("yRot");yaw.isAccessible=true
 val out=java.io.File(System.getProperty("vectors")).bufferedWriter()
 java.io.File(System.getProperty("commands")).forEachLine {line ->
  val t=line.split(" ");val input=t[5].toInt()
  M.floats["StrengthInAir"]!!.value=t[1].toFloat();M.floats["StrengthOnGround"]!!.value=t[2].toFloat()
  M.bools["StrictMovement"]!!.value=t[3].toBoolean();ground.setBoolean(M.player,t[4].toBoolean());yaw.setFloat(M.player,t[0].toFloat())
  val state=M.player.input as FixtureInput
  state.original=Input(input and 1!=0,input and 2!=0,input and 4!=0,input and 8!=0,false,false,false)
  state.vector(t[6].toFloat(),t[7].toFloat())
  val event=PlayerMoveEvent(MoverType.values()[t[8].toInt()],Vec3(t[10].toDouble(),t[11].toDouble(),t[12].toDouble()))
  if(t[9].toBoolean()) M.moveHandler(event)
  val v=event.movement
  out.write(listOf(v.x,v.y,v.z).joinToString(","){java.lang.Double.doubleToRawLongBits(it).toULong().toString()});out.newLine()
 }
 out.close()
}
''',
    }
    for name,text in fixtures.items(): (out/name).write_text(text,encoding='utf-8')
    commands=[]
    yaws=[0,-0.0,45,90,-180,361.25,-725.5,16777216,-12345.75]
    strengths=[(-4.5,2.5),(-0.0,0),(0.25,0.75),(1,1),(2.5,-3.75),(1e-30,0.10000000149011612)]
    vectors=[(0,0),(0.70710677,0.70710677),(-0.0,0),(1e-30,-1e-30)]
    movements=[(0.3,-0.0,-0.4),(-1.25,0.42,0.125),(0,0,0)]
    for yi,yaw in enumerate(yaws):
      for flags in range(16):
       for si,(air,ground) in enumerate(strengths):
        for on_ground in [False,True]:
         for strict in [False,True]:
          for vi,(left,forward) in enumerate(vectors):
           for mover in range(5):
            x,y,z=movements[(yi+si+vi)%len(movements)]
            row=[yaw,air,ground,strict,on_ground,flags,left,forward,mover,True,x,y,z]
            commands.append(' '.join(str(value).lower() for value in row))
    for mover in range(5): commands.append(f'90 1 1 true false 1 0 1 {mover} false 0.3 -0.0 -0.4')
    (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
    cache=Path(os.environ['USERPROFILE'])/'.gradle/caches'
    jars=[cache/'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar']
    for group in ['com.mojang','com.google.guava','com.google.code.gson','it.unimi.dsi','org.slf4j','org.joml','io.netty','org.apache.logging.log4j','org.apache.commons','org.lwjgl']:
        jars.extend((cache/'modules-2/files-2.1'/group).rglob('*.jar'))
    jars.extend((ROOT/'build/resource-runtime/libraries').rglob('*.jar'))
    kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib'
    stdlib=next(kotlin.glob('kotlin-stdlib-*.jar'))
    classpath=os.pathsep.join(map(str,[stdlib,*jars]))
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    result=run('kotlin-compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',classpath,'-d',str(out/'classes'),*[str(p) for p in sorted(out.glob('*.kt'))]])
    if result.returncode==0:
        result=run('kotlin-oracle',[str(java),'-Duser.language=en','-Duser.country=US','-Dfile.encoding=UTF-8','-Dcommands='+str(out/'commands.txt'),'-Dvectors='+str(out/'vectors.txt'),'-cp',os.pathsep.join([str(out/'classes'),classpath]),'MainKt'])
    changed,originals={},{}
    def edit(path,action):
        source=UPSTREAM/path
        if source.exists(): originals[path]=hashlib.sha256(source.read_bytes()).hexdigest()
        changed[path]=action(changed.get(path,source.read_text(encoding='utf-8') if source.exists() else ''))
    def once(text,before,after):
        assert text.count(before)==1,(before,text.count(before))
        return text.replace(before,after,1)
    for name in ['mining','strafe']: runpy.run_path(str(HERE/(name+'-patch.py')))['apply'](edit,once)
    native=changed['crates/horizon-client/src/game/native_hooks.rs']
    motion=changed['crates/horizon-client/src/motion.rs']
    remote=changed['crates/horizon-client/src/game/remote.rs']
    native_impl="impl Game<'_>{\n"+block(native,'    pub(super) fn native_client_strafe(')+'\n}\n'
    assert remote.index('self.native_client_strafe(')<remote.index('player.client_strafe =')<remote.index('let was_gliding')
    assert motion.index('let movement = self.client_strafe_movement(movement);')<motion.index('entity_move::move_entity(self, MoverType::SelfMovement, movement)?;')
    source=(HERE/'strafe-fixture.rs').read_text(encoding='utf-8')
    for key,body in {
        'IDENTITY':block(motion,'    pub fn native_client_queries_allowed('),
        'MODIFIER':block(motion,'    fn client_strafe_movement('),
        'MOVE':block(motion,'    pub fn move_self('),
        'SHULKER':block(motion,'    pub fn shulker_box_push('),
        'NATIVE':native_impl,
    }.items(): source=source.replace('/* '+key+' */',body)
    (out/'lib.rs').write_text(source,encoding='utf-8')
    shutil.copyfile(ROOT/'native/liquidbounce-behaviors/src/strafe.rs',out/'strafe.rs')
    api=out/'api-source';game=api/'crates/horizon-client/src/game';game.mkdir(parents=True)
    (api/'tools/native-hook-fragments').mkdir(parents=True)
    (game.parent/'mod_api.rs').write_text('',encoding='utf-8')
    (game/'native_hooks.rs').write_text(native_impl,encoding='utf-8')
    render=api/'crates/horizon-client-render/src/gfx/native_render/generated.rs';render.parent.mkdir(parents=True)
    render.write_text('use super::*;pub trait RenderHooks:Send+Sync{}\n',encoding='utf-8')
    (api/'native-hooks.json').write_text(json.dumps({'helpers':[],'hooks':[{'name':'client_strafe','target':"horizon_client::game::native_hooks::Game<'_>::native_client_strafe",'mod_api':{'contract':'listen'}}]},indent=2)+'\n',encoding='utf-8')
    (out/'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();}\n',encoding='utf-8')
    manifest='[package]\nname="liquidbounce-strafe-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\n'
    for name in ['horizon-core','horizon-world']: manifest+=name+'={path="'+(UPSTREAM/'crates'/name).as_posix()+'"}\n'
    manifest+='[build-dependencies]\nhorizon-mod-api-gen={path="'+(UPSTREAM/'crates/horizon-mod-api-gen').as_posix()+'"}\n'
    (out/'Cargo.toml').write_text(manifest,encoding='utf-8')
    if result.returncode==0:
        env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target')
        result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml'),'--','--test-threads=1'],env)
        if result.returncode==0: result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
    files=[module,entity,directional,math,HERE/'strafe-patch.py',HERE/'strafe-fixture.rs',Path(__file__),ROOT/'native/liquidbounce-behaviors/src/strafe.rs',HERE/'engine/src/config/strafe.rs']
    report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'commands':records,'cases':len(commands),
        'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in files},
        'upstreamSha256':originals,'transformedSha256':{p:hashlib.sha256(t.encode()).hexdigest() for p,t in changed.items()},
        'scope':'Unmodified ModuleStrafe and DirectionalInput; byte-exact extracted entity/vector helpers; actual MC LocalPlayer onGround/yRot/input, ClientInput, Vec2, Vec3, MoverType and JDK Math; production native adapter and real Entity.move implementation.',
        'boundaries':['ClientModule value delegates and handler running predicate shell','constructor-free actual LocalPlayer fields and fixture untransformed-input mixin storage','collision provider and Game/window shell','parent config/registry and full-game execution'],
        'fullGameIntegrationPassed':False}
    report['upstreamUnchanged']=all(hashlib.sha256((UPSTREAM/p).read_bytes()).hexdigest()==digest for p,digest in originals.items())
    report['passed']&=report['upstreamUnchanged']
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
    raise SystemExit(0 if report['passed'] else 1)


if __name__=='__main__':main()
