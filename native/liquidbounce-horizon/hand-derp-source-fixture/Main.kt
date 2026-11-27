import java.nio.file.Files
import java.nio.file.Path
import net.ccbluex.liquidbounce.features.module.modules.`fun`.ModuleHandDerp
import net.ccbluex.liquidbounce.event.Harness
import net.ccbluex.liquidbounce.event.events.PacketEvent
import net.ccbluex.liquidbounce.utils.client.*
import net.minecraft.network.protocol.game.*
fun main(args:Array<String>) {
    mc.options.arm.value=if(args[1]=="0")Arm.LEFT else Arm.RIGHT
    val module=ModuleHandDerp;val mode=module.groups.getValue("Mode");val rows=mutableListOf<String>()
    fun arm(field:String):Int { val f=module.javaClass.getDeclaredField(field);f.isAccessible=true;return (f.get(module) as Arm).id }
    for(line in Files.readAllLines(Path.of(args[0]))) {
        val p=line.split(' ');network.sent.clear();var cancelled=false
        when(p[0]) {
            "enabled" -> module.enabled=p[1].toBoolean()
            "inGame" -> Context.inGame=p[1].toBoolean()
            "mode" -> mode.setByString(p[1])
            "delay" -> mode.mode("Delay").set("Delay",p[1].toInt())
            "silent" -> module.set("Silent",p[1].toBoolean())
            "actual" -> mc.options.arm.value=if(p[1]=="0")Arm.LEFT else Arm.RIGHT
            "tick" -> Harness.tick()
            "punch" -> network.send(ServerboundPunchPacket(),p[1].toBoolean())
            "noSwing" -> network.noSwing=p[1].toBoolean()
            "metadata" -> { val event=PacketEvent(ClientboundSetEntityDataPacket(p[1].toInt(),p.drop(2).map { Data(it.toInt()) }));Harness.packet(event);cancelled=event.cancelled }
            else -> error(line)
        }
        rows.add("[${mc.options.mainHand().get().id},${arm("currentHand")},${arm("originalHand")},$cancelled,[${network.sent.joinToString(",")}]]")
    }
    print("{\"rows\":[${rows.joinToString(",")}]}\n")
}
