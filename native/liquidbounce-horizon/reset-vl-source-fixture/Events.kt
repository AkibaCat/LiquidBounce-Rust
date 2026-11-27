package net.ccbluex.liquidbounce.event.events
import net.ccbluex.liquidbounce.event.Event
class GameTickEvent:Event()
class WorldChangeEvent:Event()
class PlayerTickEvent(val isCancelled:Boolean=false):Event()
class NotificationEvent(val title:String,val message:String,val severity:Severity):Event(){enum class Severity{SUCCESS}}
