package net.ccbluex.liquidbounce.utils.aiming
import net.ccbluex.liquidbounce.event.EventListener
import net.ccbluex.liquidbounce.utils.aiming.data.Rotation
import net.ccbluex.liquidbounce.utils.kotlin.Priority
class RotationsValueGroup(owner:EventListener) { fun toRotationTarget(rotation:Rotation)=rotation }
object RotationManager {
    var target:Rotation?=null
    val serverRotation get()=Rotation(0f,0f)
    fun setRotationTarget(rotation:Rotation,priority:Priority,provider:EventListener) { target=rotation }
    fun setRotationTarget(rotation:Rotation,ignore:Boolean,group:RotationsValueGroup,priority:Priority,provider:EventListener) { target=rotation }
}
