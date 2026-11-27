@file:OptIn(net.ccbluex.agent.InternalAgentApi::class)
package fixture
import java.io.File
import java.lang.reflect.Proxy
import java.util.concurrent.Executor
import net.ccbluex.agent.pathfinder.route.*
import net.ccbluex.agent.world.block.*
import net.ccbluex.agent.math.*
import net.ccbluex.agent.navigation.*
import net.ccbluex.agent.navigation.flight.*
import net.ccbluex.agent.pathfinder.flight.*
import net.ccbluex.agent.world.*
import net.ccbluex.agent.world.entity.*

private fun d(raw:String)=Double.fromBits(raw.toULong().toLong())
private fun v(raw:String)=raw.split(',').let{Vec3(d(it[0]),d(it[1]),d(it[2]))}
private fun raw(value:Double)=value.toBits().toULong().toString()
private fun Vec3.raw()=listOf(x,y,z).joinToString(",",transform=::raw)
private fun AABB.raw()=listOf(minX,minY,minZ,maxX,maxY,maxZ).joinToString(",",transform=::raw)
private fun field(target:Any,name:String)=target.javaClass.getDeclaredField(name).also{it.isAccessible=true}.get(target)
private fun state(f:Follower,h:Handling):String {
 val r=field(f,"route")
 val route=if(r==null)"-" else listOf(field(r,"corner"),(field(r,"watch")as ProgressWatch).idleTicks,raw(field(r,"nearest")as Double),field(r,"missed"),f.path!!.start.raw(),f.path!!.points.joinToString(";"){it.raw()},f.path!!.canReach).joinToString("^")
 return listOf(field(f,"takeOffTicks"),field(f,"isLanding"),field(f,"hasLanded"),(field(f,"pressed")as Input?)?.hashCode()?:"-",f.target?.raw()?:"-",f.sprint,f.isFinished,f.isMoving,f.ahead.joinToString(";"){it.raw()},route,raw(h.drag),raw(h.verticalDrag),raw(h.speed),raw(h.verticalSpeed),raw(h.step),h.lastMove?.raw()?:"-").joinToString("|")
}

object Probe {
 val events=ArrayList<String>()
 var program=listOf("S1","F")
 var points=listOf(Vec3.ZERO,Vec3(8.0,0.0,0.0))
 var reachable=true
 fun raw(v:Double)=v.toBits().toULong().toString()
 fun vector(v:Vec3)=listOf(v.x,v.y,v.z).joinToString(",",transform=::raw)
}
private fun state(block:Block)=object:BlockState(block){
 override val isAir=true
 override val fluidState:FluidState get()=error("unused")
 override fun getCollisionShape(level:BlockGetter,pos:BlockPos):VoxelShape=error("unused")
 override fun getShape(level:BlockGetter,pos:BlockPos):VoxelShape=error("unused")
 override fun getDestroySpeed(level:BlockGetter,pos:BlockPos)=0f
 override fun canBeReplaced()=true
 override fun requiresCorrectToolForDrops()=false
 override fun hasBlockEntity()=false
 override fun isCollisionShapeFullBlock(level:BlockGetter,pos:BlockPos)=false
 override fun<T:Any>getOptionalValue(property:Property<T>):T?=null
 override fun<T:Any>setValue(property:Property<T>,value:T)=this
}
private val events=Probe.events
private class World{
 var blocked=false
 var present=true
 var known=true
 var failSnapshot=false
 val states=HashMap<BlockPos,Int>()
 val plain=state(Blocks.VOID_AIR)
 val changed=state(Blocks.STONE)
 init { Blocks.VOID_AIR.define(plain,null,emptySet(),{false},0.6f,1f,1f,0f,0f) }
 fun at(pos:BlockPos)=if(states[pos]==1)changed else plain
 val level=Proxy.newProxyInstance(Level::class.java.classLoader,arrayOf(Level::class.java)){_,method,args->
  when(method.name){
   "snapshot"->{val p=args!!.map{it as Int};events+="N:"+p.joinToString(",");if(failSnapshot){present=false;error("snapshot unavailable")};val copied=HashMap(states);val copiedKnown=known;object:LevelBlocks{
    override val minY=-64;override val height=384
    override fun isKnown(x:Int,y:Int,z:Int):Boolean{events+="L:$x,$y,$z";return copiedKnown&&y in -64..319}
    override fun isInsideBorder(x:Double,z:Double):Boolean{events+="D:${Probe.raw(x)},${Probe.raw(z)}";return true}
    override fun getBlockState(pos:BlockPos):BlockState{events+="B:${"${pos.x},${pos.y},${pos.z}"}";return if(copied[pos]==1)changed else plain}
   }}
   "noBlockCollision"->{events+="C:${(args!![1]as AABB).raw()}";!blocked}
   "getMinY"->-64
   "getMaxY"->319
   "getHeight"->384
   "hasChunkAt"->known
   "getBlockState"->{val p=args!![0]as BlockPos;events+="I:${"${p.x},${p.y},${p.z}"}";at(p)}
   else->error("unused Level.${method.name}")
  }
 }as Level
}
private class Queue:Executor{val tasks=ArrayDeque<Runnable>();override fun execute(command:Runnable){tasks+=command};fun run(){tasks.removeFirstOrNull()?.run()}}

