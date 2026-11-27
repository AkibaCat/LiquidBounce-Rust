package net.ccbluex.liquidbounce.features.module
import net.ccbluex.liquidbounce.config.types.group.Group
import net.ccbluex.liquidbounce.utils.client.Context
enum class ModuleCategories { FUN,PLAYER }
open class ClientModule(name:String,category:ModuleCategories):Group(name) {
    var enabled=true
    override val running:Boolean get()=enabled && Context.inGame
}
