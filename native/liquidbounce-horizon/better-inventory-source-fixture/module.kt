package net.ccbluex.liquidbounce.features.module
import net.ccbluex.liquidbounce.config.types.group.Group
enum class ModuleCategories { RENDER }
open class ClientModule(name:String,category:ModuleCategories):Group(name) {
    var enabled=false
    override val running:Boolean get()=enabled && fixture.Context.inGame
}
