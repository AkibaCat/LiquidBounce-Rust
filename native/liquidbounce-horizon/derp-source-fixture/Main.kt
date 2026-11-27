import java.nio.file.Files
import java.nio.file.Path
import java.util.concurrent.ThreadLocalRandom
import net.ccbluex.liquidbounce.features.module.modules.`fun`.ModuleDerp
import net.ccbluex.liquidbounce.event.Harness
import net.ccbluex.liquidbounce.utils.client.*
import net.ccbluex.liquidbounce.utils.aiming.RotationManager
fun main(args:Array<String>) {
    ThreadLocalRandom.current()
    val seed=305419896L
    Thread::class.java.getDeclaredField("threadLocalRandomSeed").also { it.isAccessible=true; it.setLong(Thread.currentThread(),seed) }
    val module=ModuleDerp
    val yaw=module.groups.getValue("Yaw")
    val pitch=module.groups.getValue("Pitch")
    val rows=mutableListOf<String>()
    for(line in Files.readAllLines(Path.of(args[0]))) {
        val parts=line.trim().split(' ')
        when(parts[0]) {
            "yaw" -> yaw.setByString(parts[1])
            "pitch" -> pitch.setByString(parts[1])
            "floatYaw" -> yaw.mode(parts[1]).set(parts[2],parts[3].toFloat())
            "intYaw" -> yaw.mode(parts[1]).set(parts[2],parts[3].toInt())
            "floatPitch" -> pitch.mode(parts[1]).set(parts[2],parts[3].toFloat())
            "bool" -> module.set(parts[1],parts[2].toBoolean())
            "enabled" -> module.enabled=parts[1].toBoolean()
            "inGame" -> Context.inGame=parts[1].toBoolean()
            "tick" -> {
                player.yRot=parts[1].toFloat(); player.xRot=parts[2].toFloat()
                mc.options.keySprint.isDown=parts[3].toBoolean(); player.isSprinting=parts[4].toBoolean()
                RotationManager.target=null
                Harness.tick()
                val target=RotationManager.target
                rows.add(if(target==null)"null" else "[${target.yaw.toRawBits().toUInt().toLong()},${target.pitch.toRawBits().toUInt().toLong()},${Harness.draws}]")
            }
            else -> error(line)
        }
    }
    print("{\"seed\":$seed,\"threadId\":${Thread.currentThread().threadId()},\"rows\":[${rows.joinToString(",")}]}\n")
}
