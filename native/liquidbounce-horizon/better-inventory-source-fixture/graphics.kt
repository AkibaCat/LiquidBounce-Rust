package net.minecraft.client.gui
import fixture.Context
class Font {val lineHeight=9;fun width(text:net.minecraft.network.chat.Component)=0}
class Pose {
    fun pushMatrix(){Context.ops+=mapOf("op" to "push")}
    fun popMatrix(){Context.ops+=mapOf("op" to "pop")}
    fun scale(scale:Float){Context.ops+=mapOf("op" to "scale","bits" to scale.toRawBits())}
}
class GuiGraphicsExtractor {
    fun pose()=Pose()
    fun fill(x0:Int,y0:Int,x1:Int,y1:Int,color:Int){Context.ops+=mapOf("op" to "fill","rect" to listOf(x0,y0,x1,y1),"color" to color)}
    fun blitSprite(pipeline:Any,texture:Any,x:Int,y:Int,w:Int,h:Int){Context.ops+=mapOf("op" to "sprite","rect" to listOf(x,y,w,h))}
    fun centeredText(font:Font,text:String,x:Int,y:Int,color:Int){Context.ops+=mapOf("op" to "text","text" to text,"x" to x,"y" to y,"color" to color)}
}
