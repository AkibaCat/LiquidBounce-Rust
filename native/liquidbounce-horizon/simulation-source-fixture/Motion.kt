@file:OptIn(net.ccbluex.agent.InternalAgentApi::class)
package net.ccbluex.agent.simulation
import java.io.File
import net.ccbluex.agent.math.*
import net.ccbluex.agent.world.*
import net.ccbluex.agent.world.block.*
import net.ccbluex.agent.world.entity.Player
import java.lang.reflect.Proxy

private fun d(v:String)=Double.fromBits(v.toULong().toLong())
private fun f(v:String)=Float.fromBits(v.toUInt().toInt())
private fun v(raw:String)=raw.split(',').let{Vec3(d(it[0]),d(it[1]),d(it[2]))}
private fun p(raw:String)=raw.split(',').let{BlockPos(it[0].toInt(),it[1].toInt(),it[2].toInt())}
private fun Double.raw()=toRawBits().toULong().toString()
private fun Float.raw()=toRawBits().toUInt().toString()
private fun Vec3.raw()=listOf(x,y,z).joinToString(","){it.raw()}
private val events=ArrayList<String>()
private val shapes=HashMap<Int,VoxelShape>()
private val states=HashMap<Int,S>()
private val kinds=listOf(BlockKind.SLIME,BlockKind.FENCE,BlockKind.WALL,BlockKind.FENCE_GATE)
private class S(val id:Int,block:Block,val fluid:Boolean,val air:Boolean,val shape:Int):BlockState(block){
 override val isAir get()=air
 override val fluidState get()=if(fluid)FluidState(Fluids.WATER,true,8)else FluidState.EMPTY
 override fun getCollisionShape(level:BlockGetter,pos:BlockPos)=shapes.getValue(shape)
 override fun getShape(level:BlockGetter,pos:BlockPos)=getCollisionShape(level,pos)
 override fun getDestroySpeed(level:BlockGetter,pos:BlockPos)=0f
 override fun canBeReplaced()=isAir
 override fun requiresCorrectToolForDrops()=false
 override fun hasBlockEntity()=false
 override fun isCollisionShapeFullBlock(level:BlockGetter,pos:BlockPos)=getCollisionShape(level,pos)===Shapes.block()
 override fun<T:Any>getOptionalValue(property:Property<T>):T?=null
 override fun<T:Any>setValue(property:Property<T>,value:T):BlockState=this
}
private class World(val floor:Int,val floorState:Int):BlockGrid{
 val blocks=HashMap<BlockPos,Int>()
 private fun id(x:Int,y:Int,z:Int)=blocks[BlockPos(x,y,z)]?:if(y==floor)floorState else 0
 override fun shape(x:Int,y:Int,z:Int):VoxelShape{events+="S:$x,$y,$z";return shapes.getValue(states.getValue(id(x,y,z)).shape)}
 override fun state(x:Int,y:Int,z:Int):BlockState{events+="B:$x,$y,$z";return states.getValue(id(x,y,z))}
}
private val groundWithout=PlayerMotion::class.java.getDeclaredField("groundWithoutBlocks").apply{isAccessible=true}
private fun snapshot(m:PlayerMotion):String=listOf(
 m.position.raw(),m.deltaMovement.raw(),m.onGround,m.horizontalCollision,m.isSprinting,m.landingFall.raw(),m.peakY.raw(),m.jumpDelay,
 "${m.hasSupport},${m.supportX},${m.supportY},${m.supportZ}",groundWithout.getBoolean(m),m.crouching,m.sneaking,m.airSprint,m.walked.raw(),m.nextStep,
 "${m.blockPosition.x},${m.blockPosition.y},${m.blockPosition.z}",m.horizontalSpeed.raw(),
 "${m.traits.walkingSpeed.raw()},${m.traits.jumpStrength.raw()},${m.traits.jumpBoostPower.raw()},${m.traits.mechanics.name}"
).joinToString(";")
fun main(){
 lateinit var world:World;lateinit var motion:PlayerMotion;var other:PlayerMotion?=null
 val results=File(System.getProperty("commands")).readLines().map{command->val a=command.split(' ');var result="-"
  when(a[0]){
   "shape"->shapes[a[1].toInt()]=when(a[2]){"E"->Shapes.empty();"F"->Shapes.block();else->VoxelShape(a[2].split(',').map(::d).toDoubleArray())}
   "define"->{val block=Block.of(a[2]);val flags=a[3].toInt();val state=S(a[1].toInt(),block,a[4].toBoolean(),a[5].toBoolean(),a[6].toInt());states[state.id]=state
    block.define(state,null,kinds.filterIndexed{i,_->flags and(1 shl i)!=0}.toSet(),{false},f(a[7]),f(a[8]),f(a[9]),f(a[10]),f(a[11]))}
   "new"->{world=World(a[1].toInt(),a[2].toInt());motion=PlayerMotion(world,MotionTraits(f(a[4]),f(a[5]),f(a[6]),if(a[3]=="modern")Mechanics.MODERN else Mechanics.V1_8));other=null}
   "set"->world.blocks[p(a[1])]=a[2].toInt()
   "reset"->motion.reset(v(a[1]),v(a[2]),a[3].toBoolean(),a[4].toBoolean(),a[5].toInt())
   "resetPlayer"->{
    val player=Proxy.newProxyInstance(Player::class.java.classLoader,arrayOf(Player::class.java)){_,method,_->
     events+="P:${method.name}"
     when(method.name){"position"->v(a[1]);"getDeltaMovement"->v(a[2]);"onGround"->a[3].toBoolean();"isSprinting"->a[4].toBoolean();"getNoJumpDelay"->a[5].toInt();"isCrouching"->a[6].toBoolean();"getMainSupportingBlockPos"->if(a[7]=="-")null else p(a[7]);else->error(method.name)}
    }as Player
    motion.reset(player)
   }
   "tick"->motion.tick(f(a[1]),a[2].toInt(),a[3].toInt(),a[4].toBoolean(),a[5].toBoolean(),a[6].toBoolean())
   "copy"->{other=motion;motion=motion.copy()}
   "swap"->{val saved=motion;motion=other!!;other=saved}
   "copyFrom"->motion.copyFrom(other!!)
   "fields"->{motion.crouching=a[1].toBoolean();motion.sneaking=a[2].toBoolean();motion.landingFall=d(a[3]);motion.peakY=d(a[4]);motion.jumpDelay=a[5].toInt()}
   "footing"->{val pos=p(a[1]);result=motion.footing(pos.x,pos.y,pos.z).raw()}
   "stands"->{val pos=p(a[1]);result=motion.standsOn(pos.x,pos.y,pos.z).toString()}
   else->error(command)
  }
  val state=if(a[0]=="shape"||a[0]=="define")"-"else snapshot(motion)
  val output="$result|$state|"+events.joinToString("/");events.clear();output
 }
 File(System.getProperty("results")).writeText(results.joinToString("\n")+"\n")
}
