import com.google.gson.*
import com.mojang.blaze3d.platform.InputConstants
import net.ccbluex.liquidbounce.features.module.Fixture
import net.ccbluex.liquidbounce.features.module.modules.misc.ModuleItemScroller
import net.ccbluex.liquidbounce.utils.client.Chronometer
import net.minecraft.world.inventory.*
import java.util.concurrent.ThreadLocalRandom

fun main(args:Array<String>) {
    ModuleItemScroller
    ThreadLocalRandom.current()
    val seed=Thread::class.java.getDeclaredField("threadLocalRandomSeed").apply{isAccessible=true}
    val timer=ModuleItemScroller::class.java.getDeclaredField("chronometer").apply{isAccessible=true}.get(ModuleItemScroller) as Chronometer
    val update=Chronometer::class.java.getDeclaredField("lastUpdate").apply{isAccessible=true}
    val rows=JsonParser.parseString(java.io.File(args[0]).readText()).asJsonArray.map{element->
        val row=element.asJsonObject
        Fixture.running=row["running"].asBoolean
        InputConstants.left=row["leftShift"].asBoolean
        InputConstants.right=row["rightShift"].asBoolean
        Fixture.mc.mouseHandler.isLeftPressed=row["leftMouse"].asBoolean
        Fixture.delay=row["from"].asInt..row["to"].asInt
        val last=System.currentTimeMillis()-row["elapsed"].asLong
        update.setLong(timer,last)
        seed.setLong(Thread.currentThread(),123456789L)
        val allowed=try {ModuleItemScroller.canPerformScroll()}catch(_:Exception){null}
        val now=System.currentTimeMillis()
        val next=ThreadLocalRandom.current().nextInt()
        mapOf("input" to row,"last" to last,"now" to now,"allowed" to allowed,"next" to next,"thread" to Thread.currentThread().threadId())
    }
    var callback:List<Any>?=null
    val slot=Slot(17)
    ModuleItemScroller.clickMode.action(AbstractContainerMenu(),slot){callbackSlot,id,button,input->callback=listOf(callbackSlot===slot,id,button,input.name)}
    val beforeReset=System.currentTimeMillis();ModuleItemScroller.resetChronometer();val afterReset=System.currentTimeMillis()
    val reset=update.getLong(timer)
    println(Gson().toJson(mapOf("rows" to rows,"click" to callback,"resetInBounds" to (reset in beforeReset..afterReset))))
}
