package net.ccbluex.liquidbounce.utils.item
import net.ccbluex.liquidbounce.utils.inventory.Stack
val Stack.isGlider get()=bits and 2 != 0
val Stack.isChestArmor get()=bits and 8 != 0
