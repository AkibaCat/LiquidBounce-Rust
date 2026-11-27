@file:OptIn(net.ccbluex.agent.InternalAgentApi::class)
package fixture
import java.io.File
import java.lang.reflect.Proxy
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
fun main(){
 var position=Vec3.ZERO;var delta=Vec3.ZERO;var tick=0;var yaw=0f;var sprint=false;var horizontal=false;var vertical=false;var ground=false;var creative=false;var mayfly=false;var flyingSpeed=.05f;var host=false;var canFly=true;var collision=true
 val events=ArrayList<String>()
 val abilities=Proxy.newProxyInstance(Abilities::class.java.classLoader,arrayOf(Abilities::class.java)){_,method,_->when(method.name){"getFlying"->{events+="af";creative};"getMayfly"->{events+="am";mayfly};"getFlyingSpeed"->{events+="as";flyingSpeed};else->error(method.name)}}as Abilities
 val player=Proxy.newProxyInstance(Player::class.java.classLoader,arrayOf(Player::class.java)){_,method,args->when(method.name){
  "position"->{events+="p";position};"getDeltaMovement"->{events+="d";delta};"getTickCount"->{events+="t";tick};"getHorizontalCollision"->{events+="h";horizontal};"getVerticalCollision"->{events+="v";vertical};"getYRot"->{events+="y";yaw};"isSprinting"->{events+="s";sprint};"getAbilities"->{events+="a";abilities};"getY"->{events+="Y";position.y};"onGround"->{events+="g";ground};"getBoundingBox"->{events+="B";AABB(position.x-.3,position.y,position.z-.3,position.x+.3,position.y+1.8,position.z+.3)};"getPosition"->{val alpha=args!![0]as Float;events+="i:${alpha.toRawBits().toUInt()}";position.add(delta.scale(alpha.toDouble()))};else->error("Player.${method.name}")
 }}as Player
 val level=Proxy.newProxyInstance(Level::class.java.classLoader,arrayOf(Level::class.java)){_,method,args->when(method.name){"noBlockCollision"->{events+="C:${(args!![1]as AABB).raw()}";!collision};else->error("Level.${method.name}")}}as Level
 val body=Proxy.newProxyInstance(Body::class.java.classLoader,arrayOf(Body::class.java)){_,method,args->when(method.name){"getPlayer"->{events+="P";player};"getLevel"->level;"isFlying"->{events+="H";host};"press"->{events+="K:${(args!![0]as Input).hashCode()}:${args[1]}";Unit};else->error("Body.${method.name}")}}as Body
 val context=NavigationContext(body){events+="o";FlightOptions(canFly)}
 var handling=Handling();var follower=Follower(context,handling);var current=FlyAbilities(Acceleration.DIRECT,1.0,.4,true);var watch=ProgressWatch(.1,60)
 val results=File(System.getProperty("commands")).readLines().map{command->
  val a=command.split(' ');var returned="-"
  when(a[0]){
   "new"->{handling=Handling();follower=Follower(context,handling)}
   "base"->{current=handling.abilities(FlyAbilities(Acceleration.valueOf(a[1]),d(a[2]),d(a[3]),a[4].toBoolean()))}
   "player"->{tick=a[1].toInt();position=v(a[2]);delta=v(a[3]);yaw=Float.fromBits(a[4].toUInt().toInt());sprint=a[5].toBoolean();horizontal=a[6].toBoolean();vertical=a[7].toBoolean();ground=a[8].toBoolean();creative=a[9].toBoolean();mayfly=a[10].toBoolean();flyingSpeed=Float.fromBits(a[11].toUInt().toInt())}
   "host"->{host=a[1].toBoolean();canFly=a[2].toBoolean();collision=a[3].toBoolean()}
   "capture"->{val captured=context.captureFlight(handling);if(captured==null)returned="null"else{current=captured;returned=listOf(captured.control,raw(captured.horizontalSpeed),raw(captured.verticalSpeed),captured.isFlying).joinToString(",")}}
   "follow"->{val points=if(a[2]=="-")emptyList()else a[2].split(';').map(::v);follower.follow(Route(v(a[1]),points,a[3].toBoolean()))}
   "clear"->follower.clear()
   "tick"->returned=follower.tick(Float.fromBits(a[1].toUInt().toInt()),current).name
   "off"->returned=follower.isOffPath().toString()
   "head"->returned=follower.head(Float.fromBits(a[1].toUInt().toInt())).raw()
   "direction"->returned=getDirectionalInputTowards(v(a[1]),Float.fromBits(a[2].toUInt().toInt())).hashCode().toString()
   "segment"->returned=raw(v(a[1]).distanceToSegment(v(a[2]),v(a[3])))
   "sharp"->{val companion=follower.javaClass.getDeclaredField("Companion").also{it.isAccessible=true}.get(null);returned=companion.javaClass.getDeclaredMethod("isSharp",List::class.java,Int::class.javaPrimitiveType).also{it.isAccessible=true}.invoke(companion,a[1].split(';').map(::v),a[2].toInt()).toString()}
   "seed"->{follower.javaClass.getDeclaredField("takeOffTicks").also{it.isAccessible=true}.setInt(follower,a[1].toInt())}
   "watch"->watch=ProgressWatch(d(a[1]),a[2].toInt())
   "stalls"->returned=watch.stalls(d(a[1])).toString()
   "excuse"->watch.excuse()
   "reset"->watch.reset()
   "watchseed"->watch.javaClass.getDeclaredField("idleTicks").also{it.isAccessible=true}.setInt(watch,a[1].toInt())
   else->error(command)
  }
  val result="$returned|${state(follower,handling)}|${watch.idleTicks}|${events.joinToString("/")}";events.clear();result
 }
 File(System.getProperty("results")).writeText(results.joinToString("\n")+"\n")
}
