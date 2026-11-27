package net.ccbluex.liquidbounce.render.gui
import net.minecraft.client.gui.GuiGraphicsExtractor
import net.ccbluex.liquidbounce.render.engine.type.BoundingBox2f
import net.ccbluex.liquidbounce.render.engine.type.Color4b
import fixture.Context
interface GuiRearrangeable {var bounds:BoundingBox2f}
sealed interface ItemStackListBackground {val padding:Float
    data object Slots:ItemStackListBackground {override val padding=0f}
}
object GuiPanel {val DEFAULT:ItemStackListBackground=ItemStackListBackground.Slots}
object ItemStackListRenderer {
    class SingleItemStackRenderer {companion object {val All=SingleItemStackRenderer()}}
    class BackgroundMode(val background:ItemStackListBackground)
    fun draw(graphics:GuiGraphicsExtractor,state:ItemStackListRenderState,rearrange:Boolean) {
        val dimensions=ItemStackListLayout.measureContent(state)
        Context.ops+=mapOf("op" to "list","indices" to state.stacks.map{it.index},"centerBits" to listOf(state.centerX.toRawBits(),state.centerY.toRawBits()),"scaleBits" to state.scale.toRawBits(),"dimensions" to listOf(dimensions.width,dimensions.height))
    }
}
fun ItemStackListBackground.withOutline(color:Color4b)=this
