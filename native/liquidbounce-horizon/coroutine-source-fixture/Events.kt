package net.ccbluex.liquidbounce.event.events
import net.ccbluex.liquidbounce.utils.movement.DirectionalInput
class GameTickEvent
class MovementInputEvent(var directionalInput:DirectionalInput=DirectionalInput.NONE,var jump:Boolean=false)
