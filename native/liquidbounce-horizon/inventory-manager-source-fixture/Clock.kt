@file:OptIn(kotlinx.coroutines.InternalCoroutinesApi::class)
package fixture
import kotlin.coroutines.*
import kotlinx.coroutines.*
import kotlinx.coroutines.internal.MainDispatcherFactory

/** Deterministic render executor and delay scheduler; original manager is unchanged. */
object Clock : MainCoroutineDispatcher(), Delay {
    override val immediate: MainCoroutineDispatcher get() = this
    override fun isDispatchNeeded(context: CoroutineContext) = true
    data class Task(val at:Long,val order:Long,val body:Runnable,var cancelled:Boolean=false)
    var now=0L
    // The test-only stdlib MonotonicTimeSource Methodref redirects its two
    // System.nanoTime calls here. JVM Long multiplication preserves overflow.
    @JvmStatic fun nanoTime():Long=now*1_000_000L
    private val sourceOrigin by lazy { kotlin.time.TimeSource.Monotonic.markNow() }
    fun startSourceClock() { sourceOrigin }
    fun sourceMillis():Long=sourceOrigin.elapsedNow().inWholeMilliseconds
    private var serial=0L
    private val tasks=java.util.PriorityQueue<Task>(compareBy<Task> { it.at }.thenBy { it.order })
    fun schedule(at:Long,body:Runnable):DisposableHandle {
        val task=Task(at,serial++,body);tasks.add(task)
        return DisposableHandle { task.cancelled=true }
    }
    override fun dispatch(context:CoroutineContext,block:Runnable) { schedule(now,block) }
    override fun scheduleResumeAfterDelay(timeMillis:Long,continuation:CancellableContinuation<Unit>) {
        val handle=schedule(now+timeMillis,Runnable { if(continuation.isActive)continuation.resume(Unit) })
        continuation.invokeOnCancellation { handle.dispose() }
    }
    override fun invokeOnTimeout(timeMillis:Long,block:Runnable,context:CoroutineContext)=schedule(now+timeMillis,block)
    fun flush()=advance(now)
    fun advance(until:Long) {
        check(until>=now)
        while(tasks.isNotEmpty() && tasks.peek().at<=until) {
            val next=tasks.remove();now=next.at
            if(!next.cancelled)next.body.run()
        }
        now=until
    }
}
class MainFactory : MainDispatcherFactory {
    override val loadPriority=Int.MAX_VALUE
    override fun createDispatcher(allFactories:List<MainDispatcherFactory>)=Clock
    override fun hintOnError()="InventoryManager source fixture"
}
