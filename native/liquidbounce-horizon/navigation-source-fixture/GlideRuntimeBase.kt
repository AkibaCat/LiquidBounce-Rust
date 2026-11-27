import java.lang.reflect.Proxy
import net.ccbluex.agent.navigation.*
import net.ccbluex.agent.navigation.glide.*
import net.ccbluex.agent.pathfinder.glide.GlideAbilities
import net.ccbluex.agent.world.entity.*
// fixture code
object RuntimeProbe {
 var events=ArrayList<String>()
 fun vector(v:Vec3)=listOf(v.x,v.y,v.z).joinToString(","){it.toRawBits().toULong().toString()}
}
private var runtimePosition=Vec3.ZERO
private var runtimeBox=AABB(0.0,0.0,0.0,1.0,1.0,1.0)
private var runtimeGround=false
private var runtimeFlying=false
private var runtimeHorizontal=false
private var runtimeVertical=false
private var runtimeBelow=false
private var runtimeRecasts=false
private lateinit var runtimeWorld:World
private val runtimeBorder=Proxy.newProxyInstance(WorldBorder::class.java.classLoader,arrayOf(WorldBorder::class.java)){_,m,a->when(m.name){
 "isWithinBounds"->runtimeWorld.isInsideBorder(a!![0]as Double,a[1]as Double)
 else->error(m.name)
}}as WorldBorder
private val runtimeLevel=Proxy.newProxyInstance(Level::class.java.classLoader,arrayOf(Level::class.java)){_,m,a->when(m.name){
 "getMinY"->runtimeWorld.minY;"getMaxY"->runtimeWorld.maxY;"getHeight"->runtimeWorld.height;"getMechanics"->runtimeWorld.mechanics
 "hasChunkAt"->{val x=a!![0]as Int;val z=a[1]as Int;events+="H:$x,$z";BlockPos(x,0,z)!in runtimeWorld.unknown}
 "getWorldBorder"->runtimeBorder
 "getBlockState"->runtimeWorld.getBlockState(a!![0]as BlockPos)
 else->error("Level.${m.name}")
}}as Level
private val runtimePlayer=Proxy.newProxyInstance(Player::class.java.classLoader,arrayOf(Player::class.java)){_,m,_->when(m.name){
 "position"->{events+="p";runtimePosition};"onGround"->{events+="g";runtimeGround};"isFallFlying"->{events+="f";runtimeFlying}
 "getHorizontalCollision"->{events+="h";runtimeHorizontal};"getVerticalCollision"->{events+="v";runtimeVertical};"getVerticalCollisionBelow"->{events+="b";runtimeBelow};"getBoundingBox"->{events+="A";runtimeBox}
 else->error("Player.${m.name}")
}}as Player
private val runtimeBody=Proxy.newProxyInstance(Body::class.java.classLoader,arrayOf(Body::class.java)){_,m,a->when(m.name){
 "getPlayer"->{events+="P";runtimePlayer};"getLevel"->{events+="L";runtimeLevel};"getRecastsElytra"->{events+="r";runtimeRecasts}
 "press"->{events+="K:${(a!![0]as Input).hashCode()}:${a[1]}";Unit}
 "look"->{val rotation=a!![0]as ViewRotation;events+="V:${rotation.yRot.toRawBits().toUInt()},${rotation.xRot.toRawBits().toUInt()}:${a[1]}";Unit}
 else->error("Body.${m.name}")
}}as Body
private val runtimeContext=NavigationContext(runtimeBody)
private var roof=RoofBounce(runtimeContext)
private var contacts=Contacts(runtimeContext)
private fun field(target:Any,name:String)=target.javaClass.getDeclaredField(name).also{it.isAccessible=true}.get(target)
private fun roofState():String {val b=field(roof,"bounce")?:return "-";return listOf((field(b,"goal")as Vec3).raw(),(field(b,"floor")as Double).raw(),(field(b,"lineEnd")as Vec3).raw(),field(b,"ticks")).joinToString(":")}
private fun contactState()=(field(contacts,"touching")as Set<*>).joinToString(";"){val q=it!!;"${(field(q,"pos")as BlockPos).raw()}:${field(q,"direction")}"}
private fun extra(a:List<String>,world:World,terrain:Terrain):String {
 runtimeWorld=world;RuntimeProbe.events=events
 return when(a[0]){
  "runtime"->{roof=RoofBounce(runtimeContext);contacts=Contacts(runtimeContext);File(System.getProperty("directionHashes")).writeText(Direction.entries.joinToString(","){it.hashCode().toString()});"-"}
  "player"->{runtimePosition=v(a[1]);runtimeGround=a[2].toBoolean();runtimeFlying=a[3].toBoolean();runtimeRecasts=a[4].toBoolean();runtimeHorizontal=a[5].toBoolean();runtimeVertical=a[6].toBoolean();runtimeBelow=a[7].toBoolean();runtimeBox=b(a[8]);"-"}
  "line"->RoofBounce.bounceLine(terrain,v(a[1]),v(a[2]),d(a[3]))?.raw()?:"-"
  "roof"->"${roof.navigate(v(a[1]),if(a[2]=="true")GlideAbilities(0,0,100)else null,Float.fromBits(a[3].toUInt().toInt()))}:${roof.isActive}:${roofState()}"
  "halt"->{roof.halt();roofState()}
  "contacts"->{contacts.tick();contactState()}
  else->error(a.joinToString(" "))
 }
}
