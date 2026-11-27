package fixture
import com.google.gson.*
import java.io.File
import net.ccbluex.liquidbounce.config.types.*
import net.ccbluex.liquidbounce.config.types.group.*
import net.ccbluex.liquidbounce.config.types.list.ListValue
import net.ccbluex.liquidbounce.features.module.modules.misc.ModuleTargetLock
import net.ccbluex.liquidbounce.event.events.*
import net.minecraft.client.player.AbstractClientPlayer
object Clock{@JvmStatic fun currentTimeMillis():Long {Fixture.trace.add("clock");return Fixture.now}}
class World{fun getEntity(id:Int):Any?{Fixture.trace.add("entity:"+id);return Fixture.players[id]}}
object Fixture {
 var running=true;var now=0L;var dead=false;var noticeTime:Int?=null;var noticeClock:Long?=null
 val local=AbstractClientPlayer(0,"Local");val world=World();val players=mutableMapOf<Int,Any>()
 val values=mutableMapOf<String,Value<*>>();lateinit var mode:ModeValueGroup<*>
 val handlers=mutableListOf<Triple<Any,Class<*>,(Any)->Unit>>();val trace=mutableListOf<String>()
 lateinit var temporary:Any
 fun locks():it.unimi.dsi.fastutil.ints.Int2LongLinkedOpenHashMap {
  val field=temporary.javaClass.getDeclaredField("lockList");field.isAccessible=true
  return field.get(temporary) as it.unimi.dsi.fastutil.ints.Int2LongLinkedOpenHashMap
 }
 @Suppress("UNCHECKED_CAST") fun set(key:String,value:Any?){(values.getValue(key) as Value<Any?>).inner=value}
 fun dispatch(event:Any){for((owner,kind,action) in handlers.toList()){if(running&&kind.isInstance(event)&&(owner !is Mode||mode.activeMode===owner))action(event)}}
 fun reset(){running=true;now=0;dead=false;noticeTime=null;noticeClock=null;players.clear();locks().clear();trace.clear()
  @Suppress("UNCHECKED_CAST") val m=mode as ModeValueGroup<Mode>;m.activeMode=m.choices.first{it.name=="Temporary"}
  set("TargetLock.Combat",false);set("Temporary.MaximumTime",30);set("Temporary.MaximumRange",20f)
  for((key,tag)in listOf("Temporary.WhenNoLock" to "AllowAll","Filter.FilterType" to "Whitelist"))enum(key,tag)
  set("Filter.Usernames",mutableListOf("Notch"))
 }
 fun enum(key:String,tag:String){val value=values.getValue(key).inner!!;set(key,value.javaClass.enumConstants.first{(it as net.ccbluex.liquidbounce.config.types.list.Tagged).tag.equals(tag,true)})}
 @JvmStatic fun main(args:Array<String>){ModuleTargetLock.hashCode();temporary=handlers.first{it.first is Mode && (it.first as Mode).name=="Temporary"}.first
  val output=JsonArray();val gson=Gson()
  for(row in JsonParser.parseString(File(args[0]).readText()).asJsonArray){reset();val steps=JsonArray()
   for(raw in row.asJsonArray){trace.clear();val op=raw.asJsonObject;var decision:String?=null;var failed=false
    when(op["op"].asString){
     "running"->running=op["value"].asBoolean
     "now"->now=op["value"].asString.toLong()
     "dead"->dead=op["value"].asBoolean
     "notice"->{noticeTime=op.get("time")?.asInt;noticeClock=op.get("clock")?.asString?.toLong()}
     "player"->{val id=op["id"].asInt;players[id]=if(op.get("client")?.asBoolean==false)Any() else AbstractClientPlayer(id,op["name"].asString,java.lang.Double.longBitsToDouble(op["distance"].asString.toULong().toLong()),op.get("removed")?.asBoolean?:false)}
     "remove"->players.remove(op["id"].asInt)
     "mode"->{@Suppress("UNCHECKED_CAST") val m=mode as ModeValueGroup<Mode>;m.activeMode=m.choices.first{it.name==op["value"].asString}}
     "combat"->set("TargetLock.Combat",op["value"].asBoolean)
     "time"->set("Temporary.MaximumTime",op["value"].asInt)
     "range"->set("Temporary.MaximumRange",java.lang.Float.intBitsToFloat(op["bits"].asString.toUInt().toInt()))
     "empty"->enum("Temporary.WhenNoLock",op["value"].asString)
     "filter"->enum("Filter.FilterType",op["value"].asString)
     "names"->{try{(values.getValue("Filter.Usernames") as ListValue<*,*>).deserializeFrom(gson,op["value"])}catch(t:Throwable){failed=true}}
     "attack"->dispatch(AttackEntityEvent(players[op["id"].asInt]?:Any(),op.get("cancelled")?.asBoolean?:false))
     "tick"->dispatch(GameTickEvent())
     "tag"->{val e=TagEntityEvent(players[op["id"].asInt]?:Any());dispatch(e);decision=e.decision}
    }
    val lockArray=JsonArray();for(e in locks().int2LongEntrySet()){lockArray.add(JsonArray().apply{add(e.intKey);add(e.longValue.toString())})}
    steps.add(JsonObject().apply{add("locks",lockArray);add("trace",gson.toJsonTree(trace));addProperty("decision",decision);addProperty("failed",failed);add("names",gson.toJsonTree(values.getValue("Filter.Usernames").inner))})
   };output.add(steps)
  };File(args[1]).writeText(output.toString())
 }
}
