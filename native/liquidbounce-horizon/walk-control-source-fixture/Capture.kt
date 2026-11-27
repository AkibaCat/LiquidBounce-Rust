// fixture code
private var captureCanBreak=false
private var captureCanPlace=false
private var captureParkour=true
private var captureRiskyJumps=false
private var captureMaxFall=3
private var captureFast=false
private var captureSafe=3.0
private var captureHealth=20f
private var captureJumpClimbs=true
private var captureAllowsBreak=true
private val captureHardness=HashMap<Int,Pair<Float,Boolean>>()
private val captureBreakable=BreakableBlocks{state->events+="breakable:${(state as S).id}";captureAllowsBreak}
private var captureReserved=LongOpenHashSet()
private val captureAvoid=LongOpenHashSet()
private val captureUnbreakable=LongOpenHashSet()
private var captured:WalkAbilities?=null
private fun WalkAbilities.captureRaw()=listOf(canBreak,canPlace,placeableBlocks,canParkour,riskyJumps,canSprint,safeFall,waterFall,air,motion.walkingSpeed.raw(),motion.jumpStrength.raw(),motion.jumpBoostPower.raw(),motion.mechanics.name,jumpClimbs,bridgeTicks.raw(),health.raw(),avoid.toLongArray().sorted().joinToString(","),unbreakable.toLongArray().sorted().joinToString(","),reserved.toLongArray().sorted().joinToString(",")).joinToString("~")
private fun longSet(raw:String)=LongOpenHashSet(if(raw=="-")longArrayOf()else raw.split(',').map{it.toLong()}.toLongArray())
private fun captureExtra(a:List<String>):String=when(a[0]){
 "capopts"->{val mask=a[1].toInt();captureCanBreak=mask and 1!=0;captureCanPlace=mask and 2!=0;captureParkour=mask and 4!=0;captureRiskyJumps=mask and 8!=0;captureFast=mask and 16!=0;risky=mask and 32!=0;captureMaxFall=a[2].toInt();captureSafe=d(a[3]);captureHealth=f(a[4]);captureJumpClimbs=a[5].toBoolean();bridgeMethod=Bridging.valueOf(a[6]);captureAllowsBreak=a[7].toBoolean();"-"}
 "capsets"->{captureAvoid.clear();captureAvoid.addAll(longSet(a[1]));captureUnbreakable.clear();captureUnbreakable.addAll(longSet(a[2]));captureReserved=longSet(a[3]);"-"}
 "capstack"->{val stack=targetStacks.getValue(a[1].toInt());stack.captureCount=a[2].toInt();stack.captureEmpty=a[3].toBoolean();"-"}
 "caphard"->{captureHardness[a[1].toInt()]=f(a[2])to a[3].toBoolean();"-"}
 "capwalk"->{captured=ctx.captureWalk(captureAvoid,captureUnbreakable);captured!!.captureRaw()}
 "capquery"->captured!!.captureRaw()
 "capticks"->captured!!.breakTicks.applyAsDouble(states.getValue(a[1].toInt())).raw()
 "capsafe"->ctx.safeFall().toString()
 "caphot"->ctx.hotbar().joinToString(","){(it as TargetStack).id.toString()}
 "capbreak"->{val times=BreakTimes.capture(ctx.hotbar());times.applyAsDouble(states.getValue(a[1].toInt())).raw()}
 else->error(a.joinToString(" "))
}
