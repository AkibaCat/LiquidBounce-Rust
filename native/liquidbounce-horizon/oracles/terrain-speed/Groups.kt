package net.ccbluex.liquidbounce.config.types.group

import fixture.FixtureState
import kotlin.reflect.KProperty
import net.ccbluex.liquidbounce.event.EventListener

class Value<T>(val name:String,var value:T,val range:Any?=null) {
 operator fun getValue(receiver:Any?,property:KProperty<*>):T=value
 operator fun setValue(receiver:Any?,property:KProperty<*>,v:T){value=v}
}
open class ValueGroup(val name:String):EventListener {
 val children=mutableListOf<Any>()
 val player get()=FixtureState.player
 val mc get()=FixtureState.client
 override val running:Boolean get()=true
 fun boolean(name:String,value:Boolean)=Value(name,value).also{children.add(it)}
 fun float(name:String,value:Float,range:ClosedFloatingPointRange<Float>)=Value(name,value,range).also{children.add(it)}
 fun <T:ValueGroup> tree(group:T):T=group.also{children.add(it)}
 protected fun <T:Mode> modes(owner:EventListener,name:String,active:T,modes:Array<T>)=
  ModeValueGroup(owner,name,modes,active).also{children.add(it)}
}
open class ToggleableValueGroup(val owner:EventListener,name:String,default:Boolean):ValueGroup(name) {
 val enabledValue=boolean("Enabled",default)
 var enabled by enabledValue
 override val running:Boolean get()=owner.running&&enabled
}
open class Mode(name:String):ValueGroup(name) {
 open val parent:ModeValueGroup<*> get()=error("no parent")
 override val running:Boolean get()=parent.running&&parent.activeMode===this
}
class ModeValueGroup<T:Mode>(val owner:EventListener,name:String,val modes:Array<T>,var activeMode:T):ValueGroup(name) {
 override val running:Boolean get()=owner.running
}
