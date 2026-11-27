import java.lang.reflect.Proxy
import net.ccbluex.agent.navigation.*
import net.ccbluex.agent.navigation.walk.*
import net.ccbluex.agent.pathfinder.walk.*
import net.ccbluex.agent.simulation.*
import net.ccbluex.agent.world.entity.*
import it.unimi.dsi.fastutil.longs.LongOpenHashSet
import java.util.function.ToDoubleFunction
import org.slf4j.Logger
import net.ccbluex.agent.math.ViewRotation
// fixture code
private fun f(raw:String)=Float.fromBits(raw.toUInt().toInt())
private fun Float.raw()=toRawBits().toUInt().toString()
private data class Facts(var tick:Int=0,var pos:Vec3=Vec3.ZERO,var delta:Vec3=Vec3.ZERO,var speed:Double=.1,var strength:Double=.42,var boost:Float=0f,var flags:Int=1,var food:Int=20,var air:Int=300,var maxAir:Int=300,var eye:Vec3=Vec3.ZERO,var support:BlockPos?=null,var below:BlockPos=BlockPos(0,-1,0),var delay:Int=0,var yaw:Float=0f)
private var facts=Facts()
private lateinit var liveWorld:World
private val border=Proxy.newProxyInstance(WorldBorder::class.java.classLoader,arrayOf(WorldBorder::class.java)){_,method,args->when(method.name){"isWithinBounds"->liveWorld.isInsideBorder(args!![0]as Double,args[1]as Double);else->error(method.name)}}as WorldBorder
private val liveLevel=Proxy.newProxyInstance(Level::class.java.classLoader,arrayOf(Level::class.java)){_,method,args->when(method.name){
 "getBlockState"->liveWorld.getBlockState(args!![0]as BlockPos)
 "isKnown"->liveWorld.isKnown(args!![0]as Int,args[1]as Int,args[2]as Int)
 "isInsideBorder"->liveWorld.isInsideBorder(args!![0]as Double,args[1]as Double)
 "getMinY"->liveWorld.minY;"getHeight"->liveWorld.height;"getMaxY"->liveWorld.maxY
 "hasChunkAt"->{events+="H:${args!![0]},${args[1]}";true};"getWorldBorder"->border
 "getMechanics"->{events+="m";liveWorld.mechanics}
 "clip"->targetClip(args!![0]as ClipContext)
 else->error("Level.${method.name}")
}}as Level
private val food=Proxy.newProxyInstance(FoodData::class.java.classLoader,arrayOf(FoodData::class.java)){_,method,_->when(method.name){"getFoodLevel"->{events+="F";facts.food};else->error(method.name)}}as FoodData
private fun makePlayer(read:()->Facts)=Proxy.newProxyInstance(Player::class.java.classLoader,arrayOf(Player::class.java)){_,method,args->val facts=read();when(method.name){
 "position"->{events+="p";facts.pos};"getY"->{events+="Y";facts.pos.y};"getDeltaMovement"->{events+="d";facts.delta};"getTickCount"->{events+="t";facts.tick}
 "getAttributeValue"->{if(args!![0]===Attributes.MOVEMENT_SPEED){events+="a";facts.speed}else if(args[0]===Attributes.JUMP_STRENGTH){events+="j";facts.strength}else if(args[0]===Attributes.SNEAKING_SPEED){events+="n";sneakSpeed}else if(args[0]===Attributes.SAFE_FALL_DISTANCE){events+="safe";captureSafe}else error("attr")}
 "getJumpBoostPower"->{events+="b";facts.boost};"isSprinting"->{events+="s";facts.flags and 2!=0};"onGround"->{events+="g";facts.flags and 1!=0}
 "getHorizontalCollision"->{events+="h";facts.flags and 4!=0};"isCrouching"->{events+="c";facts.flags and 8!=0};"isInWater"->{events+="w";facts.flags and 16!=0}
 "onClimbable"->{events+="k";facts.flags and 32!=0};"isUsingItem"->{events+="u";facts.flags and 64!=0};"isEyeInWater"->{events+="e";facts.flags and 128!=0}
 "getAirSupply"->{events+="A";facts.air};"getMaxAirSupply"->{events+="M";facts.maxAir};"getEyePosition"->{events+="E";facts.eye}
 "getFoodData"->{events+="f";food};"getMainSupportingBlockPos"->{events+="S";facts.support};"getBlockPosBelowThatAffectsMyMovement"->{events+="B";facts.below}
 "getNoJumpDelay"->{events+="J";facts.delay};"level"->{events+="l";liveLevel}
 "blockInteractionRange"->{events+="r";targetReach};"blockPosition"->{events+="V";BlockPos.containing(facts.pos)}
 "getUuid","getUUID"->{events+="U";UUID(targetUuid,0)};"getInventory"->{events+="N";targetInventory}
 "getHealth"->{events+="health";captureHealth}
 "getBoundingBox"->{events+="box";actionBox}
 "getPosition"->{events+="head:${(args!![0]as Float).raw()}";facts.pos}
 else->error("Player.${method.name}")
}}as Player
private val livePlayer=makePlayer{facts}
private var exact=true
private var sent=ViewRotation.ZERO
private var bridgeMethod=Bridging.AUTO
private var risky=false
private var sneakSpeed=.3
private var movingFrames=emptyList<Triple<Vec3,Vec3,Int>>()
private var stillFrames=emptyList<Triple<Vec3,Vec3,Int>>()
private fun frames(raw:String)=if(raw=="-")emptyList()else raw.split(';').map{val q=it.split('^');Triple(v(q[0]),v(q[1]),q[2].toInt())}
private fun SimulatedInput.raw()="${keys.hashCode()}:$forceStayOnGroundSurface:$ignoresEdgeBackOff"
private class ScriptSimulation(first:SimulatedInput):PlayerSimulation.Steered {
 private val data=facts.copy();private val frames=if(first.keys.forward||first.keys.backward||first.keys.left||first.keys.right)movingFrames else stillFrames;private var index=0;private var held=first
 override var input:SimulatedInput get()=held;set(value){events+="I:${value.raw()}";held=value}
 override val player=makePlayer{data}
 override val predictedPos:Vec3 get(){events+="Q";return data.pos}
 override val hasBackedOffEdge get()=data.flags and 256!=0
 override fun tick(){events+="T";if(frames.isNotEmpty()){val next=frames[minOf(index,frames.size-1)];data.pos=next.first;data.delta=next.second;data.flags=next.third;index++}}
}
private val liveBody=Proxy.newProxyInstance(Body::class.java.classLoader,arrayOf(Body::class.java)){_,method,args->when(method.name){
 "getPlayer"->livePlayer;"getLevel"->liveLevel;"getMovementYRot"->{events+="y";facts.yaw};"getInputYRot"->{events+="i";facts.yaw}
 "getTraits"->{events+="X";BodyTraits(exactSimulation=exact,jumpClimbsLadders=captureJumpClimbs)}
 "getSentRotation"->{events+="R";sent}
 "getViewStep"->{events+="vstep";targetViewStep}
 "look"->{events+="L:${(args!![0]as ViewRotation).raw()}:${(args[1]as LookPurpose).name}";null}
 "simulate"->{val input=args!![0]as SimulatedInput;events+="sim:${input.raw()}:${(args[1]as Float).raw()}";ScriptSimulation(input)}
 "selectHotbarSlot"->{events+="select:${args!![0]}";null};"releaseHotbarSlot"->{events+="release";null}
 "startDestroy"->{events+="start:${(args!![0]as BlockPos).raw()}:${(args[1]as Direction).name}";actionStart}
 "continueDestroy"->{events+="continue:${(args!![0]as BlockPos).raw()}:${(args[1]as Direction).name}";actionContinue}
 "stopDestroy"->{events+="stop";null}
 "useItemOn"->{val hit=args!![1]as BlockHitResult;events+="use:${(args[0]as InteractionHand).name}:${hit.blockPos.raw()}:${hit.direction.name}:${hit.location.raw()}";actionUse}
 "getLatencyMillis"->{events+="latency";actionLatency}
 else->error("Body.${method.name}")
}}as Body
private val log=Proxy.newProxyInstance(Logger::class.java.classLoader,arrayOf(Logger::class.java)){_,method,args->
 if(method.name=="warn")actionWarning(args)
 if(method.name=="debug"&&(args!![0]as String).startsWith("Crossing a bridge"))events+="cross-start:${args[1]}:${args[2]}"
 if(method.name=="info")when {
  (args!![0]as String).startsWith("Fell off")->events+="fell:${(args[1]as BlockPos).raw()}"
  (args[0]as String).contains("falls back")->events+="cross-fallback:${args[1]}:${args[2]}"
  (args[0]as String).contains("hops to reset")->events+="cross-hop:${args[1]}:${args[2]}"
 }
 if(method.returnType==Boolean::class.javaPrimitiveType)false else null
}as Logger
private val listener=object:Listener{
 override fun blockPlaced(block:PlacedBlock)=actionEvent("placed",block)
 override fun blockKept(block:PlacedBlock)=actionEvent("kept",block)
 override fun blockRejected(block:PlacedBlock)=actionEvent("rejected",block)
 override fun stepReached(index:Int,pos:BlockPos){events+="reached:$index:${pos.raw()}"}
}
private val ctx=NavigationContext(liveBody,RouteReport(listener),{events+="o";Options(canBreak=captureCanBreak,canPlace=captureCanPlace,canParkour=captureParkour,riskyJumps=captureRiskyJumps,maxFall=captureMaxFall,fastBridges=captureFast,bridging=bridgeMethod,riskyBridges=risky,placeableBlocks=targetPolicy,placesBridgeFloors=externalFloors,breakableBlocks=captureBreakable,reserved=captureReserved)},listener,log)
private var stride=Stride();private var overshoot=Overshoot(ctx);private var swim=Swimming(ctx);private var progress=StepProgress();private var setbacks=Setbacks();private var stuck=StuckSpots();private var hops=Hops(ctx,stride)
private var actions=BlockActions(ctx);private var check=StepCheck(ctx,actions)
private var telly=Telly(ctx,actions,check);private var crossing=Crossing(ctx,actions,check,stride)
private val steps=HashMap<Int,Step>();private var route:Route?=null
private fun ps(raw:String)=if(raw=="-")emptyList()else raw.split(';').map(::p)
private fun ab()=WalkAbilities(false,false,0,true,false,true,3,16,ToDoubleFunction{1.0},LongOpenHashSet())
private fun field(target:Any,name:String):Any? {var type:Class<*>?=target.javaClass;while(type!=null){try{return type.getDeclaredField(name).also{it.isAccessible=true}.get(target)}catch(_:NoSuchFieldException){type=type.superclass}};error(name)}
private fun setField(target:Any,name:String,value:Any?)=target.javaClass.getDeclaredField(name).also{it.isAccessible=true}.set(target,value)
private fun ViewRotation.raw()="${yRot.raw()},${xRot.raw()}"
private fun currentAction():String {val action=field(actions,"action")?:return "-";return "${field(action,"pos").let{it as BlockPos}.raw()}:${field(action,"ready")}:${(field(action,"held")as ViewRotation?)?.raw()?:"-"}"}
private fun crossingRaw()=listOf(crossing.method?.name?:"-",crossing.view?.raw()?:"-",crossing.acting?.raw()?:"-",field(crossing,"rejections"),field(crossing,"placed"),field(crossing,"hopTicks"),field(crossing,"doubleTapTicks"),field(crossing,"isHopping"),field(crossing,"refused")).joinToString("/")
private fun snapshot()=listOf(stride.factor.raw(),stride.moved.raw(),overshoot.count,progress.watch.idleTicks,progress.acting?.raw()?:"-",progress.jumpTicks,progress.isDueToPredict,hops.aim?.raw()?:"-",field(swim,"breathing"),stuck.version,stuck.blocks.toLongArray().sorted().joinToString(",")).joinToString("^")
private fun extra(a:List<String>,world:World,terrain:Terrain):String {
 liveWorld=world
 val result=when(a[0]){
 "cnew"->{facts=Facts();stride=Stride();overshoot=Overshoot(ctx);swim=Swimming(ctx);progress=StepProgress();setbacks=Setbacks();stuck=StuckSpots();hops=Hops(ctx,stride);actions=BlockActions(ctx);check=StepCheck(ctx,actions);steps.clear();route=null;"-"}
 "player"->{facts=Facts(a[1].toInt(),v(a[2]),v(a[3]),d(a[4]),d(a[5]),f(a[6]),a[7].toInt(),a[8].toInt(),a[9].toInt(),a[10].toInt(),v(a[11]),if(a[12]=="-")null else p(a[12]),p(a[13]),a[14].toInt(),f(a[15]));"-"}
 "step"->{val flags=a[8].toInt();steps[a[1].toInt()]=Step(p(a[2]),p(a[3]),Move.valueOf(a[4]),v(a[5]),ps(a[6]),ps(a[7]),ps(a[9]),0.0,shift=flags and 1!=0,edge=flags and 2!=0,isUnderWater=flags and 4!=0,jump=if(flags and 8!=0)Jump(v(a[10]),0,true,1,0.0,0,Vec3.ZERO)else null);"-"}
 "route"->{route=Route(v(a[1]),if(a[2]=="-")emptyList()else a[2].split(',').map{steps.getValue(it.toInt())},true,ab());"-"}
 "stride"->{stride.tick(livePlayer);"-"}
 "abilities"->{val chosen=ab().withMotion(MotionTraits(f(a[1]),f(a[2]),f(a[3]),if(world.legacy)Mechanics.V1_8 else Mechanics.MODERN));val actual=stride.abilities(chosen,livePlayer);listOf(actual.motion.walkingSpeed.raw(),actual.motion.jumpStrength.raw(),actual.motion.jumpBoostPower.raw(),actual.motion.mechanics.name,actual.motion.jumpPower.raw()).joinToString(",")}
 "center"->centerInput(v(a[1]),v(a[2]),f(a[3])).hashCode().toString()
 "stepfacts"->{val s=steps.getValue(a[1].toInt());listOf(s.builds,s.isLevelWalk,s.isRisingBridge,s.isPlain,s.pastEdge(v(a[2])).raw(),s.holdBack(v(a[2]),v(a[3]),f(a[4])).hashCode(),hasPassed(s,v(a[2])),wantsJump(s,v(a[2]),a[5].toBoolean(),a[6].toBoolean())).joinToString(",")}
 "straight"->continuesStraight(steps.getValue(a[1].toInt()),steps.getValue(a[2].toInt())).toString()
 "run"->runEnd(route!!.steps,a[1].toInt()).toString()
 "off"->route!!.isOffPath(a[1].toInt(),v(a[2])).toString()
 "join"->route!!.joinIndex(v(a[1]))?.toString()?:"-"
 "look"->route!!.lookAhead(a[1].toInt(),v(a[2]),v(a[3]),d(a[4]))?.raw()?:"-"
 "point"->route!!.pointAhead(a[1].toInt(),v(a[2]),d(a[3])).raw()
 "turn"->route!!.turnAhead(a[1].toInt(),v(a[2]),d(a[3])).raw()
 "bridge"->Bridge.at(route!!.steps,a[1].toInt())?.let{listOf(it.start,it.end,it.to.raw(),it.rises,it.isStraight,it.yRot.raw(),it.turnAfter,it.length).joinToString(",")}?:"-"
 "line"->{val line=BridgeLine(v(a[1]),v(a[2]));listOf(line.heading.raw(),line.aimFrom(v(a[3])).raw()).joinToString(",")}
 "overshoot"->{val s=steps.getValue(a[1].toInt());overshoot.forwardAim(v(a[2]),v(a[3]),s){pos,dist->events+="P:${pos.raw()}:${dist.raw()}";if(a[4]=="-")null else v(a[4])}.raw()}
 "breathes"->swim.breathes().toString()
 "shortair"->swim.isShortOfAir().toString()
 "swim"->{val keys=MovementKeys();val m=swim.swim(keys,f(a[2]),steps.getValue(a[1].toInt()));listOf(keys.toInput(false).hashCode(),m.target?.raw()?:"-",m.sprint,m.dive,m.isPlainWalk,m.safeWalk).joinToString("~")}
 "climb"->{val keys=MovementKeys();val m=ctx.climbUp(keys,f(a[2]),steps.getValue(a[1].toInt()));listOf(keys.toInput(false).hashCode(),m.target?.raw()?:"-",m.sprint,m.dive,m.isPlainWalk,m.safeWalk).joinToString("~")}
 "hop"->hops.starts(v(a[1]),v(a[2])).toString()
 "clearhop"->{hops.clear();"-"}
 "reached"->check.isReached(steps.getValue(a[1].toInt()),if(a[2]=="-")null else steps.getValue(a[2].toInt())).toString()
 "skip"->check.skipAhead(route!!,a[1].toInt()).toString()
 "confirm"->{@Suppress("UNCHECKED_CAST")val map=field(actions,"unconfirmed")as MutableMap<BlockPos,Any?>;if(a[2].toBoolean())map[p(a[1])]=null else map.remove(p(a[1]));"-"}
 "wants"->wantsJump(steps.getValue(a[1].toInt()),livePlayer).toString()
 "stairs"->leadsOntoStairs(steps.getValue(a[1].toInt()),liveLevel).toString()
 "act"->progress.actsTooLong(p(a[1])).toString()
 "unact"->{progress.acting=null;"-"}
 "watch"->progress.watch.stalls(d(a[1])).toString()
 "excuse"->{progress.watch.excuse();"-"}
 "refuse"->setbacks.refused(v(a[1]),a[2].toLong()).joinToString(";"){it.raw()}
 "stuck"->stuck.add(p(a[1])).toString()
 "forget"->{stuck.clear();"-"}
 "goal"->{stuck.tick(v(a[1]));"-"}
 "script"->{exact=a[1].toBoolean();movingFrames=frames(a[2]);stillFrames=frames(a[3]);"-"}
 "predict"->{val keys=maskKeys(a[2]);val flags=a[5].toInt();ctx.getsAhead(keys,f(a[3]),steps.getValue(a[1].toInt()),Movement(if(a[4]=="-")null else v(a[4]),flags and 1!=0,flags and 2!=0,flags and 4!=0,flags and 8!=0)).toString()}
 "floor"->{val keys=maskKeys(a[2]);val result=steps.getValue(a[1].toInt()).canClickFloor(liveBody,keys,f(a[3]),a[4].toBoolean());"$result,${keys.toInput(false).hashCode()}"}
 "correct"->getDirectionalInputForYRot(a[1].toInt(),a[2].toInt(),f(a[3]),if(a[4]=="-")null else f(a[4])).hashCode().toString()
 "leaping"->{val keys=MovementKeys();val s=steps.getValue(a[1].toInt());val measured=ab().withMotion(MotionTraits(.1f,.42f,0f,if(world.legacy)Mechanics.V1_8 else Mechanics.MODERN));val m=ctx.leap(keys,f(a[2]),s,s.jump!!,measured,progress.leap,stride);listOf(keys.toInput(false).hashCode(),m.target?.raw()?:"-",m.sprint,m.dive,m.isPlainWalk,m.safeWalk).joinToString("~")+"~"+progress.leap.hopeless}
 "rotation"->ViewRotation.fromVector(v(a[1])).raw()
 "viewvector"->ViewRotation(f(a[1]),f(a[2])).viewVector.raw()
 "turning"->{val from=ViewRotation(f(a[1]),f(a[2]));val to=ViewRotation(f(a[3]),f(a[4]));val base=when(a[5]){"instant"->Turning.instant();"linear"->Turning.linear(f(a[6]),f(a[7]));else->Turning.eased(f(a[6]),f(a[7]),f(a[8]),f(a[9]))};(if(a[10].toBoolean())Turning.humanPaced(base)else base).turn(from,to,LookPurpose.valueOf(a[11])).raw()}
 "bridgecadence"->{val method=Bridging.valueOf(a[1]);listOf(method.xRot.raw(),method.yRotOffset.raw(),method.clicksPastEdge,method.sneaksThroughout,method.hopsOnReset,method.resets(a[2].toInt()),method.tap(a[2].toInt(),d(a[3]),a[4].toInt())).joinToString(",")}
 "bridgepick"->{val method=Bridging.valueOf(a[1]);val bridge=Bridge.at(route!!.steps,a[2].toInt())!!;"${method.canCross(bridge)},${method.pick(bridge,a[3].toBoolean(),d(a[4]),a[5].toBoolean()).name}"}
 "bridgenew"->{sent=ViewRotation(f(a[1]),f(a[2]));bridgeMethod=Bridging.valueOf(a[3]);risky=a[4].toBoolean();sneakSpeed=d(a[5]);telly=Telly(ctx,actions,check);crossing=Crossing(ctx,actions,check,stride);"-"}
 "bridgesent"->{sent=ViewRotation(f(a[1]),f(a[2]));"-"}
 "telly"->{val keys=maskKeys(a[3]);val bridge=Bridge.at(route!!.steps,a[1].toInt())!!;val m=telly.tick(keys,f(a[2]),route!!.steps,bridge,f(a[4]));listOf(keys.toInput(false).hashCode(),m.target?.raw()?:"-",m.sprint,m.dive,m.isPlainWalk,m.safeWalk,telly.view?.raw()?:"-",telly.acting?.raw()?:"-",currentAction()).joinToString("~")}
 "takes"->{val result=crossing.takes(route!!.steps,a[1].toInt());"$result~${crossingRaw()}"}
 "cross"->{val keys=maskKeys(a[3]);val m=crossing.tick(keys,f(a[2]),route!!.steps,a[1].toInt());listOf(keys.toInput(false).hashCode(),m.target?.raw()?:"-",m.sprint,m.dive,m.isPlainWalk,m.safeWalk,crossingRaw(),currentAction()).joinToString("~")}
 "crossclear"->{crossing.clear();crossingRaw()}
 "reject"->{setField(actions,"rejections",a[1].toInt());"-"}
 "placed"->{setField(actions,"placement",if(a[1]=="-")null else PlacedBlock(p(a[1]),a[2].toInt()));"-"}
 "straightwalk"->isStraightWalkable(terrain,v(a[1]),v(a[2]),d(a[3]),a[4].toInt()).toString()
 "shortcut"->route!!.shortcut(a[1].toInt(),v(a[2]),terrain)?.raw()?:"-"
 else->if(a[0].startsWith("a"))actionsExtra(a)else if(a[0].startsWith("f"))followerExtra(a)else if(a[0].startsWith("cap"))captureExtra(a)else targetsExtra(a)
 }
 return result+"|"+snapshot()
}
private fun maskInput(mask:Int)=Input(mask and 64!=0,mask and 32!=0,mask and 16!=0,mask and 8!=0,mask and 4!=0,mask and 2!=0,mask and 1!=0)
private fun maskKeys(raw:String)=maskInput(raw.toInt()).let{input->MovementKeys().also{it.directionalInput=input;it.jump=input.jump;it.sneak=input.shift}}
