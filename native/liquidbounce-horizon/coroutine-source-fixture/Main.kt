import net.ccbluex.liquidbounce.event.*
import net.ccbluex.liquidbounce.features.module.modules.`fun`.ModuleDerp
import net.ccbluex.liquidbounce.features.module.modules.player.ModuleAntiAFK
import net.ccbluex.liquidbounce.utils.client.player

fun main() {
    val derp=ModuleDerp
    val anti=ModuleAntiAFK
    derp.groups.getValue("Yaw").setByString("Jitter")
    derp.groups.getValue("Yaw").mode("Jitter").set("ForwardTicks",100)
    derp.groups.getValue("Pitch").setByString("Static")
    anti.groups.getValue("Mode").setByString("Old")
    player.yRot=10f
    CoroutineTicker.beginMinecraftTick()
    CoroutineTicker.tick()
    Harness.gameTick()
    CoroutineTicker.endMinecraftTick()
    val rows=mutableListOf<String>()
    for(tick in 1..10) {
        CoroutineTicker.beginMinecraftTick()
        // Real original ticker must skip nested ticks without draining pending.
        CoroutineTicker.beginMinecraftTick()
        CoroutineTicker.tick()
        CoroutineTicker.endMinecraftTick()
        CoroutineTicker.tick()
        Harness.gameTick()
        val yaw=net.ccbluex.liquidbounce.utils.aiming.RotationManager.target!!.yaw
        rows.add("[$tick,${player.yRot},$yaw]")
        CoroutineTicker.endMinecraftTick()
    }
    println(rows.joinToString(",","[","]"))
}
