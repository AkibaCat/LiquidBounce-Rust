import net.ccbluex.agent.pathfinder.walk.*
import net.ccbluex.agent.simulation.*
import net.ccbluex.agent.navigation.walk.AssistedJump
import it.unimi.dsi.fastutil.longs.LongOpenHashSet
import java.util.function.ToDoubleFunction
// fixture code
private val replaced=HashMap<Int,Boolean>()
private lateinit var abilities:WalkAbilities
private lateinit var jumps:Jumps
private lateinit var targets:JumpTargets
private lateinit var motion:PlayerMotion
private var chosen:Jump?=null
private var leap=Leap()
private var assist=AssistedJump()
private fun f(raw:String)=Float.fromBits(raw.toUInt().toInt())
private fun Float.raw()=toRawBits().toUInt().toString()
private fun Jump.raw()=listOf(runUp.raw(),takeoff,jumps,airTicks,margin.raw(),hops,aim.raw()).joinToString("^")
private fun LeapInput.raw()=listOf(forward,strafe,jump,sprint,shift).joinToString(",")
private fun PlayerMotion.raw()=listOf(position.raw(),deltaMovement.raw(),onGround,horizontalCollision,isSprinting,landingFall.raw(),peakY.raw(),jumpDelay,"$hasSupport,$supportX,$supportY,$supportZ",crouching,sneaking,airSprint,walked.raw(),nextStep,blockPosition.raw(),horizontalSpeed.raw()).joinToString(";")
private fun AssistedJump.Landing.raw()=listOf(pos.raw(),feet.raw(),jump.raw(),step?.let{listOf(it.from.raw(),it.to.raw(),it.feet.raw(),it.move,it.cost.raw(),it.shift,it.edge,it.isUnderWater).joinToString(":")}?:"-",bounces,takesOff,settles).joinToString("#")
private fun AssistedJump.raw()=listOf(target?.raw()?:"-",arcs.joinToString("@"){listOf(it.target.raw(),it.takeoff,it.path.joinToString(";"){p->p.raw()}).joinToString("$")}).joinToString("%")
private fun step(a:List<String>,offset:Int):Step {val to=p(a[offset+1]);return Step(p(a[offset]),to,Move.PARKOUR,Vec3(to.x+.5,d(a[offset+2]),to.z+.5),emptyList(),emptyList(),emptyList(),0.0,jump=chosen)}
private fun extra(a:List<String>,world:World,terrain:Terrain):String=when(a[0]){
 "jnew"->{abilities=WalkAbilities(false,false,0,true,a[1].toBoolean(),a[2].toBoolean(),a[3].toInt(),16,ToDoubleFunction{1.0},LongOpenHashSet(),motion=MotionTraits(f(a[4]),f(a[5]),f(a[6]),if(world.legacy)Mechanics.V1_8 else Mechanics.MODERN));jumps=Jumps(terrain,abilities,a[7].toBoolean());targets=JumpTargets(terrain,abilities.motion);motion=PlayerMotion(terrain,abilities.motion);chosen=null;leap=Leap();assist=AssistedJump();"-"}
 "motion"->{motion.reset(v(a[1]),v(a[2]),a[3].toBoolean(),a[4].toBoolean(),a[5].toInt());motion.crouching=a[6].toBoolean();motion.sneaking=a[7].toBoolean();motion.raw()}
 "tickMotion"->{motion.tick(f(a[1]),a[2].toInt(),a[3].toInt(),a[4].toBoolean(),a[5].toBoolean(),a[6].toBoolean());motion.raw()}
 "jdef"->{chosen=Jump(v(a[1]),a[2].toInt(),a[3].toBoolean(),a[4].toInt(),d(a[5]),a[6].toInt(),v(a[7]));chosen!!.raw()}
 "plan"->{chosen=jumps.plan(p(a[1]),d(a[2]),p(a[3]),d(a[4]),a[5].toBoolean());chosen?.raw()?:"-"}
 "refine"->{chosen=jumps.refine(p(a[1]),d(a[2]),p(a[3]),d(a[4]));chosen?.raw()?:"-"}
 "here"->{chosen=jumps.fromHere(motion,f(a[1]),p(a[2]),d(a[3]),if(a[4]=="-")null else v(a[4]));chosen?.raw()?:"-"}
 "now"->{chosen=jumps.takingOffNow(motion,f(a[1]),p(a[2]),d(a[3]));chosen?.raw()?:"-"}
 "last"->{chosen=jumps.lastFromHere(motion,f(a[1]),p(a[2]),d(a[3]));chosen?.raw()?:"-"}
 "air"->jumps.airKeys(motion,f(a[1]),p(a[2]),d(a[3]),a[4].toBoolean())?.let{listOf(it.forward,it.strafe,it.margin.raw()).joinToString(",")}?:"-"
 "takeOff"->{val path=arrayListOf(Vec3(9.0,8.0,7.0));val success=chosen?.let{jumps.takeOff(motion,if(a[1]=="-")null else f(a[1]),p(a[2]),d(a[3]),it,path)};listOf(success?:"-",path.joinToString(";"){it.raw()},motion.raw()).joinToString("~")}
 "duration"->chosen?.ticks(v(a[1]))?.raw()?:"-"
 "lift"->targets.lift(d(a[1])).raw()
 "targets"->{val found=ArrayList<String>();var count=0;targets.forEach(p(a[1]).x,p(a[1]).y,p(a[1]).z,d(a[2]),a[3].toInt(),a[4].toInt(),d(a[5]),a[6].toBoolean()){x,y,z,top->val pos=BlockPos(x,y,z);found+=pos.raw()+":"+top.raw()+if(a[8].toBoolean())":"+(jumps.plan(p(a[1]),d(a[2]),pos,top,a[6].toBoolean())?.raw()?:"-")else"";count++;count<a[7].toInt()};found.joinToString("@")}
 "leap"->{val input=chosen?.let{leap.decide(motion,f(a[1]),step(a,2),it,jumps)};listOf(input?.raw()?:"-",leap.target?.raw()?:"-",leap.sprint,leap.hopeless).joinToString("~")}
 "assist"->{val before=assist.target;val keys=LeapInput(a[4].toInt(),a[5].toInt(),a[6].toBoolean(),a[7].toBoolean(),a[8].toBoolean());val input=assist.tick(motion,f(a[1]),f(a[2]),keys,f(a[3]),jumps,targets);listOf(input?.raw()?:"-",assist.raw(),before===assist.target,assist.aimYRot(motion.position,motion.deltaMovement,motion.onGround).raw()).joinToString("~")}
 "trace"->{assist.trace(motion,f(a[1]),f(a[2]),f(a[3]),jumps,targets);assist.raw()}
 "release"->{assist.release();assist.raw()}
 "resetAssist"->{assist.reset();assist.raw()}
 "landing"->{val j=chosen!!;val land=AssistedJump.Landing(p(a[1]),d(a[2]),j,bounces=a[3].toBoolean(),takesOff=a[4].toBoolean(),settles=a[5].toBoolean());listOf(land.isLanded(motion.position,motion.onGround),land.aimYRot(motion.position,motion.deltaMovement,motion.onGround,if(a[6]=="-")land.aim else v(a[6])).raw(),land.steer(motion,f(a[7]),jumps)?.raw()?:"-").joinToString("~")}
 "math"->listOf(kotlin.math.atan2(d(a[1]),d(a[2])).raw(),(kotlin.math.atan2(d(a[1]),d(a[2]))*Mth.RAD_TO_DEG-90.0).toFloat().raw(),Vec3(d(a[2]),0.0,d(a[1])).yRot.raw()).joinToString(",")
 else->error(a.joinToString(" "))
}
