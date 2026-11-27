package net.ccbluex.liquidbounce.render.engine.type
data class Color4b(val argb:Int) { companion object {val LIQUID_BOUNCE=Color4b(0xff4677ff.toInt());val WHITE=Color4b(-1)} }
data class BoundingBox2f(val x0:Float,val y0:Float,val x1:Float,val y1:Float) {companion object {val EMPTY=BoundingBox2f(0f,0f,0f,0f)}}
