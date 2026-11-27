package net.ccbluex.liquidbounce.features.module
import kotlin.properties.ReadOnlyProperty
import net.ccbluex.liquidbounce.config.types.list.Tagged
import net.ccbluex.liquidbounce.event.events.ScreenEvent
import net.minecraft.network.chat.Component
object Fixture {
 var running=true;var mode="Matches";var print=false
 var filters=mutableListOf(Regex("^Vote$"))
 var handler:(ScreenEvent)->Unit={}
 val messages=mutableListOf<List<Component>>()
}
enum class ModuleCategories {MISC}
class BooleanValue:ReadOnlyProperty<Any,Boolean> {
 fun doNotIncludeAlways()=this
 override fun getValue(thisRef:Any,property:kotlin.reflect.KProperty<*>)=Fixture.print
}
open class ClientModule(val name:String,val category:ModuleCategories,val aliases:List<String> = emptyList()) {
 open val baseKey=""
 fun <T> enumChoice(name:String,default:T):ReadOnlyProperty<Any,T> = ReadOnlyProperty {_,_->
  @Suppress("UNCHECKED_CAST")
  (default!!::class.java.enumConstants.first { (it as Tagged).tag.equals(Fixture.mode,true) } as T)
 }
 fun <C:MutableCollection<Regex>> regexList(name:String,default:C):ReadOnlyProperty<Any,C> = ReadOnlyProperty {_,_->
  @Suppress("UNCHECKED_CAST")
  (Fixture.filters as C)
 }
 fun boolean(name:String,default:Boolean)=BooleanValue()
}
