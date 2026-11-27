package net.ccbluex.liquidbounce.event
import kotlin.coroutines.*
import net.ccbluex.liquidbounce.event.events.PacketEvent
interface EventListener { val running:Boolean }
class Run(val owner:EventListener) { var active=true }
class TickEntry(val owner:EventListener,val body:suspend ()->Unit) { var run:Run?=null }
class PacketEntry(val owner:EventListener,val priority:Short,val body:suspend (PacketEvent)->Unit)
class Wait(val run:Run,val continuation:Continuation<Unit>,var ticks:Int)
object Harness {
    val ticks=mutableListOf<TickEntry>();val packets=mutableListOf<PacketEntry>();val pending=mutableListOf<Wait>();var current:Run?=null
    fun cancel(owner:EventListener) { for(entry in ticks)if(entry.owner===owner) { entry.run?.active=false;entry.run=null } }
    private fun enter(run:Run,body:()->Unit) { val old=current;current=run;try { body() } finally { current=old } }
    fun tick() {
        val old=pending.toList();pending.clear()
        for(wait in old) {
            if(!wait.run.active || !wait.run.owner.running) { wait.run.active=false;continue }
            wait.ticks--
            if(wait.ticks>0)pending.add(wait) else enter(wait.run) { wait.continuation.resume(Unit) }
        }
        for(entry in ticks) {
            if(!entry.owner.running || entry.run?.active==true)continue
            val run=Run(entry.owner);entry.run=run
            enter(run) { entry.body.startCoroutine(object:Continuation<Unit> { override val context=EmptyCoroutineContext;override fun resumeWith(result:Result<Unit>) { run.active=false;result.getOrThrow() } }) }
        }
    }
    fun packet(event:PacketEvent) {
        for(entry in packets.sortedByDescending { it.priority })if(entry.owner.running) {
            val run=Run(entry.owner)
            enter(run) { entry.body.startCoroutine(event,object:Continuation<Unit> { override val context=EmptyCoroutineContext;override fun resumeWith(result:Result<Unit>) { result.getOrThrow() } }) }
        }
    }
}
fun EventListener.tickHandler(body:suspend ()->Unit):TickEntry = TickEntry(this,body).also { Harness.ticks.add(it) }
inline fun<reified T> EventListener.sequenceHandler(priority:Short=0,noinline body:suspend (T)->Unit):PacketEntry = PacketEntry(this,priority) { body(it as T) }.also { Harness.packets.add(it) }
suspend fun waitTicks(ticks:Int) { if(ticks==0)return;suspendCoroutine<Unit> { Harness.pending.add(Wait(Harness.current!!,it,ticks.coerceAtLeast(1))) } }
