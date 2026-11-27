package net.ccbluex.liquidbounce.additions
import net.minecraft.client.gui.GuiGraphicsExtractor
fun GuiGraphicsExtractor.drawBorder(x:Int,y:Int,width:Int,height:Int,color:Int) {
    fill(x,y,x+width,y+1,color);fill(x,y+height-1,x+width,y+height,color)
    fill(x,y+1,x+1,y+height-1,color);fill(x+width-1,y+1,x+width,y+height-1,color)
}
