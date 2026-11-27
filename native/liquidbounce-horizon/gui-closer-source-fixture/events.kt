package net.ccbluex.liquidbounce.event.events
import net.minecraft.client.gui.screens.Screen
class ScreenEvent(val screen:Screen?) {var cancelled=false;fun cancelEvent(){cancelled=true}}
