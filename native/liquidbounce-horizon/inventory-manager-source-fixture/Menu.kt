package net.minecraft.world.inventory
import net.minecraft.world.item.ItemStack
enum class ContainerInput { PICKUP,QUICK_MOVE,SWAP,CLONE,THROW,QUICK_CRAFT,PICKUP_ALL }
class MenuSlot(val x:Int,val y:Int,initial:ItemStack=ItemStack.EMPTY,val container:Any?=null,val source:MenuSlot?=null) {
    private var own=initial
    var item:ItemStack
        get()=source?.item ?:own
        set(value) { if(source==null)own=value else source.item=value }
}
open class AbstractContainerMenu(val containerId:Int,val slots:MutableList<MenuSlot>) {
    var carried=ItemStack.EMPTY
    val typeOrNull:String? get()=if(containerId==0)null else "container"
}
