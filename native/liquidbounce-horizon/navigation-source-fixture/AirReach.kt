import net.ccbluex.agent.pathfinder.flight.Acceleration
import net.ccbluex.agent.pathfinder.flight.FlyAbilities
import net.ccbluex.agent.pathfinder.flight.Moves as FlyMoves
import net.ccbluex.agent.pathfinder.flight.RouteSearch as FlySearch
import net.ccbluex.agent.pathfinder.flight.Route as FlyRoute
import net.ccbluex.agent.pathfinder.flight.isClearLine
import net.ccbluex.agent.pathfinder.route.*

// fixture code
private var rayMode=0
private var rayCount=0
private var rayLimit=0
private var rayAt=BlockPos(0,0,0)
private fun rayClip(c:ClipContext):BlockHitResult{
 events+="R:${c.block}:${c.fluid}:${c.entity==null}:${c.from.raw()}:${c.to.raw()}"
 rayCount++
 val hit=when(rayMode){0->null;1->rayAt;2->rayAt.above();3->if(c.to.x>0.65&&c.to.y>0.4)rayAt else rayAt.above();4->if(rayCount>rayLimit)null else rayAt.above();else->error("ray mode")}
 return if(hit==null)BlockHitResult.miss(c.to,Direction.UP,BlockPos.containing(c.to))else BlockHitResult(c.to,Direction.UP,hit,false)
}
private lateinit var flySearch:FlySearch
private lateinit var flyMoves:FlyMoves
private val nanos=AStarSearch::class.java.getDeclaredField("computeNanos").also{it.isAccessible=true}
private fun goal(kind:String,raw:String,radius:String):Destination=when(kind){
 "B"->BlockDestination(raw.split(';').map(::p))
 "N"->NearDestination(v(raw),d(radius))
 "G"->object:Destination{val target=p(raw);override val ends=emptyList<Vec3>();override fun isGoal(x:Int,y:Int,z:Int)=BlockPos(x,y,z)==target;override fun distance(x:Int,y:Int,z:Int)=kotlin.math.sqrt(target.distSqr(x,y,z).toDouble())}
 else->error(kind)
}
private fun outcome(o:Outcome<FlyRoute>):String=when(o){Outcome.Searching->"S";Outcome.AlreadyThere->"A";is Outcome.Failed->"E:${o.failure}";is Outcome.Found->"F:${o.value.start.raw()}:${o.value.points.joinToString(";"){it.raw()}}:${o.value.canReach}:${o.value.end.raw()}"}
private fun extra(a:List<String>,world:World,terrain:Terrain):String=when(a[0]){
 "ray"->{rayMode=a[1].toInt();rayAt=p(a[2]);rayLimit=a[3].toInt();rayCount=0;"-"}
 "sees"->terrain.sees(v(a[3]),b(a[1]),d(a[2]),if(a[4]=="-")null else p(a[4])).toString()
 "reach"->terrain.reachPositions(b(a[1]),d(a[2]),v(a[3]),if(a[4]=="-")null else p(a[4]),if(a[5]=="-")net.ccbluex.agent.world.entity.Player.DEFAULT_EYE_HEIGHT.toDouble()else d(a[5]),a[6].toBoolean()).joinToString(";"){it.raw()}
 "creative"->{val ability=FlyAbilities.of(object:net.ccbluex.agent.world.entity.Abilities{override val mayfly=a[2].toBoolean();override val flying=a[3].toBoolean();override val flyingSpeed=Float.fromBits(a[1].toUInt().toInt())});ability?.let{listOf(it.control,it.horizontalSpeed.raw(),it.verticalSpeed.raw(),it.isFlying,it.horizontalCost.raw(),it.verticalCost.raw()).joinToString(",")}?:"-"}
 "fly"->{val ability=FlyAbilities(if(a[1]=="M")Acceleration.MOMENTUM else Acceleration.DIRECT,d(a[2]),d(a[3]),a[4].toBoolean());val destination=goal(a[6],a[7],a[8]);fun fresh()=Terrain(world,terrain.width,terrain.height,terrain.canOpenDoors)
  flyMoves=FlyMoves(fresh(),ability,destination);flySearch=FlySearch(fresh(),v(a[5]),destination,ability,Limits(d(a[9]),maxVisitedNodes=a[10].toInt(),visitedNodesPerSlice=a[11].toInt(),maxComputeNanos=a[12].toLong()));"-"}
 "moves"->{val q=p(a[1]);"${flyMoves.isOpen(q.x,q.y,q.z)}|${flyMoves.point(q).raw()}|"+flyMoves.neighbors(q).joinToString(";"){"${it.node.raw()}:${it.cost.raw()}"}}
 "line"->isClearLine(terrain,v(a[1]),v(a[2])).toString()
 "h"->(FlySearch::class.java.getDeclaredMethod("heuristic",BlockPos::class.java).also{it.isAccessible=true}.invoke(flySearch,p(a[1]))as Double).raw()
 "advance"->{val returned=try{outcome(flySearch.advance())}catch(e:IllegalArgumentException){"invalid"};"$returned~${nanos.getLong(flySearch)}~${flySearch.search?.iterations?:"-"}~${flySearch.search?.isFinished?:"-"}~${flySearch.preview()?.joinToString(";"){it.raw()}?:"-"}"}
 "preview"->flySearch.preview()?.joinToString(";"){it.raw()}?:"-"
 else->error(a.joinToString(" "))
}
