package net.ccbluex.liquidbounce.features.module
import net.ccbluex.liquidbounce.utils.client.mc
import net.ccbluex.liquidbounce.utils.client.player
enum class ModuleCategories { FUN }
open class ClientModule(val name:String,category:ModuleCategories) {
    val player get()=net.ccbluex.liquidbounce.utils.client.player
    val mc get()=net.ccbluex.liquidbounce.utils.client.mc
    fun<T> tree(value:T):T=value
}
