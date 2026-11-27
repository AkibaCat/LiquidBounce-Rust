import net.ccbluex.agent.pathfinder.walk.*
import net.ccbluex.agent.simulation.MotionTraits
import net.ccbluex.agent.simulation.PlayerMotion
import it.unimi.dsi.fastutil.longs.LongOpenHashSet
import java.util.function.ToDoubleFunction
// fixture code
private val replaced=HashMap<Int,Boolean>()
private val breakTimes=HashMap<Int,Double>()
private var walkAbilities:WalkAbilities?=null
private var walkNodes:Nodes?=null
private val walkChanges=ChangesPlaceholder()
private class ChangesPlaceholder{var value:Changes?=null}
private var walkCourse:Course?=null
private fun longs(raw:String)=LongOpenHashSet().also{s->if(raw!="-")raw.split(';').map(::p).forEach{s.add(it.asLong())}}
private fun LandingRaw(l:Nodes.Landing?)=l?.let{"${it.y},${it.drop.raw()},${it.damage}"}?:"-"
private fun Course.raw()=listOf(fromX.raw(),fromY.raw(),fromZ.raw(),target.raw(),targetX.raw(),targetZ.raw(),targetHeight.raw(),catches,dirX.raw(),dirZ.raw(),distance.raw(),aimX.raw(),aimZ.raw(),edge.raw(),obstructed).joinToString("|")
private fun extra(a:List<String>,world:World,t:Terrain):String=when(a[0]){
 "walk"->{
  val ability=WalkAbilities(a[1].toBoolean(),a[2].toBoolean(),a[3].toInt(),true,canSprint=a[4].toBoolean(),safeFall=a[5].toInt(),waterFall=a[6].toInt(),breakTicks=ToDoubleFunction{val s=it as S;events+="T:${s.id}";breakTimes[s.id]?:1.0},avoid=longs(a[10]),air=a[8].toInt(),health=Float.fromBits(a[7].toUInt().toInt()),reserved=longs(a[9]),unbreakable=longs(a[11]))
  walkAbilities=ability;walkNodes=Nodes(t,ability);walkChanges.value=Changes(t,walkNodes!!,ability);walkCourse=Course(t,ability);"-"
 }
 "abilities"->{val w=walkAbilities!!;val q=w.withMotion(MotionTraits(Float.fromBits(a[1].toUInt().toInt()),Float.fromBits(a[2].toUInt().toInt()),Float.fromBits(a[3].toUInt().toInt()),if(a[4]=="true")Mechanics.V1_8 else Mechanics.MODERN));listOf(q.canBreak,q.canPlace,q.placeableBlocks,q.canParkour,q.riskyJumps,q.canSprint,q.safeFall,q.waterFall,q.air,q.jumpClimbs,q.bridgeTicks.raw(),q.health.toRawBits().toUInt(),q.motion.walkingSpeed.toRawBits().toUInt(),q.motion.jumpStrength.toRawBits().toUInt(),q.motion.jumpBoostPower.toRawBits().toUInt(),q.motion.jumpPower.raw(),q.motion.jumpHeight.raw(),q.avoid===w.avoid,q.unbreakable===w.unbreakable,q.reserved===w.reserved,q.breakTicks===w.breakTicks,w.motion.walkingSpeed.toRawBits().toUInt()).joinToString("|")}
 "ticks"->{breakTimes[a[1].toInt()]=d(a[2]);"-"}
 "node"->{val n=walkNodes!!;val q=p(a[1]);listOf(n.isNode(q.x,q.y,q.z),n.isUnderwater(q),n.stands(q.x,q.y,q.z),n.swims(q.x,q.y,q.z),n.climbs(q.x,q.y,q.z),n.crouches(q.x,q.y,q.z),n.height(q.x,q.y,q.z).raw(),n.feet(q.x,q.y,q.z).raw(),n.floorTop(q.x,q.y,q.z).raw(),n.isHarmless(q.x,q.y,q.z),n.isSolid(q.x,q.y,q.z),n.isPlaceable(q.x,q.y,q.z),n.isDry(q.x,q.y,q.z),n.water.isLow(q.x,q.y,q.z),n.water.isSubmerged(q.x,q.y,q.z)).joinToString("|")}
 "landing"->{val q=p(a[1]);LandingRaw(walkNodes!!.landing(q.x,q.y,q.z,d(a[2])))}
 "drop"->{val q=p(a[1]);walkNodes!!.isNearDrop(q.x,q.y,q.z,d(a[2])).toString()}
 "floorBlock"->{val q=p(a[1]);walkNodes!!.floorBlock(q.x,q.z,d(a[2])).toString()}
 "cost"->{val s=states.getValue(a[1].toInt());listOf(Costs.floorFactor(s.block).raw(),Costs.bounce(s.block,a[2].toInt()).raw(),Costs.fall(a[2].toInt()).raw(),Costs.JUMP.raw(),Water.movesUnderwater(a[3].toInt())).joinToString("|")}
 "from"->{walkChanges.value!!.from=p(a[1]);"-"}
 "break"->{val q=p(a[1]);val c=walkChanges.value!!;val result=if(a[2]=="pass")c.passOrBreak(q.x,q.y,q.z).toString()else c.breakTicks(q.x,q.y,q.z,a[2].toBoolean()).raw();"$result|${c.breakCost.raw()}|${c.breaks.joinToString(";"){it.raw()}}|${c.places.joinToString(";"){it.raw()}}"}
 "placeAgainst"->{val q=p(a[1]);walkChanges.value!!.canPlaceAgainst(q.x,q.y,q.z,a[2].toBoolean()).toString()}
 "clearChanges"->{walkChanges.value!!.clear();"-"}
 "course"->{walkCourse!!.set(v(a[1]).x,v(a[1]).y,v(a[1]).z,p(a[2]),d(a[3]));walkCourse!!.raw()}
 "aim"->{walkCourse!!.aimBeside(d(a[1]));walkCourse!!.raw()}
 "behind"->walkCourse!!.behind(d(a[1])).raw()
 "way"->walkCourse!!.isWayClear(v(a[1])).toString()
 "runUp"->walkCourse!!.runUpLength(d(a[1])).raw()
 "canStand"->walkCourse!!.canStandAt(v(a[1])).toString()
 "edge"->walkCourse!!.edgeLength().raw()
 "safe"->walkCourse!!.isSafe().toString()
 "over"->{val q=v(a[1]);val c=walkCourse!!;"${c.isOver(q.x,q.z)}|${c.isInTarget(q.x,q.y,q.z)}|${c.margin(q.x,q.z).raw()}"}
 "overshoots"->walkCourse!!.overshoots(d(a[1]),d(a[2]),d(a[3]),d(a[4])).toString()
 "estimate"->{val motion=PlayerMotion(t);motion.reset(v(a[1]),v(a[2]),false,a[3].toBoolean());walkCourse!!.estimate(motion).raw()}
 else->error(a[0])
}
