package net.ccbluex.liquidbounce.utils.movement
data class DirectionalInput(val forwards:Boolean=false,val backwards:Boolean=false,val left:Boolean=false,val right:Boolean=false) {
    companion object { val NONE=DirectionalInput() }
}
