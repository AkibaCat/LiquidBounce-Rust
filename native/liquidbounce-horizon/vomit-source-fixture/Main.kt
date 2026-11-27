import java.nio.file.Files
import java.nio.file.Path
import java.util.concurrent.ThreadLocalRandom
import java.util.concurrent.atomic.AtomicLong
import kotlinx.coroutines.runBlocking
import net.ccbluex.liquidbounce.features.module.modules.`fun`.ModuleVomit
import net.ccbluex.liquidbounce.utils.inventory.*
import net.ccbluex.liquidbounce.utils.client.*
import net.minecraft.client.gui.screens.inventory.ContainerScreen
import net.minecraft.core.registries.BuiltInRegistries
import net.minecraft.util.RandomSource

fun main(args:Array<String>) {
    val module=ModuleVomit
    val random=module.javaClass.getDeclaredField("random").also { it.isAccessible=true }.get(module) as RandomSource
    random.setSeed(81985529216486895L)
    val blockSeed=random.javaClass.getDeclaredField("seed").also { it.isAccessible=true }.get(random) as AtomicLong
    ThreadLocalRandom.current();kotlin.random.Random.nextBoolean()
    val threadSeed=Thread::class.java.getDeclaredField("threadLocalRandomSeed").also { it.isAccessible=true }
    threadSeed.setLong(Thread.currentThread(),305419896L)
    val rows=mutableListOf<String>()
    for(line in Files.readAllLines(Path.of(args[0]))) {
        val p=line.split(' ')
        player.isCreative=p[0].toBoolean();Harness.constraints!!.clickDelay=p[1].toInt()..p[2].toInt();Harness.mask=p[3].toLong()
        mc.gui.value=if(p[4]=="-1")null else ContainerScreen(p[4].toInt(),p[5].toInt(),if(p[6]=="-")emptyList()else p[6].split(',').map { it.toInt() })
        BuiltInRegistries.BLOCK.size=p[7].toInt()
        Harness.actions.clear();Harness.guard=null
        var failure=false
        try { runBlocking { Harness.body!!.invoke(Plan()) } } catch(e:IllegalArgumentException) { failure=true }
        rows.add("{\"actions\":[${Harness.actions.joinToString(",")}],\"error\":$failure,\"blockSeed\":\"${blockSeed.get()}\",\"threadSeed\":\"${threadSeed.getLong(Thread.currentThread()).toULong()}\",\"guard\":${Harness.guard?.invoke()?:true}}")
    }
    println("{\"blockSeed\":\"81985529216486895\",\"threadSeed\":\"305419896\",\"thread\":${Thread.currentThread().threadId()},\"rows\":[${rows.joinToString(",")} ]}")
}
