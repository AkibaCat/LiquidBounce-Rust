@file:OptIn(net.ccbluex.agent.InternalAgentApi::class)
package fixture
import java.io.File
import java.lang.reflect.Proxy
import java.util.concurrent.Executor
import net.ccbluex.agent.math.*
import net.ccbluex.agent.pathfinder.route.*
import net.ccbluex.agent.pathfinder.level.BlockRegion
import net.ccbluex.agent.pathfinder.terrain.Terrain
import net.ccbluex.agent.world.*
import net.ccbluex.agent.world.block.*

private val events=ArrayList<String>()
private fun pos(raw:String)=raw.split(',').let{BlockPos(it[0].toInt(),it[1].toInt(),it[2].toInt())}
private fun BlockPos.raw()="$x,$y,$z"
private fun Vec3.raw()=listOf(x,y,z).joinToString(","){it.toRawBits().toULong().toString()}
private fun BlockRegion.raw()=listOf(minX,minY,minZ,maxX,maxY,maxZ).joinToString(",")
private fun vec(raw:String)=raw.split(',').let{Vec3(Double.fromBits(it[0].toULong().toLong()),Double.fromBits(it[1].toULong().toLong()),Double.fromBits(it[2].toULong().toLong()))}
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
private class World{
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
    override fun isKnown(x:Int,y:Int,z:Int)=copiedKnown
    override fun isInsideBorder(x:Double,z:Double)=true
    override fun getBlockState(pos:BlockPos):BlockState{events+="B:${pos.raw()}";return if(copied[pos]==1)changed else plain}
   }}
   "getMinY"->-64
   "getMaxY"->319
   "getHeight"->384
   "hasChunkAt"->known
   "getBlockState"->{val p=args!![0]as BlockPos;events+="I:${p.raw()}";at(p)}
   else->error("unused Level.${method.name}")
  }
 }as Level
}
private class Queue:Executor{val tasks=ArrayDeque<Runnable>();override fun execute(command:Runnable){tasks+=command};fun run(){tasks.removeFirstOrNull()?.run()}}
private class S(val terrain:Terrain,val program:List<String>):Search<Int>{
 var step=0
 init{terrain.state(0,0,0)}
 override fun advance():Outcome<Int>{val token=program.getOrElse(step){program.last()};events+="A:$step";step++;return when(token[0]){'S'->Outcome.Searching;'F'->Outcome.Found(token.substring(1).toInt());'E'->Outcome.Failed(Failure.valueOf(token.substring(1)));'A'->Outcome.AlreadyThere;else->error("fixture advance failure")}}
 override fun preview():List<Vec3>?{events+="P:$step";return if(program[(step-1).coerceAtLeast(0).coerceAtMost(program.lastIndex)]=="S1")listOf(Vec3(step.toDouble(),0.0,0.0))else null}
}
private fun outcome(value:Outcome<Int>):String=when(value){Outcome.Searching->"S";Outcome.AlreadyThere->"A";is Outcome.Found->"F:${value.value}";is Outcome.Failed->"E:${value.failure}"}
private fun field(target:Any,name:String)=target.javaClass.getDeclaredField(name).also{it.isAccessible=true}
fun main(){
 var world=World();var queue=Queue();var mode="bg";var bg=BackgroundSearch<Int>({if(world.present)world.level else null},executor=queue);var rp=Replanner<Int>({if(world.present)world.level else null},8,executor=queue)
 val results=File(System.getProperty("commands")).readLines().map{command->
  val p=command.split(' ');var returned="-"
  when(p[0]){
   "new"->{world=World();queue=Queue();mode=p[1];bg=BackgroundSearch({if(world.present)world.level else null},executor=queue);rp=Replanner({if(world.present)world.level else null},p[2].toInt(),executor=queue)}
   "world"->world.present=p[1].toBoolean()
   "known"->world.known=p[1].toBoolean()
   "state"->world.states[pos(p[1])]=p[2].toInt()
   "request"->{val key=p[1].toInt();val a=p[2].split(',').map(String::toInt);val program=p[3].split(',');bg.request(key,BlockRegion(a[0],a[1],a[2],a[3],a[4],a[5])){events+="F:$key";S(it,program)}}
   "req"->{val start=vec(p[1]);val goal=NearDestination(vec(p[2]),Double.fromBits(p[3].toULong().toLong()));val length=Double.fromBits(p[4].toULong().toLong());val range=Double.fromBits(p[5].toULong().toLong());val variant=p[6];val program=p[7].split(',');rp.request(start,goal,length,variant,range){terrain,destination->events+="F:"+destination.javaClass.simpleName+":"+listOf(BlockPos(0,0,0),BlockPos(4,0,0),BlockPos(100,0,0)).joinToString(";"){"${destination.isGoal(it)},${destination.distance(it).toRawBits().toULong()}"};S(terrain,program)}}
   "poll"->returned=outcome(if(mode=="bg")bg.poll()else rp.poll())
   "run"->queue.run()
   "tick"->repeat(p[1].toInt()){rp.tick()}
   "change"->if(mode=="bg")bg.blockChanged(pos(p[1]))else rp.blockChanged(pos(p[1]))
   "reset"->if(mode=="bg")bg.cancel()else rp.reset()
   "level"->if(mode=="bg")bg.levelChanged()else rp.levelChanged()
   "end"->rp.routeEnds(pos(p[1]))
   "forget"->rp.forget()
   "query"->{val goal=vec(p[1]);returned="${rp.hasGoalMoved(goal)},${rp.isNewGoal(goal)}"}
   "seed"->{field(rp,"ticks").setInt(rp,p[1].toInt());field(rp,"lastRequest").setInt(rp,p[2].toInt());field(rp,"retryAt").setInt(rp,p[3].toInt())}
   else->error(command)
  }
  val preview=(if(mode=="bg")bg.preview else rp.preview)?.joinToString(";"){it.raw()}?:"-"
  val status=if(mode=="bg")bg.isIdle.toString()else listOf(rp.ticks,rp.failed,rp.endsAtStage,rp.isSearching,rp.isDue(false),rp.isDue(true)).joinToString(",")
  val result="$returned|$status|$preview|${queue.tasks.size}|"+events.joinToString("/");events.clear();result
 }
 File(System.getProperty("results")).writeText(results.joinToString("\n")+"\n")
}
