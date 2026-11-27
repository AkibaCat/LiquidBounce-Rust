package net.ccbluex.liquidbounce.utils.movement
data class DirectionalInput(val forwards:Boolean,val backwards:Boolean,val left:Boolean,val right:Boolean){
 fun invert()=DirectionalInput(backwards,forwards,right,left)
 companion object{val NONE=DirectionalInput(false,false,false,false)}
}
