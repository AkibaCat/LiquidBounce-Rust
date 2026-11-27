package net.ccbluex.liquidbounce.features.module
import net.ccbluex.liquidbounce.event.EventListener
import net.ccbluex.liquidbounce.config.types.group.*
enum class ModuleCategories {WORLD}
open class ClientModule(val name:String="Provider",category:ModuleCategories=ModuleCategories.WORLD,disableOnQuit:Boolean=false):ValueGroup(),EventListener {
 override var running=true
 fun choices(name:String,default:Mode,values:Array<Mode>)=ModeValueGroup(this,default,values)
 fun tagBy(group:ModeValueGroup<Mode>){}
 open fun onDisabled(){}
}
