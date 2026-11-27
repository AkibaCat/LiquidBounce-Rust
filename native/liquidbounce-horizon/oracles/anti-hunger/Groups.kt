package net.ccbluex.liquidbounce.config.types.group
import fixture.Fixture
import kotlin.reflect.KProperty
import net.ccbluex.liquidbounce.event.EventListener
class Value<T>(val name:String,var value:T){operator fun getValue(receiver:Any?,property:KProperty<*>):T=value;operator fun setValue(receiver:Any?,property:KProperty<*>,next:T){value=next}}
open class ValueGroup(val name:String):EventListener{
 val children=mutableListOf<Any>()
 val player get()=Fixture.player
 val interaction get()=Fixture.interaction
 override val running get()=true
 fun boolean(name:String,value:Boolean)=Value(name,value).also{children.add(it)}
 fun tree(group:ValueGroup){children.add(group)}
 fun message(suffix:String)="liquidbounce.module.antiHunger.messages."+suffix
}
open class ToggleableValueGroup(val parent:EventListener,name:String,var enabled:Boolean):ValueGroup(name){override val running get()=parent.running&&enabled}
