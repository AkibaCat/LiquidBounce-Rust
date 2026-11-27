package kotlinx.atomicfu
/** Explicit atomicfu API provider; delegates to JDK's actual AtomicReference. */
class AtomicRef<T>(initial:T) {
    private val ref = java.util.concurrent.atomic.AtomicReference<T>(initial)
    var value:T get()=ref.get(); set(value){ref.set(value)}
    fun compareAndSet(expect:T,update:T)=ref.compareAndSet(expect,update)
    fun getAndSet(update:T):T=ref.getAndSet(update)
}
fun <T> atomic(initial:T)=AtomicRef(initial)
