package net.minecraft.network.protocol.game
import net.minecraft.world.item.ItemStack
class ServerboundContainerClickPacket(val containerId:Int)
class ServerboundContainerClosePacket(val containerId:Int)
class ClientboundContainerClosePacket(val containerId:Int)
class ClientboundOpenScreenPacket
class ClientboundContainerSetSlotPacket(val containerId:Int,val slot:Int,val item:ItemStack)
class ClientboundContainerSetContentPacket(val containerId:Int,val items:List<ItemStack>)
class ServerboundInteractPacket
class BlockHit(val blockPos:Int=0)
class ServerboundUseItemOnPacket(val hitResult:BlockHit=BlockHit())
