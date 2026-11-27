@file:Suppress("DEPRECATION", "UNCHECKED_CAST")
package fixture
import com.google.gson.*
import java.nio.file.*
import kotlinx.coroutines.Job
import kotlinx.atomicfu.AtomicRef
import net.ccbluex.liquidbounce.event.*
import net.ccbluex.liquidbounce.event.events.*
import net.ccbluex.liquidbounce.features.module.*
import net.ccbluex.liquidbounce.features.module.modules.exploit.ModuleResetVL
import net.ccbluex.liquidbounce.utils.client.Timer
import net.ccbluex.liquidbounce.utils.client.RequestHandler
import net.ccbluex.liquidbounce.utils.kotlin.Priority

class Player {
    var velocity=doubleArrayOf(7.0,-0.0,-8.0)
    var ground=false
    fun onGround()=ground
    fun setDeltaMovement(x:Double,y:Double,z:Double) {
        velocity=doubleArrayOf(x,y,z)
        Fixture.effect("velocity",velocity.joinToString(","){Fixture.bits(it)})
    }
}
object Fixture {
    var inGame=true
    var sameThread=true
    var player=Player()
    val events=JsonArray()
    lateinit var module:ClientModule
    lateinit var other:ClientModule
    lateinit var times:Value<Int>
    lateinit var speed:Value<Float>
    val countField by lazy { ModuleResetVL.javaClass.getDeclaredField("count").also{it.isAccessible=true} }
    fun count()=countField.getInt(ModuleResetVL)
    fun bits(value:Double)=java.lang.Long.toUnsignedString(java.lang.Double.doubleToRawLongBits(value))
    fun fbits(value:Float)=java.lang.Integer.toUnsignedString(java.lang.Float.floatToRawIntBits(value))
    fun effect(kind:String,value:String) { events.add(JsonObject().apply{addProperty("kind",kind);addProperty("value",value);addProperty("count",count())}) }
    fun notification(title:String,message:String,severity:String) { effect("notification","$title|$message|$severity") }
    fun timerRequests():RequestHandler<Float> = Timer.javaClass.getDeclaredField("requestHandler").also{it.isAccessible=true}.get(Timer) as RequestHandler<Float>
    fun busy():Boolean {
        val hook=ModuleResetVL.javaClass.getDeclaredField("repeatable").also{it.isAccessible=true}.get(ModuleResetVL) as EventHook<*>
        val ref=hook.handler.javaClass.getDeclaredField("jobRef").also{it.isAccessible=true}.get(hook.handler) as AtomicRef<*>
        return (ref.value as? Job)?.isActive==true
    }
    fun result():JsonObject=JsonObject().apply{
        addProperty("count",count());addProperty("enabled",module.enabled);addProperty("running",module.running)
        addProperty("busy",busy());addProperty("speed",fbits(Timer.timerSpeed))
        add("velocity",JsonArray().apply{player.velocity.forEach{add(bits(it))}})
        add("events",events.deepCopy());events.asList().clear()
        addProperty("timerTick",timerRequests().javaClass.getDeclaredField("currentTick").also{it.isAccessible=true}.getLong(timerRequests()))
    }
    fun mutation(action:JsonObject) {
        when(action["op"].asString) {
            "enabled" -> module.enabled=action["value"].asBoolean
            "guard" -> inGame=action["value"].asBoolean
            "times" -> times.value=action["value"].asInt
            "timer" -> speed.value=java.lang.Float.intBitsToFloat(java.lang.Integer.parseUnsignedInt(action["bits"].asString))
            "count" -> countField.setInt(ModuleResetVL,action["value"].asInt)
            "ground" -> player.ground=action["value"].asBoolean
            "sameThread" -> sameThread=action["value"].asBoolean
            "velocity" -> player.velocity=action["bits"].asJsonArray.map{java.lang.Double.longBitsToDouble(java.lang.Long.parseUnsignedLong(it.asString))}.toDoubleArray()
            "other" -> {
                other.enabled=action["enabled"].asBoolean
                if(action.has("bits"))Timer.requestTimerSpeed(java.lang.Float.intBitsToFloat(java.lang.Integer.parseUnsignedInt(action["bits"].asString)),Priority.entries.first{it.priority==action["priority"].asInt},other,action["reset"].asInt)
            }
            "world" -> EventManager.callEvent(WorldChangeEvent())
            else -> error("unknown mutation $action")
        }
    }
    @JvmStatic fun main(args:Array<String>) {
        Timer.timerSpeed // register FIRST_PRIORITY before the module
        module=ModuleResetVL;other=ClientModule("Other",ModuleCategories.EXPLOIT)
        times=module.children.single{(it as Value<*>).name=="Times"} as Value<Int>
        speed=module.children.single{(it as Value<*>).name=="Timer"} as Value<Float>
        val cases=JsonParser.parseString(Files.readString(Path.of(args[0]))).asJsonArray
        val output=JsonArray()
        for(case in cases) {
            module.enabled=false;module.removeEventListenerScope();other.enabled=false;inGame=true;sameThread=true
            CoroutineTicker.tick() // discard cancelled callbacks left by the preceding case
            timerRequests().clear();player=Player();countField.setInt(ModuleResetVL,0);times.value=10;speed.value=2f
            events.asList().clear()
            val steps=JsonArray()
            for(item in case.asJsonArray) {
                val action=item.asJsonObject
                when(action["op"].asString) {
                    "callback" -> CoroutineTicker.register(java.util.function.BooleanSupplier{mutation(action["action"].asJsonObject);true})
                    "emit" -> EventManager.callEvent(GameTickEvent())
                    "tick" -> {
                        player.ground=action["ground"].asBoolean
                        CoroutineTicker.beginMinecraftTick();CoroutineTicker.tick()
                        if(action["nested"]?.asBoolean==true) {
                            CoroutineTicker.beginMinecraftTick();CoroutineTicker.tick();EventManager.callEvent(GameTickEvent());CoroutineTicker.endMinecraftTick()
                        }
                        EventManager.callEvent(GameTickEvent());CoroutineTicker.endMinecraftTick()
                    }
                    else -> mutation(action)
                }
                steps.add(result())
            }
            output.add(steps)
        }
        Files.writeString(Path.of(args[1]),Gson().toJson(output))
        println("unchanged ResetVL/full coroutine/Timer scenarios="+output.size())
    }
}
