package net.ccbluex.liquidbounce.config.types
import kotlin.reflect.KProperty
class Value<T>(val name:String,var raw:T){
 private val changed=mutableListOf<(T)->Unit>()
 fun get()=raw
 fun set(v:T){if(raw==v)return;raw=v;changed.forEach{it(v)}}
 fun onChanged(listener:(T)->Unit){changed.add(listener)}
 operator fun getValue(receiver:Any?,property:KProperty<*>):T=get()
 operator fun setValue(receiver:Any?,property:KProperty<*>,v:T)=set(v)
}
