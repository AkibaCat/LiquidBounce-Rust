package net.ccbluex.liquidbounce.features.module
enum class ModuleCategories { PLAYER }
open class ClientModule(name:String,category:ModuleCategories) {
    protected val mc get()=fixture.Context.mc
    protected val interaction get()=fixture.Context.interaction
}
