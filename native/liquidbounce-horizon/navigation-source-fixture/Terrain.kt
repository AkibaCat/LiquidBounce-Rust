@file:OptIn(net.ccbluex.agent.InternalAgentApi::class)
package fixture
import java.io.File
import net.ccbluex.agent.math.*
import net.ccbluex.agent.pathfinder.terrain.*
import net.ccbluex.agent.world.*
import net.ccbluex.agent.world.block.*

private fun d(raw:String)=Double.fromBits(raw.toULong().toLong())
private fun v(raw:String)=raw.split(',').let{Vec3(d(it[0]),d(it[1]),d(it[2]))}
private fun p(raw:String)=raw.split(',').let{BlockPos(it[0].toInt(),it[1].toInt(),it[2].toInt())}
private fun b(raw:String)=raw.split(',').map(::d).let{AABB(it[0],it[1],it[2],it[3],it[4],it[5])}
private fun Double.raw()=toRawBits().toULong().toString()
private fun Vec3.raw()=listOf(x,y,z).joinToString(","){it.raw()}
private fun AABB.raw()=listOf(minX,minY,minZ,maxX,maxY,maxZ).joinToString(","){it.raw()}
private fun BlockPos.raw()="$x,$y,$z"
private fun VoxelShape.raw()=if(this===Shapes.block())"F"else if(this===Shapes.empty())"E"else "S:"+toArray().joinToString(","){it.raw()}
private val events=ArrayList<String>()
private val shapes=HashMap<Int,VoxelShape>()
private val states=HashMap<Int,S>()
private val kinds=listOf(BlockKind.DOOR,BlockKind.OPENS_BY_HAND,BlockKind.FENCE_GATE,BlockKind.TRAPDOOR,BlockKind.LADDER,BlockKind.FIRE)
private class S(val id:Int,block:Block,val fluid:Int,val opened:Int,val upper:Int,val facing:Int,val normal:Int,val openedShape:Int,val paired:Int,val neighbor:Int):BlockState(block){
 override val isAir get()=block===Blocks.AIR||block===Blocks.VOID_AIR
 override val fluidState get()=when(fluid){1->FluidState(Fluids.WATER,true,8);2->FluidState(Fluids.LAVA,true,8);else->FluidState.EMPTY}
 override fun getCollisionShape(level:BlockGetter,pos:BlockPos):VoxelShape{
  events+="C:$id:${pos.raw()}"
  val key=if(neighbor>=0&&!level.getBlockState(pos.offset(1,0,0)).isAir)neighbor else if(opened==1)openedShape else normal
  return shapes.getValue(key)
 }
 override fun getShape(level:BlockGetter,pos:BlockPos)=getCollisionShape(level,pos)
 override fun getDestroySpeed(level:BlockGetter,pos:BlockPos)=0f
 override fun canBeReplaced()=isAir
 override fun requiresCorrectToolForDrops()=false
 override fun hasBlockEntity()=false
 override fun isCollisionShapeFullBlock(level:BlockGetter,pos:BlockPos)=getCollisionShape(level,pos)===Shapes.block()
 @Suppress("UNCHECKED_CAST")
 override fun<T:Any>getOptionalValue(property:Property<T>):T?=when(property){
  BlockStateProperties.OPEN->if(opened<0)null else opened==1
  BlockStateProperties.DOUBLE_BLOCK_HALF->when(upper){0->DoubleBlockHalf.LOWER;1->DoubleBlockHalf.UPPER;else->null}
  BlockStateProperties.HORIZONTAL_FACING->if(facing<0)null else Direction.entries[facing]
  else->null
 }as T?
 override fun<T:Any>setValue(property:Property<T>,value:T):BlockState=if(property===BlockStateProperties.OPEN&&opened>=0&&value!= (opened==1)&&paired>=0)states.getValue(paired)else this
}
private class World:LevelBlocks{
 var min=-64;var count=384;var borderMin=-100.0;var borderMax=100.0;var legacy=false;var floorY=-1;var floorState=1
 val blocks=HashMap<BlockPos,Int>();val unknown=HashSet<BlockPos>()
 override val minY get()=min
 override val height get()=count
 override val mechanics get()=if(legacy)Mechanics.V1_8 else Mechanics.MODERN
 override fun getBlockState(pos:BlockPos):BlockState{events+="B:${pos.raw()}";return states.getValue(blocks[pos]?:if(pos.y==floorY)floorState else 0)}
 override fun isKnown(x:Int,y:Int,z:Int):Boolean{events+="K:$x,$y,$z";return y>=minY&&y<=maxY&&BlockPos(x,y,z)!in unknown}
 override fun isInsideBorder(x:Double,z:Double):Boolean{events+="D:${x.raw()},${z.raw()}";return x>=borderMin&&x<=borderMax&&z>=borderMin&&z<=borderMax}
}
fun main(){
 var world=World();lateinit var terrain:Terrain
 val results=File(System.getProperty("commands")).readLines().map{command->val a=command.split(' ');var result="-"
  when(a[0]){
   "shapeDef"->{shapes[a[1].toInt()]=when(a[2]){"F"->Shapes.block();"E"->Shapes.empty();"-"->VoxelShape(DoubleArray(0));else->VoxelShape(a[2].split(',').map(::d).toDoubleArray())}}
   "stateDef"->{val id=a[1].toInt();val block=Block.of(a[2]);val mask=a[3].toInt();val state=S(id,block,a[4].toInt(),a[5].toInt(),a[6].toInt(),a[7].toInt(),a[9].toInt(),a[10].toInt(),a[11].toInt(),a[12].toInt());states[id]=state
    block.define(state,null,kinds.filterIndexed{i,_->mask and(1 shl i)!=0}.toSet(),{false},.6f,1f,1f,Float.fromBits(a[8].toUInt().toInt()),0f)
   }
   "new"->{world=World().also{it.min=a[4].toInt();it.count=a[5].toInt();it.borderMin=d(a[6]);it.borderMax=d(a[7]);it.legacy=a[8].toBoolean();it.floorY=a[9].toInt();it.floorState=a[10].toInt()};terrain=Terrain(world,d(a[2]),d(a[3]),a[1].toBoolean())}
   "set"->world.blocks[p(a[1])]=a[2].toInt()
   "known"->{if(a[2].toBoolean())world.unknown.remove(p(a[1]))else world.unknown.add(p(a[1]))}
   "border"->{world.borderMin=d(a[1]);world.borderMax=d(a[2])}
   "state"->{val q=p(a[1]);result=(terrain.state(q.x,q.y,q.z)as S).id.toString()}
   "shape"->{val q=p(a[1]);result=terrain.shape(q.x,q.y,q.z).raw()}
   "flags"->{val q=p(a[1]);result=listOf(terrain.isKnown(q.x,q.y,q.z),terrain.isPassable(q.x,q.y,q.z),terrain.isHazard(q.x,q.y,q.z),terrain.isWater(q.x,q.y,q.z),terrain.isClimbable(q.x,q.y,q.z),terrain.climbSide(q.x,q.y,q.z)?.name?:"-").joinToString(",")}
   "stand"->{val q=p(a[1]);result=listOf(terrain.standHeight(q.x,q.y,q.z).raw(),terrain.crouchHeight(q.x,q.y,q.z).raw(),terrain.isStandable(q),terrain.feet(q)?.raw()?:"-",terrain.check(q).name,terrain.isBowl(q.x,q.y,q.z),terrain.isOpen(q.x,q.y,q.z),terrain.hasRoom(q.x,q.y,q.z),terrain.hasRoom(q.x,q.y,q.z,Terrain.CROUCH_HEIGHT)).joinToString("|")}
   "box"->result=terrain.box(v(a[1]).x,v(a[1]).y,v(a[1]).z,d(a[2])).raw()
   "clear"->result="${terrain.isClear(b(a[1]))}|${terrain.containsAnyLiquid(b(a[1]))}"
   "sweep"->result=terrain.sweep(v(a[1]),v(a[2])).toString()
   "support"->result=terrain.isSupported(v(a[1])).toString()
   "bounce"->{val q=p(a[1]);result=terrain.restitution(q.x,d(a[2]),q.z).raw()}
   "closed"->{val opens=if(a[3]=="-")ArrayList<BlockPos>()else a[3].split(';').map(::p).toMutableList();terrain.closedIn(b(a[1]),b(a[2]),opens);result=opens.joinToString(";"){it.raw()}}
   "invalidate","change"->{val q=p(a[1]);val state=states.getValue(a[2].toInt());if(a[0]=="change")world.blocks[q]=state.id;result=terrain.invalidate(q,state).toString()}
   "limits"->result=listOf(terrain.width.raw(),terrain.height.raw(),terrain.canOpenDoors,terrain.minY,terrain.maxY).joinToString(",")
   else->error(command)
  }
  val output="$result|"+events.joinToString("/");events.clear();output
 }
 File(System.getProperty("results")).writeText(results.joinToString("\n")+"\n")
}
