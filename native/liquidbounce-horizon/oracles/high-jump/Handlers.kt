package net.ccbluex.liquidbounce.event
import kotlinx.coroutines.*
import kotlin.coroutines.*
import net.ccbluex.liquidbounce.event.events.Event
interface EventListener{val running:Boolean}
class Guards(val owner:EventListener):AbstractCoroutineContextElement(Key){
 companion object Key:CoroutineContext.Key<Guards>
 fun broken():Throwable?=if(owner.running)null else CancellationException("not running")
}
fun CoroutineContext.activeGuards()=this[Guards]
class Hook(val owner:EventListener,val type:Class<*>,val body:(Event)->Unit)
val hooks=mutableListOf<Hook>()
val scopes=java.util.IdentityHashMap<EventListener,CoroutineScope>()
fun EventListener.cancelScope(){scopes.remove(this)?.cancel()}
inline fun <reified T:Event> EventListener.handler(crossinline block:(T)->Unit):Hook{
 return Hook(this,T::class.java){block(it as T)}.also{hooks.add(it)}
}
inline fun <reified T:Event> EventListener.sequenceHandler(crossinline block:suspend CoroutineScope.(T)->Unit):Hook{
 val owner=this
 return Hook(this,T::class.java){event->
  val scope=scopes.getOrPut(owner){CoroutineScope(SupervisorJob()+Dispatchers.Unconfined+Guards(owner))}
  scope.launch(start=CoroutineStart.UNDISPATCHED){block(event as T)}
 }.also{hooks.add(it)}
}
fun EventListener.tickHandler(block:suspend CoroutineScope.()->Unit){}
fun emit(event:Event){for(h in hooks.toList())if(h.type.isInstance(event)&&h.owner.running)h.body(event)}
