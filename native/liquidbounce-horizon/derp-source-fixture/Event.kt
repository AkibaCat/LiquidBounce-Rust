package net.ccbluex.liquidbounce.event
import kotlin.coroutines.*
import net.ccbluex.liquidbounce.event.events.GameTickEvent
interface EventListener { val running:Boolean }
class Run(val owner:EventListener) { var active=true }
class Entry(val owner:EventListener,val sequence:(suspend ()->Unit)?,val ordinary:(()->Unit)?) { var run:Run?=null }
object Harness {
    val entries=mutableListOf<Entry>()
    val pending=mutableListOf<Pair<Run,Continuation<Unit>>>()
    var current:Run?=null
    var draws=0
    fun cancel(owner:EventListener) {
        for(entry in entries)if(entry.owner===owner) { entry.run?.active=false; entry.run=null }
    }
    private fun enter(run:Run,body:()->Unit) {
        val old=current; current=run
        try { body() } finally { current=old }
    }
    fun tick() {
        val old=pending.toList(); pending.clear()
        for((run,continuation) in old) {
            if(!run.active || !run.owner.running) { run.active=false; continue }
            enter(run) { continuation.resume(Unit) }
        }
        for(entry in entries) {
            if(!entry.owner.running)continue
            if(entry.sequence==null) { entry.ordinary!!(); continue }
            if(entry.run?.active==true)continue
            val run=Run(entry.owner); entry.run=run
            enter(run) {
                entry.sequence.startCoroutine(object:Continuation<Unit> {
                    override val context=EmptyCoroutineContext
                    override fun resumeWith(result:Result<Unit>) { run.active=false; result.getOrThrow() }
                })
            }
        }
    }
}
inline fun<reified T> EventListener.handler(noinline body:(T)->Unit):Entry = Entry(this,null,{body(GameTickEvent() as T)}).also { Harness.entries.add(it) }
fun EventListener.tickHandler(body:suspend ()->Unit):Entry = Entry(this,body,null).also { Harness.entries.add(it) }
suspend fun waitTicks(ticks:Int) { if(ticks==0)return; check(ticks==1); suspendCoroutine<Unit> { Harness.pending.add(Harness.current!! to it) } }
