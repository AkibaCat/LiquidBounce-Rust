package net.ccbluex.liquidbounce.config.types
import com.google.gson.*
import kotlin.reflect.KProperty
enum class ValueType {INVALID,LIST,MUTABLE_LIST,NAMED_ITEM_LIST,REGISTRY_LIST,REGISTRY_MUTABLE_LIST,TEXT,TOGGLEABLE,BLOCK,ITEM,SOUND_EVENT,MOB_EFFECT,C2S_PACKET,S2C_PACKET,ENTITY_TYPE,ENCHANTMENT,MENU,CLIENT_MODULE;val deserializer:Any? get()=null}
open class Value<T>(val name:String,defaultValue:T,val valueType:ValueType=ValueType.INVALID) {
 var inner:T=defaultValue
 operator fun getValue(owner:Any?,property:KProperty<*>):T=inner
 open fun setByString(string:String){error("unused")}
 open fun deserializeFrom(gson:Gson,element:JsonElement){error("unused")}
 fun set(value:T,callback:()->Unit={}){inner=value;callback()}
}
