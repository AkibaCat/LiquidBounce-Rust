package net.ccbluex.liquidbounce.features.module
import net.ccbluex.liquidbounce.config.types.group.ValueGroup
enum class ModuleCategories{MOVEMENT}
open class ClientModule(name:String,val category:ModuleCategories):ValueGroup(name)
