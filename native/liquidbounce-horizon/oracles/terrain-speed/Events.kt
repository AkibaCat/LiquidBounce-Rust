package net.ccbluex.liquidbounce.event.events
import net.minecraft.world.entity.MoverType
import net.minecraft.world.phys.Vec3
import net.minecraft.world.level.block.Block
class PlayerMoveEvent(val type:MoverType,var movement:Vec3)
class MovementInputEvent(var jump:Boolean)
class PlayerTickEvent
class BlockSlipperinessMultiplierEvent(val block:Block,var slipperiness:Float)
