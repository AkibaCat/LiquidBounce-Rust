package net.ccbluex.liquidbounce.config.types.group
import net.ccbluex.liquidbounce.config.types.*
import net.ccbluex.liquidbounce.config.types.list.*
import fixture.Fixture
open class ValueGroup(val name:String) {
 fun <T> register(name:String,value:Value<T>):Value<T>{Fixture.values[this.name+"."+name]=value;return value}
 fun boolean(name:String,default:Boolean)=register(name,Value(name,default))
 fun int(name:String,default:Int,range:IntRange,suffix:String="")=register(name,Value(name,default))
 fun float(name:String,default:Float,range:ClosedFloatingPointRange<Float>)=register(name,Value(name,default))
 fun <T> enumChoice(name:String,default:T)=register(name,Value(name,default))
 fun textList(name:String,default:MutableList<String>)=register(name,MutableListValue(name,default,ValueType.TEXT,String::class.java))
 fun <T:Mode> choices(name:String,default:T,choices:Array<T>):ModeValueGroup<T> = ModeValueGroup(name,default,choices).also{Fixture.mode=it}
 fun message(key:String,vararg args:Any?)=key+":"+args.joinToString(":")
 fun doNotIncludeAlways()=this
}
abstract class Mode(name:String):ValueGroup(name){abstract val parent:ModeValueGroup<*>}
class ModeValueGroup<T:Mode>(name:String,var activeMode:T,val choices:Array<T>):ValueGroup(name)
