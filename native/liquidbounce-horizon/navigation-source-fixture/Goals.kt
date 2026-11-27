@file:OptIn(net.ccbluex.agent.InternalAgentApi::class)
package fixture
import java.io.File
import net.ccbluex.agent.math.*
import net.ccbluex.agent.pathfinder.route.*
import net.ccbluex.agent.pathfinder.level.BlockRegion
import net.ccbluex.agent.pathfinder.terrain.Terrain
import net.ccbluex.agent.world.*
import net.ccbluex.agent.world.block.*

private fun d(raw:String)=Double.fromBits(raw.toULong().toLong())
private fun Vec3.raw()=listOf(x,y,z).joinToString(","){it.toRawBits().toULong().toString()}
private fun BlockRegion.raw()=listOf(minX,minY,minZ,maxX,maxY,maxZ).joinToString(",")
private fun pos(raw:String)=raw.split(',').let{BlockPos(it[0].toInt(),it[1].toInt(),it[2].toInt())}
private fun points(raw:String)=if(raw=="-")emptyList()else raw.split(';').map(::pos)
private class Known(val unknown:Set<BlockPos>,val border:Double):LevelBlocks{
 override fun isKnown(x:Int,y:Int,z:Int)=BlockPos(x,y,z)!in unknown
 override fun isInsideBorder(x:Double,z:Double)=Math.abs(x)<=border&&Math.abs(z)<=border
 override fun getBlockState(pos:BlockPos):BlockState=error("goal tests never query block states")
 override val minY=-64
 override val height=384
}
private fun defineVoid(){
 val state=object:BlockState(Blocks.VOID_AIR){
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
 Blocks.VOID_AIR.define(state,null,emptySet(),{false},0.6f,1f,1f,0f,0f)
}
fun main(){
 defineVoid()
 val results=File(System.getProperty("commands")).readLines().map{command->
  val p=command.split(' ')
  fun v(i:Int)=Vec3(d(p[i]),d(p[i+1]),d(p[i+2]))
  fun b(i:Int)=BlockPos(p[i].toInt(),p[i+1].toInt(),p[i+2].toInt())
  when(p[0]){
   "block"->{val point=b(1);val packed=point.asLong();val unpacked=BlockPos.of(packed);listOf(packed,unpacked.x,unpacked.y,unpacked.z,point.distSqr(p[4].toInt(),p[5].toInt(),p[6].toInt())).joinToString(",")}
   "vec"->{val a=v(1);val c=v(4);val contained=BlockPos.containing(a);"${a==c}|${a.distanceToSqr(c).toRawBits().toULong()}|${a.fma(d(p[7]),c).raw()}|${contained.x},${contained.y},${contained.z}"}
   "near"->{val goal=NearDestination(v(1),d(p[4]));val at=b(5);"${goal.isGoal(at)}|${goal.distance(at).toRawBits().toULong()}"}
   "blocks"->{val goal=BlockDestination(points(p[1]));val at=b(2);"${goal.isGoal(at)}|${goal.distance(at).toRawBits().toULong()}|"+goal.ends.joinToString(";"){it.raw()}}
   "region"->{val ends=if(p[6]=="-")emptyList()else p[6].split(';').map{it.split(',').let{a->Vec3(d(a[0]),d(a[1]),d(a[2]))}};BlockRegion.around(v(1),ends,d(p[4]),p[5].toInt()).raw()}
   "stage"->{val near=NearDestination(v(1),d(p[4]));val start=v(5);val range=d(p[8]);val at=b(9);val known=Known(points(p[12]).toSet(),d(p[13]));val goal=StageDestination(near,start,range,Terrain(known));"${goal.isGoal(at)}|${goal.distance(at).toRawBits().toULong()}|${StageDestination.isNeeded(known,start,near.center,range)}"}
   else->error(command)
  }
 }
 File(System.getProperty("results")).writeText(results.joinToString("\n")+"\n")
}
