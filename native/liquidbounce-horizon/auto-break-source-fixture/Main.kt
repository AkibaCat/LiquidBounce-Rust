import fixture.*
import com.google.gson.Gson
import com.google.gson.JsonParser
import net.ccbluex.liquidbounce.features.module.modules.player.ModuleAutoBreak
import net.ccbluex.liquidbounce.event.events.KeybindIsPressedEvent
import net.minecraft.client.KeyMapping
import net.minecraft.core.BlockPos
import net.minecraft.core.Direction
import net.minecraft.world.phys.*

fun main(args:Array<String>) {
    ModuleAutoBreak
    val field=KeyMapping::class.java.getDeclaredField("clickCount").apply{isAccessible=true}
    val rows=JsonParser.parseString(java.io.File(args[0]).readText()).asJsonArray.map { element ->
        val row=element.asJsonObject
        KeyMapping.releaseAll()
        val original=row["original"].asBoolean
        val eventKey=if(row["attack"].asBoolean) Context.mc.options.keyAttack else Context.mc.options.keyUse
        eventKey.setDown(original)
        Context.mc.missTime=row["missTime"].asInt
        Context.interaction.isDestroying=row["destroying"].asBoolean
        Context.state=when(row["state"].asString) { "missing" -> null; "air" -> State(true); else -> State(false) }
        Context.mc.hitResult=when(row["hit"].asString) {
            "block" -> BlockHitResult(Vec3.ZERO,Direction.UP,BlockPos.ZERO,false)
            "miss" -> BlockHitResult.miss(Vec3.ZERO,Direction.UP,BlockPos.ZERO)
            "entity" -> object:HitResult(Vec3.ZERO) {override fun getType()=HitResult.Type.ENTITY}
            else -> null
        }
        val event=KeybindIsPressedEvent(eventKey,eventKey.isDown)
        if(row["running"].asBoolean) Context.handler!!(event)
        mapOf("input" to row,"pressed" to event.isPressed,
            "click" to (field.getInt(Context.mc.options.keyAttack)!=0),
            "counts" to listOf(Context.mc.options.keyAttack,Context.mc.options.keyUse,Context.mc.options.keyForward,Context.mc.options.keyScreenshot).map{field.getInt(it)},
            "held" to listOf(Context.mc.options.keyAttack,Context.mc.options.keyUse,Context.mc.options.keyForward,Context.mc.options.keyScreenshot).map{it.isDown})
    }
    println(Gson().toJson(rows))
}
