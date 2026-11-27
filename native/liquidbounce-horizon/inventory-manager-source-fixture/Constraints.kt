package net.ccbluex.liquidbounce.utils.inventory
import net.ccbluex.liquidbounce.utils.client.*
open class InventoryConstraints {
    var startDelay=50..100
    var clickDelay=100..200
    var closeDelay=50..100
    var missChance=0..0
    var slotMultiplier=0.2f
    var mask=0
    fun travelMillis(delay:IntRange,slots:Float)=(delay.random()*slots*slotMultiplier).toInt()
    fun passesRequirements(action:InventoryAction)=InventoryRequirements.entries.withIndex().all { (i,r) -> mask and (1 shl i)==0 || r.test(action) }
}
class PlayerInventoryConstraints:InventoryConstraints()
enum class InventoryRequirements {
    NO_MOVEMENT,NO_ROTATION,NOT_USING_ITEM,NOT_BREAKING,NOT_DURING_COMBAT,OPEN_INVENTORY;
    fun test(action:InventoryAction)=when(this) {
        NO_MOVEMENT -> kotlin.math.abs(player.movement.lengthSquared())<1e-5f && !player.jumping
        NO_ROTATION -> player.rotationMatches
        NOT_USING_ITEM -> !player.usingItem
        NOT_BREAKING -> !player.breaking
        NOT_DURING_COMBAT -> !player.combat
        OPEN_INVENTORY -> !action.requiresPlayerInventoryOpen() || InventoryManager.isInventoryOpen
    }
}
