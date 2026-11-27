package net.ccbluex.liquidbounce.features.module
import fixture.Fixture
import net.ccbluex.liquidbounce.config.types.group.ValueGroup
enum class ModuleCategories{MOVEMENT}
open class ClientModule(name:String,val category:ModuleCategories):ValueGroup(name){override val running get()=Fixture.running}
