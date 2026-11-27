package net.ccbluex.liquidbounce.features.module
import fixture.Fixture
import net.ccbluex.liquidbounce.event.EventListener
import net.ccbluex.liquidbounce.event.removeEventListenerScope
import kotlin.reflect.KProperty
enum class ModuleCategories{EXPLOIT}
class Value<T>(val name:String,var value:T){operator fun getValue(receiver:Any?,property:KProperty<*>):T=value}
open class ClientModule(val name:String,val category:ModuleCategories):EventListener {
    val children=mutableListOf<Any>()
    var enabled=false
        set(value){
            if(field==value)return
            field=value
            Fixture.effect("enabled","$name:$value")
            if(!value)removeEventListenerScope()
        }
    override val running get()=enabled&&Fixture.inGame
    val player get()=Fixture.player
    fun int(name:String,value:Int,range:IntRange)=Value(name,value).also{children.add(it)}
    fun float(name:String,value:Float,range:ClosedFloatingPointRange<Float>)=Value(name,value).also{children.add(it)}
}
