package net.ccbluex.liquidbounce.event.events
class PacketEvent(val packet:Any,var cancelled:Boolean=false) { fun cancelEvent() { cancelled=true } }
