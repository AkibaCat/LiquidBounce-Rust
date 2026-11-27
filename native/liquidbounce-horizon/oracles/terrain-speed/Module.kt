package net.ccbluex.liquidbounce.features.module
import fixture.FixtureState
import net.ccbluex.liquidbounce.config.types.group.ValueGroup
import net.minecraft.client.player.ClientInput
import net.minecraft.world.entity.player.Input
import net.minecraft.world.phys.Vec2
enum class ModuleCategories {MOVEMENT}
open class ClientModule(name:String,val category:ModuleCategories,val aliases:List<String> = emptyList()):ValueGroup(name) {
 override val running:Boolean get()=FixtureState.running
}
class FixtureInput:ClientInput() {
 var original=Input.EMPTY
 fun vector(x:Float,y:Float){moveVector=Vec2(x,y)}
}
