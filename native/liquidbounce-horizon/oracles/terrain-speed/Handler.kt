package net.ccbluex.liquidbounce.event
interface EventListener {val running:Boolean}
object Hooks {
 val handlers=mutableListOf<Triple<EventListener,Class<*>,(Any)->Unit>>()
 fun emit(event:Any){for((owner,type,callback)in handlers)if(type.isInstance(event)&&owner.running)callback(event)}
}
inline fun <reified E> EventListener.handler(noinline callback:(E)->Unit) {
 Hooks.handlers.add(Triple(this,E::class.java,{event->callback(event as E)}))
}
