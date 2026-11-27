package net.ccbluex.liquidbounce.utils.client
import net.minecraft.client.gui.screens.inventory.ContainerScreen
class Player { var isCreative=false }
class Gui { var value:ContainerScreen?=null;fun screen():Any?=value }
class Minecraft { val gui=Gui() }
val player=Player()
val mc=Minecraft()
