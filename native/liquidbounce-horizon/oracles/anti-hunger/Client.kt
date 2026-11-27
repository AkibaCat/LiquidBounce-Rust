package net.ccbluex.liquidbounce.utils.client
import fixture.Fixture
import net.ccbluex.liquidbounce.event.events.NotificationEvent
fun notification(title:String,message:String,severity:NotificationEvent.Severity){Fixture.trace.add("notification:$title:$message:$severity")}
