package net.ccbluex.liquidbounce.features.module
import net.ccbluex.liquidbounce.config.types.group.Group
import net.ccbluex.liquidbounce.event.Harness
import net.ccbluex.liquidbounce.utils.client.Context
enum class ModuleCategories { FUN }
open class ClientModule(name:String,category:ModuleCategories):Group(name) {
    var enabled=false
        set(value) {
            if(field && !value && Context.inGame) {
                Harness.cancel(this)
                for(group in groups.values)for(mode in group.modes)Harness.cancel(mode)
                onDisabled()
            }
            field=value
        }
    override val running:Boolean get()=enabled && Context.inGame
    open fun onDisabled() {}
}
