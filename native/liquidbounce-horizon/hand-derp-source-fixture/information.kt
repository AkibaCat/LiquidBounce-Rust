package net.minecraft.server.level
import net.ccbluex.liquidbounce.utils.client.Arm
data class ClientInformation(val language:String,val distance:Int,val chatVisibility:Int,val chatColors:Boolean,val parts:Int,val hand:Arm,val filtering:Boolean,val listing:Boolean,val particles:Int)
