package net.ccbluex.liquidbounce.event.events
import net.ccbluex.liquidbounce.utils.movement.DirectionalInput
interface Event
class MovementInputEvent(var directionalInput:DirectionalInput,var jump:Boolean,var sneak:Boolean):Event
class PlayerSafeWalkEvent(var isSafeWalk:Boolean):Event
