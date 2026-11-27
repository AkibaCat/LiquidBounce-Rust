package net.minecraft.network.protocol.game
class ServerboundPunchPacket
class Data(val id:Int)
class ClientboundSetEntityDataPacket(val id:Int,val packedItems:List<Data>)
