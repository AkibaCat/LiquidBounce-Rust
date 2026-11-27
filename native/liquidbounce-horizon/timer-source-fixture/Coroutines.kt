package net.ccbluex.liquidbounce.event
import kotlinx.coroutines.*
import kotlin.coroutines.CoroutineContext
class Guards {fun broken():Throwable?=null}
fun CoroutineContext.activeGuards():Guards?=null
val jobs=mutableMapOf<EventListener,Job>()
fun cancelInactive(){for((owner,job)in jobs.toMap())if(!owner.running){job.cancel();jobs.remove(owner)}}
fun EventListener.tickHandler(action:suspend CoroutineScope.()->Unit):Hook = handler<net.ccbluex.liquidbounce.event.events.GameTickEvent> {
 if(jobs[this]?.isActive!=true)jobs[this]=CoroutineScope(Dispatchers.Unconfined).launch(start=CoroutineStart.UNDISPATCHED){action()}
}
