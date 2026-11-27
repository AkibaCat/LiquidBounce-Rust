package com.mojang.blaze3d.platform
object InputConstants {
    const val KEY_LSHIFT=225
    const val KEY_RSHIFT=229
    const val MOUSE_BUTTON_LEFT=1
    var left=false
    var right=false
    fun isKeyDown(key:Int)=if(key==KEY_LSHIFT)left else right
}
