package net.ccbluex.liquidbounce.event
import kotlin.coroutines.*
import kotlinx.coroutines.*
import net.ccbluex.liquidbounce.event.events.GameTickEvent
interface EventListener { val running:Boolean }
class Entry(val owner:EventListener,val sequence:(suspend ()->Unit)?,val ordinary:(()->Unit)?) { var job:Job?=null }
class GuardContext(val owner:EventListener):AbstractCoroutineContextElement(Key) { companion object Key:CoroutineContext.Key<GuardContext> }
class Guards(val owner:EventListener) { fun broken():Throwable? = if(owner.running)null else CancellationException("inactive source listener") }
fun CoroutineContext.activeGuards():Guards?=this[GuardContext]?.let { Guards(it.owner) }
object Harness {
    val entries=mutableListOf<Entry>()
    var draws=0
    private val scope=CoroutineScope(Dispatchers.Unconfined)
    fun cancel(owner:EventListener) { for(entry in entries)if(entry.owner===owner)entry.job?.cancel() }
    fun gameTick() {
        for(entry in entries) {
            if(!entry.owner.running)continue
            if(entry.sequence==null) { entry.ordinary?.invoke(); continue }
            if(entry.job?.isActive==true)continue
            entry.job=scope.launch(GuardContext(entry.owner)) { entry.sequence.invoke() }
        }
    }
}
inline fun<reified T> EventListener.handler(noinline body:(T)->Unit):Entry = Entry(this,null,if(T::class==GameTickEvent::class)({body(GameTickEvent() as T)})else null).also { Harness.entries.add(it) }
inline fun<reified T> EventListener.once(noinline body:(T)->Unit) {}
fun EventListener.tickHandler(body:suspend ()->Unit):Entry = Entry(this,body,null).also { Harness.entries.add(it) }
