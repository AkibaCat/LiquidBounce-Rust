package net.ccbluex.agent.simulation
import java.io.File
import net.ccbluex.agent.math.*
import net.ccbluex.agent.world.block.*
import net.ccbluex.agent.world.Mechanics

private fun d(v:String)=Double.fromBits(v.toULong().toLong())
private fun f(v:String)=Float.fromBits(v.toUInt().toInt())
private fun v(raw:String)=raw.split(',').let{Vec3(d(it[0]),d(it[1]),d(it[2]))}
private fun p(raw:String)=raw.split(',').let{BlockPos(it[0].toInt(),it[1].toInt(),it[2].toInt())}
private fun b(raw:String)=raw.split(',').map(::d).let{AABB(it[0],it[1],it[2],it[3],it[4],it[5])}
private fun Double.raw()=toRawBits().toULong().toString()
private fun Float.raw()=toRawBits().toUInt().toString()
private fun Vec3.raw()=listOf(x,y,z).joinToString(","){it.raw()}
private val events=ArrayList<String>()
private val shapes=HashMap<Int,VoxelShape>()
private class World(val floor:Int,val floorShape:Int):BlockGrid{
 val blocks=HashMap<BlockPos,Int>()
 override fun shape(x:Int,y:Int,z:Int):VoxelShape{events+="S:$x,$y,$z";return shapes.getValue(blocks[BlockPos(x,y,z)]?:if(y==floor)floorShape else 0)}
 override fun state(x:Int,y:Int,z:Int):BlockState=error("state not used by collision source fixture")
}
fun main(){
 var world=World(0,1);var motion=PlayerMotion(world);var support=Support();var edge=EdgeBackOff()
 val results=File(System.getProperty("commands")).readLines().map{command->
  val a=command.split(' ');var result="-"
  fun position(raw:String){val pos=v(raw);motion.x=pos.x;motion.y=pos.y;motion.z=pos.z}
  when(a[0]){
   "shape"->shapes[a[1].toInt()]=when(a[2]){"E"->Shapes.empty();"F"->Shapes.block();else->VoxelShape(a[2].split(',').map(::d).toDoubleArray())}
   "new"->{world=World(a[1].toInt(),a[2].toInt());motion=PlayerMotion(world);support=Support();edge=EdgeBackOff()}
   "set"->world.blocks[p(a[1])]=a[2].toInt()
   "intersects"->{val pos=p(a[1]);result=motion.colliders.intersects(pos.x,pos.y,pos.z,b(a[2])).toString()}
   "footing"->{val pos=p(a[1]);result=motion.colliders.footing(pos.x,pos.y,pos.z,d(a[2]),d(a[3]),d(a[4])).raw()}
   "collect"->motion.colliders.collect(b(a[1]))
   "move"->{val delta=v(a[1]);val body=b(a[2]);motion.colliders.collide(delta.x,delta.y,delta.z,body.minX,body.minY,body.minZ,body.maxX,body.maxY,body.maxZ);result=Vec3(motion.colliders.movedX,motion.colliders.movedY,motion.colliders.movedZ).raw()}
   "offset"->result=motion.colliders.offset(a[1].toInt(),b(a[2]),d(a[3])).raw()
   "steps"->result=motion.colliders.stepHeights(b(a[1]),f(a[2]),f(a[3])).joinToString(","){it.raw()}
   "collide"->{position(a[1]);motion.onGround=a[2].toBoolean();val delta=v(a[3]);result=motion.collide(delta.x,delta.y,delta.z).raw()}
   "fall"->{position(a[1]);result=motion.canFall(d(a[2]),d(a[3]),d(a[4])).toString()}
   "support"->{val pos=v(a[1]);support.find(motion.colliders,pos.x,pos.y,pos.z,d(a[2]),d(a[3]));result="${support.exists},${support.x},${support.y},${support.z}"}
   "clear"->{support.clear();result="${support.exists},${support.x},${support.y},${support.z}"}
   "edge"->{position(a[1]);edge.backOff(motion,d(a[2]),d(a[3]));result="${edge.x.raw()},${edge.z.raw()}"}
   "traits"->{val t=MotionTraits(f(a[1]),f(a[2]),f(a[3]),if(a[4]=="modern")Mechanics.MODERN else Mechanics.V1_8);result="${t.walkingSpeed.raw()},${t.jumpStrength.raw()},${t.jumpBoostPower.raw()},${t.jumpPower.raw()},${t.jumpHeight.raw()},${t.mechanics.maxPacketsPerTick},${t.mechanics.isMoveJudgedPerTick}"}
   "rise"->result=MotionTraits.rise(d(a[1])).raw()
   else->error(command)
  }
  val output=result+"|"+events.joinToString("/");events.clear();output
 }
 File(System.getProperty("results")).writeText(results.joinToString("\n")+"\n")
}
