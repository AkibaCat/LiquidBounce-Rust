package net.ccbluex.liquidbounce.utils.aiming
import net.ccbluex.liquidbounce.event.EventListener
import net.ccbluex.liquidbounce.utils.aiming.data.Rotation
import net.ccbluex.liquidbounce.utils.kotlin.Priority
class RotationsValueGroup(owner:EventListener) { fun toRotationTarget(rotation:Rotation)=rotation }
object RotationManager {
    var target:Rotation?=null
    fun setRotationTarget(rotation:Rotation,priority:Priority,provider:EventListener) { target=rotation }
}
