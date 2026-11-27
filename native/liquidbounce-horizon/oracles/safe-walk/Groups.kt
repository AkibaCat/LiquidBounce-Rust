package net.ccbluex.liquidbounce.config.types.group
import fixture.Fixture
import net.ccbluex.liquidbounce.config.types.Value
import net.ccbluex.liquidbounce.config.types.list.Tagged
import net.ccbluex.liquidbounce.event.EventListener
open class ValueGroup(val name:String):EventListener{
 val children=mutableListOf<Any>();val player get()=Fixture.player
 override val running get()=Fixture.running
 fun boolean(name:String,value:Boolean)=Value(name,value).also{children.add(it)}
 fun floatRange(name:String,value:ClosedFloatingPointRange<Float>,range:ClosedFloatingPointRange<Float>)=Value(name,value).also{children.add(it)}
 fun intRange(name:String,value:IntRange,range:IntRange,suffix:String="")=Value(name,value).also{children.add(it)}
 inline fun <reified T> enumChoice(name:String,value:T)where T:Enum<T>,T:Tagged=Value(name,value).also{children.add(it)}
 fun choices(name:String,index:Int,factory:(ModeValueGroup<Mode>)->Array<Mode>)=ModeValueGroup(name,this,index,factory).also{children.add(it)}
}
class ModeValueGroup<T:Mode>(name:String,val owner:EventListener,index:Int,factory:(ModeValueGroup<T>)->Array<T>):ValueGroup(name){
 val modes:Array<T> = factory(this);var activeMode:T=modes[index]
 override val running get()=owner.running
 fun activate(next:T){if(next===activeMode)return;if(activeMode.running)activeMode.disable();activeMode=next}
}
abstract class Mode(name:String):ValueGroup(name){abstract val parent:ModeValueGroup<*>;override val running get()=parent.running&&parent.activeMode===this;open fun disable(){}}
class NoneMode(override val parent:ModeValueGroup<*>):Mode("None")
