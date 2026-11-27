package net.ccbluex.liquidbounce.config.types.group
import kotlin.reflect.KProperty
class Setting<T>(var value:T) { operator fun getValue(ref:Any?,property:KProperty<*>):T=value }
open class Group(val name:String) {
    val mc get()=fixture.Context.mc
    val settings=linkedMapOf<String,Setting<*>>()
    val groups=linkedMapOf<String,Group>()
    val modeGroups=linkedMapOf<String,ModeValueGroup<*>>()
    open val running:Boolean get()=true
    private fun<T> add(name:String,value:T):Setting<T> = Setting(value).also { settings[name]=it }
    fun boolean(name:String,value:Boolean)=add(name,value)
    fun float(name:String,value:Float,range:ClosedFloatingPointRange<Float>)=add(name,value)
    fun color(name:String,value:net.ccbluex.liquidbounce.render.engine.type.Color4b)=add(name,value)
    fun<T:Enum<T>> enumChoice(name:String,value:T)=add(name,value)
    fun<T:Mode> modes(owner:Group,name:String,initial:Int,factory:()->Array<T>):ModeValueGroup<T> =
        ModeValueGroup(factory(),initial).also { modeGroups[name]=it }
    fun tree(group:Group) { groups[group.name]=group }
    @Suppress("UNCHECKED_CAST") fun set(name:String,value:Any) { (settings.getValue(name) as Setting<Any>).value=value }
}
open class ToggleableValueGroup(owner:Group,name:String,enabled:Boolean):Group(name) {
    val enabled by boolean("Enabled",enabled)
}
class ModeValueGroup<T:Mode>(val modes:Array<T>,initial:Int) { var activeMode:T=modes[initial] }
abstract class Mode(name:String):Group(name) { abstract val parent:ModeValueGroup<*> }
