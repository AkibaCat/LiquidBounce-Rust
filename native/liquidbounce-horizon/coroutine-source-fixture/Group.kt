package net.ccbluex.liquidbounce.config.types.group
import net.ccbluex.liquidbounce.event.EventListener
import net.ccbluex.liquidbounce.event.Harness
import kotlin.reflect.KProperty
class Setting<T>(var value:T) { operator fun getValue(ref:Any?,property:KProperty<*>):T=value }
open class Group(val name:String):EventListener {
    val mc get()=net.ccbluex.liquidbounce.utils.client.mc
    val player get()=net.ccbluex.liquidbounce.utils.client.player
    val settings=linkedMapOf<String,Setting<*>>()
    val groups=linkedMapOf<String,ModeValueGroup<*>>()
    override val running:Boolean get()=true
    private fun<T> add(name:String,value:T):Setting<T> = Setting(value).also { settings[name]=it }
    fun boolean(name:String,value:Boolean)=add(name,value)
    fun float(name:String,value:Float,range:ClosedFloatingPointRange<Float>,suffix:String="")=add(name,value)
    fun int(name:String,value:Int,range:IntRange,suffix:String="")=add(name,value)
    fun intRange(name:String,value:IntRange,range:IntRange,suffix:String="")=add(name,value)
    fun<T:Enum<T>> multiEnumChoice(name:String,vararg values:T)=add(name,values.toSet())
    fun<T> tree(group:T):T=group
    fun<T:Mode> choices(name:String,default:T,modes:Array<T>):ModeValueGroup<T> = ModeValueGroup(this,name,default,modes).also { groups[name]=it }
    @Suppress("UNCHECKED_CAST") fun set(name:String,value:Any) { (settings.getValue(name) as Setting<Any>).value=value }
}
class ModeValueGroup<T:Mode>(val owner:Group,val name:String,default:T,val modes:Array<T>) {
    var activeMode:T=default
    fun setByString(tag:String) {
        val next=modes.first { it.name==tag }
        if(next===activeMode)return
        Harness.cancel(activeMode)
        activeMode=next
    }
    fun mode(tag:String):T=modes.first { it.name==tag }
}
abstract class Mode(name:String):Group(name) {
    abstract val parent:ModeValueGroup<*>
    override val running:Boolean get()=parent.owner.running && parent.activeMode===this
}
open class ToggleableValueGroup(val owner:Group,name:String,var enabled:Boolean):Group(name) {
    override val running:Boolean get()=owner.running && enabled
}
