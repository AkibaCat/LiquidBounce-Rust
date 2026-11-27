package net.ccbluex.liquidbounce.utils.client
import net.minecraft.world.entity.player.PlayerModelPart
object Context { var inGame=true }
class Options {
    val modelParts=java.util.EnumSet.allOf(PlayerModelPart::class.java)
    fun setModelPart(part:PlayerModelPart,enabled:Boolean) { if(enabled)modelParts.add(part)else modelParts.remove(part) }
    fun isModelPartEnabled(part:PlayerModelPart)=part in modelParts
    fun bits()=modelParts.fold(0) { bits,part->bits or part.mask }
    fun setBits(bits:Int) { for(part in PlayerModelPart.entries)setModelPart(part,bits and part.mask!=0) }
}
object mc { val options=Options(); fun execute(task:()->Unit) { task() } }
