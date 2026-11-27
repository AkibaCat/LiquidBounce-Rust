package net.ccbluex.liquidbounce.utils.inventory
import net.minecraft.world.item.ItemStack
import net.minecraft.client.gui.screens.inventory.AbstractContainerScreen
import net.ccbluex.liquidbounce.utils.client.*
interface ItemSlot { val itemStack:ItemStack;fun getIdForServer(screen:AbstractContainerScreen<*>?):Int? }
data class PlayerItemSlot(val index:Int):ItemSlot {
    override val itemStack get()=player.inventoryMenu.slots[getIdForServer(null)!!].item
    override fun getIdForServer(screen:AbstractContainerScreen<*>?):Int? = when(index) {
        in 0..8 -> (screen?.menu?.slots?.size?.minus(9) ?:36)+index
        in 9..35 -> (screen?.menu?.slots?.size?.minus(36) ?:9)+index-9
        in 36..39 -> if(screen==null)44-index else null
        40 -> if(screen==null)45 else null
        else -> null
    }
}
data class ContainerItemSlot(val slotInContainer:Int):ItemSlot {
    override val itemStack get()=(mc.gui.screen() as AbstractContainerScreen<*>).menu.slots[slotInContainer].item
    override fun getIdForServer(screen:AbstractContainerScreen<*>?)=slotInContainer
    fun distance(other:ContainerItemSlot):Int {
        val dx=slotInContainer%9-other.slotInContainer%9;val dy=slotInContainer/9-other.slotInContainer/9
        return dx*dx+dy*dy
    }
}
enum class HotbarItemSlot(val inventorySlot:Int):ItemSlot {
    OFFHAND(40),SLOT_0(0),SLOT_1(1),SLOT_2(2),SLOT_3(3),SLOT_4(4),SLOT_5(5),SLOT_6(6),SLOT_7(7),SLOT_8(8);
    override val itemStack get()=PlayerItemSlot(inventorySlot).itemStack
    override fun getIdForServer(screen:AbstractContainerScreen<*>?)=PlayerItemSlot(inventorySlot).getIdForServer(screen)
    val canBeSwapTarget get()=true
    val useHand get()=if(inventorySlot==40)1 else 0
}
class SlotGroup(private val list:List<ItemSlot>) { fun findSlot(predicate:(ItemStack)->Boolean)=if(inGame)list.find { predicate(it.itemStack) }else null }
object Slots {
    val Inventory=SlotGroup((9..35).map { PlayerItemSlot(it) })
    val Hotbar=SlotGroup((0..8).map { PlayerItemSlot(it) })
}
fun AbstractContainerScreen<*>.getSlotsInContainer()=menu.slots.indices.filter { menu.slots[it].container !== player.inventory }.map { ContainerItemSlot(it) }
