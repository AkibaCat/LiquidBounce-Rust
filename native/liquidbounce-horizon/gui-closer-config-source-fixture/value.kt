package net.ccbluex.liquidbounce.config.types
import com.google.gson.*
import com.google.gson.annotations.SerializedName
import net.ccbluex.liquidbounce.config.gson.stategies.*
import java.util.function.BooleanSupplier
enum class ValueType { INVALID,LIST,MUTABLE_LIST,NAMED_ITEM_LIST,REGISTRY_LIST,REGISTRY_MUTABLE_LIST,TEXT,TOGGLEABLE,BLOCK,ITEM,SOUND_EVENT,MOB_EFFECT,C2S_PACKET,S2C_PACKET,ENTITY_TYPE,ENCHANTMENT,MENU,CLIENT_MODULE;
    val deserializer: Any? get()=null
}
open class Value<T>(val name:String, defaultValue:T, @field:Exclude val valueType:ValueType) {
    @SerializedName("value") var inner:T=defaultValue
    @field:Exclude @field:ProtocolExclude var notAnOption=false
    @field:Exclude @field:ProtocolExclude var isPersistent=true
    @field:Exclude @field:ProtocolExclude var visibleCondition=BooleanSupplier{true}
    @field:Exclude @field:ProtocolExclude var privateValue=false
    fun checkIfInclude()=!privateValue
    open fun setByString(string:String) { error("not used") }
    open fun deserializeFrom(gson:Gson,element:JsonElement) { error("not used") }
    fun set(value:T,callback:()->Unit={}) { inner=value;callback() }
}
