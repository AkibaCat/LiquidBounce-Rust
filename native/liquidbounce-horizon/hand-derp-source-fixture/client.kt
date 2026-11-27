package net.ccbluex.liquidbounce.utils.client
import net.minecraft.network.protocol.common.ServerboundClientInformationPacket
import net.ccbluex.liquidbounce.event.Harness
import net.ccbluex.liquidbounce.event.events.PacketEvent
object Context { var inGame=true }
enum class Arm(val id:Int) { LEFT(0),RIGHT(1);val opposite:Arm get()=if(this==LEFT)RIGHT else LEFT }
class Option<T>(var value:T) { fun get():T=value }
enum class Part { CAPE,JACKET,LEFT_SLEEVE,RIGHT_SLEEVE,LEFT_PANTS,RIGHT_PANTS,HAT }
class Options {
    val arm=Option(Arm.RIGHT);fun mainHand()=arm
    var languageCode="en_us"
    val distance=Option(12);fun renderDistance()=distance
    val chat=Option(0);fun chatVisibility()=chat
    val colors=Option(true);fun chatColors()=colors
    val modelParts=Part.entries.toMutableSet()
    val listing=Option(true);fun allowServerListing()=listing
    val particle=Option(0);fun particles()=particle
}
object mc { val options=Options();var isTextFilteringEnabled=false }
object network {
    val sent=mutableListOf<Int>();var noSwing=false
    fun send(packet:Any,cancelled:Boolean=false) {
        if(noSwing && packet is net.minecraft.network.protocol.game.ServerboundPunchPacket)return
        val event=PacketEvent(packet,cancelled);Harness.packet(event)
        if(!event.cancelled && packet is ServerboundClientInformationPacket)sent.add(packet.information.hand.id)
    }
}
