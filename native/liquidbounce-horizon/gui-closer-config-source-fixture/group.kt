package net.ccbluex.liquidbounce.config.types.group
import net.ccbluex.liquidbounce.config.types.*
open class ValueGroup(name:String,values:MutableList<Value<*>>):Value<MutableList<Value<*>>>(name,values,ValueType.TOGGLEABLE)
class ModeValueGroup<T>(name:String):ValueGroup(name,mutableListOf()) {
    val activeMode:ValueGroup get()=error("not used")
}
