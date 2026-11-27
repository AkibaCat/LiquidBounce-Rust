import com.google.gson.*
import java.nio.file.Files
import java.nio.file.Path
import kotlin.coroutines.resume
import kotlinx.coroutines.*
import net.ccbluex.liquidbounce.features.module.modules.misc.ModuleElytraSwap
import net.ccbluex.liquidbounce.utils.inventory.*

fun main(args:Array<String>)=runBlocking {
    val m=ModuleElytraSwap
    check(m.name=="ElytraSwap" && m.category.name=="PLAYER" && m.aliases==listOf("ChestSwap") && m.disableOnQuit)
    val wants=m.javaClass.getDeclaredField("wantsGlider").also { it.isAccessible=true }
    val output=JsonArray()
    for(commands in JsonParser.parseString(Files.readString(Path.of(args[0]))).asJsonArray) {
        m.enabled=false
        Harness.stacks=Array(41) { Stack(1) }
        var job:Job?=null
        val rows=JsonArray()
        for(command in commands.asJsonArray) {
            val c=command.asJsonObject
            Harness.reads.clear(); Harness.actions.clear()
            when(c["op"].asString) {
                "enable" -> m.enabled=c["value"].asBoolean
                "stack" -> Harness.stacks[c["slot"].asInt]=Stack(c["bits"].asInt)
                "start" -> if(m.enabled && job?.isActive!=true) {
                    job=launch(start=CoroutineStart.UNDISPATCHED) { Harness.body!!.invoke(Plan()) }
                }
                "resume" -> { Harness.waiting?.resume(Unit); yield() }
                "cancel" -> { job?.cancelAndJoin(); Harness.waiting=null }
                else -> error("unknown command")
            }
            val row=JsonObject()
            row.addProperty("enabled",m.enabled)
            row.add("wants",Gson().toJsonTree(wants.get(m)))
            row.add("actions",Gson().toJsonTree(Harness.actions))
            row.add("reads",Gson().toJsonTree(Harness.reads))
            row.addProperty("pending",job?.isActive==true)
            rows.add(row)
        }
        job?.cancelAndJoin(); Harness.waiting=null
        output.add(rows)
    }
    Files.writeString(Path.of(args[1]),GsonBuilder().serializeNulls().create().toJson(output))
    Unit
}
