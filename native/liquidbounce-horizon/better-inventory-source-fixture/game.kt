package net.minecraft.world.item
import net.minecraft.core.component.DataComponents
data class ItemStack(val index:Int,val isEmpty:Boolean=false,val contents:ContainerComponent?=null) {
    operator fun get(key:DataComponents):ContainerComponent?=contents
}
class ContainerComponent(val items:List<ItemStack>) {
    fun nonEmptyItemCopyStream()=items.stream().filter{!it.isEmpty}
    fun itemCopies()=items.stream()
}
class ItemCooldowns { var progress=0f;var entry:fixture.Entry?=fixture.Entry(20,0)
    fun getCooldownPercent(stack:ItemStack,delta:Float)=progress
}
