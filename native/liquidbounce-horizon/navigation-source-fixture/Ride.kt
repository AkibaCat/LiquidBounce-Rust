import net.ccbluex.agent.pathfinder.ride.*
import net.ccbluex.agent.pathfinder.route.*
import net.ccbluex.agent.world.entity.*
import java.lang.reflect.Proxy
// fixture code
private lateinit var rideVehicle:Vehicle
private lateinit var rideMoves:Moves
private lateinit var rideSearch:RouteSearch
private val nanos=AStarSearch::class.java.getDeclaredField("computeNanos").also{it.isAccessible=true}
private fun goal(kind:String,raw:String,radius:String):Destination=when(kind){
 "B"->BlockDestination(raw.split(';').map(::p))
 "N"->NearDestination(v(raw),d(radius))
 "G"->object:Destination{val target=p(raw);override val ends=emptyList<Vec3>();override fun isGoal(x:Int,y:Int,z:Int)=BlockPos(x,y,z)==target;override fun distance(x:Int,y:Int,z:Int)=kotlin.math.sqrt(target.distSqr(x,y,z).toDouble())}
 else->error(kind)
}
private fun Vehicle.raw()=listOf(kind,width.raw(),height.raw(),stepHeight.raw(),jumpHeight.raw(),safeFall,speed.raw(),iceSpeed.raw()).joinToString(",")
private fun Moves.Spot.raw()=position.raw()+":"+speed.raw()
private fun outcome(o:Outcome<Route>):String=when(o){Outcome.Searching->"S";Outcome.AlreadyThere->"A";is Outcome.Failed->"E:${o.failure}";is Outcome.Found->o.value.let{r->listOf("F",r.start.raw(),r.points.joinToString("@"){"${it.node.raw()}:${it.position.raw()}:${it.jump}"},r.canReach,r.exhausted,r.positions.joinToString(";"){it.raw()},r.end.raw()).joinToString("$")}}
private fun capture(a:List<String>):String{
 val data=a[2].split(',').map(::d);val jump=a[3].toBoolean()
 fun actor(type:Class<*>,prefix:String)=Proxy.newProxyInstance(Entity::class.java.classLoader,arrayOf(type)){_,method,args->
  val name=method.name;events+="$prefix:$name"
  when(name){
   "getPassengerRidingPosition"->Vec3(0.0,data[0],0.0);"getY"->data[1];"getBbHeight"->data[2].toFloat();"getEyeHeight"->data[3].toFloat();"getBbWidth"->data[4].toFloat();"maxUpStep"->data[5].toFloat();"canJump"->jump;"getJumpBoostPower"->data[7].toFloat()
   "getAttributeValue"->when(args!![0]){Attributes.JUMP_STRENGTH->data[6];Attributes.SAFE_FALL_DISTANCE->data[8];Attributes.MOVEMENT_SPEED->data[9];else->error("attribute")}
   else->error(name)
  }
 }as Entity
 val type=when(a[1]){"B"->AbstractBoat::class.java;"H"->AbstractHorse::class.java;"C"->Camel::class.java;else->Entity::class.java}
 return Vehicle.of(actor(Entity::class.java,"R"),actor(type,"V"))?.raw()?:"-"
}
private fun extra(a:List<String>,world:World,terrain:Terrain):String=when(a[0]){
 "capture"->capture(a)
 "vehicle"->{val q=a[2].split(',').map(::d);rideVehicle=Vehicle(if(a[1]=="B")VehicleKind.BOAT else VehicleKind.MOUNT,q[0],q[1],q[2],q[3],a[3].toInt(),q[4],q[5]);rideMoves=Moves(Terrain(world,rideVehicle.width,rideVehicle.height),rideVehicle);"-"}
 "rideSpot"->rideMoves.spot(p(a[1]))?.raw()?:"-"
 "rideMoves"->rideMoves.neighbors(p(a[1])).joinToString(";"){"${it.node.raw()}:${it.cost.raw()}"}
 "straight"->rideMoves.isStraight(v(a[1]),v(a[2])).toString()
 "passes"->Moves::class.java.getDeclaredMethod("passes",Vec3::class.java,Vec3::class.java).also{it.isAccessible=true}.invoke(rideMoves,v(a[1]),v(a[2])).toString()
 "cost"->{val f=Moves.Spot(v(a[1]),d(a[2]));val t=Moves.Spot(v(a[3]),d(a[4]));val cost=Moves::class.java.getDeclaredMethod("cost",Moves.Spot::class.java,Moves.Spot::class.java).also{it.isAccessible=true}.invoke(rideMoves,f,t)as Double;"${rideMoves.isJump(f,t)}:${cost.raw()}"}
 "path"->{rideSearch=RouteSearch(Terrain(world,rideVehicle.width,rideVehicle.height),v(a[1]),goal(a[2],a[3],a[4]),rideVehicle,Limits(d(a[5]),maxVisitedNodes=a[6].toInt(),visitedNodesPerSlice=a[7].toInt(),maxComputeNanos=a[8].toLong()));"-"}
 "advance"->{val returned=try{outcome(rideSearch.advance())}catch(e:IllegalArgumentException){"invalid"};"$returned~${nanos.getLong(rideSearch)}~${rideSearch.search?.iterations?:"-"}~${rideSearch.search?.isFinished?:"-"}~${rideSearch.preview()?.joinToString(";"){it.raw()}?:"-"}"}
 "preview"->rideSearch.preview()?.joinToString(";"){it.raw()}?:"-"
 else->error(a.joinToString(" "))
}
