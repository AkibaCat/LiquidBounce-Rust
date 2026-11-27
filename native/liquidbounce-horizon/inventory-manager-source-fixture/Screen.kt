package net.minecraft.client.gui.screens.inventory
import net.minecraft.world.inventory.AbstractContainerMenu
open class AbstractContainerScreen<T:AbstractContainerMenu>(val menu:T):net.minecraft.client.gui.screens.Screen() { var leftPos=0;var topPos=0 }
class InventoryScreen(menu:AbstractContainerMenu):AbstractContainerScreen<AbstractContainerMenu>(menu)
class ContainerScreen(menu:AbstractContainerMenu):AbstractContainerScreen<AbstractContainerMenu>(menu)
class CreativeModeInventoryScreen(menu:AbstractContainerMenu):AbstractContainerScreen<AbstractContainerMenu>(menu)
