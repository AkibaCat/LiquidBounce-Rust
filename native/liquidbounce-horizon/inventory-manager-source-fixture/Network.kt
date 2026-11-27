package net.ccbluex.liquidbounce.utils.network
import net.ccbluex.liquidbounce.utils.client.network
import net.minecraft.network.protocol.game.ServerboundContainerClosePacket
fun network.send1_11_1OpenInventory() { fixture.Harness.effects.add("open") }
fun network.sendCloseInventory() { fixture.Harness.effects.add("close");fixture.Harness.send(ServerboundContainerClosePacket(0)) }
