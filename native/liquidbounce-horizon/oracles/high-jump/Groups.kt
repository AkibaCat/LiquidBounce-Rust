package net.ccbluex.liquidbounce.config.types.group
import kotlin.reflect.KProperty
import fixture.Fixture
import net.ccbluex.liquidbounce.event.EventListener
class Value<T>(val name:String,var value:T){operator fun getValue(receiver:Any?,property:KProperty<*>):T=value}
open class ValueGroup(val name:String):EventListener{
 val children=mutableListOf<Any>()
 val player get()=Fixture.player
 override val running get()=true
 fun boolean(name:String,value:Boolean)=Value(name,value).also{children.add(it)}
 fun float(name:String,value:Float,range:ClosedFloatingPointRange<Float>)=Value(name,value).also{children.add(it)}
 fun <T:Mode> choices(name:String,active:T,modes:Array<T>)=ModeValueGroup(this,name,modes,active).also{children.add(it)}
 fun tagBy(value:Any){}
}
open class Mode(name:String,val aliases:List<String> = emptyList()):ValueGroup(name){
 open val parent:ModeValueGroup<*> get()=error("no parent")
 override val running get()=parent.running&&parent.activeMode===this
}
class ModeValueGroup<T:Mode>(val owner:EventListener,name:String,val modes:Array<T>,var activeMode:T):ValueGroup(name){override val running get()=owner.running}
