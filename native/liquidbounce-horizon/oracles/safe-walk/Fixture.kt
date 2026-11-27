package fixture
import com.google.gson.*
import java.nio.file.*
import java.util.concurrent.ThreadLocalRandom
import net.ccbluex.liquidbounce.config.types.Value
import net.ccbluex.liquidbounce.config.types.group.*
import net.ccbluex.liquidbounce.config.types.list.Tagged
import net.ccbluex.liquidbounce.config.utils.RefreshableFloatState
import net.ccbluex.liquidbounce.event.emit
import net.ccbluex.liquidbounce.event.events.*
import net.ccbluex.liquidbounce.features.module.modules.movement.ModuleSafeWalk
import net.ccbluex.liquidbounce.utils.math.BlockPos
import net.ccbluex.liquidbounce.utils.movement.DirectionalInput
import net.minecraft.world.phys.Vec3
import kotlin.math.floor
class Player{
 var ground=true;var speed=0.0;var pos=Vec3(0.5,1.0,0.5);var yRot=0f
 fun onGround()=ground;fun position()=pos;fun blockPosition()=BlockPos(floor(pos.x).toInt(),floor(pos.y).toInt(),floor(pos.z).toInt())
}
object Fixture{
 var player=Player();var running=true;var edge=false;var fall=false;var next=Vec3(0.5,1.0,0.5);val trace=mutableListOf<String>()
 fun f(v:Float)=java.lang.Integer.toUnsignedString(java.lang.Float.floatToRawIntBits(v))
 fun d(v:Double)=java.lang.Long.toUnsignedString(java.lang.Double.doubleToRawLongBits(v))
 fun number(v:JsonElement)=java.lang.Double.longBitsToDouble(java.lang.Long.parseUnsignedLong(v.asString))
 fun vector(v:JsonElement):Vec3{val a=v.asJsonArray;return Vec3(number(a[0]),number(a[1]),number(a[2]))}
 @Suppress("UNCHECKED_CAST")
 @JvmStatic fun main(args:Array<String>){
  val module=ModuleSafeWalk;val choices=module.children.filterIsInstance<ModeValueGroup<Mode>>().single();val onEdge=choices.modes.single{it.name=="OnEdge"}
  val values=onEdge.children.filterIsInstance<Value<Any>>().associateBy{it.name}
  fun field(name:String)=onEdge.javaClass.getDeclaredField(name).apply{isAccessible=true}
  val center=field("center");val overwrite=field("overwriteTicks");val sneaks=field("sneakTicks");val refresh=field("edgeDistance").get(onEdge)as RefreshableFloatState
  ThreadLocalRandom.current();val seed=Thread::class.java.getDeclaredField("threadLocalRandomSeed").apply{isAccessible=true};val output=JsonArray()
  for(case in JsonParser.parseString(Files.readString(Path.of(args[0]))).asJsonArray){
   running=true;player=Player();edge=false;fall=false;next=player.pos;choices.activeMode=onEdge
   values.getValue("Distance").raw=0.1f..0.15f;values.getValue("Keep").raw=1..2;values.getValue("Sneak").raw=0..0;values.getValue("Jump").raw=false
   val tags=(values.getValue("Mode").raw.javaClass.enumConstants as Array<Any>);values.getValue("Mode").raw=tags.single{(it as Tagged).tag=="Stop"}
   center.set(onEdge,null);overwrite.setInt(onEdge,0);sneaks.setInt(onEdge,0);seed.setLong(Thread.currentThread(),case.asJsonObject["seed"].asString.toLong());refresh.refresh()
   val steps=JsonArray()
   for(step in case.asJsonObject["steps"].asJsonArray){trace.clear();val s=step.asJsonObject;var input:MovementInputEvent?=null;var safe:Boolean?=null;var error:String?=null
    try{when(s["op"].asString){
     "snapshot"->{player.ground=s["ground"].asBoolean;player.speed=number(s["speed"]);player.pos=vector(s["pos"]);player.yRot=java.lang.Float.intBitsToFloat(java.lang.Integer.parseUnsignedInt(s["yaw"].asString));next=vector(s["next"]);edge=s["edge"].asBoolean;fall=s["fall"].asBoolean}
     "input"->{val m=s["mask"].asInt;input=MovementInputEvent(DirectionalInput(m and 1!=0,m and 2!=0,m and 4!=0,m and 8!=0),s["jump"].asBoolean,s["sneak"].asBoolean);emit(input)}
     "safe"->{val e=PlayerSafeWalkEvent(s["value"].asBoolean);emit(e);safe=e.isSafeWalk}
     "running"->{val active=s["value"].asBoolean;if(running&&!active)choices.activeMode.disable();running=active}
     "mode"->choices.activate(choices.modes.single{it.name==s["value"].asString})
     "disable"->onEdge.disable()
     "settings"->{
      s["distance"]?.let{val v=it.asJsonArray;values.getValue("Distance").set(java.lang.Float.intBitsToFloat(java.lang.Integer.parseUnsignedInt(v[0].asString))..java.lang.Float.intBitsToFloat(java.lang.Integer.parseUnsignedInt(v[1].asString)))}
      s["keep"]?.let{val v=it.asJsonArray;values.getValue("Keep").set(v[0].asInt..v[1].asInt)}
      s["sneak"]?.let{val v=it.asJsonArray;values.getValue("Sneak").set(v[0].asInt..v[1].asInt)}
      s["jump"]?.let{values.getValue("Jump").set(it.asBoolean)}
      s["edgeMode"]?.let{val tag=it.asString;values.getValue("Mode").set(tags.single{(it as Tagged).tag==tag})}
     }
     "state"->{s["center"]?.let{center.set(onEdge,if(it.isJsonNull)null else vector(it))};s["overwrite"]?.let{overwrite.setInt(onEdge,it.asInt)};s["sneaks"]?.let{sneaks.setInt(onEdge,it.asInt)}}
    }}catch(e:Exception){error=e.javaClass.simpleName}
    val c=center.get(onEdge)as Vec3?
    val savedSeed=seed.getLong(Thread.currentThread());val nextRandom=ThreadLocalRandom.current().nextLong();seed.setLong(Thread.currentThread(),savedSeed)
    steps.add(JsonObject().apply{addProperty("overwrite",overwrite.getInt(onEdge));addProperty("sneaks",sneaks.getInt(onEdge));addProperty("distance",f(refresh.current));addProperty("seed",java.lang.Long.toUnsignedString(savedSeed));addProperty("threadId",Thread.currentThread().threadId());addProperty("nextRandom",java.lang.Long.toUnsignedString(nextRandom));add("center",if(c==null)JsonNull.INSTANCE else Gson().toJsonTree(listOf(d(c.x),d(c.y),d(c.z))));add("trace",Gson().toJsonTree(trace));if(input!=null){val v=input.directionalInput;addProperty("mask",(if(v.forwards)1 else 0)+(if(v.backwards)2 else 0)+(if(v.left)4 else 0)+(if(v.right)8 else 0));addProperty("jump",input.jump);addProperty("sneak",input.sneak)};if(safe!=null)addProperty("safe",safe);if(error!=null)addProperty("error",error)})
   };output.add(steps)
  }
  Files.writeString(Path.of(args[1]),Gson().toJson(output));println("untouched SafeWalk scenarios="+output.size())
 }
}
