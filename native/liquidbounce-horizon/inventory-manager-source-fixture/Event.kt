package net.ccbluex.liquidbounce.event
import kotlinx.coroutines.*
import kotlin.coroutines.*
import net.ccbluex.liquidbounce.event.events.*
import fixture.Clock
interface EventListener { val running:Boolean get()=true }
class EventHook<T>(val owner:EventListener,val body:(T)->Unit)
class TickEntry(val owner:EventListener,val body:suspend CoroutineScope.()->Unit) { var job:Job?=null }
object Events {
    val hooks=mutableListOf<Pair<Class<*>,EventHook<*>>>()
    val ticks=mutableListOf<TickEntry>()
    val waiters=mutableListOf<CancellableContinuation<Unit>>()
    fun fire(event:Any) {
        for((type,hook)in hooks.toList())if(type.isInstance(event) && hook.owner.running) {
            @Suppress("UNCHECKED_CAST") (hook as EventHook<Any>).body(event)
        }
    }
    fun tick() {
        val pending=waiters.toList();waiters.clear()
        for(waiter in pending)if(waiter.isActive)waiter.resume(Unit)
        Clock.flush()
        for(entry in ticks)if(entry.owner.running && entry.job?.isActive!=true) {
            entry.job=entry.owner.eventListenerScope.launch(entry.owner.wrapContinuationInterceptor(),start=CoroutineStart.UNDISPATCHED,block=entry.body)
        }
        Clock.flush()
    }
}
inline fun<reified T> EventListener.handler(priority:Short=0,noinline body:(T)->Unit):EventHook<T> = EventHook(this,body).also { Events.hooks.add(T::class.java to it) }
fun EventListener.tickHandler(body:suspend CoroutineScope.()->Unit):EventHook<GameTickEvent> = EventHook<GameTickEvent>(this) {}.also { Events.ticks.add(TickEntry(this,body)) }
suspend fun waitTicks(ticks:Int) { repeat(ticks.coerceAtLeast(1)) { suspendCancellableCoroutine<Unit> { Events.waiters.add(it) } } }
