package net.ccbluex.liquidbounce.utils.client
object Context { var inGame=true }
class Key { var isDown=false }
class Options { val keySprint=Key(); val keyUp=Key() }
object mc { val options=Options(); fun execute(task:()->Unit) { task() } }
class Inventory { var selectedSlot=0 }
object player {
    var yRot=0f; var xRot=0f; var isSprinting=false; var isSwinging=false
    val inventory=Inventory()
    fun onGround()=true
}
