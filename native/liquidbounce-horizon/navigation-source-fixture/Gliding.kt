import java.util.concurrent.Executor
import net.ccbluex.agent.pathfinder.route.*
import net.ccbluex.agent.pathfinder.glide.*
import net.ccbluex.agent.world.item.*
// fixture code
object GlideProbe {
 val events get()=RuntimeProbe.events
 fun raw(v:Double)=v.toRawBits().toULong().toString()
 fun vector(v:Vec3)=RuntimeProbe.vector(v)
 var planProgram=listOf("F");var airProgram=listOf("F")
 var points=listOf(Vec3(.5,10.0,.5),Vec3(40.5,0.0,.5));var routeRockets=0;var approach=0.0;var reaches=true
}
private class Queue:Executor{val tasks=ArrayDeque<Runnable>();override fun execute(task:Runnable){tasks+=task};fun run(){tasks.removeFirstOrNull()?.run()}}
private var glideQueue=Queue()
private var runtimeDelta=Vec3.ZERO
private var runtimeCeiling=false
private var runtimeHeightMode=0
private var runtimeSafeFall=3
fun runtimeFall():Int{events+="S";return runtimeSafeFall}
private var runtimePresent=true
private var runtimeOffhand=0
private var runtimeChest=1
private var runtimeHotbar=List(9){0}
private val runtimeStacks=HashMap<Int,Stack>()
private val runtimeInventory=Proxy.newProxyInstance(Inventory::class.java.classLoader,arrayOf(Inventory::class.java)){_,m,a->when(m.name){"getItem"->{val slot=a!![0]as Int;events+="I:$slot";runtimeStacks.getValue(runtimeHotbar[slot])};else->error(m.name)}}as Inventory
private fun runtimeHeight(x:Int,z:Int):Int{events+="Z:$x,$z";return when(runtimeHeightMode){0->0;1->120;else->if(x<0)20 else 0}}
private fun snapshotRuntime():World=World().also{copy->copy.min=runtimeWorld.min;copy.count=runtimeWorld.count;copy.borderMin=runtimeWorld.borderMin;copy.borderMax=runtimeWorld.borderMax;copy.legacy=runtimeWorld.legacy;copy.floorY=runtimeWorld.floorY;copy.floorState=runtimeWorld.floorState;copy.blocks.putAll(runtimeWorld.blocks);copy.unknown.addAll(runtimeWorld.unknown)}
private fun createGliding():Gliding {
 val gliding=Gliding(runtimeContext)
 gliding.javaClass.getDeclaredField("search").also{it.isAccessible=true}.set(gliding,BackgroundSearch<Route>({events+="L";if(runtimePresent)runtimeLevel else null},executor=glideQueue))
 val airways=field(gliding,"airways")!!
 airways.javaClass.getDeclaredField("routes").also{it.isAccessible=true}.set(airways,Replanner<net.ccbluex.agent.pathfinder.flight.Route>({events+="L";if(runtimePresent)runtimeLevel else null},64,executor=glideQueue))
 return gliding
}
private var gliding:Gliding?=null
private fun glidingState():String {
 val g=gliding!!;val glide=field(g,"glide")
 val state=if(glide==null)"-" else {
  val s=field(glide,"steering")as Steering;val path=field(glide,"path")as Route?
  val rockets=when(s){is Landing->s.rockets;is Passage->s.rockets;is Cruise->(field(s,"landing")as Landing?)?.rockets?:field(s,"rockets")as Rockets;else->error(s)}
  val passage=if(s is Passage)listOf(s.landsAtTarget,s.drift.raw(),s.remaining.raw(),s.route.joinToString(";"){it.raw()}).joinToString("~")else "-"
  listOf((field(glide,"goal")as Vec3).raw(),field(glide,"ticks"),field(glide,"hasOpened"),field(glide,"pressedAt"),field(glide,"jumped"),s.javaClass.simpleName,s.target.raw(),(path?.landing?:s.target).raw(),path?.points?.joinToString(";"){it.raw()}?:"-","${rockets.left},${rockets.boostTicks},${rockets.canFire}",passage).joinToString("^")
 }
 val search=field(g,"search")as BackgroundSearch<*>;val a=field(g,"airways")!!;val r=field(a,"routes")as Replanner<*>
 return listOf(field(g,"ticks"),field(g,"retryAt"),field(g,"shouldFireRocket"),field(g,"holdsRocketSlot"),g.isActive,g.sprint,state,search.isIdle,search.preview?.joinToString(";"){it.raw()}?:"-",field(a,"hasRoute"),field(a,"reachesGoal"),field(a,"foundAt"),r.ticks,r.failed,r.endsAtStage,r.isSearching,r.isDue(false),r.isDue(true),glideQueue.tasks.size).joinToString("|")
}
private fun extra(a:List<String>,world:World,terrain:Terrain):String {
 runtimeWorld=world;RuntimeProbe.events=events
 return when(a[0]){
  "runtime"->{baseExtra(a,world,terrain);glideQueue=Queue();gliding=createGliding();runtimePresent=true;runtimeCeiling=false;runtimeHeightMode=0;runtimeSafeFall=3;runtimeContext.current=RuntimeOptions();"-"}
  "stack"->{val id=a[1].toInt();runtimeStacks[id]=Stack(id,a[2].toBoolean(),a[3].toBoolean(),a[4].toInt(),a[5].toInt(),a[6].toBoolean(),a[7].toInt(),a[8].takeIf{it!="-"}?.toInt());"-"}
  "equipment"->{runtimeOffhand=a[1].toInt();runtimeChest=a[2].toInt();runtimeHotbar=a[3].split(',').map(String::toInt);"-"}
  "options"->{runtimeContext.current=RuntimeOptions(a[1].toBoolean(),a[2].toBoolean(),d(a[3]));runtimeSafeFall=a[4].toInt();"-"}
  "capture"->runtimeContext.captureGlide()?.let{"${it.rockets},${it.rocketTicks},${it.glideTicks}"}?:"-"
  "delta"->{runtimeDelta=v(a[1]);"-"}
  "environment"->{runtimeCeiling=a[1].toBoolean();runtimeHeightMode=a[2].toInt();runtimePresent=a[3].toBoolean();"-"}
  "program"->{GlideProbe.planProgram=a[1].split(',');GlideProbe.airProgram=a[2].split(',');GlideProbe.points=a[3].split(';').map(::v);GlideProbe.routeRockets=a[4].toInt();GlideProbe.approach=d(a[5]);GlideProbe.reaches=a[6].toBoolean();"-"}
  "nav"->{val result=gliding!!.navigate(v(a[1]),d(a[2]),if(a[3]=="-")null else a[3].split(',').map(String::toInt).let{GlideAbilities(it[0],it[1],it[2])},a[4].toBoolean(),Float.fromBits(a[5].toUInt().toInt()));"$result#${glidingState()}"}
  "glideHalt"->{gliding!!.halt();glidingState()}
  "run"->{glideQueue.run();glidingState()}
  "glideLevel"->{gliding!!.levelChanged();glidingState()}
  "glideChange"->{gliding!!.blockChanged(p(a[1]));glidingState()}
  else->baseExtra(a,world,terrain)
 }
}
