package fixture
import com.google.gson.*
import java.nio.file.*
import net.ccbluex.liquidbounce.config.types.group.*
import net.ccbluex.liquidbounce.features.module.modules.movement.highjump.ModuleHighJump
import net.ccbluex.liquidbounce.event.*
import net.ccbluex.liquidbounce.event.events.*
class Vector(var y:Double)
class Player{var deltaMovement=Vector(-0.0);var fallDistance=0.0;var tickCount=0;var grounded=false;fun onGround()=grounded}
object Fixture{
 var running=true
 var player=Player()
 lateinit var mode:ModeValueGroup<Mode>
 lateinit var motion:Value<Float>
 lateinit var glide:Value<Boolean>
 fun bits(v:Double)=java.lang.Long.toUnsignedString(java.lang.Double.doubleToRawLongBits(v))
 fun result(m:Float?=null):JsonObject=JsonObject().apply{addProperty("velocity",bits(player.deltaMovement.y));addProperty("shouldGlide",state());if(m!=null)addProperty("motion",java.lang.Integer.toUnsignedString(java.lang.Float.floatToRawIntBits(m)))}
 fun state():Boolean{val vulcan=mode.modes[1];val f=vulcan.javaClass.getDeclaredField("shouldGlide");f.isAccessible=true;return f.getBoolean(vulcan)}
 fun setMode(tag:String){val selected=mode.modes.first{it.name==tag||tag in it.aliases};if(selected!==mode.activeMode){mode.activeMode.cancelScope();mode.activeMode=selected}}
 @Suppress("UNCHECKED_CAST")
 @JvmStatic fun main(args:Array<String>){
  val module=ModuleHighJump
  mode=module.children.filterIsInstance<ModeValueGroup<Mode>>().single()
  motion=module.children.filterIsInstance<Value<Float>>().single{it.name=="Motion"}
  glide=mode.modes[1].children.filterIsInstance<Value<Boolean>>().single()
  val inputs=JsonParser.parseString(Files.readString(Path.of(args[0]))).asJsonArray
  val output=JsonArray()
  for(case in inputs){
   for(m in mode.modes)m.cancelScope()
   running=true;setMode("Vanilla");player=Player();motion.value=0.8f;glide.value=false
   val f=mode.modes[1].javaClass.getDeclaredField("shouldGlide");f.isAccessible=true;f.setBoolean(mode.modes[1],false)
   val steps=JsonArray()
   for(action in case.asJsonArray){val a=action.asJsonObject
    when(a["op"].asString){
     "mode"->{setMode(a["value"].asString);steps.add(result())}
     "running"->{val next=a["value"].asBoolean;if(running&&!next)mode.activeMode.cancelScope();running=next;steps.add(result())}
     "glide"->{glide.value=a["value"].asBoolean;steps.add(result())}
     "motion"->{motion.value=java.lang.Float.intBitsToFloat(java.lang.Integer.parseUnsignedInt(a["bits"].asString));steps.add(result())}
     "jump"->{val e=PlayerJumpEvent(0.42f);emit(e);steps.add(result(e.motion))}
     "velocity"->{player.deltaMovement.y=java.lang.Double.longBitsToDouble(java.lang.Long.parseUnsignedLong(a["bits"].asString));steps.add(result())}
     "tick"->{player.grounded=a["ground"].asBoolean;player.fallDistance=java.lang.Double.longBitsToDouble(java.lang.Long.parseUnsignedLong(a["fall"].asString));player.tickCount=a["count"].asInt
      CoroutineTicker.beginMinecraftTick();CoroutineTicker.tick()
      if(a["nested"]?.asBoolean==true){CoroutineTicker.beginMinecraftTick();CoroutineTicker.tick();CoroutineTicker.endMinecraftTick()}
      emit(GameTickEvent());CoroutineTicker.endMinecraftTick();steps.add(result())}
    }
   };output.add(steps)
  }
  Files.writeString(Path.of(args[1]),Gson().toJson(output));println("untouched HighJump/ticker scenarios="+output.size())
 }
}
