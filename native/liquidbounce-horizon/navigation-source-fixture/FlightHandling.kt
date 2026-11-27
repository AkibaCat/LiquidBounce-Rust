@file:OptIn(net.ccbluex.agent.InternalAgentApi::class)
package fixture
import java.io.File
import java.lang.reflect.Proxy
import net.ccbluex.agent.math.Vec3
import net.ccbluex.agent.navigation.flight.Handling
import net.ccbluex.agent.pathfinder.flight.*
import net.ccbluex.agent.world.Input
import net.ccbluex.agent.world.entity.Player

private fun d(raw:String)=Double.fromBits(raw.toULong().toLong())
private fun v(raw:String)=raw.split(',').let{Vec3(d(it[0]),d(it[1]),d(it[2]))}
private fun raw(value:Double)=value.toBits().toULong().toString()
private fun Vec3.raw()=listOf(x,y,z).joinToString(",",transform=::raw)
private fun key(mask:Int)=Input(mask and 64!=0,mask and 32!=0,mask and 16!=0,mask and 8!=0,mask and 4!=0,mask and 2!=0,mask and 1!=0)
private fun field(h:Handling,name:String)=h.javaClass.getDeclaredField(name).also{it.isAccessible=true}.get(h)
private fun state(h:Handling):String {
 val numbers=listOf("baseSpeed","baseVerticalSpeed","measuredDrag","measuredVerticalDrag","measuredThrust","measuredSprintThrust","measuredVerticalThrust").map{raw(field(h,it)as Double)}
 val tail=listOf((field(h,"lastPosition")as Vec3?)?.raw()?:"-",h.lastMove?.raw()?:"-",field(h,"lastTick").toString(),(field(h,"lastPressed")as Input?)?.hashCode()?.toString()?:"-")
 return (listOf(field(h,"control").toString())+numbers+tail+listOf(raw(h.drag),raw(h.verticalDrag),raw(h.speed),raw(h.verticalSpeed),raw(h.step),h.coasts.toString(),raw(h.coastDistance))).joinToString("|")
}
fun main(){
 var handling=Handling();var tick=0;var position=Vec3.ZERO;var delta=Vec3.ZERO;var yaw=0f;var sprint=false;var horizontal=false;var vertical=false
 val events=ArrayList<String>()
 val player=Proxy.newProxyInstance(Player::class.java.classLoader,arrayOf(Player::class.java)){_,method,_->
  when(method.name){
   "position"->{events+="p";position}
   "getDeltaMovement"->{events+="d";delta}
   "getTickCount"->{events+="t";tick}
   "getHorizontalCollision"->{events+="h";horizontal}
   "getVerticalCollision"->{events+="v";vertical}
   "getYRot"->{events+="y";yaw}
   "isSprinting"->{events+="s";sprint}
   else->error("unused ${method.name}")
  }
 }as Player
 val results=File(System.getProperty("commands")).readLines().map{command->
  val p=command.split(' ');var result="-"
  when(p[0]){
   "new"->handling=Handling()
   "base"->{val base=FlyAbilities(Acceleration.valueOf(p[1]),d(p[2]),d(p[3]),p[4].toBoolean());val value=handling.abilities(base);result=listOf(value.control,raw(value.horizontalSpeed),raw(value.verticalSpeed),value.isFlying,raw(value.horizontalCost),raw(value.verticalCost)).joinToString(",")}
   "player"->{tick=p[1].toInt();position=v(p[2]);delta=v(p[3]);yaw=Float.fromBits(p[4].toUInt().toInt());sprint=p[5].toBoolean();horizontal=p[6].toBoolean();vertical=p[7].toBoolean()}
   "record"->handling.record(player,if(p[1]=="-")null else key(p[1].toInt()))
   "rest"->result=handling.rest(player).raw()
   else->error(command)
  }
  val output="$result|${state(handling)}|${events.joinToString(",")}";events.clear();output
 }
 File(System.getProperty("results")).writeText(results.joinToString("\n")+"\n")
}