fun main(){
 var position=Vec3.ZERO;var delta=Vec3.ZERO;var tick=0;var yaw=0f;var sprint=false;var horizontal=false;var vertical=false;var ground=false;var creative=false;var mayfly=false;var flyingSpeed=.05f;var host=false
 val events=Probe.events
 var world=World();var queue=Queue()
 val abilities=Proxy.newProxyInstance(Abilities::class.java.classLoader,arrayOf(Abilities::class.java)){_,method,_->when(method.name){"getFlying"->{events+="af";creative};"getMayfly"->{events+="am";mayfly};"getFlyingSpeed"->{events+="as";flyingSpeed};else->error(method.name)}}as Abilities
 val player=Proxy.newProxyInstance(Player::class.java.classLoader,arrayOf(Player::class.java)){_,method,args->when(method.name){
  "position"->{events+="p";position};"getDeltaMovement"->{events+="d";delta};"getTickCount"->{events+="t";tick};"getHorizontalCollision"->{events+="h";horizontal};"getVerticalCollision"->{events+="v";vertical};"getYRot"->{events+="y";yaw};"isSprinting"->{events+="s";sprint};"getAbilities"->{events+="a";abilities};"getY"->{events+="Y";position.y};"onGround"->{events+="g";ground};"getBoundingBox"->{events+="B";AABB(position.x-.3,position.y,position.z-.3,position.x+.3,position.y+1.8,position.z+.3)};"getPosition"->{val alpha=args!![0]as Float;events+="i:${alpha.toRawBits().toUInt()}";position.add(delta.scale(alpha.toDouble()))};else->error("Player.${method.name}")
 }}as Player
 val body=Proxy.newProxyInstance(Body::class.java.classLoader,arrayOf(Body::class.java)){_,method,args->when(method.name){"getPlayer"->{events+="P";player};"getLevel"->if(world.present)world.level else null;"isFlying"->{events+="H";host};"press"->{events+="K:${(args!![0]as Input).hashCode()}:${args[1]}";Unit};else->error("Body.${method.name}")}}as Body

 val context=NavigationContext(body,FlightReport())
 fun create():Flying {
  val flying=Flying(context)
  flying.javaClass.getDeclaredField("routes").also{it.isAccessible=true}.set(flying,Replanner<Route>({if(world.present)world.level else null},32,executor=queue))
  return flying
 }
 var flying=create();var current=FlyAbilities(Acceleration.DIRECT,1.0,.4,true)
 val results=File(System.getProperty("commands")).readLines().map{command->
  val a=command.split(' ')
  when(a[0]){
   "new"->{world=World();queue=Queue();flying=create()}
   "base"->current=FlyAbilities(Acceleration.valueOf(a[1]),d(a[2]),d(a[3]),a[4].toBoolean())
   "player"->{tick=a[1].toInt();position=v(a[2]);delta=v(a[3]);yaw=Float.fromBits(a[4].toUInt().toInt());sprint=a[5].toBoolean();horizontal=a[6].toBoolean();vertical=a[7].toBoolean();ground=a[8].toBoolean();creative=a[9].toBoolean();mayfly=a[10].toBoolean();flyingSpeed=Float.fromBits(a[11].toUInt().toInt())}
   "program"->{Probe.program=a[1].split(',');Probe.points=a[2].split(';').map(::v);Probe.reachable=a[3].toBoolean()}
   "world"->world.present=a[1].toBoolean()
   "known"->world.known=a[1].toBoolean()
   "state"->{val q=a[1].split(',').map(String::toInt);world.states[BlockPos(q[0],q[1],q[2])]=a[2].toInt()}
   "collision"->world.blocked=a[1].toBoolean()
   "run"->queue.run()
   "nav"->flying.navigate(v(a[1]),d(a[2]),d(a[3]),current,Float.fromBits(a[4].toUInt().toInt()))
   "halt"->flying.halt()
   "level"->flying.levelChanged()
   "change"->{val q=a[1].split(',').map(String::toInt);flying.blockChanged(BlockPos(q[0],q[1],q[2]))}
   else->error(command)
  }
  val routes=field(flying,"routes")as Replanner<*>
  val follower=field(flying,"follower")as Follower
  val status=listOf(flying.isActive,field(flying,"stuck"),routes.ticks,routes.failed,routes.endsAtStage,routes.isSearching,routes.isDue(false),routes.isDue(true),flying.target?.raw()?:"-",flying.sprint).joinToString(",")
  val result="$status|${state(follower,flying.handling)}|${routes.preview?.joinToString(";"){it.raw()}?:"-"}|${queue.tasks.size}|${events.joinToString("/")}"
  events.clear();result
 }
 File(System.getProperty("results")).writeText(results.joinToString("\n")+"\n")
}
