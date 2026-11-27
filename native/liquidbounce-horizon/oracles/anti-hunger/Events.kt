package net.ccbluex.liquidbounce.event.events
interface Event
class SprintEvent(var sprint:Boolean,val source:Source):Event{enum class Source{INPUT,NETWORK,MOVEMENT}}
class PacketEvent(val packet:Any,val cancelled:Boolean):Event
class NotificationEvent(val title:String,val message:String,val severity:Severity):Event{enum class Severity{INFO,SUCCESS,ERROR,ENABLED,DISABLED}}
