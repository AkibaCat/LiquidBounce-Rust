import net.ccbluex.agent.pathfinder.glide.AirwaySearch
import net.ccbluex.agent.pathfinder.glide.Passage
import net.ccbluex.agent.pathfinder.route.*
import net.ccbluex.agent.pathfinder.flight.Route
// fixture code
private lateinit var airway:AirwaySearch
private val nanos=AStarSearch::class.java.getDeclaredField("computeNanos").also{it.isAccessible=true}
private fun call(name:String,q:BlockPos):Any=AirwaySearch::class.java.getDeclaredMethod(if(name=="neighbors")"neighbors\$main" else name,BlockPos::class.java).also{it.isAccessible=true}.invoke(airway,q)
private fun outcome(o:Outcome<Route>):String=when(o){Outcome.Searching->"S";Outcome.AlreadyThere->"A";is Outcome.Failed->"E:${o.failure}";is Outcome.Found->"F:${o.value.start.raw()}:${o.value.points.joinToString(";"){it.raw()}}:${o.value.canReach}:${o.value.end.raw()}"}
private fun extra(a:List<String>,world:World,terrain:Terrain):String=when(a[0]){
 "air"->{val fresh={Terrain(world,terrain.width,terrain.height,terrain.canOpenDoors)};val start=v(a[1]);val base=NearDestination(v(a[2]),d(a[3]));val destination=if(a[5]=="-")base else StageDestination(base,start,d(a[5]),fresh());airway=AirwaySearch(fresh(),start,destination,d(a[4]),Limits(d(a[6]),maxVisitedNodes=a[7].toInt(),visitedNodesPerSlice=a[8].toInt(),maxComputeNanos=a[9].toLong()));"-"}
 "node"->{val q=p(a[1]);@Suppress("UNCHECKED_CAST") val edges=call("neighbors",q)as Iterable<WeightedEdge<BlockPos>>;"${call("isGoal",q)}|${(call("heuristic",q)as Double).raw()}|"+edges.joinToString(";"){"${it.node.raw()}:${it.cost.raw()}"}}
 "open"->{val q=p(a[1]);val params=arrayOf(Int::class.javaPrimitiveType,Int::class.javaPrimitiveType,Int::class.javaPrimitiveType);val open=AirwaySearch::class.java.getDeclaredMethod("isOpen",*params).also{it.isAccessible=true}.invoke(airway,q.x,q.y,q.z);val roomy=AirwaySearch::class.java.getDeclaredMethod("isRoomy",*params).also{it.isAccessible=true}.invoke(airway,q.x,q.y,q.z);"$open,$roomy"}
 "line"->AirwaySearch::class.java.getDeclaredMethod("isClearLine",Vec3::class.java,Vec3::class.java,Double::class.javaPrimitiveType).also{it.isAccessible=true}.invoke(airway,v(a[1]),v(a[2]),d(a[3])).toString()
 "glideBox"->Passage.box(v(a[1]),d(a[2])).raw()
 "advance"->{val returned=try{outcome(airway.advance())}catch(e:IllegalArgumentException){"invalid"};"$returned~${nanos.getLong(airway)}~${airway.search?.iterations?:"-"}~${airway.search?.isFinished?:"-"}~${airway.preview()?.joinToString(";"){it.raw()}?:"-"}"}
 "preview"->airway.preview()?.joinToString(";"){it.raw()}?:"-"
 else->error(a.joinToString(" "))
}
