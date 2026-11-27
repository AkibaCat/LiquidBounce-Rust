import java.nio.file.Files
import java.nio.file.Path
import java.util.concurrent.ThreadLocalRandom
import net.ccbluex.liquidbounce.event.*
import net.ccbluex.liquidbounce.features.module.modules.`fun`.ModuleSkinDerp
import net.ccbluex.liquidbounce.utils.client.*
import net.minecraft.world.entity.player.PlayerModelPart

fun main(args:Array<String>) {
    ThreadLocalRandom.current();kotlin.random.Random.nextBoolean()
    val seedField=Thread::class.java.getDeclaredField("threadLocalRandomSeed").also { it.isAccessible=true }
    val seed=305419896L;seedField.setLong(Thread.currentThread(),seed)
    val module=ModuleSkinDerp
    val rows=mutableListOf<String>()
    val previous=module.javaClass.getDeclaredField("prevModelParts").also { it.isAccessible=true }
    for(line in Files.readAllLines(Path.of(args[0]))) {
        val p=line.split(' ')
        when(p[0]) {
            "actual"->mc.options.setBits(p[1].toInt())
            "enabled"->module.enabled=p[1].toBoolean()
            "inGame"->Context.inGame=p[1].toBoolean()
            "delay"->module.set("Delay",p[1].toInt())
            "sync"->module.set("Sync",p[1].toBoolean())
            "parts"->module.setParts(if(p.size==1)emptyList()else p.drop(1))
            "tick"->{ CoroutineTicker.beginMinecraftTick();CoroutineTicker.tick();Harness.gameTick();CoroutineTicker.endMinecraftTick() }
            else->error(line)
        }
        @Suppress("UNCHECKED_CAST") val old=previous.get(module) as Set<PlayerModelPart>
        val bits=old.fold(0) { bits,part->bits or part.mask }
        rows.add("[${mc.options.bits()},$bits,\"${seedField.getLong(Thread.currentThread()).toULong()}\"]")
    }
    println("{\"seed\":$seed,\"thread\":${Thread.currentThread().threadId()},\"rows\":[${rows.joinToString(",")} ]}")
}
