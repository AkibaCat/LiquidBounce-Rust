package net.ccbluex.liquidbounce.utils.inventory
import kotlinx.coroutines.CancellableContinuation
import kotlinx.coroutines.suspendCancellableCoroutine
import net.ccbluex.liquidbounce.features.module.ClientModule
data class Stack(val bits:Int) {
    val isEmpty get()=bits and 1 != 0
    fun nextDamageWillBreak()=bits and 4 != 0
}
class PlayerInventoryConstraints
object Harness {
    var body:(suspend Plan.()->Unit)?=null
    var stacks=Array(41) { Stack(1) }
    val reads=mutableListOf<Int>()
    val actions=mutableListOf<Int>()
    var waiting:CancellableContinuation<Unit>?=null
}
data class Slot(val index:Int) {
    val itemStack get():Stack { Harness.reads.add(index); return Harness.stacks[index] }
}
object ArmorItemSlot { val CHEST=Slot(38) }
object HotbarItemSlot { val OFFHAND=Slot(40) }
class Slots(private val slots:List<Slot>) {
    fun findSlot(predicate:(Stack)->Boolean)=slots.find { predicate(it.itemStack) }
    operator fun plus(other:Slots)=Slots(slots+other.slots)
    operator fun plus(other:Slot)=Slots(slots+other)
    companion object {
        val Hotbar=Slots((0..8).map { Slot(it) })
        val Inventory=Slots((9..35).map { Slot(it) })
    }
}
class Plan {
    suspend fun perform(actions:List<Int>) {
        Harness.actions.addAll(actions)
        suspendCancellableCoroutine<Unit> { Harness.waiting=it }
        Harness.waiting=null
    }
}
fun ClientModule.inventoryPlan(constraints:PlayerInventoryConstraints, body:suspend Plan.()->Unit):Int {
    Harness.body=body
    return 1
}
object InventoryAction { object Click { fun performPickup(slot:Slot)=slot.index } }
