package net.ccbluex.liquidbounce.features.module
import net.ccbluex.liquidbounce.config.types.group.Group
import net.ccbluex.liquidbounce.event.Harness
import net.ccbluex.liquidbounce.utils.client.Context
enum class ModuleCategories { FUN }
open class ClientModule(name:String,category:ModuleCategories):Group(name) {
    var enabled=false
        set(value) {
            if(field==value)return
            if(Context.inGame) {
                if(value)onEnabled()else { Harness.cancel(this);onDisabled() }
            }
            field=value
        }
    override val running:Boolean get()=enabled && Context.inGame
    open fun onEnabled() {}
    open fun onDisabled() {}
}
