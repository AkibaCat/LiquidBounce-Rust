package net.minecraft.network.protocol.game
class ServerboundPlayerCommandPacket(val action:Action){enum class Action{START_SPRINTING,STOP_SPRINTING}}
class ServerboundPlayerActionPacket(val action:Action){enum class Action{START_DESTROY_BLOCK,STOP_DESTROY_BLOCK}}
open class ServerboundMovePlayerPacket(var onGround:Boolean)
