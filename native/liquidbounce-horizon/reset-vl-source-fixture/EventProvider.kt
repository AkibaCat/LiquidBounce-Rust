package net.ccbluex.liquidbounce.event

import java.util.function.Consumer

abstract class Event
interface EventListener { val running: Boolean get() = true }
class EventHook<T:Event>(val handlerClass:EventListener,val priority:Short=0,val handler:Consumer<T>)
object EventManager {
    private val hooks = mutableListOf<Pair<Class<*>,EventHook<*>>>()
    fun <T:Event> registerEventHook(type:Class<T>,hook:EventHook<T>):EventHook<T> { hooks.add(type to hook); return hook }
    fun <T:Event> unregisterEventHook(type:Class<T>,hook:EventHook<T>) { hooks.removeAll { it.first==type && it.second===hook } }
    fun callEvent(event:Event) {
        for ((type,hook) in hooks.toList().sortedByDescending { it.second.priority }) {
            if(type.isInstance(event) && hook.handlerClass.running) {
                @Suppress("UNCHECKED_CAST")
                (hook.handler as Consumer<Event>).accept(event)
            }
        }
    }
}
inline fun <E:Event> EventListener.newEventHook(priority:Short=0,handler:Consumer<E>) = EventHook(this,priority,handler)
fun <T:Event> EventListener.handler(eventClass:Class<T>,priority:Short=0,handler:Consumer<T>):EventHook<T> = EventManager.registerEventHook(eventClass,newEventHook(priority,handler))
inline fun <reified T:Event> EventListener.handler(priority:Short=0,handler:Consumer<T>):EventHook<T> = handler(T::class.java,priority,handler)
