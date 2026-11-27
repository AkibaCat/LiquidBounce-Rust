package net.minecraft.util
object Mth{const val RAD_TO_DEG:Float=57.295776f
 @JvmStatic fun wrapDegrees(value:Float):Float{var v=value%360f;if(v>=180f)v-=360f;if(v< -180f)v+=360f;return v}
}
