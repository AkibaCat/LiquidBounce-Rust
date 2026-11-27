#!/usr/bin/env python3
"""Unchanged ModuleAutoWalk with real 26.3 math and scoped navigator/world seams."""
from pathlib import Path
import argparse
import hashlib
import json
import os
import shutil
import struct
import subprocess
import time
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
LB=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce'
def bits(value):return struct.unpack('<Q',struct.pack('<d',value))[0]
def fbits(value):return struct.unpack('<I',struct.pack('<f',value))[0]
def vec(x,y=0,z=0):return ','.join(str(bits(float(v)))for v in [x,y,z])
def text(value):return value.encode().hex()or'-'
def block(source,anchor):
 start=source.index(anchor);end=source.index('{',start)+1;depth=1
 while depth:depth+=(source[end]=='{')-(source[end]=='}');end+=1
 return source[start:end]
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
 if out.exists():raise SystemExit('Use a fresh evidence directory')
 out.mkdir(parents=True);records=[];commands=[]
 def add(*parts):commands.append(' '.join(map(str,parts)))
 # Forward preserves every other input bit; disabled listeners do not run.
 for mask in range(128):add('input',mask)
 add('enabled','true')
 for mask in range(128):add('input',mask)
 # Direction caches across small turns, refreshes at exact half distance or >20 degrees.
 add('mode','Direction');add('input',0);add('yaw',fbits(20));add('input',65);add('yaw',fbits(20.00001));add('input',0);add('position',vec(0,0,512));add('input',0);add('position',vec(0,0,1024));add('input',0)
 for yaw in [180,-180,360,-360,179.99999,-179.99999,1e20,float('nan'),float('inf')]:add('yaw',fbits(yaw));add('input',0)
 add('yaw',fbits(0));add('mode','Forward');add('mode','Direction');add('input',0);add('disable');add('input',0)
 # Unknown chunks recompute feet; once cached, disabling Goto deliberately retains goalOf.
 add('goto',vec(5.9,-1.1,7.2));add('chunk','false');add('stand','5,-2,7','false');add('stand','5,-1,7','true');add('input',0);add('stand','5,-2,7','true');add('input',0);add('chunk','true');add('input',0);add('stand','5,-2,7','false');add('input',0);add('enabled','false');add('enabled','true');add('input',0)
 # Root vehicle determines arrival and notification precedes reset/halt.
 add('vehicle',vec(5.5,-2,7.5));add('input',93);add('enabled','true');add('vehicle',vec(6.5,-2,7.5));add('input',0);add('vehicle','self');add('goto',vec(-0.1,64.99,-1.01));add('input',0)
 # Goto destinations use Java floor/cast, including nonfinite coordinates.
 for x,y,z in [(float('nan'),float('inf'),float('-inf')),(2147483648,-2147483649,0)]:add('goto',vec(x,y,z));add('input',0)
 # Follow uses the first nonlocal Java equalsIgnoreCase name, then lastSeen until disable.
 add('follow',text('akiba'));add('players',text('AKIBA')+':'+vec(1,2,3)+';'+text('Akiba')+':'+vec(7,8,9));add('input',0);add('players','-');add('input',0);add('distance',fbits(7.25));add('input',0);add('target',text('none'));add('input',0);add('enabled','false');add('enabled','true');add('input',0)
 # Selecting another mode while not running retains caches; explicit disable clears active cache.
 add('players',text('none')+':'+vec(2,3,4));add('input',0);add('ingame','false');add('mode','Direction');add('mode','Follow');add('ingame','true');add('players','-');add('input',0);add('disable');add('input',0)
 add('parent','false');add('mode','Goto');add('input',0);add('parent','true');add('input',0)
 names=[('i','I'),('i','İ'),('i','ı'),('Σ','ς'),('σ','Σ'),('ſ','S'),('K','k'),('ß','SS'),('ß','ẞ'),('ﬀ','FF'),('𐐀','𐐨'),('Ⰰ','ⰰ'),('中文','中文'),('A\0B','a\0b'),('',''),('a','aa')]
 for a,b in names:
  add('case',text(a),text(b));add('follow',text(a));add('players',text(b)+':'+vec(9,3,1));add('input',0);add('enabled','false');add('enabled','true')
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=LB/'features/module/modules/player/ModuleAutoWalk.kt';mode_source=LB/'config/types/group/ModeValueGroup.kt';math_source=LB/'utils/math/MinecraftVectorExtensions.kt'
 shutil.copyfile(original,out/'ModuleAutoWalk.kt');shutil.copyfile(HERE/'navigation-source-fixture/AutoWalk.kt',out/'Main.kt')
 source=mode_source.read_text(encoding='utf-8');lifecycle=block(source,'internal fun updateChildState')+'\n'+block(source,'private fun setAndUpdate')
 (out/'Modes.kt').write_text('''package net.ccbluex.liquidbounce.config.types.group
import fixture.*
import net.ccbluex.liquidbounce.features.module.ClientModule
open class Mode(val name:String){val tag get()=name;lateinit var base:ModeValueGroup<*>;open val parent:ModeValueGroup<*> get()=base;val running get()=parent.owner.running&&parent.activeMode===this;val player get()=fixture.player;val world get()=fixture.world;open fun enable(){};open fun disable(){};fun removeEventListenerScope(){};fun vec3i(key:String)=Value(net.minecraft.core.Vec3i.ZERO).also{values["$name/$key"]=it};fun text(key:String,default:String)=Value(default).also{values["$name/$key"]=it};fun float(key:String,default:Float,range:ClosedFloatingPointRange<Float>)=Value(default).also{values["$name/$key"]=it};fun message(key:String,vararg args:Any)="$key:"+args.joinToString(",")}
class ModeValueGroup<T:Mode>(val owner:ClientModule,var activeMode:T,val choices:Array<T>){init{choices.forEach{it.base=this}};fun setByString(name:String){setAndUpdate(choices.first{it.name==name})};fun set(value:MutableList<T>,apply:(MutableList<T>)->Unit){apply(value)}
'''+lifecycle+'\n}\n',encoding='utf-8')
 stubs={
 'Module.kt':'''package net.ccbluex.liquidbounce.features.module
import fixture.*
import net.ccbluex.liquidbounce.config.types.group.*
enum class ModuleCategories{PLAYER}
open class ClientModule(val name:String,val category:ModuleCategories){lateinit var group:ModeValueGroup<*>;var enabled=false;set(value){if(field==value)return;if(inGame)group.updateChildState(value);field=value};val running get()=enabled&&inGame&&parentRunning;fun<T:Mode>choices(name:String,active:T,choices:Array<T>)=ModeValueGroup(this,active,choices).also{group=it};fun tagBy(group:ModeValueGroup<*>){} }
''',
 'Handler.kt':'''package net.ccbluex.liquidbounce.event
import fixture.*
import net.ccbluex.liquidbounce.config.types.group.Mode
inline fun<reified E>Mode.handler(priority:Int=0,noinline action:(E)->Unit){handlers+=this to {event->action(event as E)}}
''',
 'Events.kt':'''package net.ccbluex.liquidbounce.event.events
data class DirectionalInput(val forwards:Boolean,val backwards:Boolean,val left:Boolean,val right:Boolean)
class MovementInputEvent(mask:Int){var directionalInput=DirectionalInput(mask and 64!=0,mask and 32!=0,mask and 16!=0,mask and 8!=0);var rest=mask and 7;fun mask()=(if(directionalInput.forwards)64 else 0)or(if(directionalInput.backwards)32 else 0)or(if(directionalInput.left)16 else 0)or(if(directionalInput.right)8 else 0)or rest}
class NotificationEvent{enum class Severity{SUCCESS}}
''',
 'Priority.kt':'''package net.ccbluex.liquidbounce.utils.kotlin
object EventPriorityConvention{const val CRITICAL_MODIFICATION=500}
''',
 'Rotation.kt':'''package net.ccbluex.liquidbounce.utils.aiming.data
class Rotation(val yaw:Float,val pitch:Float){val directionVector get()=net.minecraft.world.phys.Vec3.directionFromRotation(pitch,yaw)}
''',
 'Block.kt':'''package net.ccbluex.liquidbounce.utils.block
fun net.minecraft.core.Vec3i.toBlockPos()=net.minecraft.core.BlockPos(x,y,z)
''',
 'Agent.kt':'''package net.ccbluex.agent.v26_3
import fixture.*
import net.ccbluex.agent.pathfinder.terrain.Terrain
fun net.minecraft.core.BlockPos.toAgent()=net.ccbluex.agent.math.BlockPos(x,y,z)
fun Terrain.Companion.live(world:World)=LiveTerrain(world)
class LiveTerrain(val world:World){fun isStandable(pos:net.ccbluex.agent.math.BlockPos):Boolean{events+="T:${pos.x},${pos.y},${pos.z}";return world.stands[pos]?:false}}
''',
 'Notification.kt':'''package net.ccbluex.liquidbounce.utils.client
fun notification(name:String,message:String,severity:net.ccbluex.liquidbounce.event.events.NotificationEvent.Severity){fixture.events+="C:$message"}
''',
 'Navigator.kt':'''package net.ccbluex.liquidbounce.utils.navigation
import fixture.*
import net.ccbluex.liquidbounce.event.events.*
class Navigator(owner:Any,values:Any){fun halt(){events+="H"};fun reset(){events+="R"};fun navigate(event:MovementInputEvent,goal:net.minecraft.world.phys.Vec3,radius:Double){events+="N:${goal.raw()}:${radius.toRawBits().toULong()}";event.directionalInput=DirectionalInput(true,false,true,false);event.rest=6}}
'''}
 for name,code in stubs.items():(out/name).write_text(code,encoding='utf-8')
 m=math_source.read_text(encoding='utf-8');fma=m[m.index('fun Vec3.fma('):m.index('\n)',m.index('fun Vec3.fma('))+2];horizontal='\n'.join(line for line in m.splitlines()if line.startswith('fun Vec3.horizontalDistanceToSqr(other: Vec3):')or line.startswith('fun Vec3.horizontalDistanceToSqr(x:'))
 (out/'Math.kt').write_text('package net.ccbluex.liquidbounce.utils.math\nimport net.minecraft.world.phys.Vec3\nimport net.minecraft.util.Mth\nimport java.lang.Math.fma\n'+fma+'\n'+horizontal+'\n',encoding='utf-8')
 cache=Path(os.environ['USERPROFILE'])/'.gradle/caches';mc=cache/'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar';kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar'
 jars=[mc,binary,next(kotlin.glob('kotlin-stdlib-*.jar'))]
 for group in ['com.mojang','com.google.guava','com.google.code.gson','it.unimi.dsi','org.slf4j','org.joml','io.netty','org.apache.logging.log4j','org.apache.commons','org.lwjgl']:jars.extend((cache/'modules-2/files-2.1'/group).rglob('*.jar'))
 cp=os.pathsep.join(map(str,jars))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=out,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
  if r.returncode:print((r.stdout+r.stderr)[-12000:])
  return r
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-Ddirections='+str(out/'directions.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.MainKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');shutil.copyfile(HERE/'navigation-source-fixture/auto_walk.rs',out/'auto_walk_oracle.rs')
 (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;#[cfg(test)]mod auto_walk_oracle;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-auto-walk-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[original,mode_source,math_source,mc,binary,Path(__file__),HERE/'navigation-source-fixture/AutoWalk.kt',HERE/'navigation-source-fixture/auto_walk.rs',HERE/'navigation-source-fixture/JavaCaseFold.java',*sorted((ROOT/'native/liquidbounce-behaviors/src/navigation').glob('*.rs'))]
 report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)if p.is_relative_to(ROOT)else p):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'scope':'unchanged AutoWalk; extracted original mode lifecycle/math helpers; actual MC26.3 Vec3/BlockPos/Mth and Java String.equalsIgnoreCase','boundaries':['minimal framework/world/navigator shells; no real navigation, block physics or body','actual MC directionFromRotation outputs supplied to Rust host trig seam; route goals are not supplied','mode lifecycle functions are copied unchanged, surrounding configuration/event framework is a shell'],'fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
