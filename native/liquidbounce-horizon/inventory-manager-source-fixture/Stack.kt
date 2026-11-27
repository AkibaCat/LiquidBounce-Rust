package net.minecraft.world.item
data class ItemStack(var components:String="",var count:Int=0) {
    val isEmpty get()=count<=0 || components=="air" || components==""
    fun copy()=ItemStack(components,count)
    companion object {
        val EMPTY get()=ItemStack()
        fun matches(a:ItemStack,b:ItemStack)=(a.isEmpty && b.isEmpty) || (a.count==b.count && isSameItemSameComponents(a,b))
        fun isSameItemSameComponents(a:ItemStack,b:ItemStack)=!a.isEmpty && !b.isEmpty && a.components==b.components
    }
}
