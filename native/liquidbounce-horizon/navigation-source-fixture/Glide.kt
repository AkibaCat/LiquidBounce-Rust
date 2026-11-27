@file:OptIn(net.ccbluex.agent.InternalAgentApi::class)
package fixture
import java.io.File
import net.ccbluex.agent.math.*
import net.ccbluex.agent.pathfinder.glide.*
import net.ccbluex.agent.simulation.ElytraMotion
import net.ccbluex.agent.world.block.BlockState
import net.ccbluex.agent.world.entity.*
import net.ccbluex.agent.world.item.*

private fun d(v:String)=Double.fromBits(v.toULong().toLong())
private fun v(s:String)=s.split(',').let{Vec3(d(it[0]),d(it[1]),d(it[2]))}
private fun raw(v:Double)=v.toBits().toULong().toString()
private fun vector(v:Vec3)=listOf(v.x,v.y,v.z).joinToString(",",transform=::raw)
private fun field(target:Any,name:String)=target.javaClass.getDeclaredField(name).also{it.isAccessible=true}.get(target)
private fun rocket(r:Rockets)="${r.left},${r.boostTicks},${r.canFire}"
private fun landing(l:Landing):String {val s=field(l,"sweep");return listOf("bestError","bestXRot","bestAlong","farthestXRot","farthestAlong").joinToString(","){val x=field(s,it);if(x is Float)x.toRawBits().toUInt().toString()else raw(x as Double)}+":"+rocket(l.rockets)}
private fun cruise(c:Cruise):String {val l=field(c,"landing")as Landing?;return raw(field(c,"altitude")as Double)+","+field(c,"ticks")+":"+(l?.let(::landing)?:"-")+":"+vector(c.target)+":"+rocket(if(l==null)field(c,"rockets")as Rockets else l.rockets)}
private fun controls(c:Controls)="${c.yRot.toRawBits().toUInt()},${c.xRot.toRawBits().toUInt()},${c.shouldFireRocket},${c.isBoosted}"
private val events=ArrayList<String>()
private class Stack(val id:Int,val glide:Boolean,val damageable:Boolean,val maximum:Int,val damaged:Int,val rocket:Boolean,val amount:Int,val duration:Int?):ItemStack(){
 private fun log(s:String){events+="$id:$s"}
 override val item:Item get(){log("item");return if(rocket)Items.FIREWORK_ROCKET else Item.AIR}
 override val count:Int get(){log("count");return amount}
 override val maxStackSize=64
 override val damageValue:Int get(){log("damage");return damaged}
 override val maxDamage:Int get(){log("max");return maximum}
 override val isDamageableItem:Boolean get(){log("damageable");return damageable}
 override fun canGlideUsing(slot:EquipmentSlot):Boolean{log("glide:$slot");return glide}
 override val fireworkFlightDuration:Int? get(){log("flight");return duration}
 override fun copy()=this
 override fun getDestroySpeed(state:BlockState)=1f
 override fun isCorrectToolForDrops(state:BlockState)=false
 override fun getEnchantmentLevel(enchantment:Enchantment)=0
 override val enchantments=emptyMap<Enchantment,Int>()
 override val food:FoodProperties?=null
 override val potionEffects:List<MobEffectInstance>?=null
 override fun computeAttribute(attribute:Attribute,base:Double,slot:EquipmentSlot)=base
 override val equipmentSlot:EquipmentSlot?=null
 override val useEffects=UseEffects.DEFAULT
 override fun sameData(other:ItemStack)=false
}
fun main(){
 val stacks=HashMap<Int,Stack>();var r=Rockets(0,0);var l=Landing(Vec3.ZERO,r);var heightMode=0;var fallback:Vec3?=null
 val heights=ColumnHeights{x,z->events+="H:$x,$z";when(heightMode){0->0;1->null;2->if((x+z)%3==0)null else (x*x+z*z)%80;else->if(x<0)120 else 5}}
 fun newCruise(goal:Vec3,rockets:Rockets,ceiling:Int)=Cruise(goal,heights,rockets,ceiling){at->events+="F:${vector(at)}";fallback}
 var c=newCruise(Vec3.ZERO,r,320);var position=Vec3.ZERO;var motion=Vec3.ZERO
 val results=File(System.getProperty("commands")).readLines().map{command->
  val a=command.split(' ')
  val result=when(a[0]){
   "stack"->{val id=a[1].toInt();stacks[id]=Stack(id,a[2].toBoolean(),a[3].toBoolean(),a[4].toInt(),a[5].toInt(),a[6].toBoolean(),a[7].toInt(),a[8].takeIf{it!="-"}?.toInt());"-"}
   "abilities"->GlideAbilities.of(stacks.getValue(a[1].toInt()),if(a[3]=="-")emptyList()else a[3].split(',').map{stacks.getValue(it.toInt())},a[2].toBoolean())?.let{"${it.rockets},${it.rocketTicks},${it.glideTicks}"}?:"-"
   "rocket"->{r=Rockets(a[1].toInt(),a[2].toInt());rocket(r)}
   "fire"->{r.fire();rocket(r)}
   "burn"->"${r.tick()}:"+rocket(r)
   "motion"->vector(ElytraMotion.updateFallFlyingMovement(v(a[1]),Float.fromBits(a[2].toUInt().toInt()),Float.fromBits(a[3].toUInt().toInt()),a[4].toBoolean()))
   "angles"->"${ElytraMotion.yRotTowards(d(a[1]),d(a[2])).toRawBits().toUInt()},${ElytraMotion.xRotTowards(d(a[3]),d(a[4])).toRawBits().toUInt()}"
   "landing"->{l=Landing(v(a[1]),Rockets(a[2].toInt(),a[3].toInt()),d(a[4]));landing(l)}
   "land"->controls(l.steer(v(a[1]),v(a[2])))+":"+landing(l)
   "height"->{heightMode=a[1].toInt();"-"}
   "altitude"->raw(Cruise.cruiseAltitude(v(a[1]),v(a[2]),heights))
   "needed"->Cruise.rocketsNeeded(d(a[1]),d(a[2]),a[3].toInt()).toString()
   "fallback"->{fallback=if(a[1]=="-")null else v(a[1]);"-"}
   "cruise"->{c=newCruise(v(a[1]),Rockets(a[2].toInt(),a[3].toInt()),a[4].toInt());cruise(c)}
   "sail"->controls(c.steer(v(a[1]),v(a[2])))+":"+cruise(c)
   "position"->{position=v(a[1]);motion=v(a[2]);"-"}
   "step"->{val input=if(a[1]=="L")l.steer(position,motion)else c.steer(position,motion);motion=ElytraMotion.updateFallFlyingMovement(motion,input.yRot,input.xRot,input.isBoosted);position=position.add(motion);controls(input)+":"+vector(position)+":"+vector(motion)+":"+if(a[1]=="L")landing(l)else cruise(c)}
   else->error(command)
  }
  val output=result+"|"+events.joinToString("/");events.clear();output
 }
 File(System.getProperty("results")).writeText(results.joinToString("\n")+"\n")
}
