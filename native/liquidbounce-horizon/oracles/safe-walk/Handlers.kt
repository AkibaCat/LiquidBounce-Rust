package net.ccbluex.liquidbounce.event
import net.ccbluex.liquidbounce.event.events.Event
interface EventListener{val running:Boolean}
class Hook(val owner:EventListener,val type:Class<*>,val priority:Short,val body:(Event)->Unit)
val hooks=mutableListOf<Hook>()
inline fun <reified T:Event> EventListener.handler(priority:Short=0,crossinline block:(T)->Unit)=Hook(this,T::class.java,priority){block(it as T)}.also{hooks.add(it)}
fun emit(event:Event){for(h in hooks.sortedByDescending{it.priority})if(h.owner.running&&h.type.isInstance(event))h.body(event)}
