package net.ccbluex.liquidbounce.features.module
import kotlin.properties.ReadOnlyProperty
enum class ModuleCategories {MISC}
object Fixture {
    var running=true
    var delay=2..3
    val mc=MC()
}
class Mouse {var isLeftPressed=false}
class MC {val mouseHandler=Mouse()}
open class ClientModule(val name:String,val category:ModuleCategories) {
    val running get()=Fixture.running
    val mc get()=Fixture.mc
    fun <T> enumChoice(name:String,default:T):ReadOnlyProperty<Any,T> = ReadOnlyProperty{_,_->default}
    fun intRange(name:String,default:IntRange,range:IntRange,suffix:String):ReadOnlyProperty<Any,IntRange> = ReadOnlyProperty{_,_->Fixture.delay}
}
