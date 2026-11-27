package net.ccbluex.liquidbounce.config.types.group
import net.ccbluex.liquidbounce.event.EventListener
import net.ccbluex.liquidbounce.config.types.list.Tagged
import kotlin.reflect.KProperty
class Setting<T>(var value:T) { operator fun getValue(ref:Any?,property:KProperty<*>):T=value }
open class Group(val name:String):EventListener {
    val mc get()=net.ccbluex.liquidbounce.utils.client.mc
    val settings=linkedMapOf<String,Setting<*>>()
    val choices=linkedMapOf<String,List<*>>()
    override val running:Boolean get()=true
    private fun<T> add(name:String,value:T):Setting<T> = Setting(value).also { settings[name]=it }
    fun boolean(name:String,value:Boolean)=add(name,value)
    fun int(name:String,value:Int,range:IntRange,suffix:String="")=add(name,value)
    fun<T:Enum<T>> multiEnumChoice(name:String,values:Iterable<T>):Setting<MutableSet<T>> {
        val list=values.toList();choices[name]=list
        return add(name,java.util.EnumSet.copyOf(list) as MutableSet<T>)
    }
    @Suppress("UNCHECKED_CAST") fun set(name:String,value:Any) { (settings.getValue(name) as Setting<Any>).value=value }
    @Suppress("UNCHECKED_CAST") fun setParts(tags:List<String>) {
        val selected=(settings.getValue("Parts") as Setting<MutableSet<Any>>).value
        selected.clear()
        for(tag in tags)choices.getValue("Parts").firstOrNull { (it as Tagged).tag.equals(tag,true) }?.let { selected.add(it) }
    }
}
