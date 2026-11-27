import net.ccbluex.agent.pathfinder.walk.*
import net.ccbluex.agent.pathfinder.walk.Moves as WalkMoves
import net.ccbluex.agent.pathfinder.walk.RouteSearch as WalkSearch
import net.ccbluex.agent.pathfinder.walk.Route as WalkRoute
import net.ccbluex.agent.pathfinder.route.*
import net.ccbluex.agent.simulation.MotionTraits
import it.unimi.dsi.fastutil.longs.LongOpenHashSet
import java.util.function.ToDoubleFunction
// fixture code
private val replaced=HashMap<Int,Boolean>()
private val breakTimes=HashMap<Int,Double>()
private lateinit var fullAbilities:WalkAbilities
private lateinit var fullMoves:WalkMoves
private lateinit var walkSearch:WalkSearch
private val nanos=AStarSearch::class.java.getDeclaredField("computeNanos").also{it.isAccessible=true}
private fun longs(raw:String)=LongOpenHashSet().also{s->if(raw!="-")raw.split(';').map(::p).forEach{s.add(it.asLong())}}
private fun goal(kind:String,raw:String,radius:String):Destination=when(kind){
 "B"->BlockDestination(raw.split(';').map(::p))
 "N"->NearDestination(v(raw),d(radius))
 "G"->object:Destination{val target=p(raw);override val ends=emptyList<Vec3>();override fun isGoal(x:Int,y:Int,z:Int)=BlockPos(x,y,z)==target;override fun distance(x:Int,y:Int,z:Int)=kotlin.math.sqrt(target.distSqr(x,y,z).toDouble())}
 else->error(kind)
}
private fun Jump.raw()=listOf(runUp.raw(),takeoff,jumps,airTicks,margin.raw(),hops,aim.raw()).joinToString("^")
private fun offer(to:BlockPos,move:Move,cost:Double,breaks:List<BlockPos>,places:List<BlockPos>,opens:List<BlockPos>,jump:Jump?)=listOf(to.raw(),move,cost.raw(),breaks.joinToString(";"){it.raw()},places.joinToString(";"){it.raw()},opens.joinToString(";"){it.raw()},jump?.raw()?:"-").joinToString(":")
private fun Step.raw()=listOf(from.raw(),offer(to,move,cost,breaks,places,opens,jump),feet.raw(),shift,edge,isUnderWater,isBridge).joinToString("#")
private fun outcome(o:Outcome<WalkRoute>):String=when(o){Outcome.Searching->"S";Outcome.AlreadyThere->"A";is Outcome.Failed->"E:${o.failure}";is Outcome.Found->o.value.let{r->listOf("F",r.start.raw(),r.steps.joinToString("@"){it.raw()},r.canReach,r.end.raw(),r.points.joinToString(";"){it.raw()},r.cost.raw(),r.points===r.points).joinToString("$")}}
private fun extra(a:List<String>,world:World,terrain:Terrain):String=when(a[0]){
 "ability"->{
  fullAbilities=WalkAbilities(a[1].toBoolean(),a[2].toBoolean(),a[3].toInt(),a[4].toBoolean(),a[5].toBoolean(),a[6].toBoolean(),a[7].toInt(),a[8].toInt(),ToDoubleFunction{val s=it as S;events+="T:${s.id}";breakTimes[s.id]?:1.0},longs(a[9]),a[10].toInt(),MotionTraits(Float.fromBits(a[11].toUInt().toInt()),Float.fromBits(a[12].toUInt().toInt()),Float.fromBits(a[13].toUInt().toInt()),if(world.legacy)Mechanics.V1_8 else Mechanics.MODERN),a[14].toBoolean(),d(a[15]),longs(a[16]),Float.fromBits(a[17].toUInt().toInt()),longs(a[18]))
  fullMoves=WalkMoves(Terrain(world,terrain.width,terrain.height,terrain.canOpenDoors),fullAbilities);"-"
 }
 "ticks"->{breakTimes[a[1].toInt()]=d(a[2]);"-"}
 "moves"->{val offers=ArrayList<String>();val filter=if(a[2]=="-")null else p(a[2]);fullMoves.forEach(p(a[1]),MoveSink{to,move,cost,breaks,places,opens,jump->offers+=offer(to,move,cost,breaks,places,opens,jump)},JumpFilter{x,y,z->filter==null||BlockPos(x,y,z)==filter});offers.joinToString("|")}
 "step"->fullMoves.step(p(a[1]),p(a[2]))?.raw()?:"-"
 "path"->{val fresh=Terrain(world,terrain.width,terrain.height,terrain.canOpenDoors);walkSearch=WalkSearch(fresh,v(a[1]),goal(a[2],a[3],a[4]),fullAbilities,Limits(d(a[5]),maxVisitedNodes=a[6].toInt(),visitedNodesPerSlice=a[7].toInt(),maxComputeNanos=a[8].toLong()));"-"}
 "advance"->{val returned=try{outcome(walkSearch.advance())}catch(e:IllegalArgumentException){"invalid"};"$returned~${nanos.getLong(walkSearch)}~${walkSearch.search?.iterations?:"-"}~${walkSearch.search?.isFinished?:"-"}~${walkSearch.preview()?.joinToString(";"){it.raw()}?:"-"}"}
 "preview"->walkSearch.preview()?.joinToString(";"){it.raw()}?:"-"
 else->error(a.joinToString(" "))
}
