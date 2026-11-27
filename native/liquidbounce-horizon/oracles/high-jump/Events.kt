package net.ccbluex.liquidbounce.event.events
interface Event
class GameTickEvent:Event
class PlayerJumpEvent(var motion:Float):Event
