package net.ccbluex.liquidbounce.event.events
class GameTickEvent
class WorldChangeEvent
class PacketEvent(val packet:Any,var isCancelled:Boolean=false) { fun cancelEvent(){isCancelled=true} }
class ScreenEvent(val screen:net.minecraft.client.gui.screens.Screen?,var isCancelled:Boolean=false)
