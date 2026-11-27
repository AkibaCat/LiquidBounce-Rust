@file:OptIn(net.ccbluex.agent.InternalAgentApi::class)
package fixture
import java.io.File
import net.ccbluex.liquidbounce.features.module.modules.player.ModuleAutoWalk as M
import net.ccbluex.liquidbounce.event.events.MovementInputEvent
import net.minecraft.core.*
import net.minecraft.world.phys.Vec3

val events=ArrayList<String>()
val handlers=ArrayList<Pair<net.ccbluex.liquidbounce.config.types.group.Mode,(MovementInputEvent)->Unit>>()
val values=HashMap<String,Value<*>>()
var inGame=true
var parentRunning=true
class Value<T>(var value:T){operator fun getValue(receiver:Any?,property:kotlin.reflect.KProperty<*>)=value;operator fun setValue(receiver:Any?,property:kotlin.reflect.KProperty<*>,v:T){value=v}}
class Profile(val name:String)
class Player(val gameProfile:Profile,var location:Vec3=Vec3.ZERO){var yRot=0f;var rootVehicle:Player=this;fun position()=location}
val player=Player(Profile("local"))
val vehicle=Player(Profile("vehicle"))
class World{var chunk=true;val stands=HashMap<net.ccbluex.agent.math.BlockPos,Boolean>();val others=ArrayList<Player>();fun hasChunkAt(x:Int,z:Int)=chunk;fun players()=listOf(player)+others}
val world=World()
fun bytes(raw:String)=if(raw=="-")""else String(raw.chunked(2).map{it.toInt(16).toByte()}.toByteArray(),Charsets.UTF_8)
fun hex(value:String)=value.toByteArray(Charsets.UTF_8).joinToString(""){"%02x".format(it)}.ifEmpty{"-"}
fun vec(raw:String)=raw.split(',').let{Vec3(Double.fromBits(it[0].toULong().toLong()),Double.fromBits(it[1].toULong().toLong()),Double.fromBits(it[2].toULong().toLong()))}
fun Vec3.raw()=listOf(x,y,z).joinToString(","){it.toRawBits().toULong().toString()}
private fun defineVoid(){
 val state=object:net.ccbluex.agent.world.block.BlockState(net.ccbluex.agent.world.block.Blocks.VOID_AIR){
  override val isAir=true
  override val fluidState:net.ccbluex.agent.world.block.FluidState get()=error("unused")
  override fun getCollisionShape(level:net.ccbluex.agent.world.BlockGetter,pos:net.ccbluex.agent.math.BlockPos):net.ccbluex.agent.world.block.VoxelShape=error("unused")
  override fun getShape(level:net.ccbluex.agent.world.BlockGetter,pos:net.ccbluex.agent.math.BlockPos):net.ccbluex.agent.world.block.VoxelShape=error("unused")
  override fun getDestroySpeed(level:net.ccbluex.agent.world.BlockGetter,pos:net.ccbluex.agent.math.BlockPos)=0f
  override fun canBeReplaced()=true
  override fun requiresCorrectToolForDrops()=false
  override fun hasBlockEntity()=false
  override fun isCollisionShapeFullBlock(level:net.ccbluex.agent.world.BlockGetter,pos:net.ccbluex.agent.math.BlockPos)=false
  override fun<T:Any>getOptionalValue(property:net.ccbluex.agent.world.block.Property<T>):T?=null
  override fun<T:Any>setValue(property:net.ccbluex.agent.world.block.Property<T>,value:T)=this
 }
 net.ccbluex.agent.world.block.Blocks.VOID_AIR.define(state,null,emptySet(),{false},0.6f,1f,1f,0f,0f)
}
fun main(){
 net.minecraft.SharedConstants.tryDetectVersion()
 defineVoid()
 val directions=ArrayList<String>()
 val results=File(System.getProperty("commands")).readLines().map{command->
  val p=command.split(' ');var result="-"
  @Suppress("UNCHECKED_CAST")fun<T>value(key:String)=values[key]as Value<T>
  when(p[0]){
   "enabled"->M.enabled=p[1].toBoolean()
   "mode"->M.group.setByString(p[1])
   "input"->{val event=MovementInputEvent(p[1].toInt());if(M.running)for((owner,handler)in handlers)if(owner.running)handler(event);result=event.mask().toString()}
   "position"->player.location=vec(p[1])
   "vehicle"->{if(p[1]=="self")player.rootVehicle=player else{vehicle.location=vec(p[1]);player.rootVehicle=vehicle}}
   "yaw"->player.yRot=Float.fromBits(p[1].toUInt().toInt())
   "dest"->{val a=p[1].split(',').map(String::toInt);value<Vec3i>("Goto/Destination").value=Vec3i(a[0],a[1],a[2])}
   "stand"->{val a=p[1].split(',').map(String::toInt);world.stands[net.ccbluex.agent.math.BlockPos(a[0],a[1],a[2])]=p[2].toBoolean()}
   "chunk"->world.chunk=p[1].toBoolean()
   "target"->value<String>("Follow/Player").value=bytes(p[1])
   "distance"->value<Float>("Follow/Distance").value=Float.fromBits(p[1].toUInt().toInt())
   "players"->{world.others.clear();if(p[1]!="-")for(raw in p[1].split(';')){val a=raw.split(':');world.others+=Player(Profile(bytes(a[0])),vec(a[1]))}}
   "goto"->M.goTo(vec(p[1]))
   "follow"->M.follow(bytes(p[1]))
   "ingame"->inGame=p[1].toBoolean()
   "parent"->parentRunning=p[1].toBoolean()
   "disable"->M.group.updateChildState(false)
   "case"->result=bytes(p[1]).equals(bytes(p[2]),true).toString()
   else->error(command)
  }
  directions+=Vec3.directionFromRotation(0f,player.yRot).raw()
  val dest=value<Vec3i>("Goto/Destination").value
  val state=listOf(M.enabled,M.group.activeMode.name,"${dest.x},${dest.y},${dest.z}",hex(value<String>("Follow/Player").value),value<Float>("Follow/Distance").value.toRawBits().toUInt()).joinToString("|")
  val output="$result|$state|"+events.joinToString("/");events.clear();output
 }
 File(System.getProperty("directions")).writeText(directions.joinToString("\n")+"\n")
 File(System.getProperty("results")).writeText(results.joinToString("\n")+"\n")
}
