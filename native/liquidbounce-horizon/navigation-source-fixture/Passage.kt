import net.ccbluex.agent.pathfinder.glide.Passage
import net.ccbluex.agent.pathfinder.glide.Rockets
import net.ccbluex.agent.pathfinder.flight.Route
// fixture code
private lateinit var passage:Passage
private fun field(target:Any,name:String)=target.javaClass.getDeclaredField(name).also{it.isAccessible=true}.get(target)
private fun projection(p:Any)=listOf(field(p,"segment"),(field(p,"arc")as Double).raw(),(field(p,"point")as Vec3).raw(),(field(p,"distance")as Double).raw()).joinToString(":")
private fun project(position:Vec3,from:Int)=Passage::class.java.getDeclaredMethod("project",Vec3::class.java,Int::class.javaPrimitiveType).also{it.isAccessible=true}.invoke(passage,position,from)
private fun passageState():String=listOf(passage.route.joinToString(";"){it.raw()},(field(passage,"arcs")as DoubleArray).joinToString(","){it.raw()},(field(passage,"endArc")as Double).raw(),field(passage,"segment"),(field(passage,"lastXRot")as Float).toRawBits().toUInt(),field(passage,"reachesTarget"),field(passage,"passesTarget"),passage.drift.raw(),passage.remaining.raw(),passage.rockets.left,passage.rockets.boostTicks,passage.rockets.canFire).joinToString("^")
private fun extra(a:List<String>,world:World,terrain:Terrain):String=when(a[0]){
 "passage"->{passage=Passage(v(a[1]),Rockets(a[2].toInt(),a[3].toInt()),d(a[4]),a[5].toBoolean()){events+="T";Terrain(world,terrain.width,terrain.height,terrain.canOpenDoors)};passageState()}
 "follow"->{val points=a[1].split(';').map(::v);val route=Route(points.first(),points,a[2].toBoolean());if(a[3]=="-")passage.follow(route)else passage.follow(route,a[3].toBoolean());passageState()}
 "lands"->{passage.landsAtTarget=a[1].toBoolean();passageState()}
 "steer"->{val c=passage.steer(v(a[1]),v(a[2]));"${c.yRot.toRawBits().toUInt()},${c.xRot.toRawBits().toUInt()},${c.shouldFireRocket},${c.isBoosted}#${passageState()}"}
 "project"->projection(project(v(a[1]),a[2].toInt()))
 "look"->{val p=project(v(a[1]),a[2].toInt());(Passage::class.java.getDeclaredMethod("lookAhead",p.javaClass,Double::class.javaPrimitiveType).also{it.isAccessible=true}.invoke(passage,p,d(a[3]))as Vec3).raw()}
 "rollout"->{val p=project(v(a[1]),a[3].toInt());(Passage::class.java.getDeclaredMethod("rollout",Terrain::class.java,Vec3::class.java,Vec3::class.java,p.javaClass,Float::class.javaPrimitiveType,Float::class.javaPrimitiveType).also{it.isAccessible=true}.invoke(passage,Terrain(world,terrain.width,terrain.height,terrain.canOpenDoors),v(a[1]),v(a[2]),p,Float.fromBits(a[4].toUInt().toInt()),Float.fromBits(a[5].toUInt().toInt()))as Double).raw()}
 else->error(a.joinToString(" "))
}
