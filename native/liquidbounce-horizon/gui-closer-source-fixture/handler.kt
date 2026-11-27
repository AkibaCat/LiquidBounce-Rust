package net.ccbluex.liquidbounce.event
import net.ccbluex.liquidbounce.features.module.ClientModule
import net.ccbluex.liquidbounce.features.module.Fixture
import net.ccbluex.liquidbounce.event.events.ScreenEvent
fun <T> ClientModule.handler(block:(T)->Unit):Any {
 Fixture.handler={event->
  @Suppress("UNCHECKED_CAST")
  if(Fixture.running)block(event as T)
 }
 return block
}
