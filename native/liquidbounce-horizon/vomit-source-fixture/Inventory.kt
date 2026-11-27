package net.ccbluex.liquidbounce.utils.inventory
import net.ccbluex.liquidbounce.features.module.ClientModule
import net.minecraft.world.item.ItemStack
import net.minecraft.client.gui.screens.inventory.ContainerScreen
class PlayerInventoryConstraints { var clickDelay=100..200 }
data class Slot(val kind:Int,val index:Int)
object Harness {
    var body:(suspend Plan.()->Unit)?=null
    var constraints:PlayerInventoryConstraints?=null
    var mask=0L
    val actions=mutableListOf<String>()
    var guard:(()->Boolean)?=null
}
class Plan {
    suspend fun perform(action:String) { Harness.actions.add(action) }
    suspend fun perform(actions:List<String>) { Harness.actions.addAll(actions) }
}
fun ClientModule.inventoryPlan(constraints:PlayerInventoryConstraints,body:suspend Plan.()->Unit):Int {
    Harness.constraints=constraints;Harness.body=body;return 1
}
fun findEmptyStorageSlotsInInventory()=(9..35).toList().plus((0..8).toList()).filter { Harness.mask and (1L shl it)==0L }.map { Slot(0,it) }
fun findNonEmptyStorageSlotsInInventory()=(9..35).toList().plus((0..8).toList()).filter { Harness.mask and (1L shl it)!=0L }.map { Slot(0,it) }
fun ContainerScreen.findItemsInContainer()=nonempty.map { Slot(1,it) }
object InventoryAction {
    object Creative {
        fun performThrow(stack:ItemStack)="[0,${stack.block.index},${stack.count}]"
        fun performFillSlot(stack:ItemStack,slot:Slot)="[1,${stack.block.index},${stack.count},${slot.kind},${slot.index}]"
    }
    object Click {
        fun performThrow(screen:ContainerScreen?,slot:Slot)="[2,${screen?.syncId?:-1},${screen?.count?:-1},${slot.kind},${slot.index}]"
    }
}
