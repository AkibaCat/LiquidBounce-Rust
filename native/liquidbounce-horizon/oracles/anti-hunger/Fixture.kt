package fixture
import com.google.gson.*
import java.nio.file.*
import net.ccbluex.liquidbounce.config.types.group.*
import net.ccbluex.liquidbounce.features.module.modules.exploit.ModuleAntiHunger
import net.ccbluex.liquidbounce.event.emit
import net.ccbluex.liquidbounce.event.events.*
import net.minecraft.network.protocol.game.*
class Player{var isSwimming=false;var isPassenger=false;var isInWater=false;var isUnderWater=false;var fallDistance=0.0;fun jumpFromGround(){Fixture.trace.add("jump")}}
class Interaction{var isDestroying=false}
object Clock{
 var values=listOf(1000L,1000L);var next=0
 @JvmStatic fun currentTimeMillis():Long{val v=values[minOf(next,values.size-1)];next++;Fixture.trace.add("clock:$v");return v}
}
object Fixture{
 var running=true;var player=Player();var interaction=Interaction();val trace=mutableListOf<String>()
 lateinit var noSprint:ToggleableValueGroup
 lateinit var swimming:Value<Boolean>
 lateinit var floating:Value<Boolean>
 lateinit var timer:Any
 lateinit var last:java.lang.reflect.Field
 @Suppress("UNCHECKED_CAST")
 @JvmStatic fun main(args:Array<String>){
  val module=ModuleAntiHunger
  noSprint=module.children.filterIsInstance<ToggleableValueGroup>().single()
  swimming=noSprint.children.filterIsInstance<Value<Boolean>>().single()
  floating=module.children.filterIsInstance<Value<Boolean>>().single()
  val field=module.javaClass.getDeclaredField("notificationCooldown");field.isAccessible=true;timer=field.get(module)
  last=timer.javaClass.getDeclaredField("lastUpdate");last.isAccessible=true
  val output=JsonArray()
  for(case in JsonParser.parseString(Files.readString(Path.of(args[0]))).asJsonArray){
   running=true;player=Player();interaction=Interaction();noSprint.enabled=true;swimming.value=false;floating.value=true;last.setLong(timer,0)
   val steps=JsonArray()
   for(step in case.asJsonArray){val s=step.asJsonObject;trace.clear();Clock.next=0
    Clock.values=s["clock"]?.asJsonArray?.map{it.asString.toLong()}?:listOf(1000L,1000L)
    var sprint:Boolean?=null;var ground:Boolean?=null
    when(s["op"].asString){
     "settings"->{noSprint.enabled=s["noSprint"].asBoolean;swimming.value=s["whileSwimming"].asBoolean;floating.value=s["floating"].asBoolean}
     "running"->running=s["value"].asBoolean
     "last"->last.setLong(timer,s["value"].asString.toLong())
     "context"->{val flags=s["flags"].asInt;player.isSwimming=flags and 1!=0;player.isPassenger=flags and 2!=0;interaction.isDestroying=flags and 4!=0;player.isInWater=flags and 8!=0;player.isUnderWater=flags and 16!=0;player.fallDistance=java.lang.Double.longBitsToDouble(java.lang.Long.parseUnsignedLong(s["fall"].asString))}
     "sprint"->{val e=SprintEvent(s["value"].asBoolean,SprintEvent.Source.valueOf(s["source"].asString));emit(e);sprint=e.sprint}
     "packet"->{val packet=when(s["kind"].asString){
       "startSprint"->ServerboundPlayerCommandPacket(ServerboundPlayerCommandPacket.Action.START_SPRINTING)
       "stopSprint"->ServerboundPlayerCommandPacket(ServerboundPlayerCommandPacket.Action.STOP_SPRINTING)
       "startBreak"->ServerboundPlayerActionPacket(ServerboundPlayerActionPacket.Action.START_DESTROY_BLOCK)
       "stopBreak"->ServerboundPlayerActionPacket(ServerboundPlayerActionPacket.Action.STOP_DESTROY_BLOCK)
       "move"->ServerboundMovePlayerPacket(s["ground"].asBoolean)
       else->Any()
      };emit(PacketEvent(packet,s["cancelled"]?.asBoolean?:false));if(packet is ServerboundMovePlayerPacket)ground=packet.onGround}
    }
    steps.add(JsonObject().apply{addProperty("last",last.getLong(timer).toString());add("trace",Gson().toJsonTree(trace));if(sprint!=null)addProperty("sprint",sprint);if(ground!=null)addProperty("ground",ground)})
   };output.add(steps)
  }
  Files.writeString(Path.of(args[1]),Gson().toJson(output));println("untouched AntiHunger/Chronometer scenarios="+output.size())
 }
}
