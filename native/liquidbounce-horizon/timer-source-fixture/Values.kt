package net.ccbluex.liquidbounce.config.types.group
import kotlin.reflect.KProperty
import net.ccbluex.liquidbounce.event.EventListener
class Value<T>(var value:T) {operator fun getValue(receiver:Any?,property:KProperty<*>):T=value}
open class ValueGroup {
 val values=mutableMapOf<String,Value<*>>()
 val player get()=fixture.Minecraft.player!!
 fun float(name:String,value:Float,range:ClosedFloatingPointRange<Float>)=Value(value).also{values[name]=it}
 fun int(name:String,value:Int,range:IntRange,suffix:String)=Value(value).also{values[name]=it}
 fun boolean(name:String,value:Boolean)=Value(value).also{values[name]=it}
 fun intRange(name:String,value:IntRange,range:IntRange,suffix:String)=Value(value).also{values[name]=it}
}
open class Mode(val name:String):ValueGroup(),EventListener {
 open val parent:ModeValueGroup<Mode> get()=error("parent")
 override val running get()=parent.owner.running && parent.active===this
 open fun enable(){}
}
class ModeValueGroup<T:Mode>(val owner:EventListener,var active:T,val choices:Array<T>)
