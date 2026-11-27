package net.ccbluex.liquidbounce.features.module
enum class ModuleCategories { PLAYER }
open class ClientModule(val name:String, val category:ModuleCategories, val aliases:List<String>, val disableOnQuit:Boolean) {
    var enabled=false
        set(value) { val old=field; field=value; if(value && !old) onEnabled() }
    open fun onEnabled() {}
    fun<T> tree(value:T):T=value
}
