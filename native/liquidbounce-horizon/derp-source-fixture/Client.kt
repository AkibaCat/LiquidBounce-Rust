package net.ccbluex.liquidbounce.utils.client
object Context { var inGame=true }
class Key { var isDown=false }
class Options { val keySprint=Key() }
object mc { val options=Options() }
object player { var yRot=0f; var xRot=0f; var isSprinting=false }
