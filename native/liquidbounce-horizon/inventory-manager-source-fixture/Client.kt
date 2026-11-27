package net.ccbluex.liquidbounce.utils.client
import net.minecraft.world.inventory.*
import net.minecraft.world.item.ItemStack
import net.minecraft.network.protocol.game.*
import net.ccbluex.liquidbounce.event.*
import net.ccbluex.liquidbounce.event.events.*
import fixture.*
import org.slf4j.LoggerFactory
import org.joml.Vector2f

var inGame=true
var isOlderThanOrEqual1_11_1=false
val logger=LoggerFactory.getLogger("InventoryOracle")
class Gui { var value:net.minecraft.client.gui.screens.Screen?=null;fun screen()=value }
class Mouse { var isMouseGrabbed=false;var x=0.0;var y=0.0;fun xpos()=x;fun ypos()=y }
class Window { var screenWidth=854;var guiScaledWidth=427 }
class Client { val gui=Gui();val mouseHandler=Mouse();val window=Window();val player:Player? get()=if(inGame)net.ccbluex.liquidbounce.utils.client.player else null;fun execute(body:()->Unit){Clock.dispatch(kotlin.coroutines.EmptyCoroutineContext,Runnable(body))} }
class Player {
    val inventory=Any()
    val inventoryMenu=AbstractContainerMenu(0,MutableList(46) { MenuSlot((it%9)*18,(it/9)*18,container=inventory) })
    var containerMenu=inventoryMenu
    var isCreative=true
    var movement=Vector2f()
    var jumping=false
    var rotationMatches=true
    var usingItem=false
    var breaking=false
    var combat=false
    fun closeContainer() {
        val id=containerMenu.containerId
        Harness.send(ServerboundContainerClosePacket(id))
        containerMenu=inventoryMenu;mc.gui.value=null
    }
}
val mc=Client()
val player=Player()
object world {
    fun getBlockState(pos:Int)=BlockState()
    class BlockState { fun getMenuProvider(world:Any,pos:Int):Any?=if(Harness.menuProvider)Any()else null }
}
object network
object interaction {
    var isServerControlledInventory=false
    fun handleContainerInput(id:Int,slot:Int,button:Int,input:ContainerInput,player:Player) {
        Harness.effects.add("click:$id:$slot:$button:${input.ordinal}")
        if(id!=player.containerMenu.containerId)return
        val menu=player.containerMenu
        val item=menu.slots.getOrNull(slot)?.item
        if(item!=null) when(input) {
            ContainerInput.THROW -> if(menu.carried.isEmpty)menu.slots[slot].item=if(button==0 && item.count>1)ItemStack(item.components,item.count-1)else ItemStack.EMPTY
            ContainerInput.SWAP -> {
                val target=if(button==40)45 else 36+button
                if(target in player.inventoryMenu.slots.indices) {
                    val other=player.inventoryMenu.slots[target].item
                    player.inventoryMenu.slots[target].item=item
                    menu.slots[slot].item=other
                }
            }
            ContainerInput.PICKUP -> { menu.slots[slot].item=menu.carried;menu.carried=item }
            ContainerInput.QUICK_MOVE -> menu.slots[slot].item=ItemStack.EMPTY
            else -> {}
        }
        Harness.send(ServerboundContainerClickPacket(id))
    }
    // Verified official helpers only send; no local prediction is introduced.
    fun handleCreativeModeItemAdd(stack:ItemStack,slot:Int) { if(player.isCreative)Harness.effects.add("creative:$slot:${stack.components}:${stack.count}") }
    fun handleCreativeModeItemDrop(stack:ItemStack) {
        val screen=mc.gui.screen()
        val blocked=screen is net.minecraft.client.gui.screens.inventory.AbstractContainerScreen<*> && screen !is net.minecraft.client.gui.screens.inventory.CreativeModeInventoryScreen
        if(player.isCreative && !stack.isEmpty && !blocked)Harness.effects.add("creative:-1:${stack.components}:${stack.count}")
    }
}
object SilentHotbar {
    fun selectSlotSilently(requester:Any?,slot:net.ccbluex.liquidbounce.utils.inventory.HotbarItemSlot,ticks:Int) { Harness.effects.add("select:${slot.inventorySlot}:$ticks") }
}
