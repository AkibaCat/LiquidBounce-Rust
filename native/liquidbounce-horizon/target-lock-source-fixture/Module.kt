package net.ccbluex.liquidbounce.features.module
import net.ccbluex.liquidbounce.config.types.group.*
import fixture.Fixture
enum class ModuleCategories{MISC}
open class ClientModule(name:String,category:ModuleCategories):ValueGroup(name){val player get()=Fixture.local;val world get()=Fixture.world;val running get()=Fixture.running}
