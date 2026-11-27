package net.ccbluex.liquidbounce.event
import fixture.Context
import net.ccbluex.liquidbounce.event.events.KeybindIsPressedEvent
@Suppress("UNCHECKED_CAST")
fun <T> handler(action:(T)->Unit):(T)->Unit {
    Context.handler={event -> action(event as T)}
    return action
}
